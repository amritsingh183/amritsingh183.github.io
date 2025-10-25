---
layout: post
title: "Mastering Variables, Constants and Lifetimes in Rust: A Complete Guide"
date: 2025-10-8 11:23:00 +0530
categories: rust concepts
---

# A Complete Guide to Rust Ownership, Lifetimes, and Memory Management

## Index

1. **Foundation: Mental Model**
    - Aliasing XOR mutability principle
    - Prerequisites and goals
2. **Variables in Rust**
    - Immutability by default
    - Mutable variables
    - Variable shadowing
    - Scope and dropping
3. **Constants**
    - Declaring constants
    - Naming conventions
    - When to use constants
    - Constants vs variables
4. **Ownership Fundamentals**
    - The three ownership rules
    - Memory and allocation
    - Move semantics
    - Clone and Copy traits
    - Stack-only data and Copy
5. **Borrowing and References**
    - Shared references (&T)
    - Mutable references (&mut T)
    - The borrowing rules
    - Dangling references prevention
6. **Non-Lexical Lifetimes (NLL)**
    - What NLL solves
    - How NLL works
    - Examples with NLL
7. **Advanced Borrowing Patterns**
    - Two-phase borrows
    - Reborrowing
    - Partial moves
    - Interior mutability
8. **Lifetimes**
    - What are lifetimes
    - Lifetime annotations syntax
    - Lifetime elision rules
    - Lifetimes in structs
    - Lifetimes in methods
    - The 'static lifetime
9. **Static Items**
    - What is static
    - Static vs const comparison
    - When to use static
    - Mutable statics and safety
    - The Sync requirement
10. **Rust 2024 Edition Changes**
    - The static_mut_refs lint
    - Migration to safe patterns
11. **Safe Global State Patterns**
    - Atomic types
    - Mutex and RwLock
    - OnceLock and LazyLock
    - thread_local! macro
12. **Best Practices and Decision Guide**
    - Choosing between const and static
    - When to move vs borrow
    - Common pitfalls
    - Performance considerations

***

## 1. Foundation: Mental Model

### Aliasing XOR mutability principle

Rust's safety model is built on one core principle: you can have many readers or one writer, but not both at the same time.  This is called "aliasing XOR mutability" and it prevents data races at compile time.

Think of it like a library book: either many people can read it at once (shared access), or one person can write notes in it (exclusive access), but you cannot have someone writing while others are reading.

This rule is enforced by the borrow checker, which analyzes your code at compile time to make sure no two parts of your program can modify the same data simultaneously.

### Prerequisites and goals

This guide assumes you know basic Rust syntax like variables, functions, and control flow, but you don't need prior systems programming experience.  The goal is to give you a clear mental model so you can design APIs and debug borrow checker errors with confidence.

***

## 2. Variables in Rust

### Immutability by default

In Rust, variables are immutable by default.  This means once you assign a value to a variable, you cannot change it unless you explicitly say it is mutable.

```rust
fn main() {
    let x = 5;
    println!("The value of x is: {}", x);
    // x = 6; // ERROR: cannot assign twice to immutable variable
}
```

If you try to change `x`, the compiler will stop you with an error.  This design encourages writing code with fewer side effects and clearer data flow.

### Mutable variables

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

### Variable shadowing

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

### Scope and dropping

Variables live within a scope, which is usually marked by curly braces `{}`.  When a variable goes out of scope, Rust automatically cleans up its memory by calling the `Drop` trait.

```rust
fn main() {
    {
        let s = String::from("hello"); // s is valid from here
        // you can use s here
    } // s goes out of scope and is dropped here
    // s is no longer valid here
}
```

This automatic cleanup is one of Rust's key features: no manual memory management, and no garbage collector.

***

## 3. Constants

### Declaring constants

Constants are declared with the `const` keyword and must always have a type annotation.  Constants can be declared in any scope, including the global scope.

```rust
const MAX_POINTS: u32 = 100_000;

fn main() {
    const HOURS_IN_DAY: u32 = 24;
    println!("Max points: {}", MAX_POINTS);
    println!("Hours: {}", HOURS_IN_DAY);
}
```


