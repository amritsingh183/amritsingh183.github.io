---
layout: post
title: "Variables and Constants in Rust: A Complete Guide"
date: 2025-10-8 11:23:00 +0530
categories: rust concepts
---

# Variables and Constants in Rust

> **Note:** This document reflects Rust as of October 2025, including features from the Rust 2024 Edition (released February 2025 with Rust 1.85.0).

## Variables

### Immutability by Default

Rust variables are declared using the `let` keyword and are **immutable by default**. Once a value is bound to a variable, it cannot be changed unless explicitly declared as mutable.

```rust
fn main() {
    let x = 5;
    println!("The value of x is: {}", x);
    // x = 6; // ERROR: cannot assign twice to immutable variable
}
```


### Mutable Variables

To allow a variable's value to change, you must explicitly use the `mut` keyword:

```rust
fn main() {
    let mut y = 10;
    println!("The value of y is: {}", y);
    y = 20; // OK: y is mutable
    println!("The value of y is now: {}", y);
}
```

**Important**: Mutable variables cannot change their type - only their value can be modified.

### Variable Shadowing

Shadowing allows you to declare a new variable with the same name as a previous variable. The new variable "shadows" the original, and this can even change the variable's type:

```rust
fn main() {
    let x = 5;
    let x = x + 1; // Shadows the first x: now x is 6

    {
        let x = x * 2; // Shadows within inner scope: x is now 12
        println!("The value of x in the inner scope is: {}", x);
    }

    println!("The value of x is: {}", x); // x is 6 (outer scope)
}
```

Shadowing is particularly useful when transforming values or changing types:

```rust
fn main() {
    let spaces = "   "; // Type: &str
    let spaces = spaces.len(); // Type: usize - completely different type
    println!("Number of spaces: {}", spaces);
}
```


## Constants

### Declaring Constants

Constants are declared using the `const` keyword and are **always immutable**. Unlike variables, constants must have their type explicitly annotated:

```rust
const MAX_POINTS: u32 = 100_000;
const PI: f64 = 3.14159265359;
```


### Key Characteristics

Constants have several important properties:

1. **Type annotation is mandatory** - the compiler cannot infer the type
2. **Must be set to a constant expression** - cannot be the result of a function call or any value computed at runtime
3. **No fixed memory address** - they are inlined at compile time wherever used
4. **Can be declared in any scope**, including global scope
5. **Naming convention**: Use `UPPER_SNAKE_CASE`
```rust
// Valid constant expressions
const SECONDS_IN_MINUTE: u32 = 60;
const MINUTES_IN_HOUR: u32 = 60;
const SECONDS_IN_HOUR: u32 = SECONDS_IN_MINUTE * MINUTES_IN_HOUR;

// Invalid - function call at runtime
// const CURRENT_TIME: SystemTime = SystemTime::now(); // ERROR
```


### Constants vs Variables

| Feature | Constant (`const`) | Variable (`let`) |
| :-- | :-- | :-- |
| Can change? | No | Yes, if `mut` is used |
| Type required? | Yes | No (optional) |
| Memory location | No fixed address (inlined) | Has a stack location |
| Scope | Any scope, including global | Block scope |
| Runtime computation | No - compile-time only | Yes |

## Static Variables

### Basic Static Variables

Static variables are declared using the `static` keyword and represent global state with a fixed memory address:

```rust
static LANGUAGE: &str = "Rust";
static VERSION: i32 = 2024;

fn main() {
    println!("Language: {}, Version: {}", LANGUAGE, VERSION);
}
```


### Static vs Const

| Feature | `static` | `const` |
| :-- | :-- | :-- |
| Memory location | Fixed address | Inlined (no fixed address) |
| Performance | Slightly slower (load from address) | Faster (inlined) |
| Binary size | Smaller (when used frequently) | Larger (multiple copies) |
| Use case | Global state, FFI | Compile-time constants |

### Mutable Static Variables

Static variables can be mutable, but accessing or modifying them is **unsafe**:

```rust
static mut COUNTER: u32 = 0;

// Direct access (not a reference) to static mut
// this is the "older approach"
// it's accessing the value directly (not taking a reference)
fn increment_counter() {
    unsafe {
        COUNTER += 1;
    }
}

fn main() {
    unsafe {
        println!("COUNTER: {}", COUNTER);
        increment_counter();
        println!("COUNTER: {}", COUNTER);
    }
}
```

### Rust 2024 Edition: Changes to static mut

Starting with the Rust 2024 Edition (released February 2025), creating references to static mut variables is denied by default through the static_mut_refs lint. This is because taking a reference to a static mut creates instantaneous undefined behavior, even if the reference is never used.

**INCORRECT in Rust 2024 Edition:**

```rust
static mut COUNTER: u32 = 0;

fn main() {
    unsafe {
        let reference = &mut COUNTER;  // ERROR: reference to static mut
        *reference += 1;
    }
}
```

The correct approach in Rust 2024 Edition is to use raw pointers:

```rust
static mut COUNTER: u32 = 0;

fn main() {
    unsafe {
        let ptr = &raw mut COUNTER;  // OK: raw pointer, not reference
        *ptr += 1;
    }
}
```

Recommended Alternatives:

Instead of mutable static variables, consider these safer alternatives:

1. AtomicU64 for thread-safe counters without locks
2. Mutex<T> for thread-safe mutable data
3. LazyLock<T> for lazy initialization (stable in Rust 1.80+)
4. thread_local! for thread-local mutable state

Example using AtomicU64:

```rust
use std::sync::atomic::{AtomicU64, Ordering};

static COUNTER: AtomicU64 = AtomicU64::new(0);

fn increment_counter() {
    COUNTER.fetch_add(1, Ordering::SeqCst);  // No unsafe needed!
}

fn main() {
    increment_counter();
    println!("Counter: {}", COUNTER.load(Ordering::SeqCst));
}
```

**Warning**: Mutable static variables can cause data races in multithreaded contexts, which is why they require `unsafe` blocks.

### Static Lifetime

Static variables have the `'static` lifetime, meaning they live for the entire duration of the program:

```rust
static MESSAGE: &'static str = "This exists for the entire program";
```


## Ownership

### Ownership Rules

Rust's ownership system manages memory through three core rules:

1. Each value has one owner
2. When the owner goes out of scope, the value is deleted
3. There can only be one owner at a time; borrowing creates references, not additional owners.

### Move Semantics

For types that don't implement the `Copy` trait (like `String`), assignment **moves** ownership:

```rust
fn main() {
    let s1 = String::from("hello");
    let s2 = s1; // Ownership moves from s1 to s2

    // println!("{}", s1); // ERROR: s1 no longer owns the value
    println!("{}", s2); // OK: s2 now owns the value
}
```


### Copy Types

Simple types like integers, floats, booleans, and characters implement the `Copy` trait and are copied instead of moved. This also includes fixed-size arrays of Copy types and tuples containing only Copy types.

```rust
fn main() {
    let x = 5;
    let y = x; // x is copied to y

    println!("x: {}, y: {}", x, y); // Both work - x and y are independent
}
```


## Borrowing

### Immutable References

Instead of transferring ownership, you can **borrow** a value using references (`&`). Immutable references allow read-only access:

```rust
fn main() {
    let s1 = String::from("hello");
    let len = calculate_length(&s1); // Borrow s1

    println!("The length of '{}' is {}.", s1, len); // s1 is still valid
}

fn calculate_length(s: &String) -> usize {
    s.len()
} // s goes out of scope, but because it doesn't own the data, nothing happens
```


### Mutable References

Mutable references (`&mut`) allow modification of borrowed values:

```rust
fn main() {
    let mut s = String::from("hello");
    change(&mut s);
    println!("{}", s); // Prints "hello, world"
}

fn change(s: &mut String) {
    s.push_str(", world");
}
```


### Borrowing Rules

Rust enforces strict borrowing rules at compile time to prevent data races:

1. **At any given time**, you can have **either**:
    - One mutable reference (`&mut T`), **OR**
    - Any number of immutable references (`&T`)
