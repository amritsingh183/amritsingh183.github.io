---
layout: post
title: "Mastering Variables, Constants and Lifetimes in Rust: A Complete Guide"
date: 2025-01-01 11:23:00 +0530
categories: rust concepts
last_updated: 2025-10-11
---

# A Complete Guide to Rust Ownership, Lifetimes, and Memory Management <a href="#a-complete-guide-to-rust-ownership-lifetimes-and-memory-management-" class="header-link">🔗</a>

> **Please note that memory layout is covered very briefly in this article. After reading this article you can later [check this](https://amritsingh183.github.io/rust/concepts/2025/01/05/rust-mem-ref.html) about memory layout used by Rust**


## Index <a href="#index-" class="header-link">🔗</a>

1. [Foundation: Mental Model](#1-foundation-mental-model)
    - [Aliasing XOR mutability principle](#aliasing-xor-mutability-principle)
    - [Prerequisites and goals](#prerequisites-and-goals)

2. [Variables in Rust](#2-variables-in-rust)
    - [Immutability by default](#immutability-by-default)
    - [Mutable variables](#mutable-variables)
    - [Variable shadowing](#variable-shadowing)
    - [Scope and dropping](#scope-and-dropping)
    - [The Drop trait](#the-drop-trait)
    - [Why Drop takes &mut self](#why-drop-takes-mut-self)
    - [Example: Drop implementation](#example-drop-implementation)

3. [Constants](#3-constants)
    - [Declaring constants](#declaring-constants)
    - [Naming conventions](#naming-conventions)
    - [When to use constants](#when-to-use-constants)
    - [Constants vs variables](#constants-vs-variables)

4. [Ownership Fundamentals](#4-ownership-fundamentals)
    - [The Three Ownership Rules](#the-three-ownership-rules)
    - [Memory and Allocation](#memory-and-allocation)
    - [Stack vs Heap: Where Does a Struct Live?](#stack-vs-heap-where-does-a-struct-live)
    - [Move Semantics](#move-semantics)
    - [Copy trait: Opt-in stack semantics](#copy-trait-opt-in-stack-semantics)
    - [Heap-allocated data and move](#heap-allocated-data-and-move)
    - [The Clone trait](#the-clone-trait)

5. [Borrowing and References](#5-borrowing-and-references)
    - [Binding mutability vs reference mutability](#binding-mutability-vs-reference-mutability)
    - [The four combinations](#the-four-combinations)
    - [Why &s is immutable](#why-s-is-immutable)
    - [Shared references (&T)](#shared-references-t)
    - [Mutable references (&mut T)](#mutable-references-mut-t)
    - [The borrowing rules](#the-borrowing-rules)
    - [Dangling references prevention](#dangling-references-prevention)

6. [Non-Lexical Lifetimes (NLL)](#6-non-lexical-lifetimes-nll)
    - [What NLL solves](#what-nll-solves)
    - [How NLL works](#how-nll-works)
    - [Examples with NLL](#examples-with-nll)

7. [Advanced Borrowing Patterns](#7-advanced-borrowing-patterns)
    - [Two-phase borrows](#two-phase-borrows)
    - [The Three Parameter-Passing Mechanisms](#the-three-parameter-passing-mechanisms)
    - [Reborrowing (applies only to mutable reference)](#reborrowing-applies-only-to-mutable-reference)
    - [Partial moves](#partial-moves)
    - [Interior mutability](#interior-mutability)

8. [Lifetimes](#8-lifetimes)
    - [What are lifetimes](#what-are-lifetimes)
    - [Lifetime annotations syntax](#lifetime-annotations-syntax)
    - [Lifetime elision rules](#lifetime-elision-rules)
    - [Lifetimes in structs](#lifetimes-in-structs)
    - [Lifetimes in methods](#lifetimes-in-methods)
    - [The 'static lifetime](#the-static-lifetime)

9. [Static Items](#9-static-items)
    - [What is static](#what-is-static)
    - [Static vs const comparison](#static-vs-const-comparison)
    - [When to use static](#when-to-use-static)
    - [Mutable statics and safety](#mutable-statics-and-safety)
    - [The Sync requirement](#the-sync-requirement)

10. [Rust 2024 Edition: Key Behaviors](#10-rust-2024-edition-key-behaviors)
    - [Mutable Static References Are Unsafe](#mutable-static-references-are-unsafe)
    - [Return-Position impl Trait Captures All In-Scope Lifetimes](#return-position-impl-trait-captures-all-in-scope-lifetimes)
    - [Temporaries in Tail Expressions Drop Earlier](#temporaries-in-tail-expressions-drop-earlier)

11. [Safe Global State Patterns](#11-safe-global-state-patterns)
    - [Atomic types](#atomic-types)
    - [Mutex and RwLock](#mutex-and-rwlock)
    - [OnceLock and LazyLock](#oncelock-and-lazylock)
    - [LazyCell for thread-local lazy initialization](#lazycell-for-thread-local-lazy-initialization)
    - [thread_local! Macro](#thread_local-macro)
    - [Arc<Mutex<T>> for shared ownership across threads](#arcmutext-for-shared-ownership-across-threads)
    - [Best Practices and Pitfalls](#best-practices-and-pitfalls)
    - [Performance Considerations](#performance-considerations)

12. [Best Practices and Decision Guide](#12-best-practices-and-decision-guide)
    - [Choosing between const and static](#choosing-between-const-and-static)
    - [When to move vs borrow](#when-to-move-vs-borrow)
    - [Ownership patterns in practice](#ownership-patterns-in-practice)
    - [Common pitfalls](#common-pitfalls)
    - [Performance considerations](#performance-considerations)

***

## 1. Foundation: Mental Model <a href="#1-foundation-mental-model-" class="header-link">🔗</a>

### Aliasing XOR mutability principle <a href="#aliasing-xor-mutability-principle-" class="header-link">🔗</a>

Rust's safety model is built on one core principle: you can have many readers or one writer, but not both at the same time.  This is called "aliasing XOR mutability" and it prevents data races at compile time.

Think of it like a library book: either many people can read it at once (shared access), or one person can write notes in it (exclusive access), but you cannot have someone writing while others are reading.

This rule is enforced by the borrow checker, which analyzes your code at compile time to make sure no two parts of your program can modify the same data simultaneously.

### Prerequisites and goals <a href="#prerequisites-and-goals-" class="header-link">🔗</a>

This guide assumes you know basic Rust syntax like variables, functions, and control flow, but you don't need prior systems programming experience.  The goal is to give you a clear mental model so you can design APIs and debug borrow checker errors with confidence.

***

## 2. Variables in Rust <a href="#2-variables-in-rust-" class="header-link">🔗</a>

### Immutability by default <a href="#immutability-by-default-" class="header-link">🔗</a>

In Rust, variables are immutable by default.  This means once you assign a value to a variable, you cannot change it unless you explicitly say it is mutable.

```rust
fn main() {
    let x = 5;
    println!("The value of x is: {}", x);
    // x = 6; // ERROR: cannot assign twice to immutable variable
}
```

If you try to change `x`, the compiler will stop you with an error.  This design encourages writing code with fewer side effects and clearer data flow.

### Mutable variables <a href="#mutable-variables-" class="header-link">🔗</a>

To allow a variable's value to change, add the `mut` keyword when declaring it:

```rust
fn main() {
    let mut y = 10;
    println!("The value of y is: {}", y);
    y = 20; // OK: y is mutable
    println!("The value of y is now: {}", y);
}
```

**Important**: Mutable variables can only change their value, not their type.  Once `y` is declared as an integer, it must remain an integer.

### Variable shadowing <a href="#variable-shadowing-" class="header-link">🔗</a>

Shadowing lets you declare a new variable with the same name as a previous variable.  The new variable "shadows" the old one, making the old one inaccessible.

```rust
fn main() {
    let x = 5;
    let x = x + 1;    // shadows the first x
    {
        let x = x * 2;  // shadows again, only in this scope
        println!("Inner x: {}", x); // prints 12
    }
    println!("Outer x: {}", x);     // prints 6
}
```

Shadowing is different from mutability because you are creating a new variable each time.  This means you can also change the type:

```rust
fn main() {
    let spaces = "   ";        // string type
    let spaces = spaces.len(); // now it's a number
    println!("{}", spaces);    // prints 3
}
```

You cannot do this with a mutable variable because mutability only lets you change the value, not the type.

### Scope and dropping <a href="#scope-and-dropping-" class="header-link">🔗</a>

Variables live within a scope, which is usually marked by curly braces `{}`. When a variable goes out of scope, Rust automatically cleans up its memory by calling the `Drop` trait.

```rust
fn main() {
    let s = String::from("hello"); // s is valid from here
    // you can use s here
} // s goes out of scope and is dropped here
// s is no longer valid here
```

This automatic cleanup is one of Rust's key features: no manual memory management, and no garbage collector.

#### The Drop trait <a href="#the-drop-trait-" class="header-link">🔗</a>

The `Drop` trait is a special trait that allows you to customize what happens when a value is dropped. Implementing `Drop` requires implementing a single method:

```rust
pub trait Drop {
    fn drop(&mut self) { }
}
```

#### Why Drop takes &mut self <a href="#why-drop-takes-mut-self-" class="header-link">🔗</a>

Notably, `Drop::drop` takes a mutable reference (`&mut self`), not ownership. This is a special exception to Rust's normal rules. Here's why this is safe and necessary:

**Compiler-controlled invocation**: You cannot explicitly call `drop()` on a non-mut binding—it's a compiler error . Only the compiler can invoke it as part of automatic cleanup during scope exit.

**Post-drop validity**: Because the mutable reference is created only during the drop and the value is about to be destroyed anyway, Rust allows creating a mutable reference to an immutable binding at this point. The `&mut self` is valid only during the `drop()` call itself .

**Resource cleanup requirement**: Destructors often need mutation to clean up resources. For example, a `Box<T>` needs to deallocate heap memory, which requires mutating internal state .

#### Example: Drop implementation <a href="#example-drop-implementation-" class="header-link">🔗</a>

```rust
struct SmartPointer {
    data: String,
}

impl Drop for SmartPointer {
    fn drop(&mut self) {
        println!("Dropping SmartPointer with data: {}", self.data);
        // self.data is automatically dropped after this function returns
    }
}

fn main() {
    let ptr = SmartPointer { 
        data: String::from("my data") 
    }; // ptr is not declared as mut
    
    // When ptr goes out of scope, the compiler implicitly calls:
    // Drop::drop(&mut ptr)
} // Output: "Dropping SmartPointer with data: my data"
```

Even though `ptr` is not declared `mut`, the compiler safely creates a mutable reference to it specifically for the `drop()` call, because this is the only place it happens and the value is about to be destroyed anyway.

This automatic cleanup is one of Rust's key features: no manual memory management, and no garbage collector.

***

## 3. Constants <a href="#3-constants-" class="header-link">🔗</a>

### Declaring constants <a href="#declaring-constants-" class="header-link">🔗</a>

Constants are declared with the `const` keyword and must always have a type annotation.  Constants can be declared in any scope, including the global scope.

```rust
const MAX_POINTS: u32 = 100_000;

fn main() {
    const HOURS_IN_DAY: u32 = 24;
    println!("Max points: {}", MAX_POINTS);
    println!("Hours: {}", HOURS_IN_DAY);
}
```


### Naming conventions <a href="#naming-conventions-" class="header-link">🔗</a>

Constants use SCREAMING_SNAKE_CASE by convention.  This makes them easy to spot in your code.

### When to use constants <a href="#when-to-use-constants-" class="header-link">🔗</a>

Use constants for values that never change and are known at compile time.  Examples include mathematical constants, configuration limits, or fixed array sizes.

```rust
const PI: f64 = 3.14159265359;
const MAX_BUFFER_SIZE: usize = 1024;
const THREE_HOURS: u32 = 60 * 60 * 3; // OK: computed at compile time
fn main() {
    // const RUNTIME_VAL: u32 = get_value(); // ERROR: cannot call functions in const
}
```
### Constants vs variables <a href="#constants-vs-variables-" class="header-link">🔗</a>


| Feature | `const` | `let` |
| :-- | :-- | :-- |
| Mutability | Always immutable; `mut` cannot be used. | Immutable by default, but can be made mutable with the `mut` keyword. |
| Type Annotation | Mandatory. The type must be explicitly declared. | Optional. The compiler can infer the type if not specified. |
| Value Assignment | Must be a constant expression evaluated at compile time. | Can be a value computed at runtime. |
| Memory Address | Does not have a fixed address; **the value is inlined by the compiler where it is used.** | Has a specific memory location, which the compiler manages. |
| Scope | Can be declared in any scope, including globally. | Restricted to the block in which it is declared. |



***
## 4. Ownership Fundamentals <a href="#4-ownership-fundamentals-" class="header-link">🔗</a>

### The Three Ownership Rules <a href="#the-three-ownership-rules-" class="header-link">🔗</a>

Rust's ownership system has three fundamental rules that prevent memory leaks, double frees, and use-after-free bugs at compile time:

1. Each value in Rust has exactly one owner at a time.
2. When the owner goes out of scope, the value is dropped automatically.
3. Ownership can be transferred (moved) from one variable to another.

These rules are enforced by the compiler, ensuring memory safety without requiring a garbage collector.

### Memory and Allocation <a href="#memory-and-allocation-" class="header-link">🔗</a>

Rust stores data in two places: the **stack** and the **heap**. The stack stores values with a known, fixed size, while the heap stores values that can grow or shrink at runtime. Understanding where data lives is crucial to understanding Rust's ownership model.

Types stored entirely on the stack (like integers, booleans, and simple structs) can implement the `Copy` trait, allowing them to be duplicated efficiently. Types that allocate heap memory (like `String` and `Vec<T>`) use move semantics to transfer ownership, preventing multiple owners from accessing the same heap allocation.

```rust
fn main() {
    let s = String::from("hello"); // Allocates heap memory
    // s is valid here
} // s goes out of scope, memory is freed automatically
```

When `s` goes out of scope, Rust calls the `drop` function automatically, freeing the heap memory.

### Stack vs Heap: Where Does a Struct Live? <a href="#stack-vs-heap-where-does-a-struct-live-" class="header-link">🔗</a>

**By default, Rust allocates all structs on the stack**, just like in C++. To store a struct on the heap, you must explicitly use heap-allocating types like `Box<T>`, `Rc<T>`, or `Arc<T>`.


#### Stack Allocation (Default) <a href="#stack-allocation-default-" class="header-link">🔗</a>

When you create a struct normally, it lives entirely on the stack:

```rust
struct Point {
    x: f64,
    y: f64,
}

struct Rectangle {
    top_left: Point,
    bottom_right: Point,
}

fn main() {
    // Both allocated on the stack
    let point = Point { x: 3.0, y: 4.0 };
    let rectangle = Rectangle {
        top_left: Point { x: 0.0, y: 0.0 },
        bottom_right: Point { x: 10.0, y: 10.0 },
    };

    // These structs occupy stack memory
    println!("Point occupies {} bytes on the stack", std::mem::size_of_val(&point));
    println!("Rectangle occupies {} bytes on the stack", std::mem::size_of_val(&rectangle));
}
```


#### Heap Allocation (Explicit) <a href="#heap-allocation-explicit-" class="header-link">🔗</a>

To allocate a struct on the heap, wrap it in `Box<T>` or similar smart pointers:

```rust
struct Point {
    x: f64,
    y: f64,
}

struct Rectangle {
    top_left: Point,
    bottom_right: Point,
}

fn main() {
    // Heap-allocated structs
    let boxed_point: Box<Point> = Box::new(Point { x: 3.0, y: 4.0 });
    let boxed_rectangle: Box<Rectangle> = Box::new(Rectangle {
        top_left: Point { x: 0.0, y: 0.0 },
        bottom_right: Point { x: 10.0, y: 10.0 },
    });

    // The Box itself occupies only pointer-size bytes on the stack
    // The actual struct data lives on the heap
    println!("Boxed point occupies {} bytes on the stack",
             std::mem::size_of_val(&boxed_point)); // Prints pointer size (8 bytes on 64-bit)
}
```


#### The Hybrid Case: Structs with Heap-Allocated Fields <a href="#the-hybrid-case-structs-with-heap-allocated-fields-" class="header-link">🔗</a>

Some structs are allocated on the stack but contain fields that point to heap memory. This is the case for types like `String`, `Vec<T>`, `HashMap<K, V>`, `Arc<T>`, and `Rc<T>`:

```rust
struct Person {
    name: String,        // Stack-allocated struct, but points to heap buffer
    age: u32,            // Stack-allocated
    hobbies: Vec<String>, // Stack-allocated struct, but points to heap buffer
}

fn main() {
    // The Person struct itself is on the stack
    // But name and hobbies contain pointers to heap-allocated buffers
    let person = Person {
        name: String::from("Alice"),  // "Alice" bytes live on heap
        age: 30,                       // Lives on stack as part of Person
        hobbies: vec![                 // Vec elements live on heap
            String::from("Reading"),
            String::from("Gaming"),
        ],
    };

    // The Person struct occupies a fixed size on the stack:
    // - name: 24 bytes (pointer + capacity + length)
    // - age: 4 bytes
    // - hobbies: 24 bytes (pointer + capacity + length)
    // But the actual string data lives on the heap
}
```


#### Memory Layout Rules <a href="#memory-layout-rules-" class="header-link">🔗</a>

***

To see a comprehensive breakdown of what goes where please check [this](https://amritsingh183.github.io/rust/concepts/2025/01/05/rust-mem-ref.html).

***

```rust
fn main() {
    // Stack-only struct: All fields are primitive types.
    // Total size: 8 bytes (two i32s) stored directly on the stack frame.
    struct StackOnly {
        x: i32,
        y: i32,
    }

    // Hybrid struct: Contains types that manage heap-allocated memory.
    // The struct itself lives on stack, but owns data stored on heap.
    struct HasHeapData {
        id: u32,              // Stored directly in struct on stack (4 bytes)
        name: String,         // Pointer + metadata on stack (24 bytes), actual string on heap
        scores: Vec<i32>,     // Pointer + metadata on stack (24 bytes), array elements on heap
    }

    // Explicitly heap-allocated struct: Box moves large data to heap.
    // Prevents stack overflow by keeping only a pointer on the stack.
    struct ExplicitHeap {
        data: Box<[u8; 1000]>, // Pointer on stack (8 bytes), array on heap (1000 bytes)
    }

    // Creates StackOnly instance: All 8 bytes allocated on current stack frame
    let stack_struct = StackOnly { x: 10, y: 20 };

    // Creates HasHeapData instance: Struct lives on stack (56 bytes)
    // 4 (id) + 4 (padding) + 24 (name) + 24 (scores) = 56 bytes.
    // String and Vec perform separate heap allocations for their contents.
    let hybrid_struct = HasHeapData {
        id: 1,                                             // Stored inline in struct
        name: String::from("Bob"),                         // "Bob" bytes allocated on heap
        scores: vec![95, 87, 92],                          // Three i32s allocated on heap
    };

    // Creates ExplicitHeap instance: Only pointer stored on stack (8 bytes)
    // The 1000-byte array is allocated on heap to avoid consuming stack space
    let heap_struct = ExplicitHeap {
        data: Box::new([0; 1000]),                         // Allocates 1000 bytes on heap
    };
}

```

#### Why This Matters for Ownership <a href="#why-this-matters-for-ownership-" class="header-link">🔗</a>

The distinction between stack and heap allocation is critical for understanding move vs. copy semantics:

- **Stack-only structs** can implement `Copy`, allowing them to be duplicated cheaply
- **Structs with heap allocations** must use move semantics to prevent double-free errors
- **Explicitly heap-allocated structs** (`Box<T>`) move ownership of the heap allocation, not the data itself

```rust
#[derive(Copy, Clone)]
struct StackPoint {
    x: i32,
    y: i32,
}

struct HeapContainer {
    data: Vec<i32>,
}

fn main() {
    // Copy: cheap bitwise duplication
    let p1 = StackPoint { x: 1, y: 2 };
    let p2 = p1;  // Copied, both valid
    println!("{}, {}", p1.x, p2.x);

    // Move: ownership transfer to prevent double-free
    let h1 = HeapContainer { data: vec![1, 2, 3] };
    let h2 = h1;  // Moved, h1 invalid
    // println!("{:?}", h1.data);  // ERROR: h1 was moved
    println!("{:?}", h2.data);     // OK
}
```


### Move Semantics <a href="#move-semantics-" class="header-link">🔗</a>

By default, Rust **moves** ownership when you assign a value to another variable or pass it to a function. **Move semantics apply to all types**
**by default**, whether stack-allocated or heap-allocated . For heap-allocated types, moving prevents double-free errors. For
stack-only types, moving is the default until you explicitly opt into `Copy` semantics.

> Move is universal unless a type has COPY trait

#### Move on Assignment <a href="#move-on-assignment-" class="header-link">🔗</a>

```rust
fn main() {
    let s1 = String::from("hello");
    let s2 = s1; // Ownership moves from s1 to s2

    // println!("{}", s1); // ERROR: s1 is no longer valid
    println!("{}", s2);    // OK: s2 is the owner now
}
```

After the move, `s1` is no longer valid. Only `s2` owns the string now.

#### Move When Passing to Functions <a href="#move-when-passing-to-functions-" class="header-link">🔗</a>

When you pass a heap-allocated value to a function, ownership moves into the function. The original variable becomes invalid in the caller's scope.

```rust
fn main() {
    let s = String::from("hello");
    takes_ownership(s); // s is moved into the function

    // println!("{}", s); // ERROR: s is no longer valid
}

fn takes_ownership(some_string: String) {
    println!("{}", some_string);
} // some_string is dropped here, memory is freed
```


#### Move When Returning from Functions <a href="#move-when-returning-from-functions-" class="header-link">🔗</a>

Functions can create values and transfer ownership to the caller by returning them. This extends the value's lifetime beyond the function scope.

```rust
fn main() {
    let s = gives_ownership(); // Ownership is transferred to s
    println!("{}", s);         // OK: s owns the string
}

fn gives_ownership() -> String {
    let some_string = String::from("hello"); // Local variable created
    some_string // Ownership is moved to the caller
} // some_string is NOT dropped because ownership was moved out
```


#### Taking and Returning Ownership <a href="#taking-and-returning-ownership-" class="header-link">🔗</a>

A common pattern in Rust is for functions to take ownership and return ownership back, allowing the function to modify the value:

```rust
fn main() {
    let s1 = String::from("hello");
    let s2 = takes_and_gives_back(s1); // s1 is moved in, s2 receives ownership back

    // println!("{}", s1); // ERROR: s1 was moved
    println!("{}", s2);    // OK: s2 owns the string
}

fn takes_and_gives_back(a_string: String) -> String {
    a_string // Ownership is returned to the caller
}
```

Ownership flows through your program: from variables to functions, from functions back to variables, ensuring that each value has exactly one owner at any given time.

### Copy Trait: Opt-In Stack Semantics <a href="#copy-trait-opt-in-stack-semantics-" class="header-link">🔗</a>

⚠️ **Note** As described earlier, all types in Rust are move by default unless they implement the COPY trait.

**Stack-only types CAN implement the `Copy` trait**, but only if you explicitly opt in. Types that implement `Copy` enable implicit bitwise
duplication. When you assign or pass a `Copy` type, Rust creates an independent copy rather than moving ownership . Both the original and the
copy remain valid, and no ownership transfer occurs. **Without explicit `Copy` implementation, stack-only types still use move semantics by default**

#### Critical Clarification: Copy is Always Opt-In <a href="#critical-clarification-copy-is-always-opt-in-" class="header-link">🔗</a>

⚠️ **Note** **Just because a type is stack-only does NOT mean it automatically uses copy semantics.** 

All types, even simple stack-only structs like the one given below, use **move semantics by default** . You must explicitly implement or derive `Copy` to change this behavior. This is why your `Point` struct moves ownership even though it contains only stack-allocated fields.

```rust
// This struct moves, even though it's entirely stack-allocated
struct Point { 
    x: i32, 
    y: i32,
}

// To make it copy, you must opt in:
#[derive(Copy, Clone)]
struct Point { 
    x: i32, 
    y: i32, 
}
```

#### Characteristics of Copy Types <a href="#characteristics-of-copy-types-" class="header-link">🔗</a>

Types that **can** implement `Copy` must be stored entirely on the stack and contain no heap allocations . The `Copy` trait is a marker
trait that depends on `Clone`, meaning any `Copy` type must also implement `Clone` . You cannot implement `Copy` for types that allocate heap
memory or implement the `Drop` trait. **Eligibility does not mean automatic implementation—you must explicitly derive or implement `Copy`** 

#### Common Copy Types <a href="#common-copy-types-" class="header-link">🔗</a>

- All integer types: `i8`, `i16`, `i32`, `i64`, `i128`, `u8`, `u16`, `u32`, `u64`, `u128`, `isize`, `usize`
- Boolean type: `bool`
- Floating-point types: `f32`, `f64`
- Character type: `char`
- Function pointers: `fn()`
- Immutable references: `&T` (but not mutable references `&mut T`)
- Raw pointers: `*const T` and `*mut T`
- Tuples containing only `Copy` types: `(i32, i32)`, `(bool, char, f64)`


#### Copy Semantics in Action <a href="#copy-semantics-in-action-" class="header-link">🔗</a>

```rust
#[derive(Copy, Clone, Debug)]
struct Point {
    x: i32,
    y: i32,
}

#[derive(Copy, Clone, Debug)]
enum Direction {
    North,
    South,
    East,
    West,
}

fn main() {
    println!("=== Stack-Only Data: Copy Semantics ===\n");

    // --- Primitive Types ---
    let num1 = 42;                          // `num1` stored on stack
    let num2 = num1;                        // `copy`: Bitwise copy created, both valid
    println!("i32: num1 = {}, num2 = {}", num1, num2); // OK: Both remain valid

    let flag1 = true;                       // `flag1` stored on stack
    let flag2 = flag1;                      // `copy`: Independent copy created
    println!("bool: flag1 = {}, flag2 = {}", flag1, flag2); // OK: Both remain valid

    let letter1 = 'A';                      // `letter1` stored on stack
    let letter2 = letter1;                  // `copy`: Bitwise copy created
    println!("char: letter1 = '{}', letter2 = '{}'\n", letter1, letter2); // OK: Both remain valid

    // --- Custom Struct with Copy ---
    let p1 = Point { x: 10, y: 20 };       // `p1` stored on stack
    let p2 = p1;                            // `copy`: Entire struct copied bitwise
    println!("Point: p1 = {:?}", p1);       // OK: p1 still valid
    println!("Point: p2 = {:?}\n", p2);     // OK: p2 is independent copy

    // --- Enum with Copy ---
    let dir1 = Direction::North;            // `dir1` stored on stack
    let dir2 = dir1;                        // `copy`: Enum variant copied
    println!("Direction: dir1 = {:?}", dir1); // OK: dir1 still valid
    println!("Direction: dir2 = {:?}\n", dir2); // OK: dir2 is independent copy

    // --- Passing to Functions ---
    process_number(num1);                   // `copy`: Copy passed to function
    println!("After function call, num1 is still valid: {}\n", num1); // OK: num1 unchanged

    process_point(p1);                      // `copy`: Struct copied to function
    println!("After function call, p1 is still valid: {:?}\n", p1); // OK: p1 unchanged

    // --- Returning from Functions ---
    let new_num = create_number();          // `copy`: Function returns a copy
    println!("Created number: {}", new_num);

    let new_point = create_point();         // `copy`: Function returns copy of struct
    println!("Created point: {:?}", new_point);
}

fn process_number(n: i32) {                 // `copy`: Receives copy of argument
    println!("Processing number: {}", n);
} // `n` goes out of scope, no special cleanup needed

fn process_point(p: Point) {                // `copy`: Receives copy of struct
    println!("Processing point: {:?}", p);
} // `p` goes out of scope, no special cleanup needed

fn create_number() -> i32 {
    let num = 100;                          // Local variable on stack
    num                                     // `copy`: Returns a copy
} // Original `num` goes out of scope normally

fn create_point() -> Point {
    let p = Point { x: 5, y: 15 };         // Local struct on stack
    p                                       // `copy`: Returns a copy
} // Original `p` goes out of scope normally
```


#### Key Takeaway <a href="#key-takeaway-" class="header-link">🔗</a>

With `Copy` types, assignment and function calls create independent copies. The original variable remains valid because no ownership transfer occurs. This behavior is safe because stack-only data is cheap to duplicate and doesn't require special cleanup.

### Heap-Allocated Data and Move <a href="#heap-allocated-data-and-move-" class="header-link">🔗</a>

Heap-allocated types do **not** implement `Copy` because copying them would create multiple owners of the same heap memory, leading to double-free errors. Instead, these types use **move semantics** to transfer ownership. After a move, the original variable becomes invalid, ensuring that only one owner exists at any time.

#### Characteristics of Move Types <a href="#characteristics-of-move-types-" class="header-link">🔗</a>

Types that allocate heap memory (like `String`, `Vec<T>`, `Box<T>`, and custom structs containing heap data) cannot implement `Copy`. When assigned or passed to functions, ownership moves from the source to the destination. The compiler prevents you from using the moved variable, guaranteeing memory safety.


#### Common Move Types <a href="#common-move-types-" class="header-link">🔗</a>

- `String`: Heap-allocated, growable text
- `Vec<T>`: Heap-allocated, growable array
- `Box<T>`: Heap-allocated single value
- `HashMap<K, V>`: Heap-allocated hash map
- Custom structs containing heap-allocated fields


#### Move Semantics in Action <a href="#move-semantics-in-action-" class="header-link">🔗</a>

```rust
#[derive(Debug)]
struct Person {
    name: String,              // String is heap-allocated
    age: u32,
}

fn main() {
    println!("=== Heap-Allocated Data: Move Semantics ===\n");

    // --- String (heap-allocated) ---
    let s1 = String::from("Hello, Rust!");  // `s1` owns heap-allocated string
    let s2 = s1;                            // `move`: Ownership transferred from s1 to s2
                                            // s1 is now invalid
    // println!("s1: {}", s1);              // ERROR: Cannot use s1 after move
    println!("s2: {}\n", s2);               // OK: s2 is the owner

    // --- Vec (heap-allocated) ---
    let v1 = vec![1, 2, 3];                 // `v1` owns heap-allocated vector
    let v2 = v1;                            // `move`: Ownership transferred to v2
                                            // v1 is now invalid
    // println!("v1: {:?}", v1);            // ERROR: Cannot use v1 after move
    println!("v2: {:?}\n", v2);             // OK: v2 is the owner

    // --- Custom Struct with Heap Data ---
    let person1 = Person {
        name: String::from("Alice"),        // Heap-allocated String inside struct
        age: 30,
    };
    let person2 = person1;                  // `move`: Entire Person moved, including String
                                            // person1 is now invalid
    // println!("person1: {:?}", person1);  // ERROR: Cannot use person1 after move
    println!("person2: {:?}\n", person2);   // OK: person2 owns the data

    // --- Passing to Functions ---
    let s3 = String::from("Moving to function");
    println!("Before function: s3 = {}", s3);
    process_string(s3);                     // `move`: Ownership moved into function
                                            // s3 is now invalid
    // println!("After function: {}", s3);  // ERROR: s3 was moved into function

    // --- Returning from Functions ---
    let s4 = create_string();               // `move out`: Ownership transferred to s4
    println!("\nCreated string: {}", s4);   // OK: s4 owns the returned string

    let person3 = create_person();          // `move out`: Ownership transferred to person3
    println!("Created person: {:?}\n", person3); // OK: person3 owns the struct

    // --- Taking and Returning Ownership ---
    let s5 = String::from("Original");
    println!("Original string: {}", s5);
    let s6 = modify_string(s5);             // `move in` then `move out`
                                            // s5 moved in, new value moved to s6
    // println!("s5: {}", s5);              // ERROR: s5 was moved
    println!("Modified string: {}\n", s6);  // OK: s6 owns the modified string
}

fn process_string(s: String) {              // `move in`: Takes ownership
    println!("Processing: {}", s);
} // `s` goes out of scope, Drop is called, heap memory freed

fn create_string() -> String {
    let local_string = String::from("Created in function"); // Local heap allocation
    local_string                            // `move out`: Ownership transferred to caller
} // local_string NOT dropped because ownership moved

fn create_person() -> Person {
    let local_person = Person {
        name: String::from("Bob"),
        age: 25,
    };
    local_person                            // `move out`: Ownership transferred to caller
} // local_person NOT dropped because ownership moved

fn modify_string(mut s: String) -> String { // `move in`: Takes ownership
    s.push_str(" - modified!");             // Modify owned data
    s                                       // `move out`: Return ownership
} // Nothing dropped because s was moved out
```

#### Why All Types Move by Default <a href="#why-all-types-move-by-default-" class="header-link">🔗</a>

The reason move is the default for **all types**—not just heap types—is conceptual integrity. Rust's ownership model is universal: one owner at a time
. Heap-allocated types must move to prevent double-free errors, but the principle applies to all types. Stack-only types that implement
`Copy` are an **exception to this rule**, not the rule itself .

This is why your `Point` struct moves by default—it hasn't opted into `Copy`:

```rust
struct Point { 
    x: usize,
    y: usize
} // Moves (default)

#[derive(Copy, Clone)] 
struct Point {
    x: usize,
    y: usize
} // Copies (opted in)
```

The difference is not about where the data lives—it's about whether the type has explicitly requested copy semantics . ```

#### Key Takeaway <a href="#key-takeaway-" class="header-link">🔗</a>

With heap-allocated types, assignment and function calls transfer ownership via moves. The original variable becomes invalid, preventing multiple owners from accessing the same heap memory. When ownership is returned from a function, it transfers to the caller, extending the value's lifetime.

### The Clone Trait <a href="#the-clone-trait-" class="header-link">🔗</a>

If you need to keep the original value valid after creating a copy of heap-allocated data, use the `clone` method. Cloning creates a deep copy, duplicating the heap allocation so both variables own independent data.

```rust
fn main() {
    let s1 = String::from("hello");
    let s2 = s1.clone(); // Creates a deep copy of the heap data

    println!("s1: {}", s1); // OK: s1 is still valid
    println!("s2: {}", s2); // OK: s2 has its own independent copy
}
```

Cloning is explicit and potentially expensive because it duplicates heap allocations. Use it when you genuinely need two independent copies of the data. For more details on the differences between `Copy` and `Clone`, refer to the dedicated trait documentation.

## 5. Borrowing and References <a href="#5-borrowing-and-references-" class="header-link">🔗</a>

Instead of transferring ownership, you can let a function borrow a value by passing a reference. But before we go further, let us understand `Binding Mutability vs Reference Mutability`

> **Binding Mutability vs Reference Mutability**

> A crucial distinction in Rust is that **binding mutability** and **reference mutability** are independent concepts. They operate on different levels and do not depend on each other.

> **Binding mutability** (controlled by `let mut`) determines whether you can reassign the variable to hold a different value. **Reference mutability** (controlled by `&` vs `&mut`) determines whether a reference has permission to modify the data it points to.

These two properties are orthogonal: knowing that a binding is mutable tells you nothing about what type of reference the `&` operator will create.

### The Four Combinations <a href="#the-four-combinations-" class="header-link">🔗</a>

You can have four combinations of binding and reference mutability:

| Binding | Reference | Example | Result |
|---------|-----------|---------|--------|
| Immutable | Immutable | `let s = String::from("hi"); let r = &s;` | Read-only, cannot rebind or modify through reference |
| Immutable | Mutable | `let s = String::from("hi"); let r = &mut s;` | **Compile error:** Cannot create mutable reference from immutable binding |
| Mutable | Immutable | `let mut s = String::from("hi"); let r = &s;` | Read-only through reference, but binding can be rebound |
| Mutable | Mutable | `let mut s = String::from("hi"); let r = &mut s;` | Can modify through reference and rebind the binding |

### Why `&s` is Immutable <a href="#why-s-is-immutable-" class="header-link">🔗</a>

The `&` operator always creates an **immutable reference**, regardless of whether the binding is mutable. The binding being mutable only means you can rebind the variable to a different value; it does not change what type of reference the `&` operator produces.


Now we can go ahead and continue our journey.

### Shared references (&T) <a href="#shared-references-t-" class="header-link">🔗</a>

A shared reference lets you read a value without taking ownership.  You create a shared reference with the `&` operator.

```rust
fn main() {
    let s1 = String::from("hello");
    let len = calculate_length(&s1); // borrow s1

    println!("Length of '{}' is {}", s1, len); // s1 is still valid
}

fn calculate_length(s: &String) -> usize {
    s.len()
} // s goes out of scope, but it doesn't own the data, so nothing happens
```

The `&s1` syntax creates a reference to `s1` without moving ownership.  The function `calculate_length` borrows the string but does not own it, so the string is not dropped when the function ends.

You can have multiple shared references to the same value at the same time:

```rust
fn main() {
    let s = String::from("hello");
    let r1 = &s;
    let r2 = &s;
    println!("{} and {}", r1, r2); // OK: multiple readers
}
```


### Mutable references (&mut T) <a href="#mutable-references-mut-t-" class="header-link">🔗</a>

> A mutable reference lets you modify a borrowed value.  You create it with `&mut`.

> **THEY DONT IMPLEMENT COPY, THEY ARE MOVE TYPE (except when reborrowed by functions)**

```rust
fn main() {
    let mut s = String::from("hello");
    change(&mut s);
    println!("{}", s); // prints "hello, world"
}

// as we will learn in the next section
// when passing mutable references to a function
// the mut ref is not moved
// instead compiler derefs it(not exposed to developers) it and 
// creates a new mutable ref `&mut *t` to it. This new mut ref gets moved into the function
// the orignal mut ref is suspended (because we can not have multiple writers)
fn change(some_string: &mut String) { 
    some_string.push_str(", world");
}
```


```rust
// WRONG, DOES NOT WORK
fn main() {
    let mut s = String::from("hello");
    let mutRef1: &mut String = &mut s;
    let mutRef2 = mutRef1; // REBORROW | &mut *mutRef1 | &mut does not implement COPY
    println!("{}", mutRef1); // compiler ERROR: borrow of moved value: mutRef1
    println!("{}", mutRef2);
}
```

```rust
// CORRECT, WORKS
fn main() {
    let mut s = String::from("hello");
    let mutRef1: &mut String = &mut s;
    let mutRef2 = mutRef1; // &mut *mutRef1 // &mut does not implement COPY
    println!("{}", mutRef2);
}
```

### The borrowing rules <a href="#the-borrowing-rules-" class="header-link">🔗</a>

Rust enforces two strict rules about references:

1. You can have either one mutable reference or any number of shared references, but not both at the same time.
2. References must always be valid (no dangling references).

These rules prevent data races at compile time.

Example of the first rule:

```rust
fn main() {
    let mut s = String::from("hello");

    let r1 = &s;     // OK: shared reference
    let r2 = &s;     // OK: another shared reference
    // let r3 = &mut s; // ERROR: cannot have mutable reference while shared references exist

    println!("{} and {}", r1, r2);
}
```

You cannot create a mutable reference if shared references already exist.

### Dangling references prevention <a href="#dangling-references-prevention-" class="header-link">🔗</a>

Rust's compiler prevents dangling references, which are references to memory that has been freed:

```rust
fn dangle() -> &String { // ERROR: this function tries to return a reference to local data
    let s = String::from("hello");
    &s // s is dropped here, so the reference would be invalid
} // the solution is to return the String itself, transferring ownership
```

The correct version returns the owned value:

```rust
fn no_dangle() -> String {
    let s = String::from("hello");
    s // ownership is moved to the caller
}
```


***

## 6. Non-Lexical Lifetimes (NLL) <a href="#6-non-lexical-lifetimes-nll-" class="header-link">🔗</a>

>> **"It's important to note: lexical scoping (determined by curly braces {}) defines where variables live and are dropped. Non-lexical lifetimes define where borrows end. These are different concepts—a variable may live longer than its borrows, and you can use the same variable again after its borrow has ended, even within the same lexical scope."** 

### What NLL solves <a href="#what-nll-solves-" class="header-link">🔗</a>

Before Non-Lexical Lifetimes, Rust used lexical scopes to determine how long borrows lasted.  This meant a borrow would last from the point it was created until the end of the block, even if it was never used again.

This older model was too conservative and rejected valid code:

```rust
fn main() {
    let mut scores = vec![1, 2, 3];
    let score = &scores; // borrow starts here
    scores.push(4);         // ERROR in old Rust: cannot modify while borrowed
                           // even though score is never used after this
}
```

A human can see that `score` is never used after the borrow, so there is no real problem.

### How NLL works <a href="#how-nll-works-" class="header-link">🔗</a>

Non-Lexical Lifetimes change the borrow checker to track borrows more precisely.  A borrow now ends at its last use, not at the end of the scope.

```rust
fn main() {
    let mut s = String::from("hello");

    let r1 = &s;
    let r2 = &s;
    println!("{} and {}", r1, r2); // last use of r1 and r2

    let r3 = &mut s; // OK: r1 and r2 are no longer in use
    r3.push_str(" world");
    println!("{}", r3);
}
```

With NLL, the shared references `r1` and `r2` end after the `println!`, so the mutable reference `r3` can be created safely.

### Examples with NLL <a href="#examples-with-nll-" class="header-link">🔗</a>

Another example showing NLL in action:

```rust
fn main() {
    let mut data = vec![1, 2, 3];
    let first = &data; // shared borrow
    println!("First element: {}", first); // last use of first

    data.push(4); // OK: first is no longer used
    println!("{:?}", data);
}
```

Without NLL, this would fail because `first` would be considered borrowed until the end of the function.  With NLL, `first` ends after its last use, so modifying `data` is allowed.

***

## 7. Advanced Borrowing Patterns <a href="#7-advanced-borrowing-patterns-" class="header-link">🔗</a>

### Two-phase borrows <a href="#two-phase-borrows-" class="header-link">🔗</a>

Two-phase borrows solve a specific problem: calling a method that takes `&mut self` while also reading from `self` in the arguments.

Consider this example:

```rust
fn main() {
    let mut v = vec![1, 2, 3];
    v.push(v.len()); // reads v.len() then mutably borrows v
    println!("{:?}", v); // prints [1, 2, 3, 3]
}
```

This looks like it should fail: `v.push()` takes `&mut self`, but we are also reading `v.len()` at the same time.  However, Rust uses two-phase borrows to make this work.

Here is how it works:

1. When you call `v.push(v.len())`, Rust first evaluates all the arguments.
2. During argument evaluation, only a shared borrow is needed for `v.len()`.
3. After all arguments are evaluated, the mutable borrow for `push` becomes active. **(There can be only one writer or many readers)**

This sequencing prevents overlap between the shared read and the mutable write.

### The Three Parameter-Passing Mechanisms <a href="#the-three-parameter-passing-mechanisms-" class="header-link">🔗</a>

When passing arguments to functions, Rust uses three distinct mechanisms depending on the type:

| Feature | Copy | Move | Reborrow |
| :-- | :-- | :-- | :-- |
| Applies to | Immutable references `&T` and stack-only types (`i32`, `bool`, `char`, etc.) | Owned types and heap-allocated data (`String`, `Vec<T>`, `Box<T>`) | Mutable references `&mut T` only |
| What happens | Bitwise duplication of the value | Ownership transfer; original loses access | Compiler creates temporary borrow with shorter lifetime |
| Original variable after call | Remains valid and usable | Invalid; cannot be used (moved) | Valid immediately; reborrow ends after function returns |
| Memory cost | Zero runtime cost; compiler optimization | Zero runtime cost; pointer transfer only | Zero runtime cost; pointer aliasing with lifetime constraints |
| Compiler assistance | Implicit; automatic bitwise copy | Implicit; automatic ownership transfer | Implicit; compiler transforms `f(x)` to `f(&mut *x)` when needed |
| Why this design | Stack-only data is cheap to duplicate | Prevents double-free errors; ensures single owner | Avoids awkward "take and return" pattern; enables ergonomic APIs |
| Example | `let x = 5; let y = x;` ✅ Both valid | `let s = String::from("hi"); let t = s;` ❌ `s` is moved | `vec.push(42);` ✅ `vec` still usable |


This explains why functions like `vec.push()` work without requiring the reference to be returned — the original mutable reference is immediately available again after the reborrow ends.

### Reborrowing (applies only to mutable reference) <a href="#reborrowing-applies-only-to-mutable-reference-" class="header-link">🔗</a>

Reborrowing happens when you create a new reference from an **existing mutable reference**.  The new reference temporarily "pauses" the original reference.

```rust
fn main() {
    let mut x = 5;
    let r1 = &mut x;    // first mutable borrow, r1 is the first writer
    let r2 = &mut *r1;  // reborrow: creates a new mutable reference, r2 is the second writer
    *r2 += 1;           // use r2. Can't use r1 until r2 is in use (only one writer can be there) 
    // r2 ends here, 
    *r1 += 1;           // r1 is active again
    println!("{}", x);  // prints 7
}
```

When you create `r2`, it borrows from `r1`, so you cannot use `r1` until `r2` ends.  This is called reborrowing.

Reborrowing is also implicit in many cases:

```rust
fn modify(x: &mut i32) {
    *x += 1;
}

fn main() {
    let mut n = 0;
    let r = &mut n;
    modify(r);  // implicitly reborrows r
    *r += 1;    // r is still usable after the function
    println!("{}", n);
}
```

The function `modify` receives a reborrow of `r`, not a move, so `r` is still valid afterward.

### Partial moves <a href="#partial-moves-" class="header-link">🔗</a>

A partial move happens when you move some fields out of a struct while leaving other fields in place.

```rust
struct Point {
    x: i32,
    y: String,
}

fn main() {
    let p = Point { // the struct itself is on the stack, x on stack, y on stack pointing to heap data
        x: 10,// therefore P will have move characteristics
        y: String::from("hello"),
    };

    let x_val = p.x;  // Copy: x is i32, which implements Copy
    let y_val = p.y;  // Move: y is String, which does not implement Copy

    // println!("{}", p.y); // ERROR: y was moved
    println!("{}", p.x);    // OK: x was copied, not moved
}
```

After the partial move, you cannot use the whole struct `p` anymore, but you can still access the fields that were not moved (like `p.x` in this case).

### Interior mutability <a href="#interior-mutability-" class="header-link">🔗</a>

Interior mutability is a design pattern that lets you mutate data even when there are shared references to it.  This is done using types like `Cell`, `RefCell`, `Mutex`, or `RwLock` that provide controlled mutation.

```rust
use std::cell::RefCell;

fn main() {
    let data = RefCell::new(5);

    let r1 = data.borrow();     // shared borrow
    let r2 = data.borrow();     // another shared borrow
    println!("{} {}", r1, r2);
    drop(r1);
    drop(r2);

    let mut r3 = data.borrow_mut(); // mutable borrow
    *r3 += 1;
    println!("{}", r3);
}
```

`RefCell` enforces borrowing rules at runtime instead of compile time.  If you violate the rules (like trying to borrow mutably while a shared borrow exists), the program will panic.

***

## 8. Lifetimes <a href="#8-lifetimes-" class="header-link">🔗</a>

### What are lifetimes <a href="#what-are-lifetimes-" class="header-link">🔗</a>

A lifetime is Rust's way of tracking how long references are valid.  Every reference has a lifetime, which is the scope for which that reference is valid.

Most of the time, lifetimes are inferred automatically, just like types.  But in some cases, you need to annotate them explicitly to help the compiler understand the relationships between references.

### Lifetime annotations syntax <a href="#lifetime-annotations-syntax-" class="header-link">🔗</a>

Lifetime annotations use an apostrophe followed by a name, like `'a` or `'b`.  The names are usually short, like `'a`, `'b`, or `'c`.

```rust
fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {
    if x.len() > y.len() {
        x
    } else {
        y
    }
}
```

This function says: "The references `x`, `y`, and the return value all have the same lifetime `'a`."  This means the returned reference will be valid as long as both `x` and `y` are valid.

The lifetime annotation does not change how long any reference lives.  It only describes the relationships between the lifetimes of multiple references.

### Lifetime elision rules <a href="#lifetime-elision-rules-" class="header-link">🔗</a>

To reduce annotation burden, Rust has three lifetime elision rules that let the compiler infer lifetimes in common patterns:

**Rule 1**: Each input reference gets its own distinct lifetime parameter.

```rust
fn foo(x: &str, y: &str) // becomes
fn foo<'a, 'b>(x: &'a str, y: &'b str)
```

**Rule 2**: If there is exactly one input lifetime, that lifetime is assigned to all output lifetimes.

```rust
fn first_word(s: &str) -> &str // becomes
fn first_word<'a>(s: &'a str) -> &'a str
```

**Rule 3**: If there are multiple input lifetimes but one is `&self` or `&mut self`, the lifetime of `self` is assigned to all output lifetimes.

```rust
impl MyStruct {
    fn get_data(&self) -> &str // becomes
    fn get_data<'a>(&'a self) -> &'a str
}
```

If the compiler cannot infer all lifetimes using these rules, you must annotate them explicitly.

### Lifetimes in structs <a href="#lifetimes-in-structs-" class="header-link">🔗</a>

When a struct holds references, you must annotate the lifetimes:

```rust
struct Book<'a> {
    title: &'a str,
    author: &'a str,
}

fn main() {
    let title = String::from("Rust Book");
    let author = String::from("Steve");

    let book = Book {
        title: &title,
        author: &author,
    };

    println!("{} by {}", book.title, book.author);
} // book, title, and author all dropped here
```

The `'a` annotation says that the struct `Book` cannot outlive the references it holds.  This prevents dangling references.

### Lifetimes in methods <a href="#lifetimes-in-methods-" class="header-link">🔗</a>

When implementing methods on a struct with lifetimes, you need to declare the lifetime in the `impl` block:

```rust
struct Book<'a> {
    title: &'a str,
    author: &'a str,
}

impl<'a> Book<'a> {
    fn get_title(&self) -> &str {
        self.title
    }
}

fn main() {
    let title = String::from("Rust Book");
    let book = Book {
        title: &title,
        author: "Steve Klabnik",
    };
    println!("Title: {}", book.get_title());
}

```

Here, lifetime elision rule 3 applies: since `get_title` takes `&self`, the returned reference has the same lifetime as `self`.

### The 'static lifetime <a href="#the-static-lifetime-" class="header-link">🔗</a>

The `'static` lifetime is special: it means the reference is valid for the entire program duration.  All string literals have the `'static` lifetime:

```rust
fn main() {
    // String literals have 'static lifetime because they're stored in the program binary
    let s: &'static str = "I have a static lifetime";
    println!("Static string: {}", s);
    
    // You can also use string literals without explicit type annotation
    let literal = "This also has 'static lifetime";
    println!("{}", literal);
    
    // Static lifetime means the reference is valid for the entire program duration
    let result = returns_static_str();
    println!("Returned: {}", result);
}

fn returns_static_str() -> &'static str {
    "This string literal is always valid"
}

// Static string: I have a static lifetime
// This also has 'static lifetime
// Returned: This string literal is always valid

```

The text of string literals is stored directly in the program's binary, so it is always available.

Be careful with `'static` bounds.  Often, the error message suggests adding `'static`, but this is usually not the right solution.  Most of the time, the problem is a dangling reference or a mismatch in lifetimes, not a need for `'static`.

Only use `'static` when the data truly needs to live for the entire program.

***

## 9. Static Items <a href="#9-static-items-" class="header-link">🔗</a>

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

***

## 10. Rust 2024 Edition: Key Behaviors <a href="#10-rust-2024-edition-key-behaviors-" class="header-link">🔗</a>

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

### Return-Position `impl Trait` Captures All In-Scope Lifetimes <a href="#return-position-impl-trait-captures-all-in-scope-lifetimes-" class="header-link">🔗</a>

When a function returns `impl Trait`, Rust automatically captures any lifetime parameters in scope. This makes iterator chains, closure captures, and `async` functions more intuitive:

```rust
fn get_iterator<'a>(data: &'a [u8]) -> impl Iterator<Item = &'a u8> {
    data.iter()
}
```

The lifetime `'a` is automatically available to the returned iterator—you don't need to manually add it to bounds.

### Temporaries in Tail Expressions Drop Earlier <a href="#temporaries-in-tail-expressions-drop-earlier-" class="header-link">🔗</a>

In Rust 2024, temporary values created in the final expression of a block drop immediately after they're used, not after the entire block. This prevents unexpected borrow checker errors:

```rust
use std::cell::RefCell;

fn get_length() -> usize {
    let data = RefCell::new("hello");
    data.borrow().len()  // ✅ Works: borrow is dropped immediately after .len()
}
```

**Be aware of scope changes:**

```rust
fn main() {
    let s = String::from("hello");
    let len = s.len();  // ✅ OK: `s` stays in scope
}

fn main() {
    // ❌ This pattern fails: the temporary is dropped after the expression
    let x = { String::from("hello") }.len();
}

// ✅ Fix: Keep temporaries as named bindings
fn main() {
    let s = String::from("hello");
    let x = s.len();
}
```

Extract temporaries into named variables or explicitly manage their lifetimes.

## 11. Safe Global State Patterns <a href="#11-safe-global-state-patterns-" class="header-link">🔗</a>

### Atomic types <a href="#atomic-types-" class="header-link">🔗</a>

For simple counters and flags, use atomic types from `std::sync::atomic`:

```rust
use std::sync::atomic::{AtomicUsize, Ordering};

static COUNTER: AtomicUsize = AtomicUsize::new(0);

fn main() {
    COUNTER.fetch_add(1, Ordering::SeqCst);
    println!("Counter: {}", COUNTER.load(Ordering::SeqCst));
}
```

Atomics provide thread-safe operations without locks through CPU-level guarantees. However, **memory ordering is critical**. The `Ordering` parameter determines synchronization strength:

- **`Ordering::Relaxed`**: No synchronization; only use for statistics where exact accuracy doesn't matter (e.g., performance counters). Unsafe for coordination between threads as operations can be reordered on weak-memory architectures like ARM.
- **`Ordering::Acquire`/`Ordering::Release`**: One-way synchronization. Load with Acquire or store with Release to coordinate work between threads. Sufficient for most use cases and generally preferred over SeqCst due to better performance on weak-memory architectures.
- **`Ordering::SeqCst`** (Sequentially Consistent): Total ordering across all threads; the safest choice when unsure. While it carries a performance cost, on x86/x86_64 the cost is minimal since all atomic operations use `lock` prefixes regardless; on ARM and weak-memory systems, the difference is more significant.

**Practical guidance**: For simple counters where exact accuracy isn't critical, `Relaxed` is often sufficient and offers the best performance across all architectures. For coordination primitives, prefer `Acquire`/`Release` pairs over `SeqCst` unless you specifically need total ordering.

### Mutex and RwLock <a href="#mutex-and-rwlock-" class="header-link">🔗</a>

For more complex shared state, use `Mutex` or `RwLock`:

```rust
use std::sync::Mutex;

static NAMES: Mutex<Vec<String>> = Mutex::new(Vec::new());

fn main() {
    {
        let mut names = NAMES.lock().unwrap();
        names.push(String::from("Alice"));
        names.push(String::from("Bob"));
        // lock is released here
    }

    let names = NAMES.lock().unwrap();
    println!("Names: {:?}", names);
}
```

`Mutex` ensures only one thread can access the data at a time. `RwLock` allows multiple readers or one writer, mirroring Rust's borrowing rules. Both use **poisoning** to detect panics: if a thread panics while holding a lock, subsequent lock attempts return `Err`, preventing undefined behavior from corrupted state.

**Important**: The `lock()` method blocks the calling thread until the lock is acquired. If you need non-blocking behavior, use `try_lock()` which returns immediately with `Ok` or `Err`.

For types that don't have const-stable constructors like `HashMap`, use `LazyLock` for lazy initialization:

```rust
use std::sync::{LazyLock, RwLock};
use std::collections::HashMap;

static CACHE: LazyLock<RwLock<HashMap<String, String>>> = LazyLock::new(|| {
    RwLock::new(HashMap::new())
});

fn main() {
    {
        let mut cache = CACHE.write().unwrap();
        cache.insert(String::from("key"), String::from("value"));
    }

    let cache = CACHE.read().unwrap();
    println!("{:?}", cache.get("key"));
}
```

**Why LazyLock is necessary**: `HashMap::new()` is not a `const fn`, so it cannot be used in a `static` initializer. `LazyLock` defers initialization until first access. Conversely, `Mutex::new()` and `RwLock::new()` are `const fn`, so they can be used directly in `static` with types that have const constructors like `Vec::new()`.

### OnceLock and LazyLock <a href="#oncelock-and-lazylock-" class="header-link">🔗</a>

`OnceLock` and `LazyLock` provide one-time initialization with different ergonomics:

**OnceLock** (explicit initialization):

```rust
use std::sync::OnceLock;

static CONFIG: OnceLock<String> = OnceLock::new();

fn main() {
    CONFIG.set(String::from("production")).unwrap();
    println!("Config: {}", CONFIG.get().unwrap());

    // CONFIG.set(String::from("dev")).unwrap(); // ERROR: already initialized
}
```

**Key behavior**: `set()` succeeds only once; subsequent attempts return `Err`. The more ergonomic `get_or_init()` handles initialization in a single call without error handling ceremony.

```rust
use std::sync::OnceLock;

static DB_CONNECTION: OnceLock<String> = OnceLock::new();

fn get_db() -> &'static str {
    DB_CONNECTION.get_or_init(|| {
        println!("Connecting to database...");
        String::from("postgres://localhost")
    })
}
```

**Thread-safety guarantee**: Only one thread's closure executes, preventing duplicate initialization costs even under concurrent access; other threads block until initialization completes.

**LazyLock** (automatic initialization):

```rust
use std::sync::LazyLock;

static EXPENSIVE: LazyLock<Vec<i32>> = LazyLock::new(|| {
    println!("Initializing expensive computation...");
    vec![1, 2, 3, 4, 5]
});

fn main() {
    println!("Before access");
    println!("{:?}", *EXPENSIVE); // initialization happens here
    println!("{:?}", *EXPENSIVE); // uses cached value
}
```

**Design philosophy**: `LazyLock` is **preferred over OnceLock for most use cases** because `LazyLock::new(|| computation())` is simpler than `OnceLock::get_or_init(|| ...)`. OnceLock shines when initialization parameters must come from external sources after the static is defined (e.g., configuration loaded at runtime). 

**Difference**: `LazyLock` implements `Deref`, allowing direct access via `*EXPENSIVE`, while `OnceLock` requires `get()` or `get_or_init()`.

### LazyCell for thread-local lazy initialization <a href="#lazycell-for-thread-local-lazy-initialization-" class="header-link">🔗</a>

Since **Rust 1.80** (July 2024), `LazyCell` complements `LazyLock` for non-thread-safe lazy initialization:

```rust
use std::cell::LazyCell;
use std::cell::RefCell;

thread_local! {
    static BUFFER: LazyCell<Vec<u8>> = LazyCell::new(|| {
        println!("Allocating per-thread buffer");
        Vec::with_capacity(4096)
    });
}

fn main() {
    BUFFER.with(|buf| {
        println!("Buffer capacity: {}", buf.capacity());
    });
}
```

**Key distinction**: `LazyCell` is single-threaded (like `RefCell`), while `LazyLock` is thread-safe. Use `LazyCell` inside `thread_local!` for per-thread lazy initialization.

### thread_local! macro <a href="#threadlocal-macro-" class="header-link">🔗</a>

For per-thread state, use `thread_local!`:

```rust
use std::cell::RefCell;

thread_local! {
    static COUNTER: RefCell<u32> = RefCell::new(0);
}

fn main() {
    COUNTER.with(|c| {
        *c.borrow_mut() += 1;
        println!("Thread counter: {}", c.borrow());
    });
}
```

Each thread gets its own copy of the data, preventing cross-thread interference. **Important**: `RefCell::borrow_mut()` panics on double borrow at runtime. For simple types, use `Cell` to avoid this overhead:

```rust
use std::cell::Cell;

thread_local! {
    static REQUESTS: Cell<usize> = Cell::new(0);
}

fn main() {
    REQUESTS.with(|r| {
        r.set(r.get() + 1);
        println!("Requests: {}", r.get());
    });
}
```

`Cell` is safer for move-able types since it doesn't track borrow state; `RefCell` is necessary for containing references.

### Arc<Mutex<T>> for shared ownership across threads <a href="#arcmutext-for-shared-ownership-across-threads-" class="header-link">🔗</a>

When multiple threads need to own and mutate shared data, use `Arc<Mutex<T>>`:

```rust
use std::sync::{Arc, Mutex};
use std::thread;

fn main() {
    let counter = Arc::new(Mutex::new(0));
    let mut handles = vec![];

    for _ in 0..3 {
        let counter = Arc::clone(&counter);
        let handle = thread::spawn(move || {
            let mut num = counter.lock().unwrap();
            *num += 1;
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    println!("Final count: {}", *counter.lock().unwrap());
}
```

`Arc` (Atomic Reference Counting) provides shared ownership across threads with atomic reference counting. **Contrast with statics**: statics have `'static` lifetime and fixed addresses; `Arc<Mutex<T>>` is dynamic and can be created/destroyed at runtime.

### Best Practices and Pitfalls <a href="#best-practices-and-pitfalls-" class="header-link">🔗</a>

**✓ DO:**

- Use atomics for simple counters and flags with minimal contention
- Prefer `LazyLock` over external `lazy_static` crate (standardized since 1.80)
- Use `Acquire`/`Release` ordering for coordination; reserve `SeqCst` for rare cases
- Minimize lock scope: acquire only when accessing protected data
- Use `Cell` over `RefCell` in `thread_local!` for `Copy` types to avoid runtime borrow checks

**✗ DON'T:**

- Use globals to hide poor architecture; prefer dependency injection where possible
- Assume `Relaxed` ordering is safe for inter-thread coordination
- Hold locks across await points in async code (use `tokio::sync` instead)
- Ignore lock poisoning; check for `Err` on `lock()` calls
- Use `RefCell` when `Cell` suffices; `Cell` has no runtime overhead


### Performance Considerations <a href="#performance-considerations-" class="header-link">🔗</a>

- **x86_64**: `SeqCst` atomics compile to `lock` prefix, same as `Acquire`/`Release` in most cases
- **ARM/weak-memory**: `Acquire`/`Release` is significantly cheaper than `SeqCst`
- **Mutex vs RwLock**: RwLock is slower on single-threaded paths; use only with heavy read contention
- **LazyLock initialization cost**: Paid once on first access; negligible for most applications

# 12. Best Practices and Decision Guide <a href="#12-best-practices-and-decision-guide-" class="header-link">🔗</a>

**Note:** Written for Rust 1.90.0. All code examples use stable features available in this version and later.

### Choosing between const and static <a href="#choosing-between-const-and-static-" class="header-link">🔗</a>

Use `const` when:

- The value is known at compile time and never changes
- You don't need a fixed memory address or shared global state
- The value is small and you want it inlined (substituted directly at compile time)
- Examples: mathematical constants, configuration values, lookup tables, type-level constants

Const values are **inlined at compile** time, meaning the compiler substitutes them directly into the code where used. This doesn't mean runtime copying—it's compile-time substitution. For non-`Copy` types, multiple copies can exist at different memory addresses, but this is not a performance concern.

**Example: Using const effectively**

```rust
// Good use of const - compile-time substitution
const PI: f64 = 3.14159265359;
const MAX_CONNECTIONS: usize = 100;
const DEFAULT_TIMEOUT_SECS: u64 = 30;

// Const arrays and tuples
const FIBONACCI_SEQUENCE: [u32; 5] = [1, 1, 2, 3, 5];
const STATUS_CODES: (&str, u16) = ("OK", 200);

// Const functions evaluated at compile time
const fn fibonacci(n: usize) -> u32 {
    match n {
        0 | 1 => 1,
        2 => 2,
        3 => 3,
        4 => 5,
        _ => 0,
    }
}

fn calculate_circumference(radius: f64) -> f64 {
    2.0 * PI * radius  // PI is substituted here at compile time
}

fn main() {
    println!("Circumference: {}", calculate_circumference(5.0));
    println!("Max connections: {}", MAX_CONNECTIONS);
    println!("Fibonacci(4): {}", fibonacci(4));  // Evaluated at compile time
}
```

Use `static` when:

- You need a fixed memory address (for FFI, pointer comparisons, or global registration)
- The data is large and you want to avoid duplicating it across your binary
- You need global state initialized at runtime
- You need interior mutability for shared mutable state
- Examples: global caches, logger instances, runtime-loaded configuration

The static has a guaranteed fixed memory address for the entire program lifetime and never moves. All references to the same static refer to the same memory location. In Rust 1.90.0, `OnceLock` (stable since 1.70) and `Mutex` are the **strongly preferred** approaches for managing global state safely instead of `static mut`.

**Example: Using static with OnceLock (recommended)**

```rust
use std::sync::OnceLock;
use std::sync::Mutex;

// Lazy initialization with OnceLock - computed once at runtime
static CONFIG: OnceLock<AppConfig> = OnceLock::new();

#[derive(Clone, Debug)]
struct AppConfig {
    database_url: String,
    max_retries: u32,
}

fn get_config() -> &'static AppConfig {
    CONFIG.get_or_init(|| AppConfig {
        database_url: "postgres://localhost/mydb".to_string(),
        max_retries: 3,
    })
}

// Global mutable state - safely protected by Mutex
static COUNTER: Mutex<u64> = Mutex::new(0);

fn increment_counter() {
    let mut count = COUNTER.lock().unwrap();
    *count += 1;
}

fn main() {
    let config = get_config();
    println!("Database: {}", config.database_url);
    
    increment_counter();
    let count = COUNTER.lock().unwrap();
    println!("Counter: {}", *count);
}
```

**Example: Static for large data and FFI**

```rust
// Large data stored once at a fixed address - no duplication
static LOOKUP_TABLE: &[u32] = &[
    0, 1, 1, 2, 3, 5, 8, 13, 21, 34, 55, 89, 144,
];

// FFI with C - requires fixed address
// Note: This requires linking to a C library compiled with compatible ABI.
// This example shows the syntax only; actual usage depends on your C library.
static mut C_BUFFER: [u8; 1024] = [0; 1024];

extern "C" {
    fn c_function_needs_buffer(buffer: *mut u8);
}

fn main() {
    // LOOKUP_TABLE has a fixed address throughout program lifetime
    println!("Fibonacci: {:?}", LOOKUP_TABLE);
    
    // Use unsafe only for necessary FFI interop
    // Safety: Assumes c_function_needs_buffer respects the buffer size
    #[allow(unsafe_code)]
    unsafe {
        c_function_needs_buffer(C_BUFFER.as_mut_ptr());
    }
}
```


### When to move vs borrow <a href="#when-to-move-vs-borrow-" class="header-link">🔗</a>

Move ownership when:

- The caller no longer needs the value after the function call
- You're transferring a resource with cleanup logic (file handles, database connections)
- The function consumes the value to produce something new (parsing, transforming)
- Performance matters and avoiding a reference layer is necessary for large types

Borrow when:

- The caller still needs the value after the function returns
- You only need to read the value
- You want temporary mutable access without taking ownership
- You're designing a library API that should work with many types

The borrow checker enforces these patterns at compile time, making your code thread-safe and eliminating entire categories of data races and use-after-free bugs.

**Example: Move vs borrow decision tree**

```rust
use std::fs::File;
use std::io::{self, Read, Write};

// Example 1: MOVE - resource cleanup with ownership transfer
fn open_and_read(path: &str) -> io::Result<String> {
    let mut file = File::open(path)?;  // owns file
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(contents)  // ownership transfers to caller
    // file is dropped automatically here
}

// Example 2: BORROW - read without taking ownership
fn count_lines(text: &str) -> usize {
    text.lines().count()  // text is borrowed, caller retains ownership
}

// Example 3: MOVE - value transformation
fn parse_numbers(input: String) -> Vec<i32> {
    input  // ownership moved into function
        .split(',')
        .filter_map(|s| s.trim().parse().ok())
        .collect()  // Vec ownership transfers to caller
    // original String is consumed
}

// Example 4: BORROW - multiple readers with slice
fn analyze(data: &[i32]) -> (f64, i32, i32) {
    let sum: i32 = data.iter().sum();
    let avg = sum as f64 / data.len() as f64;
    let max = *data.iter().max().unwrap_or(&0);
    let min = *data.iter().min().unwrap_or(&0);
    (avg, max, min)
}

fn main() -> io::Result<()> {
    // Create test file for demonstration
    let path = "test_example.txt";
    let mut file = File::create(path)?;
    file.write_all(b"line one\nline two\nline three\n")?;
    drop(file);
    
    // Takes ownership and returns it
    let content = open_and_read(path)?;
    
    // Borrows content - doesn't consume it
    let lines = count_lines(&content);
    println!("Lines: {}", lines);
    
    // Takes ownership of a clone (forces a copy)
    let numbers = parse_numbers(content.clone());
    
    // Borrows the slice - content still owned by main
    let (avg, max, min) = analyze(&numbers);
    println!("Average: {}, Max: {}, Min: {}", avg, max, min);
    
    // Both content and numbers still available here
    println!("Content length: {}", content.len());
    println!("Numbers: {:?}", numbers);
    
    // Cleanup
    std::fs::remove_file(path)?;
    
    Ok(())
}
```


### Ownership patterns in practice <a href="#ownership-patterns-in-practice-" class="header-link">🔗</a>

**Pattern 1: Builders with ownership transfer**

Builders move `self` through chains to enable compile-time type safety and prevent misuse:

```rust
#[derive(Debug)]
struct HttpRequest {
    method: String,
    url: String,
    headers: Vec<(String, String)>,
    body: Option<String>,
}

struct HttpRequestBuilder {
    method: String,
    url: String,
    headers: Vec<(String, String)>,
    body: Option<String>,
}

impl HttpRequestBuilder {
    fn new(method: &str, url: &str) -> Self {
        HttpRequestBuilder {
            method: method.to_string(),
            url: url.to_string(),
            headers: Vec::new(),
            body: None,
        }
    }

    // Takes and returns ownership - chainable and type-safe
    fn header(mut self, key: &str, value: &str) -> Self {
        self.headers.push((key.to_string(), value.to_string()));
        self
    }

    fn body(mut self, content: &str) -> Self {
        self.body = Some(content.to_string());
        self
    }

    // Terminal operation - consumes builder, produces final type
    fn build(self) -> HttpRequest {
        HttpRequest {
            method: self.method,
            url: self.url,
            headers: self.headers,
            body: self.body,
        }
    }
}

fn main() {
    // Ownership flows through the chain
    let request = HttpRequestBuilder::new("POST", "https://api.example.com/users")
        .header("Content-Type", "application/json")
        .header("Authorization", "Bearer token123")
        .body(r#"{"name": "Alice"}"#)
        .build();

    println!("{:#?}", request);
    // Builder is consumed - cannot reuse after build()
}
```

**Pattern 2: Iterator adapters with borrowed closures**

Iterator methods borrow inputs and closures, enabling seamless composition:

```rust
fn process_data(numbers: &[i32]) -> Vec<i32> {
    numbers
        .iter()  // borrows numbers
        .filter(|&&n| n > 0)
        .map(|n| n * 2)
        .collect()  // collects into owned Vec
    // numbers still owned by caller
}

fn summarize_data(data: &[i32]) -> String {
    data.iter()
        .filter(|&&x| x % 2 == 0)  // only evens
        .map(|x| x.to_string())
        .collect::<Vec<_>>()
        .join(", ")
}

fn main() {
    let numbers = vec![1, -2, 3, -4, 5, 6];
    
    let positive_doubled = process_data(&numbers);
    println!("Doubled: {:?}", positive_doubled);
    
    // numbers still available here
    let summary = summarize_data(&numbers);
    println!("Even numbers: {}", summary);
    
    println!("Original: {:?}", numbers);
}
```

**Pattern 3: Resource management (RAII)**

Move semantics ensure predictable cleanup via the `Drop` trait:

```rust
struct Connection {
    id: u32,
    is_open: bool,
}

impl Connection {
    fn new(id: u32) -> Self {
        println!("Connection {} opened", id);
        Connection {
            id,
            is_open: true,
        }
    }

    fn send(&mut self, msg: &str) -> std::io::Result<()> {
        if !self.is_open {
            panic!("Connection {} is closed", self.id);
        }
        println!("Connection {}: {}", self.id, msg);
        Ok(())
    }
}

impl Drop for Connection {
    fn drop(&mut self) {
        if self.is_open {
            println!("Connection {} closed (dropped)", self.id);
            self.is_open = false;
        }
    }
}

fn process_with_resource(id: u32) {
    let mut conn = Connection::new(id);  // acquire
    let _ = conn.send("Hello");
    let _ = conn.send("World");
    // conn is dropped here - automatically closed
}

fn main() {
    process_with_resource(1);
    println!("---");
    process_with_resource(2);
    
    // No manual cleanup needed - Drop trait handles it
    // This pattern is called RAII: Resource Acquisition Is Initialization
}
```


### Common pitfalls <a href="#common-pitfalls-" class="header-link">🔗</a>

**Pitfall 1: Fighting the borrow checker instead of understanding it**

If you get a borrow checker error, resist adding `.clone()` immediately. Usually, the error represents a **real safety issue**. The borrow checker prevents data races and use-after-free bugs that would cause undefined behavior in other languages. Understanding *why* the error exists is more valuable than working around it.

```rust
// WRONG: Trying to borrow mutably twice
fn buggy_swap(a: &mut i32, b: &mut i32) {
    let temp = *a;
    *a = *b;
    *b = temp;
}

// This WON'T compile if you call: buggy_swap(&mut x, &mut x);
// Because two mutable references to the same data are impossible

// RIGHT: Accept ownership and swap
fn fixed_swap(mut a: i32, mut b: i32) -> (i32, i32) {
    std::mem::swap(&mut a, &mut b);
    (a, b)
}

// Or use utility function:
fn swap_refs(a: &mut i32, b: &mut i32) {
    std::mem::swap(a, b);
}

fn main() {
    let mut x = 5;
    let mut y = 10;
    
    swap_refs(&mut x, &mut y);
    println!("x: {}, y: {}", x, y);  // x: 10, y: 5
}
```

**Pitfall 2: Excessive cloning**

Cloning is sometimes necessary, but widespread cloning signals a design problem. If you're cloning strings, vectors, or custom types everywhere, you're fighting ownership rather than working with it. Refactor to use borrowing strategically or redesign your data flow.

```rust
// WRONG: Cloning in tight loops
fn process_inefficient(data: Vec<String>) -> Vec<String> {
    let mut result = Vec::new();
    
    for item in &data {
        result.push(item.clone());  // unnecessary clone
        println!("{}", item);
    }
    
    let transformed = data.clone();  // unnecessary clone
    let mut processed = Vec::new();
    
    for item in &transformed {
        processed.push(format!("processed: {}", item));
    }
    
    processed
}

// RIGHT: Use borrowing
fn process_efficient(data: &[String]) -> Vec<String> {
    let mut result = Vec::new();
    
    for item in data {
        // Only clone when transferring ownership is necessary
        result.push(format!("processed: {}", item));
        println!("{}", item);
    }
    
    result
}

// RIGHT: Use Cow for conditional cloning
use std::borrow::Cow;

fn process_cow(data: &[String]) -> Vec<Cow<'_, str>> {
    data.iter()
        .map(|s| {
            if s.contains("error") {
                Cow::Owned(format!("ERROR: {}", s))
            } else {
                Cow::Borrowed(s)
            }
        })
        .collect()
}

fn main() {
    let data = vec![
        "hello".to_string(),
        "error in processing".to_string(),
        "world".to_string(),
    ];
    
    let processed = process_efficient(&data);
    println!("Processed: {:?}", processed);
    
    // data is still available and not cloned unnecessarily
}
```

**Pitfall 3: Using `'static` bounds unnecessarily**

Adding a `'static` lifetime bound restricts your generic function to only static or owned data, preventing it from accepting borrowed references. This is often too restrictive and should only be used when actually needed for storage or certain trait requirements.

```rust
// WRONG: Too restrictive - won't work with borrowed data
fn print_owned<T: 'static + std::fmt::Debug>(value: T) {
    println!("{:?}", value);
}

// This won't compile:
// let text = "hello";
// print_owned(&text);  // Error: &str doesn't live 'static

// RIGHT: Remove 'static unless actually needed
fn print_flexible<T: std::fmt::Debug>(value: T) {
    println!("{:?}", value);
}

// This compiles and works with borrowed data:
// let text = "hello";
// print_flexible(&text);  // Works!

// RIGHT: Use 'static only when storing values indefinitely
fn store_in_global<T: 'static>(value: T) {
    use std::sync::OnceLock;
    static STORED: OnceLock<Box<dyn std::any::Any>> = OnceLock::new();
    let _ = STORED.set(Box::new(value));
}

fn main() {
    let borrowed_string = "temporary".to_string();
    
    // Works - no lifetime restriction
    print_flexible(&borrowed_string);
    
    // Works - passing owned value
    print_flexible(borrowed_string.clone());
    
    // Works - storing owned value
    store_in_global(vec![1, 2, 3]);
}
```

**Pitfall 4: Using `static mut` when safer alternatives exist**

In Rust 1.90.0, `OnceLock`, `Mutex`, `RwLock`, `AtomicU64`, and other atomic types cover nearly all global state needs safely. Use `static mut` only for FFI or after profiling confirms safer alternatives are bottlenecks.

```rust
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, OnceLock};

// WRONG: Unsafe mutable global state - data races possible!
static mut BAD_COUNTER: u64 = 0;

fn unsafe_increment() {
    unsafe {
        BAD_COUNTER += 1;  // Data race with concurrent access!
    }
}

// RIGHT: Use atomics for simple counters
static GOOD_COUNTER: AtomicU64 = AtomicU64::new(0);

fn safe_increment() {
    GOOD_COUNTER.fetch_add(1, Ordering::SeqCst);
}

// RIGHT: Use Mutex for complex shared state
static COMPLEX_STATE: OnceLock<Mutex<Vec<String>>> = OnceLock::new();

fn get_state() -> &'static Mutex<Vec<String>> {
    COMPLEX_STATE.get_or_init(|| Mutex::new(Vec::new()))
}

fn add_to_state(item: String) {
    let state = get_state();
    let mut guard = state.lock().unwrap();
    guard.push(item);
}

// RIGHT: Use OnceLock for immutable lazy initialization
static CONFIG: OnceLock<AppSettings> = OnceLock::new();

#[derive(Clone)]
struct AppSettings {
    name: String,
    version: String,
}

fn get_config() -> &'static AppSettings {
    CONFIG.get_or_init(|| AppSettings {
        name: "MyApp".to_string(),
        version: "1.0".to_string(),
    })
}

fn main() {
    // Safe counters
    safe_increment();
    safe_increment();
    println!("Counter: {}", GOOD_COUNTER.load(Ordering::SeqCst));
    
    // Safe state management
    add_to_state("first item".to_string());
    add_to_state("second item".to_string());
    
    let state = get_state();
    let guard = state.lock().unwrap();
    println!("State: {:?}", *guard);
    
    // Safe lazy configuration
    let config = get_config();
    println!("App: {} v{}", config.name, config.version);
}
```


### Performance considerations <a href="#performance-considerations-" class="header-link">🔗</a>

Rust's ownership system has **zero runtime cost**. All checking happens at compile time.

For types implementing the `Copy` trait (integers, booleans, small fixed-size types), moving and copying are semantically identical and compile to identical machine code. For larger types like `Vec` and `String`, moving transfers only the pointer/metadata (typically 24 bytes), not the heap data—this is extremely fast.

Borrowing also has zero overhead: a reference is simply a pointer at runtime. The borrow checker's work is entirely compile-time.

Performance impact comes only from:

- **Excessive cloning**: Cloning large types allocates and copies heap memory. Avoid in tight loops.
- **Lock contention**: If multiple threads frequently lock the same `Mutex`, contention becomes the bottleneck—not the Mutex itself.
- **Unnecessary reference layers**: In rare cases where reference indirection impacts cache locality.

In practice, Rust's ownership rules encourage code that is both safe and efficient by default.

**Example: Performance patterns**

```rust
use std::sync::Mutex;
use std::time::Instant;

// SLOW: Cloning in a tight loop allocates repeatedly
fn slow_approach(data: Vec<String>) -> usize {
    let mut total = 0;
    for _ in 0..1000 {
        let cloned = data.clone();  // Allocates 1000 times!
        total += cloned.len();
    }
    total
}

// FAST: Borrowing avoids allocation
fn fast_approach(data: &[String]) -> usize {
    let mut total = 0;
    for _ in 0..1000 {
        total += data.len();  // No allocation, just reads
    }
    total
}

// SLOW: Locking on every iteration causes contention
fn slow_counter() {
    static COUNTER: Mutex<u64> = Mutex::new(0);
    
    for _ in 0..100_000 {
        let mut guard = COUNTER.lock().unwrap();
        *guard += 1;
        drop(guard);  // Lock contention on every iteration!
    }
}

// FAST: Local accumulation, lock once at the end
fn fast_counter() {
    static COUNTER: Mutex<u64> = Mutex::new(0);
    
    let mut local = 0u64;
    for _ in 0..100_000 {
        local += 1;
    }
    
    let mut guard = COUNTER.lock().unwrap();
    *guard += local;  // One lock operation
}

// Move semantics are free - only pointer transferred
fn ownership_transfer(data: Vec<i32>) -> Vec<i32> {
    // Moving data transfers only the pointer, not heap contents
    data.into_iter().map(|x| x * 2).collect()
}

fn main() {
    let data = (0..100)
        .map(|i| format!("item{}", i))
        .collect::<Vec<_>>();
    
    let start = Instant::now();
    let _result_slow = slow_approach(data.clone());
    println!("Slow (cloning): {:?}", start.elapsed());
    
    let start = Instant::now();
    let _result_fast = fast_approach(&data);
    println!("Fast (borrowing): {:?}", start.elapsed());
    
    // Move semantics example
    let numbers = vec![1, 2, 3, 4, 5];
    let doubled = ownership_transfer(numbers);  // cheap pointer move
    println!("Doubled: {:?}", doubled);
}
```