### Naming conventions

Constants use SCREAMING_SNAKE_CASE by convention.  This makes them easy to spot in your code.

### When to use constants

Use constants for values that never change and are known at compile time.  Examples include mathematical constants, configuration limits, or fixed array sizes.

```rust
const PI: f64 = 3.14159265359;
const MAX_BUFFER_SIZE: usize = 1024;
const THREE_HOURS: u32 = 60 * 60 * 3; // OK: computed at compile time
fn main() {
    // const RUNTIME_VAL: u32 = get_value(); // ERROR: cannot call functions in const
}
```
### Constants vs variables


| Feature | `const` | `let` |
| :-- | :-- | :-- |
| Mutability | Always immutable; `mut` cannot be used. | Immutable by default, but can be made mutable with the `mut` keyword. |
| Type Annotation | Mandatory. The type must be explicitly declared. | Optional. The compiler can infer the type if not specified. |
| Value Assignment | Must be a constant expression evaluated at compile time. | Can be a value computed at runtime. |
| Memory Address | Does not have a fixed address; the value is inlined by the compiler where it is used. | Has a specific memory location, which the compiler manages. |
| Scope | Can be declared in any scope, including globally. | Restricted to the block in which it is declared. |



***
## 4. Ownership Fundamentals

### The Three Ownership Rules

Rust's ownership system has three fundamental rules that prevent memory leaks, double frees, and use-after-free bugs at compile time:

1. Each value in Rust has exactly one owner at a time.
2. When the owner goes out of scope, the value is dropped automatically.
3. Ownership can be transferred (moved) from one variable to another.

These rules are enforced by the compiler, ensuring memory safety without requiring a garbage collector.

### Memory and Allocation

Rust stores data in two places: the **stack** and the **heap**. The stack stores values with a known, fixed size, while the heap stores values that can grow or shrink at runtime. Understanding where data lives is crucial to understanding Rust's ownership model.

Types stored entirely on the stack (like integers, booleans, and simple structs) can implement the `Copy` trait, allowing them to be duplicated efficiently. Types that allocate heap memory (like `String` and `Vec<T>`) use move semantics to transfer ownership, preventing multiple owners from accessing the same heap allocation.

```rust
fn main() {
    let s = String::from("hello"); // Allocates heap memory
    // s is valid here
} // s goes out of scope, memory is freed automatically
```

When `s` goes out of scope, Rust calls the `drop` function automatically, freeing the heap memory.

### Stack vs Heap: Where Does a Struct Live?

```
**By default, Rust allocates all structs on the stack**, just like in C++. To store a struct on the heap, you must explicitly use heap-allocating types like `Box<T>`, `Rc<T>`, or `Arc<T>`.
```


#### Stack Allocation (Default)

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


#### Heap Allocation (Explicit)

To allocate a struct on the heap, wrap it in `Box<T>` or similar smart pointers:

