---
layout: post
title: "Mastering Variables, Constants and Lifetimes in Rust: A Complete Guide"
date: 2025-01-01 11:23:00 +0530
categories: rust concepts
last_updated: 2025-12-30
---
# Variables, Constants, and Statics in Rust: A Complete Guide to Ownership and Borrowing

**Note on related topics:**

> Memory layout details are covered extensively in this article, and for deeper technical background, you can reference our [comprehensive memory layout guide](/rust/concepts/2025/01/05/rust-mem-ref.html).

> Lifetime syntax and reference semantics are covered in a [separate guide](/rust/concepts/2025/02/09/rust-ownership.html) after you've mastered the concepts in this article.

## Foundation: The Core Safety Principle

### Aliasing XOR Mutability

Rust’s core safety model enforces the “aliasing XOR mutability” rule: you may have either:

- (1) multiple aliased shared references (`&T`), or 
- (2) a single unique mutable reference (`&mut T`), but never both at the same time. 

This applies universally, not just for thread safety or data races, but also for single-threaded code—it prevents iterator invalidation, side-effect ordering bugs, and ensures optimizing compilers can make aggressive assumptions. Data allocation location (stack or heap) does not affect this rule. The rule applies to *all pointers*, including raw pointers: even `*const T` and `*mut T` follow the same logical principle, though enforcement shifts to the programmer in unsafe code.


In simple terms, **you can have many readers OR one writer, but never both simultaneously**. It prevents data races at compile time—a guarantee no other mainstream language without a garbage collector provides.

Think of it like a shared document: either many people can read it (shared access) or one person can edit it (exclusive access), but you cannot have someone editing while others are reading. This rule is enforced by the borrow checker, which analyzes your code at compile time to guarantee no two parts of your program can modify the same data simultaneously.

### How This Guide Is Organized

This guide assumes you understand basic Rust syntax (variables, functions, control flow) but doesn't require systems programming experience. The goal is to build a mental model so you can write safe code confidently, design APIs that prevent misuse, and debug compiler errors with understanding rather than frustration.

***

## Part One: Variable Bindings

### Immutability by Default

In Rust, all bindings are immutable by default. Once you assign a value to a variable, you cannot change it unless you explicitly opt in with `mut`. This design encourages writing code with fewer side effects and makes data flow clearer.

```rust
fn main() {
    let x = 5;
    println!("The value of x is: {}", x);
    // x = 6; // ERROR: cannot assign twice to immutable binding
}
```

The compiler prevents reassignment, catching entire classes of bugs that would cause subtle issues in other languages.

### Mutable Bindings

To enable reassignment, add `mut` when declaring the binding:

```rust
fn main() {
    let mut y = 10;
    println!("The value of y is: {}", y);
    y = 20; // OK: y is mutable
    println!("The value of y is now: {}", y);
}
```

**Important**: Mutability only allows changing the value, never the type. A binding declared as an integer remains an integer for its entire lifetime.

### Variable Shadowing

Shadowing declares a new binding with the same name, making the old one inaccessible. Unlike mutability, shadowing creates a completely new variable, so you can change types:

```rust
fn main() {
    let x = 5;
    let x = x + 1;    // Shadow x with new binding
    {
        let x = x * 2;  // Shadow again (scope-limited)
        println!("Inner x: {}", x); // prints 12
    }
    println!("Outer x: {}", x);     // prints 6
}
```

This is fundamentally different from mutability because each `let` creates a new variable:

```rust
fn main() {
    let spaces = "   ";        // String type
    let spaces = spaces.len(); // Now it's usize—type changed!
    println!("{}", spaces);    // prints 3
}
```

You cannot do this with a mutable binding because `mut` only allows value changes, not type changes.

### Variable Shadowing: Advantages and Pitfalls

While Rust permits variable shadowing for convenience—particularly for transforming types or values—it is important to recognize that excessive shadowing harms readability and introduces maintenance risks. Shadowing is most appropriate when:

- Transforming a value into a new type (e.g., `String` to `usize`)
- A variable's purpose changes semantically between phases
- The shadowed binding's last use is immediately nearby (within ~5 lines)

Prefer distinct variable names when shadowing creates ambiguity, especially in functions longer than 20 lines or where multiple contributors may misinterpret the intent. In production codebases, excessive shadowing has caused real bugs where developers accidentally reused names without realizing they'd been shadowed, leading to logic errors in refactoring. Consider this a strong code smell in collaborative environments.

**Example of problematic shadowing:**

```rust
let config = parse_config(input);
let config = apply_defaults(config);
let config = validate(config);
let config = optimize(config);  // Multiple transformations bury intent
```

**Better:**

```rust
let config = parse_config(input);
let config_with_defaults = apply_defaults(config);
let config_validated = validate(config_with_defaults);
let config_optimized = optimize(config_validated);  // Clear data flow
```

### Scope and Automatic Cleanup

Every variable lives within a scope, delimited by curly braces `{}`. When a variable goes out of scope, Rust calls the `Drop` trait to automatically clean up its memory. There is no garbage collector and no manual memory management—Rust ensures resources are freed at exactly the right time.

```rust
fn main() {
    let s = String::from("hello"); // s is valid from here
    // you can use s here
} // s goes out of scope and Drop is called here
// s is no longer valid
```


#### The Drop Trait

The `Drop` trait allows you to customize what happens when a value is destroyed. Any type implementing `Drop` must provide a `drop` method:

```rust
pub trait Drop {
    fn drop(&mut self) { }
}
```

#### Why Drop Takes `&mut self`

`Drop::drop` takes a mutable reference (`&mut self`) rather than ownership because destructors need to mutate the value's internal state (deallocating heap memory, closing file handles) without consuming it. This is a **language-level exception** [why it's an exception? Because Normally, obtaining `&mut T` from an immutable binding is forbidden] that only the compiler can invoke:

1. **Compiler-controlled invocation**: Only the compiler calls `Drop::drop` during automatic cleanup. You cannot manually call it—attempting `value.drop()` results in compile error E0040.

2. **Temporary mutable reference**: When dropping an immutable binding, the compiler creates a temporary mutable reference for the drop call. This is safe because:
   - The value is being destroyed (no user code can observe it)
   - No other references exist at drop time (enforced by borrow checker)
   - This happens at a point where normal borrowing rules don't apply

3. **Why mutation is necessary**: Destructors must perform side effects like freeing memory, closing files, or releasing locks. These operations require `&mut self` semantics.

The key insight: **Drop receives special compiler handling.** The compiler automatically invokes Drop::drop() during scope cleanup without user code explicitly calling it. Only Drop receives this compiler treatment because it's fundamental to resource management. You cannot create custom traits with automatic-invocation behavior; that's a compiler privilege reserved for Drop.


**Concrete example of compiler-controlled Drop:**

```rust
struct SmartPointer {
    data: String,
}

impl Drop for SmartPointer {
    fn drop(&mut self) {
        println!("Dropping SmartPointer with data: {}", self.data);
    }
}

fn main() {
    let ptr = SmartPointer {
        data: String::from("my data")
    }; // ptr is not declared as mut
    // ptr is immutable, but the compiler will create a temporary
    // mutable reference when dropping it—this is a compiler privilege
    // When ptr goes out of scope, the compiler safely calls (not manually written):
    // Drop::drop(&mut ptr)
 } // Output: "Dropping SmartPointer with data: my data"
```

Even though `ptr` is immutable, the compiler creates a temporary mutable reference for the drop call because this is the only place it happens and the value is about to be destroyed anyway.

**Critical limitation:** You cannot manually invoke `Drop::drop(&mut value)` in user code. Attempting to do so results in compiler error E0040. Only the compiler is permitted to call `Drop::drop()` during scope cleanup. If you want to explicitly trigger cleanup, use `std::mem::drop(value)`, which takes ownership and causes the value to be dropped when it goes out of scope (immediately in this context).

***

## Part Two: Constants

### Declaring Constants

Constants are declared with the `const` keyword and **must always have a type annotation**. Unlike variables, constants can be declared in any scope, including global scope:

```rust
const MAX_POINTS: u32 = 100_000;

fn main() {
    const HOURS_IN_DAY: u32 = 24;
    println!("Max points: {}", MAX_POINTS);
    println!("Hours: {}", HOURS_IN_DAY);
}
```


### When to Use Constants

Use constants for values that are known at compile time and never change. Examples: mathematical constants, configuration limits, fixed array sizes, or compile-time lookup tables.

```rust
const PI: f64 = 3.14159265359;
const MAX_BUFFER_SIZE: usize = 1024;
const THREE_HOURS_IN_SECONDS: u32 = 60 * 60 * 3; // Computed at compile time

fn main() {
    // const RUNTIME_VAL: u32 = get_user_input();  // ERROR: not a const fn
    // Const initializers can only call const fn or evaluate constant expressions.
}
```

**Why?** Constants are compile-time values inlined at each use site, so the compiler must know their value before generating machine code. Runtime operations (file I/O, system time, random values) violate this requirement.
- Literal values: `5`, `"hello"`, `3.14`
- Compile-time arithmetic: `60 * 60 * 3`
- Const function calls: `u32::MAX`
- Const generic expansions (Rust 1.79+): `std::array::from_fn::<_, LEN, _>(|i| i as u32)`

Runtime-dependent values (results that vary per execution), I/O operations, and calls to non-const functions are forbidden

