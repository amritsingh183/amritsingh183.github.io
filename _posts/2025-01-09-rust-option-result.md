---
layout: post
title: "Mastering Rust's `Option` and `Result` Types: A Complete Guide"
date: 2025-01-09 13:23:00 +0530
categories: rust concepts
last_updated: 2025-10-18
---
# Mastering Rust's Option and Result Types: A Complete Guide

Rust eliminates entire classes of bugs found in other languages—null pointer exceptions, uncaught exceptions, and ignored errors—through its type system. This guide explores how Rust achieves this through two fundamental enum types: `Option` and `Result`.

## Prerequisites

This guide assumes you understand Rust's ownership, borrowing, and type safety fundamentals. We'll build from pattern matching through increasingly sophisticated error handling patterns.

**References and pointers in Rust refer to related but distinct concepts**

- References (&T or &mut T) are safe pointers with strict rules enforced by the compiler.
    - References have extra semantic rules and safety enforced by the compiler, making them safe to use for borrowing data.
    - In safe Rust, references are guaranteed to be non-null, properly aligned, and valid for the duration of their lifetimes; the borrow checker enforces aliasing rules (many `&T` or exactly one `&mut T`) so dereferencing is always safe.
    - They have associated lifetimes ensuring they do not outlive the data they reference.
    - Mutable references enforce exclusive access (no aliasing).
    - You cannot perform pointer arithmetic or raw memory access with references.
    - References behave as aliases to the underlying data with compiler guarantees for safety.
- Raw pointers (*const T and *mut T) are unsafe pointers akin to C pointers.
    - Pointers are just addresses without safety or lifetime guarantees, requiring explicit unsafe code to dereference.
    - They are simple memory addresses without lifetimes or borrowing rules.
    - They can be null, dangling, or invalid.
    - You can perform arithmetic on raw pointers using methods like `add()`, `offset()`, or by casting to `usize` and back.
    - Creating references to uninitialized, improperly aligned, or invalid memory through raw pointers is undefined behavior.
    - Accessing the memory they point to requires unsafe blocks.
    - They provide low-level control but without safety guarantees enforced by Rust.

***
## Enums and Pattern Matching: The Foundation

Before diving into `Option` and `Result`, you need to understand Rust's enum pattern matching, as it's the foundation for working with these types.

### Rust Enums

Enums in Rust can hold data within their variants:

```rust

enum Message {
    Quit,                       // No data
    Move { x: i32, y: i32 },   // Named fields
    Write(String),              // Single value
    Color(u8, u8, u8),         // Multiple values
}

```

### Pattern Matching with match

The `match` expression destructures enums and handles all possible variants:

```rust

let msg = Message::Write(String::from("hello"));

match msg {
    Message::Quit => println!("Quit message"),
    Message::Move { x, y } => println!("Move to x:{}, y:{}", x, y),
    Message::Write(text) => println!("Text: {}", text),
    Message::Color(r, g, b) => println!("RGB: {}, {}, {}", r, g, b),
}

```

The compiler enforces **exhaustiveness**: you must handle every possible variant. This guarantee is what makes Rust's error handling so robust.

### Advanced Pattern Matching

**Match guards** add conditions to patterns:

```rust

let some_number = 42;

match some_number {
    x if x < 0 => println!("Negative: {}", x),
    x if x > 100 => println!("Large: {}", x),
    x => println!("Normal: {}", x),
}

```

**Destructuring with `if let`** handles a single pattern:

```rust

let msg = Message::Write(String::from("hello"));
if let Message::Write(text) = msg {
    println!("Got text: {}", text);
}
// Other variants are ignored

```

**The `let...else` pattern** handles one case and exits for others:

```rust

let msg = Message::Write(String::from("hello"));
let Message::Write(text) = msg else {
    println!("Not a write message!");
    return;
};
// text is now available here

```

***

## The Option Type: Handling Optional Values

### Understanding Option, Some, and None

The `Option` type represents a value that might not exist. It's defined as:

```rust

enum Option<T> {
    None,
    Some(T),
}

```

This replaces the concept of "null" found in other languages, but **with type safety**. You cannot use an `Option<T>` as if it were a `T`—you must explicitly handle both cases.

### Understanding Some() and None

**Some(value)** is a variant constructor that **wraps a value** inside an Option. When a function returns `Some(5)`, it's saying "I have a value, and that value is 5."

**None** represents the absence of a value—it's Rust's type-safe replacement for null. Unlike null in other languages, you cannot accidentally use `None` as if it were a value; the compiler forces you to handle it explicitly.

```rust

let some_number: Option<i32> = Some(42);  // Wraps the value 42
let no_number: Option<i32> = None;        // No value present

// Some and None can be written without Option::
// The compiler resolves the enum through type inference
let x = Some(5);
let y: Option<i32> = None;

```

**Key insight:** `Some()` and `None` aren't magic—they're just enum variants, similar to `Message::Write()` or `Message::Quit` from earlier examples. The difference is that `Some` wraps any type `T`, making `Option` generic and reusable for any situation where a value might be absent.

### Creating Option Values

```rust

// Explicit type annotation
let some_number: Option<i32> = Some(42);
let no_number: Option<i32> = None;

// Type inference
let x = Some(5);                   // Rust infers Option<i32>
let name = Some(String::from("Alice"));  // Option<String>

// None requires type annotation (Rust can't infer the type)
let y: Option<i32> = None;

```