```rust
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


#### The Hybrid Case: Structs with Heap-Allocated Fields

```
Some structs are allocated on the stack but contain fields that point to heap memory. This is the case for types like `String`, `Vec<T>`, `HashMap<K, V>`, `Arc<T>`, and `Rc<T>`:
```

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


#### Memory Layout Rules

Here's a comprehensive breakdown of what goes where:

1. **Stack (by default):**
    - All primitive types: integers, floats, booleans, characters
    - Structs and enums (the struct/enum itself)
    - Arrays with fixed size: `[i32; 100]`
    - Tuples
    - Function parameters and local variables
    - The fixed-size portions of heap-allocating types (e.g., the pointer/length/capacity metadata of `String` and `Vec`)
2. **Heap (explicit allocation):**

```
- Values wrapped in `Box<T>`, `Rc<T>`, `Arc<T>`
```

    - The contents of `String` (the actual character bytes)
    - The contents of `Vec<T>` (the actual elements)
    - The contents of `HashMap<K, V>`, `BTreeMap<K, V>`, etc.
    - Any value explicitly allocated with heap allocators
3. **Static memory:**
    - Static variables declared with `static`
    - String literals: `"hello"`
```rust
fn main() {
    // Stack-only struct
    struct StackOnly {
        x: i32,
        y: i32,
    }
    
    // Struct with heap allocations
    struct HasHeapData {
        id: u32,              // On stack
        name: String,         // Metadata on stack, content on heap
        scores: Vec<i32>,     // Metadata on stack, elements on heap
    }
    
    // Explicitly heap-allocated struct
    struct ExplicitHeap {
        data: Box<[u8; 1000]>, // Array lives on heap, not stack
    }
    
    let stack_struct = StackOnly { x: 10, y: 20 };        // Entirely on stack
    let hybrid_struct = HasHeapData {                      // Struct on stack
        id: 1,                                             // Field on stack
        name: String::from("Bob"),                         // Content on heap
        scores: vec![95, 87, 92],                          // Content on heap
    };
    let heap_struct = ExplicitHeap {                       // Struct on stack
        data: Box::new([0; 1000]),                         // Array on heap
    };
}
```


#### Why This Matters for Ownership

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


### Move Semantics

By default, Rust **moves** ownership when you assign a value to another variable or pass it to a function. This is particularly important for heap-allocated types, where moving prevents double-free errors. After a move, the original variable becomes invalid and cannot be used.

#### Move on Assignment

```rust
fn main() {
    let s1 = String::from("hello");
    let s2 = s1; // Ownership moves from s1 to s2

    // println!("{}", s1); // ERROR: s1 is no longer valid
    println!("{}", s2);    // OK: s2 is the owner now
}
```

After the move, `s1` is no longer valid. Only `s2` owns the string now.

#### Move When Passing to Functions

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


#### Move When Returning from Functions

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


#### Taking and Returning Ownership

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

### Stack-Only Data and Copy

Stack-only types implement the `Copy` trait, which enables implicit bitwise duplication. When you assign or pass a `Copy` type, Rust creates an independent copy rather than moving ownership. Both the original and the copy remain valid, and no ownership transfer occurs.

#### Characteristics of Copy Types

Types that implement `Copy` must be stored entirely on the stack and contain no heap allocations. The `Copy` trait is a marker trait that depends on `Clone`, meaning any `Copy` type must also implement `Clone`. You cannot implement `Copy` for types that allocate heap memory or implement the `Drop` trait.

#### Common Copy Types

- All integer types: `i8`, `i16`, `i32`, `i64`, `i128`, `u8`, `u16`, `u32`, `u64`, `u128`, `isize`, `usize`
- Boolean type: `bool`
- Floating-point types: `f32`, `f64`
- Character type: `char`
- Function pointers: `fn()`
- Immutable references: `&T` (but not mutable references `&mut T`)
- Raw pointers: `*const T` and `*mut T`
- Tuples containing only `Copy` types: `(i32, i32)`, `(bool, char, f64)`


#### Copy Semantics in Action

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


#### Key Takeaway

With `Copy` types, assignment and function calls create independent copies. The original variable remains valid because no ownership transfer occurs. This behavior is safe because stack-only data is cheap to duplicate and doesn't require special cleanup.

### Heap-Allocated Data and Move

Heap-allocated types do **not** implement `Copy` because copying them would create multiple owners of the same heap memory, leading to double-free errors. Instead, these types use **move semantics** to transfer ownership. After a move, the original variable becomes invalid, ensuring that only one owner exists at any time.

#### Characteristics of Move Types

```
Types that allocate heap memory (like `String`, `Vec<T>`, `Box<T>`, and custom structs containing heap data) cannot implement `Copy`. When assigned or passed to functions, ownership moves from the source to the destination. The compiler prevents you from using the moved variable, guaranteeing memory safety.
```


#### Common Move Types

- `String`: Heap-allocated, growable text
- `Vec<T>`: Heap-allocated, growable array
- `Box<T>`: Heap-allocated single value
- `HashMap<K, V>`: Heap-allocated hash map
- Custom structs containing heap-allocated fields


#### Move Semantics in Action

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


#### Key Takeaway

With heap-allocated types, assignment and function calls transfer ownership via moves. The original variable becomes invalid, preventing multiple owners from accessing the same heap memory. When ownership is returned from a function, it transfers to the caller, extending the value's lifetime.

### The Clone Trait

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

## 5. Borrowing and References

Instead of transferring ownership, you can let a function borrow a value by passing a reference.

### Shared references (&T)

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


### Mutable references (&mut T)

A mutable reference lets you modify a borrowed value.  You create it with `&mut`:

```rust
fn main() {
    let mut s = String::from("hello");
    change(&mut s);
    println!("{}", s); // prints "hello, world"
}