2. References must always be valid (no dangling references)
```rust
fn main() {
    let mut s = String::from("hello");

    let r1 = &s; // OK: first immutable borrow
    let r2 = &s; // OK: second immutable borrow

    // let r3 = &mut s; // ERROR: cannot borrow as mutable
                        // because it's already borrowed as immutable

    println!("{} and {}", r1, r2);
    // r1 and r2 are no longer used after this point

    let r3 = &mut s; // OK: immutable borrows are no longer in scope
    r3.push_str(" world");
    println!("{}", r3);
}
```


### Non-Lexical Lifetimes (NLL)

Modern Rust uses **Non-Lexical Lifetimes**, meaning a reference's scope ends at its last use, not at the end of the block:

```rust
fn main() {
    let mut s = String::from("hello");

    let r1 = &s;
    let r2 = &s;
    println!("{} and {}", r1, r2);
    // r1 and r2 are no longer used, so their borrows end here

    let r3 = &mut s; // OK: no conflict with r1 and r2
    r3.push_str(" world");
    println!("{}", r3);
}
```


### Borrow Contagion

When you borrow a field of a struct, you indirectly borrow the entire struct. This is known as **borrow contagion**:

```rust
struct Car {
    engine: String,
    wheels: Vec<String>,
}

fn main() {
    let mut car = Car {
        engine: String::from("V8"),
        wheels: vec![String::from("wheel1"), String::from("wheel2")],
    };

    let engine_ref = &mut car.engine; // Borrows the entire car mutably

    // let wheel_ref = &car.wheels[0]; // ERROR: cannot borrow car as immutable
                                       // because it's already borrowed as mutable

    engine_ref.push_str(" Turbo");
} // Mutable borrow ends here
```


## Lifetimes

### Lifetime Annotations

Lifetimes are a way of describing how long references are valid. The Rust compiler usually infers lifetimes, but sometimes explicit annotations are needed:

```rust
// This function returns a reference with lifetime 'a
fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {
    if x.len() > y.len() {
        x
    } else {
        y
    }
}

fn main() {
    let string1 = String::from("long string");
    let string2 = String::from("short");

    let result = longest(string1.as_str(), string2.as_str());
    println!("The longest string is {}", result);
}
```

The lifetime annotation `'a` tells the compiler that the returned reference will be valid as long as both input references are valid.

### The `'static` Lifetime

The `'static` lifetime indicates that a reference is valid for the entire duration of the program:

```rust
let s: &'static str = "I have a static lifetime.";
```

String literals always have the `'static` lifetime because they're stored directly in the program's binary.

### Lifetime Elision

In many cases, Rust can infer lifetimes automatically without explicit annotations. For example:

```rust
// These two are equivalent
fn first_word(s: &str) -> &str { /* ... */ }
fn first_word<'a>(s: &'a str) -> &'a str { /* ... */ }
```


### Lifetime with Structs

When a struct holds references, it needs lifetime annotations:

```rust
struct ImportantExcerpt<'a> {
    part: &'a str,
}

fn main() {
    let novel = String::from("Call me Ishmael. Some years ago...");
    let first_sentence = novel.split('.').next().expect("Could not find a '.'");

    let excerpt = ImportantExcerpt {
        part: first_sentence,
    };

    println!("Excerpt: {}", excerpt.part);
}
```

The lifetime annotation `'a` ensures that the `ImportantExcerpt` instance cannot outlive the reference it holds.

## Best Practices

1. **Prefer `const` over `static`** when possible for better optimization
2. **Use immutable variables by default** and add `mut` only when necessary
3. **Leverage shadowing** for type transformations instead of multiple variable names
4. **Borrow instead of moving** ownership when possible to avoid unnecessary copies
5. **Use immutable references** unless mutation is required to allow multiple readers
6. **Avoid mutable static variables**. In Rust 2024 Edition, references to static mut are denied by default. Use interior mutability patterns (AtomicT, Mutex<T>, LazyLock<T>) instead.

## Summary

Rust's variable and borrowing system provides memory safety guarantees at compile time. Variables are immutable by default, encouraging safe concurrent programming. The ownership system, combined with borrowing rules, eliminates entire classes of bugs such as null pointer dereferences, use-after-free, and data races. Constants and static variables offer different performance characteristics for compile-time and global values. Understanding these concepts is fundamental to writing safe, efficient Rust code.
