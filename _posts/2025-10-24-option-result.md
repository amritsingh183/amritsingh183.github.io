---
layout: post
title: "Mastering Rust's Option and Result Types: A Complete Guide"
date: 2025-10-24 13:23:00 +0530
categories: rust programming tutorial
---

# Mastering Rust's Option and Result Types: A Complete Guide

Rust eliminates entire classes of bugs found in other languages—null pointer exceptions, uncaught exceptions, and ignored errors—through its type system. This guide explores how Rust achieves this through two fundamental enum types: `Option` and `Result`.

## Prerequisites

This guide assumes you understand Rust's ownership, borrowing, and type safety fundamentals. We'll build from pattern matching through increasingly sophisticated error handling patterns.

**References and pointers in Rust refer to related but distinct concepts**

- References (\&T or \&mut T) are safe pointers with strict rules enforced by the compiler.
    - References have extra semantic rules and safety enforced by the compiler, making them safe to use for borrowing data.
    - They always point to valid memory.
    - They have associated lifetimes ensuring they do not outlive the data they reference.
    - Mutable references enforce exclusive access (no aliasing).
    - You cannot perform pointer arithmetic or raw memory access with references.
    - References behave as aliases to the underlying data with compiler guarantees for safety.
- Raw pointers (*const T and *mut T) are unsafe pointers akin to C pointers.
    - Pointers are just addresses without safety or lifetime guarantees, requiring explicit unsafe code to dereference.
    - They are simple memory addresses without lifetimes or borrowing rules.
    - They can be null, dangling, or invalid.
    - You can perform arithmetic on raw pointers.
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
if let Message::Write(text) = msg {
    println!("Got text: {}", text);
}
// Other variants are ignored
```

**The `let...else` pattern** handles one case and exits for others:

```rust
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
    Some(T),  // Contains a value of type T
    None,     // Represents absence of a value
}
```

This replaces the concept of "null" found in other languages, but **with type safety**. You cannot use an `Option<T>` as if it were a `T`—you must explicitly handle both cases.

### Understanding Some() and None

**Some(value)** is a variant constructor that **wraps a value** inside an Option. When a function returns `Some(5)`, it's saying "I have a value, and that value is 5."

**None** represents the absence of a value—it's Rust's type-safe replacement for null. Unlike null in other languages, you cannot accidentally use `None` as if it were a value; the compiler forces you to handle it explicitly.

```rust
let some_number: Option<i32> = Some(42);  // Wraps the value 42
let no_number: Option<i32> = None;        // No value present

// Option is so common it's in the prelude
// You can omit the Option:: prefix
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

**Use case:** Prototyping or when you're absolutely certain the value exists.

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

The `and_then` method is used when your transformation **can itself fail**. The function must return an `Option` or `Result`.

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

The `inspect()` methods allow you to perform side effects (like logging) while temporarily borrowing the value. The value is then passed through unchanged, enabling method chaining:

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

**Use case:** Debugging, logging, or metrics collection without modifying the data flow. The `inspect()` methods borrow the value temporarily for the closure but return the original `Option` or `Result` unchanged.

### Boolean Combinators

The `and` and `or` methods provide boolean-like logic for combining `Option` and `Result` values.

#### The and Method

Returns the second value if the first is `Ok`/`Some`, otherwise returns the first:


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


#### The or Method

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

***

## The Question Mark Operator: Ergonomic Error Propagation

The `?` operator is Rust's most powerful tool for error handling. It provides concise error propagation without sacrificing type safety.

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
    let a = list1.last()?;  // Returns None if list is empty
    let b = list2.last()?;  // Returns None if list is empty
    Some(a + b)
}
```


### Chaining Operations with ?

The `?` operator makes complex error handling readable:

```rust
use std::fs;
use std::io;