fn change(some_string: &mut String) {
    some_string.push_str(", world");
}
```


### The borrowing rules

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

### Dangling references prevention

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

## 6. Non-Lexical Lifetimes (NLL)

### What NLL solves

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

### How NLL works

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

### Examples with NLL

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

## 7. Advanced Borrowing Patterns

### Two-phase borrows

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
3. After all arguments are evaluated, the mutable borrow for `push` becomes active.

This sequencing prevents overlap between the shared read and the mutable write.

### Reborrowing

Reborrowing happens when you create a new reference from an existing mutable reference.  The new reference temporarily "pauses" the original reference.

```rust
fn main() {
    let mut x = 5;
    let r1 = &mut x;    // first mutable borrow
    let r2 = &mut *r1;  // reborrow: creates a new mutable reference
    *r2 += 1;           // use r2
    // r2 ends here
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

### Partial moves

A partial move happens when you move some fields out of a struct while leaving other fields in place.

```rust
struct Point {
    x: i32,
    y: String,
}

fn main() {
    let p = Point {
        x: 10,
        y: String::from("hello"),
    };
    
    let x_val = p.x;  // Copy: x is i32, which implements Copy
    let y_val = p.y;  // Move: y is String, which does not implement Copy
    
    // println!("{}", p.y); // ERROR: y was moved
    println!("{}", p.x);    // OK: x was copied, not moved
}
```

After the partial move, you cannot use the whole struct `p` anymore, but you can still access the fields that were not moved (like `p.x` in this case).

### Interior mutability

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

## 8. Lifetimes

### What are lifetimes

A lifetime is Rust's way of tracking how long references are valid.  Every reference has a lifetime, which is the scope for which that reference is valid.

Most of the time, lifetimes are inferred automatically, just like types.  But in some cases, you need to annotate them explicitly to help the compiler understand the relationships between references.

### Lifetime annotations syntax

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

### Lifetime elision rules

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

### Lifetimes in structs

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

### Lifetimes in methods

When implementing methods on a struct with lifetimes, you need to declare the lifetime in the `impl` block:

```rust
impl<'a> Book<'a> {
    fn get_title(&self) -> &str {
        self.title
    }
}
```

Here, lifetime elision rule 3 applies: since `get_title` takes `&self`, the returned reference has the same lifetime as `self`.

### The 'static lifetime

The `'static` lifetime is special: it means the reference is valid for the entire program duration.  All string literals have the `'static` lifetime:

```rust
let s: &'static str = "I have a static lifetime";
```

The text of string literals is stored directly in the program's binary, so it is always available.

Be careful with `'static` bounds.  Often, the error message suggests adding `'static`, but this is usually not the right solution.  Most of the time, the problem is a dangling reference or a mismatch in lifetimes, not a need for `'static`.

Only use `'static` when the data truly needs to live for the entire program.

***

## 9. Static Items

### What is static

A `static` item is a value that lives for the entire duration of the program.  It occupies a single fixed memory address.

```rust
static MAX_CONNECTIONS: u32 = 100;

fn main() {
    println!("Maximum connections: {}", MAX_CONNECTIONS);
}
```

All references to a `static` item point to the same memory location.  This is different from `const`, where each use gets its own copy.

### Static vs const comparison

The differences between `static` and `const` are important:


| Feature | const | static |
| :-- | :-- | :-- |
| Memory location | No fixed address; inlined at each use  | Single fixed address  |
| Lifetime | N/A (inlined)  | 'static  |
| Mutability | Always immutable  | Can be mutable with `static mut`  |
| Address stability | Different address for each use  | Same address always  |
| Thread safety requirement | None  | Must implement Sync (for immutable)  |



### When to use static

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


### Mutable statics and safety

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

### The Sync requirement

Immutable `static` items must implement the `Sync` trait, which means they are safe to access from multiple threads.  Most types with only immutable data are automatically `Sync`.

```rust
static NUMBERS: [i32; 3] = [1, 2, 3]; // OK: arrays of i32 are Sync
```

Types like `Cell` and `RefCell` are not `Sync`, so you cannot use them in a `static` directly.  You would need to wrap them in a thread-safe type.

***

## 10. Rust 2024 Edition Changes

### The static_mut_refs lint

In the Rust 2024 Edition, creating any reference to a `static mut` (either `&` or `&mut`) is forbidden by default through the `static_mut_refs` lint.

This is because creating such references can lead to undefined behavior, even if you never use them.  The compiler cannot guarantee safety when references to mutable statics exist.

Old code that worked in previous editions:

```rust
static mut COUNTER: u32 = 0;

fn main() {
    unsafe {
        let r = &COUNTER; // ERROR in 2024 edition
        println!("{}", r);
    }
}
```


### Migration to safe patterns

Instead of taking references to `static mut`, use raw pointers:

```rust
static mut COUNTER: u32 = 0;

fn main() {
    unsafe {
        let ptr = &raw const COUNTER; // OK: raw pointer, not reference
        println!("{}", *ptr);
        
        let mut_ptr = &raw mut COUNTER;
        *mut_ptr += 1;
    }
}
```

Better yet, use safe alternatives like atomics, `Mutex`, or `OnceLock` instead of `static mut`.

***

## 11. Safe Global State Patterns

### Atomic types

For simple counters and flags, use atomic types from `std::sync::atomic`:

```rust
use std::sync::atomic::{AtomicUsize, Ordering};

static COUNTER: AtomicUsize = AtomicUsize::new(0);

fn main() {
    COUNTER.fetch_add(1, Ordering::Relaxed);
    println!("Counter: {}", COUNTER.load(Ordering::Relaxed));
}
```

Atomics provide thread-safe operations without locks.  They are perfect for counters, flags, and other simple state.

### Mutex and RwLock

For more complex shared state, use `Mutex` or `RwLock`:

```rust
use std::sync::Mutex;

static NAMES: Mutex<Vec<String>> = Mutex::new(Vec::new());

fn main() {
    let mut names = NAMES.lock().unwrap();
    names.push(String::from("Alice"));
    names.push(String::from("Bob"));
    // lock is released here

    let names = NAMES.lock().unwrap();
    println!("Names: {:?}", names);
}
```

Mutex ensures only one thread can access the data at a time. RwLock allows multiple readers or one writer, similar to Rust's borrowing rules.

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

### OnceLock and LazyLock

`OnceLock` and `LazyLock` are for one-time initialization:

```rust
use std::sync::OnceLock;

static CONFIG: OnceLock<String> = OnceLock::new();

fn main() {
    CONFIG.set(String::from("production")).unwrap();
    
    println!("Config: {}", CONFIG.get().unwrap());
    
    // CONFIG.set(String::from("dev")).unwrap(); // ERROR: already initialized
}
```

`LazyLock` is similar but takes a function for initialization:

```rust
use std::sync::LazyLock;

static EXPENSIVE: LazyLock<Vec<i32>> = LazyLock::new(|| {
    println!("Initializing...");
    vec![1, 2, 3, 4, 5]
});

fn main() {
    println!("Before access");
    println!("{:?}", *EXPENSIVE); // initialization happens here
    println!("{:?}", *EXPENSIVE); // uses cached value
}
```


### thread_local! macro

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

Each thread gets its own copy of the data, preventing cross-thread interference.

***

## 12. Best Practices and Decision Guide

### Choosing between const and static

Use `const` when:

- The value is known at compile time and never changes
- You do not need a fixed memory address
- The value is small and cheap to copy
- Examples: mathematical constants, configuration values

Use `static` when:

- You need a fixed memory address (for FFI or pointer comparison)
- The data is large and you want to avoid duplicating it
- You need global mutable state with interior mutability
- Examples: global caches, configuration loaded at runtime


### When to move vs borrow

Move ownership when:

- The caller no longer needs the value
- You are transferring a resource (like a file handle or network connection)
- The function needs to modify and return the value

Borrow when:

- The caller still needs the value after the function call
- You only need to read the value
- You want to modify the value temporarily but return control to the caller


### Common pitfalls

**Pitfall 1**: Fighting the borrow checker instead of understanding it.  If you get an error, think about why Rust is preventing it.  Usually, there is a real safety issue.

**Pitfall 2**: Cloning too much.  Cloning is sometimes the right solution, but if you find yourself cloning everywhere, you might be working against ownership instead of with it.

**Pitfall 3**: Using `'static` bounds unnecessarily.  This restricts your API to only work with static data and prevents it from working with borrowed data.

**Pitfall 4**: Using `static mut` when safe alternatives exist.  Prefer atomics, locks, or `OnceLock` over `static mut`.

### Performance considerations

Rust's ownership system has zero runtime cost.  All the checks happen at compile time.

Moving small values (like integers) is as cheap as copying.  Moving large structures just transfers ownership of the pointer, not the entire data.

Borrowing also has zero cost: a reference is just a pointer under the hood.

The only potential performance impact comes from excessive cloning or locking.  Design your APIs to minimize both.

***

This guide has covered all the fundamental concepts of Rust's ownership, borrowing, lifetimes, and memory management systems.  With this knowledge, you can write safe, efficient Rust code and understand how to debug borrow checker errors when they occur.  All information has been validated against Rust 1.90.0 and accounts for the 2024 Edition changes.
<span style="display:none"></span>

<div align="center">⁂</div>

: 2025-10-7-rust-ownership.md

: 2025-10-8-var-const-lifetimes.md

: https://stackoverflow.com/questions/50251487/what-are-non-lexical-lifetimes

: https://masteringbackend.com/posts/understanding-lifetime-elision-in-rust

: https://web.mit.edu/rust-lang_v1.26.0/arch/amd64_ubuntu1404/share/doc/rust/html/reference/lifetime-elision.html

: 2025-10-10-static.md

: https://doc.rust-lang.org/edition-guide/rust-2024/static-mut-references.html

: https://releases.rs/docs/1.90.0/

: https://www.reddit.com/r/rust/comments/1nk8mi2/rust_1900_is_out/

: https://doc.rust-lang.org/beta/releases.html

: https://www.youtube.com/watch?v=kEtNlTV14Ms

: https://github.com/rust-lang/rust-wiki-backup/blob/master/Doc-detailed-release-notes.md

: https://www.linuxcompatible.org/story/rust-1900-released/

: https://github.com/Hunterdii/30-Days-Of-Rust/blob/main/21_Rust%20Lifetimes/21_rust_lifetimes.md

: http://blog.pnkfx.org/blog/2019/06/26/breaking-news-non-lexical-lifetimes-arrives-for-everyone/

: https://releases.rs

: https://users.rust-lang.org/t/about-non-lexical-lifetimes/111614

: https://doc.rust-lang.org/stable/releases.html

: https://doc.rust-lang.org/nomicon/lifetime-elision.html

: https://blog.rust-lang.org/2022/08/05/nll-by-default.html

: https://lwn.net/Articles/1038649/

: https://doc.rust-lang.org/reference/lifetime-elision.html

: https://rust-lang.github.io/rfcs/2094-nll.html

: https://news.tuxmachines.org/n/2025/09/19/Announcing_Rust_1_90_0.shtml

