---
layout: post
title: "Rust Concurrency explained for Go (Golang) Developers"
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
            counter++ // DATA RACE! Multiple goroutines write simultaneously
        }()
    }
    time.Sleep(time.Second)
    fmt.Println(counter) // Unpredictable result (rarely 1000)
}
```

The Rust equivalent won't compile:

```rust
// Rust - Compilation error prevents the race
fn main() {
    let mut counter = 0;
    for _ in 0..1000 {
        std::thread::spawn(|| {
            counter += 1; 
            // ERROR: cannot borrow `counter` as mutable
            // 
            // REASON: Each spawned thread would need its own mutable reference
            // to `counter`. Rust's borrowing rules prevent multiple mutable 
            // references to the same data because:
            // 1. Multiple writers = data races (undefined behavior)
            // 2. thread::spawn requires 'static lifetime (thread may outlive scope)
            // 3. Even with one writer, readers could see torn writes
            //
            // This compile error forces you to use proper synchronization
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
        // The scope 's' has a lifetime tied to this block
        // Any thread spawned through 's' MUST complete before scope ends
        
        for &item in &data {
            s.spawn(move || {
                // 'move' copies the value 'item' (i32 implements Copy)
                // The thread owns this copy, no sharing needed
                println!("Processing {}", item);
                item * 2
            });
        }
    }); // Compiler enforces: all threads join here before data/results can be used
    
    // SAFETY: We can use 'data' and 'results' here because Rust guarantees
    // all spawned threads have completed. This is checked at compile time!
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
        // CRITICAL: This loop demonstrates why scoped threads are powerful
        for (i, item) in data.iter().enumerate() {
            // We're borrowing 'data' (immutable) and 'results' (mutable)
            // from the outer scope - this would be IMPOSSIBLE with thread::spawn
            
            let result_ref = &mut results[i];
            // SAFETY: Each thread gets a unique mutable reference to a different
            // element of 'results'. No two threads access the same index, so
            // no data races are possible. Rust verifies this because we're
            // splitting the mutable borrow at compile time via indexing.
            
            s.spawn(move || {
                // 'move' transfers ownership of 'result_ref' to this thread
                // 'item' is &str, also moved (but it's Copy-like for &str)
                *result_ref = process_data(item);
                // When thread ends, 'result_ref' borrow ends
            });
        }
        // The scope GUARANTEES all threads complete here
        // This is why we can safely borrow stack data - Rust knows
        // the borrows won't outlive the borrowed data
    });
    
    // SAFETY: All mutable borrows have ended (threads completed)
    // We can use 'results' again with full ownership
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
    wg.Add(1)  // Must remember to Add before spawn
    go func(s string) {
        defer wg.Done()  // Must remember Done, or deadlock!
        process(s)
    }(item)
}
wg.Wait()  // Blocks until all Done() calls match Add() calls
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
        // The scope 's' lifetime is bounded by this block
        for item in &items {
            s.spawn(move || {
                // No Add() needed - thread registration is automatic
                process(item);
                // No Done() needed - thread completion is tracked by scope
                // No defer needed - RAII handles cleanup
            });
        }
    }); // COMPILER GUARANTEE: All spawned threads have joined here
        // This is enforced by Rust's lifetime system:
        // - The scope 's' cannot outlive this block
        // - Threads spawned on 's' cannot outlive 's'
        // - Therefore, threads cannot outlive this block
        // This is mathematically proven at compile time!
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
        .par_iter()  // Convert to parallel iterator
        .map(|&n| expensive_computation(n))  // Each closure gets exclusive access to its item
        // SAFETY: Rayon ensures:
        // 1. Each item is processed by exactly one thread
        // 2. No two threads process the same item
        // 3. The closure receives an immutable reference - no mutation possible
        // 4. Results are collected in the correct order despite parallel execution
        // This is enforced by Rayon's type system and the ParallelIterator trait
        .collect();
    
    assert_eq!(sequential, parallel);  // Order preserved!
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
    // SAFETY: Sum is associative and commutative, so parallel reduction is safe
    // Rayon splits work across threads and combines results correctly
    
    // Parallel filter and collect
    let filtered: Vec<f64> = data
        .par_iter()
        .filter(|&&x| x % 2.0 == 0.0)  // Predicate is pure - no side effects
        .copied()  // Creates owned values from references
        .collect();
    // SAFETY: Each element is independently evaluated
    // No shared mutable state between filter evaluations
    
    // Parallel fold for custom aggregation
    let product = data
        .par_iter()
        .take(10)
        .product::<f64>();
    // SAFETY: Product operation must be associative
    // Rayon may compute (a*b)*(c*d) or a*(b*c*d) but result is the same
    
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
    // BOUNDED CHANNEL: Can hold max 10 items
    // - Sender blocks when full (backpressure)
    // - Prevents unbounded memory growth
    
    thread::scope(|s| {
        // Producer thread
        s.spawn(move || {
            // 'move' transfers ownership of 'sender' to this thread
            // No other thread can use this sender now (enforced at compile time)
            
            for i in 0..5 {
                sender.send(i).unwrap();
                // send() can fail only if receiver is dropped (channel closed)
                println!("Sent: {}", i);
            }
            // CRITICAL: When 'sender' goes out of scope here, it's dropped
            // Dropping the last Sender automatically closes the channel
            // This signals the receiver that no more data will come
        });
        
        // Consumer thread  
        s.spawn(move || {
            // 'move' transfers ownership of 'receiver' to this thread
            // Rust's ownership system ensures only one thread can receive
            
            for msg in receiver {
                // Iterator ends when channel is closed (all senders dropped)
                // This is RAII in action - resource cleanup is automatic
                println!("Received: {}", msg);
            }
            // Loop exits automatically when channel closes - no manual checking!
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
    // Signal-only channel using unit type
    // () is zero-sized at runtime - no memory overhead for the signal itself
    let (signal_tx, signal_rx) = channel::bounded::<()>(0);
    // Bounded(0) = "rendezvous channel" - sender blocks until receiver is ready
    
    thread::scope(|s| {
        s.spawn(move || {
            println!("Worker: Starting work...");
            thread::sleep(Duration::from_secs(1));
            println!("Worker: Work complete!");
            
            // Send unit value as signal - contains no data, just notification
            signal_tx.send(()).unwrap();
            // Compiler optimizes this - no actual data is moved, just synchronization
        });
        
        println!("Main: Waiting for worker...");
        signal_rx.recv().unwrap();
        // Blocks until signal received - thread-safe synchronization
        // The type system ensures we can't accidentally try to extract data
        // from this signal (it's just (), not a struct with fields)
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
    let ticker = tick(Duration::from_millis(500));  // Periodic timer channel
    let timeout = after(Duration::from_secs(3));    // One-shot timer channel
    
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
            // SELECT: Check multiple channels, execute first ready branch
            crossbeam::select! {
                // Each branch has the pattern: recv(channel) -> result => action
                recv(rx1) -> msg => {
                    // Type safety: msg is Result<i32, RecvError>
                    // Compiler ensures we handle the right type
                    println!("Got number: {:?}", msg);
                }
                recv(rx2) -> msg => {
                    // Type safety: msg is Result<String, RecvError>
                    // Can't accidentally treat this as an i32
                    println!("Got string: {:?}", msg);
                }
                recv(ticker) -> _ => {
                    // Ticker fires every 500ms
                    // We ignore the value (it's just Instant)
                    println!("Tick!");
                }
                recv(timeout) -> _ => {
                    // This branch executes exactly once after 3 seconds
                    // Then breaks the loop
                    println!("Timeout reached, exiting");
                    break;
                }
                // No default branch = blocks until a channel is ready
                // With default => {...} it would be non-blocking
            }
            // SELECT GUARANTEES:
            // 1. Exactly one branch executes per iteration
            // 2. Fair selection - no channel is starved
            // 3. Type-safe - each channel's type is preserved
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
        close(ch1)  // Must remember to close
    }()
    
    // Transformer  
    go func() {
        for val := range ch1 {
            ch2 <- val * 2
        }
        close(ch2)  // Must remember to close
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
        // Producer thread
        s.spawn(move || {
            // OWNERSHIP: tx1 is moved into this thread
            // No other code can use tx1 now (compiler enforced)
            
            for i in 0..10 {
                tx1.send(i).unwrap();
            }
            // CRITICAL: tx1 is dropped here when it goes out of scope
            // Rust's RAII means dropping the last sender closes the channel
            // You CANNOT forget to close - it's automatic!
            // This is superior to Go's manual close() which can be forgotten
        });
        
        // Transformer thread
        s.spawn(move || {
            // rx1 is moved here - exclusive ownership
            // tx2 is also moved - this thread owns both
            
            for val in rx1 {
                // Iterator automatically stops when channel closes
                // No need to check for closed channel manually
                tx2.send(val * 2).unwrap();
            }
            // tx2 drops here, automatically closing second channel
            // Chain of closures is guaranteed by ownership system
        });
        
        // Consumer runs in scope
        for val in rx2 {
            // Type safety: val is guaranteed to be i32
            // Iterator ends when all tx2 senders are dropped
            println!("{}", val);
        }
        // SAFETY: Scope ensures all threads complete before exiting
        // No goroutine leaks possible - compiler enforced!
    });
}
```

### Pattern 2: Worker Pool

> Example 1

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
        // Spawn worker threads
        for worker_id in 0..4 {
            // Clone receivers/senders BEFORE moving into thread
            // Each worker gets its own copy of the channel endpoints
            let job_rx = job_rx.clone();
            let result_tx = result_tx.clone();
            
            s.spawn(move || {
                // WORK STEALING: Multiple workers share the same job_rx
                // Crossbeam ensures thread-safe access - only one worker
                // gets each job (atomic operation under the hood)
                
                for job in job_rx {
                    // This loop continues until ALL job_tx senders are dropped
                    // Each iteration atomically takes one job from the queue
                    let result = process_job(worker_id, job);
                    result_tx.send(result).unwrap();
                    // Results can arrive in any order - concurrent execution
                }
                // When job_rx channel closes, worker exits cleanly
                // No need for "poison pill" or shutdown signal
            });
        }
        
        // Submit jobs from another thread
        s.spawn(move || {
            for job in 0..20 {
                job_tx.send(job).unwrap();
                // Blocks if channel is full (bounded channel)
                // This provides natural backpressure
            }
            // job_tx dropped here - this is the LAST sender
            // Channel closes, signaling workers to finish
        });
        
        // CRITICAL: Drop our copy of result_tx
        // Otherwise result_rx will never close (we'd still have a sender)
        drop(result_tx);
        
        // Collect all results
        for result in result_rx {
            // Loop ends when:
            // 1. All workers finish (they drop their result_tx copies)
            // 2. Our result_tx was already dropped above
            // Therefore all senders are gone and channel closes
            println!("{}", result);
        }
        // Scope guarantees all workers have exited here
        // No thread leaks possible!
    });
}
```

> Example 2: Worker Pool with Multiple Producers and Deadlines

The basic worker pool pattern is great for simple scenarios, but production systems often need additional features:
- **Multiple concurrent producers** creating work items
- **Deadline-based submission** to prevent indefinite blocking
- **Explicit timeout handling** to distinguish between different failure modes
- **Result aggregation** with success/error tracking

```rust
use crossbeam::channel::{bounded, Receiver, SendTimeoutError, Sender};
use std::time::{Duration, Instant};

/// Represents a unit of work to be processed by the worker pool
#[derive(Debug)]
struct Job {
    id: usize,
    data: String,
}

impl Job {
    fn new(id: usize, data: String) -> Self {
        Job { id, data }
    }
}

/// Type alias for results returned by workers
/// Ok variant contains the processed result, Err contains error message
type JobResult = Result<String, String>;

/// Worker function that processes jobs from a channel
/// Each worker continuously receives jobs until the channel is closed
fn handle_job(job_rx: Receiver<Job>, result_tx: Sender<JobResult>) {
    for job in job_rx {
        // Simulate some processing work
        std::thread::sleep(Duration::from_millis(10));
        
        // Process the job and send result
        let result = Ok(format!("Processed job {} with data: {}", job.id, job.data));
        
        // If result channel is closed, we can't send - worker should exit
        if result_tx.send(result).is_err() {
            eprintln!("Result channel closed, worker exiting");
            break;
        }
    }
}

fn main() {
    // Create bounded channels for results and jobs
    // Bounded channels provide backpressure when full
    let (result_tx, result_rx) = bounded::<JobResult>(10);
    let (job_tx, job_rx) = bounded::<Job>(10);
    
    // Set a deadline for all job submissions
    // After this deadline, producers will timeout instead of blocking
    let deadline = Instant::now() + Duration::from_secs(2);

    crossbeam::thread::scope(|s| {
        // === WORKER THREADS ===
        // Spawn 4 worker threads that share the same job_rx
        // Crossbeam ensures thread-safe access - only one worker gets each job
        for worker_id in 0..4 {
            let worker_job_rx = job_rx.clone();
            let worker_result_tx = result_tx.clone();
            
            s.spawn(move |_| {
                println!("Worker {} started", worker_id);
                handle_job(worker_job_rx, worker_result_tx);
                println!("Worker {} finished", worker_id);
            });
        }
        
        // CRITICAL: Drop the original handles so only workers have them
        // When workers finish and drop their clones, channels will close
        drop(job_rx);
        drop(result_tx);

        // === PRODUCER THREADS ===
        // Spawn multiple producers that create jobs concurrently
        // This demonstrates handling many producers with deadline constraints
        for idx in 0..=100 {
            let producer_job_tx = job_tx.clone();
            
            s.spawn(move |_| {
                let new_job = Job::new(idx, format!("data-{}", idx));
                
                // Try to send with deadline - will timeout if channel is full
                // or if the deadline has passed
                match producer_job_tx.send_deadline(new_job, deadline) {
                    Ok(_) => {
                        // Job successfully queued
                    }
                    Err(SendTimeoutError::Timeout(_job)) => {
                        // Deadline passed or channel full for too long
                        eprintln!("Producer {}: Job timed out", idx);
                    }
                    Err(SendTimeoutError::Disconnected(_job)) => {
                        // All receivers (workers) have been dropped
                        eprintln!("Producer {}: Channel disconnected", idx);
                    }
                }
            });
        }
        
        // CRITICAL: Drop the original job_tx
        // Once all producer threads finish and drop their clones,
        // the job channel will close, signaling workers to finish
        drop(job_tx);

        // === RESULT CONSUMER ===
        // Single consumer collects all results from workers
        s.spawn(move |_| {
            let mut success_count = 0;
            let mut error_count = 0;
            
            for result in result_rx {
                match result {
                    Ok(msg) => {
                        println!("{}", msg);
                        success_count += 1;
                    }
                    Err(err) => {
                        eprintln!("Job failed: {}", err);
                        error_count += 1;
                    }
                }
            }
            
            println!("\n=== Processing Complete ===");
            println!("Successful: {}", success_count);
            println!("Failed: {}", error_count);
        });
        
        // Scope automatically waits for all threads to complete
        // No thread leaks possible - compiler enforced!
    })
    .expect("Thread pool execution failed");
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
        // SELECT: Non-deterministic choice between ready channels
        crossbeam::select! {
            recv(shutdown) -> _ => {
                // When shutdown channel is closed (sender dropped),
                // recv() returns Err, which matches any pattern
                // This is how we detect cancellation
                println!("Worker {} shutting down", id);
                return;  // Exit the worker
            }
            recv(ticker) -> _ => {
                // Ticker produces values every 500ms
                // We don't care about the value (it's an Instant)
                println!("Worker {} working...", id);
                // Continue loop to check for shutdown again
            }
        }
        // FAIRNESS: select! randomly chooses if both channels are ready
        // This prevents starvation - shutdown is always checked
    }
}

fn main() {
    let (shutdown_tx, shutdown_rx) = channel::bounded(0);
    // bounded(0) creates a rendezvous channel
    // Not used for passing data, just for signaling
    
    thread::scope(|s| {
        // Spawn workers, each with a clone of the receiver
        for i in 0..3 {
            let shutdown = shutdown_rx.clone();
            // Each worker owns a Receiver - multiple receivers are allowed
            // All receivers will be notified when the channel closes
            
            s.spawn(move || worker(i, shutdown));
        }
        
        // Let workers run for a while
        thread::sleep(Duration::from_secs(2));
        
        println!("Initiating shutdown...");
        drop(shutdown_tx);  
        // CRITICAL: Dropping the sender closes the channel
        // This is detected by ALL receivers simultaneously
        // No need to send shutdown messages to each worker
        // This is superior to Go's context cancellation because:
        // 1. It's impossible to forget to cancel (RAII)
        // 2. All workers are guaranteed to be notified
        // 3. No goroutine/thread leaks possible
        
        // Give time to see shutdown messages
        thread::sleep(Duration::from_millis(100));
    });
    // Scope guarantees all workers have terminated
    // Even if they ignored shutdown, scope forces join
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
            // Rayon automatically partitions work across CPU cores
            // Each thread in the pool gets a chunk of the data
            data.par_iter().for_each(|&item| {
                // This closure runs in parallel on multiple threads
                // But each item is processed exactly once
                let result = expensive_transform(item);
                tx_clone.send(result).unwrap();
                // Results arrive in arbitrary order due to parallelism
            });
            // tx_clone dropped here, but main tx still exists
        });
        drop(tx); // Drop original to signal completion
        // Now NO senders exist, so channel will close
        
        // Sequential post-processing
        s.spawn(move || {
            let results: Vec<String> = rx
                .iter()  // Iterates until channel closes
                .map(post_process)
                .collect();
            println!("Processed {} items", results.len());
        });
        // Both threads must complete before scope ends
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
        // BOUNDED CHANNEL AS SEMAPHORE:
        // The channel capacity IS the semaphore count
        // Each () in the channel represents an available permit
        
        // Fill channel with permits
        for _ in 0..capacity {
            tx.send(()).unwrap();
        }
        // Now the channel contains 'capacity' number of () values
        
        Semaphore {
            permits: tx,
            acquire: rx,
        }
    }
    
    fn acquire(&self) {
        // BLOCKING ACQUIRE: Take one () from the channel
        // If channel is empty, this blocks until a permit is available
        // This is thread-safe: only one thread can receive each ()
        self.acquire.recv().unwrap();
        // We now "hold" the permit (by removing it from the channel)
    }
    
    fn release(&self) {
        // RELEASE: Put () back into the channel
        // This allows another thread to acquire
        self.permits.send(()).unwrap();
        // Can only fail if receiver is dropped (semaphore destroyed)
    }
}

fn do_work(id: usize) {
    println!("Worker {} starting", id);
    thread::sleep(Duration::from_millis(500));
    println!("Worker {} done", id);
}

fn main() {
    let sem = Semaphore::new(3);  // Max 3 concurrent workers
    
    thread::scope(|s| {
        for i in 0..10 {
            let sem_ref = &sem;
            // Borrowing semaphore - all threads share the same channels
            
            s.spawn(move || {
                sem_ref.acquire();
                // CRITICAL SECTION: Only 3 threads can be here at once
                do_work(i);
                sem_ref.release();
                // If thread panics before release(), permit is lost
                // Better pattern: Use RAII guard (shown in advanced section)
            });
        }
    });
    // Scope ensures all threads complete
    // But note: permits might be lost if threads panic
    // This is why RAII guards are preferred in production
    
    println!("All workers complete!");
}
```

## Type-State Pattern for Compile-Time Protocol Enforcement

This pattern uses Rust's type system to enforce state machines at compile time:

```rust
use std::marker::PhantomData;

// State markers - Zero runtime cost (phantom types)
// These exist only at compile time for type checking
struct Initialized;
struct Running;
struct Stopped;

// State machine with compile-time state tracking
struct StateMachine<State> {
    data: String,
    _state: PhantomData<State>,
    // PhantomData tells compiler we're "using" the State type
    // Without this, compiler would complain State is unused
    // Has zero runtime cost - no memory allocation
}

// Methods only available in Initialized state
impl StateMachine<Initialized> {
    fn new(data: String) -> Self {
        StateMachine {
            data,
            _state: PhantomData,
        }
    }
    
    // Consumes self, returns NEW type StateMachine<Running>
    fn start(self) -> StateMachine<Running> {
        // OWNERSHIP TRANSFER: 'self' is consumed (moved)
        // The old StateMachine<Initialized> no longer exists
        // Cannot call any Initialized methods on it anymore
        
        println!("Starting with: {}", self.data);
        StateMachine {
            data: self.data,  // Move data to new state
            _state: PhantomData,  // Change phantom type to Running
        }
        // Return value has different type than input!
        // This is the key to compile-time state enforcement
    }
}

// Methods only available in Running state
impl StateMachine<Running> {
    fn process(&mut self) {
        // Can mutate in Running state
        // But note: doesn't change state type
        println!("Processing: {}", self.data);
        self.data.push_str(" [processed]");
    }
    
    // Consumes self, returns NEW type StateMachine<Stopped>
    fn stop(self) -> StateMachine<Stopped> {
        println!("Stopping...");
        StateMachine {
            data: self.data,
            _state: PhantomData,
        }
    }
}

// Methods only available in Stopped state
impl StateMachine<Stopped> {
    fn get_result(self) -> String {
        // Consumes the machine to get result
        // Machine cannot be used after this
        self.data
    }
}

fn main() {
    let machine = StateMachine::new("Hello".to_string());
    // Type: StateMachine<Initialized>
    
    // VALID sequence - compiler enforces correct order:
    let mut running = machine.start();
    // Type changed: StateMachine<Running>
    // 'machine' no longer exists - moved into 'running'
    
    running.process();  // Can call multiple times
    running.process();  // Still StateMachine<Running>
    
    let stopped = running.stop();
    // Type changed: StateMachine<Stopped>
    // 'running' no longer exists
    
    let result = stopped.get_result();
    // 'stopped' consumed - cannot use again
    
    println!("Final result: {}", result);
    
    // THESE WOULD NOT COMPILE - compiler errors:
    // machine.process();     
    // ERROR: no method `process` for StateMachine<Initialized>
    // StateMachine<Initialized> doesn't have process() method!
    
    // stopped.start();       
    // ERROR: no method `start` for StateMachine<Stopped>
    // Can't restart a stopped machine!
    
    // running.get_result();  
    // ERROR: no method `get_result` for StateMachine<Running>
    // Must stop before getting result!
    
    // machine.start();
    // ERROR: use of moved value 'machine'
    // Already consumed by first start() call!
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
            // Each spawn creates a new OS thread (~2MB stack)
            // 1000 items = 1000 threads = ~2GB just for stacks!
            // OS has thread limits (often ~10K-30K max)
            // Thread creation/destruction has significant overhead
            process_item(item);
        });
    }
    // Also: no way to wait for completion without JoinHandles
}

// ✅ Good: Use Rayon for automatic pooling
fn good_example(items: Vec<i32>) {
    items.par_iter().for_each(|&item| {
        // Rayon uses a thread pool sized to CPU cores
        // Work-stealing ensures balanced load
        // No thread creation overhead per item
        process_item(item);
    });
    // Automatically waits for all work to complete
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