```rust
const INVALID: u32 = std::time::SystemTime::now().elapsed().unwrap().as_secs() as u32;
// ERROR: time operations aren't const; result is runtime-dependent
```


**Note:** Const functions are a separate feature (marked `const fn`) that enables compile-time evaluation. Most standard library functions are not const; for those cases, use runtime initialization with `LazyLock` or `OnceLock` for alternatives.


### Constants vs Variables

| Feature | `const` | `let` |
| :-- | :-- | :-- |
| **Mutability** | Always immutable; `mut` cannot be used | Immutable by default; can use `mut` |
| **Type Annotation** | Mandatory—must be explicitly declared | Optional—compiler infers the type |
| **Value** | Must be constant expression evaluated at compile time | Can be computed at runtime |
| **Memory** | No fixed address; each use is replaced with the value directly (inlining). `Copy` types (`i32`, `bool`, `&T`) inline cost-free. Larger types like `&str` or `&[T]` are stored once per compilation unit and referenced at use sites. Const is a **compile-time value**, not a storage location; to guarantee a single address, use `static`. | Has a guaranteed fixed address in memory at runtime |
| **Scope** | Can be declared anywhere, including globally | Scoped to the block where declared |

**Key distinction**: Constants don't have a fixed address in the way statics do. Instead, const values are **inlined** at each use site (for `Copy` types like `i32`) 
or stored once per compilation unit (for non-`Copy` types like `&str`). For example, `const GREETING: &str = "hello"` might result in the string literal appearing once in your binary, with references at each use site. 

**Implementation detail**: The `&str` value itself (the pointer and length) is stored once per compilation unit and dereferenced at use sites. This is different from `Copy` types like `i32`, which are truly inlined (each use site has the literal `42` embedded). The key difference from `static`: if the same const is used across multiple compilation units (e.g., different .so files), separate copies may exist. Use `static` when you need a **single guaranteed address** throughout the entire program.

**Concrete examples:**

```rust
// Example: Demonstrating that static has a fixed address
static STATIC_VAL: i32 = 42;
const CONST_VAL: i32 = 42;

fn main() {
    let ptr1 = &STATIC_VAL as *const i32;
    let ptr2 = &STATIC_VAL as *const i32;
    assert_eq!(ptr1, ptr2);  // Same address: static has a fixed location
    
    // const has no guaranteed address—compiler may inline or deduplicate
    let ptr3 = &CONST_VAL as *const i32;
    let ptr4 = &CONST_VAL as *const i32;
    // ptr3 and ptr4 may or may not be equal (implementation-defined)
}
```



```rust
const GREETING: &str = "Hello";   // Compiler may inline this string literal
const NUMBERS: [u32; 2] = [1, 2]; // Duplicated if used in multiple .so files

// Each of these may have different memory addresses:
fn greet_alice() { println!("{}", GREETING); }
fn greet_bob() { println!("{}", GREETING); }

// vs static guarantees single address:
static GREETING_STATIC: &str = "Hello";
// All references point to identical memory
```

> All uses of a `const` are replaced directly by their value at each use site (inlining). This can increase binary size if the value is large or used often, but improves access speed compared to loading from an address[web:13]. Use `static` for a single address, especially for large data or FFI.

**Warning:** Not all types support const initialization. Types that require runtime computation (filesystem I/O, network access, system time) cannot be used in const contexts. If you need to initialize a non-const type globally, use `LazyLock` or `OnceLock`:


```rust
// ❌ WRONG: Compiler error
const DB_CONNECTION: String = String::from("would be runtime");

// ✅ RIGHT: Lazy initialization
use std::sync::LazyLock;
static DB_CONNECTION: LazyLock<String> = LazyLock::new(|| {
    // This closure runs on first access, not at compile time
    String::from("postgres://localhost")
});
```

## Part Three: Static Items

### What Is Static

A `static` item is a value that lives for the entire duration of the program and occupies a single fixed memory address. All references to the same static point to identical memory.

```rust
static MAX_CONNECTIONS: u32 = 100;

// This uses a const initializer, evaluated at compile time.
// For runtime initialization, use OnceLock or LazyLock (covered below).

fn main() {
    println!("Maximum connections: {}", MAX_CONNECTIONS);
}
```

This differs fundamentally from `const`, where each use may result in different memory locations (or no location at all if inlined).

### Static vs Const Comparison

| Feature | `const` | `static` |
| :-- | :-- | :-- |
| **Memory Address** | No fixed address; compiler inlines the value | Single fixed address throughout program |
| **Initialization** | Evaluated at compile time; no runtime cost | Evaluated at program startup (before main); or lazily via LazyLock/LazyCell (Rust 1.80+) |
| **Mutability** | Always immutable | Can be mutable with `static mut` (unsafe) |
| **Thread Safety** | N/A (no runtime concept) | Immutable statics must implement `Sync` |
| **Use Case** | Compile-time constants, values to inline | Global state, FFI, large read-only data |


Static items can be initialized in two ways:

1. **Compile-time (eager):** The value is computed at compile time and stored in the binary. This requires a constant expression.

```rust
static PORT: u16 = 8080;  // Compile-time
```

2. **Lazy initialization (runtime):** The value is computed on first access via `LazyLock` or `LazyCell` (Rust 1.80+). This allows runtime computation and reduces startup time.

```rust
use std::sync::LazyLock;

static DB: LazyLock<Database> = LazyLock::new(|| {
    Database::connect("postgres://localhost")  // Evaluated on first access
});

```

For most new code, **prefer `LazyLock` over `OnceLock`** when initialization logic is known at definition time; it provides the same thread-safety with a cleaner API.

### When to Use Static

Use `static` when you need:

- A single fixed memory address (essential for FFI—Foreign Function Interface)
- Global mutable state with interior mutability (using `Mutex`, `RwLock`, `OnceLock`, `LazyLock`)
- Large read-only data that should not be duplicated across your binary
- Per-program-lifetime state

```rust
static LANGUAGE: &str = "Rust";

fn main() {
    let ptr1 = &LANGUAGE as *const _; // Address 1
    let ptr2 = &LANGUAGE as *const _; // Same address
    assert_eq!(ptr1, ptr2);
}
```


### Mutable Statics and Safety

**CRITICAL in Rust 2024:** Mutable statics are problematic and should be avoided entirely in new code. Taking **any reference** to a `static mut`—even without reading or writing through it—is instantaneous undefined behavior and violates the aliasing XOR mutability principle. In Rust 2024 and later, the `static_mut_refs` lint is **deny-by-default**, preventing this footgun at compile time. Creating a reference includes implicit cases (method calls, format macros).

This limitation makes `static mut` unsuitable for almost all real-world use cases. Instead, use thread-safe alternatives listed below.


```rust
// OUTDATED CODE - DO NOT USE
static mut COUNTER: u32 = 0;

fn increment_counter() {
    unsafe {
        COUNTER += 1;
    }
}

fn main() {
    unsafe {
        increment_counter();
        // ❌ ERROR in Rust 2024: static_mut_refs lint (deny-by-default)
        // println!("{}", COUNTER);  // Taking implicit reference
    }
}

```

### The Sync Requirement for Immutable Statics

Immutable `static` items must implement the `Sync` trait, which certifies they are safe to access from multiple threads. Most types composed entirely of immutable data are automatically `Sync`:

```rust
static NUMBERS: [i32; 3] = [1, 2, 3]; // OK: [i32; 3] is Sync
```

Types like `RefCell` are **not** `Sync` and cannot be used directly in a `static`. 
`Cell<T>` is `Sync` if `T` is `Sync`, but it's still unsuitable for statics because 
`Cell` doesn't provide thread-safe mutation—only single-threaded interior mutability. 
For thread-safe shared state, use `Mutex` or `RwLock` (or atomics for simple types). 
You must wrap them in thread-safe alternatives like `Mutex` or `RwLock`.

### Mutable Static References: A Rust 2024 Change

**Mutable References: A Rust 2024 Change**

**In Rust 2024, the `static_mut_refs` lint is deny-by-default**, preventing any reference (shared or mutable) to a `static mut`. Taking such a reference—even without reading or writing through it—violates Rust's aliasing XOR mutability principle and is **instantaneous undefined behavior**. The compiler treats this as unrecoverable because global reasoning about thread safety for mutable statics is impossible in real programs with reentrancy and multithreading.

### Why References to `static mut` Are Undefined Behavior

Taking **any** reference (shared or mutable) to a `static mut`—even without reading or writing through it—is **instantaneous undefined behavior** that violates the aliasing XOR mutability principle. This is fundamental:

- A reference represents a **borrow promise** to the Rust type system
- The compiler optimizes based on this promise
- For `static mut`, the compiler cannot verify global reasoning (thread reentrancy makes it impossible)
- Therefore, taking a reference (visible or implicit) is UB regardless of whether you use it

**Explicitly creating references:**

```rust
static mut X: i32 = 0;
unsafe {
    let r = &X; // ❌ ERROR in Rust 2024+: UB, lint denies this
}
```

**Implicit references (also UB):**

```rust
static mut NUMS: [i32; 3] =;​​
unsafe {
    println!("{:?}", NUMS); // ❌ ERROR: println! creates implicit reference
    let n = NUMS.len(); // ❌ ERROR: method calls create implicit reference
}
```

**Using `&raw const` or `&raw mut` DOES bypass the lint:**

```rust
static mut X: i32 = 0;
unsafe {
    let ptr = &raw const X; // ✅ Compiles (raw pointers bypass checks)
    println!("{}", *ptr); // ❌ Still UB: same aliasing violation
}
```


