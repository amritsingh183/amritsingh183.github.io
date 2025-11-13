---
layout: post
title: "Rust Concurrency and Parallelism"
date: 2025-11-12 16:36:00 +0530
categories: rust concepts
last_updated: 2025-11-12
---

# Rust Concurrency for Go Developers: Achieving Full Compile-Time Safety

When transitioning from Go to Rust, you'll discover a fundamental shift in how concurrency safety is achieved. While Go relies on runtime checks and developer discipline, Rust can guarantee memory safety and data-race freedom at compile time—before your code ever runs. This guide focuses exclusively on Rust patterns that provide 100% compile-time verification, showing Go developers how to write concurrent code that simply cannot fail in ways that Go code might.

## The Philosophy: Runtime Trust vs Compile-Time Proof

Go trusts you to handle concurrency correctly. You get powerful tools—goroutines, channels, mutexes—and the language expects you to use them wisely. Race detectors and careful testing help catch mistakes.

Rust takes a different approach: if concurrent code compiles, it's memory-safe and free from data races. This isn't a linter warning or an optional check—it's enforced by the type system itself.

Consider this Go code that compiles but contains a race condition:

```go
// Go - Compiles fine, race condition at runtime
var counter int

func main() {
    for i := 0; i < 1000; i++ {
        go func() {
            counter++ // DATA RACE!
        }()
    }
    time.Sleep(time.Second)
    fmt.Println(counter) // Unpredictable result
}
```

The Rust equivalent won't compile:

```rust
// Rust - Compilation error prevents the race
fn main() {
    let mut counter = 0;
    for _ in 0..1000 {
        std::thread::spawn(|| {
            counter += 1; // ERROR: cannot borrow `counter` as mutable
        });
    }
}
```

This guide will show you how to leverage Rust's compile-time guarantees to write concurrent code that's not just safe, but provably safe.

## Core Tool #1: Scoped Threads for Guaranteed Completion

The foundation of compile-time safe concurrency in Rust is `std::thread::scope`. Unlike Go's WaitGroups which require manual tracking, scoped threads guarantee at compile time that all spawned threads complete before the scope exits.

### Basic Scoped Thread Usage

```rust
use std::thread;

fn main() {
    let data = vec![1, 2, 3, 4, 5];
    let mut results = vec![];
    
    thread::scope(|s| {
        for &item in &data {
            s.spawn(move || {
                println!("Processing {}", item);
                item * 2
            });
        }
    }); // All threads guaranteed complete here
    
    println!("All threads completed!");
}
```

### Borrowing Stack Data Safely

The killer feature of scoped threads is safe borrowing of stack data without any runtime overhead:

```rust
use std::thread;

fn process_data(input: &str) -> String {
    input.to_uppercase()
}

fn parallel_processing() {
    let data = vec!["hello", "world", "from", "rust"];
    let mut results = vec![String::new(); data.len()];
    
    thread::scope(|s| {
        for (i, item) in data.iter().enumerate() {
            let result_ref = &mut results[i];
            s.spawn(move || {
                *result_ref = process_data(item);
            });
        }
    });
    
    println!("Results: {:?}", results);
}

fn main() {
    parallel_processing();
}
```

### Replacing Go's WaitGroup Pattern

Here's how common Go patterns translate to Rust with compile-time safety:

**Go WaitGroup:**
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

**Rust Scoped Threads:**
```rust
use std::thread;

fn process(item: &str) {
    println!("Processing: {}", item);
}

fn main() {
    let items = vec!["a", "b", "c", "d"];
    
    thread::scope(|s| {
        for item in &items {
            s.spawn(move || {
                process(item);
            });
        }
    }); // Automatic wait, can't forget!
}
```

The Rust version is impossible to get wrong—you can't forget to call `Done()`, can't have mismatched Add/Done counts, and the compiler ensures all threads complete.

## Core Tool #2: Rayon for Data-Parallel Processing

When you need to process collections in parallel, Rayon provides zero-cost abstractions with full compile-time safety. It automatically manages thread pools and work-stealing without any manual synchronization.

### Basic Parallel Iterator