### Real-World Example: Finding Elements

```rust

fn find_first_positive(numbers: &[i32]) -> Option<i32> {
    for &num in numbers {
        if num > 0 {
            return Some(num);  // Found a positive number
        }
    }
    None  // No positive number found
}

fn main() {
    let list = vec![-1, -5, 3, -2, 7];

    match find_first_positive(&list) {
        Some(num) => println!("First positive: {}", num),
        None => println!("No positive numbers found"),
    }
}

```

### Extracting Values from Option

**Using pattern matching:**

```rust

let value = Some(5);

match value {
    Some(x) => println!("Value is {}", x),
    None => println!("No value"),
}

```

**Using `if let` for the success case only:**

```rust

if let Some(x) = value {
    println!("Value is {}", x);
}

```

**Using `let...else` to handle None and exit:**

```rust

fn process_value(opt: Option<i32>) {
    let Some(x) = opt else {
        println!("No value provided!");
        return;
    };

    // Use x here - we know it exists
    println!("Processing: {}", x);
    }

```

***

## The Result Type: Handling Success and Failure

### Understanding Result, Ok, and Err

The `Result` type represents an operation that can succeed or fail:

```rust

enum Result<T, E> {
    Ok(T),   // Success: contains value of type T
    Err(E),  // Failure: contains error of type E
}

```

Unlike `Option`, which only indicates presence or absence, `Result` carries **meaningful error information** when operations fail.

### Understanding Ok() and Err()

**Ok(value)** is a variant constructor that **wraps a successful result**. When a function returns `Ok(42)`, it's saying "the operation succeeded, and here's the result: 42."

**Err(error)** wraps an error value, indicating that an operation failed and providing information about **why** it failed. The error can be any type—a string, a custom error struct, or any type implementing the `Error` trait.

```rust

// Success case
let success: Result<i32, String> = Ok(42);

// Failure case with error message
let failure: Result<i32, String> = Err(String::from("Something went wrong"));

// Different error types
let parse_error: Result<i32, std::num::ParseIntError> = "abc".parse();  // Err
let io_error: Result<String, std::io::Error> = std::fs::read_to_string("missing.txt");  // Err

```

**Key insight:** Like `Some` and `None` for `Option`, `Ok` and `Err` are enum variants. But `Result` has **two** type parameters: `T` for the success type and `E` for the error type. This allows you to specify exactly what kind of error information your function provides when it fails.

### Creating Result Values

```rust

fn divide(numerator: f64, denominator: f64) -> Result<f64, String> {
    if denominator == 0.0 {
        Err(String::from("Cannot divide by zero"))
    } else {
        Ok(numerator / denominator)
    }
}

```

### Real-World Example: Input Validation

```rust

fn validate_age(age: i32) -> Result<String, String> {
    if age < 0 {
        return Err(String::from("Age cannot be negative"));
    }
    if age > 150 {
        return Err(String::from("Age is unrealistically high"));
    }

    let category = if age < 13 {
        "child"
    } else if age < 20 {
        "teenager"
    } else if age < 65 {
        "adult"
    } else {
        "senior"
    };

    Ok(String::from(category))
}

fn main() {
    match validate_age(25) {
        Ok(category) => println!("Category: {}", category),
        Err(error) => eprintln!("Error: {}", error),
    }
}

```

### Extracting Values from Result

**Using pattern matching:**

```rust

let result = divide(10.0, 2.0);

match result {
    Ok(value) => println!("Result: {}", value),
    Err(error) => eprintln!("Error: {}", error),
}

```

**Pattern matching with guards:**

```rust

match divide(100.0, 5.0) {
    Ok(value) if value > 10.0 => println!("Large result: {}", value),
    Ok(value) => println!("Small result: {}", value),
    Err(e) => eprintln!("Error: {}", e),
}

```

**Using `if let` for one case:**

```rust

if let Ok(value) = divide(10.0, 2.0) {
    println!("Success: {}", value);
}

```

**Using `let...else` to handle errors and exit:**

```rust

fn process_result(input: &str) {
    let Ok(number) = input.parse::<i32>() else {
        eprintln!("Failed to parse: {}", input);
        return;
    };

    // Use number here
    println!("Parsed number: {}", number);
}

```

***

## Essential Methods: Working with Option and Result

Now that you understand `Some`, `None`, `Ok`, and `Err`, let's explore the methods that make these types powerful for real-world programming.

### Unwrapping Methods

These methods extract values but behave differently when encountering `None` or `Err`.

#### unwrap() - Panics on Failure

```rust

let x = Some(5).unwrap();              // x = 5
let y: Option<i32> = None;
// let z = y.unwrap();                 // Panics: "called `Option::unwrap()` on a `None` value"

let result: Result<i32, &str> = Ok(10);
let value = result.unwrap();           // value = 10

```

**Use case:** Prototyping, tests, or when you're absolutely certain the value exists. Warning: In production code, prefer using the ? operator or explicit error handling (match, if let) over unwrap()/expect(), as these methods will panic and crash your program if called on None/Err.

#### expect() - Panics with Custom Message

```rust

let config = load_config()
.expect("Config file must exist for application to run");

```

