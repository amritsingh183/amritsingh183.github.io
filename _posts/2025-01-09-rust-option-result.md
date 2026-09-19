---
layout: post
title: "Mastering Rust's `Option` and `Result` Types: A Complete Guide"
date: 2025-01-09 13:23:00 +0530
categories: rust concepts
last_updated: 2026-09-19
---
# Mastering Rust's Option and Result Types: A Complete Guide

A customer walks into our café. They might give us a name for the cup, or they might not. Either is fine. Then they type how many coffees they want. If they type `two` into a box that expects a number, we need to explain what went wrong.

Those are two different questions:

- **Is there a value?** `Option<T>` gives us `Some(value)` or `None`.
- **Did the operation succeed?** `Result<T, E>` gives us `Ok(value)` or `Err(error)`.

Rust makes these alternatives visible in the type. You cannot pass an `Option<String>` to a function that expects a `String` and hope that a name happens to be present. You must make a decision about the missing case first. That decision may be to supply a default, return early, report an error, or deliberately panic.

This does not mean Rust prevents every ignored error or every crash. A `Result` can be discarded, and a function returning `Result` can still panic. The benefit is that the alternatives are explicit and ordinary Rust code cannot directly use the missing payload as though it existed.

**Checked with Rust 1.98.1, edition 2024, in September 2026.** Every `rust` code block below is a complete, independent program. Blocks marked **Does not compile** deliberately demonstrate a compiler error. The file-reading example takes a path on the command line; its instructions explain how to run it. Diagrams of types and abbreviated signatures use `text` blocks.

## Prerequisites

This is the third step in the series. The [first post](/rust/concepts/2025/01/01/rust-var-const-lifetimes.html) explains ownership, borrowing, cleanup, and closures. The [second post](/rust/concepts/2025/01/05/rust-mem-ref.html) explains how values and references are represented in memory. Here we apply those ideas to missing values and failed operations.

Keep one distinction from those posts in mind: owning a `String`, borrowing it through `&String`, and borrowing its text through `&str` are different. Wrapping any of them in `Option` does not erase that distinction.

An `Option` is not automatically a heap allocation. It stores its active variant and payload according to its layout; the payload may itself own separate storage. The memory post explains when compact representations are guaranteed. We do not need raw pointers or `unsafe` to use any of the techniques taught here.