However, this doesn't solve the underlying safety problem. Raw pointers move verification from the compiler to the programmer, who must manually ensure no data races occur. For production code with static mut accessed across threads, safer alternatives (atomics, Mutex) are always preferable. Using raw pointers here trades compile-time guarantees for runtime bugs.


**How to handle mutable global state correctly:**

For counters and coordination, use atomic types:

```rust
use std::sync::atomic::{AtomicU32, Ordering};

static COUNTER: AtomicU32 = AtomicU32::new(0);

fn main() {
    COUNTER.fetch_add(1, Ordering::SeqCst);
    println!("{}", COUNTER.load(Ordering::SeqCst));
}
```

For other patterns, choose based on your needs:

- **Atomic types** (`AtomicU64`, `AtomicBool`, etc.) for counters and flags
- **Mutex** for shared state requiring mutual exclusion
- **RwLock** for read-heavy scenarios
- **OnceLock** for one-time initialization with external setup
- **LazyLock** for lazy-initialized static data (preferred for most cases)

***

## Part Four: Ownership Fundamentals

### The Three Ownership Rules

Rust's ownership system has three rules that prevent memory leaks, double-free errors, and use-after-free bugs at compile time:

1. Each value has exactly one owner at any point in time.
2. When the owner goes out of scope, the value is dropped automatically.
3. Ownership can be transferred (moved) from one variable to another.

These are enforced by the compiler, providing memory safety without a garbage collector.

### Stack vs Heap: Where Does Data Live?

**By default, Rust allocates all data on the stack**, just like C++. To explicitly allocate on the heap, use `Box<T>`, `Vec<T>`, `String`, or similar heap-allocating types.

> For deeper technical background, you can reference our [comprehensive memory layout guide](/rust/concepts/2025/01/05/rust-mem-ref.html). But you can read it later after this article.

#### Stack Allocation (Default)

```rust
struct Point {
    x: f64,
    y: f64,
}

fn main() {
    let point = Point { x: 3.0, y: 4.0 }; // Stack-allocated
    println!("{} bytes on stack", std::mem::size_of_val(&point));
}
```


#### Heap Allocation (Explicit)

```rust
fn main() {
    let boxed = Box::new(Point { x: 3.0, y: 4.0 }); // Heap-allocated
    // Box stores pointer (8 bytes on 64-bit) on stack, data on heap
}
```


#### Hybrid: Stack Struct with Heap-Allocated Fields

Many types like `String`, `Vec<T>`, and `HashMap` are stack-allocated but contain pointers to heap memory:

```rust
struct Person {
    name: String,        // Stack struct pointing to heap data
    age: u32,            // Stack-allocated
    hobbies: Vec<String>, // Stack struct pointing to heap data
}

fn main() {
    let person = Person {
        name: String::from("Alice"),
        age: 30,
        hobbies: vec![
            String::from("Reading"),
            String::from("Gaming"),
        ],
    };
    // person struct on stack: ~50 bytes
    // "Alice", "Reading", "Gaming" on heap
}
```

#### When Stack vs Heap Matters

For **most Rust code**, you don't consciously choose stack vs heap. Instead:

1. **Primitives and small types**: Stack automatically (design-time choice by the type)
2. **Large data or unknown size**: Heap automatically via `Vec`, `String`, `Box` (design-time choice by the type)
3. **Dynamic collections**: Always heap (runtime size, so must be heap)
4. **Thread-local data**: Usually stack within `thread_local!` blocks
5. **Global state**: Usually static (fixed address)

**You only micromanage allocation when profiling reveals a bottleneck.** Rust's type system encourages correct choices by default. Premature optimization—wrapping everything in `Box` or prematurely chunking heap allocations—adds complexity without measurable benefit.

For detailed allocation analysis, see our [memory layout guide](/rust/concepts/2025/01/05/rust-mem-ref.html).

### Move Semantics: The Default Behavior

**Move semantics (ownership transfer) are the default for all types in Rust.** When you assign a value to another variable or pass it to a function, ownership moves to the new location. After the move, the original binding becomes invalid and the compiler prevents further use.

> **Critical point**: Even stack-allocated types move by default unless they explicitly implement the `Copy` trait. Just because data lives on the stack doesn't mean it uses Copy trait behavior.

```rust
struct Point { x: i32, y: i32 } // Moves by default

fn main() {
    let p1 = Point { x: 1, y: 2 };
    let p2 = p1;  // Ownership moves to p2
    // println!("{:?}", p1);  // ERROR: p1 moved
    println!("{:?}", p2);     // OK
}
```


#### Move on Assignment

```rust
fn main() {
    let s1 = String::from("hello");
    let s2 = s1; // Ownership transfers to s2

    // println!("{}", s1); // ERROR: s1 no longer valid
    println!("{}", s2);    // OK
}
```


#### Move When Passing to Functions

```rust
fn takes_ownership(s: String) {
    println!("{}", s);
} // s dropped here

fn main() {
    let s = String::from("hello");
    takes_ownership(s); // s moved into function

    // println!("{}", s); // ERROR: s moved
}
```


#### Move When Returning from Functions

Functions can transfer ownership to the caller:

```rust
fn gives_ownership() -> String {
    String::from("hello")  // Ownership transferred to caller
}

fn main() {
    let s = gives_ownership();
    println!("{}", s);  // OK: s owns the string
}
```


### Copy Trait: Opt-In Stack Semantics (Marker for Safe Bitwise Duplication)

⚠️ **Critical clarification**: **All types use move semantics (ownership transfer) by default.** The `Copy` trait doesn't create a separate "Copy trait behavior" mode. Instead, it's a permission that **allows the compiler to bitwise-duplicate values as an implementation detail** instead of tracking ownership transfer through the type system. Without `Copy`, ownership tracking is explicit; with `Copy`, bitwise duplication is permitted.

> Even stack-allocated types move by default unless they explicitly implement the 
> `Copy` trait. Stack allocation is not related to move vs. Copy trait behavior.

```rust
struct Point { x: i32, y: i32 }  // Non-Copy: uses move semantics (ownership transfer)

#[derive(Copy, Clone)]
struct Point { x: i32, y: i32 }  // Copy: compiler bitwise-copies instead of tracking moves
```

#### Why Copy Requires Certain Constraints

For a type to implement `Copy`:
- **`Copy` is a marker trait**: It has no methods and exists only to signal "bitwise duplication is semantically safe."

The compiler auto-generates bitwise duplication when you assign or pass `Copy` values; `Clone::clone()` is the explicit user-facing counterpart for deep copies.

#### Common Copy Types

Types that can safely implement `Copy` (and usually do):

- All integer types: `i8`, `u32`, `i64`, etc.
- Boolean: `bool`
- Floating-point: `f32`, `f64`
- Character: `char`
- Function pointers: `fn()`
- **Immutable references: `&T`** — Safe to copy because multiple pointers to the same data don't violate aliasing rules. Copying a pointer doesn't affect the borrow.
- **NOT mutable references: `&mut T`** — Cannot be copied because `Copy` would break the exclusivity guarantee. If you could copy `&mut T`, two mutable references to the same data would exist, violating "one writer, many readers."
- Raw pointers: `*const T`, `*mut T`
- Tuples of Copy types: `(i32, i32)`, `(bool, char)`


```rust
// &T IS Copy (let me demonstrate):
let x = 5;
let r1: &i32 = &x;
let r2 = r1;   // r1 is copied here (bitwise duplication of the pointer)
let r3 = r1;   // r1 is copied again
// r1, r2, r3 all point to the same data: multiple readers = safe

// &mut T is NOT Copy:
let mut y = 10;
let m1: &mut i32 = &mut y;
// let m2 = m1;  // ERROR: cannot copy exclusive reference
// Reason: if copying were allowed, m1 and m2 would both claim exclusive access 
// to the same data—a data race.

```
**Why `&T` is Copy but `&mut T` is not:**

Copy means "bitwise-duplicate the bytes creates a valid independent copy". For &mut T, duplicating the bytes creates TWO pointers, each claiming exclusive access to the same data. This violates the fundamental exclusivity guarantee. Bitwise duplication of an exclusive pointer = data race.

`Copy` means the compiler can **bitwise-duplicate** the value (copy the bytes) instead of moving ownership. For `&T` (immutable reference), duplicating the pointer is safe—multiple pointers to the same read-only data don't violate the borrowing rules. But for `&mut T`, bitwise duplication would create multiple independent mutable pointers, each believing they have exclusive access. This violates the core safety invariant. Therefore, `&mut T` cannot be `Copy`.


```rust
#[derive(Copy, Clone)]
struct Ref<'a, T: 'a>(&'a T);  // If this were implemented, it would be Copy

let x = 5;
let r1: &i32 = &x;  // r1 copies freely
let r2 = r1;        // r2 is a copy of r1's bits (same pointer)
let r3 = r1;        // r3 is also a copy of r1's bits

// Multiple readers of the same data via different pointers: ✅ SAFE

// Now imagine mutable references were Copy (they're not):
let mut y = 10;
let m1: &mut i32 = &mut y;
// let m2 = m1;  // If Copy, would bitwise-copy the pointer
// let m3 = m1;  // If Copy, would bitwise-copy the pointer again
// Now m1, m2, m3 all point to the same mutable data, each thinking they have 
// exclusive access: ❌ UNSAFE
// Bitwise duplication of exclusive references = data race
//
// Therefore, &mut T cannot implement Copy. Copy means "bitwise duplication is 
// safe," and bitwise duplication of exclusive pointers violates Rust's aliasing rules.
```

