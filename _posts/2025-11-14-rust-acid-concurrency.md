---
layout: post
title: "Atomicity and Isolation in Concurrent Rust: Channels vs. Mutexes"
date: 2025-11-14 10:00:48 +0530
categories: rust concepts
last_updated: 2026-09-19
---

# Achieving Atomicity and Isolation in Concurrent Rust: Channels vs. Mutexes

You have already learned how ownership prevents unsafe access to memory. Now comes a different question: **can a program use memory safely and still move money incorrectly?** Yes. It can debit one account, fail to credit another, and never violate a borrowing rule.

This follows the [concurrency post](/rust/concepts/2025/11/12/rust-concurreny-vs-go.html). Keep the first post's lessons about scopes and guards in mind. Here we apply them to an entire operation, including its failure paths.

**Reviewed on 19 September 2026 against Rust 1.98.1, edition 2024.** The examples use the standard library. These are small in-memory ledgers for learning, not a complete payment system. Amounts are integer minor units, such as cents or paise; all accounts here use the same currency.

## The Fork in the Road

Imagine Alice has 1,000 units and Bob has 500. Alice transfers 200 to Bob. A successful transfer should leave 800 and 700. A rejected transfer should leave both balances unchanged.

Two useful designs are:

1. **One owner receives commands.** Other threads send requests to the owner of the ledger.
2. **Callers share a protected ledger.** Each caller takes a mutex before examining or changing it.