```rust
use rayon::prelude::*;

fn expensive_computation(n: i32) -> i32 {
    // Simulate expensive work
    std::thread::sleep(std::time::Duration::from_millis(10));
    n * n
}

fn main() {
    let numbers: Vec<i32> = (0..100).collect();
    
    // Sequential version
    let sequential: Vec<i32> = numbers
        .iter()
        .map(|&n| expensive_computation(n))
        .collect();
    
    // Parallel version - same API, automatic parallelization
    let parallel: Vec<i32> = numbers
        .par_iter()
        .map(|&n| expensive_computation(n))
        .collect();
    
    assert_eq!(sequential, parallel);
    println!("Processed {} items", parallel.len());
}
```

### Parallel Reduction and Aggregation

```rust
use rayon::prelude::*;

fn main() {
    let data: Vec<f64> = (0..1_000_000).map(|i| i as f64).collect();
    
    // Parallel sum
    let sum: f64 = data.par_iter().sum();
    
    // Parallel filter and collect
    let filtered: Vec<f64> = data
        .par_iter()
        .filter(|&&x| x % 2.0 == 0.0)
        .copied()
        .collect();
    
    // Parallel fold for custom aggregation
    let product = data
        .par_iter()
        .take(10)
        .product::<f64>();
    
    println!("Sum: {}, Filtered count: {}, Product: {}", 
             sum, filtered.len(), product);
}
```

### Complex Transformations with Guaranteed Safety

```rust
use rayon::prelude::*;

#[derive(Debug, Clone)]
struct Record {
    id: u32,
    value: f64,
}

fn validate(record: &Record) -> bool {
    record.value > 0.0
}

fn transform(record: Record) -> Record {
    Record {
        id: record.id,
        value: record.value * 2.0,
    }
}

fn main() {
    let records: Vec<Record> = (0..1000)
        .map(|i| Record {
            id: i,
            value: (i as f64) - 500.0,
        })
        .collect();
    
    let processed: Vec<Record> = records
        .into_par_iter()
        .filter(validate)
        .map(transform)
        .collect();
    
    println!("Processed {} valid records", processed.len());
}
```

Rayon guarantees data-race freedom through Rust's type system. If your code compiles, parallel execution is safe.

## Core Tool #3: Crossbeam Channels for Type-Safe Communication

Crossbeam provides compile-time safe channels without requiring runtime reference counting. These channels integrate perfectly with scoped threads.

### Basic Channel Usage

```rust
use crossbeam::channel;
use std::thread;

fn main() {
    let (sender, receiver) = channel::bounded(10);
    
    thread::scope(|s| {
        // Producer
        s.spawn(move || {
            for i in 0..5 {
                sender.send(i).unwrap();
                println!("Sent: {}", i);
            }
            // Channel closes when sender is dropped
        });
        
        // Consumer
        s.spawn(move || {
            for msg in receiver {
                println!("Received: {}", msg);
            }
        });
    });
}
```

### The Unit Type for Signaling

Go uses `struct{}` for signal-only channels. Rust uses `()`, the unit type—a true zero-sized type:

```rust
use crossbeam::channel;
use std::thread;
use std::time::Duration;

fn main() {
    // Signal-only channel
    let (signal_tx, signal_rx) = channel::bounded::<()>(0);
    
    thread::scope(|s| {
        s.spawn(move || {
            println!("Worker: Starting work...");
            thread::sleep(Duration::from_secs(1));
            println!("Worker: Work complete!");
            signal_tx.send(()).unwrap();
        });
        
        println!("Main: Waiting for worker...");
        signal_rx.recv().unwrap();
        println!("Main: Worker finished!");
    });
}
```

### Select for Multiple Channels

```rust
use crossbeam::channel::{self, after, tick};
use std::time::Duration;

fn main() {
    let (tx1, rx1) = channel::unbounded::<i32>();
    let (tx2, rx2) = channel::unbounded::<String>();
    let ticker = tick(Duration::from_millis(500));
    let timeout = after(Duration::from_secs(3));
    
    thread::scope(|s| {
        s.spawn(move || {
            thread::sleep(Duration::from_millis(300));
            tx1.send(42).unwrap();
        });
        
        s.spawn(move || {
            thread::sleep(Duration::from_millis(700));
            tx2.send("Hello".to_string()).unwrap();
        });
        
        loop {
            crossbeam::select! {
                recv(rx1) -> msg => {
                    println!("Got number: {:?}", msg);
                }
                recv(rx2) -> msg => {
                    println!("Got string: {:?}", msg);
                }
                recv(ticker) -> _ => {
                    println!("Tick!");
                }
                recv(timeout) -> _ => {
                    println!("Timeout reached, exiting");
                    break;
                }
            }
        }
    });
}
```

