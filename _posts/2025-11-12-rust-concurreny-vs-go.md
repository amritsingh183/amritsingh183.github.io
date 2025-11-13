---
layout: post
title: "Rust Concurrency explained for Go (Golang) Developers"
date: 2025-11-12 16:36:00 +0530
categories: rust concepts
last_updated: 2025-11-12
---

# Rust Concurrency for Go Developers: A Different Kind of Safety

If you're coming from Go, you already know how to write concurrent code. You've spawned countless goroutines, orchestrated them with channels, and probably have a few battle scars from debugging race conditions at 3 AM. You know the tools, you know the patterns, and honestly? Go makes concurrency pretty approachable.

So why would you want to learn Rust's approach?

Here's the thing: Go and Rust take fundamentally different philosophies toward concurrency safety. Go gives you powerful, flexible tools and trusts you to use them correctly. It's pragmatic—give developers the rope to build amazing things, and trust their testing and discipline to catch mistakes. The race detector helps, code reviews help, and honestly, this approach works really well for a lot of teams.

Rust takes a different path: it moves many of those safety checks from runtime to compile time. Not because Go's approach is wrong, but because there are certain kinds of bugs that are really hard to catch even with good practices. If you've ever had a data race slip through testing and hit production, you know what I'm talking about.

## Two Valid Philosophies

Let's start with a simple example. Here's Go code that compiles and runs:

```go
// Go: Runtime checks catch this if you run with -race
var counter int

func main() {
    for i := 0; i < 1000; i++ {
        go func() {
            counter++ // Data race - but it compiles fine
        }()
    }
    time.Sleep(time.Second)
    fmt.Println(counter) // Results vary
}
```

Is this bad code? Not necessarily—you'd catch it with the race detector, fix it, and move on. That's the Go way: build quickly, test thoroughly, iterate.

Here's the equivalent in Rust:

```rust
fn main() {
    let mut counter = 0;
    for _ in 0..1000 {
        std::thread::spawn(|| {
            counter += 1; // Won't compile
        });
    }
}
```

The compiler stops you immediately. Not because you're a bad programmer, but because the language can mathematically prove this is unsafe. It's just a different tradeoff: less flexibility in exchange for certain guarantees.

Neither approach is "right"—they optimize for different things. Go optimizes for developer velocity and runtime flexibility. Rust optimizes for compile-time correctness. Both are valid choices depending on what you're building.

## Scoped Threads: A Different Take on WaitGroups

In Go, you've probably written this pattern a hundred times:

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

This works great! It's explicit, it's clear what's happening, and it's flexible. You can add goroutines from anywhere, at any time. That flexibility is a strength.

Rust's scoped threads are less flexible but offer different guarantees:

```rust
use std::thread;

fn main() {
    let items = vec!["a", "b", "c", "d"];
    
    thread::scope(|s| {
        for item in &items {
            s.spawn(move || {
                process(item);
            });
        }
    }); // All threads guaranteed done here
    
    println!("All threads completed!");
}
```

The scope enforces that all threads complete before you exit. You can't forget to wait, can't have mismatched counts, and the compiler verifies all of this. It's more rigid, but that rigidity prevents certain classes of bugs.

### Borrowing Stack Data

Here's where the approaches really diverge. In Go, sharing data between goroutines usually means:

1. Passing copies through channels (safe but can be costly for large data)
2. Using mutexes (safe but requires discipline)
3. Using atomic operations (safe but tricky to get right)

All of these work, and experienced Go developers handle them well. But they require care.

Rust's scoped threads let you borrow data directly from the stack:

```rust
use std::thread;

fn parallel_processing() {
    let data = vec!["hello", "world", "from", "rust"];
    let mut results = vec![String::new(); data.len()];
    
    thread::scope(|s| {
        for (i, item) in data.iter().enumerate() {
            let result_ref = &mut results[i];
            
            s.spawn(move || {
                *result_ref = item.to_uppercase();
            });
        }
    });
    
    println!("Results: {:?}", results);
}
```

Each thread gets a mutable reference to a different slice of the results vector. The compiler verifies at compile time that no two threads touch the same memory. Zero runtime overhead, zero chance of races.

Could you do something similar in Go with careful slice indexing and discipline? Absolutely. But the compiler won't verify it for you—you're relying on code review and testing to catch mistakes.

## Rayon: When You Just Want to Parallelize a Loop