**Use case:** When failure is unrecoverable and you want a meaningful error message.

#### unwrap_or() - Provide a Default Value

```rust

let x = Some(5).unwrap_or(10);         // x = 5
let y: Option<i32> = None;
let z = y.unwrap_or(10);               // z = 10

let result: Result<i32, &str> = Err("failed");
let value = result.unwrap_or(0);       // value = 0

```

**Use case:** When you have a sensible default value.

#### unwrap_or_else() - Compute Default Lazily

```rust

let x: Option<String> = None;
let y = x.unwrap_or_else(|| {
    expensive_computation()            // Only runs if None
});

let result: Result<i32, String> = Err(String::from("error"));
let value = result.unwrap_or_else(|err| {
    log_error(&err);
    0  // Fallback value
});

```

**Use case:** When computing the default is expensive or has side effects. This is more efficient than `unwrap_or()` because the fallback value is only computed when needed.

#### unwrap_or_default() - Use Type's Default

```rust

let x: Option<String> = None;
let y = x.unwrap_or_default();         // y = "" (empty string)

let z: Option<Vec<i32>> = None;
let v = z.unwrap_or_default();         // v = [] (empty vector)

```

**Use case:** When the type implements the `Default` trait and its default makes sense.

#### is_none_or() - Checking None or Conditional Some

The `is_none_or` method complements `is_some_and` by returning `true` if the `Option` is `None` **or** if it's `Some` and the value satisfies the predicate. This is particularly useful for validation scenarios where absence is acceptable, or a present value must meet certain criteria.

```rust

let x: Option<u32> = Some(2);
assert_eq!(x.is_none_or(|x| x > 1), true);   // Predicate passes

let x: Option<u32> = Some(0);
assert_eq!(x.is_none_or(|x| x > 1), false);  // Predicate fails

let x: Option<u32> = None;
assert_eq!(x.is_none_or(|x| x > 1), true);   // None is accepted

```

**Real-world example:**

```rust

struct Config {
    max_connections: Option<usize>,
}

fn validate_config(config: &Config) -> bool {
    // Valid if no limit is set, or if the limit is reasonable
    config.max_connections.is_none_or(|&n| n > 0 && n <= 10000)
}

```

**Use case:** Replacing verbose patterns like `opt.is_none() || opt.is_some_and(|x| predicate)` with the more concise `opt.is_none_or(|x| predicate)`.

***


**`is_some_and` - Test Option with a Predicate**

Returns `true` if the option is `Some` and the value satisfies the predicate:

```rust

let x: Option<u32> = Some(42);
assert!(x.is_some_and(|n| n > 40));  // true

let y: Option<u32> = Some(5);
assert!(!y.is_some_and(|n| n > 40)); // false - predicate fails

let z: Option<u32> = None;
assert!(!z.is_some_and(|n| n > 40)); // false - is None

```

**`is_ok_and` - Test Result with a Predicate**

Returns `true` if the result is `Ok` and the value satisfies the predicate:

```rust

let result: Result<i32, &str> = Ok(42);
assert!(result.is_ok_and(|n| n > 40));  // true

let error: Result<i32, &str> = Err("failed");
assert!(!error.is_ok_and(|n| n > 40));  // false - is Err

```

*Use case:* Replacing verbose patterns like `if let Some(x) = opt { x > 5 } else { false }` with the more concise `opt.is_some_and(|x| x > 5)`.

**Why these methods matter:**

```rust

// Before: verbose pattern matching
fn is_valid_age(age: Option<i32>) -> bool {
    match age {
        Some(a) if a >= 18 && a <= 120 => true,
        _ => false,
    }
}

// After: concise predicate
fn is_valid_age(age: Option<i32>) -> bool {
    age.is_some_and(|a| a >= 18 && a <= 120)
}

```

### Transformation Methods

These methods transform values while keeping them wrapped in `Option` or `Result`.

#### map() - Transform the Success Value

The `map` method applies a function to the contained value. The key characteristic: **the function returns a plain value**, not an `Option` or `Result`.

```rust

let x = Some(5).map(|n| n * 2);        // Some(10)
let y: Option<i32> = None;
let z = y.map(|n| n * 2);              // None

let result: Result<i32, String> = Ok(5);
let doubled = result.map(|n| n * 2);   // Ok(10)

```

**Real-world example:**

```rust

fn parse_and_square(input: &str) -> Option<i32> {
    input.parse::<i32>()
    .ok()
    .map(|n| n * n)
}

// "5" -> Some(25)
// "abc" -> None

```

#### and_then() - Chain Fallible Operations

The `and_then` method is used when your transformation **can itself fail**. Unlike `map`, the function must return an `Option<U>` or `Result<U, E>` (not just `U`).

```rust

fn parse_positive(s: &str) -> Option<i32> {
    s.parse::<i32>().ok()
    .and_then(|n| if n > 0 { Some(n) } else { None })
}

// "5" -> Some(5)
// "-3" -> None
// "abc" -> None

```

**The key difference between `map` and `and_then`:**

```rust

// map: function returns plain value (auto-wrapped)
Some(5).map(|x| x * 2)              // Some(10)

// and_then: function returns Option (not wrapped again)
Some(5).and_then(|x| Some(x * 2))   // Some(10)
Some(5).and_then(|x| None)          // None - transformation can fail

```

