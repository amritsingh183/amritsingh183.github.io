---
layout: post
title: "Mastering Tokio Semaphores in Rust: A Complete Guide"
date: 2025-12-29 18:23:00 +0530
categories: rust concepts
last_updated: 2026-09-19
---

# The Two Faces of Tokio's Semaphore: Wait or Reject?

*A permit controls admission. Its owner controls how long the resource remains reserved.*

## The Night Everything Slowed Down

Imagine a café taking online orders. It has two coffee machines. Letting 10,000 order tasks wait for those machines does not create more machines; it creates a large waiting room. Each waiting task may still own an order, a connection or a response buffer.

A server faces the same problem. A database pool might already cap its connections at 200 while thousands of requests wait outside the pool. The pool limit can be working correctly and the application can still run out of memory.

We need to decide **how much work may enter, where it waits, and when it is refused**. A semaphore helps with one part of that design.

This builds on the [concurrency guide](/rust/concepts/2025/11/12/rust-concurreny-vs-go.html) and [atomicity and isolation](/rust/concepts/2025/11/14/rust-acid-concurrency.html). The first post's ownership, scope and underscore-binding lessons are prerequisites. We will use them directly rather than postpone the tricky parts.

**Reviewed on 19 September 2026 for Rust 1.98.1, edition 2024.** Tokio is a separate library, not part of the Rust compiler. API claims were checked against Tokio 1.53.1 documentation; the examples use the available Tokio 1.52.3 release. Their Cargo dependency is:

```toml
[dependencies]
tokio = { version = "=1.52.3", features = ["macros", "rt-multi-thread", "sync", "time"] }
```

Each Rust block with a `main` is a separate program. The one marked **intentionally does not compile** is an ownership lesson; smaller fragments are labeled. Timers simulate waiting for equipment or I/O. They do not perform real coffee-making, database access or HTTP requests.

## First, a Quick Primer: What Even Is a Semaphore?

Think of a semaphore as a box of permission cards. A café with two machines starts with two cards. Before using a machine, take a card; keep it until you finish. Returning it allows another order to start.

For a fixed-capacity design, the rule is: **every operation using the resource holds a permit for the entire period it uses that resource**. Any path that skips acquisition or releases early escapes your limit.

A mutex guard gives access to protected data. A semaphore permit reserves capacity; it does not hand you a `&mut T` or identify a particular coffee machine. A resource pool may do both allocation and admission for you. Giving out two permits does not make unsynchronized mutation of one shared value safe.