fn process_config(path: &str) -> Result<Config, io::Error> {
    let content = fs::read_to_string(path)?;
    let parsed = parse_toml(&content)?;
    let validated = validate_config(parsed)?;
    Ok(validated)
}
```


### Constraints on Using ?

The `?` operator can only be used in functions that return `Result` or `Option`:

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

***

## Quick Reference

### Option<T> Methods

| Method | Signature | Purpose |
| :-- | :-- | :-- |
| `unwrap()` | `Option<T> -> T` | Get value or panic |
| `expect(msg)` | `Option<T> -> T` | Get value or panic with message |
| `unwrap_or(default)` | `Option<T> -> T` | Get value or return default |
| `unwrap_or_else(f)` | `Option<T> -> T` | Get value or compute default |
| `unwrap_or_default()` | `Option<T> -> T` | Get value or type's default |
| `map(f)` | `Option<T> -> Option<U>` | Transform contained value |
| `and_then(f)` | `Option<T> -> Option<U>` | Chain fallible transformations |
| `inspect(f)` | `Option<T> -> Option<T>` | Observe value without consuming |
| `ok_or(err)` | `Option<T> -> Result<T, E>` | Convert to Result |
| `ok_or_else(f)` | `Option<T> -> Result<T, E>` | Convert to Result (lazy error) |

### Result<T, E> Methods

| Method | Signature | Purpose |
| :-- | :-- | :-- |
| `unwrap()` | `Result<T, E> -> T` | Get value or panic |
| `expect(msg)` | `Result<T, E> -> T` | Get value or panic with message |
| `unwrap_or(default)` | `Result<T, E> -> T` | Get value or return default |
| `unwrap_or_else(f)` | `Result<T, E> -> T` | Get value or compute default |
| `unwrap_or_default()` | `Result<T, E> -> T` | Get value or type's default |
| `unwrap_unchecked()` ⚠️ | `Result<T, E> -> T` | **UNSAFE**: Get value without checking (undefined behavior if Err) |
| `map(f)` | `Result<T, E> -> Result<U, E>` | Transform success value |
| `map_err(f)` | `Result<T, E> -> Result<T, F>` | Transform error value |
| `and_then(f)` | `Result<T, E> -> Result<U, E>` | Chain fallible operations |
| `inspect(f)` | `Result<T, E> -> Result<T, E>` | Observe success value |
| `inspect_err(f)` | `Result<T, E> -> Result<T, E>` | Observe error value |
| `ok()` | `Result<T, E> -> Option<T>` | Convert to Option (discard error) |
| `err()` | `Result<T, E> -> Option<E>` | Extract error as Option |

⚠️ **Safety Note**: `unwrap_unchecked()` is an unsafe operation that produces undefined behavior if called on an `Err` variant. Only use in performance-critical code where you can guarantee the Result is Ok.

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
- Choose methods based on your needs: safe defaults, transformations, or explicit handling
- Use `inspect()` methods for debugging and logging—they borrow values temporarily and return them unchanged for method chaining
- Follow Rust naming conventions: use snake_case for functions and variables

This approach trades a bit of verbosity for complete elimination of null pointer exceptions and unhandled errors—a trade-off that makes Rust's reputation for reliability well-deserved.

***

- https://users.rust-lang.org/t/rust-generic-return-type/79816

- https://stackoverflow.com/questions/59097840/understanding-rusts-function-return-type

- https://rust-lang.github.io/rfcs/3654-return-type-notation.html

- https://doc.rust-lang.org/std/keyword.return.html

- https://doc.rust-lang.org/std/result/

- https://loige.co/rust-shenanigans-return-type-polymorphism/

- https://www.reddit.com/r/rust/comments/wy9sk1/can_you_have_a_function_return_different_types/

- https://www.ncameron.org/blog/abstract-return-types-aka--impl-trait-/

- https://doc.rust-lang.org/stable/releases.html

- https://doc.rust-lang.org/rust-by-example/fn.html

- https://doc.rust-lang.org/std/convert/trait.From.html

