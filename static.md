## 4. Static Items <a href="#9-static-items-" class="header-link">🔗</a>

### What is static <a href="#what-is-static-" class="header-link">🔗</a>

A `static` item is a value that lives for the entire duration of the program.  It occupies a single fixed memory address.

```rust
static MAX_CONNECTIONS: u32 = 100;

fn main() {
    println!("Maximum connections: {}", MAX_CONNECTIONS);
}
```

All references to a `static` item point to the same memory location.  This is different from `const`, where each use gets its own copy.

### Static vs const comparison <a href="#static-vs-const-comparison-" class="header-link">🔗</a>

The differences between `static` and `const` are important:


| Feature | const | static |
| :-- | :-- | :-- |
| Memory location | No fixed address; inlined at each use  | Single fixed address  |
| Lifetime | N/A (inlined)  | 'static  |
| Mutability | Always immutable  | Can be mutable with `static mut`  |
| Address stability | Different address for each use  | Same address always  |
| Thread safety requirement | None  | Must implement Sync (for immutable)  |



### When to use static <a href="#when-to-use-static-" class="header-link">🔗</a>

Use `static` when you need:

- A single fixed memory address, like for FFI (Foreign Function Interface)
- Global mutable state with interior mutability (like `Mutex` or `RwLock`)
- Large read-only data that should not be duplicated

```rust
static LANGUAGE: &str = "Rust";

fn main() {
    let ptr1 = &LANGUAGE as *const _;
    let ptr2 = &LANGUAGE as *const _;
    assert_eq!(ptr1, ptr2); // same address
}
```


### Mutable statics and safety <a href="#mutable-statics-and-safety-" class="header-link">🔗</a>

You can declare a `static mut` for global mutable state, but accessing it requires `unsafe`:

```rust
static mut COUNTER: u32 = 0;

fn increment_counter() {
    unsafe {
        COUNTER += 1;
    }
}

fn main() {
    increment_counter();
    unsafe {
        println!("Counter: {}", COUNTER);
    }
}
```

Mutable statics are unsafe because multiple threads could access them simultaneously, causing data races.  Prefer safe alternatives like atomics or locks.

### The Sync requirement <a href="#the-sync-requirement-" class="header-link">🔗</a>

Immutable `static` items must implement the `Sync` trait, which means they are safe to access from multiple threads.  Most types with only immutable data are automatically `Sync`.

```rust
static NUMBERS: [i32; 3] = [1, 2, 3]; // OK: arrays of i32 are Sync
```

Types like `Cell` and `RefCell` are not `Sync`, so you cannot use them in a `static` directly.  You would need to wrap them in a thread-safe type.

### Mutable Static References Are Unsafe <a href="#mutable-static-references-are-unsafe-" class="header-link">🔗</a>

When you create a reference to a mutable static variable, you bypass Rust's safety guarantees. The `static_mut_refs` lint is **deny-by-default** because even creating such a reference (without using it) can lead to undefined behavior. The compiler cannot verify safety when multiple mutable references to the same static data could exist.

**What NOT to do:**

```rust
static mut COUNTER: u32 = 0;

fn main() {
    unsafe {
        let r = &COUNTER;  // ❌ Denied: references to mutable statics
        println!("{}", r);
    }
}
```

**How to handle mutable state correctly:**

For thread-safe counters and shared state, use atomic types:

```rust
use std::sync::atomic::{AtomicU32, Ordering};

static COUNTER: AtomicU32 = AtomicU32::new(0);

fn main() {
    COUNTER.fetch_add(1, Ordering::SeqCst);
    println!("{}", COUNTER.load(Ordering::SeqCst));
}
```

For other patterns, reach for:

- **Atomic types** for counters and flags
- **Mutex** for shared state requiring mutual exclusion
- **RwLock** for read-heavy scenarios
- **OnceLock** for one-time initialization
- **LazyLock** for lazy-initialized static data

If you absolutely need raw pointers for FFI or low-level code, use `&raw const` or `&raw mut` instead of references. Raw pointers bypass all safety checks and require careful manual synchronization.