**Real-world chaining example:**
```rust

fn process_input(input: &str) -> Option<i32> {
    parse_number(input)
    .and_then(validate_positive)
    .and_then(compute_result)
}

fn parse_number(s: &str) -> Option<i32> {
    s.parse().ok()
}

fn validate_positive(n: i32) -> Option<i32> {
    if n > 0 { Some(n) } else { None }
}

fn compute_result(n: i32) -> Option<i32> {
    if n < 1000 { Some(n * 2) } else { None }
}

```

#### map_err() - Transform the Error

For `Result`, you can transform the error type while leaving success values unchanged:

```rust

fn parse_number(s: &str) -> Result<i32, String> {
    s.parse::<i32>()
    .map_err(|e| format!("Failed to parse '{}': {}", s, e))
}

```

#### inspect() - Observe Values Without Consuming

The `inspect()` methods consume `self` by value, pass a reference (`&T`) to the contained value to the closure (allowing temporary observation), and return the original `Option` or `Result` unchanged, enabling method chaining.

In the code below:
- The container is moved (consumed)
- The closure receives a reference
- The container is returned (not a copy)

```rust

let result = Some(5)
.inspect(|x| println!("Got value: {}", x))
.map(|x| x * 2);
// Prints: "Got value: 5"
// result = Some(10)

let parsed: Result<i32, _> = "123".parse()
.inspect(|x| println!("Parsed successfully: {}", x))
.inspect_err(|e| eprintln!("Parse error: {}", e));

```

**Use case:** Debugging, logging, or metrics collection without modifying the data flow.

### Boolean Combinators

The `and` and `or` methods provide boolean-like logic for combining `Option` and `Result` values.

#### The `and` Method

Returns the second value if the first is `Some`/`Ok`, otherwise returns the first (`None`/`Err`):

| First | Second | Result |
| :-- | :-- | :-- |
| `Some(x)` | `Some(y)` | `Some(y)` |
| `Some(x)` | `None` | `None` |
| `None` | `Some(y)` | `None` |
| `None` | `None` | `None` |

```rust

// Option
Some(2).and(Some(100))              // Some(100)
Some(2).and(None)                   // None
None.and(Some(100))                 // None

// Result
Ok(2).and(Ok(100))                  // Ok(100)
Ok(2).and(Err("error"))             // Err("error")
Err("early").and(Ok(100))           // Err("early")

```

#### The `or` Method

Returns the first value if it's `Ok`/`Some`, otherwise returns the second:

| First | Second | Result |
| :-- | :-- | :-- |
| `Some(x)` | `Some(y)` | `Some(x)` |
| `Some(x)` | `None` | `Some(x)` |
| `None` | `Some(y)` | `Some(y)` |
| `None` | `None` | `None` |

```rust

// Option
Some(2).or(Some(100))               // Some(2)
None.or(Some(100))                  // Some(100)
None.or(None)                       // None

// Result
Ok(2).or(Ok(100))                   // Ok(2)
Err("error").or(Ok(100))            // Ok(100)
Err("error1").or(Err("error2"))     // Err("error2")

```

**Use case - fallback chains:**

```rust

fn get_config() -> Option<Config> {
    load_from_file()
    .or(load_from_env())
    .or(default_config())
}

```

***

## Working with Borrowed Data: as_ref, as_deref, and Friends

When working with `Option` and `Result`, you often need to access inner values without taking ownership. Rust provides methods to borrow references through these wrappers.

### Borrowing Without Moving: as_ref and as_mut

The `as_ref()` method converts `Option<T>` → `Option<&T>` and `Result<T, E>` → `Result<&T, &E>`, allowing you to work with references instead of consuming owned values:

```rust

let name: Option<String> = Some("Alice".to_string());

// Borrow to compute length without moving the String
let len: Option<usize> = name.as_ref().map(|s| s.len());
// name is still usable here
println!("Name: {:?}", name);  // Still owns the String

```

For mutable access, use `as_mut()`:

```rust

let mut count: Option<i32> = Some(1);
if let Some(v) = count.as_mut() {
    *v += 1;  // Mutate in place
}
// count = Some(2)

```

**With Result:**

```rust

let result: Result<String, std::io::Error> = Ok("success".into());
// Option A: borrow the owned String
let borrowed: Result<&String, &std::io::Error> = result.as_ref();
// Option B: borrow the deref target (&str)
let borrowed_str: Result<&str, &std::io::Error> = result.as_deref();
// result still owns the String

```

### Working with Smart Pointers: as_deref and as_deref_mut

For types like `String`, `Vec<T>`, `Box<T>`, and others that implement `Deref`, use `as_deref()` to get a reference to the dereferenced type:

```rust

// Option<String> -> Option<&str>
let owned: Option<String> = Some("hello".into());
let borrowed: Option<&str> = owned.as_deref();
// No allocation, just borrows

// Option<Vec<i32>> -> Option<&[i32]>
let vec: Option<Vec<i32>> = Some(vec![1, 2, 3]);
let slice: Option<&[i32]> = vec.as_deref();

```

**With Result:**