#### Copy trait behavior in Action

```rust
#[derive(Copy, Clone)]
struct Point { x: i32, y: i32 }

fn main() {
    let p1 = Point { x: 1, y: 2 };
    let p2 = p1;  // Copy: bitwise duplication, both valid
    println!("{:?} and {:?}", p1, p2); // Both still valid
    
    process_point(p1);  // Copy passed to function
    println!("{:?}", p1);  // Still valid after function call
}

fn process_point(p: Point) {
    println!("{:?}", p);
}
```

With `Copy`, the original binding remains valid because the compiler bitwise-copies the value instead of tracking ownership transfer.

**Critical clarification:** Copy trait behavior apply only to how values transition between scopes. Function calls with Copy types still "pass" the value (the compiler bitwise-copies it), but from the programmer's perspective, the original binding remains valid because `Copy` authorizes the compiler to duplicate instead of tracking ownership transfer. This is an implementation detail—the semantics are "the function receives an independent copy."

#### Why Copy Requires Certain Constraints

For a type to implement `Copy`:

- It must be stored entirely on the stack (no heap allocations)
- It cannot implement `Drop` (which would require compiler-controlled cleanup logic)
- It must implement `Clone` (a requirement enforced by the compiler). Types deriving `Copy` must also derive or implement `Clone` because `Copy` is semantically a promise that bitwise duplication is safe. Since `Clone::clone()` is the user-facing way to duplicate values, `Copy` implicitly requires it. The compiler makes both derivable together: `#[derive(Copy, Clone)]`.
- **`Copy` is a marker trait**: It has no methods and exists only to signal that bitwise duplication is semantically equivalent to value semantics. This means `Copy` is purely a compile-time marker indicating "duplicating the bits creates a valid, independent copy."

> Why Copy requires Clone: If a type is Copy, the compiler auto-duplicates it. To ensure users have an explicit way to request duplication, Copy requires Clone—the user-facing method for duplication. Together, they guarantee bitwise duplication is both automatic (compiler) and explicit (user code via clone()).


Additionally, a type cannot implement `Drop` and `Copy` simultaneously. If a type requires custom cleanup logic (Drop), it is inherently tied to a specific owner, so bitwise copying would bypass that cleanup, causing resource leaks or double-frees. This is enforced by the compiler:


```rust
#[derive(Copy)]
struct FileHandle { /* ... */ }

impl Drop for FileHandle {  // ERROR: cannot implement Drop for Copy type
    fn drop(&mut self) { /* cleanup */ }
}
```


This mutual exclusion ensures that every value's cleanup is guaranteed to run exactly once.


```rust
#[derive(Copy, Clone)]
struct Safe { x: i32, y: i32 }  // OK: all Copy fields

// #[derive(Copy, Clone)]
// struct Unsafe { data: String }  // ERROR: String not Copy

// #[derive(Copy, Clone)]
// struct Unsafe { data: Box<i32> }  // ERROR: Box has Drop
```


### Non-Copy Types: Move-Only Data

Types that allocate heap memory or implement `Drop` **cannot** be `Copy` and therefore use move semantics (ownership transfer):

```rust
#[derive(Debug)]
struct Person {
    name: String,
}

fn main() {
    let p1 = Person { name: String::from("Alice") };
    let p2 = p1;  // Move: p1 invalid after this
    
    // println!("{:?}", p1);  // ERROR: moved
    println!("{:?}", p2);     // OK
}
```


#### Common Move Types

- `String`: Heap-allocated text
- `Vec<T>`: Heap-allocated array
- `Box<T>`: Heap-allocated single value
- `HashMap<K, V>`: Heap-allocated mapping
- Any custom struct containing move types


#### Move Semantics in Action

```rust
fn main() {
    let s1 = String::from("hello");
    let s2 = s1;  // Move to s2

    // println!("{}", s1);  // ERROR: Cannot use s1 after move
    println!("{}", s2);     // OK

    let numbers = vec![1, 2, 3];
    take_ownership(numbers);  // Move into function
    // println!("{:?}", numbers);  // ERROR: moved
}

fn take_ownership(v: Vec<i32>) {
    println!("{:?}", v);
}
```


### The Clone Trait

If you need to create a deep copy of heap-allocated data while keeping the original, use `clone()`:

```rust
fn main() {
    let s1 = String::from("hello");
    let s2 = s1.clone();  // Deep copy of heap data

    println!("{}", s1);  // OK: both valid
    println!("{}", s2);  // OK: both valid
}
```

Cloning is explicit and potentially expensive because it duplicates heap memory. Use it when you genuinely need two independent copies.

***

### Move Semantics and Drop

When a value is **moved** to a new location, the compiler ensures `Drop` is called exactly once—at the end of the new owner's scope, not the old binding's scope. This is critical for resource management:

```rust
struct File {
    fd: i32,
}

impl Drop for File {
    fn drop(&mut self) {
        println!("Closing file descriptor {}", self.fd);
        // In real code: close_fd(self.fd)
    }
}

fn main() {
    let f1 = File { fd: 3 };
    let f2 = f1;  // f1 moved to f2; f1 is invalid

    // f1's scope ends, but Drop is NOT called (f1 no longer owns anything)
    // f2's scope ends; Drop IS called (f2 owns the file)
    // Output: "Closing file descriptor 3" (exactly once)
}
```
This is why move semantics (ownership transfer) guarantee resource safety. The `Drop` trait cooperates with ownership transfer to eliminate double-frees and resource leaks.

## Part Five: Non-Lexical Lifetimes (NLL)

**Note:** This section explains how borrow *scopes* work in Rust 2018+. Full lifetime syntax (explicit annotations like `'a`) is covered in our [separate lifetimes guide](/rust/concepts/2025/02/09/rust-ownership.html). This section focuses on the borrow checker's inference, not on lifetime parameters.

### The Problem NLL Solves

Before Non-Lexical Lifetimes (stabilized in Rust 2018), the borrow checker used lexical block scopes to determine how long **borrows** lasted. A borrow would last from its creation until the end of the entire enclosing block, even if the reference was never used again. This was overly conservative and rejected valid code:

```rust
fn main() {
    let mut scores = vec![1, 2, 3];
    let score = &scores;      // Shared borrow begins
    println!("{:?}", score);    // Last use of score
    // score's borrow ends here—it's not used after this line
    scores.push(4);             // ERROR in pre-2018 Rust: mutable access conflicts
}
```

In pre-2018 Rust, `score` would be "borrowed" until the end of the `main` function, preventing the `push()`. A human knows the borrow is dead, but the old checker couldn't see that.


### How NLL Works

> NLL applies to all borrow types, but the improvement is most dramatic for shared borrows. Mutable borrows were already relatively restricted, so the impact is less visible.

NLL changes the borrow checker to determine the **precise endpoint of each borrow based on control-flow analysis**, not lexical scope. The compiler identifies the last point in the control-flow graph where a reference is actually **used**, and the borrow ends after that point. This enables earlier reuse of the binding:

```rust
fn main() {
    let mut s = String::from("hello");

    let r1 = &s;
    let r2 = &s;
    println!("{} and {}", r1, r2);  // Final use of r1 and r2 here
    
    // NLL: Borrows end after the last use (argument to println!), not after println! returns
    let r3 = &mut s;  // OK: r1 and r2 no longer borrowed
    r3.push_str(" world");
    println!("{}", r3);
}

```

The shared references `r1` and `r2` end after `println!`, so the mutable reference `r3` can be created.

### NLL in Practice

```rust
fn main() {
    let mut data = vec![1, 2, 3];
    let first = &data;
    println!("First: {}", first);  // Last use

    data.push(4);  // OK: first is no longer active
    println!("Data: {:?}", data);
}
```

Without NLL, this would fail because `first` would be considered "borrowed" until the end of the scope. With NLL, the borrow ends after `println!`, so mutation is allowed.

***

Borrow Scope Inference (NLL) is a compiler optimization you don't need to think about—it just makes the borrow checker less conservative.

## Part Six: Borrowing and References

> Lifetime syntax and reference semantics are covered deeply in a [separate guide](/rust/concepts/2025/02/09/rust-ownership.html) after you've mastered the concepts in this article. But you can read it later after this article.

**Note on method call optimization:** In Part Eight, we'll cover two-phase borrows, a compiler optimization that allows certain patterns (like `v.push(v.len())`) to work despite appearing to conflict with borrowing rules. This is a compile-time convenience; understanding basic borrowing rules first is essential.

### Binding Mutability vs Reference Mutability

**These are independent concepts.** Binding mutability (controlled by `let mut`) determines whether you can reassign the variable. Reference mutability (controlled by `&` vs `&mut`) determines whether a reference has permission to modify the data.

Knowing a binding is mutable tells you nothing about what type of reference `&` will create:

```rust
let s = String::from("hi");
let r = &s;  // Immutable reference

let mut s = String::from("hi");
let r = &s;  // Still immutable reference! mut on binding doesn't affect &

let mut s = String::from("hi");
let r = &mut s;  // Mutable reference—binding AND reference are mutable
```


### The Four Combinations