## Pattern Translations: Go to Rust

### Pattern 1: Producer-Consumer Pipeline

**Go Version:**
```go
func pipeline() {
    ch1 := make(chan int, 100)
    ch2 := make(chan int, 100)
    
    // Producer
    go func() {
        for i := 0; i < 10; i++ {
            ch1 <- i
        }
        close(ch1)
    }()
    
    // Transformer
    go func() {
        for val := range ch1 {
            ch2 <- val * 2
        }
        close(ch2)
    }()
    
    // Consumer
    for val := range ch2 {
        fmt.Println(val)
    }
}
```

**Rust Version with Compile-Time Safety:**
```rust
use crossbeam::channel;
use std::thread;

fn main() {
    let (tx1, rx1) = channel::bounded(100);
    let (tx2, rx2) = channel::bounded(100);
    
    thread::scope(|s| {
        // Producer
        s.spawn(move || {
            for i in 0..10 {
                tx1.send(i).unwrap();
            }
            // tx1 drops, closing channel
        });
        
        // Transformer
        s.spawn(move || {
            for val in rx1 {
                tx2.send(val * 2).unwrap();
            }
            // tx2 drops, closing channel
        });
        
        // Consumer
        for val in rx2 {
            println!("{}", val);
        }
    });
}
```

### Pattern 2: Worker Pool

**Rust Implementation:**
```rust
use crossbeam::channel;
use std::thread;

fn process_job(id: usize, job: i32) -> String {
    format!("Worker {} processed job {}", id, job)
}

fn main() {
    let (job_tx, job_rx) = channel::bounded(100);
    let (result_tx, result_rx) = channel::unbounded();
    
    thread::scope(|s| {
        // Spawn workers
        for worker_id in 0..4 {
            let job_rx = job_rx.clone();
            let result_tx = result_tx.clone();
            
            s.spawn(move || {
                for job in job_rx {
                    let result = process_job(worker_id, job);
                    result_tx.send(result).unwrap();
                }
            });
        }
        
        // Submit jobs
        s.spawn(move || {
            for job in 0..20 {
                job_tx.send(job).unwrap();
            }
            // job_tx drops, signaling no more jobs
        });
        
        // Drop our result_tx so receiver knows when done
        drop(result_tx);
        
        // Collect results
        for result in result_rx {
            println!("{}", result);
        }
    });
}
```

### Pattern 3: Cancellation Without Context

Instead of Go's context.Context, use channel-based cancellation:

```rust
use crossbeam::channel::{self, Receiver};
use std::thread;
use std::time::Duration;

fn worker(id: usize, shutdown: Receiver<()>) {
    let ticker = crossbeam::channel::tick(Duration::from_millis(500));
    
    loop {
        crossbeam::select! {
            recv(shutdown) -> _ => {
                println!("Worker {} shutting down", id);
                return;
            }
            recv(ticker) -> _ => {
                println!("Worker {} working...", id);
            }
        }
    }
}

fn main() {
    let (shutdown_tx, shutdown_rx) = channel::bounded(0);
    
    thread::scope(|s| {
        // Spawn workers
        for i in 0..3 {
            let shutdown = shutdown_rx.clone();
            s.spawn(move || worker(i, shutdown));
        }
        
        // Let them work
        thread::sleep(Duration::from_secs(2));
        
        // Shutdown
        println!("Initiating shutdown...");
        drop(shutdown_tx); // Closing channel signals shutdown
        
        // Give time to see shutdown messages
        thread::sleep(Duration::from_millis(100));
    });
}
```

## Advanced Patterns with Compile-Time Guarantees

### Combining Rayon with Channels

```rust
use rayon::prelude::*;
use crossbeam::channel;
use std::thread;

fn expensive_transform(input: i32) -> i32 {
    std::thread::sleep(std::time::Duration::from_millis(1));
    input * input
}

fn post_process(value: i32) -> String {
    format!("Result: {}", value)
}

fn main() {
    let data: Vec<i32> = (0..100).collect();
    let (tx, rx) = channel::bounded(50);
    
    thread::scope(|s| {
        // Parallel processing with Rayon
        let tx_clone = tx.clone();
        s.spawn(move || {
            data.par_iter().for_each(|&item| {
                let result = expensive_transform(item);
                tx_clone.send(result).unwrap();
            });
        });
        drop(tx); // Signal completion
        
        // Sequential post-processing
        s.spawn(move || {
            let results: Vec<String> = rx
                .iter()
                .map(post_process)
                .collect();
            println!("Processed {} items", results.len());
        });
    });
}
```