```rust

// Result<String, E> -> Result<&str, &E>
let result: Result<String, std::io::Error> = Ok("data".into());
let deref: Result<&str, &std::io::Error> = result.as_deref();

// Mutable deref
let mut s: Result<String, String> = Ok("hello".into());
if let Ok(text) = s.as_deref_mut() {
    text.make_ascii_uppercase();
}
// s = Ok("HELLO")

```

### Materializing Values: copied and cloned

When you have `Option<&T>` or `Result<&T, E>` and need owned values, use `copied()` for `Copy` types or `cloned()` for `Clone` types:

```rust

let x = 42;
let reference: Option<&i32> = Some(&x);
let owned: Option<i32> = reference.copied();  // i32 is Copy

// With Clone
let s = String::from("hello");
let reference: Option<&String> = Some(&s);
let owned: Option<String> = reference.cloned();

```

**With Result:**

```rust

let x = 42;
let r: Result<&i32, &str> = Ok(&x);
let owned: Result<i32, &str> = r.copied();

```

**Why this matters:** These methods prevent unnecessary clones and moves, making your code more efficient:

```rust

// ❌ Inefficient: clones every time
fn check_length(opt: Option<String>) -> bool {
    opt.clone().map(|s| s.len() > 5).unwrap_or(false)
}

// ✅ Efficient: only borrows
fn check_length(opt: &Option<String>) -> bool {
    opt.as_ref().map(|s| s.len() > 5).unwrap_or(false)
}

```

## Converting Between Option and Result

Sometimes you need to convert between these types. Rust provides methods for both directions.

### Option to Result: Providing Error Context

#### ok_or() - Static Error Value

Converts `Option<T>` to `Result<T, E>` by providing an error value for the `None` case:

```rust

let some_value: Option<i32> = Some(5);
let result: Result<i32, &str> = some_value.ok_or("Value not found");
// result = Ok(5)

let no_value: Option<i32> = None;
let result2 = no_value.ok_or("Value not found");
// result2 = Err("Value not found")

```

**Use case:** When you need to treat `None` as an error with a specific error message.

#### ok_or_else() - Computed Error Value

Use a closure to compute the error value lazily (only if `None`):

```rust

fn find_user(id: u32) -> Option<User> {
    // search logic
}

fn get_user(id: u32) -> Result<User, String> {
    find_user(id).ok_or_else(|| {
        format!("User with id {} not found", id)
    })
}

```

**When to use `ok_or_else`:** When creating the error value is expensive or involves computation.

### Result to Option: Discarding Error Information

#### ok() - Extract Success Value

Converts `Result<T, E>` to `Option<T>`, discarding error information:

```rust

let result: Result<i32, String> = Ok(42);
let option: Option<i32> = result.ok();     // Some(42)

let error: Result<i32, String> = Err(String::from("failed"));
let option2 = error.ok();                  // None

```

**Use case:** When you only care about success and want to ignore error details.

#### err() - Extract Error Value

Converts `Result<T, E>` to `Option<E>`, keeping only the error:

```rust

let result: Result<i32, String> = Err(String::from("failed"));
let error_option: Option<String> = result.err();  // Some("failed")

let success: Result<i32, String> = Ok(42);
let error_option2 = success.err();                // None

```

**Use case:** When you want to examine or log only errors.

### Interconverting Nested Types: transpose

The `transpose()` method swaps the layers between `Option<Result<T, E>>` and `Result<Option<T>, E>`:

```rust

// Option<Result<T, E>> -> Result<Option<T>, E>
let nested: Option<Result<i32, &str>> = Some(Ok(5));
let swapped: Result<Option<i32>, &str> = nested.transpose();
// Ok(Some(5))

let error_case: Option<Result<i32, &str>> = Some(Err("failed"));
let swapped2 = error_case.transpose();
// Err("failed")

// Result<Option<T>, E> -> Option<Result<T, E>>
let result: Result<Option<i32>, &str> = Ok(Some(10));
let option: Option<Result<i32, &str>> = result.transpose();
// Some(Ok(10))

```

**Use case:** When parsing optional fields that can fail, or combining optional and fallible operations:

```rust

fn parse_optional_field(s: Option<&str>) -> Result<Option<i32>, ParseError> {
    s.map(|s| s.parse()).transpose()
}

```

***

## The Question Mark Operator: Ergonomic Error Propagation

The `?` operator is Rust's most powerful tool for error handling. It provides concise error propagation without sacrificing type safety.

```rust

use std::io;
use std::num::ParseIntError;

#[derive(Debug)]
enum MyError {
    Parse(ParseIntError),
    Io(io::Error),
}

impl From<ParseIntError> for MyError {
    fn from(err: ParseIntError) -> Self {
        MyError::Parse(err)
    }
}

impl From<io::Error> for MyError {
    fn from(err: io::Error) -> Self {
        MyError::Io(err)
    }
}

fn read_number() -> Result<i32, MyError> {
    let s = std::fs::read_to_string("number.txt")?; // io::Error -> MyError
    let n = s.trim().parse()?; // ParseIntError -> MyError
    Ok(n)
}

```

### How ? Works

When applied to a `Result` or `Option`:

- If `Ok(value)` or `Some(value)`: unwraps and continues with the value
- If `Err(e)` or `None`: returns early from the function with the error

The `?` operator also performs automatic type conversion using the `From` trait, allowing different error types to be automatically converted when propagating errors up the call stack.