| Binding | Reference | Example | Behavior |
| :-- | :-- | :-- | :-- |
| Immutable | Immutable | `let s = String::from("hi"); let r = &s;` | Read-only; cannot modify or rebind |
| Immutable | Mutable | `let s = String::from("hi"); let r = &mut s;` | **Compile error**: Cannot create mutable reference from immutable binding |
| Mutable | Immutable | `let mut s = String::from("hi"); let r = &s;` | Read-only through reference; binding can be rebound |
| Mutable | Mutable | `let mut s = String::from("hi"); let r = &mut s;` | Can modify through reference and rebind binding |


```rust
// ❌ Does not compile:
let s = String::from("hi");
let r = &mut s; // ERROR: cannot borrow s as mutable because it is not declared as mutable

// ✅ Fix: declare s as mutable
let mut s = String::from("hi");
let r = &mut s; // OK

```
### Shared References (&T)

A shared reference lets you read a value without taking ownership. Create one with the `&` operator:

```rust
fn main() {
    let s = String::from("hello");
    let len = calculate_length(&s);

    println!("Length of '{}' is {}", s, len);  // s still valid
}

fn calculate_length(s: &String) -> usize {
    s.len()
}
```

The function borrows the string without taking ownership, so the string is not dropped when the function returns.

You can have multiple shared references to the same value simultaneously:

```rust
fn main() {
    let s = String::from("hello");
    let r1 = &s;
    let r2 = &s;
    println!("{} and {}", r1, r2);  // Both valid simultaneously
}
```

**Shared references are `Copy`**: Each reference is a pointer (8 bytes on 64-bit systems). Copying a reference duplicates the pointer, creating an independent reference to the same underlying data. This doesn't violate aliasing rules because multiple readers are safe.

```rust
fn read_string(s: &String) {
    println!("{}", s);
}

let text = String::from("hello");
read_string(&text);  // Reference copied (8 bytes)
read_string(&text);  // Can pass again; previous reference was Copy
```

**String literals as shared references:**

```rust
let text: &'static str = "hello";  // String literals have 'static lifetime
let r1 = text;                      // r1 copies the reference
let r2 = text;                      // r2 copies the reference
```
String literals are immutable and live for the program's entire duration, making them particularly copy-friendly.


### References to Non-Copy Types: Why Cloning the Reference Doesn't Clone the Data

A common mistake is attempting to clone a reference to get the underlying data:

```rust
let vec_ref: &Vec<String> = &vec![String::from("hello")];
// let vec_clone = vec_ref.clone();  // ❌ ERROR: &Vec<String> doesn't implement clone
```