### Parallel Pipeline with Multiple Stages

```rust
use rayon::prelude::*;
use crossbeam::channel;
use std::thread;

fn stage1(input: &str) -> String {
    input.to_uppercase()
}

fn stage2(input: String) -> usize {
    input.len()
}

fn stage3(lengths: Vec<usize>) -> usize {
    lengths.iter().sum()
}

fn main() {
    let inputs = vec!["hello", "world", "from", "rust", "concurrency"];
    let (tx1, rx1) = channel::bounded(10);
    let (tx2, rx2) = channel::bounded(10);
    
    thread::scope(|s| {
        // Stage 1: Parallel string processing
        let inputs_clone = inputs.clone();
        s.spawn(move || {
            inputs_clone
                .par_iter()
                .map(|s| stage1(s))
                .for_each(|result| {
                    tx1.send(result).unwrap();
                });
        });
        
        // Stage 2: Length calculation
        s.spawn(move || {
            for item in rx1 {
                let length = stage2(item);
                tx2.send(length).unwrap();
            }
        });
        
        // Stage 3: Aggregation
        let handle = s.spawn(move || {
            let lengths: Vec<usize> = rx2.iter().collect();
            stage3(lengths)
        });
        
        let total = handle.join().unwrap();
        println!("Total length: {}", total);
    });
}
```

## Semaphore Pattern with Channels

```rust
use crossbeam::channel;
use std::thread;
use std::time::Duration;

struct Semaphore {
    permits: channel::Sender<()>,
    acquire: channel::Receiver<()>,
}

impl Semaphore {
    fn new(capacity: usize) -> Self {
        let (tx, rx) = channel::bounded(capacity);
        // Fill with permits
        for _ in 0..capacity {
            tx.send(()).unwrap();
        }
        Semaphore {
            permits: tx,
            acquire: rx,
        }
    }
    
    fn acquire(&self) {
        self.acquire.recv().unwrap();
    }
    
    fn release(&self) {
        self.permits.send(()).unwrap();
    }
}

fn do_work(id: usize) {
    println!("Worker {} starting", id);
    thread::sleep(Duration::from_millis(500));
    println!("Worker {} done", id);
}

fn main() {
    let sem = Semaphore::new(3);
    
    thread::scope(|s| {
        for i in 0..10 {
            let sem_ref = &sem;
            s.spawn(move || {
                sem_ref.acquire();
                do_work(i);
                sem_ref.release();
            });
        }
    });
    
    println!("All workers complete!");
}
```

## Type-State Pattern for Compile-Time Protocol Enforcement

This pattern uses Rust's type system to enforce state machines at compile time:

```rust
use std::marker::PhantomData;

// States (zero runtime cost)
struct Initialized;
struct Running;
struct Stopped;

// State machine that enforces correct transitions
struct StateMachine<State> {
    data: String,
    _state: PhantomData<State>,
}

impl StateMachine<Initialized> {
    fn new(data: String) -> Self {
        StateMachine {
            data,
            _state: PhantomData,
        }
    }
    
    fn start(self) -> StateMachine<Running> {
        println!("Starting with: {}", self.data);
        StateMachine {
            data: self.data,
            _state: PhantomData,
        }
    }
}

impl StateMachine<Running> {
    fn process(&mut self) {
        println!("Processing: {}", self.data);
        self.data.push_str(" [processed]");
    }
    
    fn stop(self) -> StateMachine<Stopped> {
        println!("Stopping...");
        StateMachine {
            data: self.data,
            _state: PhantomData,
        }
    }
}

impl StateMachine<Stopped> {
    fn get_result(self) -> String {
        self.data
    }
}

fn main() {
    let machine = StateMachine::new("Hello".to_string());
    
    // This is the ONLY valid sequence
    let mut running = machine.start();
    running.process();
    running.process();
    let stopped = running.stop();
    let result = stopped.get_result();
    
    println!("Final result: {}", result);
    
    // These would NOT compile:
    // machine.process();     // Error: no method `process` for Initialized
    // stopped.start();       // Error: no method `start` for Stopped
    // running.get_result();  // Error: no method `get_result` for Running
}
```

## Decision Framework: When to Use What