Tokio distributes queued acquisitions fairly. A request for three permits at the front can delay a request for one, even when two are available. This is **head-of-line blocking**: the large group at the front holds up the smaller group behind it. Fair admission does not promise a deadline or the order in which admitted jobs finish. [Semaphore](https://docs.rs/tokio/latest/tokio/sync/struct.Semaphore.html).

## The `'static` Problem: Why There Are Owned Permits

Before choosing whether to wait, understand who owns the permit. `tokio::spawn` requires both its future and its result to be `Send + 'static`. `std::thread::spawn` has corresponding requirements for its closure and result. These APIs may let the work outlive the scope that starts it. [Tokio spawning](https://tokio.rs/tokio/tutorial/spawning), [thread spawning](https://doc.rust-lang.org/std/thread/fn.spawn.html).

`T: 'static` does **not** mean a value must stay alive forever. It means its type contains no borrow that requires some shorter lifetime. You may still drop that value immediately.

| Type | Satisfies `'static`? | Why |
| --- | --- | --- |
| `String`, `Vec<u8>`, `u64` | Yes | No borrowed data inside these types |
| `&'static str` | Yes | A genuinely static borrow is allowed |
| `&'a str` for a shorter `'a` | No | The borrowed text has a shorter guarantee |
| `Arc<Mutex<Data>>` | If `Data: 'static` | `Arc` does not extend references stored inside `Data` |

`Send` is a separate question: can the value be transferred between threads? Neither `'static` nor an `Arc` wrapper automatically supplies thread safety for arbitrary contents. [Arc's guarantees](https://doc.rust-lang.org/std/sync/struct.Arc.html#thread-safety).

### A borrowed permit cannot outlive its lender

`acquire()` returns `SemaphorePermit<'_>`, which borrows the semaphore. This complete example **intentionally does not compile**:

```rust
use tokio::sync::Semaphore;

#[tokio::main]
async fn main() {
    let sem = Semaphore::new(2);
    let permit = sem.acquire().await.unwrap();
    tokio::spawn(async move {
        drop(permit);
    }).await.unwrap();
}
```

The error is that `sem` does not live long enough for the required `'static` borrow. Even immediately awaiting the handle does not relax `spawn`'s signature. This is not a claim that borrowed permits are non-`Send`; `SemaphorePermit` implements `Send`. [Borrowed permit](https://docs.rs/tokio/latest/tokio/sync/struct.SemaphorePermit.html).

`acquire_owned()` instead consumes an `Arc<Semaphore>` and returns an `OwnedSemaphorePermit` that keeps an owned reference to the semaphore. In `Arc::clone(&sem).acquire_owned()`, **your `Arc::clone` creates the extra owner**; the acquisition method consumes that clone. [Owned acquisition implementation](https://docs.rs/tokio/latest/src/tokio/sync/semaphore.rs.html).

## The Patient One: `acquire_owned`

If capacity is unavailable, awaiting acquisition suspends the task rather than blocking its OS thread. If capacity is available, it can complete without suspending. Creating the future alone does not perform acquisition; it must be polled, normally through `.await`.

Here six café orders share two machines. We acquire **before** spawning each order and explicitly move the permit into the task:

```rust
use std::sync::Arc;
use tokio::{sync::Semaphore, time::{sleep, Duration}};

#[tokio::main]
async fn main() {
    let machines = Arc::new(Semaphore::new(2));
    let mut orders = Vec::new();

    for id in 1..=6 {
        let permit = Arc::clone(&machines).acquire_owned().await.unwrap();
        orders.push(tokio::spawn(async move {
            let _permit = permit; // A real binding, held through the work.
            println!("Order {id}: starting");
            sleep(Duration::from_millis(20)).await;
            println!("Order {id}: finished");
        })); // Returning from the task drops its permit.
    }

    for order in orders {
        order.await.expect("order task panicked");
    }
    println!("Free machines: {}", machines.available_permits());
}
```

At most two orders are inside the guarded work at a time. The exact print order can vary. At the end, all six tasks have been joined and both permits are available again. The `unwrap` on acquisition is specific to this demo: it never closes the semaphore. A service that supports closure should handle that error.

This finite example retains six task handles. For an endless input stream, a growing `Vec<JoinHandle<_>>` would retain more and more task results even if only two jobs were active. We will bound that too later.

### Waiting is a policy, not a promise of completion

Acquisition can fail because the semaphore closes. A waiter can be cancelled. A current holder can retain its permit indefinitely. And a job can fail after acquiring. Therefore waiting for a permit is not a guarantee that a database write or payment will finish.

Use waiting when you have room for waiting work and a useful deadline. For important work, separately decide when you acknowledge acceptance, how it is recorded, and how retry avoids duplicates. That is the same distinction as “message sent” versus “transfer completed” in the previous post.

## The Immediate One: `try_acquire_owned`

`try_acquire_owned()` is synchronous: it returns a permit or an error without awaiting capacity. Its errors distinguish **no permits now** from **a closed semaphore**. This caller does not enter the waiting queue; other callers may already be queued on the same semaphore. [TryAcquireError](https://docs.rs/tokio/latest/tokio/sync/enum.TryAcquireError.html).

The café can use it to tell a customer “busy; try again” before accepting the order. That is useful even for important work. Refusing an order honestly is different from silently losing an order you already promised to fulfill.

This complete demonstration uses a single-thread runtime and makes four admission attempts without yielding. Thus the first two reserve all capacity before the spawned tasks start; the next two are refused:

```rust
use std::sync::Arc;
use tokio::{sync::{Semaphore, TryAcquireError}, time::{sleep, Duration}};

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let machines = Arc::new(Semaphore::new(2));
    let mut accepted = Vec::new();

    for id in 1..=4 {
        match Arc::clone(&machines).try_acquire_owned() {
            Ok(permit) => {
                println!("Order {id}: accepted");
                accepted.push(tokio::spawn(async move {
                    let _permit = permit;
                    sleep(Duration::from_millis(20)).await;
                    println!("Order {id}: finished");
                }));
            }
            Err(TryAcquireError::NoPermits) => {
                println!("Order {id}: busy; please retry later");
            }
            Err(TryAcquireError::Closed) => {
                println!("Order {id}: admissions are closed");
            }
        }
    }

    for order in accepted {
        order.await.expect("order task panicked");
    }
}
```

The limit covers this admitted work. It does not guarantee a process-wide memory ceiling: the request may already have been buffered elsewhere, or one accepted job may allocate a very large object. Avoid turning the error path into a flood of expensive logs or an unbounded retry queue.

## The Showdown: When to Choose What

These are two independent decisions:

| Ownership decision | Method family |
| --- | --- |
| Borrow the semaphore while doing work in a suitable scope | `acquire`, `try_acquire` |
| Own a semaphore reference, often to transfer a permit into a new task | `acquire_owned`, `try_acquire_owned` |

| Admission decision | Policy |
| --- | --- |
| A bounded backlog is acceptable | Await acquisition, usually within a deadline |
| The caller needs an immediate answer | Try acquisition; handle busy and closed separately |
| Acknowledged work must survive restart | Add durable acceptance and recovery; neither acquisition method provides this |
| Optional telemetry can be dropped | Immediate refusal may be appropriate; measure drops |

Important requests can be rejected for retry. Optional work can wait in a small queue. The business contract decides; “important means wait forever” is not a safe rule.

Health checks need a truthful, quick result. Decide whether a check reports basic process life or readiness to accept work. Do not turn overload into a false healthy response, and do not accidentally make every check wait behind the same saturated work queue.

### Concurrency is not a request rate

Two machines means at most two orders use machines **at once**. If each order takes a second, a rough steady-state ceiling is two per second. If each takes a millisecond, the same capacity could allow roughly 2,000 per second, before other costs. No time-based rate was enforced.

A rate limiter needs a time policy, such as replenishing tokens periodically. If a semaphore implements a token bucket, finishing work must not automatically replenish the spent rate token; replenishment follows time instead. Keep that design separate from the acquire-work-drop concurrency pattern used here. [Tokio's token-bucket example](https://docs.rs/tokio/latest/tokio/sync/struct.Semaphore.html#rate-limiting-using-a-token-bucket).

## For Go Developers: A Familiar Dance

A buffered Go channel can count active work. This is a fragment inside a Go function:

```go
sem := make(chan struct{}, 2)
sem <- struct{}{}             // Reserve capacity; may block this goroutine.
go func() {
    defer func() { <-sem }() // Return capacity when this function exits.
    doWork()
}()
```

In this convention, a buffered token means an occupied slot. With Tokio, a held permit represents that slot. `select` with a `default` branch gives Go's immediate-admission alternative. This is a conceptual mapping, not a promise of identical fairness or scheduling. [Go send, select and defer rules](https://go.dev/ref/spec).

Rust ties release to the owned permit's destructor. The first post's RAII rule is doing useful work here: when that value is dropped, its capacity is returned. You must still put the value under the right owner.

## The Traps: Follow the Permit

### Trap #1: Thinking `acquire()` Is Wrong Inside `spawn`

A borrowed permit obtained **outside** a task from a local semaphore caused our earlier lifetime error. Borrowing **inside** a task that owns an `Arc` is different. This complete example works:

```rust
use std::sync::Arc;
use tokio::{sync::Semaphore, time::{sleep, Duration}};

#[tokio::main]
async fn main() {
    let sem = Arc::new(Semaphore::new(2));
    let task_sem = Arc::clone(&sem);
    let task = tokio::spawn(async move {
        let _permit = task_sem.acquire().await.unwrap();
        sleep(Duration::from_millis(20)).await;
    });
    task.await.unwrap();
}
```

The task owns `task_sem`; the permit borrows from it during that task's execution. A permit borrowed from a genuinely static semaphore is another valid case. Owned acquisition is particularly useful when you want to acquire **before spawning**, then transfer the permit.

But do not generalize this one-task example into “spawn every incoming request, acquire inside.” That design bounds permit holders while allowing the population of waiting tasks to grow.

### Trap #2: Holding a Standard Mutex Across `.await`

An uncontended `std::sync::Mutex::lock()` can return promptly. Holding its guard does not itself park the OS thread, and awaiting a semaphore does not itself block that thread either.

The danger appears when another task calls the mutex's **blocking** `lock()` while the first task is suspended with its guard. That contender can occupy the worker needed to resume the holder. A standard `MutexGuard` also is not `Send`, so retaining it across `.await` makes many `tokio::spawn` examples fail to compile. [MutexGuard](https://doc.rust-lang.org/std/sync/struct.MutexGuard.html).

Usually, read or change the shared state in a small synchronous block, release the guard, then await. If your operation really needs a lock across suspension, an async mutex is designed for that. It still does not prevent logical cycles: a task holding a mutex can wait for a permit held by a task waiting for that mutex. Tokio's mutex also does not poison on panic; interrupted state needs an application policy. [Shared state](https://tokio.rs/tokio/tutorial/shared-state), [Tokio Mutex](https://docs.rs/tokio/latest/tokio/sync/struct.Mutex.html).

### Trap #3: Assuming `async move` Captures Everything Nearby

This fragment is wrong because the task never mentions the permit:

```rust
// Flawed fragment: `permit` remains owned by the outer scope.
let permit = Arc::clone(&sem).acquire_owned().await?;
tokio::spawn(async move {
    do_work().await;
});
```

`move` controls **how used values are captured**. It does not move every local variable into the task. Here the outer scope might return and release capacity while the work continues, or retain it unnecessarily after the work ends. A comment saying “permit dropped here” changes nothing. [Async capture modes](https://doc.rust-lang.org/reference/expressions/block-expr.html#capture-modes).

The correct fragment is:

```rust
let permit = Arc::clone(&sem).acquire_owned().await?;
let task = tokio::spawn(async move {
    let _permit = permit;
    do_work().await;
});
```

`_permit` is a real binding. Its destructor is not arbitrarily moved to its last use by non-lexical lifetimes. Unless you move or explicitly drop it, it remains owned through the end of that scope, including the await. An explicit `drop(permit)` **after** the work is also clear and correct. [Local drop scopes](https://doc.rust-lang.org/reference/destructors.html#scopes-of-local-variables).

Recall the three different underscore cases from the [first post](/rust/concepts/2025/01/01/rust-var-const-lifetimes.html):

| Expression | What happens |
| --- | --- |
| `let _permit = sem.acquire().await?;` | A named binding holds the newly acquired permit. |
| `let _ = sem.acquire().await?;` | No binding retains the returned permit; it is dropped in that statement. |
| `let _ = permit;` | The wildcard does not move an existing variable. If this is its only appearance inside a closure, it does not cause capture. |

The last two cases are different. Do not teach “bare underscore always drops the existing variable.” [Wildcard patterns](https://doc.rust-lang.org/reference/patterns.html#wildcard-pattern), [wildcards and capture](https://doc.rust-lang.org/reference/types/closure.html#wildcard-pattern-bindings).

### Trap #4: Confusing Cancellation With Rollback

An acquired permit is returned when its owner is dropped: on normal scope exit, an early return, or panic unwinding. Destroying a cancelled async task's future also drops its owned permit. `abort()` requests cancellation; await the handle if you need to observe that termination has finished. A non-yielding task can delay cancellation. Dropping a plain `JoinHandle` merely detaches its task. [Task handles and cancellation](https://docs.rs/tokio/latest/tokio/task/struct.JoinHandle.html).

Dropping the permit does not undo a database write or cancel a remote payment already sent. If resource use continues in a child task that can outlive its parent, move the permit into that child. Merely awaiting the child's `JoinHandle` while keeping the permit in the parent is insufficient: cancelling the parent drops its permit and detaches the still-running child. The reservation must remain owned by the work it covers through cancellation too.

`OwnedSemaphorePermit::forget()` deliberately prevents returning the permits. `std::mem::forget` also skips destruction; process abort does not run ordinary cleanup. RAII is reliable under its drop rules, not a guarantee that every process termination runs code. [Owned permits](https://docs.rs/tokio/latest/tokio/sync/struct.OwnedSemaphorePermit.html), [destructors](https://doc.rust-lang.org/reference/destructors.html).

## Production Wisdom: Bound the Whole Path

### 1. Match the Limit to the Resource and Its Lifetime

Decide whether a permit covers an entire connection, one query, a response body being read, or one CPU job. A connection-level permit held by an idle client has a very different effect from a query-level permit. Release only when that particular resource use ends.

A database pool already limits connections. Adding a matching semaphore may simply add another queue. Use a separate limit when you have another budget to enforce, such as expensive reports that should occupy only part of the pool. Avoid waiting for a permit while holding some other scarce resource that permit holders need.

Counts also apply only to callers sharing the **same semaphore**. Ten processes each allowing 20 operations can admit 200 in total. A per-process count is not a cluster-wide budget.

Do not check `available_permits() > 0` and assume you have reserved anything. Another caller can acquire between that observation and your action. Call `try_acquire_owned()` to make the actual attempt.

### 2. Put a Deadline on Acquisition

This program holds the only permit while another acquisition times out. Releasing the first permit restores capacity:

```rust
use std::sync::Arc;
use tokio::{sync::Semaphore, time::{timeout, Duration}};

#[tokio::main]
async fn main() {
    let sem = Arc::new(Semaphore::new(1));
    let held = Arc::clone(&sem).acquire_owned().await.unwrap();

    match timeout(
        Duration::from_millis(20),
        Arc::clone(&sem).acquire_owned(),
    ).await {
        Ok(Ok(_permit)) => println!("Admitted"),
        Ok(Err(_)) => println!("Admissions closed"),
        Err(_) => println!("Stopped waiting for capacity"),
    }

    drop(held);
    println!("Free permits: {}", sem.available_permits());
}
```

It prints “Stopped waiting for capacity” and then one free permit. The timeout covers **acquisition**, not work performed after acquisition. A timeout also depends on the runtime being able to poll it; it cannot preempt arbitrary code that never yields. [Tokio timeout](https://docs.rs/tokio/latest/tokio/time/fn.timeout.html).

Cancelling a pending acquisition loses its queue position without giving you a permit. Repeatedly recreating it in a `select!` loop can keep putting you at the back. Preserve and poll the same acquisition future when preserving position matters. This is a fairness/cancellation question, separate from leaking a permit. [Cancellation safety](https://docs.rs/tokio/latest/tokio/macro.select.html#cancellation-safety).

`sem.close()` prevents new permits and wakes waiting acquisitions with an error. It does not revoke permits already held or abort their tasks. Graceful shutdown needs a policy for those accepted jobs. Zero initial permits is valid, but nothing progresses until permits are supplied; a many-permit request larger than capacity can also wait indefinitely if capacity never grows. [Semaphore acquisition and closure](https://docs.rs/tokio/latest/tokio/sync/struct.Semaphore.html#method.close).

### 3. Bound Buffered Jobs, Admitted Tasks and Retained Results

A bounded channel plus a semaphore is only useful if the dispatcher respects both bounds. This flawed sequence defeats the queue bound:

```text
Receive job → spawn task immediately → task waits for a permit
```

The dispatcher keeps emptying the bounded channel into an unbounded population of waiting tasks. The jobs still exist; they have merely moved.

Instead, this complete example limits the inbox to four jobs and the task set to two entries. It acquires before spawning. `JoinSet` retains tasks until their results are collected, so bounding its length also bounds retained completion entries.

```rust
use std::sync::Arc;
use tokio::{
    sync::{mpsc, Semaphore},
    task::JoinSet,
    time::{sleep, Duration},
};

#[tokio::main]
async fn main() {
    const LIMIT: usize = 2;
    let (tx, mut rx) = mpsc::channel::<u32>(4);
    let machines = Arc::new(Semaphore::new(LIMIT));

    let producer = tokio::spawn(async move {
        for id in 1..=12 {
            if tx.send(id).await.is_err() {
                return; // Dispatcher has stopped accepting jobs.
            }
        }
    });

    let mut tasks = JoinSet::new();
    while let Some(id) = rx.recv().await {
        if tasks.len() >= LIMIT {
            let done = tasks.join_next().await.unwrap().expect("order panicked");
            println!("Collected order {done}");
        }

        let permit = Arc::clone(&machines).acquire_owned().await.unwrap();
        tasks.spawn(async move {
            let _permit = permit;
            sleep(Duration::from_millis(20)).await;
            id
        });
    }

    while let Some(result) = tasks.join_next().await {
        println!("Collected order {}", result.expect("order panicked"));
    }
    producer.await.expect("producer panicked");
}
```

All twelve IDs are eventually collected in a normal run; completion order is unspecified. The single producer waits when its inbox is full, so pressure reaches the source of work. The dispatcher can also hold one job it has already removed, and the producer can hold one pending send. “Four buffered” never meant “only four jobs exist anywhere.” [Bounded async channels](https://docs.rs/tokio/latest/tokio/sync/mpsc/index.html), [sending](https://docs.rs/tokio/latest/tokio/sync/mpsc/struct.Sender.html#method.send).

Here the capped task set alone would suffice to limit these isolated jobs; the semaphore becomes useful when **other paths share the same machine budget**. It must be the same `Arc`, not a freshly constructed semaphore in each path. The example shows the two responsibilities separately: permits reserve a shared resource; the task set owns and collects this dispatcher's tasks. [JoinSet](https://docs.rs/tokio/latest/tokio/task/struct.JoinSet.html).

This demo stops on a task panic. A service should choose whether to drain, cancel or persist remaining jobs. Dropping a `JoinSet` requests abortion of its tasks; waiting for shutdown is a separate step. Also bound payload sizes and the number of producers: ten thousand tasks blocked in `send().await` can still own ten thousand payloads outside a four-slot buffer.

### 4. Know What Synchronization Does and Does Not Give You

Tokio 1.53.1 explicitly documents memory ordering for its semaphore: successful acquire/release operations have acquire-and-release synchronization semantics. Earlier writes can therefore be made visible through the documented handoff. This is more than bookkeeping, but it does not supply exclusive access to arbitrary contents. With multiple permits, multiple holders may still access a resource concurrently; safe sharing rules still apply. [Semaphore memory ordering](https://docs.rs/tokio/latest/tokio/sync/struct.Semaphore.html#memory-ordering).

The practical question is the same as in the previous post: what invariant is actually protected? “At most two holders” does not imply “these two account updates form a transaction,” “every request finishes,” or “at most two requests arrive each second.”

## Check Your Understanding

1. You spawn 100,000 tasks, each awaiting a ten-permit semaphore. How many tasks can be waiting?
2. You write `async move`, but the block never uses the previously acquired permit. Who owns it?
3. A named `_permit` is unused after acquisition. Does Rust drop it before the next `.await`?
4. Acquiring a permit times out. Does this cancel another task's database query?
5. A semaphore has two permits. Does that enforce two requests per second?
6. You close the semaphore. Are the existing permit holders automatically stopped?

**Answers:** (1) Almost all of them; the permit count bounds holders, not tasks. (2) The outer scope. (3) No; it remains owned until its drop scope ends unless moved or explicitly dropped. (4) No; that timeout abandons that acquisition. (5) No; it limits concurrent holders. (6) No; accepted work needs its own shutdown policy.

When reviewing an async operation, follow the actual permit from acquisition to drop, then follow every place a job can wait. That trace tells you what your limit really controls. The [next post on Drop](/rust/2025/12/30/rust-drop.html) continues the ownership side of this story.