The reference itself is Copy (it's just a pointer), so cloning it creates another pointer to the same data, not a copy of the data. To duplicate the underlying Vec:

```rust
let vec_clone = vec_ref.clone();        // ✅ Wait, this DOES work!
let vec_clone = (*vec_ref).clone();     // ✅ Explicit deref then clone
let vec_clone = vec_ref.as_slice().to_vec();  // ✅ Alternative via slice
```

Actually, Vec implements Clone, so vec_ref.clone() works due to deref coercion—the compiler automatically dereferences the reference to call clone on the Vec. This is convenient but important to understand: you're cloning the Vec, not the reference. The distinction matters when working with types that don't implement Clone.

### Mutable References (&mut T)

A mutable reference lets you modify a borrowed value:

```rust
fn main() {
    let mut s = String::from("hello");
    change(&mut s);
    println!("{}", s);  // prints "hello, world"
}

fn change(s: &mut String) {
    s.push_str(", world");
}
```

**Mutable references do NOT implement `Copy`** because Rust guarantees only one mutable reference exists at a time. When you pass a mutable reference to a function, special handling occurs (reborrowing, covered below).

### The Borrowing Rules

Rust enforces two strict rules about references:

1. **Aliasing XOR Mutability (revisited):** At any point in the program, you can have either multiple concurrent shared references (`&T`) to the same data OR exactly one exclusive mutable reference (`&mut T`), but never both active simultaneously. This rule is enforced across **all execution paths** via the borrow checker's flow-sensitive analysis.
    Why does this matter?
    - **Iterator invalidation prevention**: You cannot mutate a collection while iterating (`&mut vec.push()` while `vec.iter()` is active = error)
    - **Safe aliasing under mutation**: Compiler can assume mutable references have exclusive access, enabling optimizations that would be unsafe with aliased pointers
    - **No data races**: Multiple threads reading is safe; one writer is safe; both simultaneously is caught at compile time
2. **No dangling references:** A reference must not outlive the data it points to. The compiler prevents returning references to local variables, which would point to deallocated memory.

```rust
fn main() {
    let mut s = String::from("hello");

    let r1 = &s;
    let r2 = &s;
    // let r3 = &mut s;  // ERROR: cannot have mutable reference while shared refs exist

    println!("{} and {}", r1, r2);  // OK
}
```


### Dangling References Prevention

The compiler prevents dangling references—references to freed memory:

```rust
fn dangle() -> &String {  // ERROR: cannot return reference to local data
    let s = String::from("hello");
    &s  // s is dropped; reference invalid
}

// Correct approach: return owned value
fn no_dangle() -> String {
    let s = String::from("hello");
    s  // Ownership transferred to caller
}
```


***

## Part Seven: Parameter Passing Mechanisms

Rust uses three mechanisms for parameter passing, all with zero runtime cost and enforced at compile time:


| Mechanism | Applies To | What Happens | Original After Call |
| :-- | :-- | :-- | :-- |
| **Copy** | Types implementing `Copy` | Compiler auto-duplicates bitwise; original binding stays valid | Valid, unchanged |
| **Move** | Non-`Copy` types (default for all types) | Ownership transfers to function; original binding becomes invalid | Invalid (compile error if accessed) |
| **Borrow (NLL)** | References (`&T` and `&mut T`) | Reference passed; borrow ends at last use via NLL; original remains usable when borrow ends | Valid; borrow-checker tracks timing |

### Copy: Trivial Duplication

```rust
fn square(x: i32) -> i32 {  // i32 implements Copy
    x * x
}

let n = 5;
square(n);   // n copied; original binding remains valid
square(n);   // Can use n again
```


### Move: Ownership Transfer

```rust
fn consume(s: String) {
    println!("{}", s);
}

let text = String::from("hello");
consume(text);  // text moved; ownership transferred
// consume(text);  // ERROR: value used after move
```


### Borrow Management with NLL

```rust
fn update(v: &mut Vec<i32>) {
    v.push(42);
}

let mut data = vec![1, 2];
update(&mut data);  // Mutable borrow occurs and ends within call
println!("{:?}", data);  // OK: borrow already ended
update(&mut data);  // Can call again
```


***

## Part Eight: Advanced Borrowing Patterns

### Two-Phase Borrows

Two-phase borrows are a **compiler optimization that applies exclusively to method calls** where the receiver (`self`) and an argument both borrow the same data. This special handling does NOT apply to free function calls. Misunderstanding when two-phase borrows apply causes confusion in production code.

**Key principle**: Two-phase borrows are a **convenience**, not a general rule. The compiler has special handling for the `receiver.method(args)` syntax but not for `function(args)`, due to limitations in reasoning about argument evaluation order in free functions. Understanding when two-phase borrows apply—and critically, when they don't—prevents confusing borrow checker errors in real code.


**Method call (two-phase borrow applies):**

```rust

fn main() {
    let mut v = vec![];
    v.push(v.len());  // Looks like conflict: mutable borrow (push) + shared borrow (v.len)
    println!("{:?}", v);
}

```

Here's what happens internally:

1. `v.len()` is evaluated first, creating a temporary shared borrow
2. After all arguments are evaluated, the mutable borrow for `push(&mut self, ...)` becomes active
3. Borrows never overlap in time—reading ends before writing begins

**Free function call (two-phase borrow does NOT apply):**

```rust

fn process(v: &mut Vec<usize>, len: usize) {
    v.push(len);
}

fn main() {
    let mut v = vec![];

    // ❌ Does NOT work: free functions don't get two-phase borrow treatment
    // process(&mut v, v.len());  // ERROR: cannot borrow mutably and immutably
    
    // ✅ Workaround: separate the borrow operations
    let len = v.len();
    process(&mut v, len);
}

```

**Why the difference?** The Rust compiler has special handling for the method call syntax `receiver.method(args)`. For regular function calls `function(args)`, the compiler cannot reliably reason about the argument evaluation order, so it's conservative.


```rust
// ❌ Does NOT work: free functions don't get two-phase borrow treatment
process(&mut v, v.len());  // ERROR: cannot borrow mutably and immutably

// ✅ Workaround: separate the borrow operations
let len = v.len();
process(&mut v, len);
```

**Why this restriction?** The compiler cannot guarantee the evaluation order of function arguments. By separating the operations into distinct statements, you explicitly order the borrows: the immutable borrow (`v.len()`) ends before the mutable borrow (`process(&mut v, ...)`) begins.

**Method calls differ:** With `receiver.method(args)`, the receiver is always evaluated first, and arguments are evaluated left-to-right. This deterministic order allows two-phase borrows to work.


**Quick recap:** Two-phase borrows are a **compiler optimization for method calls only**. They enable patterns like `v.push(v.len())` by ensuring argument evaluation completes before method application. Free functions don't receive this treatment because the compiler conservatively reasons about argument order. When you encounter a borrow checker error in a free function call, separate the borrows into distinct statements.

**Practical guidance**: If you get a borrow checker error with function calls, split the borrow into separate statements rather than relying on argument evaluation ordering.

### Reborrowing (Mutable References Only)

Reborrowing creates a new mutable reference from an existing mutable reference. The original reference becomes suspended until the reborrow ends. **In practice, you rarely write explicit reborrow syntax—the compiler handles this implicitly when passing mutable references to functions.**

**Explicit reborrow (uncommon):**

```rust
fn main() {
    let mut x = 5;
    let r1 = &mut x;
    let r2 = &mut *r1;  // Explicit reborrow syntax using dereference
    *r2 += 1;
    *r1 += 1;  // r1 usable after r2's scope ends
    println!("{}", x);  // prints 7
}
```

**Implicit reborrow (common pattern):**

```rust
fn modify(x: &mut i32) {
    *x += 1;
}

fn main() {
    let mut n = 0;
    let r = &mut n;
    modify(r);  // Compiler implicitly reborrows; r remains valid
    modify(r);  // Can call again—previous reborrow already ended
    println!("{}", n);  // prints 2
}
```


The function receives a temporary reborrow of `r`. When the function returns, the reborrow ends and `r` becomes usable again. This is why you can call `modify(r)` multiple times.

**Important:** You are not explicitly writing reborrow syntax in this code. The compiler **implicitly reborrows** whenever you pass a mutable reference to a function. This is a convenience mechanism—the compiler converts `modify(r)` into `modify(&mut *r)` automatically, suspending `r` during the call and resuming it afterward. This implicit reborrow is why mutable references feel flexible despite the "one mutable ref at a time" rule.

**Implicit reborrow in iterators:**


```rust
let mut v = vec![1, 2, 3];
let r = &mut v;

for item in &*r {  // Implicit: borrows the iterator from r
    println!("{}", item);
}

r.push(4);  // OK: implicit borrow ended
```


In this example, the `for` loop implicitly reborrows `r` to iterate. When the loop exits, the reborrow ends and `r` is available again.

### Closures and the `move` Keyword

Closures (anonymous functions) capture variables from their environment by reference by default. To transfer ownership into a closure, use the `move` keyword:

```rust
fn main() {
    let s = String::from("hello");
    
    // Without move: closure borrows s
    let borrowed = || println!("{}", s);
    borrowed();
    println!("{}", s);  // Still valid
    
    // With move: closure takes ownership of s
    let moved = move || println!("{}", s);
    moved();
    // println!("{}", s);  // ERROR: s moved into closure
}

```
This is essential when passing closures to threads or storing them in data structures:

```rust
use std::thread;

let numbers = vec![1, 2, 3];

// ✅ Correct: move captures ownership
let handle = thread::spawn(move || {
    for n in numbers {
        println!("{}", n);
    }
});

handle.join().unwrap();
// println!("{:?}", numbers);  // ERROR: numbers moved

```

Without `move`, the closure would hold a reference to `numbers`, but `numbers` lives on the main thread's stack. When the thread spawned, that reference would outlive the original scope, violating the no-dangling-references rule.

### Partial Moves: A Production Pitfall

When you move individual fields out of a struct, the struct becomes "partially moved"—some fields are gone while others remain accessible. This asymmetry causes real production bugs because the compiler allows accessing unmoved `Copy` fields while forbidding whole-struct access.


**This is a real source of production bugs.**

After a partial move, you cannot use the entire struct via dot notation, even though you can access unmoved fields. This asymmetry causes confusion and introduces subtle errors.

#### The Problem

```rust

#[derive(Debug)]
struct Point {
    x: i32,        // Copy
    y: String,     // Non-Copy; can be moved
}

fn main() {
    let p = Point {
    x: 10,
    y: String::from("hello"),
};

    let y_val = p.y;  // Move: ownership of y transferred out of p
    
    println!("{}", p.x);      // ✅ OK: x is Copy, still valid
    // println!("{:?}", p);   // ❌ ERROR: p is partially moved; cannot use as a whole
    // println!("{}", p.y);   // ❌ ERROR: y was moved out; invalid access
    }

```

The asymmetry: `p.x` works because `x` implements `Copy`, but `p` (the whole struct) is invalid because `y` moved. This is confusing because the compiler allows accessing `p.x` but forbids using `p`.

#### The Fix: Use Destructuring

**Pattern**: When extracting fields from mixed `Copy`/`Move` structs, use destructuring to make ownership transfer explicit:

```rust

fn main() {
    let p = Point {
        x: 10,
        y: String::from("hello"),
    };

    // Destructure: explicitly separate Copy and Move fields
    let Point { x, y } = p;

    // Now ownership transfer is clear:
    use_x(x); // x copied (Copy trait)
    use_y(y); // y moved

    // No surprises: p is no longer accessible (intentional)
}

fn use_x(x: i32) {
    println!("x: {}", x);
}
fn use_y(y: String) {
    println!("y: {}", y);
}


```

This pattern eliminates the confusing mix of "some fields work, but the whole struct doesn't."

#### Updating Fields After Partial Moves

If you need to update a field after a partial move, rebuild the struct:

```rust

struct Data {
    id: u32,         // Copy
    content: String, // Non-Copy
}

fn main() {
    let d = Data {
        id: 42,
        content: String::from("data"),
    };

    let content = d.content; // Move out
    let d = Data {
        id: d.id, // Can still read d.id (Copy)
        content: String::from("updated"),
    };

    println!("{:?}", d); // OK: d is fully reconstructed
}

```

This pattern makes ownership flow explicit: the moved field is gone, and you're intentionally creating a new struct value.


#### Why This Matters in Production

```rust

// ❌ Antipattern found in real code:
impl Data {
    fn process(mut self) {
        let config = self.config.clone();  // Move out of config field
        let result = self.compute();       // Uses self (partially moved!) — confusing
        // Later: someone adds self.config.log() by mistake → confusing error
    }
}

// ✅ Better:
impl Data {
    fn process(self) {
        let Data { config, .. } = self;  // Explicit destructure
        let result = self.compute();     // Clear that self is no longer valid
    }
}

```


This pattern appears **extremely frequently in web frameworks** where request/response handlers extract fields. Misunderstanding partial moves causes real production bugs where code compiles but handlers mysteriously fail.

```rust
pub async fn handle_request(req: HttpRequest) -> HttpResponse {
    let body = req.body().to_vec();  // Move
    
    // Later: someone adds logging that tries to use the whole request
    tracing::error!("Request failed: {:?}", req);  // ❌ COMPILE ERROR
}
```

The error seems random because they don't understand partial moves. Fix: destructure at entry point:

```rust
pub async fn handle_request(req: HttpRequest) -> HttpResponse {
    let HttpRequest { body, headers, method, .. } = req;
    
    // Now it's clear: req is gone; individual fields are available
    let body_bytes = body.to_vec();
    tracing::debug!("Method: {}", method);  // ✅ Clear and works
}

```

#### A Pattern to Avoid: Partial Moves in Request Handlers

Partial moves commonly appear in request/response handlers where developers extract fields without realizing the struct becomes partially-moved:


```rust
#[derive(Debug)]
pub struct Request {
    pub id: u32,           // Copy
    pub body: Vec<u8>,     // Non-Copy; can be moved
    pub headers: String,   // Non-Copy; can be moved
}

// ❌ COMMON MISTAKE: Partial move in handler
fn process_request(mut req: Request) {
    let body = req.body;  // Move out
    
    // Log the request... but what do we log?
    println!("Request: {:?}", req);  // ERROR: req is partially moved
    
    // Even though these work:
    println!("ID: {}", req.id);  // OK: id is Copy
    
    // The original object is unusable as a whole
    save_metadata(&req);  // ERROR: can't pass partially-moved struct
}

// ✅ FIX: Destructure at the entry point
fn process_request(req: Request) {
    let Request { id, body, headers } = req;
    
    // Ownership transfer is now explicit
    handle_body(body);
    handle_headers(headers);
    
    // id is independent; no confusion
    log_request_id(id);
    
    // No attempt to use `req` (which doesn't exist anymore)
}
```

This pattern appears in request/response handlers, event processors, and async tasks where fields need to be extracted and moved to different handlers.

#### Real-World Fix: Extracting State at Entry Points

Production-grade handlers should extract mutable state at entry and pass immutable views to downstream functions:

```rust
// ✅ PRODUCTION PATTERN: Separate extraction from processing
#[derive(Debug)]
pub struct Request {
    pub id: u32,
    pub body: Vec<u8>,
    pub headers: String,
}

// Extract at entry; pass immutable references to handlers
pub async fn handle_request(req: Request) -> Response {
    let Request { id, body, headers } = req;
    
    let parsed_body = parse_body(&body);
    let request_headers = HeaderMap::from(&headers);
    
    // Downstream handlers receive what they need; no RefCell required
    process_with_headers(&parsed_body, &request_headers)
        .await
}

fn process_with_headers(body: &[u8], headers: &HeaderMap) -> Response {
    // Pure function; no state coordination needed
    Response::ok()
}

```

This pattern eliminates interior mutability entirely by ensuring handlers receive exactly what they need at entry points.

## Part Nine: Why Interior Mutability Is Out of Scope

Interior mutability (`Cell`, `RefCell`, `UnsafeCell`) allows mutation through shared references by deferring borrow checking to runtime. While powerful, these patterns:

1. **Require runtime checks** that can panic (`RefCell`)
2. **Bypass compiler guarantees** (you must manually ensure safety)
3. **Belong in advanced guides** focused on `unsafe` abstractions

Since this guide focuses on **compiler-verified safe patterns**, we intentionally skip interior mutability. For global state, the patterns in Part Ten (atomics, `Mutex`, `LazyLock`) provide thread-safe alternatives without runtime borrow checking panics.

We will cover interior mutability in another post (coming soon...)

## Part Ten: Safe Global State Patterns

### Atomic Types for Counters and Flags

For simple counters and flags, atomic types provide thread-safe operations without locks:

```rust
use std::sync::atomic::{AtomicUsize, Ordering};

static COUNTER: AtomicUsize = AtomicUsize::new(0);

fn main() {
    COUNTER.fetch_add(1, Ordering::SeqCst);
    println!("Counter: {}", COUNTER.load(Ordering::SeqCst));
}
```

**Memory ordering matters**. The `Ordering` parameter determines how the operation synchronizes with other threads:

- **`Ordering::Relaxed`**: No synchronization; no memory fence. Use only for statistics where exact accuracy doesn't matter. ⚠️ **Unsafe on weak-memory architectures (ARM) for coordination patterns.**

- **`Ordering::Acquire`/`Ordering::Release`**: One-way synchronization. Release writes are visible to subsequent Acquire reads. Sufficient for most coordination patterns (signaling, flags). Better performance on ARM than SeqCst. **Use this for ~90% of real-world coordination.**

- **`Ordering::SeqCst`**: Total ordering; full memory fence on both sides. Safest but carries performance costs on weak-memory systems (ARM, PowerPC). **Use only when documenting why other orderings are insufficient.**

****Platform Reality Check**: On x86-64 (Intel, AMD), the x86-TSO memory model is strong, so `Acquire`/`Release` and `SeqCst` compile to nearly identical machine code. On weak-memory architectures (ARM, PowerPC, RISC-V), `SeqCst` requires additional memory barriers, resulting in measurable performance costs. For portable code, default to `Acquire`/`Release` unless you document why `SeqCst`'s total ordering is required. Benchmark on your target platform if performance is critical.


**Quick Decision Tree for Memory Ordering:**

1. Is this a statistics counter (hit counts, metrics)?
   → Use `Ordering::Relaxed` (fastest, no sync overhead)

2. Are you signaling readiness/completion between threads?
   → Use `Ordering::Release` (writer) + `Ordering::Acquire` (reader)
   → Most coordination patterns; good ARM performance

3. Do you have multiple independent atomic variables that must be coordinated?
   → Reach for `Ordering::SeqCst` ONLY after confirming Relaxed/Acquire-Release don't suffice
   → Document WHY SeqCst is necessary for future maintainers

4. Multithreaded coordination you're not 100% sure about?
   → Default to `SeqCst`, document the question, benchmark later
   → Correctness first; optimize after profiling shows need

This tree prevents over-engineering and ensures correct choices for 90% of real code.


**Memory Ordering Practical Examples**:

```rust
// ❌ WRONG: SeqCst for a statistics counter (overkill, expensive, especially on ARM)
static PAGE_VIEWS: AtomicU64 = AtomicU64::new(0);
fn record_view() {
    PAGE_VIEWS.fetch_add(1, Ordering::SeqCst);  // Unnecessary full fence
}

// ✅ RIGHT: Relaxed for non-critical statistics
static PAGE_VIEWS: AtomicU64 = AtomicU64::new(0);
fn record_view() {
    PAGE_VIEWS.fetch_add(1, Ordering::Relaxed);  // No sync overhead
}

// ✅ RIGHT: Acquire/Release for thread coordination (95% of use cases)
static READY: AtomicBool = AtomicBool::new(false);
// Thread A:
READY.store(true, Ordering::Release);  // Signal readiness; visibility guaranteed
// Thread B:
while !READY.load(Ordering::Acquire) { }  // Wait for signal; sees Thread A's writes

// ✅ RIGHT: SeqCst only when documented
static INIT_COMPLETE: AtomicBool = AtomicBool::new(false);
// SeqCst needed here because we must establish a total order across
// multiple synchronization variables
```

```rust
// Statistics counter: Relaxed is safe (accuracy loss is acceptable)
static REQUESTS: AtomicU64 = AtomicU64::new(0);
fn record_request() {
    REQUESTS.fetch_add(1, Ordering::Relaxed);
}

// Single boolean flag signaling initialization completion: Release/Acquire
static INITIALIZED: AtomicBool = AtomicBool::new(false);
// Thread A:
INITIALIZED.store(true, Ordering::Release);  // Writers use Release
// Thread B:
while !INITIALIZED.load(Ordering::Acquire) { }  // Readers use Acquire
// Guarantees: Thread B sees all of Thread A's writes before the flag

// Impossible to use Relaxed for flags; weak synchronization breaks the pattern
```


**When to use each:**

- **Relaxed:** Statistics (hit counters, telemetry). Accuracy loss is acceptable, performance critical.
- **Acquire/Release:** Synchronization between threads (flags, condition variables). Balances safety and performance across architectures.
- **SeqCst:** Multi-variable coordination requiring total order. Use only when you can document why weaker orderings fail. Most code doesn't need this.


**Practical guidance**: Use `Relaxed` for stats, `Acquire`/`Release` for coordination, and `SeqCst` only when you can document why weaker orderings fail.

### Mutex and RwLock

For more complex shared state, `Mutex` and `RwLock` provide safe access:

```rust
use std::sync::Mutex;

static NAMES: Mutex<Vec<String>> = Mutex::new(Vec::new());

fn main() {
    {
        let mut names = NAMES.lock().unwrap();  // Acquire lock
        names.push(String::from("Alice"));
        names.push(String::from("Bob"));
    }  // MutexGuard dropped here, releasing the lock automatically

    let names = NAMES.lock().unwrap();  // Can acquire lock again
    println!("Names: {:?}", names);
}  // Second MutexGuard dropped here
```

`Mutex` allows only one thread to access the data at a time. `RwLock` allows multiple readers OR one writer, mirroring Rust's borrowing rules.

**Important**: `lock()` blocks until the lock is acquired. For non-blocking behavior, use `try_lock()`.

> **Best Practice:** Always document panic conditions and deadlock risks if you expose global locks. Acquiring a poisoned or recursive lock will panic; use `.lock().expect("mutex not poisoned")` for clearer error messages.


### OnceLock for One-Time Initialization

`OnceLock` enables one-time initialization with external setup:

```rust
use std::sync::OnceLock;

static CONFIG: OnceLock<String> = OnceLock::new();

fn main() {
    CONFIG.set(String::from("production")).unwrap();
    println!("Config: {}", CONFIG.get().unwrap());
    
    // CONFIG.set(...);  // ERROR: already set
}
```

`set()` succeeds only once. The more ergonomic `get_or_init()` handles initialization in one call:

```rust
static DB_CONNECTION: OnceLock<String> = OnceLock::new();

fn get_db() -> &'static str {
    DB_CONNECTION.get_or_init(|| {
        println!("Initializing database connection...");
        String::from("postgres://localhost")
    })
}

fn main() {
    println!("{}", get_db());
    println!("{}", get_db());  // Second call uses cached value
}
```

**Thread-safety guarantee**: Only one thread's closure executes; others block until initialization completes, preventing duplicate initialization costs.

### LazyLock for Lazy Initialization (Preferred for 2024+)

`LazyLock` is the **preferred pattern for lazy static initialization in Rust 2024 and later**. It provides automatic lazy evaluation with a cleaner API than `OnceLock`:

```rust
use std::sync::LazyLock;

static EXPENSIVE: LazyLock<Vec<i32>> = LazyLock::new(|| {
    println!("Initializing...");
    vec![1, 2, 3, 4, 5]
});

fn main() {
    println!("Before access");
    println!("{:?}", *EXPENSIVE);  // Initialization happens here
    println!("{:?}", *EXPENSIVE);  // Uses cached value
}
```

**Design philosophy**: `LazyLock` is simpler than `OnceLock` for the common pattern where initialization logic is known at definition time. `OnceLock` shines when initialization parameters come from runtime sources.

**When to use LazyLock vs OnceLock:**

- **LazyLock**: Initialization logic is known at definition time (e.g., `LazyLock::new(|| { parse_config_file() })`)
- **OnceLock**: Initialization comes from runtime sources external to the definition (e.g., accepting a value from `fn set()` called elsewhere)


**Example distinguishing the two:**

```rust
// LazyLock: initialization at definition
static CONFIG: LazyLock<Config> = LazyLock::new(|| {
    Config::from_file("app.toml") // Known at definition time
});

// OnceLock: initialization external
static RUNTIME_VALUE: OnceLock<String> = OnceLock::new();

fn main() {
    let user_input = read_user_input();
    RUNTIME_VALUE.set(user_input).unwrap(); // Set externally
    println!("{}", RUNTIME_VALUE.get().unwrap());
}
```

**Design philosophy**: `LazyLock` simplifies the common pattern where initialization is self-contained; `OnceLock` is for decoupled initialization.

### LazyCell for Thread-Local Lazy Initialization

⚠️ **Availability Note**: `LazyCell` was stabilized in Rust 1.80.0 (July 2024). If your MSRV (Minimum Supported Rust Version) is earlier, use the external `once_cell` crate, which provides `once_cell::unsync::Lazy` (equivalent to `LazyCell`) for thread-local and non-thread-safe contexts. Many production codebases still target Rust 1.70 or earlier, so check your project's MSRV before using this feature.

**For projects using `once_cell` crate:**


```rust
use once_cell::unsync::Lazy;  // Replace std::cell::LazyCell

thread_local! {
    static BUFFER: Lazy<Vec<u8>> = Lazy::new(|| {
        Vec::with_capacity(4096)
    });
}
```

+⚠️ **Critical:** `LazyCell` is **not thread-safe**. Use it only inside `thread_local!` blocks or single-threaded contexts. Attempting to share a `LazyCell` across threads (or pass it to another thread) will cause data races and undefined behavior.

```rust

use std::cell::LazyCell;

thread_local! {
    static BUFFER: LazyCell<Vec<u8>> = LazyCell::new(|| {
        println!("Allocating per-thread buffer");
        Vec::with_capacity(4096)
    });
}

fn main() {
    BUFFER.with(|buf| {
        println!("Capacity: {}", buf.capacity());
        // Each thread has its own BUFFER instance
    });
}

```

**Design distinction**: 
- `LazyCell` is to `RefCell` as `LazyLock` is to `Mutex`
- Use `LazyCell` inside `thread_local!` for per-thread lazy initialization
- Use `LazyLock` for program-wide lazy initialization


### Arc<Mutex<T>> for Shared Ownership Across Threads

When multiple threads need to own and mutate shared data:

```rust
use std::sync::{Arc, Mutex};
use std::thread;

fn main() {
    let counter = Arc::new(Mutex::new(0));
    let mut handles = vec![];

    for _ in 0..3 {
        let counter = Arc::clone(&counter);  // Explicit: clones the Arc (cheap pointer copy)
        // Note: counter.clone() also works, but Arc::clone() is preferred in
        // production code because it makes the shallow pointer copy explicit,
        // reducing reader confusion about data duplication costs.
        let handle = thread::spawn(move || {
            let mut num = counter.lock().expect("Counter mutex was poisoned; a thread panicked while holding it");
            *num += 1;
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    println!("Final: {}", *counter.lock().unwrap());
}
```

**Critical production notes:**
- `lock()` panics if the mutex is poisoned (a thread panicked while holding the lock). Use `lock().expect("mutex not poisoned")` for clearer errors.
- `lock()` blocks indefinitely; use `try_lock()` to avoid potential deadlocks.
- Avoid acquiring multiple locks in different orders across threads; this causes deadlocks.
- Hold locks for the **minimum time** needed; long-held locks reduce concurrency.

For production code, document lock acquisition order and deadlock prevention strategy.

`Arc` (Atomic Reference Counting) enables shared ownership with atomic reference counting. Unlike statics, `Arc` values are dynamic and can be created/destroyed at runtime.

**Production consideration:** The `.unwrap()` here will panic if the mutex is poisoned (a thread panicked while holding the lock). For production code, use `.expect(msg)` to provide context. Better yet, structure your code to avoid panicking while holding locks, or use `.lock()` in contexts where poisoning is acceptable (e.g., per-thread operations where a poison indicates a fatal error).

***

## Part Eleven: Best Practices and Decision Guide

### Choosing Between Const and Static

**Use `const` when:**

- The value is known at compile time and never changes
- You don't need a fixed memory address
- The value is small and you want it inlined
- Examples: mathematical constants, configuration values, lookup tables

```rust
const PI: f64 = 3.14159265359;
const MAX_CONNECTIONS: usize = 100;
const FIBONACCI: [u32; 5] = [1, 1, 2, 3, 5];

fn calculate_circumference(radius: f64) -> f64 {
    2.0 * PI * radius  // PI inlined at compile time
}
```

**Use `static` when:**

- A single fixed memory address (essential for FFI—Foreign Function Interface with C/C++/other languages, which require stable memory addresses for data shared across language boundaries)
- The data is large and should not be duplicated
- You need global state initialized at runtime
- You need interior mutability for shared mutable state

```rust
use std::sync::OnceLock;
use std::sync::Mutex;

static CONFIG: OnceLock<AppConfig> = OnceLock::new();

#[derive(Clone)]
struct AppConfig {
    database_url: String,
}

fn get_config() -> &'static AppConfig {
    CONFIG.get_or_init(|| AppConfig {
        database_url: "postgres://localhost".to_string(),
    })
}

static COUNTER: Mutex<u64> = Mutex::new(0);

fn main() {
    let config = get_config();
    println!("Database: {}", config.database_url);
}
```
> **Warning:** Never use types with interior mutability (e.g., `AtomicU32`, `Cell`, `RefCell`) in a `const`. It compiles, but leads to dangerous, non-thread-safe behavior, and Clippy will warn. Use `static` for any atomic, cell, or lock type

### When to Move vs Borrow

**Move ownership when:**

- The caller no longer needs the value
- Transferring a resource with cleanup logic (file handles, connections)
- The function consumes the value to produce something new
- Performance optimization requires bypassing reference layers

**Borrow when:**

- The caller still needs the value after the call
- You only need to read the value
- You need temporary mutable access
- Designing library APIs that should work with many types

```rust
// MOVE: Takes ownership
fn open_and_read(path: &str) -> std::io::Result<String> {
    let mut file = std::fs::File::open(path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(contents)  // Ownership transferred to caller
}

// BORROW: Doesn't need ownership
fn count_lines(text: &str) -> usize {
    text.lines().count()
}
```


### Common Pitfalls

**Pitfall 1: Excessive Cloning**

Widespread cloning signals a design problem. Refactor to use borrowing strategically:

```rust
// WRONG
fn process(data: Vec<String>) -> Vec<String> {
    let mut result = Vec::new();
    for item in &data {
        result.push(item.clone());  // Unnecessary
    }
    let transformed = data.clone();  // Unnecessary
    transformed
}

// RIGHT
fn process(data: &[String]) -> Vec<String> {
    data.iter()
        .map(|s| format!("processed: {}", s))
        .collect()
}
```

**Pitfall 2: Fighting the Borrow Checker**

Borrow checker errors represent real safety issues. Understand the error instead of immediately reaching for `.clone()`:

```rust
// WRONG: Fighting the borrow checker
fn swap_bad(a: &mut i32, b: &mut i32) {
    let temp = *a;
    *a = *b;
    *b = temp;
}

// This doesn't work: swap_bad(&mut x, &mut x);  // ERROR: two mutable refs
// Because two mutable references to the same data are impossible

// RIGHT: Use the utility function
fn main() {
    let mut x = 5;
    let mut y = 10;
    std::mem::swap(&mut x, &mut y);
}
```

**Pitfall 3: Using `static mut` When Safer Alternatives Exist**

> **Guidance:** Declaring a `static mut` is almost always **avoided in practice**. Modern Rust (2024+) makes it a lint error to take references to `static mut`, and for good reason. Prefer interior mutability patterns — `Mutex`, `RwLock`, atomics, or `LazyLock` — for safe shared state. In rare FFI scenarios requiring C-compatible mutable statics, use atomics or re-architecture the FFI boundary to minimize unsafe code.

In Rust 1.90.0, `OnceLock`, `Mutex`, atomics, and other types cover nearly all use cases safely:

```rust
// WRONG: unsafe mutable static
static mut BAD_COUNTER: u64 = 0;

fn unsafe_increment() {
    unsafe {
        BAD_COUNTER += 1;  // Data races possible
    }
}

// RIGHT: use atomics
use std::sync::atomic::{AtomicU64, Ordering};

static GOOD_COUNTER: AtomicU64 = AtomicU64::new(0);

fn safe_increment() {
    GOOD_COUNTER.fetch_add(1, Ordering::SeqCst);
}
```


***

## Part Twelve: Choosing the Right Global State Pattern

| Pattern | When to Use | Example |
| :-- | :-- | :-- |
| **Atomic types** (`AtomicU32`, etc.) | Counters, flags, simple coordination | `static HITS: AtomicU64 = AtomicU64::new(0)` |
| **Mutex/RwLock** | Protected shared state with multiple threads | `static DATA: Mutex<Vec<_>> = Mutex::new(vec![])` |
| **LazyLock** | Lazy-initialized immutable static (preferred for 2024+) | `static CONFIG: LazyLock<AppConfig> = LazyLock::new(\|\| {...})` |
| **OnceLock** | One-time initialization from external sources | `static ONCE: OnceLock<T> = OnceLock::new()` then `ONCE.set(val)` |
| **Arc<Mutex<T>>** | Shared ownership across threads (heap-allocated) | `let shared = Arc::new(Mutex::new(data));` in threads |

**Quick decision:**
- Counters/flags → Atomic
- Shared mutable state → Mutex/RwLock
- Initialize once at compile time → LazyLock
- Initialize once at runtime (custom params) → OnceLock
- Multi-thread ownership (heap) → Arc<Mutex<T>>


## Conclusion

Rust's ownership system provides memory safety and thread safety guarantees that would require runtime overhead (garbage collection) or extensive manual verification in other languages. The borrow checker may seem strict initially, but it enforces patterns that are both safe and efficient.

The key mental model: **one owner at a time**. This rule, combined with the borrow checker, eliminates entire categories of bugs—use-after-free, double-free, data races—at compile time with zero runtime cost.

Master these concepts and you'll write Rust code that compiles cleanly and runs efficiently, with the compiler helping you catch mistakes that would cause subtle bugs in other languages.

For deeper exploration of memory layout specifics, see our [memory reference guide](/rust/concepts/2025/01/05/rust-mem-ref.html). For lifetime syntax and reference semantics, see our [ownership and lifetimes guide](/rust/concepts/2025/02/09/rust-ownership.html).

***