### Use `thread::scope` when:
- You need guaranteed thread completion
- You want to borrow stack data safely
- You're replacing Go's WaitGroup patterns
- You need fine-grained control over thread lifecycle

### Use Rayon when:
- Processing large collections in parallel
- CPU-bound computations that can be parallelized
- You want automatic work-stealing and load balancing
- Iterator-based transformations

### Use Crossbeam channels when:
- Building producer-consumer pipelines
- Implementing message-passing architectures
- Need select-like behavior across multiple channels
- Creating custom synchronization primitives

### Avoid async unless:
- You genuinely need thousands of concurrent I/O operations
- Working in WebAssembly where threads aren't available
- Interfacing with async-only libraries

## Performance Characteristics

| Operation | Scoped Threads | Rayon | Crossbeam Channels |
|-----------|---------------|-------|-------------------|
| Overhead | ~2MB stack/thread | Work-stealing pool | ~20ns/send |
| Scaling | Up to core count | Near-linear | Lock-free |
| Best for | Known parallelism | Data parallelism | Message passing |
| Safety | 100% compile-time | 100% compile-time | 100% compile-time |

## Common Pitfalls to Avoid

### 1. Creating Too Many OS Threads

```rust
// ❌ Bad: Creating thread per item
fn bad_example(items: Vec<i32>) {
    for item in items {
        thread::spawn(move || {
            process_item(item);
        });
    }
}

// ✅ Good: Use Rayon for automatic pooling
fn good_example(items: Vec<i32>) {
    items.par_iter().for_each(|&item| {
        process_item(item);
    });
}

fn process_item(item: i32) {
    println!("Processing: {}", item);
}
```

### 2. Forgetting Channel Capacity

```rust
// ❌ Bad: Unbounded channel can cause memory issues
fn bad_channel() {
    let (tx, rx) = crossbeam::channel::unbounded();
    // Sender could outpace receiver
}

// ✅ Good: Bounded channel provides backpressure
fn good_channel() {
    let (tx, rx) = crossbeam::channel::bounded(100);
    // Sender blocks when channel is full
}
```

### 3. Not Leveraging Rayon for Collections

```rust
// ❌ Bad: Manual thread management
fn manual_parallel(data: Vec<i32>) -> Vec<i32> {
    let mut handles = vec![];
    for chunk in data.chunks(100) {
        let chunk = chunk.to_vec();
        handles.push(thread::spawn(move || {
            chunk.iter().map(|x| x * 2).collect::<Vec<_>>()
        }));
    }
    handles.into_iter()
        .flat_map(|h| h.join().unwrap())
        .collect()
}

// ✅ Good: Let Rayon handle it
fn rayon_parallel(data: Vec<i32>) -> Vec<i32> {
    data.par_iter().map(|x| x * 2).collect()
}
```

## Quick Reference Table

| Go Pattern | Rust Equivalent | Compile-Time Safety |
|-----------|-----------------|---------------------|
| `go func(){}` | `thread::scope()` | ✅ Lifetime checked |
| `sync.WaitGroup` | `thread::scope()` | ✅ Automatic completion |
| `chan T` | `crossbeam::channel` | ✅ Type-safe |
| `select {}` | `crossbeam::select!` | ✅ Exhaustiveness checked |
| `context.Context` | Channel-based cancellation | ✅ Drop = cancel |
| `struct{}` | `()` unit type | ✅ Zero-sized |
| Manual worker pools | Rayon parallel iterators | ✅ Automatic |

## Conclusion

Rust's approach to concurrency isn't just about safety—it's about proving safety at compile time. By using scoped threads, Rayon, and Crossbeam channels, you can write concurrent code that:

1. **Cannot have data races** - The compiler prevents them
2. **Cannot leak threads** - Scopes guarantee completion
3. **Cannot misuse protocols** - Type states enforce correct usage
4. **Cannot forget cleanup** - RAII handles it automatically

The patterns shown here provide the same capabilities as Go's concurrency primitives, but with compile-time guarantees instead of runtime hopes. Yes, the learning curve is steeper, but the payoff is code that simply cannot fail in ways that Go code might.

Start with `thread::scope` for basic concurrency, add Rayon for parallel data processing, and use Crossbeam for channel-based communication. With these three tools, you can handle virtually any concurrent workload while maintaining 100% compile-time safety.

Remember: if your concurrent Rust code compiles, it's memory-safe and free from data races. That's not a best practice or a guideline—it's a mathematical guarantee from the type system.