```rust

use std::fs::File;
use std::io::{self, Read};

fn read_file_to_string(path: &str) -> Result<String, io::Error> {
    let mut file = File::open(path)?;  // Returns Err if file doesn't open
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;  // Returns Err if read fails
    Ok(contents)
}

```

### Without ? (Verbose)

```rust

fn read_file_verbose(path: &str) -> Result<String, io::Error> {
    let mut file = match File::open(path) {
        Ok(f) => f,
        Err(e) => return Err(e),
    };

    let mut contents = String::new();
    match file.read_to_string(&mut contents) {
        Ok(_) => Ok(contents),
        Err(e) => Err(e),
    }
    }

```

### Using ? with Option

The `?` operator also works with `Option`, where `None` causes an early return:

```rust

fn add_last_numbers(list1: &[i32], list2: &[i32]) -> Option<i32> {
    let a = list1.last()?;  // Option<&i32>
    let b = list2.last()?;  // Option<&i32>
    Some(*a + *b)  // Dereference to get i32 values
}

```

### Chaining Operations with ?

The `?` operator makes complex error handling readable:

```rust

use std::fs;

fn process_config(path: &str) -> Result<Config, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(path)?;
    let parsed = parse_toml(&content)?;
    let validated = validate_config(parsed)?;
    Ok(validated)
}

```

### Constraints on Using ?

On stable Rust, the `?` operator works ergonomically with `Result` and `Option` return types (including `main() -> Result<...>`). Generic integration via the `Try` and `FromResidual` traits remains nightly-only:

```rust

// ✓ Valid
fn foo() -> Result<i32, String> {
    let x = some_result?;
    Ok(x)
}

// ✗ Invalid - main returns ()
fn main() {
    let x = some_result?;  // Compile error!
}

// ✓ Valid - main can return Result
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let x = some_result?;
    Ok(())
}

```

***

## Understanding main() -> Result: The Termination Trait

You may have noticed that `main` can return a `Result`, which enables the `?` operator to work at the top level. This works because of the `std::process::Termination` trait.

For a type to be a valid return type for `main`, it must implement `Termination`. The standard library provides an implementation for `Result<(), E>` where `E` implements `std::fmt::Debug`. When the result is `Err(e)`, the `Termination` implementation prints the error's `Debug` representation to stderr via the standard runtime and returns a non-zero exit code (typically 1) to indicate failure. When the result is `Ok(())`, it returns exit code 0 to indicate success.
**Example:**

```rust

use std::fs;

fn main() -> Result<(), std::io::Error> {
    let content = fs::read_to_string("config.toml")?;
    println!("Config loaded successfully");
    Ok(())
}

```

If the file doesn't exist, the program automatically exits with an error message:

> Error: Os { code: 2, kind: NotFound, message: "No such file or directory" }

This eliminates the need for explicit error handling in `main`, making your program's entry point cleaner while preserving proper error reporting and exit codes.
***

## Practical Decision Guide

### When to Use Option

Use `Option<T>` when:

- A value might legitimately not exist
- Absence is **not an error condition**
- Examples: finding an element in a collection, optional configuration fields, nullable database columns

```rust

fn find_user_by_email(email: &str) -> Option<User> {
    // Not finding a user isn't an error - they might not exist
}

struct Config {
    required_field: String,
    optional_field: Option<String>,  // May or may not be present
}

```

### When to Use Result

Use `Result<T, E>` when:

- An operation can fail with **meaningful error information**
- You need to propagate errors up the call stack
- Failure requires different handling than success
- Examples: file I/O, parsing, network requests, validation

```rust

fn parse_json(data: &str) -> Result<JsonValue, ParseError> {
    // Parsing can fail - need to know why
}

fn open_database(url: &str) -> Result<Connection, DbError> {
    // Connection might fail - caller needs error details
}

```

### Choosing Between Methods

**For extraction:**

- `unwrap()` / `expect()` → Use in examples, tests, or when failure is truly impossible
- `unwrap_or()` / `unwrap_or_else()` → Use when you have a sensible fallback
- `?` operator → Use when you want to propagate errors to the caller
- `match` → Use when you need different logic for each case

**For transformation:**

- `map()` → Transform a value (transformation always succeeds)
- `and_then()` → Chain operations that might fail
- `or()` / `or_else()` → Provide alternatives when operations fail
- `inspect()` → Observe values for debugging/logging without modifying them

**For conversion:**

- `ok_or()` / `ok_or_else()` → Convert Option to Result when you need error context
- `ok()` → Convert Result to Option when you don't care about error details
- **`flatten`**: Collapse nested `Result<Result<T, E>, E>` when validation or transformation chains produce double-wrapped results
- **`as_slice`/`as_mut_slice`**: Convert `Option<T>` to slices for uniform iteration with slice-based APIs
***

## Quick Reference

### Option<T> Methods

