---
layout: post
title: "Rust Concurrency explained for Go (Golang) Developers"
date: 2025-11-12 16:36:00 +0530
categories: rust concepts
last_updated: 2026-09-19
---

# Rust Concurrency for Go Developers: A Different Kind of Safety

In [the previous post](/rust/concepts/2025/11/01/rust-vs-go.html), we used ownership to control what a caller could do next. Now several workers need to make progress at once. The questions become: **who owns the data, who may touch it, and who waits for the work to finish?**

This is post 11 in the series. The [foundation post](/rust/concepts/2025/01/01/rust-var-const-lifetimes.html) and [ownership post](/rust/concepts/2025/02/09/rust-ownership.html) already introduced borrowing, `move`, `Arc`, mutex guards, and the difference between `T: 'static` and a value living forever. We will apply those ideas, keeping the gotchas beside the examples that need them.

Our everyday model is the café. There are order slips to process, a limited number of machines, a queue at the counter, and a closing time. A program needs the same decisions: how many workers, how much waiting work, what happens when a worker fails, and how to shut down.

**Reviewed on 19 September 2026 for Rust 1.98.1, edition 2024.** The examples use the standard library, Rayon, Crossbeam channels, and one small Tokio example. These are separate libraries with separate versions; installing a Rust compiler does not install an async runtime. [Rust 1.98.1](https://blog.rust-lang.org/2026/09/03/Rust-1.98.1/).

Each complete Rust block has its own `main` and is intended to run separately. Fragments and deliberate failures are labelled. For examples using external crates, create a Cargo binary project and add these dependencies to its manifest:

```toml
[package]
name = "concurrency-notes"
version = "0.1.0"
edition = "2024"

[dependencies]
rayon = "=1.12.0"
crossbeam-channel = "=0.5.17"
tokio = { version = "=1.53.1", features = ["macros", "rt-multi-thread", "time", "sync"] }
```

Put one program in `src/main.rs` and use `cargo run`. These pins fix the direct dependency versions; retain `Cargo.lock` to fix the resolved dependency graph. These are the versions used for this revision, not a claim that every later release behaves identically. [Rayon](https://docs.rs/rayon/1.12.0/rayon/), [Crossbeam channel](https://docs.rs/crossbeam-channel/latest/crossbeam_channel/), [Tokio](https://docs.rs/tokio/1.53.1/tokio/), [Cargo's lockfile guidance](https://doc.rust-lang.org/cargo/guide/cargo-toml-vs-cargo-lock.html).

## Two Valid Philosophies, With a Precise Safety Boundary

**Concurrency** means several activities can make progress during the same period. One barista can start a brew, take another order while it runs, then return to the brew. **Parallelism** means work actually happens at the same time, such as two baristas grinding beans on separate machines.

Do not translate `go f()` mechanically into an OS thread:

| Tool | What runs the work? | Useful mental picture |
| :-- | :-- | :-- |
| Go goroutine | Go's runtime schedules goroutines across OS threads | Many jobs managed by the runtime |
| `std::thread::spawn` or `scope` | An OS thread for each spawned worker | Give this job its own worker |
| Rayon parallel iterator | A pool of worker threads | Divide a CPU job among available workers |
| Tokio task | An async runtime polls futures on its worker threads | Resume a job when it can make progress |

These models have different costs and lifetime rules. An async task is not a new OS thread, and a scoped thread is not a lightweight goroutine. [Go goroutines](https://go.dev/doc/faq#goroutines), [Rust threads](https://doc.rust-lang.org/std/thread/), [Tokio tasks](https://tokio.rs/tokio/tutorial/spawning).

### A data race is a bug, not a performance trade-off

This is a **deliberately incorrect but runnable Go program**. Its WaitGroup ensures completion; it does not protect `counter++`.

```go
package main

import (
    "fmt"
    "sync"
)

func main() {
    var counter int
    var wg sync.WaitGroup
    for i := 0; i < 100; i++ {
        wg.Add(1)
        go func() {
            defer wg.Done()
            counter++ // Incorrect: concurrent unsynchronized writes.
        }()
    }
    wg.Wait()
    fmt.Println(counter)
}
```

Run it with `go run -race main.go` to look for the race. The detector observes executed paths; a clean run is not a proof about every possible execution. Replacing `Wait` with a sleep would add another mistake: elapsed time is not completion tracking. [Go race detector](https://go.dev/doc/articles/race_detector).

This Rust program is **deliberately non-compiling**:

```rust
fn main() {
    let mut counter = 0;
    std::thread::scope(|scope| {
        for _ in 0..100 {
            scope.spawn(|| counter += 1);
        }
    });
}
```

Each closure wants exclusive access to the same counter while earlier workers may still use it. The compiler rejects those overlapping borrows. This is a conservative ownership check, not a claim that every program rejected by Rust would necessarily fail at runtime.

Safe Rust prevents data races through its rules and sound library abstractions. It still permits deadlocks, stale decisions, lost work, and wrong arithmetic. We will meet examples of those distinctions below. [Rust's distinction between data races and race conditions](https://doc.rust-lang.org/nomicon/races.html).

### Read the bounds: Send, Sync, move, and 'static

These ideas from the ownership lesson now explain the thread APIs:

| Concept | Meaning | What it does not mean |
| :-- | :-- | :-- |
| `T: Send` | A value of type `T` may safely cross a thread boundary | Any sequence of operations on it is correct |
| `T: Sync` | `&T` is `Send`; shared references may cross that boundary | The value must be immutable internally |
| `move` closure | Captures what it uses by value | Borrowed data is copied into owned storage |
| `T: 'static` | The type contains no borrows limited to a shorter lifetime | The value must remain alive until program exit |

`String` is `Send` and `Sync`. `Rc<T>` is neither. `Cell<u32>` can move to another thread but cannot be shared between threads through ordinary shared references. `Arc<T>` synchronizes its reference counts; it does not add synchronization to `T`. An `Arc<RefCell<T>>` is therefore not a replacement for `Arc<Mutex<T>>`. [Send](https://doc.rust-lang.org/std/marker/trait.Send.html), [Sync](https://doc.rust-lang.org/std/marker/trait.Sync.html), [Arc thread safety](https://doc.rust-lang.org/std/sync/struct.Arc.html#thread-safety).

Ordinary `thread::spawn` requires its closure and return value to be `Send + 'static`. Moving an owned `String` into it works. Moving a reference to a local `String` only transfers the reference; the short borrow is still short. Calling `join` immediately does not change `spawn`'s signature. [thread::spawn](https://doc.rust-lang.org/std/thread/fn.spawn.html).

## Scoped Threads: Borrow Now, Finish Before Returning

Go's familiar waiting pattern is still valid. This is a **fragment**, assuming `items` and `process` exist:

```go
var wg sync.WaitGroup
for _, item := range items {
    wg.Add(1)
    go func(s string) {
        defer wg.Done()
        process(s)
    }(item)
}
wg.Wait()
```

The `Add` happens before the goroutine starts. You cannot add to an empty WaitGroup arbitrarily while another goroutine may already return from `Wait`. Current Go also provides `WaitGroup.Go` to combine starting and counting suitable functions; the older pattern is not obsolete. [WaitGroup](https://pkg.go.dev/sync#WaitGroup).

Rust's scope can prove something a plain WaitGroup does not express in a type: borrowed data will outlive all the scoped worker closures.

```rust
use std::thread;

fn main() {
    let orders = [String::from("coffee"), String::from("tea")];
    thread::scope(|scope| {
        for order in &orders {
            scope.spawn(move || println!("Preparing {order}"));
        }
    });
    println!("Both worker closures finished; {} orders remain here", orders.len());
}
```

The two preparation lines can appear in either order. The final line runs after both workers finish. No `Arc` is needed because the borrowed orders outlive the scope.

A scope **waits before returning**; it cannot promise that every worker will eventually finish. A stuck worker can hold the scope open forever. An automatically joined worker panic makes the scope panic; explicitly joining lets you inspect its panic result. `std::thread::scope` returns the closure's result directly, so a closure returning `()` does not produce a `Result` to unwrap. [Scope contract](https://doc.rust-lang.org/std/thread/fn.scope.html).

These are real OS threads. Two short strings illustrate lifetimes, not a reason to create a thread per customer in a busy service.

### Borrow different output slots without a mutex

Suppose the café is making uppercase labels for four order slips. Each worker reads one order and writes one distinct output slot:

```rust
use std::thread;

fn main() {
    let orders = ["coffee", "tea", "cocoa", "water"];
    let mut labels = vec![String::new(); orders.len()];

    thread::scope(|scope| {
        for (order, label) in orders.iter().zip(labels.iter_mut()) {
            scope.spawn(move || {
                *label = order.to_uppercase();
            });
        }
    });

    println!("{labels:?}");
}
```

The output remains `["COFFEE", "TEA", "COCOA", "WATER"]`, regardless of which worker finishes first. Each worker writes its assigned slot.

Why `iter_mut()` instead of repeatedly borrowing `&mut labels[i]`? The iterator safely hands out distinct mutable references. Repeated indexing asks the borrow checker to keep borrowing the collection while earlier borrows are still held by workers; that form is rejected. The ownership lesson's `split_at_mut` and `chunks_mut` solve related partitioning problems. `zip` stops when either input ends, so matching the lengths is part of this example's design. [Borrow splitting](https://doc.rust-lang.org/nomicon/borrow-splitting.html), [Iterator::zip](https://doc.rust-lang.org/std/iter/trait.Iterator.html#method.zip).

There is no runtime borrow checker here. There is still runtime work: allocation, thread creation, scheduling, string conversion, and joining.

Go can also share immutable data or let workers write disjoint elements before the caller waits. A channel send copies a Go value, but a slice or pointer can still refer to shared underlying data. Do not confuse copying a descriptor with copying its entire payload. [Go representation of values](https://go.dev/ref/spec#Representation_of_values).

## Rayon: When You Want to Parallelize CPU Work

For a large independent computation, use a pool instead of creating one OS thread for every item. Rayon partitions work and can let idle workers take work from busier workers.

```rust
use rayon::prelude::*;

fn main() {
    let numbers: Vec<i64> = (0..1_000_000).collect();
    let sum: i64 = numbers.par_iter().copied().sum();
    let squares: Vec<i64> = numbers.par_iter().map(|&n| n * n).collect();
    let sum_of_squares: i64 = squares.par_iter().copied().sum();

    println!("Sum: {sum}");
    println!("Sum of squares: {sum_of_squares}");
    println!("First three squares: {:?}", &squares[..3]);
}
```

The sums are `499999500000` and `333332833333500000`. They fit in `i64`; they do not fit in `i32`. The multiplication also happens in `i64`, rather than overflowing a smaller integer before conversion. Parallelism does not change the need to choose an appropriate numeric domain. [Rust integer overflow rules](https://doc.rust-lang.org/reference/expressions/operator-expr.html#overflow).

This example demonstrates the API, not a speedup. For a tiny operation, scheduling and memory traffic may cost more than the saved CPU time. Compare a sequential version with a release build on representative input.

### Order and reduction are part of correctness

Mapping this vector and collecting into a vector preserves its logical item order. The order in which the mapping closures run is different: a `println!` inside the closure can appear out of order. Sending results from parallel `for_each` into a channel likewise does not preserve input order. [Rayon indexed iteration](https://docs.rs/rayon/latest/rayon/iter/trait.IndexedParallelIterator.html).

A **fragment** showing local partial totals and a final combination:

```rust
let total: i64 = numbers.par_iter()
    .fold(|| 0_i64, |subtotal, &n| subtotal + n * n)
    .reduce(|| 0_i64, |left, right| left + right);
```

There can be several partial folds; the initializer is not called just once for the entire input. The reduction needs an identity and an associative combination for a stable mathematical result. Integer addition works here because these values stay in range. Floating-point addition is not associative, so regrouping can change rounding. Subtraction is another poor choice for an order-independent reduction. [Rayon fold and reduce](https://docs.rs/rayon/latest/rayon/iter/trait.ParallelIterator.html#method.reduce).

Use CPU work to evaluate a CPU pool. Sleeping or waiting for a socket blocks a Rayon worker; it is not a stand-in for expensive arithmetic.

## Channels: Move Work, Then Account for Its Lifetime

At the café, a channel is the tray where the cashier puts order slips for a barista. The tray can have a limited capacity. When it is full, the cashier must wait, reject an order, or choose another explicit policy. That pressure on the producer is *backpressure*.

Rust has several channel families:

| Channel | Receivers | How waiting works |
| :-- | :-- | :-- |
| `std::sync::mpsc` | One receiver; senders can be cloned | Blocking receive; `sync_channel` offers bounded capacity |
| Crossbeam channel | Senders and receivers can be cloned | Blocking operations, selection, and timeout APIs |
| Tokio `mpsc` | One receiver; senders can be cloned | Async waiting suitable for async tasks |

The standard library is sufficient for many cases. We use Crossbeam here because later workers need separate receiver handles. Its crate name in the manifest is `crossbeam-channel`; the Rust import is `crossbeam_channel`. [Standard channels](https://doc.rust-lang.org/std/sync/mpsc/), [Crossbeam](https://docs.rs/crossbeam-channel/latest/crossbeam_channel/), [Tokio mpsc](https://docs.rs/tokio/latest/tokio/sync/mpsc/).

### A producer and consumer that actually finish

```rust
use crossbeam_channel::bounded;
use std::thread;

fn main() {
    let (tx, rx) = bounded::<String>(2);

    thread::scope(|scope| {
        scope.spawn(move || {
            for drink in ["coffee", "tea", "cocoa"] {
                let order = drink.to_owned();
                if tx.send(order).is_err() {
                    return; // The consumer has gone away.
                }
                // order has moved into the send operation.
            }
        }); // This worker's tx is dropped when the worker returns.

        for order in rx {
            println!("Preparing {order}");
        }
    });
}
```

The consumer drains the buffered orders and stops after the final sender disappears. The scope also waits for the producer's closure. Discarding ordinary thread `JoinHandle`s would detach them; it would not make `main` wait.

Keep these four rules together:

1. Cloning a sender keeps the sending side alive. One forgotten clone can keep a receive loop waiting.
2. Cloning a receiver creates competing consumers of the **same** queue. It does not broadcast every message to every worker.
3. A successful send means the channel accepted the value, not that the order was prepared. A reply or acknowledgement is a separate protocol.
4. Sending an owned `String` transfers ownership. Sending an `Arc<T>` transfers one shared owner. The channel does not erase the payload's sharing rules.

Crossbeam disconnection is observed when all senders or all receivers are dropped; queued messages can still be received after the senders are gone. [Crossbeam disconnection](https://docs.rs/crossbeam-channel/latest/crossbeam_channel/#disconnection), [Receiver::recv](https://docs.rs/crossbeam-channel/latest/crossbeam_channel/struct.Receiver.html#method.recv).

### Select chooses one ready operation

This **complete example** models a wait for an order with a timeout. Its tiny producer may finish before or after the timeout; neither outcome is promised.

```rust
use crossbeam_channel::{bounded, select};
use std::{thread, time::Duration};

fn main() {
    let (tx, rx) = bounded(1);
    thread::scope(|scope| {
        scope.spawn(move || {
            let _ = tx.send("coffee"); // Receiver leaving is acceptable here.
        });
        select! {
            recv(rx) -> result => println!("Receive result: {result:?}"),
            default(Duration::from_millis(50)) => println!("Stopped waiting"),
        }
        drop(rx);
    });
}
```

One `select!` selects one operation; it does not drain every ready channel. Disconnection is also a ready result, so ignoring an error and blindly looping can create a busy loop. Timing out here only stops this receive attempt; the scope still waits for its worker. [Crossbeam selection](https://docs.rs/crossbeam-channel/latest/crossbeam_channel/#selection).

## Worker Pools: Bound the Workers and Both Queues

Now four baristas share a queue, and completed labels go into a second queue. Both queues are bounded. One producer submits while the main thread receives results.

```rust
use crossbeam_channel::bounded;
use std::thread;

fn main() {
    let (job_tx, job_rx) = bounded::<u32>(4);
    let (result_tx, result_rx) = bounded::<(u32, u32)>(4);

    thread::scope(|scope| {
        for _ in 0..4 {
            let jobs = job_rx.clone();
            let results = result_tx.clone();
            scope.spawn(move || {
                for id in jobs {
                    if results.send((id, id * id)).is_err() {
                        break;
                    }
                }
            });
        }
        drop(job_rx);     // Only real workers should keep the job queue alive.
        drop(result_tx);  // Only workers will produce results.

        scope.spawn(move || {
            for id in 0..20 {
                if job_tx.send(id).is_err() {
                    return;
                }
            }
        });

        let mut completed = 0;
        for (id, square) in result_rx {
            println!("Job {id}: {square}");
            completed += 1;
        }
        println!("Completed {completed} jobs");
    });
}
```

In an ordinary successful run it prints 20 results in an unspecified order. IDs preserve which result belongs to which input.

Trace shutdown: the producer finishes and drops the last job sender; workers drain the jobs and exit; their result senders drop; the consumer drains the results and finishes. If workers panic, the scope propagates an unhandled panic. A real service should choose how to record per-job failures, not treat missing outputs as successes.

**Why separate submission from collection?** Imagine one thread sends every job before receiving any result. The result queue can fill; all workers then wait to send results; the job queue fills; the producer waits to send more jobs. Nobody receives. Every access is memory-safe, yet the program deadlocks.

The bounded queues limit queued item counts, not the whole program's memory. Worker-held values, producer-held values, and payload size count too. Creating a million blocked producers would merely move the backlog out of the channel.

The Go counterpart needs an explicit owner that closes the result channel after **all** workers finish. Closing the jobs channel does not close the results channel, and each worker must not independently close the same shared result channel. Go's pipeline article demonstrates that coordination. [Go pipelines and fan-in](https://go.dev/blog/pipelines).

### Queue admission timeouts are not job deadlines

This complete example puts one item into a full queue, then times out trying to add another. We keep the receiver alive to distinguish “full” from “disconnected.”

```rust
use crossbeam_channel::{bounded, SendTimeoutError};
use std::time::{Duration, Instant};

fn main() {
    let (tx, rx) = bounded(1);
    tx.send(String::from("coffee")).unwrap();
    let deadline = Instant::now() + Duration::from_millis(10);

    match tx.send_deadline(String::from("tea"), deadline) {
        Ok(()) => println!("Tea entered the queue"),
        Err(SendTimeoutError::Timeout(order)) => {
            println!("Queue stayed full; still own {order}");
        }
        Err(SendTimeoutError::Disconnected(order)) => {
            println!("No receiver; still own {order}");
        }
    }
    println!("Already queued: {}", rx.recv().unwrap());
}
```

The observable result is that tea never enters the full queue, and coffee remains available. `send_deadline` bounds waiting for admission. It does not bound execution of an accepted job, cancel work, or guarantee the whole scope returns by that time. If a send is immediately possible, Crossbeam documents that it may succeed even after the deadline. [Sender::send_deadline](https://docs.rs/crossbeam-channel/latest/crossbeam_channel/struct.Sender.html#method.send_deadline).

For a processing deadline, carry the deadline with the job, check it before starting and during cooperative work, and configure timeouts on underlying I/O. There is no general safe operation that instantly kills an arbitrary OS thread. Report rejected, failed, cancelled, and completed jobs separately.

## Cancellation: Request, Observe, Then Wait

Go's `context.Context` carries cancellation and deadlines through a call chain. Its cancellation is cooperative: functions must observe it, and cancelling does not wait for them to stop. Rust has several choices, including a shutdown channel for blocking workers and Tokio's `CancellationToken` for async work. [Go Context](https://pkg.go.dev/context), [Tokio graceful shutdown](https://tokio.rs/tokio/topics/shutdown).

Here an empty channel acts as a shutdown signal. Dropping its only sender disconnects it for every worker:

```rust
use crossbeam_channel::{bounded, select_biased, tick, Receiver};
use std::{thread, time::Duration};

fn worker(id: usize, shutdown: Receiver<()>) {
    let timer = tick(Duration::from_millis(10));
    loop {
        select_biased! {
            recv(shutdown) -> _ => {
                println!("Worker {id} stopped");
                return;
            }
            recv(timer) -> _ => println!("Worker {id} checked for work"),
        }
    }
}

fn main() {
    let (shutdown_tx, shutdown_rx) = bounded::<()>(0);
    thread::scope(|scope| {
        for id in 0..3 {
            let shutdown = shutdown_rx.clone();
            scope.spawn(move || worker(id, shutdown));
        }
        drop(shutdown_rx);
        thread::sleep(Duration::from_millis(30)); // Demo time before closing.
        drop(shutdown_tx);
    }); // Waits for all worker closures; no extra shutdown sleep needed.
}
```

`select_biased!` checks ready branches in the listed order, so shutdown wins when both are ready. One sent `()` would reach one competing receiver; disconnection is what every worker can observe. [Crossbeam select_biased](https://docs.rs/crossbeam-channel/latest/crossbeam_channel/macro.select_biased.html).

If the work branch spends an hour in a blocking operation, this signal cannot interrupt that operation. Design shutdown in this order: stop admission, request cancellation, decide whether queued work is drained or abandoned, and wait for workers. “I asked it to stop” and “it has stopped” are different facts.

## Combining Tools: Rayon + Channels

A CPU stage may feed a sequential writer. The writer must be able to progress while Rayon workers are sending to a bounded queue:

```rust
use crossbeam_channel::bounded;
use rayon::prelude::*;
use std::thread;

fn main() {
    let inputs: Vec<u64> = (0..100).collect();
    let (tx, rx) = bounded(8);

    thread::scope(|scope| {
        scope.spawn(move || {
            inputs.par_iter().for_each(|&id| {
                tx.send((id, id * id)).expect("consumer should remain alive");
            });
        });

        let mut total = 0_u64;
        for (_id, square) in rx {
            total += square;
        }
        println!("Sum of squares: {total}"); // 328350
    });
}
```

The arithmetic is deliberately small; this illustrates ownership and coordination, not a workload needing parallelism. The consumer runs outside the Rayon pool. Blocking all pool workers while expecting a consumer queued in the same pool to rescue them can deadlock.

For a multi-stage pipeline, apply the same reasoning to every edge. Give each sender a clear owner, bound waiting work, and keep downstream stages progressing. Preserve IDs if you need to reconstruct input order. A pipeline is only as fast as its limiting stage; adding threads does not remove that bottleneck.

## Semaphores: A Permit Must Come Back

A semaphore limits how many operations may use a resource at once. Think of three coffee machines and three tokens. A worker must hold a token while using a machine.

The earlier ownership lesson gives us the useful design: return a guard that gives the token back when it drops. Requiring a separate manual `release()` is easy to get wrong on an early return or panic.

This small synchronous implementation is a teaching example. Its fields are private, callers cannot create or clone permits, and zero capacity is rejected.

```rust
mod permits {
    use crossbeam_channel::{bounded, Receiver, Sender};

    pub struct Semaphore {
        available: Receiver<()>,
        returned: Sender<()>,
    }

    pub struct Permit<'a> {
        semaphore: &'a Semaphore,
    }

    impl Semaphore {
        pub fn new(capacity: usize) -> Self {
            assert!(capacity > 0, "semaphore needs at least one permit");
            let (returned, available) = bounded(capacity);
            for _ in 0..capacity {
                returned.send(()).unwrap();
            }
            Self { available, returned }
        }

        pub fn acquire(&self) -> Permit<'_> {
            self.available.recv().expect("semaphore owns a sender");
            Permit { semaphore: self }
        }
    }

    impl Drop for Permit<'_> {
        fn drop(&mut self) {
            // Each live permit removed one token, so a slot must be free.
            // Never block or panic during cleanup if that invariant is broken.
            let _ = self.semaphore.returned.try_send(());
        }
    }
}

fn main() {
    let machines = permits::Semaphore::new(3);
    std::thread::scope(|scope| {
        for id in 0..6 {
            let machines = &machines;
            scope.spawn(move || {
                let _permit = machines.acquire();
                if id == 2 {
                    return; // The permit is returned on this path too.
                }
                println!("Worker {id} has a machine");
            });
        }
    });
}
```

Why is a slot available on drop? Initially there are three tokens. Each acquisition removes one and creates exactly one guard; each guard can be dropped only once. Other acquisitions remove more tokens, not the hole owed to this guard. The private API preserves that accounting.

The guard is called `_permit`, which is a real binding retained to the end of its scope. Writing `let _ = machines.acquire();` would discard and immediately drop the returned guard, releasing the permit before the work. This is the same underscore gotcha from the first post.

RAII handles normal scope exit, early returns, and panic unwinding. It does not guarantee cleanup after process abort or a deliberately leaked guard. This example also creates six OS threads and limits only the number holding permits; it does not bound thread creation. For async code, an established primitive such as [Tokio's Semaphore](https://docs.rs/tokio/latest/tokio/sync/struct.Semaphore.html) provides async acquisition and permit guards. Do not call this blocking `acquire` on a runtime worker.

## Shared State: Put the Whole Decision Under One Guard

Channels are not the only answer. If the data is a small shared counter or collection, a mutex may be simpler. The question is whether the entire invariant is protected.

Here two tills try to reserve the last pastry:

```rust
use std::{sync::Mutex, thread};

fn main() {
    let stock = Mutex::new(1_u32);
    thread::scope(|scope| {
        for till in 1..=2 {
            let stock = &stock;
            scope.spawn(move || {
                let reserved = {
                    let mut remaining = stock.lock().expect("stock mutex poisoned");
                    if *remaining == 0 {
                        false
                    } else {
                        *remaining -= 1;
                        true
                    }
                }; // Unlock before printing or doing unrelated work.
                println!("Till {till}: reserved = {reserved}");
            });
        }
    });
}
```

Exactly one reservation succeeds in a normal run; which till succeeds is unspecified. Scoped borrowing supplies the lifetime, so this example needs no `Arc`. For separately spawned owning threads, `Arc<Mutex<T>>` is a common combination.

The guard groups “is anything left?” with “reserve one.” Taking and releasing separate locks for those two steps could preserve data-race safety while breaking the reservation rule. A mutex does not provide rollback if work fails halfway through a larger operation.

Shared access goes through the mutex guard. With exclusive access to the mutex itself, APIs such as `get_mut` can provide access without runtime locking. Poisoning warns that a panic may have interrupted an invariant; it is not automatic repair or a memory-safety guarantee. Taking multiple locks also requires a consistent ordering to avoid deadlock. [Mutex](https://doc.rust-lang.org/std/sync/struct.Mutex.html).

For an independent statistic, an atomic counter may suffice. `fetch_add` is one atomic read-modify-write; a separate `load` and `store` is not equivalent. `Ordering::Relaxed` keeps that atomic operation indivisible but does not publish unrelated data. Even stronger ordering does not turn several operations into one transaction. [Atomic ordering](https://doc.rust-lang.org/std/sync/atomic/enum.Ordering.html).

## Async: Waiting Without Occupying a Worker Thread

Async is useful when operations spend time waiting and the libraries support it. There is no universal threshold of “thousands of connections,” and async Rust does not have weaker ownership guarantees than threaded Rust.

Calling an `async fn` produces a future. Its body runs when polled, normally by an executor through `.await`. Simply creating it does not spawn a task. `tokio::spawn` submits an independently scheduled task; directly awaiting two operations one after the other is sequential. [Rust Future](https://doc.rust-lang.org/std/future/trait.Future.html).

This example lets the café wait for three simulated pickup notifications. The timer stands in for waiting; it does not implement real network I/O.

```rust
use tokio::{task::JoinSet, time::{sleep, Duration}};

#[tokio::main]
async fn main() -> Result<(), tokio::task::JoinError> {
    let mut pickups = JoinSet::new();
    for id in 1..=3 {
        pickups.spawn(async move {
            sleep(Duration::from_millis(10)).await;
            format!("Order {id} is ready for pickup")
        });
    }

    while let Some(result) = pickups.join_next().await {
        println!("{}", result?);
    }
    Ok(())
}
```

`JoinSet` collects completion results. This demo returns on the first task error, dropping the set and requesting abortion of remaining tasks. A service needing completed cleanup must explicitly wait for shutdown. Dropping a plain Tokio `JoinHandle` instead detaches its task; it does not abort it. These are library lifecycle rules, not interchangeable names. [JoinSet](https://docs.rs/tokio/latest/tokio/task/struct.JoinSet.html), [JoinHandle](https://docs.rs/tokio/latest/tokio/task/struct.JoinHandle.html).

Five boundaries to learn early:

- Spawned Tokio tasks normally require `Send + 'static`; data retained across suspension matters. A local-task API can run non-`Send` futures, but does not make them thread-safe.
- `.await` is a possible suspension point, not a guarantee of yielding every time. A long calculation without yielding can occupy a worker.
- `std::thread::sleep`, blocking channel receives, and blocking I/O still block inside `async`. Use matching async APIs, or deliberately offload blocking work. A started `spawn_blocking` operation generally cannot be aborted; CPU-heavy use also needs a concurrency limit. [Blocking work](https://docs.rs/tokio/latest/tokio/task/fn.spawn_blocking.html).
- Keep a standard mutex guard in a short block with no `.await`. An async mutex allows holding its guard across `.await`, but holding a lock during slow external work still needs a reason. [Tokio shared state](https://tokio.rs/tokio/tutorial/shared-state).
- Cancellation is not rollback. Dropping a future can abandon its remaining work, but it does not undo a message already sent or a remote write already committed. Check whether an operation can safely be dropped and retried. A timeout also cannot preempt a future that keeps running without yielding. [Tokio timeout](https://docs.rs/tokio/latest/tokio/time/fn.timeout.html), [Select and cancellation safety](https://docs.rs/tokio/latest/tokio/macro.select.html#cancellation-safety).

The example has only three tasks. For a stream of requests, also bound admitted tasks and retained results; a semaphore acquired inside every task does not prevent an unbounded population of waiting tasks.

## Type States: A Protocol Handle Is Not a Promise From the Network

The previous post's pattern still helps: a `Connection<Connected>` can offer a `send` method, while `Connection<Closed>` does not. This prevents certain local call-order mistakes.

It cannot prove the peer is still connected when the next packet is sent. The peer can disappear after the type is created, so sending still needs a runtime result. Likewise, ownership can transfer a job into a worker; it cannot prove the worker's external operation succeeded.

Keep this distinction when we move on to atomicity: “no data race,” “legal API sequence,” and “the entire business operation succeeded” are three separate claims.

## When to Use What

| Your actual need | A useful starting point | First question to ask |
| :-- | :-- | :-- |
| A few parallel operations borrowing local data | `thread::scope` | Can all work finish within this scope? |
| An independent blocking worker | `thread::spawn` plus an owned handle | Who joins it and requests shutdown? |
| Splittable CPU-heavy collection work | Rayon | Is the work large enough to benefit? |
| A bounded queue for blocking workers | Standard or Crossbeam channels | Who drops each final handle? |
| Waiting on async I/O | Tokio tasks and async channels | What bounds admission and handles cancellation? |
| A shared in-process invariant | `Mutex<T>` or a single owner receiving commands | Which steps must happen together? |
| A limit on active resource users | A semaphore with guards | Are queued jobs and tasks bounded too? |

For WebAssembly, check the target and host. “WebAssembly has no threads” is too broad; Rust has threaded targets, while a particular browser or target may impose restrictions. [Threaded WASI target](https://doc.rust-lang.org/rustc/platform-support/wasm32-wasip1-threads.html).

## Performance: Measure the Actual Bottleneck

| Mechanism | Costs to account for | What it does not promise |
| :-- | :-- | :-- |
| OS threads | Stack address space, creation, scheduling, synchronization | One cheap worker per input |
| Rayon | Splitting, scheduling, memory bandwidth, combining results | Near-linear speedup on every workload |
| Channels | Queue storage, coordination, possible blocking, payload work | A universal nanosecond cost or lock-free progress |
| Async tasks | Future state, allocation where applicable, scheduling, I/O runtime | Bounded memory merely because tasks are lightweight |

Rust documents a current 2 MiB default stack for spawned threads on Tier-1 platforms, subject to change and configuration. Reserved stack space is not necessarily fully resident physical memory. Rayon pool size is configurable; CPU availability and workload determine whether more workers help. [Rust stack sizes](https://doc.rust-lang.org/std/thread/#stack-size), [Rayon pool configuration](https://docs.rs/rayon/latest/rayon/struct.ThreadPoolBuilder.html#method.num_threads).

Measure release builds with representative input, bounded load, and the actual I/O or CPU work. Record throughput, latency, memory, worker count, and queue capacity. A faster happy path that hangs during shutdown is not a successful optimization.

## Check Your Understanding

1. **Why does `iter_mut` work where repeated indexing fails?** Its contract provides distinct mutable references; the indexed loop does not establish that separation for the outstanding borrows.
2. **Does `move` make a borrowed string `'static`?** No. It moves or copies the reference, not its backing storage.
3. **Does `Arc` remove the need for synchronization?** No. It manages shared ownership. Mutation of the payload needs an appropriate design of its own.
4. **What if one unused sender remains alive?** A draining receiver may keep waiting because more messages are still possible.
5. **Does successful `send` mean successful processing?** No. The receiver may fail after acceptance; use a result protocol when completion matters.
6. **Can two bounded queues deadlock?** Yes, if every producer waits for space while no consumer can make progress.
7. **What does a send timeout bound?** Waiting to enter the queue, not the job's runtime or shutdown time.
8. **Why `_permit` instead of `_`?** The named guard stays alive through the work; a wildcard discard drops the returned guard immediately.
9. **Can memory-safe code sell the last pastry twice?** Yes, if checking and reserving are separate synchronized steps. Protect the whole invariant.
10. **Does cancellation undo a payment or stop arbitrary blocking work?** No. Cancellation needs observation, and external effects need their own recovery rules.

For practice, turn the worker-pool example into a café label processor: reject invalid orders explicitly, keep both queues bounded, preserve order IDs, and explain what happens if the result consumer leaves early. Before adding any code, trace who owns every sender, receiver, job, and worker handle.

That ownership trace is the bridge to the next post, [atomicity and isolation with channels and mutexes](/rust/concepts/2025/11/14/rust-acid-concurrency.html): memory-safe access is the foundation; a correct multi-step operation needs its own design.