Go's worker pool pattern is solid. You've probably written it before:

```go
jobs := make(chan int, 100)
results := make(chan int, 100)

// Start workers
for w := 0; w < numWorkers; w++ {
    go worker(jobs, results)
}

// Send jobs
go func() {
    for _, job := range data {
        jobs <- job
    }
    close(jobs)
}()

// Collect results
for result := range results {
    // process result
}
```

This is good code! It's clear, it's flexible, and it works. But it's also boilerplate you've written many times.

Rayon asks: what if the common case was just... easier?

```rust
use rayon::prelude::*;

fn main() {
    let data: Vec<i32> = (0..1_000_000).collect();
    
    // Sequential
    let results: Vec<i32> = data
        .iter()
        .map(|&n| expensive_computation(n))
        .collect();
    
    // Parallel - literally just add "par_"
    let results: Vec<i32> = data
        .par_iter()
        .map(|&n| expensive_computation(n))
        .collect();
}
```

That's it. No workers to manage, no channels to coordinate, no cleanup code. Rayon handles the thread pool, work stealing, and load balancing automatically. And the compiler still verifies it's safe.

### When Parallel is This Easy

```rust
use rayon::prelude::*;

fn main() {
    let numbers: Vec<i32> = (0..1_000_000).collect();
    
    // Sum in parallel
    let sum: i32 = numbers.par_iter().sum();
    
    // Find max in parallel
    let max: Option<&i32> = numbers.par_iter().max();
    
    // Filter and map in parallel
    let results: Vec<i32> = numbers
        .par_iter()
        .filter(|&&x| x % 2 == 0)
        .map(|&x| x * x)
        .collect();
    
    // Custom fold operation
    let custom = numbers
        .par_iter()
        .fold(|| 0, |acc, &x| acc + x * x)
        .reduce(|| 0, |a, b| a + b);
}
```

Could you build this in Go? Sure! You'd set up worker pools, partition the data, coordinate results—it's doable. But would you actually parallelize every filter or map operation? Probably not, because the overhead of setting it up isn't always worth it.

Rayon makes it cheap enough (in terms of code) that you actually do it when it helps.

## Channels: Different Flavors of the Same Idea

Go's channels are brilliant. They're a first-class feature, they're easy to understand, and they make concurrent code readable. `select` statements let you coordinate multiple channels cleanly.

Crossbeam channels in Rust are similar but with some nice quality-of-life improvements:

```rust
use crossbeam::channel::{bounded, select};
use std::time::Duration;

fn main() {
    let (tx, rx) = bounded(10);
    
    // Clone for multiple senders - just like Go
    let tx2 = tx.clone();
    
    thread::spawn(move || {
        tx.send("Hello").unwrap();
    });
    
    thread::spawn(move || {
        tx2.send("World").unwrap();
    });
    
    // Select with timeout
    select! {
        recv(rx) -> msg => println!("Got: {:?}", msg),
        default(Duration::from_millis(100)) => println!("Timeout"),
    }
}
```

The API is pretty similar to Go's, which is nice if you're switching between languages. The main difference? Channels close automatically when the last sender is dropped—no manual `close()` calls needed.

### The Classic Producer-Consumer

Here's the pattern you've written in Go dozens of times:

```go
ch := make(chan int, 100)

// Producer
go func() {
    for i := 0; i < 1000; i++ {
        ch <- i
    }
    close(ch)
}()

// Consumer
for val := range ch {
    process(val)
}
```

And here's the Rust equivalent:

```rust
use crossbeam::channel::bounded;
use std::thread;

fn main() {
    let (tx, rx) = bounded(100);
    
    // Producer
    thread::spawn(move || {
        for i in 0..1000 {
            tx.send(i).unwrap();
        }
        // Channel closes when tx drops
    });
    
    // Consumer
    thread::spawn(move || {
        for item in rx {
            process(item);
        }
    });
}
```

The core pattern is the same. The main difference is that Rust uses RAII (drop) instead of explicit `close()`. Not necessarily better, just different. RAII means you can't forget, but explicit `close()` is more... well, explicit.

### Producer-Consumer Pipeline

Let's look at a slightly more complex pattern:

```rust
use crossbeam::channel;
use std::thread;

fn main() {
    let (tx1, rx1) = channel::bounded(100);
    let (tx2, rx2) = channel::bounded(100);
    
    thread::scope(|s| {
        // Producer thread
        s.spawn(move || {
            for i in 0..10 {
                tx1.send(i).unwrap();
            }
            // tx1 drops here, closing the channel
        });
        
        // Transformer thread
        s.spawn(move || {
            for val in rx1 {
                tx2.send(val * 2).unwrap();
            }
            // tx2 drops here, closing the channel
        });
        
        // Consumer (runs in scope)
        for val in rx2 {
            println!("{}", val);
        }
    });
}
```

Channels close automatically as senders go out of scope. This cascade of closures is guaranteed by the ownership system. In Go, you'd explicitly close each channel, which is more manual but also more transparent about what's happening.

## Worker Pools: Manual Control When You Need It

Sometimes Rayon isn't the right tool. Maybe you need precise control over worker lifetime, or you're dealing with I/O-bound work, or you need backpressure. That's when you reach for channels:

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
            // job_tx drops, closing channel
        });
        
        // Must drop our result_tx so channel can close
        drop(result_tx);
        
        // Collect results
        for result in result_rx {
            println!("{}", result);
        }
    });
}
```

This looks pretty similar to Go, right? Workers pull from a shared queue, send results back. The main difference is the automatic channel closure and the scope guarantee that all threads complete.

### Worker Pool with Deadlines

When you need more control—like deadlines or timeout handling—you can get as sophisticated as you need:

```rust
use crossbeam::channel::{bounded, SendTimeoutError};
use std::time::{Duration, Instant};

#[derive(Debug)]
struct Job {
    id: usize,
    data: String,
}

fn handle_job(job_rx: Receiver<Job>, result_tx: Sender<Result<String, String>>) {
    for job in job_rx {
        std::thread::sleep(Duration::from_millis(10));
        let result = Ok(format!("Processed job {}", job.id));
        
        if result_tx.send(result).is_err() {
            break;
        }
    }
}

fn main() {
    let (result_tx, result_rx) = bounded(10);
    let (job_tx, job_rx) = bounded(10);
    let deadline = Instant::now() + Duration::from_secs(2);

    thread::scope(|s| {
        // Workers
        for worker_id in 0..4 {
            let worker_job_rx = job_rx.clone();
            let worker_result_tx = result_tx.clone();
            
            s.spawn(move || {
                handle_job(worker_job_rx, worker_result_tx);
            });
        }
        
        drop(job_rx);
        drop(result_tx);

        // Producers with deadline
        for idx in 0..100 {
            let producer_job_tx = job_tx.clone();
            
            s.spawn(move || {
                let job = Job::new(idx, format!("data-{}", idx));
                
                match producer_job_tx.send_deadline(job, deadline) {
                    Ok(_) => {},
                    Err(SendTimeoutError::Timeout(_)) => {
                        eprintln!("Job {} timed out", idx);
                    }
                    Err(SendTimeoutError::Disconnected(_)) => {
                        eprintln!("Channel disconnected");
                    }
                }
            });
        }
        
        drop(job_tx);

        // Result consumer
        s.spawn(move || {
            let mut success_count = 0;
            let mut error_count = 0;
            
            for result in result_rx {
                match result {
                    Ok(msg) => {
                        println!("{}", msg);
                        success_count += 1;
                    }
                    Err(err) => {
                        eprintln!("Failed: {}", err);
                        error_count += 1;
                    }
                }
            }
            
            println!("Successful: {}, Failed: {}", success_count, error_count);
        });
    }).unwrap();
}
```

This is more complex than the basic pattern, but it shows that Rust can handle sophisticated concurrent architectures when you need them. The difference is that the compiler is checking your work at every step.

## Cancellation: Different Approaches to the Same Problem

Go's `context.Context` is elegant. You pass it through your call chain, check for cancellation at key points, and it works really well. It's flexible and composable.

Rust typically uses channel-based cancellation:

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
        for i in 0..3 {
            let shutdown = shutdown_rx.clone();
            s.spawn(move || worker(i, shutdown));
        }
        
        thread::sleep(Duration::from_secs(2));
        
        println!("Initiating shutdown...");
        drop(shutdown_tx);  // Dropping sender closes channel
        
        thread::sleep(Duration::from_millis(100));
    });
}
```