| Category | Method | Signature | Purpose |
| :-- | :-- | :-- | :-- |
| **Extraction** | `unwrap()` | `Option<T> -> T` | Get value or panic |
|  | `expect(msg)` | `Option<T> -> T` | Get value or panic with message |
|  | `unwrap_or(default)` | `Option<T> -> T` | Get value or return default |
|  | `unwrap_or_else(f)` | `Option<T> -> T` | Get value or compute default lazily |
|  | `unwrap_or_default()` | `Option<T> -> T` | Get value or use type's Default |
|  | `unwrap_unchecked()` ⚠️ | `Option<T> -> T` | Get value without checking (unsafe) |
| **Querying** | `is_some()` | `&Option<T> -> bool` | Returns true if Some |
|  | `is_none()` | `&Option<T> -> bool` | Returns true if None |
|  | `is_some_and(f)` | `Option<T> -> bool` | Returns true if Some and predicate holds |
| **Transformation** | `map(f)` | `Option<T> -> Option<U>` | Transform contained value |
|  | `and_then(f)` | `Option<T> -> Option<U>` | Chain fallible transformations |
|  | `inspect(f)` | `Option<T> -> Option<T>` | Observe value without consuming |
| **Borrowing** | `as_ref()` | `&Option<T> -> Option<&T>` | Borrow the contained value |
|  | `as_mut()` | `&mut Option<T> -> Option<&mut T>` | Mutably borrow the contained value |
|  | `as_deref()` | `&Option<T> -> Option<&T::Target>` | Deref and borrow (for smart pointers) |
|  | `as_deref_mut()` | `&mut Option<T> -> Option<&mut T::Target>` | Mutably deref and borrow |
|  | `as_slice()` | `&Option<T> -> &[T]` | View as slice (0 or 1 elements) |
|  | `as_mut_slice()` | `&mut Option<T> -> &mut [T]` | Mutably view as slice |
| **Materializing** | `copied()` | `Option<&T> -> Option<T>` | Copy out of reference (T: Copy) |
|  | `cloned()` | `Option<&T> -> Option<T>` | Clone out of reference (T: Clone) |
| **Boolean Logic** | `and(opt)` | `Option<T> -> Option<U>` | Returns None if None, else returns opt |
|  | `or(opt)` | `Option<T> -> Option<T>` | Returns Some if Some, else returns opt |
|  | `or_else(f)` | `Option<T> -> Option<T>` | Returns Some if Some, else computes opt |
| **Conversion** | `ok_or(err)` | `Option<T> -> Result<T, E>` | Convert to Result with static error |
|  | `ok_or_else(f)` | `Option<T> -> Result<T, E>` | Convert to Result with computed error |
|  | `transpose()` | `Option<Result<T,E>> -> Result<Option<T>,E>` | Swap nesting layers |

### Result<T, E> Methods

| Category | Method | Signature | Purpose |
| :-- | :-- | :-- | :-- |
| **Extraction** | `unwrap()` | `Result<T, E> -> T` | Get value or panic |
|  | `expect(msg)` | `Result<T, E> -> T` | Get value or panic with message |
|  | `unwrap_or(default)` | `Result<T, E> -> T` | Get value or return default |
|  | `unwrap_or_else(f)` | `Result<T, E> -> T` | Get value or compute default lazily |
|  | `unwrap_or_default()` | `Result<T, E> -> T` | Get value or use type's Default |
|  | `unwrap_unchecked()` ⚠️ | `Result<T, E> -> T` | Get value without checking (unsafe) |
| **Querying** | `is_ok()` | `&Result<T, E> -> bool` | Returns true if Ok |
|  | `is_err()` | `&Result<T, E> -> bool` | Returns true if Err |
|  | `is_ok_and(f)` | `Result<T, E> -> bool` | Returns true if Ok and predicate holds |
|  | `is_err_and(f)` | `Result<T, E> -> bool` | Returns true if Err and predicate holds |
| **Transformation** | `map(f)` | `Result<T, E> -> Result<U, E>` | Transform success value |
|  | `map_err(f)` | `Result<T, E> -> Result<T, F>` | Transform error value |
|  | `and_then(f)` | `Result<T, E> -> Result<U, E>` | Chain fallible operations |
|  | `flatten()` | `Result<Result<T, E>, E> -> Result<T, E>` | Collapse nested Results (stable 1.89.0) |
|  | `inspect(f)` | `Result<T, E> -> Result<T, E>` | Observe success value |
|  | `inspect_err(f)` | `Result<T, E> -> Result<T, E>` | Observe error value |
| **Borrowing** | `as_ref()` | `&Result<T, E> -> Result<&T, &E>` | Borrow both Ok and Err values |
|  | `as_mut()` | `&mut Result<T, E> -> Result<&mut T, &mut E>` | Mutably borrow both values |
|  | `as_deref()` | `&Result<T, E> -> Result<&T::Target, &E>` | Deref Ok value and borrow |
|  | `as_deref_mut()` | `&mut Result<T, E> -> Result<&mut T::Target, &mut E>` | Mutably deref Ok value |
| **Materializing** | `copied()` | `Result<&T, E> -> Result<T, E>` | Copy Ok value (T: Copy) |
|  | `cloned()` | `Result<&T, E> -> Result<T, E>` | Clone Ok value (T: Clone) |
| **Boolean Logic** | `and(res)` | `Result<T, E> -> Result<U, E>` | Returns Err if Err, else returns res |
|  | `or(res)` | `Result<T, E> -> Result<T, F>` | Returns Ok if Ok, else returns res |
|  | `or_else(f)` | `Result<T, E> -> Result<T, F>` | Returns Ok if Ok, else computes res |
| **Conversion** | `ok()` | `Result<T, E> -> Option<T>` | Convert to Option (discard error) |
|  | `err()` | `Result<T, E> -> Option<E>` | Extract error as Option |
|  | `transpose()` | `Result<Option<T>, E> -> Option<Result<T, E>>` | Swap nesting layers |