These are common choices, not Rust's only choices. Scoped threads can borrow disjoint pieces of data; atomics support particular shared operations; read/write locks allow multiple readers. You can also send an `Arc<Mutex<T>>` through a channel. A channel does not require every message to contain independently owned application data. [Rust's synchronization tools](https://doc.rust-lang.org/std/sync/index.html).

Both designs still rely on Rust's type system. With a mutex, runtime locking and compile-time restrictions cooperate: a guard gives controlled access to the protected value. Shared ownership through `Arc` alone does not make arbitrary contents safe to mutate. [Arc](https://doc.rust-lang.org/std/sync/struct.Arc.html).

### First, separate the promises

The word *atomic* causes trouble because people use it for different promises:

| Promise | What it means for our transfer |
| --- | --- |
| No data race | Threads do not access the same memory without the required synchronization when at least one access writes and at least one is non-atomic. |
| Indivisible observation | A reader following our protocol cannot observe the half-finished transfer. |
| Failure atomicity | A rejected operation leaves neither a debit nor a credit behind. |
| Isolation | Concurrent operations do not interfere in ways our consistency rules forbid. |
| Durability | A confirmed change survives the failures the storage system promises to tolerate. |

ACID stands for atomicity, consistency, isolation and durability. Consistency includes application rules: valid balances, matching debits and credits, and no unauthorized transfer. A mutex cannot invent those rules. A database transaction supplies useful guarantees, but the application still has to ask for the right changes and isolation level. [PostgreSQL transactions](https://www.postgresql.org/docs/current/tutorial-transactions.html).

Safe Rust rules out data races, assuming the `unsafe` code underneath its safe APIs is sound. It does not rule out deadlocks, lost business updates, or other timing-dependent mistakes. Rust's own documentation distinguishes these from data races. [Data races and race conditions](https://doc.rust-lang.org/nomicon/races.html).

## Channels: The "Don't Touch My Stuff" Approach

Think of a channel as a mailbox. An *actor* is the owner that reads commands from that mailbox and operates on its private state. The channel transports messages; the actor's command handler defines what a message means.

One actor per account sounds natural. Alice's actor can debit Alice and send Bob's actor a deposit command. But consider this deliberately incomplete, flawed handler fragment:

```rust
// Flawed design sketch, not a standalone program.
if self.balance >= amount {
    self.balance -= amount;
    let _ = destination.send(Deposit(amount));
}
```

The memory accesses can be perfectly safe. The money rule is still broken:

1. Alice's actor subtracts 200.
2. Bob's receiver might already be gone; the send fails and this code discards the error.
3. Even if sending succeeds, Bob has not necessarily processed the deposit yet.

`Sender::send` returning `Ok(())` is not a processing acknowledgment. The receiver can disappear before receiving the message. [Standard channel sender](https://doc.rust-lang.org/std/sync/mpsc/struct.Sender.html#method.send).

A reader can also see Alice after the debit and Bob before the credit. Carefully ordering a particular pair of queries can hide that window in a happy-path run; it does not repair the protocol for all callers.

If the whole process crashes, neither account's in-memory balance is durable, regardless of whether we used channels or mutexes. Whether money is lost after restart depends on what was persisted and how recovery works. We cannot deduce that from an in-memory example.

### Put the whole transfer under one owner

There is a straightforward channel design: **one ledger actor owns both balances and handles `Transfer` as one command**. Network handling and unrelated work can still run concurrently. Only this ledger's commands are serialized, just as a mutex serializes its critical sections.

First, define the operation independently of how threads reach it. Copy these shared definitions above either of the next two examples:

```rust
#[derive(Debug)]
enum TransferError {
    InsufficientFunds,
    BalanceOverflow,
}

fn move_funds(
    source: &mut u64,
    destination: &mut u64,
    amount: u64,
) -> Result<(), TransferError> {
    let new_source = source
        .checked_sub(amount)
        .ok_or(TransferError::InsufficientFunds)?;
    let new_destination = destination
        .checked_add(amount)
        .ok_or(TransferError::BalanceOverflow)?;

    // All rejection paths are above. These assignments cannot overflow.
    *source = new_source;
    *destination = new_destination;
    Ok(())
}
```

The order matters. We calculate **both** new balances before changing either old balance. If Bob's balance cannot fit in `u64`, Alice keeps her money. `checked_add` and `checked_sub` make failure explicit in both debug and release builds. [Checked integer arithmetic](https://doc.rust-lang.org/std/primitive.u64.html#method.checked_add).

Here is the actor example. Add the shared definitions above it to make one complete program:

```rust
use std::{sync::mpsc, thread};

enum Command {
    Transfer {
        amount: u64,
        reply: mpsc::SyncSender<Result<(), TransferError>>,
    },
    Snapshot(mpsc::SyncSender<(u64, u64)>),
}

fn main() {
    let (commands, inbox) = mpsc::sync_channel::<Command>(32);
    let worker = thread::spawn(move || {
        let (mut alice, mut bob) = (1_000, 500);
        for command in inbox {
            match command {
                Command::Transfer { amount, reply } => {
                    let result = move_funds(&mut alice, &mut bob, amount);
                    // A disappeared caller does not undo the operation.
                    let _ = reply.send(result);
                }
                Command::Snapshot(reply) => {
                    let _ = reply.send((alice, bob));
                }
            }
        }
    });

    for amount in [200, 2_000] {
        // One reply fits without making the worker wait for our receive.
        let (reply, result) = mpsc::sync_channel(1);
        commands.send(Command::Transfer { amount, reply }).unwrap();
        println!("Transfer {amount}: {:?}", result.recv().unwrap());
    }

    let (reply, snapshot) = mpsc::sync_channel(1);
    commands.send(Command::Snapshot(reply)).unwrap();
    println!("Balances: {:?}", snapshot.recv().unwrap());

    drop(commands); // Close the inbox after queued commands are drained.
    worker.join().expect("ledger worker panicked");
}
```

In a normal run this prints:

```text
Transfer 200: Ok(())
Transfer 2000: Err(InsufficientFunds)
Balances: (800, 700)
```

One handler finishes before the next begins. `Snapshot` reads both balances within that same boundary. The helper leaves balances unchanged on its two explicit rejection paths. Together these choices provide the in-memory behavior we asked for; the channel alone did not supply it.

The queue holds at most 32 commands. A full `sync_channel` makes `send` block the calling OS thread; capacity bounds buffered messages, not the number of blocked producers or their memory. The ordinary `channel()` instead has an unbounded buffer. [Bounded channel](https://doc.rust-lang.org/std/sync/mpsc/fn.sync_channel.html), [unbounded channel](https://doc.rust-lang.org/std/sync/mpsc/fn.channel.html).

Messages enter a receiving order, but competing producers do not acquire a meaningful business order merely by sharing a mailbox. Add sequence rules if your application needs them. Here the reply explicitly tells the caller that its handler finished. Losing that reply leaves the caller uncertain: retrying blindly could repeat a transfer. A real payment API needs a recorded request identity and a duplicate-handling policy.

An actor can also deadlock. If actor A waits for a reply from B while B waits for A, neither can progress. Bounded sends can create similar cycles. Changing locks into messages does not remove the need to reason about waiting.

Keeping separate account owners is also possible, but cross-owner transfers then need a coordination protocol. Channels can carry that protocol; they do not create transaction semantics by themselves.

## Mutexes: The "Take a Number" System

A mutex lets callers take turns using one protected value. For this example, put **both balances in one value**. Add the same `TransferError` and `move_funds` definitions above this program:

```rust
use std::{sync::{Arc, Mutex}, thread};

fn main() {
    let ledger = Arc::new(Mutex::new((1_000_u64, 500_u64)));
    let worker_ledger = Arc::clone(&ledger);

    let worker = thread::spawn(move || {
        for amount in [200, 2_000] {
            let result = {
                let mut balances = worker_ledger.lock().expect("ledger poisoned");
                let (alice, bob) = &mut *balances;
                move_funds(alice, bob, amount)
            }; // Release before printing or doing unrelated work.
            println!("Transfer {amount}: {result:?}");
        }
    });

    worker.join().expect("ledger worker panicked");
    let snapshot = {
        let balances = ledger.lock().expect("ledger poisoned");
        *balances // Copy both numbers while holding one guard.
    };
    println!("Balances: {snapshot:?}");
}
```

This has the same output as the actor example. We have changed how callers reach the operation, while keeping its money rule unchanged.

`Arc` shares ownership of the mutex allocation. `Mutex` controls access; its guard unlocks on drop. Locking also supplies the synchronization needed to observe earlier protected changes. A critical section may contain many CPU instructions; it is not one machine-level atomic instruction. [Standard synchronization](https://doc.rust-lang.org/std/sync/index.html).

### A lock does not roll changes back

Suppose a larger handler changes Alice, then a later step returns an error or panics before changing Bob. Releasing the lock does not restore Alice. Our helper avoids this particular failure by validating first and leaving only two plain assignments after validation. That is a property of **this operation**, not automatic transactional behavior from `Mutex`.

During panic unwinding, a held standard mutex normally becomes poisoned. Poisoning warns that protected data may need inspection. It is advisory, can be missed in some circumstances, and is neither rollback nor the foundation of memory safety. This teaching program stops on poison with `expect`; production code needs an explicit stop-or-repair policy. Calling `into_inner()` merely obtains the guard; it does not repair the data. [Mutex poisoning](https://doc.rust-lang.org/std/sync/struct.Mutex.html#poisoning).

Guard cleanup applies to normal scope exit and unwinding. Process abort and deliberate leaks do not promise to run destructors. Persistence is a separate problem. [Drop scopes and termination](https://doc.rust-lang.org/reference/destructors.html).

### The Deadly Embrace: More Than One Lock

Separate account locks can allow unrelated account pairs to be updated at the same time. They also introduce a waiting problem:

| Thread A: Alice → Bob | Thread B: Bob → Alice |
| --- | --- |
| Locks Alice | Locks Bob |
| Waits for Bob | Waits for Alice |
| Cannot continue | Cannot continue |

Taking two mutexes means acquiring them sequentially; there is no built-in operation here that grabs both simultaneously. Every path taking these locks must follow one common order, including readers that need both.

For `Arc<Mutex<u64>>`, an allocation address can define a process-local order. **The address of an `Arc` handle is the wrong identity:** two cloned handles can sit in different stack slots while pointing to the same lock. Use `Arc::as_ptr` for the shared allocation, and detect identical accounts with `Arc::ptr_eq`. The held `Arc`s keep those allocations alive. [Arc pointer identity](https://doc.rust-lang.org/std/sync/struct.Arc.html#method.as_ptr), [pointer addresses](https://doc.rust-lang.org/std/primitive.pointer.html#method.addr).

This is an optional extension, not a reason to replace the simpler ledger mutex. It reuses `move_funds` and `TransferError` above:

```rust
use std::sync::{Arc, Mutex};

#[derive(Debug)]
enum LockedTransferError {
    SameAccount,
    Poisoned,
    Rejected(TransferError),
}

fn transfer_between(
    source: &Arc<Mutex<u64>>,
    destination: &Arc<Mutex<u64>>,
    amount: u64,
) -> Result<(), LockedTransferError> {
    // Our API rejects a self-transfer before trying to lock twice.
    if Arc::ptr_eq(source, destination) {
        return Err(LockedTransferError::SameAccount);
    }

    let source_first = Arc::as_ptr(source).addr() < Arc::as_ptr(destination).addr();
    let (first, second) = if source_first {
        (source, destination)
    } else {
        (destination, source)
    };
    let mut first_guard = first.lock().map_err(|_| LockedTransferError::Poisoned)?;
    let mut second_guard = second.lock().map_err(|_| LockedTransferError::Poisoned)?;

    let (from, to) = if source_first {
        (&mut *first_guard, &mut *second_guard)
    } else {
        (&mut *second_guard, &mut *first_guard)
    };
    move_funds(from, to, amount).map_err(LockedTransferError::Rejected)
}

fn main() {
    let alice = Arc::new(Mutex::new(1_000));
    let bob = Arc::new(Mutex::new(500));
    for result in [
        transfer_between(&alice, &bob, 200),
        transfer_between(&bob, &alice, 100),
        transfer_between(&alice, &Arc::clone(&alice), 1),
        transfer_between(&alice, &bob, 2_000),
    ] {
        match result {
            Err(LockedTransferError::Rejected(reason)) => println!("Rejected: {reason:?}"),
            other => println!("{other:?}"),
        }
    }
}
```

This prints two successes, a same-account error and an insufficient-funds rejection. The demonstration is sequential; the deadlock argument comes from every concurrent caller taking the same allocation order. A canonical account ID can also define an order if your design guarantees one stable ordering key per lock. Neither ordering choice fixes a cycle involving some other lock taken inconsistently elsewhere.

**Readers must follow the same boundary.** Reading Alice, releasing her lock, then reading Bob can combine balances from different moments even when every transfer locks both accounts. A combined snapshot must hold both locks in the common order, or ask the single ledger owner for both values.

## A Real Choice: Metrics Collection

Return to the café from the first post. Each till records a completed order: one more request, its elapsed time, and whether it failed. A dashboard wants a consistent view of those three totals.

### The Channel Approach: The Dedicated Accountant

One metrics actor handles `Record` and `Snapshot` commands, just like our ledger actor. A `Record` changes the related totals; one `Snapshot` copies them together.

Decide what happens when the actor cannot keep up:

| Queue policy | What the caller experiences |
| --- | --- |
| Unbounded `channel().send(...)` | Does not wait for buffer space; memory can grow if the consumer falls behind. |
| Bounded `sync_channel(...).send(...)` | Waits on a full buffer; the caller is slowed down. |
| Bounded `try_send(...)` | Returns immediately with success, full, or disconnected; the application handles refusal. |

These are capacity policies, not different guarantees that the metric has been recorded. A single accountant can be a bottleneck. Reply handling must also avoid blocking that accountant indefinitely. [SyncSender methods](https://doc.rust-lang.org/std/sync/mpsc/struct.SyncSender.html).

Do not infer one heap allocation per message from the channel API. Storage strategy, message payloads, string creation and batching all matter. Likewise, no application-level mutex in the caller does not mean the channel has no internal synchronization costs.

### The Mutex Approach: The Shared Spreadsheet

Keep related totals together and update them under one guard. This self-contained example chooses **saturation** for diagnostic counters: once a counter reaches its maximum it stops growing, so it is no longer exact. That policy is explicit and is not suitable for silently accepting invalid money transfers.

```rust
use std::{sync::Mutex, thread};

#[derive(Clone, Copy, Debug, Default)]
struct Metrics {
    requests: u64,
    total_latency_ms: u64,
    errors: u64,
}

impl Metrics {
    fn record(&mut self, latency_ms: u64, failed: bool) {
        self.requests = self.requests.saturating_add(1);
        self.total_latency_ms = self.total_latency_ms.saturating_add(latency_ms);
        self.errors = self.errors.saturating_add(u64::from(failed));
    }
}

fn main() {
    let metrics = Mutex::new(Metrics::default());
    thread::scope(|scope| {
        for (latency, failed) in [(40, false), (60, true)] {
            let metrics = &metrics;
            scope.spawn(move || {
                metrics.lock().expect("metrics poisoned").record(latency, failed);
            });
        }
    });
    let snapshot = *metrics.lock().expect("metrics poisoned");
    println!("{snapshot:?}");
}
```

The totals are two requests, 100 ms and one error. Scoped threads borrow the mutex and finish before the snapshot, so no `Arc` is needed. Keep formatting and export work outside the lock. For per-endpoint maps, also bound the set of labels: arbitrary URLs can grow either design's map without limit.

### Would Atomic Counters Be Simpler?

For one independent count, often yes. `fetch_add` is one atomic read-modify-write operation; a separate `load`, addition and `store` can lose increments. `Relaxed` still makes that atomic operation indivisible, but does not establish ordering for unrelated data. Acquire/Release can publish and observe other writes when the required synchronization relationship is established. [Atomic operations](https://doc.rust-lang.org/std/sync/atomic/index.html), [Ordering](https://doc.rust-lang.org/std/sync/atomic/enum.Ordering.html).

Three separate atomic counters are still three separate values. Even `SeqCst` does not turn three loads into one simultaneous snapshot. A dashboard can read the new request count and the old latency total. Decide whether that approximation is acceptable before replacing a mutex with atomics. Also choose what should happen on counter overflow.

There is no useful universal claim that channels handle one particular request rate and mutexes another. Measure your own producer count, critical-section work, queue depth, allocations, rejected updates, snapshot latency and tail latency. An unbounded queue hiding a slow consumer is not proof of higher sustainable throughput.

## Making the Choice

Start with the invariant: **which values must be checked, changed and observed together?** Then choose an owner for that operation.

| Need | A reasonable starting point |
| --- | --- |
| Commands with explicit replies, batching or a controlled work queue | One owner receiving messages |
| A small, synchronous shared update | One mutex around the complete invariant |
| An independent counter | An atomic operation with a justified ordering |
| Parallel updates to unrelated groups of data | Separate owners or locks, with a protocol for operations crossing groups |
| Durable changes shared by application instances | A storage transaction with appropriate constraints and isolation |

Both an actor and a mutex can offer a coherent in-memory operation. Both can wait too long. Both can be combined with async code, but the blocking examples above must not be copied into a runtime worker that needs to keep servicing other tasks. For short, lightly contended in-memory access, a standard mutex may be appropriate inside async code; release its guard before awaiting. [Tokio's shared-state guidance](https://tokio.rs/tokio/tutorial/shared-state).

## The Plot Twist: It's All Single-Process

The standard channels and `Arc<Mutex<_>>` values shown here coordinate threads inside one process. Starting ten application instances gives you ten separate in-memory ledgers unless you deliberately share a storage authority. A local mutex remains useful inside each instance; it cannot protect a different instance's copy.

A database transaction can group the debit and credit in the same database, and constraints can enforce parts of the money rule. But a transaction is not automatic global atomicity across independent databases, an email service and a payment provider.

Isolation still matters. PostgreSQL's default Read Committed mode gives each statement its own snapshot; two queries in the same transaction can see different committed states. Serializable provides stronger protection against concurrency anomalies, but applications must handle serialization failures and retry whole transactions correctly. Row locking or conditional updates may be appropriate for a particular invariant. [PostgreSQL isolation levels](https://www.postgresql.org/docs/current/transaction-iso.html).

For external effects, plan for uncertainty: a timeout does not prove a payment was never sent. Request identities, recorded outcomes and recovery rules answer questions that ownership and locks cannot answer on their own.

## Check Your Understanding

Before looking at the answers, explain these aloud:

1. The debit and credit both use mutexes. Is the transfer automatically all-or-nothing?
2. One actor owns both accounts. Can it offer a coherent transfer and snapshot?
3. Two `Arc` clones have different variable addresses. Do they protect different accounts?
4. All three dashboard counters use `SeqCst`. Is reading them a coherent snapshot?
5. A command was sent successfully, but its reply disappeared. Is retrying definitely safe?

**Answers:** (1) No; one boundary must protect the whole operation, and failure paths must preserve the invariant. (2) Yes, with one complete handler and a snapshot command; durability remains separate. (3) No; inspect shared allocation identity. (4) No; separate loads can still span an update. (5) No; the command may already have changed the state.

The habit to carry forward is to trace **who owns the state, where the whole operation is protected, and what remains true if the next step fails**. In the [semaphore post](/rust/concepts/2025/12/29/rust-semaphore.html), we apply the same reasoning to a different invariant: how much work may hold a resource at once.