When you drop the sender, all receivers see the closure. It's less flexible than Go's context (you can't attach values, for example), but it's simple and it works. Different design, different tradeoffs.

## Combining Tools: Rayon + Channels

One nice thing about Rust's concurrency tools is that they compose well:

```rust
use rayon::prelude::*;
use crossbeam::channel;
use std::thread;

fn expensive_transform(input: i32) -> i32 {
    std::thread::sleep(std::time::Duration::from_millis(1));
    input * input
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
        drop(tx);
        
        // Sequential post-processing
        s.spawn(move || {
            let results: Vec<String> = rx
                .iter()
                .map(|x| format!("Result: {}", x))
                .collect();
            println!("Processed {} items", results.len());
        });
    });
}
```

Rayon handles CPU-bound parallelism, channels handle coordination. Each tool does what it's best at.

### Multi-Stage Pipeline

You can build complex pipelines mixing sequential and parallel stages:

```rust
use rayon::prelude::*;
use crossbeam::channel;
use std::thread;

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
                .map(|s| s.to_uppercase())
                .for_each(|result| {
                    tx1.send(result).unwrap();
                });
        });
        
        // Stage 2: Length calculation
        s.spawn(move || {
            for item in rx1 {
                let length = item.len();
                tx2.send(length).unwrap();
            }
        });
        
        // Stage 3: Aggregation
        let handle = s.spawn(move || {
            let lengths: Vec<usize> = rx2.iter().collect();
            lengths.iter().sum::<usize>()
        });
        
        let total = handle.join().unwrap();
        println!("Total length: {}", total);
    });
}
```

Each stage does its thing, and the channels coordinate between them. Pretty similar to how you'd do it in Go, honestly.

## Semaphores: A Cool Channel Trick

Here's a neat pattern—building a semaphore from a channel:

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
        
        // Fill channel with permits
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

fn main() {
    let sem = Semaphore::new(3);  // Max 3 concurrent workers
    
    thread::scope(|s| {
        for i in 0..10 {
            let sem_ref = &sem;
            
            s.spawn(move || {
                sem_ref.acquire();
                println!("Worker {} starting", i);
                thread::sleep(Duration::from_millis(500));
                println!("Worker {} done", i);
                sem_ref.release();
            });
        }
    });
}
```

The channel capacity becomes your semaphore count. Each `()` in the channel is a permit. It's elegant once you see it.

## Type States: Encoding Protocols in Types

This one doesn't have a direct Go equivalent, but it's worth showing because it demonstrates a different way of thinking about concurrency safety.

In Go, you might have a connection that goes through states:

```go
type Connection struct {
    state string // "disconnected", "connected", "closed"
}

func (c *Connection) Send(data string) error {
    if c.state != "connected" {
        return errors.New("not connected")
    }
    // send data...
    return nil
}
```

You check the state at runtime. This works fine, especially with good testing.

Rust lets you encode states in the type system:

```rust
use std::marker::PhantomData;

struct Disconnected;
struct Connected;
struct Closed;

struct Connection<State> {
    data: String,
    _state: PhantomData<State>,
}

impl Connection<Disconnected> {
    fn new(data: String) -> Self {
        Connection {
            data,
            _state: PhantomData,
        }
    }
    
    fn connect(self) -> Connection<Connected> {
        println!("Connecting...");
        Connection {
            data: self.data,
            _state: PhantomData,
        }
    }
}

impl Connection<Connected> {
    fn send(&mut self, msg: &str) {
        println!("Sending: {}", msg);
    }
    
    fn close(self) -> Connection<Closed> {
        println!("Closing...");
        Connection {
            data: self.data,
            _state: PhantomData,
        }
    }
}

impl Connection<Closed> {
    fn get_result(self) -> String {
        self.data
    }
}

fn main() {
    let conn = Connection::new("Hello".to_string());
    // Type: Connection<Disconnected>
    
    let mut conn = conn.connect();
    // Type: Connection<Connected>
    
    conn.send("data");
    
    let conn = conn.close();
    // Type: Connection<Closed>
    
    // These won't compile:
    // conn.send("more data");  // Error: no method 'send' on Connection<Closed>
    // conn.connect();          // Error: no method 'connect' on Connection<Closed>
}
```

Try to send on a disconnected connection? Won't compile. Try to connect an already-connected connection? Won't compile. The invalid states are unrepresentable.

Is this better than runtime checks? Depends on your situation. For protocols with complex state machines (think database connection pools, network protocols, etc.), encoding states in types can prevent entire classes of bugs. For simpler cases, Go's runtime checks might be clearer and more straightforward.

## When to Use What

After all this, you might be wondering which tool to reach for. Here's what I'd suggest:

**Use `thread::scope` when:**
- You're replacing a `sync.WaitGroup` pattern
- You need to borrow data safely across threads
- You want guaranteed thread completion

**Use Rayon when:**
- Processing collections in parallel
- CPU-bound work that can be split up
- You want automatic load balancing

**Use Crossbeam channels when:**
- Producer-consumer patterns
- Message passing between threads
- Coordinating multiple threads

**Avoid async unless:**
- You need thousands of concurrent I/O operations
- You're in WebAssembly (no thread support)
- A library forces you to

The async ecosystem in Rust is powerful but complex. If threads work for your use case, they're often simpler and the compiler guarantees are stronger.

## Common Pitfalls

### Don't Create Too Many OS Threads

```rust
// ❌ Creates a thread per item (expensive!)
fn bad_example(items: Vec<i32>) {
    for item in items {
        thread::spawn(move || {
            process_item(item);
        });
    }
}

// ✅ Use Rayon for automatic pooling
fn good_example(items: Vec<i32>) {
    items.par_iter().for_each(|&item| {
        process_item(item);
    });
}
```

Each OS thread gets about 2MB of stack. A thousand threads = 2GB just for stacks. Rayon uses a pool sized to your CPU cores.

### Mind Your Channel Capacity

```rust
// ❌ Unbounded can cause memory issues
let (tx, rx) = crossbeam::channel::unbounded();

// ✅ Bounded provides backpressure
let (tx, rx) = crossbeam::channel::bounded(100);
```

If producers outpace consumers, unbounded channels grow without limit. Bounded channels make producers wait, which is usually what you want.

### Let Rayon Handle Collection Processing

```rust
// ❌ Manual thread management for collection processing
fn manual(data: Vec<i32>) -> Vec<i32> {
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

// ✅ Let Rayon do the work
fn rayon_way(data: Vec<i32>) -> Vec<i32> {
    data.par_iter().map(|x| x * 2).collect()
}
```

Rayon handles partitioning, load balancing, and work stealing automatically. Unless you have a specific reason to manage threads manually, let Rayon handle it.

## Performance Characteristics

Here's roughly what you can expect:

| Operation | Overhead | Scaling | Best For |
|-----------|----------|---------|----------|
| Scoped Threads | ~2MB stack/thread | Up to core count | Known parallelism |
| Rayon | Work-stealing pool | Near-linear | Data parallelism |
| Crossbeam Channels | ~20ns/send | Lock-free | Message passing |

All three provide compile-time safety guarantees.

## Quick Translation Guide

Coming from Go, here's how common patterns map:

| Go Pattern | Rust Equivalent | Key Difference |
|-----------|-----------------|----------------|
| `go func(){}` | `thread::scope()` | Lifetime checked by compiler |
| `sync.WaitGroup` | `thread::scope()` | Automatic completion tracking |
| `chan T` | `crossbeam::channel` | Auto-close on drop |
| `select {}` | `crossbeam::select!` | Similar syntax |
| `context.Context` | Channel + drop | Drop triggers cancellation |
| Worker pools | Rayon | Automatic work-stealing |
| `sync.Mutex` | `std::sync::Mutex` | Can't access without lock |

## The Bottom Line

Go and Rust take different approaches to concurrency, and both are thoughtfully designed. Go optimizes for simplicity and flexibility—it gives you powerful tools and trusts you to use them well. Rust optimizes for compile-time correctness—it moves many checks earlier and enforces them automatically.

Neither approach is inherently better. It depends on what you're building:

- Building a microservice that needs to move fast? Go's simplicity is great.
- Building a database engine where correctness is paramount? Rust's guarantees shine.
- Prototyping a new idea? Go lets you iterate quickly.
- Maintaining code for years? Rust's compile-time checks age well.

If you're coming from Go, you already understand concurrent programming. Learning Rust's approach isn't about unlearning Go—it's about adding another tool to your toolkit. Sometimes you need runtime flexibility. Sometimes you want compile-time proofs. Both are valuable.

The learning curve exists. The compiler will reject code that would work in Go. You'll fight with the borrow checker. But once you internalize the model, you gain the ability to write concurrent code with a level of confidence that's hard to achieve otherwise.

Welcome to Rust concurrency. It's different, but in ways that might grow on you.