**⚠️ Safety Note**: `unwrap_unchecked()` produces undefined behavior if called on `None` or `Err`. Only use in performance-critical code where you can guarantee the value is `Some`/`Ok`.



### Pattern Matching Syntax

```rust

// Full match
match result {
    Ok(value) => { /* handle success */ },
    Err(error) => { /* handle error */ },
}

// If let (single case)
if let Ok(value) = result {
    // use value
}

// Let...else (handle one case, exit for others)
let Ok(value) = result else {
    // handle error and return/break/continue
    return;
};
// use value here

```

***

## Summary

Rust's `Option` and `Result` types eliminate entire categories of bugs by forcing explicit handling of absent values and errors. The type system ensures you can never accidentally use a value that might not exist or ignore an error that might occur.

**Key takeaways:**

- Pattern matching with `match` is the foundation—it guarantees exhaustive handling
- `Some(T)` wraps values that exist; `None` represents absence without error
- `Ok(T)` wraps successful results; `Err(E)` wraps errors with meaningful context
- `Option<T>` represents optional values; use when absence isn't an error
- `Result<T, E>` represents fallible operations; use when you need error information
- The `?` operator provides concise error propagation with automatic type conversion via the `From` trait
- Use `as_ref()`, `as_deref()`, and related methods to work with borrowed data efficiently
- Choose methods based on your needs: safe defaults, transformations, or explicit handling
- Use `inspect()` methods for debugging and logging—they observe values via references and return the container unchanged for method chaining

This approach trades a bit of verbosity for complete elimination of null pointer exceptions and unhandled errors—a trade-off that makes Rust's reputation for reliability well-deserved.

***## Additional material

#### Predicate Methods

These methods provide concise ways to test conditions on contained values without explicitly pattern matching.

#### Converting to Slices: `as_slice` and `as_mut_slice`

The `as_slice` and `as_mut_slice` methods convert an `Option<T>` into a slice with zero or one element, simplifying iteration and interop with slice-based APIs:

```rust
let some_value: Option<i32> = Some(42);
let slice: &[i32] = some_value.as_slice();
assert_eq!(slice, &[42]);

let none_value: Option<i32> = None;
let empty: &[i32] = none_value.as_slice();
assert_eq!(empty, &[]);
```

**Mutable variant:**

```rust
let mut opt = Some(5);
if let [value] = opt.as_mut_slice() {
    *value += 10;
}
assert_eq!(opt, Some(15));
```

**Real-world use case - Avoiding special-case logic:**

```rust
// Before: handling Option and slice separately
fn process_numbers(numbers: &[i32], extra: Option<i32>) {
    for n in numbers {
        println!("{}", n);
    }
    if let Some(e) = extra {
        println!("{}", e);
    }
}

// After: uniform slice handling
fn process_numbers(numbers: &[i32], extra: Option<i32>) {
    for n in numbers.iter().chain(extra.as_slice()) {
        println!("{}", n);
    }
}
```

**Why this matters:**

These methods allow functions that accept slices to work seamlessly with optional values, eliminating the need for separate `Option` and slice parameters or manual conversion logic.

#### Flattening Nested Results: `flatten`

When you have a `Result<Result<T, E>, E>` (a nested Result with the same error type), the `flatten` method collapses it into a single `Result<T, E>`:

```rust
let nested_ok: Result<Result<i32, &str>, &str> = Ok(Ok(42));
let flat: Result<i32, &str> = nested_ok.flatten();
assert_eq!(flat, Ok(42));

let nested_inner_err: Result<Result<i32, &str>, &str> = Ok(Err("inner error"));
let flat2 = nested_inner_err.flatten();
assert_eq!(flat2, Err("inner error"));

let nested_outer_err: Result<Result<i32, &str>, &str> = Err("outer error");
let flat3 = nested_outer_err.flatten();
assert_eq!(flat3, Err("outer error"));
```

**Real-world example - Chained validation:**

```rust
fn parse_and_validate(input: &str) -> Result<i32, String> {
    input
        .parse::<i32>()
        .map_err(|e| format!("Parse error: {}", e))
        .map(|n| {
            if n > 0 && n < 100 {
                Ok(n)
            } else {
                Err(String::from("Number out of range"))
            }
        })
        .flatten()  // Result<Result<i32, String>, String> → Result<i32, String>
}
```

**Alternative for code that needs to work before 1.89:**

`and_then` can achieve the same result on older Rust versions:

```rust
// Equivalent to flatten() for backwards compatibility
let nested: Result<Result<i32, &str>, &str> = Ok(Ok(42));
let flat: Result<i32, &str> = nested.and_then(|inner| inner);
```

**Why this matters:**

`flatten` complements `transpose` for nested type manipulation. While `transpose` swaps layers between `Option` and `Result`, `flatten` removes one layer of nesting when both layers are `Result` with the same error type. This is common in validation pipelines where each step can fail with the same error type.

**Note:** `Result::flatten` requires both the inner and outer error types to be the same (`E`). If they differ, use `and_then` with explicit error conversion instead.