Read this guide in order the first time. Later, use the [quick reference](#quick-reference). The recurring questions are: **which states are possible, who owns the value, what runs, and what information survives?**

The route through the guide:

1. [Enums and matching](#guide-matching)
2. [Optional values](#guide-option) and [successful or failed operations](#guide-result)
3. [Borrowing through wrappers](#guide-borrowing)
4. [Methods and when they run](#guide-methods)
5. [Changing an optional slot](#guide-mutation)
6. [Conversions and nested states](#guide-conversions)
7. [Propagation with `?`](#guide-propagation) and [program exit](#guide-termination)
8. [The complete café example](#guide-cafe)
9. [Slices and iterators](#guide-iterators)
10. [Decision guide](#guide-decisions), [reference](#quick-reference), and [practice](#guide-practice)

<a id="guide-matching"></a>

## Enums and Pattern Matching: The Foundation

### Rust Enums

An enum is a choice between named alternatives. Each alternative is called a **variant**, and it can carry its own data. Our till can receive different kinds of messages:

```rust
enum Message {
    CloseTill,
    SetTable { number: u32 },
    WriteName(String),
}

fn main() {
    let messages = [
        Message::CloseTill,
        Message::SetTable { number: 4 },
        Message::WriteName(String::from("Amrit")),
    ];

    for message in messages {
        match message {
            Message::CloseTill => println!("Close the till"),
            Message::SetTable { number } => println!("Table {number}"),
            Message::WriteName(name) => println!("Write {name} on the cup"),
        }
    }
}
```

`WriteName(String::from("Amrit"))` constructs one variant. The `match` finds that variant and gives its contents a name. Here the `String` moves into the matching arm. An enum does not carry the payloads of all its variants at once.

### Pattern Matching with match

A `match` must cover every possible value. This is **exhaustiveness**: Rust checks that no input can fall through the match. A catch-all `_` also covers values, so exhaustiveness alone does not guarantee useful handling. `Err(_) => {}` is exhaustive handling of an error variant that deliberately does nothing.

Patterns can also borrow. We will use `match &value` when we want to look without taking the owned contents. The same ownership rules apply inside a pattern as elsewhere in Rust.

### Advanced Pattern Matching

A **guard** adds a condition after a pattern. The unguarded `Some(n)` below is still needed: the guarded arm only handles some possible quantities.

```rust
fn main() {
    let quantity = Some(12_u32);

    match quantity {
        Some(n) if n > 10 => println!("Large order: {n} coffees"),
        Some(n) => println!("Order: {n} coffees"),
        None => println!("No quantity entered"),
    }
}
```

Use `if let` when only one case needs work. It is a pattern check; it does not report the other case for you.

```rust
fn main() {
    let name = Some(String::from("Amrit"));

    if let Some(text) = &name {
        println!("Name on cup: {text}");
    }

    println!("We still own the name: {name:?}");
}
```

Because we matched `&name`, `text` is `&String`. This automatic borrowing in patterns is often called **match ergonomics**. Matching `name` by value and binding its `String` instead would move that string.

Use `let ... else` when the rest of a function only makes sense for one case:

```rust
fn greet(name: Option<&str>) {
    let Some(name) = name else {
        println!("Hello, customer!");
        return;
    };

    println!("Hello, {name}!");
}

fn main() {
    greet(Some("Amrit"));
    greet(None);
}
```

The `else` must **diverge**: it cannot continue to the next statement. `return`, a suitable `break` or `continue`, or a panic can do that. Otherwise Rust could reach `println!` without a `name` to use. [Pattern rules](https://doc.rust-lang.org/reference/patterns.html) and [`let ... else`](https://doc.rust-lang.org/reference/statements.html#let-statements) specify these details.

<a id="guide-option"></a>

## The Option Type: Handling Optional Values

### Understanding Option, Some, and None

The standard library's definition has this shape:

```text
enum Option<T> {
    None,
    Some(T),
}
```

`T` stands for the contained type. `Option<String>` may own a name; `Option<u32>` may contain a quantity. `Some` is a variant constructor, not a special storage operation. `None` is the variant with no payload. `Option`, `Some`, and `None` are available through Rust's prelude, so ordinary code needs no import for them.

### Creating Option Values

```rust
fn main() {
    let name = Some(String::from("Amrit"));
    let no_name: Option<String> = None;

    // Later context can determine the type of an earlier None.
    let missing = None;
    let quantity: Option<u32> = missing;

    println!("{name:?}, {no_name:?}, {quantity:?}");
}
```

`None` needs enough surrounding information to determine `T`, not necessarily an annotation on that exact line. A function's parameter or return type can provide that information too. The same issue appears with `Ok`, which alone does not tell Rust the error type, and `Err`, which alone does not tell Rust the success type.

### Real-World Example: Finding Elements

The café receives possible table numbers. Its table numbers start at one, so we look for the first positive number. The list might not contain one:

```rust
fn first_table(tables: &[u32]) -> Option<u32> {
    for &table in tables {
        if table > 0 {
            return Some(table);
        }
    }
    None
}

fn main() {
    match first_table(&[0, 0, 4, 7]) {
        Some(table) => println!("Use table {table}"),
        None => println!("No table available"),
    }
}
```

The `u32` is copied out of the borrowed slice. If this were a collection of owned records, returning `Option<&Record>` could let the caller borrow a found record without cloning it.

`None` has no built-in reason attached. It could mean no customer name, no matching table, or failed checked arithmetic. The function's contract tells you which meaning applies. Use `Result` when the caller needs an error value to distinguish failure reasons. [The Option overview](https://doc.rust-lang.org/std/option/index.html) includes both optional data and simple failures among its uses.

<a id="guide-result"></a>

## The Result Type: Handling Success and Failure

### Understanding Result, Ok, and Err

The corresponding shape for a fallible operation is:

```text
enum Result<T, E> {
    Ok(T),
    Err(E),
}
```

`T` describes success; `E` describes failure. An error is an ordinary value. `E` does not have to implement `std::error::Error`; that trait adds useful conventions for error reporting, which we will meet later. Even `Result<T, ()>` is valid when the caller only needs a failure flag.

### Real-World Example: Input Validation

The customer must order between one and ten coffees. This first example uses text for errors so we can focus on the branching:

```rust
fn read_quantity(input: &str) -> Result<u32, &'static str> {
    let quantity = match input.trim().parse::<u32>() {
        Ok(n) => n,
        Err(_) => return Err("Enter a whole number"),
    };

    if !(1..=10).contains(&quantity) {
        return Err("Order between 1 and 10 coffees");
    }

    Ok(quantity)
}

fn main() {
    for input in ["2", "two", "0"] {
        match read_quantity(input) {
            Ok(n) => println!("Prepare {n} coffees"),
            Err(message) => println!("Cannot place order: {message}"),
        }
    }
}
```

The output is:

```text
Prepare 2 coffees
Cannot place order: Enter a whole number
Cannot place order: Order between 1 and 10 coffees
```

Here `Err(_)` intentionally replaces the parser's detailed error with a simpler message. Later we will preserve the original error instead. `Ok(())` is also common: it means an operation succeeded but has no useful success payload, as when saving a receipt.

### Handling a result is a decision

`if let Ok(value) = result` runs its body only on success and silently skips failure. That is fine when intentional. When the error matters, use a `match`, transform it, or propagate it.

Rust warns about an unused `Result` by default; the warning is not a proof that every error gets handled:

```rust
fn save_receipt() -> Result<(), &'static str> {
    Err("Printer is disconnected")
}

fn main() {
    // An intentionally simulated failure; there is no real printer here.
    let _ = save_receipt(); // Explicitly discards the error; compiles.
}
```

Writing just `save_receipt();` instead produces an `unused_must_use` warning by default. A project can make that lint an error, but explicit discarding is still possible. [Compiler lint documentation](https://doc.rust-lang.org/rustc/lints/listing/warn-by-default.html#unused-must-use)

<a id="guide-borrowing"></a>

## Working with Borrowed Data: as_ref, as_deref, and Friends

Before learning many methods, check their receiver: `self` means by value, `&self` means a shared borrow, and `&mut self` means a mutable borrow. A by-value call moves a non-`Copy` receiver; it can copy a `Copy` receiver.

`Option<T>` is `Copy` when `T` is `Copy`. `Result<T, E>` is `Copy` when both `T` and `E` are `Copy`. The current variant does not change those requirements. An `Ok(5)` stored in `Result<i32, String>` is not `Copy` just because it currently holds no string.

### Borrowing Without Moving: as_ref and as_mut

`as_ref` borrows the wrapper and makes a new wrapper containing references. It does not clone the payload:

```rust
fn main() {
    let name = Some(String::from("Amrit"));
    let borrowed: Option<&String> = name.as_ref();

    if let Some(text) = borrowed {
        println!("Name has {} bytes", text.len());
    }
    println!("Original still owns the text: {name:?}");

    let mut note = Some(String::from("oat milk"));
    if let Some(text) = note.as_mut() {
        text.push_str(", no sugar");
    }
    println!("{note:?}");
}
```

`as_mut` similarly produces an optional mutable reference. Its borrow must end before you use the original in a conflicting way; here it ends before the final print.

These similar-looking types grant different access:

| Type | What it gives you |
| :-- | :-- |
| `Option<T>` | An optional owned `T` |
| `&Option<T>` | Shared access to an existing optional slot |
| `Option<&T>` | An optional shared reference to a `T` |
| `&mut Option<T>` | Access that can replace the slot, including changing `Some` to `None` |
| `Option<&mut T>` | Optional exclusive access to a `T`; it does not by itself give access to the original slot |

For `Result`, `as_ref` borrows whichever variant is present:

```rust
fn main() {
    let result: Result<String, String> = Err(String::from("Printer offline"));
    let borrowed: Result<&String, &String> = result.as_ref();

    match borrowed {
        Ok(receipt) => println!("Receipt: {receipt}"),
        Err(error) => println!("Problem: {error}"),
    }
    println!("Original still available: {result:?}");
}
```

### Working with Smart Pointers: as_deref and as_deref_mut

Sometimes you want the thing a type points to. `String` dereferences to `str`, `Vec<T>` to `[T]`, and `Box<T>` to `T`. `as_deref` combines borrowing with that dereference:

```rust
fn main() {
    let name = Some(String::from("Amrit"));
    let text: Option<&str> = name.as_deref();
    println!("{text:?}");

    let quantities = Some(vec![1, 2, 3]);
    let slice: Option<&[i32]> = quantities.as_deref();
    println!("{slice:?}");

    let mut result: Result<String, String> = Ok(String::from("paid"));
    if let Ok(text) = result.as_deref_mut() {
        text.make_ascii_uppercase();
    }
    println!("{result:?}"); // Ok("PAID")
}
```

`Result::as_deref` dereferences the success payload and borrows the error payload as-is: `Result<String, E>` becomes `Result<&str, &E>`. It does not dereference both sides. These examples borrow existing storage; they do not allocate new text or a new vector.

An optional mutable reference can be borrowed again too:

```rust
fn main() {
    let mut name = String::from("Amrit");
    let mut optional_ref = Some(&mut name);

    if let Some(text) = optional_ref.as_deref_mut() {
        text.push('!');
    }
    if let Some(text) = optional_ref.as_deref_mut() {
        text.push('?');
    }
    println!("{name}"); // Amrit!?
}
```

Passing `optional_ref` by value to a consuming operation would move it: `&mut String` is not `Copy`. Calling `as_deref_mut` instead makes a shorter reborrow, allowing the next call after the first borrow ends. This is the first post's reborrowing rule applied through an `Option`.

### Materializing Values: copied and cloned

Use `copied` to turn a borrowed `Copy` payload into a value; use `cloned` when you intentionally want a clone:

```rust
fn main() {
    let quantity = 2_u32;
    let borrowed = Some(&quantity);
    let owned: Option<u32> = borrowed.copied();
    println!("{owned:?}");

    let name = String::from("Amrit");
    let borrowed = Some(&name);
    let duplicate: Option<String> = borrowed.cloned();
    println!("Original: {name}, duplicate: {duplicate:?}");

    let result: Result<&u32, &str> = Ok(&quantity);
    let owned_result: Result<u32, &str> = result.copied();
    println!("{owned_result:?}");
}
```

`cloned` is still cloning. For this nonempty `String`, it creates an owned copy of the text. If you only need to read, keep the reference. On a `Result`, these methods copy or clone the success payload; they leave the error type unchanged. The [Option](https://doc.rust-lang.org/std/option/enum.Option.html) and [Result](https://doc.rust-lang.org/std/result/enum.Result.html) signatures show the borrowing and trait requirements.

<a id="guide-methods"></a>

## Essential Methods: Working with Option and Result

### Unwrapping Methods

#### unwrap() and expect(): make a promise about success

Both extract the payload and panic if it is missing or the operation failed. `expect` lets you explain why success should be guaranteed here:

```rust
fn main() {
    let quantity = "2"
        .parse::<u32>()
        .expect("the hardcoded quantity should be a valid u32");
    println!("{quantity}");
}
```

The reason is the fixed input, not merely that the program needs a quantity. Input typed by a customer needs ordinary error handling. A missing required configuration file can also be an ordinary startup error: report it and exit unsuccessfully rather than assuming it indicates a programming bug.

`unwrap` and `expect` are useful for demonstrated invariants, prototypes, and examples. They are not universally forbidden in production. Panic can unwind the current thread or abort the process, depending on the panic strategy; some unwinding panics can be caught. Returning `Result` is the normal way to let a caller decide what to do about expected failures. [Panic guidance](https://doc.rust-lang.org/book/ch09-03-to-panic-or-not-to-panic.html) and [unwinding limits](https://doc.rust-lang.org/std/panic/fn.catch_unwind.html)

For `Result<T, E>`, `unwrap` and `expect` require `E: Debug` so an error can be included in the panic message. Their `Option` equivalents do not have that error-type requirement.

#### unwrap_or() - Provide a Default Value

If no customer name was entered, a greeting can use a default:

```rust
fn main() {
    let name: Option<&str> = None;
    println!("Hello, {}!", name.unwrap_or("customer"));

    let quantity: Result<u32, &str> = Err("Invalid quantity");
    println!("{}", quantity.unwrap_or(0)); // 0; the error was discarded.
}
```

The second line of behavior is intentional but not automatically a good order policy. A default can hide a real mistake. Decide whether the missing or failed case is allowed before choosing the method.

#### unwrap_or_else() - Compute Default Lazily

Rust evaluates ordinary function arguments before calling the function. So `unwrap_or(make_default())` calls `make_default` even when the value is already present:

```rust
fn make_default() -> String {
    println!("Preparing a default label");
    String::from("customer")
}

fn main() {
    let first = Some(String::from("Amrit")).unwrap_or(make_default());
    println!("First: {first}");

    let second = Some(String::from("Amrit")).unwrap_or_else(make_default);
    println!("Second: {second}");
}
```

Output:

```text
Preparing a default label
First: Amrit
Second: Amrit
```

The second call passes the function itself, not the result of calling it. `unwrap_or_else` only calls it for `None`. With `Result`, the callback receives the error:

```rust
fn main() {
    let saved_label: Result<String, &str> = Err("No saved greeting");
    let label = saved_label.unwrap_or_else(|error| {
        eprintln!("Using a generic greeting: {error}");
        String::from("customer")
    });
    println!("{label}");
}
```

The café has explicitly allowed a generic greeting here. A payment failure would need a different policy.

#### A lazy callback can still move a value now

Creating a closure and calling it are separate events. Its body can move a captured value, so Rust moves that value into the closure when the closure is created—even if the method never calls it.

**Does not compile: E0382, use after move.**

```rust
fn main() {
    let fallback = String::from("customer");
    let name = Some(String::from("Amrit")).unwrap_or_else(|| fallback);

    println!("{name}");
    println!("{fallback}"); // The closure took ownership of fallback.
}
```

When the value is already `Some`, the unused closure is dropped, including the fallback it owns. Laziness means its body is not called; it does not mean no capture, borrow, or cleanup occurs. [Closure capture rules](https://doc.rust-lang.org/reference/types/closure.html#capture-modes)

#### unwrap_or_default() - Use Type's Default

```rust
fn main() {
    let note: Option<String> = None;
    let quantity: Option<u32> = None;

    println!("Note: {:?}", note.unwrap_or_default()); // ""
    println!("Quantity: {}", quantity.unwrap_or_default()); // 0
}
```

This requires `T: Default`. A type's default is not necessarily your business's default: zero coffees is a valid `u32`, but an invalid order in our café.

### Predicate Methods: asking a yes-or-no question

`is_some` and `is_none` borrow the wrapper and check its variant. `is_some_and` and `is_none_or` take it by value and, when present, pass the payload by value to a predicate:

```rust
fn main() {
    let name = Some(String::from("Amrit"));
    let has_name = name.as_deref().is_some_and(|s| !s.trim().is_empty());
    println!("Has name: {has_name}; original: {name:?}");

    let optional_quantity: Option<u32> = None;
    let acceptable = optional_quantity.is_none_or(|n| (1..=10).contains(&n));
    println!("Absent or valid: {acceptable}"); // true

    let result: Result<u32, &str> = Ok(2);
    println!("Valid order: {}", result.is_ok_and(|n| (1..=10).contains(&n)));
    println!("Failed: {}", result.is_err_and(|e| e == "Printer offline"));
}
```

`is_none_or` accepts absence; `is_some_and` rejects it. Choose the one that matches your requirement. The numeric wrappers above are `Copy`. For an owned `String`, first borrow with `as_ref` or `as_deref` if you want to keep it. `Result` also has borrowing `is_ok` and `is_err` queries.

### Transformation Methods

#### map() - Transform the Success Value

`map` takes the present or successful value, calls your function, and wraps what it returns. It leaves `None` or `Err` on that path without calling the function.

```rust
fn main() {
    let name = Some(String::from("Amrit"));
    let length = name.as_ref().map(|s| s.len());
    println!("Bytes: {length:?}; name: {name:?}");

    let price: Result<u32, &str> = Ok(250);
    let label = price.map(|cents| format!("{cents} cents"));
    println!("{label:?}");
}
```

The callback can return any suitable type `U`, including another `Option` or `Result`. `map` does not automatically interpret or remove that extra wrapper. It also does not promise that the callback cannot panic.

#### and_then() - Chain Fallible Operations

Suppose parsing a quantity can fail, and checking its permitted range can also fail. If we only need an accepted quantity or nothing, each step can return `Option<u32>`:

```rust
fn allowed_quantity(n: u32) -> Option<u32> {
    if (1..=10).contains(&n) { Some(n) } else { None }
}

fn main() {
    let nested = Some(2).map(allowed_quantity);
    let flat = Some(2).and_then(allowed_quantity);
    println!("{nested:?}"); // Some(Some(2))
    println!("{flat:?}");   // Some(2)

    let quantity = "12".parse::<u32>().ok().and_then(allowed_quantity);
    println!("{quantity:?}"); // None; rejection reason was discarded.
}
```

Read the signatures as a description of what the callback must provide:

```text
Option<T>.map(       T -> U)             -> Option<U>
Option<T>.and_then(  T -> Option<U>)     -> Option<U>
Result<T, E>.map(      T -> U)              -> Result<U, E>
Result<T, E>.and_then( T -> Result<U, E>)  -> Result<U, E>
```

Here `T -> U` means a callback that takes a `T` and returns a `U`, not actual Rust syntax. The callback bounds are `FnOnce`: these methods need to call the callback at most once. On `Result`, `and_then` keeps the same error type `E`; convert incompatible errors explicitly.

#### The wrapper does not check arithmetic for you

Parsing an integer can succeed even when squaring it would overflow. With overflow checks enabled, `n * n` can panic; with them disabled, integer multiplication wraps. Putting that expression inside `Some(...)` or `map(...)` does not change it.

```rust
fn parse_and_square(input: &str) -> Option<i32> {
    input.parse::<i32>().ok().and_then(|n| n.checked_mul(n))
}

fn main() {
    println!("{:?}", parse_and_square("5"));     // Some(25)
    println!("{:?}", parse_and_square("five"));  // None
    println!("{:?}", parse_and_square("50000")); // None: square will not fit.
}
```

`checked_mul` makes overflow an explicit missing result. We intentionally merge parsing failure and overflow into `None`; use separate error variants if the caller needs to tell them apart. [Integer overflow rules](https://doc.rust-lang.org/reference/expressions/operator-expr.html#overflow) and [checked multiplication](https://doc.rust-lang.org/std/primitive.i32.html#method.checked_mul)

#### map_err() - Transform the Error

`map_err` operates on the other side of a `Result`: it changes `E` to another error type while leaving the success type alone.

```rust
fn main() {
    let quantity = "two".parse::<u32>().map_err(|error| {
        format!("Could not read the coffee quantity: {error}")
    });
    println!("{quantity:?}");
}
```

This produces a useful message but turns the parser's structured error into a `String`. If callers need the original error, store it in a custom error variant instead. We will do that below.

#### inspect() - Observe and return the wrapper

`inspect` takes the wrapper by value, lends the success payload to its callback, and returns the wrapper. `inspect_err` does the same for an error payload:

```rust
fn main() {
    let name = Some(String::from("Amrit"));
    let name = name.inspect(|text| println!("Preparing a cup for {text}"));
    println!("Still owned, through the returned wrapper: {name:?}");

    let quantity = "two"
        .parse::<u32>()
        .inspect(|n| println!("Quantity: {n}"))
        .inspect_err(|e| eprintln!("Quantity was rejected: {e}"));
    println!("Result is still available: {quantity:?}");
}
```

The first `name` moves into the method and the returned wrapper is bound to the new `name`. Nothing clones the string. Ignoring that return would not let you use the original non-`Copy` wrapper again. A `Copy` wrapper such as `Option<u32>` can instead be copied into the call.

The callbacks receive shared references. They can still log, panic, or use interior-mutability APIs: observing the payload does not promise that nothing in the program changes.

#### map_or, map_or_else, and map_or_default

Sometimes you want a final plain value, not another wrapper:

```rust
fn main() {
    let name: Option<&str> = Some("Amrit");
    let bytes = name.map_or(0, str::len);
    println!("{bytes}"); // 5

    let greeting = name.map_or_else(
        || String::from("Hello, customer!"),
        |name| format!("Hello, {name}!"),
    );
    println!("{greeting}");

    let missing: Option<&str> = None;
    println!("{}", missing.map_or_default(str::len)); // 0
}
```

`map_or` evaluates its default argument eagerly. `map_or_else` calls only the selected callback. Its failure callback receives no argument for `Option`, and receives `E` for `Result`. `map_or_default` uses the output type's `Default` value for `None` or `Err`; it is stable for both wrappers since **Rust 1.98.0**. That method's ordinary availability does not mean its `const` use is stable.

### Boolean Combinators

`and` keeps the second wrapper when the first succeeds. `or` keeps the first wrapper when it succeeds. Their result tables resemble Boolean logic, but their argument evaluation is different from short-circuiting `&&` and `||`.

| First option | Second option | `and` returns | `or` returns |
| :-- | :-- | :-- | :-- |
| `Some(a)` | `Some(b)` | `Some(b)` | `Some(a)` |
| `Some(a)` | `None` | `None` | `Some(a)` |
| `None` | `Some(b)` | `None` | `Some(b)` |
| `None` | `None` | `None` | `None` |

```rust
fn main() {
    println!("{:?}", Some(2).and(Some("receipt"))); // Some("receipt")
    println!("{:?}", None::<u32>.or(Some(2)));       // Some(2)

    let first: Result<u32, &str> = Err("Printer offline");
    let second: Result<&str, &str> = Ok("Receipt ready");
    println!("{:?}", first.and(second)); // Err("Printer offline")

    let retry: Result<u32, &str> = Err("Retry failed");
    println!("{:?}", first.or(retry)); // Err("Retry failed")
}
```

For `Result`, `and` can change the success type but keeps the error type. `or` keeps the success type but can change the error type. When both results fail, `or` returns the second error; it does not retain a history of both failures.

### Fallback chains: choose when the next source runs

Suppose a cup label comes from the current order, then a saved customer profile, then a generic greeting. Here the functions only print their names so you can see the order:

```rust
fn from_order() -> Option<&'static str> {
    println!("Checking order");
    Some("Amrit")
}

fn from_profile() -> Option<&'static str> {
    println!("Checking profile");
    Some("A. Singh")
}

fn generic_label() -> Option<&'static str> {
    println!("Preparing generic label");
    Some("customer")
}

fn main() {
    println!("Eager:");
    let eager = from_order().or(from_profile()).or(generic_label());
    println!("{eager:?}");

    println!("Lazy:");
    let lazy = from_order().or_else(from_profile).or_else(generic_label);
    println!("{lazy:?}");
}
```

The eager chain prints all three source messages. The lazy chain prints only `Checking order`. Both return `Some("Amrit")`, but they did different work. `and_then` provides conditional execution for a following success step; `or_else` provides it for an alternative after failure. For `Result::or_else`, the callback receives the first error.

The same eager/lazy distinction applies to `ok_or`/`ok_or_else`, and `get_or_insert`/`get_or_insert_with`. There is no universal rule that the longer spelling is better: an already available cheap default is often exactly what you need.

<a id="guide-mutation"></a>

## Changing an Option in place

### take: hand something over and leave the slot empty

The till owns a receipt until it hands it to the customer. After that, there is no receipt left to hand over a second time:

```rust
struct Till {
    receipt: Option<String>,
}

impl Till {
    fn hand_over_receipt(&mut self) -> Option<String> {
        self.receipt.take()
    }
}

fn main() {
    let mut till = Till { receipt: Some(String::from("2 coffees, paid")) };

    let customer_receipt = till.hand_over_receipt();
    println!("Customer has: {customer_receipt:?}");
    println!("Second attempt: {:?}", till.hand_over_receipt()); // None
}
```

`take` returns the old `Option` and leaves `None` behind. Nothing clones the receipt. This solves a common ownership problem: you cannot simply move a non-`Copy` field out through `&mut self` and leave it uninitialized. `None` is a valid replacement state.

### replace, insert, and get_or_insert

These methods answer different questions:

| Method | What happens to the slot? | What is returned? |
| :-- | :-- | :-- |
| `take()` | Becomes `None` | The old `Option<T>` |
| `replace(value)` | Becomes `Some(value)` | The old `Option<T>` |
| `insert(value)` | Becomes `Some(value)`; the previous payload is dropped | `&mut T` pointing to the new payload |
| `get_or_insert(value)` | Fills only an empty slot | `&mut T` pointing to the existing or inserted payload |
| `get_or_insert_with(f)` | Calls `f` and fills only an empty slot | `&mut T` pointing to the existing or inserted payload |
| `get_or_insert_default()` | Fills an empty slot with `T::default()` | `&mut T`; requires `T: Default` |

The `value` passed to `get_or_insert` has already been evaluated. If the slot was full, that unused value is dropped. With `replace`, the old value belongs to the returned option; the method does not destroy that old payload itself.

```rust
fn main() {
    let mut note: Option<String> = None;
    note.get_or_insert_with(|| String::from("regular milk"))
        .push_str(", no sugar");
    println!("{note:?}");

    let old_note = note.replace(String::from("oat milk"));
    println!("Old: {old_note:?}; current: {note:?}");
}
```

`take_if` is a conditional `take`. Its callback receives `&mut T`, so it can change the value even when it returns `false` and leaves the slot full:

```rust
fn main() {
    let mut remaining = Some(2_u32);
    let finished = remaining.take_if(|n| {
        *n -= 1;
        *n == 0
    });
    println!("Taken: {finished:?}; remaining: {remaining:?}");
    // Taken: None; remaining: Some(1)
}
```

This fixed example starts at two. A general counter would also need to decide what to do at zero before subtracting. [In-place Option methods](https://doc.rust-lang.org/std/option/enum.Option.html#method.take)

<a id="guide-conversions"></a>

## Converting Between Option and Result

### Option to Result: Providing Error Context

An optional name becomes required if the café needs a name for a delivery. `ok_or` supplies an error for the missing case:

```rust
fn delivery_name(name: Option<&str>) -> Result<&str, &'static str> {
    name.ok_or("A delivery needs a customer name")
}

fn main() {
    println!("{:?}", delivery_name(Some("Amrit")));
    println!("{:?}", delivery_name(None));
}
```

This checks presence, not whether a present name is blank. If blank names are disallowed, that is another validation step.

`ok_or(error)` evaluates the error eagerly. The error can be any suitable value; it does not have to be a static string. `ok_or_else(|| make_error())` creates it only for `None`:

```rust
fn main() {
    let order_number = 42;
    let name: Option<String> = None;
    let result = name.ok_or_else(|| {
        format!("Order {order_number} needs a customer name")
    });
    println!("{result:?}");
}
```

### Result to Option: Discarding Error Information

`ok` keeps only success. `err` keeps only failure. Both take the result by value and discard the other variant's payload if present:

```rust
fn main() {
    let parsed = "two".parse::<u32>();
    let optional = parsed.ok();
    println!("{optional:?}"); // None; the parse error is gone.

    let result: Result<String, String> = Err(String::from("Printer offline"));
    let borrowed_error = result.as_ref().err();
    println!("{borrowed_error:?}");
    println!("Original result: {result:?}");
}
```

The second example borrows first because we want to retain the original result. Use `inspect_err` when the aim is to observe an error while continuing a result chain. Conversions should reflect a decision about which information the caller still needs.

### Interconverting Nested Types: transpose

The café's order form has an optional table-number field. No field means a takeaway order. A present field must contain a valid number. These are three distinct outcomes:

| Outcome | Type representation |
| :-- | :-- |
| No table entered | `Ok(None)` |
| A valid table number | `Ok(Some(number))` |
| A table was entered but could not be parsed | `Err(error)` |

Start with `Option<&str>`. Mapping a parser over it produces `Option<Result<u32, ParseIntError>>`. `transpose` moves the failure layer outside:

```rust
use std::num::ParseIntError;

fn parse_table(input: Option<&str>) -> Result<Option<u32>, ParseIntError> {
    input.map(|text| text.parse::<u32>()).transpose()
}

fn main() {
    println!("{:?}", parse_table(None));        // Ok(None)
    println!("{:?}", parse_table(Some("4")));   // Ok(Some(4))
    println!("{:?}", parse_table(Some("four"))); // Err(...)
}
```

This example checks numeric syntax only; checking that table 4 actually exists would be a separate operation. The complete state mapping is:

```text
Option<Result<T, E>>       Result<Option<T>, E>
None                  <-> Ok(None)
Some(Ok(value))        <-> Ok(Some(value))
Some(Err(error))       <-> Err(error)
```

`Option::transpose` goes from left to right; `Result::transpose` goes back. No success, absence, or error distinction is lost.

### Flattening: when one wrapper is enough

`flatten` removes one layer of the same kind of wrapper:

```rust
fn main() {
    let nested = Some(Some(2_u32));
    println!("{:?}", nested.flatten()); // Some(2)

    let result: Result<Result<u32, &str>, &str> = Ok(Err("Invalid quantity"));
    println!("{:?}", result.flatten()); // Err("Invalid quantity")
}
```

`Option::flatten` maps both `None` and `Some(None)` to `None`. That is not always what you want. For an update to a customer's delivery note, you could deliberately give `Option<Option<String>>` these meanings:

```text
None                 Leave the existing note unchanged.
Some(None)           Clear the existing note.
Some(Some(new_note))  Replace the note.
```

Flattening would erase the difference between leaving the note alone and clearing it. This is a contract you can choose for a Rust API; do not assume a serialization format automatically implements the same distinction.

`Result::flatten`, stable since **Rust 1.89**, requires the inner and outer errors to have the same type `E`. Both an outer error and an inner error become `Err(E)`, so their original nesting level is not retained unless the error value records it. Before 1.89, `nested.and_then(|inner| inner)` provides the same flattening behavior. If error types differ, first convert them to an appropriate common type.

For a fresh pipeline, `result.and_then(next_step)` often expresses the intent more directly than `result.map(next_step).flatten()`. Both are useful to recognize. [Transpose](https://doc.rust-lang.org/std/option/enum.Option.html#method.transpose) and [Result flatten](https://doc.rust-lang.org/std/result/enum.Result.html#method.flatten) document the exact types.

<a id="guide-propagation"></a>

## The Question Mark Operator: Ergonomic Error Propagation

We have chosen our missing-value and error meanings. Now we can shorten the code that passes a failure back to its caller.

### How ? Works

For `Result`, `?` gives you the success payload or returns an error early. Compare these two equivalent functions:

```rust
use std::num::ParseIntError;

fn quantity_verbose(input: &str) -> Result<u32, ParseIntError> {
    let n = match input.parse::<u32>() {
        Ok(n) => n,
        Err(error) => return Err(error),
    };
    Ok(n)
}

fn quantity_short(input: &str) -> Result<u32, ParseIntError> {
    let n = input.parse::<u32>()?;
    Ok(n)
}

fn main() {
    println!("{:?}", quantity_verbose("2"));
    println!("{:?}", quantity_short("two"));
}
```

For these tiny functions, returning `input.parse()` directly would be even shorter. The expanded form is here so the early return is visible.

When the enclosing function uses a different error type, ordinary `Result` propagation uses a `From` conversion. The important part has this shape:

```text
match expression {
    Ok(value) => value,
    Err(error) => return Err(From::from(error)),
}
```

The destination error type needs `From<SourceError>`. `?` does not automatically convert the successful value to a different success type.

### Typed errors: preserve the reason

Error variants let code distinguish cases without searching message text. In this example, a quantity either failed to parse or was outside our permitted range:

```rust
use std::num::ParseIntError;

#[derive(Debug)]
enum OrderError {
    Parse(ParseIntError),
    OutOfRange(u32),
}

impl From<ParseIntError> for OrderError {
    fn from(error: ParseIntError) -> Self {
        Self::Parse(error)
    }
}

fn read_quantity(input: &str) -> Result<u32, OrderError> {
    let n = input.trim().parse::<u32>()?;
    if !(1..=10).contains(&n) {
        return Err(OrderError::OutOfRange(n));
    }
    Ok(n)
}

fn main() {
    for input in ["2", "two", "12"] {
        match read_quantity(input) {
            Ok(n) => println!("Prepare {n} coffees"),
            Err(OrderError::Parse(source)) => println!("Invalid number: {source}"),
            Err(OrderError::OutOfRange(n)) => println!("{n} is outside 1 through 10"),
        }
    }
}
```

`OrderError` does not implement `std::error::Error`, and this still works. For standard error reporting and interoperability, implement `Display` for a human-readable explanation and `Error` to expose an underlying cause through `source()`. `Error` requires `Debug` and `Display`. The complete file example below shows all three.

`From` only receives the source error. If you need to attach the filename being read or the order number being processed, use `map_err` at the call site, where that context is available. Wrapping an error in `format!(...)` alone loses its structured source. [Standard error conventions](https://doc.rust-lang.org/std/error/trait.Error.html)

### Using ? with Option

For `Option`, `?` extracts `Some` or returns `None`:

```rust
fn add_last_numbers(first: &[i32], second: &[i32]) -> Option<i32> {
    let a = first.last()?;
    let b = second.last()?;
    a.checked_add(*b)
}

fn main() {
    println!("{:?}", add_last_numbers(&[2], &[3]));       // Some(5)
    println!("{:?}", add_last_numbers(&[], &[3]));        // None
    println!("{:?}", add_last_numbers(&[i32::MAX], &[1])); // None
}
```

Here `a` and `b` are `&i32`, because `last` borrows the final element. The method call can copy the integer receiver; `*b` provides the other integer value. Both an empty input and arithmetic overflow mean `None` under this function's contract.

To propagate an absent value from a function returning `Result`, first decide what absence means as an error:

```rust
fn required_name(name: Option<&str>) -> Result<&str, &'static str> {
    let name = name.ok_or("Missing delivery name")?;
    Ok(name)
}

fn main() {
    println!("{:?}", required_name(None));
}
```

`Option` and `Result` do not mix automatically through `?`. In the other direction, `.ok()?` inside an `Option`-returning function discards the error information before propagating absence.

### Constraints on Using ?

`?` needs a compatible enclosing return type. Ordinary `main() { ... }` returns `()`, which cannot carry the error from a parsing `Result`.

**Does not compile: E0277, incompatible enclosing return type.**

```rust
fn main() {
    let quantity = "two".parse::<u32>()?;
    println!("{quantity}");
}
```

Inside a closure, `?` returns from that closure, not straight through it from the surrounding function. That matters with iterators: a closure using a parsing `?` needs to return a compatible type, such as `Result`, which the outer code must then handle. Inside an `async` block, it affects that block's output.

This guide concentrates on `Option` and `Result`. Stable Rust also supports `?` for `ControlFlow` and certain `Poll` combinations. The general `Try` and `FromResidual` traits remain unstable for custom implementations in Rust 1.98.1. You do not need those internals to use the patterns here. [The Reference defines propagation and its supported types](https://doc.rust-lang.org/reference/expressions/operator-expr.html#the-try-propagation-expression).

### Early return cleans up; it does not undo previous work

If recording an order succeeds and a later step fails, `?` does not erase the record:

```rust
fn place_order(log: &mut Vec<String>) -> Result<(), &'static str> {
    log.push(String::from("Order recorded"));
    let printing: Result<(), &str> = Err("Printer offline");
    printing?;
    Ok(())
}

fn main() {
    let mut log = Vec::new();
    println!("{:?}", place_order(&mut log));
    println!("{log:?}");
}
```

Output:

```text
Err("Printer offline")
["Order recorded"]
```

Locals still owned by the returning function are dropped as their scopes are exited. The caller's log survives with the change. Sending a payment request, writing a file, or changing a database likewise needs an explicit recovery or transaction policy if later failure must undo the operation. Returning an error is control flow, not automatic rollback. [Drop-scope rules](https://doc.rust-lang.org/reference/destructors.html#drop-scopes)

<a id="guide-termination"></a>

## Understanding main() -> Result: The Termination Trait

Rust permits `main` to return a type implementing `std::process::Termination`. The standard implementation for `Result<T, E>` requires `T: Termination` and `E: Debug`:

```rust
use std::num::ParseIntError;

fn main() -> Result<(), ParseIntError> {
    let quantity = "2".parse::<u32>()?;
    println!("Prepare {quantity} coffees");
    Ok(())
}
```

On `Ok(())`, the program reports successful termination. On `Err(error)`, the standard implementation prints the error with its **Debug** formatter to stderr and reports `ExitCode::FAILURE`. More generally, `Ok(value)` delegates to that value's `Termination::report`, so `Ok` does not guarantee exit success for every possible `T`.

`ExitCode::FAILURE` is typically 1, but use the named value instead of depending on a numeric value across platforms. Returning `Result` does not automatically format a friendly error chain with `Display`. To choose the wording or exit policy, handle the result explicitly at the program boundary, as the next example does. [Termination](https://doc.rust-lang.org/std/process/trait.Termination.html) and [exit-code portability](https://doc.rust-lang.org/std/process/struct.ExitCode.html)

<a id="guide-cafe"></a>

## A complete café example: an optional saved quantity

The café can save its usual order quantity in a text file. The rules are small and explicit:

- A missing file is allowed: use one coffee as the default.
- An existing file must contain a whole number from 1 through 10.
- A read failure other than `NotFound`, or invalid contents, is an error. Do not silently replace it with the default.
- Errors retain the path and, where there is one, the underlying error.

Thus the reader returns `Result<Option<u32>, ConfigError>`: success with a quantity, success without a saved quantity, or failure. The `NotFound` rule is our chosen policy for that I/O error kind; it can also cover a missing parent directory, not just a missing final filename.

Save this complete program as `cafe.rs`. Compile with `rustc --edition=2024 cafe.rs`. Run `./cafe quantity.txt`, choosing a path you control. With that file absent it prints `Coffees to prepare: 1`. Put `2` in the file and it prints `Coffees to prepare: 2`. Put `two` or `0` in it and the program reports an error and exits unsuccessfully. It only reads the file; it does not create or change it.

```rust
use std::error::Error;
use std::fmt;
use std::fs;
use std::io;
use std::num::ParseIntError;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

#[derive(Debug)]
enum ConfigError {
    Read { path: PathBuf, source: io::Error },
    Parse { path: PathBuf, source: ParseIntError },
    OutOfRange { path: PathBuf, quantity: u32 },
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Read { path, .. } => write!(f, "Cannot read {}", path.display()),
            Self::Parse { path, .. } => write!(f, "Expected a whole number in {}", path.display()),
            Self::OutOfRange { path, quantity } => {
                write!(f, "{} contains {quantity}; use 1 through 10", path.display())
            }
        }
    }
}

impl Error for ConfigError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Read { source, .. } => Some(source),
            Self::Parse { source, .. } => Some(source),
            Self::OutOfRange { .. } => None,
        }
    }
}

fn read_saved_quantity(path: &Path) -> Result<Option<u32>, ConfigError> {
    let text = match fs::read_to_string(path) {
        Ok(text) => text,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(source) => return Err(ConfigError::Read { path: path.to_owned(), source }),
    };

    let quantity = text.trim().parse::<u32>().map_err(|source| {
        ConfigError::Parse { path: path.to_owned(), source }
    })?;

    if !(1..=10).contains(&quantity) {
        return Err(ConfigError::OutOfRange { path: path.to_owned(), quantity });
    }

    Ok(Some(quantity))
}

fn main() -> ExitCode {
    let mut args = std::env::args_os().skip(1);
    let Some(path) = args.next() else {
        eprintln!("Usage: cafe <quantity-file>");
        return ExitCode::FAILURE;
    };
    if args.next().is_some() {
        eprintln!("Usage: cafe <quantity-file>");
        return ExitCode::FAILURE;
    }

    match read_saved_quantity(Path::new(&path)) {
        Ok(saved) => {
            let quantity = saved.unwrap_or(1);
            println!("Coffees to prepare: {quantity}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("{error}");
            let mut cause = error.source();
            while let Some(source) = cause {
                eprintln!("  caused by: {source}");
                cause = source.source();
            }
            ExitCode::FAILURE
        }
    }
}
```

Notice where the decisions live. The reader distinguishes a missing file from a broken one. `map_err` attaches the path while preserving the parser's error. `?` propagates that structured error. The caller supplies a default only for `Ok(None)` and prints failures once at the boundary.

The `source` signature can look imposing. `dyn Error` lets the reference point to different concrete error types: here `io::Error` or `ParseIntError`. The `'static` bound describes those underlying types; it does **not** make this borrowed reference live forever. Its reference lifetime is tied to `&self` by lifetime elision. A later [trait-object post](/rust/concepts/2025/10/23/rust-dyn.html) develops that topic.

`Display` provides context and `source()` exposes the cause. The outer message does not repeat the source's text, because the reporting loop prints that cause separately. If you instead format the cause into every outer message and also walk the chain, users can see the same explanation repeatedly. [Error-source guidance](https://doc.rust-lang.org/std/error/trait.Error.html#error-source)

For a small application, `Result<T, Box<dyn Error>>` is another useful boundary: it can hold different concrete errors through one trait object. Ordinary `?` can use the standard conversions for compatible error types. That convenience does not provide your domain's named error variants or automatically attach operation-specific context. Choose a concrete enum when callers need a stable set of cases to match.

<a id="guide-iterators"></a>

## From one value to several: slices and iterators

### Converting to Slices: as_slice and as_mut_slice

`Option<T>::as_slice` borrows zero or one `T` as a slice. `as_mut_slice` gives the mutable version; it cannot grow that slice or turn `None` into `Some`:

```rust
fn main() {
    let extra = Some(2_u32);
    let none: Option<u32> = None;
    println!("{:?}", extra.as_slice()); // [2]
    println!("{:?}", none.as_slice());  // []

    let mut quantity = Some(2_u32);
    if let [n] = quantity.as_mut_slice() {
        *n += 1;
    }
    println!("{quantity:?}"); // Some(3)
}
```

Do not confuse this with `as_deref`: `Option<Vec<u32>>::as_slice()` gives a slice of zero or one **vectors** (`&[Vec<u32>]`), whereas `as_deref()` gives an optional slice of the vector's **elements** (`Option<&[u32]>`). Let the types tell you which layer you are borrowing.

### Optional values can be iterated

An `Option` can supply zero or one item to a loop or iterator chain. `iter` borrows, `iter_mut` mutably borrows, and `into_iter` takes the option by value:

```rust
fn main() {
    let regular = [1_u32, 2];
    let extra = Some(3_u32);

    for quantity in regular.iter().chain(extra.iter()) {
        println!("{quantity}");
    }
}
```

### Collecting results: fail together or deliberately skip failures

Parsing several order quantities forces a policy choice:

```rust
fn main() {
    let inputs = ["2", "two", "3"];

    let all = inputs.iter()
        .map(|text| text.parse::<u32>())
        .collect::<Result<Vec<_>, _>>();
    println!("All quantities: {all:?}"); // Err(...)

    let accepted = inputs.iter()
        .filter_map(|text| text.parse::<u32>().ok())
        .collect::<Vec<_>>();
    println!("Accepted only: {accepted:?}"); // [2, 3]
}
```

Collecting into `Result<Vec<_>, E>` stops consuming the iterator at the first error and returns it. It does not collect every error. Any side effects already performed by earlier items remain. Collecting into `Option<Vec<_>>` similarly stops at the first `None`.

The second pipeline deliberately throws parsing errors away. `Iterator::flatten` over `Result` items also skips errors, because a `Result` iterates over zero or one success value. That is different from `Result::flatten`, which collapses nested results and retains an error. When you need all error reports, write an explicit accumulation policy instead. The later [collect post](/rust/concepts/2025/11/23/rust-collect.html) goes deeper. [Iterator flatten](https://doc.rust-lang.org/std/iter/trait.Iterator.html#method.flatten) and [Result collection](https://doc.rust-lang.org/std/result/enum.Result.html#impl-FromIterator%3CResult%3CA,+E%3E%3E-for-Result%3CV,+E%3E)

<a id="guide-decisions"></a>

## Practical Decision Guide

Choose the return type by the distinctions the caller needs:

| Situation | A useful type |
| :-- | :-- |
| A customer may omit their cup label | `Option<String>` |
| A quantity can fail validation with a reason | `Result<u32, OrderError>` |
| Reading an optional setting can itself fail | `Result<Option<u32>, ConfigError>` |
| A successful action has nothing useful to return | `Result<(), E>` |
| A found value should remain owned by a collection | `Option<&T>` |
| An update must distinguish unchanged, cleared, and replaced | A clearly documented `Option<Option<T>>`, or a named enum |

Then choose the operation:

| Question | Start with |
| :-- | :-- |
| Do both variants need different work? | `match` |
| Does only one pattern need work? | `if let` |
| Should the rest of this scope require one variant? | `let ... else` |
| Should the caller handle failure? | `?` with a compatible return type |
| Is this default actually allowed? | `unwrap_or` or `unwrap_or_else` |
| Do I want to keep ownership? | Borrow first with `as_ref`, `as_mut`, or `as_deref` |
| Does the next step return another wrapper? | Consider `and_then`; use `map` if nesting is intentional |
| Does the fallback need to run only after failure? | `or_else` |
| Am I handing an owned field over? | `take` or `replace` |
| Am I discarding an error or a state distinction? | Make that policy explicit before using `ok`, `flatten`, or a default |

Prefer the expression that makes the policy easiest to read. A longer `match` is often clearer than a chain with several different meanings of failure.

## Quick Reference

The table shows ordinary type flow, not complete Rust declarations. By-value methods move non-`Copy` wrappers and can copy `Copy` wrappers. Lifetimes and some generic bounds are abbreviated; use the linked standard-library signatures for the full contracts.

### `Option<T>` Methods

| Method | Receiver or type flow | What to remember |
| :-- | :-- | :-- |
| `unwrap`, `expect` | `Option<T> -> T` | Panic on `None`; `expect` explains the expectation |
| `unwrap_or`, `unwrap_or_else` | `Option<T> -> T` | Eager value versus callback `() -> T` |
| `unwrap_or_default` | `Option<T> -> T` | Requires `T: Default` |
| `is_some`, `is_none` | `&Option<T> -> bool` | Borrowing variant checks |
| `is_some_and`, `is_none_or` | `Option<T> -> bool` | Predicate receives `T`; absence gives false / true respectively |
| `map` | `Option<T> -> Option<U>` | Callback returns any `U` |
| `map_or`, `map_or_else`, `map_or_default` | `Option<T> -> U` | Produce an unwrapped value; last requires `U: Default` |
| `and_then` | `Option<T> -> Option<U>` | Callback returns `Option<U>` |
| `filter` | `Option<T> -> Option<T>` | Keep `Some` only if predicate on `&T` is true |
| `inspect` | `Option<T> -> Option<T>` | Callback borrows `&T`; wrapper is returned |
| `as_ref`, `as_mut` | Borrow wrapper -> `Option<&T>` / `Option<&mut T>` | Keep the owned payload in place |
| `as_deref`, `as_deref_mut` | Borrow wrapper -> optional reference to dereference target | Require `Deref` / `DerefMut` |
| `copied`, `cloned` | `Option<&T> -> Option<T>` | Require `Copy` / `Clone`; mutable-reference forms also exist |
| `and` | `Option<T> -> Option<U>` | Second option already evaluated |
| `or`, `or_else` | `Option<T> -> Option<T>` | Eager alternative versus callback `() -> Option<T>` |
| `ok_or`, `ok_or_else` | `Option<T> -> Result<T, E>` | Eager error versus callback `() -> E` |
| `transpose` | `Option<Result<T, E>> -> Result<Option<T>, E>` | Preserve all three outcomes |
| `flatten` | `Option<Option<T>> -> Option<T>` | Remove one layer; merge absence states |
| `take`, `replace` | `&mut Option<T> -> Option<T>` | Return the old option; leave `None` / `Some(new)` |
| `take_if` | `&mut Option<T> -> Option<T>` | Predicate can mutate `T` even when it returns false |
| `insert`, `get_or_insert`, `get_or_insert_with`, `get_or_insert_default` | `&mut Option<T> -> &mut T` | Replace or fill the slot; see the mutation section |
| `as_slice`, `as_mut_slice` | Borrow wrapper -> `&[T]` / `&mut [T]` | Zero or one payloads |
| `iter`, `iter_mut`, `into_iter` | Borrow / mutably borrow / take wrapper | Zero or one `&T` / `&mut T` / `T` items |
| `zip` | `(Option<T>, Option<U>) -> Option<(T, U)>` | Both must be present; both arguments already evaluated |

### `Result<T, E>` Methods

| Method | Receiver or type flow | What to remember |
| :-- | :-- | :-- |
| `unwrap`, `expect` | `Result<T, E> -> T` | Panic on `Err`; require `E: Debug` |
| `unwrap_or`, `unwrap_or_else` | `Result<T, E> -> T` | Eager value versus callback `E -> T` |
| `unwrap_or_default` | `Result<T, E> -> T` | Requires `T: Default`; discards error |
| `is_ok`, `is_err` | `&Result<T, E> -> bool` | Borrowing variant checks |
| `is_ok_and`, `is_err_and` | `Result<T, E> -> bool` | Predicates receive `T` / `E` by value |
| `map` | `Result<T, E> -> Result<U, E>` | Change success type |
| `map_err` | `Result<T, E> -> Result<T, F>` | Change error type |
| `map_or`, `map_or_else`, `map_or_default` | `Result<T, E> -> U` | Produce an unwrapped value; `map_or_else`'s failure callback receives `E`; `map_or_default` requires `U: Default` |
| `and_then` | `Result<T, E> -> Result<U, E>` | Next fallible step uses the same error type |
| `inspect`, `inspect_err` | `Result<T, E> -> Result<T, E>` | Callback borrows `&T` / `&E`; wrapper is returned |
| `as_ref`, `as_mut` | Borrow wrapper -> `Result<&T, &E>` / `Result<&mut T, &mut E>` | Borrow either payload |
| `as_deref`, `as_deref_mut` | Borrow wrapper -> `Result<&T::Target, &E>` / mutable equivalents | Dereference only the success payload |
| `copied`, `cloned` | `Result<&T, E> -> Result<T, E>` | Copy / clone success; mutable-reference forms also exist |
| `and` | `Result<T, E> -> Result<U, E>` | Eager second result; retain first error if present |
| `or`, `or_else` | `Result<T, E> -> Result<T, F>` | Eager alternative versus callback `E -> Result<T, F>` |
| `ok`, `err` | `Result<T, E> -> Option<T>` / `Option<E>` | Discard the other payload |
| `transpose` | `Result<Option<T>, E> -> Option<Result<T, E>>` | Preserve all three outcomes |
| `flatten` | `Result<Result<T, E>, E> -> Result<T, E>` | Same error type in both layers |
| `iter`, `iter_mut`, `into_iter` | Borrow / mutably borrow / take wrapper | Iterate over success only; errors provide no item |

These are selected methods for the problems taught here, not every method in the library. In particular, `unwrap_unchecked` also exists, but calling it on `None` or `Err` is **undefined behavior**, not a panic. An `unsafe` block does not check or establish its precondition. None of this guide's examples needs it; use safe operations unless a separately justified unsafe implementation proves the required invariant. [Option safety contract](https://doc.rust-lang.org/std/option/enum.Option.html#method.unwrap_unchecked) and [Result safety contract](https://doc.rust-lang.org/std/result/enum.Result.html#method.unwrap_unchecked)

### Version notes: a version is not an edition

| Feature | Stable since | Edition requirement |
| :-- | :-- | :-- |
| `let ... else` | Rust 1.65 | No special edition requirement |
| `is_some_and`, `is_ok_and`, `is_err_and` | Rust 1.70 | No special edition requirement |
| `Option::as_slice`, `as_mut_slice` | Rust 1.75; `const` use since 1.84 | No special edition requirement |
| `inspect`, `Result::inspect_err` | Rust 1.76 | No special edition requirement |
| `Option::take_if` | Rust 1.80 | No special edition requirement |
| `Option::is_none_or` | Rust 1.82 | No special edition requirement |
| `Option::get_or_insert_default` | Rust 1.83 | No special edition requirement |
| `Result::flatten` | Rust 1.89 | No special edition requirement |
| `Option::map_or_default`, `Result::map_or_default` | Rust 1.98 | No special edition requirement |
| `if let` / `while let` chains | Rust 1.88 | Edition 2024 |

Let chains let you combine a pattern and another condition in one `if`. This is a convenient newer spelling, not a replacement for understanding the alternatives:

```rust
fn main() {
    let quantity = Some(2_u32);
    if let Some(n) = quantity && (1..=10).contains(&n) {
        println!("Prepare {n} coffees");
    }
}
```

Compile that example in edition 2024. The first post already explains edition-2024 changes to temporary lifetimes; those can matter when the expression being matched holds a lock or a borrow guard. [Rust 1.88 let chains](https://blog.rust-lang.org/2025/06/26/Rust-1.88.0/) and [the edition guide](https://doc.rust-lang.org/edition-guide/rust-2024/temporary-if-let-scope.html) give the details. API stabilization versions are also displayed beside methods in the standard-library docs.

<a id="guide-practice"></a>

## Practise predicting the behavior

Before running code, explain these in your own words:

1. Why can `Some(2_u32).map(...)` leave a named original usable while `Some(String::from("Amrit")).map(...)` can move it?
2. Why does `name.as_ref().map(|s| s.len())` preserve the string?
3. If a fallback is never called, how can its closure still consume a string?
4. What extra work happens in `first().or(second())` compared with `first().or_else(second)`?
5. When reading the saved quantity, why must `Ok(None)` and `Err(...)` stay different?
6. Which distinction does flattening the delivery-note update erase?
7. Why can `take` hand a field out through `&mut self` without cloning?
8. If printing fails after the order log changes, what does `?` undo?
9. Which parsing pipeline rejects the batch, and which quietly drops bad entries?
10. Why does `main() -> Result` use `Debug`, even if your error implements `Display`?

Answers: `Copy` is conditional on the wrapper's payload types; borrowing makes a wrapper of references instead of moving the owned payload; closure capture happens before invocation; `or` evaluates its argument eagerly; absence and failure have different business policies; `flatten` merges unchanged and clear; `take` leaves a valid `None`; `?` undoes no earlier side effects; result collection stops at the first error while `filter_map(...ok())` discards errors; the standard `Termination` implementation chooses `Debug`.

## Summary

At the café, a missing cup name, a rejected quantity, a receipt waiting to be handed over, and an unreadable settings file are different situations. `Option` and `Result` let the types express those differences. Ownership determines whether you move, copy, or borrow their contents. Combinators determine which branch runs and which information continues. `?` passes a failure to the appropriate caller, while cleanup and rollback remain separate questions.

To deepen this skill, keep asking what each expression owns, what each callback returns, and which states are being preserved or merged. The next [enum and pattern-matching post](/rust/concepts/2025/02/05/rust-enums.html) generalizes these ideas to your own types; the later [ownership post](/rust/concepts/2025/02/09/rust-ownership.html) develops the borrowing rules further.
