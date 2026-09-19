---
layout: post
title: "Mastering Rust Ownership: Advanced Patterns, Performance, and Real-World Applications"
date: 2025-02-09 10:23:00 +0530
categories: rust concepts
last_updated: 2026-09-19
rust_version: "1.98.1"
rust_edition: "2024"
---

# Mastering Rust Ownership: Advanced Patterns, Performance, and Real-World Applications <a href="#mastering-rust-ownership-advanced-patterns-performance-and-real-world-applications-" class="header-link">🔗</a>

This is the fifth post in the Rust series. The first four established [bindings, ownership and borrowing](/rust/concepts/2025/01/01/rust-var-const-lifetimes.html), [memory layout](/rust/concepts/2025/01/05/rust-mem-ref.html), [Option and Result](/rust/concepts/2025/01/09/rust-option-result.html), and [enums and patterns](/rust/concepts/2025/02/05/rust-enums.html). We now use those ideas to design APIs, explain lifetime errors, and judge advanced ownership patterns.

The order is deliberate: cleanup and representation → moves → borrowing → lifetime contracts → application design → unsafe boundaries → async work → performance → debugging → edition-sensitive details. Earlier posts already introduced gotchas such as partial moves and reborrowing. Here we ask what those rules mean when several pieces must work together.

**Updated 19 September 2026 for Rust 1.98.1, edition 2024.** The compiler version and edition are separate choices: set `edition = "2024"` in your package's `Cargo.toml`, or use `rustc --edition=2024 example.rs`. An up-to-date compiler can still compile a 2021-edition crate under its older rules. [Rust 1.98.1 release](https://blog.rust-lang.org/2026/09/03/Rust-1.98.1/).

**How to use the examples.** Each Rust block with `fn main` is a separate program unless labelled otherwise. Blocks labelled **Does not compile** are deliberate lessons. The double-panic example is labelled **Aborts**. Examples using Tokio need the dependency shown in Part I. These are teaching examples, not complete database, web-server, or foreign-language libraries; `unwrap()` keeps incidental error handling short.

Keep the café from post one in mind. An order belongs to someone; a kitchen worker can borrow it; a receipt may be handed away; a background job may need its own copy. For every example, answer three questions before running it:

1. **Who is responsible for this value now?** A move hands over that responsibility. `Rc` and `Arc` share responsibility through owning handles.
2. **Who may access it, and for how long?** A reference grants temporary access; it does not keep its owner alive.
3. **What ends the obligation?** Ordinary cleanup, an explicit operation such as `finish()`, and business actions such as committing a sale are different events.

## Table of Contents <a href="#table-of-contents-" class="header-link">🔗</a>

- [Part I: Deep Ownership Mechanics](#part-i-deep-ownership-mechanics-)
  - [Drop Semantics and RAII Patterns](#drop-semantics-and-raii-patterns-)
  - [Memory Layout Internals](#memory-layout-internals-)
  - [Ownership Transfer Patterns](#ownership-transfer-patterns-)
  - [Zero-Sized Types and Phantom Data](#zero-sized-types-and-phantom-data-)

- [Part II: Advanced Move Semantics](#part-ii-advanced-move-semantics-)
  - [Partial Moves Mastery](#partial-moves-mastery-)
  - [Move and Panic Interactions](#move-and-panic-interactions-)
  - [Closure Ownership (FnOnce/Fn/FnMut)](#closure-ownership-fnoncefnfnmut-)
  - [Iterator Ownership Patterns](#iterator-ownership-patterns-)

- [Part III: Advanced Borrowing](#part-iii-advanced-borrowing-)
  - [Splitting Borrows and Field Sensitivity](#splitting-borrows-and-field-sensitivity-)
  - [Interior Mutability Deep Dive](#interior-mutability-deep-dive-)
  - [Coercion and Deref Magic](#coercion-and-deref-magic-)
  - [Variance and Lifetime Subtyping](#variance-and-lifetime-subtyping-)

- [Part IV: Lifetime Mastery](#part-iv-lifetime-mastery-)
  - [Reading and Writing Lifetime Contracts](#reading-and-writing-lifetime-contracts)
  - [Higher-Ranked Trait Bounds (HRTB)](#higher-ranked-trait-bounds-hrtb-)
  - [Variance Rules and Implications](#variance-rules-and-implications-)
  - [Self-Referential Structs and Pin](#self-referential-structs-and-pin-)
  - [Generic Associated Types (GATs)](#generic-associated-types-gats-)

- [Part V: Ownership in Practice](#part-v-ownership-in-practice-)
  - [Graph Structures and Arena Allocation](#graph-structures-and-arena-allocation-)
  - [Observer Patterns Without Cycles](#observer-patterns-without-cycles-)
  - [Plugin Architectures](#plugin-architectures-)
  - [Real-World Case Study: HTTP Router](#real-world-case-study-http-router-)

- [Part VI: Unsafe and Ownership](#part-vi-unsafe-and-ownership-)
  - [Raw Pointer Ownership Conventions](#raw-pointer-ownership-conventions-)
  - [Building Safe Abstractions](#building-safe-abstractions-)
  - [FFI Ownership Patterns](#ffi-ownership-patterns-)
  - [ManuallyDrop and mem::forget](#manuallydrop-and-memforget-)

- [Part VII: Async Ownership](#part-vii-async-ownership-)
  - [Send and Sync Deep Dive](#send-and-sync-deep-dive-)
  - [Lifetime Challenges in Async](#lifetime-challenges-in-async-)
  - [Scoped Tasks and Non-Static Borrows](#scoped-tasks-and-non-static-borrows-)

- [Part VIII: Performance and Optimization](#part-viii-performance-and-optimization-)
  - [Copy-on-Write Patterns (Cow)](#copy-on-write-patterns-cow-)
  - [Memory Locality Strategies](#memory-locality-strategies-)
  - [Cache-Conscious Design](#cache-conscious-design-)

- [Part IX: Anti-Patterns and Debugging](#part-ix-anti-patterns-and-debugging-)
  - [Common Ownership Mistakes](#common-ownership-mistakes-)
  - [Refactoring Strategies](#refactoring-strategies-)
  - [Debugging the Borrow Checker](#debugging-the-borrow-checker-)

- [Part X: Advanced Topics](#part-x-advanced-topics-)
  - [RPIT Capture Rules and Lifetime Control](#rpit-capture-rules-and-lifetime-control-)
  - [Advanced Async Patterns](#advanced-async-patterns-)
  - [Temporary Scope Behavior](#temporary-scope-behavior-)
  - [Scoped Borrowing in Complex Patterns](#scoped-borrowing-in-complex-patterns-)

- [Key Takeaways](#key-takeaways-)
- [Further Reading](#further-reading-)

***

## Part I: Deep Ownership Mechanics <a href="#part-i-deep-ownership-mechanics-" class="header-link">🔗</a>

### Drop Semantics and RAII Patterns <a href="#drop-semantics-and-raii-patterns-" class="header-link">🔗</a>

A resource-owning value is like the key to the café's cash drawer: give the key to a guard, and the guard takes responsibility for putting things away. **RAII** means tying resource management to a value's lifetime. Rust automatically drops fields; implement `Drop` only for extra cleanup your type itself must perform.

**Custom Drop Implementation**:

```rust
use std::fs::File;

struct FileGuard {
    path: String,
    handle: File,
}

impl Drop for FileGuard {
    fn drop(&mut self) {
        println!("Closing file: {}", self.path);
    }
}

fn main() {
    let _guard = FileGuard {
        path: "data.txt".into(),
        handle: File::create("data.txt").unwrap(),
    };
    println!("FileGuard created");
}
```

`FileGuard::drop` prints a message; the `File` field closes the handle afterward. You do not manually drop its fields. Real file writes can fail, so use an explicit operation returning `Result` when the caller must learn about flushing or persistence errors. `Drop::drop` cannot return such an error. [File cleanup](https://doc.rust-lang.org/std/fs/struct.File.html).

**Drop Order Guarantees**:

For ordinary scope exit, these are language-defined orders. They describe cleanup order; memory safety also depends on borrowing and validity rules:

1. **Fields**: Dropped in declaration order (top to bottom)
2. **Tuples/Arrays**: Dropped in index order (first to last)
3. **Local bindings in a scope**: Dropped in reverse declaration order

A custom `Drop::drop` runs before its fields are dropped. After a partial move, only the fields still present are dropped at the original location. Do not extend the array rule to every container: `Vec` does not promise its elements' drop order. [Destructor rules](https://doc.rust-lang.org/reference/destructors.html), [Vec guarantees](https://doc.rust-lang.org/std/vec/struct.Vec.html#guarantees).

```rust
struct Outer {
    first: Inner,
    second: Inner,
}

struct Inner(&'static str);

impl Drop for Inner {
    fn drop(&mut self) {
        println!("Dropping {}", self.0);
    }
}

fn main() {
    let _container = Outer {
        first: Inner("first"),
        second: Inner("second"),
    };

    // Output:
    // Dropping first
    // Dropping second
}
```

**Drop and Panic Interactions**:

**Aborts — compile if you want to inspect it, but do not run it as an ordinary example.** If a destructor's panic escapes while another panic is unwinding, the process aborts. With `panic = "abort"`, the first panic aborts without unwinding. Avoid panicking destructors. [Panic behavior](https://doc.rust-lang.org/reference/panic.html).

```rust
struct PanicDrop;

impl Drop for PanicDrop {
    fn drop(&mut self) {
        panic!("Drop panic!");
    }
}

fn main() {
    let _guard = PanicDrop;
    // If another panic occurs, dropping during unwinding causes abort
    panic!("First panic");
}
```

**The Drop Bomb Pattern**:

A “drop bomb” complains when a value is dropped without a required action. It is a diagnostic convention, not a guarantee that the action happens: safe code can forget the value, and a process can abort. A real transaction should expose explicit commit/rollback operations and use a non-panicking cleanup policy; the tiny type below only demonstrates consuming `self`.

```rust
struct Transaction {
    committed: bool,
}

impl Transaction {
    fn new() -> Self {
        Self { committed: false }
    }

    fn commit(mut self) {
        self.committed = true;
        println!("Transaction committed");
    }
}

impl Drop for Transaction {
    fn drop(&mut self) {
        if !self.committed {
            panic!("Transaction dropped without commit!");
        }
    }
}

fn main() {
    let tx = Transaction::new();
    tx.commit();
}
```

**Why Drop and Copy Are Mutually Exclusive**:

`Copy` permits implicit duplication; `Drop` gives each value a destruction action. Rust forbids implementing both on one type. For resource owners, duplicating the ownership bytes could cause double cleanup; that is a motivating example, not a claim that every possible destructor frees memory. `Copy` also requires `Clone`, and a derived `Copy` needs every field to be `Copy`. [Copy's contract](https://doc.rust-lang.org/std/marker/trait.Copy.html).

```rust
#[derive(Debug, Copy, Clone)]
struct Numbers {
    x: i32,
    y: i32,
}

// This doesn't compile:
// impl Drop for Numbers {
//     fn drop(&mut self) {
//         // ERROR: Copy types cannot implement Drop
//     }
// }

fn main() {
    let nums = Numbers { x: 1, y: 2 };
    let copy = nums;
    let another_copy = nums;

    println!("Multiple copies: {:?}, {:?}", copy, another_copy);
}
```

### Memory Layout Internals <a href="#memory-layout-internals-" class="header-link">🔗</a>

Ownership says who is responsible for a value. Layout says where its bytes go. Do not use one as a shortcut for the other: a move is not necessarily a heap copy, and field drop order does not reveal physical field order.

**Struct layout and padding**

Rust's default representation may reorder fields. To demonstrate the cost of a particular field order, use `repr(C)`, which preserves that order and follows the target's C layout rules.

```rust
use std::mem::{align_of, size_of};

#[repr(C)]
struct TicketA {
    kind: u8,
    total: u64,
    table: u16,
}

#[repr(C)]
struct TicketB {
    total: u64,
    table: u16,
    kind: u8,
}

fn main() {
    println!("u64 alignment: {}", align_of::<u64>());
    println!("A: {}, B: {}", size_of::<TicketA>(), size_of::<TicketB>());
    // Where u64 has 8-byte alignment: A is 24 bytes, B is 16.
}
```

These sizes are conditional on the target's alignment rules. Removing `repr(C)` also removes the field-order promise; both structs may then be 16 bytes. Measure your target instead of assuming “64-bit” specifies every layout detail.

**Pointers with metadata**

```rust
use std::mem::size_of;

fn main() {
    println!("thin pointer: {}", size_of::<*const i32>());
    println!("slice reference: {}", size_of::<&[i32]>());
    println!("trait-object reference: {}", size_of::<&dyn std::fmt::Debug>());
    println!("String value: {}", size_of::<String>());
}
```

On the 64-bit target used for these examples these print 8, 16, 16, and 24. A slice reference carries an element count; a trait-object reference carries information for dynamic dispatch. Treat observed sizes as measurements, not a portable foreign-language ABI. `size_of::<String>()` excludes the separately owned text buffer. See the [layout rules](https://doc.rust-lang.org/reference/type-layout.html) and [String representation](https://doc.rust-lang.org/std/string/struct.String.html#representation).

**Representation attributes**

- `repr(Rust)` is the default; do not assume source-order fields.
- `repr(C)` supports layout interoperability, but does not make a `String` field understandable to C or define who frees anything.
- `repr(transparent)` gives an eligible wrapper the layout and ABI of its one nontrivial field.
- `repr(packed)` reduces alignment. A field can become unaligned, and forming a reference to it is invalid. It is not a general performance switch. Copy a supported field by value, or use the documented raw-pointer unaligned operations when truly needed. [Packed-field error E0793](https://doc.rust-lang.org/error_codes/E0793.html).

### Ownership Transfer Patterns <a href="#ownership-transfer-patterns-" class="header-link">🔗</a>

**Ownership in Closures**:

A closure is a value containing what its body needs from the surrounding scope. The body determines how it must capture those values; `move` can force capture by value. The call traits then describe what *calling* the closure requires. Capture mode and call trait are related, but are not the same choice.

```rust
fn main() {
    let data = vec![1, 2, 3];

    let consume = || {
        println!("{:?}", data);
        drop(data);
    };
    consume();

    let data2 = vec![4, 5, 6];
    let borrow = || {
        println!("{:?}", data2);
    };
    borrow();
    borrow();

    let mut data3 = vec![7, 8, 9];
    let mut mutate = || {
        data3.push(10);
    };
    mutate();
    mutate();
    println!("{:?}", data3);
}
```

**Forcing Move Capture**:

```rust
fn main() {
    let data = vec![1, 2, 3];

    let closure = move || {
        println!("{:?}", data);
    };

    closure();
    closure(); // Owning data does not prevent repeated shared access to it.
}
```

Here the closure owns `data`, yet implements `Fn`: printing only borrows the captured vector. Consuming the captured vector with `drop(data)` would restrict it to `FnOnce`. [Closure capture and call traits](https://doc.rust-lang.org/reference/types/closure.html#call-traits-and-coercions).

**Ownership with Async/Await**:

All Tokio examples in this post use this dependency (the local checks use Tokio 1.52.3):

```toml
[dependencies]
tokio = { version = "1.52.3", features = ["macros", "rt-multi-thread"] }
```

Calling an `async fn` produces a future. Passing an owned argument transfers it into that future when you call the function; its body does work when the future is polled. `.await` waits for it within the current task; it does not itself spawn a task. [Async functions](https://doc.rust-lang.org/reference/items/functions.html#async-functions).

```rust
async fn process_data(data: Vec<i32>) -> i32 {
    data.iter().sum()
}

#[tokio::main]
async fn main() {
    let numbers = vec![1, 2, 3, 4, 5];

    let result = process_data(numbers).await;
    println!("Sum: {}", result);
}
```

### Zero-Sized Types and Phantom Data <a href="#zero-sized-types-and-phantom-data-" class="header-link">🔗</a>

A zero-sized type stores no value bytes, but it can still carry meaning and obey ownership rules. A café's “paid” marker need not store a number. It can be a type that tells the compiler which operations are allowed. A zero-sized value can even have a destructor; zero size is not a promise of zero runtime work.

```rust
use std::mem::size_of;

struct Paid;

fn main() {
    println!("Paid: {} bytes", size_of::<Paid>()); // 0
    let first = Paid;
    let _second = first; // A move: Paid does not implement Copy.
}
```

`PhantomData` describes a type relationship without storing that type's value. It affects variance, automatic `Send`/`Sync` behavior, and some drop checking. It does not allocate, own, or destroy an actual `T` by itself. Here it keeps two kinds of identifier distinct:

```rust
use std::marker::PhantomData;

struct Order;
struct Table;

struct Id<Kind> {
    number: u32,
    _kind: PhantomData<fn() -> Kind>,
}

impl<Kind> Id<Kind> {
    fn new(number: u32) -> Self {
        Self { number, _kind: PhantomData }
    }
}

fn serve(order: Id<Order>) {
    println!("Serving order {}", order.number);
}

fn main() {
    let order = Id::<Order>::new(7);
    let _table = Id::<Table>::new(7);
    serve(order);
    // serve(_table); // Does not compile: a table ID is not an order ID.
}
```

The `fn() -> Kind` marker expresses a type tag without claiming to store a `Kind`. An owning raw-pointer abstraction may instead need `PhantomData<T>`; a borrowed view may need `PhantomData<&'a T>`. Choose the marker to match the real contract. No manual `unsafe impl Send` or `Sync` is needed for the example. [PhantomData](https://doc.rust-lang.org/std/marker/struct.PhantomData.html).

One precision worth remembering for Part III: `PhantomData<&'a mut T>` is covariant in `'a`, but invariant in `T`. Calling the whole marker “invariant” hides the distinction.

***

## Part II: Advanced Move Semantics <a href="#part-ii-advanced-move-semantics-" class="header-link">🔗</a>

### Partial Moves Mastery <a href="#partial-moves-mastery-" class="header-link">🔗</a>

A partial move hands away one field. The remaining fields still belong to the original variable, but you cannot use the whole value as if it were intact. This is useful when the kitchen takes an order's item list while the till keeps its payment information.

**Copying, borrowing, then moving a field**:

```rust
#[derive(Debug)]
struct Config {
    version: u32,
    database_url: String,
    api_key: String,
}

fn main() {
    let config = Config {
        version: 1,
        database_url: String::from("postgres://localhost"),
        api_key: String::from("secret_key"),
    };

    let Config { version, ref database_url, .. } = config;

    println!("Version: {}", version);
    println!("DB: {}", database_url);

    println!("Version again: {}", config.version);
    println!("DB again: {}", config.database_url);

    let key = config.api_key;
    println!("Key length: {}", key.len());
    println!("Remaining version: {}", config.version);
    // println!("{:?}", config); // Does not compile: api_key has moved.
}
```

The first pattern copies the `u32` and explicitly borrows `database_url`; no non-`Copy` field moves there. The later `config.api_key` access performs the partial move. A struct implementing `Drop` cannot have a non-`Copy` field moved out this way: its destructor needs the whole value. Use `Option::take` or replace the field with a valid value instead. [Partial moves](https://doc.rust-lang.org/rust-by-example/scope/move/partial_move.html).

**Moving Out of Arrays**:

```rust
fn main() {
    let array = [
        String::from("a"),
        String::from("b"),
        String::from("c"),
    ];

    let [first, second, third] = array;
    println!("{} {} {}", first, second, third);

    let array2 = [String::from("x"), String::from("y")];
    for item in array2 {
        println!("{}", item);
    }
}
```

An array pattern can consume its elements, but `let first = array[0]` cannot move a `String` out through indexing. For a vector, choose `remove`, `swap_remove`, `pop`, or consuming iteration according to the operation you intend.

**Taking a value while leaving the owner valid**:

```rust
fn main() {
    let mut data = Some(String::from("hello"));

    let value = data.take();
    println!("{:?}", value);

    use std::mem;

    let mut data2 = String::from("world");
    let old_value = mem::replace(&mut data2, String::from("new"));
    println!("Old: {}, New: {}", old_value, data2);

    let mut data3 = vec![1, 2, 3];
    let taken = mem::take(&mut data3);
    println!("Taken: {:?}, Remaining: {:?}", taken, data3);
}
```

### Move and Panic Interactions <a href="#move-and-panic-interactions-" class="header-link">🔗</a>

Suppose processing a ticket panics after taking one field. With unwinding enabled, Rust drops initialized owned values along the path out; moving a field does not make it disappear from cleanup.

```rust
struct Resource(&'static str);

impl Drop for Resource {
    fn drop(&mut self) {
        println!("Dropping {}", self.0);
    }
}

fn main() {
    let data = (Resource("ticket"), Resource("receipt"));

    let result = std::panic::catch_unwind(|| {
        let _ticket = data.0;
        panic!("Printer failed");
    });

    println!("Caught panic: {}", result.is_err());
    println!("Still owned here: {}", data.1.0);
}
```

In editions 2021 and 2024, the closure captures the used tuple field precisely. The ticket drops during unwinding, the receipt remains available, and it drops at the end of `main`. The panic hook normally prints the panic even though it is caught.

`catch_unwind` catches unwinding panics, not aborting ones. It is not a replacement for `Result`, and it does not undo a charge already sent to a payment service. Its `UnwindSafe` bound is about inspecting state after a panic; adding `AssertUnwindSafe` blindly only silences that signal. [catch_unwind](https://doc.rust-lang.org/std/panic/fn.catch_unwind.html).

### Closure Ownership (FnOnce/Fn/FnMut) <a href="#closure-ownership-fnoncefnfnmut-" class="header-link">🔗</a>

Read the traits as permissions required to call a stored callback:

| Bound | What calling needs | Café example |
|---|---|---|
| `Fn()` | Shared access to the closure | Read a captured menu repeatedly |
| `FnMut()` | Exclusive access to the closure | Increment a captured counter |
| `FnOnce()` | Ownership of the closure | Hand away a captured receipt |

Every closure implements `FnOnce`; some can also implement `FnMut`, and some `Fn`. `FnOnce` means the API may consume it, not that it must be called exactly once.

```rust
fn call_fn<F: Fn()>(f: F) {
    f();
}

fn call_fn_mut<F: FnMut()>(f: &mut F) {
    f();
}

fn call_fn_once<F: FnOnce()>(f: F) {
    f();
}

fn main() {
    let data = vec![1, 2, 3];

    let print = || println!("{:?}", data);
    call_fn(print);
    call_fn(print);

    let mut counter = 0;
    let mut increment = || {
        counter += 1;
        println!("Count: {}", counter);
    };
    call_fn_mut(&mut increment);
    call_fn_mut(&mut increment);

    let consume = || drop(data);
    call_fn_once(consume);
}
```

**Trait Hierarchy**:

`Fn` extends `FnMut`, which extends `FnOnce`. A type implementing `Fn` satisfies all three bounds. In this example `print` can be passed by value twice because this particular closure captures a shared reference and is `Copy`. `Fn` alone does **not** imply `Copy`; for repeated calls to an owned non-`Copy` callback, pass `&callback` to `call_fn` or make the helper borrow it.

#### Important Note: `move` Closure Behavior with `Copy` Types <a href="#important-note-move-closure-behavior-with-copy-types-" class="header-link">🔗</a>

> When a `move` closure captures a `Copy` type, it captures a bitwise copy of the value. The original variable is **not moved** and remains fully accessible. This can be surprising because the original variable is still usable in its scope, even though the `move` keyword suggests ownership transfer.
>
> Example:
> ```rust
> fn main() {
>     let x = 42i32; // Copy type
>     let closure = move || println!("{}", x); // Captures a copy
>     println!("{}", x); // x still accessible!
> }
> ```
>
> This is by design, as `Copy` semantics mean the value is duplicated. For non-`Copy` types, `move` works as expected. For this integer, modifications inside the closure affect only the captured copy. Copying a shared reference still points to the same value; interior mutation through that reference can affect shared state.

### Iterator Ownership Patterns <a href="#iterator-ownership-patterns-" class="header-link">🔗</a>

For a `Vec<T>`, `.iter()` yields `&T`, `.iter_mut()` yields `&mut T`, and consuming the vector yields `T`. Think “read each order”, “edit each order”, and “hand each order to the kitchen”.

```rust
fn main() {
    let data = vec![String::from("a"), String::from("b")];

    for item in data.iter() {
        println!("{}", item);
    }
    println!("data still valid: {:?}", data);

    let mut data_mut = vec![String::from("x"), String::from("y")];
    for item in data_mut.iter_mut() {
        item.push_str("!");
    }
    println!("modified: {:?}", data_mut);

    for item in data {
        println!("{}", item);
    }
}
```

***

For a vector, `for item in &data` is the borrowed form and `for item in &mut data` is the exclusive form. `into_iter()` means consuming its **receiver**: when the receiver is `&Vec<T>`, it consumes a reference and yields references, not owned elements. Ask what type the receiver and item actually have. [IntoIterator](https://doc.rust-lang.org/std/iter/trait.IntoIterator.html).

## Part III: Advanced Borrowing <a href="#part-iii-advanced-borrowing-" class="header-link">🔗</a>

### Splitting Borrows and Field Sensitivity <a href="#splitting-borrows-and-field-sensitivity-" class="header-link">🔗</a>

The borrower needs exclusive access to the part it changes, not necessarily the entire object. Rust can distinguish named struct fields. For slices, use an API such as `split_at_mut` to establish that two regions do not overlap; two separate indexed `&mut data[i]` expressions do not generally establish this proof.

**Splitting Slices**:

```rust
fn main() {
    let mut data = vec![1, 2, 3, 4, 5, 6];

    let (left, right) = data.split_at_mut(3);

    left[0] = 10;
    right[0] = 40;

    println!("{:?}", data);
}
```

**Splitting Struct Fields**:

```rust
struct Point {
    x: i32,
    y: i32,
}

fn main() {
    let mut p = Point { x: 0, y: 0 };

    let x = &mut p.x;
    let y = &mut p.y;

    *x = 10;
    *y = 20;

    println!("Point: ({}, {})", p.x, p.y);
}
```

### Interior Mutability Deep Dive <a href="#interior-mutability-deep-dive-" class="header-link">🔗</a>

Interior mutability lets a type offer controlled mutation through `&self`. It changes where access is enforced; it does not give you permission to create overlapping exclusive references.

| Tool | How it controls access | Typical use |
|---|---|---|
| `Cell<T>` | Moves values in/out; `get` needs `T: Copy` | A small single-thread counter |
| `RefCell<T>` | Runtime borrow guards | A single-thread model shared by UI callbacks |
| `Mutex<T>` | A lock with an exclusive guard | State accessed by several threads |

`Cell` can hold non-`Copy` values too; use operations such as `replace` or `take` instead of `get`. `RefCell::borrow_mut` panics on conflict; `try_borrow_mut` returns an error. Neither `Cell` nor `RefCell` is `Sync`. [Interior mutability](https://doc.rust-lang.org/std/cell/index.html).

**Cell vs RefCell Comparison**:

```rust
use std::cell::{Cell, RefCell};

fn main() {
    let counter = Cell::new(0);
    let ref1 = &counter;
    let ref2 = &counter;

    ref1.set(ref1.get() + 1);
    ref2.set(ref2.get() + 1);

    println!("Counter: {}", counter.get());

    let data = RefCell::new(vec![1, 2, 3]);

    {
        let mut borrowed = data.borrow_mut();
        borrowed.push(4);
    }

    let borrowed = data.borrow();
    println!("{:?}", *borrowed);
}
```

The `Ref` and `RefMut` guards own the runtime borrow. Their destruction releases it. A borrow's last use and a guard's drop are different events: use a smaller block or `drop(guard)` before taking another conflicting borrow.

**Thread-Safe Interior Mutability**:

```rust
use std::sync::{Arc, Mutex};

fn main() {
    let counter = Arc::new(Mutex::new(0));

    let handles: Vec<_> = (0..3).map(|_| {
        let counter = Arc::clone(&counter);
        std::thread::spawn(move || {
            let mut num = counter.lock().unwrap();
            *num += 1;
        })
    }).collect();

    for handle in handles {
        handle.join().unwrap();
    }

    println!("Counter: {}", *counter.lock().unwrap());
}
```

### Coercion and Deref Magic <a href="#coercion-and-deref-magic-" class="header-link">🔗</a>

A function that accepts `&str` can read both a borrowed string slice and the text owned by a `String`. Deref coercions let the latter be viewed through the former without cloning it.

**Deref Coercion Chains**:

```rust
fn print_str(s: &str) {
    println!("{}", s);
}

fn main() {
    let owned = String::from("hello");
    print_str(&owned);

    let boxed = Box::new(String::from("world"));
    print_str(&boxed);
}
```

Deref coercion borrows the target; it does not transfer its ownership. Method calls also search through dereferences. This is useful for pointer-like wrappers, but implementing `Deref` on every business type can make its API surprising. [Deref's API implications](https://doc.rust-lang.org/std/ops/trait.Deref.html).

**Custom Deref Implementation**:

```rust
use std::ops::Deref;

struct MyBox<T>(T);

impl<T> MyBox<T> {
    fn new(value: T) -> Self {
        MyBox(value)
    }
}

impl<T> Deref for MyBox<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.0
    }
}

fn main() {
    let boxed = MyBox::new(String::from("Rust"));
    let len = boxed.len();
    println!("Length: {}", len);
}
```

### Variance and Lifetime Subtyping <a href="#variance-and-lifetime-subtyping-" class="header-link">🔗</a>

Here is the practical question: **if a reference is valid for a long time, may I use it where a shorter-lived reference is expected?** For a shared reference, yes. Promising less is safe. This direction is called covariance.

```rust
fn shorten<'long: 'short, 'short>(text: &'long str) -> &'short str {
    text
}

fn main() {
    let permanent: &'static str = "Today's menu";
    let view = shorten(permanent);
    println!("{view}");
}
```

`'long: 'short` means “`'long` outlives `'short`”. This never lengthens a reference to a local `String` or makes its owner live longer.

| Type | In lifetime `'a` | In type `T` |
|---|---|---|
| `&'a T` | Covariant | Covariant |
| `&'a mut T` | Covariant | Invariant |
| `*const T` | — | Covariant |
| `*mut T` | — | Invariant |
| `Cell<T>` | — | Invariant |
| `fn(T)` | — | Contravariant |
| `fn() -> T` | — | Covariant |

The surprising row is `&mut T`: you can shorten its **outer borrow**, often through a reborrow, but cannot arbitrarily substitute the **stored type**. If a writable slot intended to hold `&'static str` could be treated as a slot for a short local reference, a caller could put a dangling reference into it. Part IV makes this failure concrete. [Subtyping and variance](https://doc.rust-lang.org/nomicon/subtyping.html).

Contravariance reverses the direction for function inputs: a callback that handles a string borrowed for any lifetime can handle a static string too. A callback that accepts *only* static strings cannot serve every short-lived input. Think about what the caller is allowed to pass, rather than memorizing an arrow.

***

## Part IV: Lifetime Mastery <a href="#part-iv-lifetime-mastery-" class="header-link">🔗</a>

### Reading and Writing Lifetime Contracts

Post one explained how long values and borrows stay usable. Now we need to express a relationship at a function boundary. A café receipt parser can return a view into its input rather than allocate a new string:

```rust
fn customer_name<'text>(receipt: &'text str) -> &'text str {
    receipt.split(':').next().unwrap_or("")
}

fn main() {
    let receipt = String::from("Amrit: two coffees");
    let name = customer_name(&receipt);
    println!("Customer: {name}");
}
```

Read `'text` as “the period this input may be borrowed”. The returned view must fit within that period. The annotation does not extend the `String`'s life or schedule a destructor. If the owner disappears before the view's last use, the caller is rejected. Returning a new `String` is appropriate when the result must outlive that owner. [Lifetime annotations](https://doc.rust-lang.org/book/ch10-03-lifetime-syntax.html).

Often you omit the names. With one input lifetime, Rust assigns it to elided output references. For methods, an `&self` or `&mut self` receiver supplies the default output lifetime. With two independent input borrows and no receiver, you must explain the relationship. [Elision rules](https://doc.rust-lang.org/reference/lifetime-elision.html).

```rust
fn choose_label<'a>(first: &'a str, second: &'a str, use_first: bool) -> &'a str {
    if use_first { first } else { second }
}

fn main() {
    let menu = String::from("Coffee");
    {
        let special = String::from("Tea");
        let label = choose_label(&menu, &special, true);
        println!("{label}");
    }
}
```

Both inputs must be borrowable for the period in which `label` is used. They need not have been created together or be destroyed together. Even though this call passes `true`, the signature permits either input as the result; callers must respect that contract. If a result only borrows from the first input, give the second an independent lifetime rather than tying everything to `'a`.

A borrowed struct follows the same reasoning: `struct ReceiptView<'a> { name: &'a str }` holds a view whose use depends on an owner elsewhere. A struct holding a `String` owns its text and needs no reference lifetime for that field.

Distinguish `&'static str` from `T: 'static`. The former is a reference valid for the rest of the program. The latter says that `T` contains no borrows that expire sooner; an owned `String` satisfies it and can still be dropped a moment later. This distinction will matter when we spawn tasks.

### Higher-Ranked Trait Bounds (HRTB) <a href="#higher-ranked-trait-bounds-hrtb-" class="header-link">🔗</a>

A stored callback may receive a fresh borrowed receipt on each call. The caller should not have to choose one fixed lifetime for every receipt. `for<'a>` means the callback must work for **every** suitable input lifetime, with its output tied to that particular input.

**Basic HRTB Syntax**:

```rust
fn call_with_ref<F>(f: F)
where
    F: for<'a> Fn(&'a str) -> &'a str,
{
    let data = String::from("hello");
    let result = f(&data);
    println!("{}", result);
}

fn main() {
    call_with_ref(|s| s);
}
```

The bound says “given a receipt view, return a view valid for that same borrow”. An ordinary generic lifetime chosen outside this function would not necessarily fit a temporary created inside it. This is useful for reusable validators and parsers. [Higher-ranked bounds](https://doc.rust-lang.org/reference/trait-bounds.html#higher-ranked-trait-bounds).

**Using the same callback for different inputs**:

```rust
fn use_processor<P>(processor: P)
where
    P: for<'a> Fn(&'a str) -> &'a str,
{
    let data1 = String::from("first");
    let result1 = processor(&data1);
    println!("{}", result1);

    let data2 = String::from("second");
    let result2 = processor(&data2);
    println!("{}", result2);
}

fn main() {
    use_processor(|s| s);
}
```

### Variance Rules and Implications <a href="#variance-rules-and-implications-" class="header-link">🔗</a>

Let us make Part III's writable-slot problem concrete. Imagine a slot that promises to hold a permanent menu title. Allowing a short-lived daily special to be written into it would break that promise.

**Does not compile:**

```rust
fn replace_title<'a>(slot: &mut &'a str, replacement: &'a str) {
    *slot = replacement;
}

fn main() {
    let mut title: &'static str = "Menu";
    {
        let special = String::from("Friday special");
        replace_title(&mut title, &special); // Error: special is not 'static.
    }
    println!("{title}");
}
```

The outer `&mut` only needs to last for the call. But its inner type is explicitly `&'static str`; invariance prevents treating that slot as a slot for an arbitrary shorter reference. If the slot should contain temporary titles, change its contract and ensure each title owner remains alive for every use. Lifetime annotations cannot repair an invalid ownership plan.

Do not generalize this to “mutable references cannot be shortened”: reborrowing an `&mut T` for a shorter call is normal. Rust 1.98 also added a specific lifetime-shortening allowance during **unsizing coercions**, even inside some invariant positions. That separate coercion rule does not permit the dangling assignment above. [Rust 1.98 language changes](https://doc.rust-lang.org/releases.html#version-1980-2026-08-20).

### Self-Referential Structs and Pin <a href="#self-referential-structs-and-pin-" class="header-link">🔗</a>

Suppose a parsed order owns its text and also remembers where the customer's name appears. The easiest design stores a byte range and creates a borrow when asked:

```rust
use std::ops::Range;

struct ParsedOrder {
    text: String,
    customer: Range<usize>,
}

impl ParsedOrder {
    fn new(text: String) -> Self {
        let end = text.find(':').unwrap_or(text.len());
        Self { text, customer: 0..end }
    }

    fn customer(&self) -> &str {
        &self.text[self.customer.clone()]
    }
}

fn main() {
    let order = ParsedOrder::new(String::from("Amrit: coffee"));
    let moved_order = order;
    println!("{}", moved_order.customer());
}
```

The colon's index is a UTF-8 boundary, and the type exposes no method that changes the text. Moving the order keeps the range meaningful. For library code, keep these fields private so callers cannot invalidate that relationship.

**When an address really must stay fixed**

A raw pointer to an inline field can become invalid if the enclosing value moves. A pointer to a `String` **object** points to its inline bookkeeping, not directly to its heap text. Moving the enclosing struct relocates that object; moving the `String` alone normally keeps its text allocation.

`Pin` lets an API express a promise that a pointee stays at its location until its destruction. It does not discover self-references, repair pointers, or freeze all mutation. Most types implement `Unpin`, which allows safe code to move them even through a pinned pointer. `PhantomPinned` opts a type out of automatic `Unpin`.

```rust
use std::marker::PhantomPinned;
use std::pin::Pin;

struct AddressCard {
    name: String,
    _pin: PhantomPinned,
}

fn main() {
    let card = Box::pin(AddressCard {
        name: String::from("Kitchen printer"),
        _pin: PhantomPinned,
    });
    let before = card.as_ref().get_ref() as *const AddressCard;
    let moved_handle: Pin<Box<AddressCard>> = card;
    let after = moved_handle.as_ref().get_ref() as *const AddressCard;
    println!("Same address: {}", before == after);
    println!("{}", moved_handle.as_ref().get_ref().name);
}
```

Moving the owning **handle** is allowed; moving the pinned pointee out through safe APIs is restricted. This example teaches that distinction without dereferencing a raw self-pointer. An actual self-referential implementation must establish its internal pointers at the final address, keep fields encapsulated, and preserve the pinning and destruction contract through every method. Read the [standard library's Pin explanation and self-referential example](https://doc.rust-lang.org/std/pin/index.html) before writing that unsafe implementation. For parsed text, offsets often avoid the need entirely.

### Generic Associated Types (GATs) <a href="#generic-associated-types-gats-" class="header-link">🔗</a>

An ordinary iterator has one `Item` type for the whole iterator. A **lending iterator** can choose an item whose lifetime is tied to the current borrow of the iterator itself. That matters when each item temporarily exposes the iterator's own mutable state.

Imagine reviewing three consecutive sales at a time and adjusting a value. The windows overlap, so retaining two mutable windows at once would expose the same sales through two exclusive references. A GAT expresses the restriction:

```rust
trait LendingIterator {
    type Item<'a> where Self: 'a;
    fn next<'a>(&'a mut self) -> Option<Self::Item<'a>>;
}

struct WindowsMut<'data, T> {
    slice: &'data mut [T],
    width: usize,
    position: usize,
}

impl<'data, T> WindowsMut<'data, T> {
    fn new(slice: &'data mut [T], width: usize) -> Self {
        assert!(width > 0, "window width must be positive");
        Self { slice, width, position: 0 }
    }
}

impl<T> LendingIterator for WindowsMut<'_, T> {
    type Item<'a> = &'a mut [T] where Self: 'a;

    fn next<'a>(&'a mut self) -> Option<Self::Item<'a>> {
        let start = self.position;
        let end = start.checked_add(self.width)?;
        if end > self.slice.len() {
            return None;
        }
        self.position = start.checked_add(1)?;
        self.slice.get_mut(start..end)
    }
}

fn main() {
    let mut sales = [10, 20, 30, 40];
    let mut windows = WindowsMut::new(&mut sales, 3);
    while let Some(window) = windows.next() {
        window[0] += 1;
        println!("{window:?}");
    }
    println!("Final sales: {sales:?}"); // [11, 21, 30, 40]
}
```

Read `Item<'a>` as “the item I can lend for this call's borrow”. `Self: 'a` requires the iterator's contents to remain valid for that borrow. A second call cannot happen while the first window is still going to be used. Ordinary `Iterator::Item` cannot express this per-call borrowing relationship, so this is a separate trait and uses `while let` rather than a `for` loop.

An empty slice or a width larger than the input produces no window; zero width is rejected by the constructor. Checked arithmetic and safe slicing make bounds mistakes return `None` instead of manufacturing an invalid reference. [Generic associated types](https://doc.rust-lang.org/reference/items/associated-items.html#associated-types).

***

## Part V: Ownership in Practice <a href="#part-v-ownership-in-practice-" class="header-link">🔗</a>

### Graph Structures and Arena Allocation <a href="#graph-structures-and-arena-allocation-" class="header-link">🔗</a>

An order workflow may loop from “needs correction” back to “preparing”. A cycle in the **relationships** does not require a cycle in **ownership**. Let one graph own every node; edges store IDs that do not own their targets.

```rust
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct NodeId(usize);

#[derive(Debug)]
struct Node {
    name: String,
    neighbors: Vec<NodeId>,
}

struct Graph {
    nodes: HashMap<NodeId, Node>,
    next_id: usize,
}

impl Graph {
    fn new() -> Self {
        Self { nodes: HashMap::new(), next_id: 0 }
    }

    fn add_node(&mut self, name: &str) -> NodeId {
        let id = NodeId(self.next_id);
        self.next_id = self.next_id.checked_add(1).expect("node IDs exhausted");
        self.nodes.insert(id, Node { name: name.into(), neighbors: Vec::new() });
        id
    }

    fn add_edge(&mut self, from: NodeId, to: NodeId) -> bool {
        if !self.nodes.contains_key(&to) {
            return false;
        }
        let Some(node) = self.nodes.get_mut(&from) else { return false };
        node.neighbors.push(to);
        true
    }
}

fn main() {
    let mut graph = Graph::new();
    let preparing = graph.add_node("preparing");
    let checking = graph.add_node("checking");
    graph.add_edge(preparing, checking);
    graph.add_edge(checking, preparing);
    if let Some(node) = graph.nodes.get(&preparing) {
        println!("{} -> {:?}", node.name, node.neighbors);
    }
}
```

Dropping `graph` drops its nodes even though edges form a cycle: the IDs do not keep anything alive. These IDs are meaningful only within the graph that issued them; this small design does not detect IDs accidentally mixed between graphs.

This is an owned graph with stable IDs, **not a generational arena**. An arena stores many objects under one storage owner. If it reuses deleted slots, a handle can pair a slot number with a generation so an old handle is rejected instead of silently naming a new node. Such a design also needs policies for generation exhaustion, cross-arena IDs, and edges to removed nodes. None of those properties follows merely from adding an unused `generation` field.

### Observer Patterns Without Cycles <a href="#observer-patterns-without-cycles-" class="header-link">🔗</a>

A café order board notifies display panels. The board should not keep a closed panel alive merely because it subscribed. Store `Weak` handles; the UI that owns each panel keeps the strong `Rc`. A strong-reference cycle can leak; a weak edge does not keep the inner value alive. [Weak references](https://doc.rust-lang.org/std/rc/struct.Weak.html).

**Weak References Pattern**:

```rust
use std::rc::{Rc, Weak};
use std::cell::RefCell;

trait Observer {
    fn notify(&self, message: &str);
}

struct Subject {
    observers: RefCell<Vec<Weak<dyn Observer>>>,
}

impl Subject {
    fn new() -> Self {
        Self {
            observers: RefCell::new(Vec::new()),
        }
    }

    fn subscribe(&self, observer: Weak<dyn Observer>) {
        self.observers.borrow_mut().push(observer);
    }

    fn notify_all(&self, message: &str) {
        let listeners: Vec<_> = {
            let mut registered = self.observers.borrow_mut();
            registered.retain(|weak| weak.strong_count() != 0);
            registered.iter().filter_map(Weak::upgrade).collect()
        }; // Release the RefCell borrow before calling application code.
        for observer in listeners {
            observer.notify(message);
        }
    }
}

struct ConcreteObserver {
    id: u32,
}

impl Observer for ConcreteObserver {
    fn notify(&self, message: &str) {
        println!("Observer {} received: {}", self.id, message);
    }
}

fn main() {
    let subject = Subject::new();

    {
        let observer1: Rc<dyn Observer> = Rc::new(ConcreteObserver { id: 1 });
        let observer2: Rc<dyn Observer> = Rc::new(ConcreteObserver { id: 2 });

        subject.subscribe(Rc::downgrade(&observer1));
        subject.subscribe(Rc::downgrade(&observer2));

        subject.notify_all("Event 1");
    }

    subject.notify_all("Event 2");
}
```

The temporary list keeps the current listeners alive during delivery and releases the registry borrow before a callback runs. A callback can subscribe without a borrow panic; the new listener starts with the next notification. Recursive notifications still need an application policy to avoid unbounded recursion. Weak ownership solves lifetime coupling, not every event-delivery problem.

### Plugin Architectures <a href="#plugin-architectures-" class="header-link">🔗</a>

A till may run several receipt formatters through one interface. The registry owns each formatter in a `Box`; calls borrow it through a trait object. `Send + Sync` asks implementations to permit thread transfer and shared cross-thread access. It does not spawn threads or make execution parallel.

**Trait Object Plugin System**:

```rust
trait Plugin: Send + Sync {
    fn name(&self) -> &str;
    fn execute(&self, input: &str) -> String;
}

struct PluginRegistry {
    plugins: Vec<Box<dyn Plugin>>,
}

impl PluginRegistry {
    fn new() -> Self {
        Self {
            plugins: Vec::new(),
        }
    }

    fn register(&mut self, plugin: Box<dyn Plugin>) {
        println!("Registered plugin: {}", plugin.name());
        self.plugins.push(plugin);
    }

    fn execute_all(&self, input: &str) {
        for plugin in &self.plugins {
            let result = plugin.execute(input);
            println!("{}: {}", plugin.name(), result);
        }
    }
}

struct UppercasePlugin;

impl Plugin for UppercasePlugin {
    fn name(&self) -> &str {
        "Uppercase"
    }

    fn execute(&self, input: &str) -> String {
        input.to_uppercase()
    }
}

struct ReversePlugin;

impl Plugin for ReversePlugin {
    fn name(&self) -> &str {
        "Reverse"
    }

    fn execute(&self, input: &str) -> String {
        input.chars().rev().collect()
    }
}

fn main() {
    let mut registry = PluginRegistry::new();

    registry.register(Box::new(UppercasePlugin));
    registry.register(Box::new(ReversePlugin));

    registry.execute_all("hello");
}
```

`Box<dyn Plugin>` here defaults to a `'static` trait-object bound: a plugin may own a `String`, but cannot hide a short-lived borrowed configuration under this type. A registry intended to borrow configuration would need that lifetime in its types. This is in-process polymorphism, not a stable ABI for loading arbitrary Rust dynamic libraries. Also, `chars().rev()` reverses Unicode scalar values, not whole user-perceived grapheme clusters. [Trait-object lifetimes](https://doc.rust-lang.org/reference/lifetime-elision.html#default-trait-object-lifetimes), [str::chars](https://doc.rust-lang.org/std/primitive.str.html#method.chars).

### Real-World Case Study: HTTP Router <a href="#real-world-case-study-http-router-" class="header-link">🔗</a>

A small routing table ties the pieces together: the router owns its handlers, a handler borrows the incoming request, and it returns an owned response. This is only a dispatcher; it does not parse HTTP or open a server socket.

```rust
use std::collections::HashMap;

type Handler = Box<dyn Fn(&Request) -> Response + Send + Sync>;

struct Request {
    path: String,
    method: String,
}

struct Response {
    status: u16,
    body: String,
}

struct Router {
    routes: HashMap<String, HashMap<String, Handler>>,
}

impl Router {
    fn new() -> Self {
        Self {
            routes: HashMap::new(),
        }
    }

    fn route<F>(&mut self, method: &str, path: &str, handler: F)
    where
        F: Fn(&Request) -> Response + Send + Sync + 'static,
    {
        self.routes.entry(method.to_owned()).or_default()
            .insert(path.to_owned(), Box::new(handler));
    }

    fn handle(&self, request: &Request) -> Response {
        let handler = self.routes.get(request.method.as_str())
            .and_then(|paths| paths.get(request.path.as_str()));
        if let Some(handler) = handler {
            handler(request)
        } else {
            Response {
                status: 404,
                body: "Not Found".to_string(),
            }
        }
    }
}

fn main() {
    let mut router = Router::new();

    router.route("GET", "/hello", |_req| Response {
        status: 200,
        body: "Hello, World!".to_string(),
    });

    router.route("POST", "/echo", |req| Response {
        status: 200,
        body: req.path.clone(),
    });

    let req = Request {
        path: "/hello".to_string(),
        method: "GET".to_string(),
    };

    let res = router.handle(&req);
    println!("Status: {}, Body: {}", res.status, res.body);
}
```

***

The router stores owned route keys once and looks them up with borrowed `&str` values; no lookup clone is needed. The response owns its body because it may outlive the request. A handler can use `move` to own configuration and still implement `Fn` if it only reads it. Registering the same method/path replaces the earlier handler. [HashMap borrowed lookup](https://doc.rust-lang.org/std/collections/struct.HashMap.html#method.get).

## Part VI: Unsafe and Ownership <a href="#part-vi-unsafe-and-ownership-" class="header-link">🔗</a>

### Raw Pointer Ownership Conventions <a href="#raw-pointer-ownership-conventions-" class="header-link">🔗</a>

A raw pointer is an address with a type, not an ownership policy. Creating or copying one can be safe; dereferencing it requires you to establish validity, alignment, initialization, and permitted access. `unsafe` does not switch off the ordinary borrow checker or make invalid memory access acceptable.

A pointer from `Vec::as_ptr` borrows no ownership: the vector remains responsible for its allocation, and destruction or reallocation can invalidate the pointer. `Box::into_raw` instead transfers cleanup responsibility to the caller. [Raw pointers](https://doc.rust-lang.org/std/primitive.pointer.html), [Box::from_raw](https://doc.rust-lang.org/std/boxed/struct.Box.html#method.from_raw).

**Ownership Conventions**:

```rust
fn main() {
    let data = vec![1, 2, 3];
    let ptr = data.as_ptr();

    // SAFETY: data is alive, nonempty, and has not been mutated or reallocated.
    unsafe {
        println!("First element: {}", *ptr);
    }

    let boxed = Box::new(42);
    let raw = Box::into_raw(boxed);

    // SAFETY: raw came from this Box; this is its one and only reclamation.
    unsafe {
        let _reclaimed = Box::from_raw(raw);
    }
}
```

Never call `Box::from_raw` on the vector's element pointer. It did not come from an independently owned `Box<T>` allocation. “It is non-null” is nowhere near enough evidence.

**A linked list that does not need unsafe**:

```rust
struct Node<T> {
    value: T,
    next: Option<Box<Node<T>>>,
}

struct LinkedList<T> {
    head: Option<Box<Node<T>>>,
}

impl<T> LinkedList<T> {
    fn new() -> Self {
        Self { head: None }
    }

    fn push(&mut self, value: T) {
        let new_node = Box::new(Node {
            value,
            next: self.head.take(),
        });
        self.head = Some(new_node);
    }

    fn pop(&mut self) -> Option<T> {
        self.head.take().map(|node| {
            self.head = node.next;
            node.value
        })
    }
}

fn main() {
    let mut list = LinkedList::new();
    list.push(1);
    list.push(2);
    list.push(3);

    while let Some(value) = list.pop() {
        println!("{}", value);
    }
}
```

`take()` leaves a valid empty head while ownership moves. The list can be manipulated entirely in safe Rust. Its default destruction recursively drops remaining links, which can overflow the stack for a very long list; draining it iteratively needs deliberate design. See the later [linked-list post](/rust/concepts/2025/05/07/rust-linked-list.html) for a separate collection-focused discussion.

### Building Safe Abstractions <a href="#building-safe-abstractions-" class="header-link">🔗</a>

A safe API must remain memory-safe for **every input its safe callers can supply**, not just the input in `main`. To see a manageable proof, revisit splitting a mutable slice. In application code, use the standard library's `split_at_mut`; the following implementation is a study of its safety boundary.

```rust
fn split_for_study<T>(items: &mut [T], middle: usize) -> (&mut [T], &mut [T]) {
    let len = items.len();
    assert!(middle <= len, "split point exceeds length");
    let pointer = items.as_mut_ptr();

    // SAFETY: the original slice supplies aligned, initialized storage.
    // Both ranges stay within that slice and do not overlap.
    // Their returned lifetimes are tied to the original exclusive borrow.
    unsafe {
        (
            std::slice::from_raw_parts_mut(pointer, middle),
            std::slice::from_raw_parts_mut(pointer.add(middle), len - middle),
        )
    }
}

fn main() {
    let mut orders = [10, 20, 30, 40];
    let (left, right) = split_for_study(&mut orders, 2);
    left[0] += 1;
    right[0] += 1;
    println!("{orders:?}"); // [11, 20, 31, 40]
}
```

The check also prevents subtraction underflow. Splitting at zero or at the length returns an empty side. Empty slices and zero-sized elements still require non-null, aligned pointers; deriving them from a valid slice preserves that requirement. No allocation is created or freed here. [Borrow splitting](https://doc.rust-lang.org/nomicon/borrow-splitting.html), [raw mutable slice requirements](https://doc.rust-lang.org/std/slice/fn.from_raw_parts_mut.html).

A raw `Vec` implementation is a much larger proof: zero-sized types, allocation failure, capacity/layout limits, which elements are initialized, and what happens if an element's destructor panics. The earlier version of this post's small `MyVec` did not satisfy that contract. Use `Vec<T>` for storage while learning these concerns separately. [Zero-sized allocation pitfalls](https://doc.rust-lang.org/nomicon/vec/vec-zsts.html), [allocation failure handling](https://doc.rust-lang.org/std/alloc/fn.alloc.html).

### FFI Ownership Patterns <a href="#ffi-ownership-patterns-" class="header-link">🔗</a>

A C caller cannot see Rust's borrow checker. The interface must document who creates an object, which calls merely inspect it, and which call consumes it. Here a small point is created in Rust and must be destroyed in Rust exactly once.

```rust
#[repr(C)]
pub struct Point {
    x: f64,
    y: f64,
}

// SAFETY: this standalone example reserves these symbol names.
// A library must ensure they are unique in its final linked program.
#[unsafe(no_mangle)]
pub extern "C" fn ownership_blog_create_point(x: f64, y: f64) -> *mut Point {
    Box::into_raw(Box::new(Point { x, y }))
}

/// # Safety
/// A non-null pointer must be an unreclaimed result of create_point above.
/// No outstanding use may race with destruction; no access is allowed afterward.
#[unsafe(no_mangle)] // SAFETY: same symbol-uniqueness obligation as above.
pub unsafe extern "C" fn ownership_blog_destroy_point(pointer: *mut Point) {
    if !pointer.is_null() {
        // SAFETY: the caller transfers the one remaining ownership obligation.
        unsafe { drop(Box::from_raw(pointer)); }
    }
}

/// # Safety
/// A non-null pointer must point to a live, aligned, initialized Point.
/// It must remain readable without concurrent mutation throughout this call.
#[unsafe(no_mangle)] // SAFETY: same symbol-uniqueness obligation as above.
pub unsafe extern "C" fn ownership_blog_point_distance(pointer: *const Point) -> f64 {
    if pointer.is_null() {
        return f64::NAN; // This small interface's explicit invalid-input result.
    }
    // SAFETY: validity and shared access are required from the caller.
    let point = unsafe { &*pointer };
    point.x.hypot(point.y)
}

fn main() {
    let pointer = ownership_blog_create_point(3.0, 4.0);
    // SAFETY: pointer is live, exclusively controlled here, and destroyed once.
    unsafe {
        println!("Distance: {}", ownership_blog_point_distance(pointer)); // 5
        ownership_blog_destroy_point(pointer);
    }
}
```

Null handling is just one branch of the API contract; it cannot detect a stale pointer or a second destruction. A C caller must not use C's `free` for this Rust-owned allocation. Making the consuming and dereferencing functions `unsafe` also prevents **safe Rust callers** from invoking them without acknowledging these obligations. [Box ownership transfer](https://doc.rust-lang.org/std/boxed/struct.Box.html#method.from_raw).

Edition 2024 requires `#[unsafe(no_mangle)]`: the symbol name itself has a global safety obligation. That is separate from an `unsafe fn`'s caller contract and an `unsafe` block's implementation proof. `repr(C)` governs layout, not ownership. A real interface also specifies error reporting and prevents unwinding across a non-unwinding C boundary. [Unsafe attributes](https://doc.rust-lang.org/edition-guide/rust-2024/unsafe-attributes.html), [unsafe operations in unsafe functions](https://doc.rust-lang.org/edition-guide/rust-2024/unsafe-op-in-unsafe-fn.html).

### ManuallyDrop and mem::forget <a href="#manuallydrop-and-memforget-" class="header-link">🔗</a>

`ManuallyDrop<T>` suppresses automatic destruction of its contained `T`; it does not make an uninitialized or already destroyed `T` valid. `mem::forget` consumes a value without running its destructor. Neither is a normal way to “fix” an ownership error. [ManuallyDrop](https://doc.rust-lang.org/std/mem/struct.ManuallyDrop.html), [forget](https://doc.rust-lang.org/std/mem/fn.forget.html).

**ManuallyDrop Usage**:

```rust
use std::mem::ManuallyDrop;

struct Resource {
    id: u32,
}

impl Drop for Resource {
    fn drop(&mut self) {
        println!("Dropping resource {}", self.id);
    }
}

fn main() {
    {
        let r = Resource { id: 1 };
    }

    {
        let mut r = ManuallyDrop::new(Resource { id: 2 });
        unsafe {
            // SAFETY: the value is initialized, dropped once, and never used again.
            ManuallyDrop::drop(&mut r);
        }
    }

    {
        let r = Resource { id: 3 };
        std::mem::forget(r);
    }
}
```

***

The first two blocks print their resource IDs; the third does not. If that last resource owned a heap allocation, file handle, or lock, forgetting it could leak the resource or keep it held. Safe Rust permits leaks. Therefore, an unsafe abstraction cannot rely on its destructor running to make earlier unsafe operations sound. For ordinary handoff, prefer ownership-taking APIs such as `Box::into_raw` with a documented reclamation path.

## Part VII: Async Ownership <a href="#part-vii-async-ownership-" class="header-link">🔗</a>

### Send and Sync Deep Dive <a href="#send-and-sync-deep-dive-" class="header-link">🔗</a>

A café background job raises two separate questions: may this value travel to another worker thread, and may several threads access it through shared references?

- `T: Send` means ownership of a `T` may safely cross a thread boundary.
- `T: Sync` means `&T` is `Send`: shared references to `T` may safely cross that boundary.

These are usually derived automatically from a type's fields. `Rc<T>` is neither `Send` nor `Sync`; its reference counts are not synchronized. `Arc<T>` synchronizes ownership counts, but does not add synchronization to `T`. In particular, `Arc<RefCell<T>>` is not a substitute for `Arc<Mutex<T>>`. [Send](https://doc.rust-lang.org/std/marker/trait.Send.html), [Sync](https://doc.rust-lang.org/std/marker/trait.Sync.html), [Arc thread safety](https://doc.rust-lang.org/std/sync/struct.Arc.html#thread-safety).

```rust
use std::sync::Arc;

fn main() {
    let menu = Arc::new(String::from("Coffee and tea"));
    let worker_menu = Arc::clone(&menu);
    let worker = std::thread::spawn(move || println!("{worker_menu}"));
    println!("Till still has: {menu}");
    worker.join().unwrap();
}
```

`Arc::clone` creates another owning handle to the same string; it does not duplicate the string contents. The value is destroyed after the final strong owner goes away.

**Markers and automatic traits**

A `PhantomData<*const ()>` field prevents automatic `Send` **and** `Sync`. A `PhantomData<Cell<()>>` field prevents `Sync`, but permits `Send`. These markers are useful when a type's real obligations are not fully visible in its stored fields. They are not explicit negative implementations; general `impl !Send for MyType` syntax remains unstable in Rust 1.98.1. Do not add manual unsafe trait implementations just to silence a compiler error. [Negative impls](https://doc.rust-lang.org/unstable-book/language-features/negative-impls.html).

### Lifetime Challenges in Async <a href="#lifetime-challenges-in-async-" class="header-link">🔗</a>

`tokio::spawn` may keep a task alive after the function that spawned it returns. Therefore both its future and output must be `Send + 'static`. The `'static` bound prevents dependence on short-lived external borrows; `Send` separately permits movement between worker threads. Neither bound says the value must actually live forever. [tokio::spawn](https://docs.rs/tokio/latest/tokio/task/fn.spawn.html).

**Does not compile — the task borrows its caller's local string:**

```rust
#[tokio::main]
async fn main() {
    let order = String::from("Two coffees");
    tokio::spawn(async {
        println!("Preparing {order}");
    }).await.unwrap();
}
```

**Move the owned value into the task:**

```rust
#[tokio::main]
async fn main() {
    let order = String::from("Two coffees");
    tokio::spawn(async move {
        println!("Preparing {order}");
    }).await.unwrap();
}
```

The task now owns its `String`, so it no longer borrows the caller's stack. Moving an `&str` into a task would only move/copy the reference, not acquire its text. If both caller and task need the text, choose a deliberate owned clone or shared `Arc` according to their actual needs.

Awaiting a `JoinHandle` immediately does not relax `spawn`'s signature. Use a directly awaited future when independent spawning is unnecessary. [Tokio's ownership explanation](https://tokio.rs/tokio/tutorial/spawning).

### Scoped Tasks and Non-Static Borrows <a href="#scoped-tasks-and-non-static-borrows-" class="header-link">🔗</a>

To do two asynchronous operations while borrowing local orders, keep their futures inside the current task. `tokio::join!` polls them concurrently; it does not create independent spawned tasks or promise CPU parallelism.

```rust
async fn inspect(order: &str) -> usize {
    tokio::task::yield_now().await;
    order.len()
}

#[tokio::main]
async fn main() {
    let first = String::from("Coffee");
    let second = String::from("Tea");
    let (first_size, second_size) = tokio::join!(inspect(&first), inspect(&second));
    println!("Byte lengths: {first_size}, {second_size}");
    println!("Still owned here: {first}, {second}");
}
```

Both local strings outlive the borrowed operations. If this enclosing future is dropped, its contained branch futures are dropped too; completed external side effects are not undone. A blocking operation in one branch can stop the other from progressing because the branches share a task. [join!](https://docs.rs/tokio/latest/tokio/macro.join.html).

The heading's key distinction is **scoped work versus independently spawned work**. Standard threads offer `std::thread::scope` for genuine borrowed thread spawning. Tokio's ordinary `spawn` does not become scoped just because you later await its handle. Dropping a `JoinHandle` detaches its task rather than cancelling it. `spawn_local` permits non-`Send` futures, but still requires `'static`. [Thread scopes](https://doc.rust-lang.org/std/thread/fn.scope.html), [JoinHandle](https://docs.rs/tokio/latest/tokio/task/struct.JoinHandle.html), [spawn_local](https://docs.rs/tokio/latest/tokio/task/fn.spawn_local.html).

***

## Part VIII: Performance and Optimization <a href="#part-viii-performance-and-optimization-" class="header-link">🔗</a>

### Copy-on-Write Patterns (Cow) <a href="#copy-on-write-patterns-cow-" class="header-link">🔗</a>

`Cow` means “borrow the original when possible; own a replacement when necessary”. A receipt sanitizer can return the input unchanged without copying its text, but return an owned string when it changes wording.

```rust
use std::borrow::Cow;

fn process_data(input: Cow<'_, str>) -> Cow<'_, str> {
    if input.contains("bad") {
        Cow::Owned(input.replace("bad", "good"))
    } else {
        input
    }
}

fn main() {
    let original = "hello world";
    let result = process_data(Cow::Borrowed(original));
    println!("{}", result);

    let original2 = "bad data";
    let result2 = process_data(Cow::Borrowed(original2));
    println!("{}", result2);
}
```

`Cow::to_mut` gives access to an owned form, cloning borrowed data first if necessary. `into_owned` moves an already owned value out or clones a borrowed one. The sanitizer above uses `replace`, which constructs a new string whenever the replacement branch is taken, even if its input was already owned. `Cow` makes ownership flexible; it does not automatically optimize every operation. [Cow](https://doc.rust-lang.org/std/borrow/enum.Cow.html).

**Cow with Collections**:

```rust
use std::borrow::Cow;

fn deduplicate<'a>(data: &'a [i32]) -> Cow<'a, [i32]> {
    let mut seen = std::collections::HashSet::new();
    let mut has_duplicates = false;

    for &item in data {
        if !seen.insert(item) {
            has_duplicates = true;
            break;
        }
    }

    if has_duplicates {
        let mut unique: Vec<i32> = data.to_vec();
        seen.clear();
        unique.retain(|item| seen.insert(*item));
        Cow::Owned(unique)
    } else {
        Cow::Borrowed(data)
    }
}

fn main() {
    let no_dupes = vec![1, 2, 3, 4];
    let result1 = deduplicate(&no_dupes);
    println!("No dupes: {:?}", result1);

    let has_dupes = vec![1, 2, 1, 3, 2];
    let result2 = deduplicate(&has_dupes);
    println!("Has dupes: {:?}", result2);
}
```

The result preserves the first occurrence of each number: `[1, 2, 1, 3, 2]` becomes `[1, 2, 3]`. `Vec::dedup` alone removes only **adjacent** duplicates. Also notice the cost of detection: the `HashSet` can allocate even when the function returns `Cow::Borrowed`. Avoiding an output copy is not the same as avoiding all allocation. [Vec::dedup](https://doc.rust-lang.org/std/vec/struct.Vec.html#method.dedup).

### Memory Locality Strategies <a href="#memory-locality-strategies-" class="header-link">🔗</a>

A kitchen that reads every order in sequence may benefit from storing order records next to each other. `Vec<Data>` stores its elements contiguously; `Vec<Box<Data>>` stores contiguous box handles whose pointees live in separate allocations. The extra pointer chase can matter, but its effect must be measured for the workload.

**Vec vs Vec<Box>**:

```rust
#[derive(Clone, Copy)]
struct Data {
    values: [u64; 8],
}

fn sum_contiguous() {
    let data: Vec<Data> = (0..1000)
        .map(|i| Data { values: [i; 8] })
        .collect();

    let sum: u64 = data.iter().map(|d| d.values[0]).sum();
    println!("Contiguous sum: {}", sum);
}

fn sum_boxed() {
    let data: Vec<Box<Data>> = (0..1000)
        .map(|i| Box::new(Data { values: [i; 8] }))
        .collect();

    let sum: u64 = data.iter().map(|d| d.values[0]).sum();
    println!("Boxed sum: {}", sum);
}

fn main() {
    sum_contiguous();
    sum_boxed();
}
```

These functions produce the same sum; they are **not benchmarks**. Boxing can be useful when stable pointee addresses or small movable handles matter. Contiguity can help sequential scans. Allocation, element size, access order, and compiler optimization all affect the result; measure before claiming a speedup.

### Cache-Conscious Design <a href="#cache-conscious-design-" class="header-link">🔗</a>

This layout stores each particle's position and velocity together: an **array of structs**. It is convenient when an update uses all those fields. If a program scans only one field across millions of particles, separate arrays for each field may use memory bandwidth more effectively. That alternative is a **struct of arrays**; it also makes keeping all arrays in step your responsibility.

```rust
#[derive(Clone, Copy)]
struct Particle {
    position_x: f32,
    position_y: f32,
    velocity_x: f32,
    velocity_y: f32,
}

struct ParticlesAoS {
    particles: Vec<Particle>,
}

fn update_aos(particles: &mut ParticlesAoS) {
    for particle in &mut particles.particles {
        particle.position_x += particle.velocity_x;
        particle.position_y += particle.velocity_y;
    }
}

fn main() {
    let mut particles = ParticlesAoS {
        particles: vec![Particle {
            position_x: 0.0,
            position_y: 0.0,
            velocity_x: 1.0,
            velocity_y: 1.0,
        }; 100],
    };

    update_aos(&mut particles);
    println!("Updated {} particles", particles.particles.len());
}
```

***

## Part IX: Anti-Patterns and Debugging <a href="#part-ix-anti-patterns-and-debugging-" class="header-link">🔗</a>

### Common Ownership Mistakes <a href="#common-ownership-mistakes-" class="header-link">🔗</a>

A clone is not automatically a mistake. Cloning a small string to give a task independence can be simpler than coordinating shared ownership. The mistake is adding copies without deciding which value actually needs another owner.

**Anti-Pattern 1: Unnecessary intermediate clones**:

```rust
fn bad_example(data: &str) -> String {
    let owned = data.to_string();
    let copy = owned.clone();
    copy.to_uppercase()
}

fn good_example(data: &str) -> String {
    data.to_uppercase()
}

fn main() {
    let input = "hello";
    println!("{}", bad_example(input));
    println!("{}", good_example(input));
}
```

**Anti-Pattern 2: Reaching for `Arc<Mutex<T>>` before deciding who owns the work**

The following use is legitimate: several threads intentionally update one collection. If workers can instead own separate inputs and return separate outputs, that design may need no shared mutable collection. Use the lock when sharing is part of the problem, not merely because a borrow was difficult.

```rust
use std::sync::{Arc, Mutex};

fn actually_need_sharing() {
    let shared = Arc::new(Mutex::new(vec![1, 2, 3]));

    let handles: Vec<_> = (0..3).map(|_| {
        let data = Arc::clone(&shared);
        std::thread::spawn(move || {
            let mut guard = data.lock().unwrap();
            guard.push(42);
        })
    }).collect();

    for h in handles {
        h.join().unwrap();
    }
}

fn main() {
    actually_need_sharing();
}
```

### Refactoring Strategies <a href="#refactoring-strategies-" class="header-link">🔗</a>

**Strategy 1: Express what the function needs**

If a function only reads numbers during the call, take `&[i32]`; if it keeps or consumes the vector, take `Vec<i32>`. Neither form is universally better. The signature tells a caller which responsibility is being transferred.

```rust
fn process_v1(data: Vec<i32>) -> i32 {
    data.iter().sum()
}

fn process_v2(data: &[i32]) -> i32 {
    data.iter().sum()
}

fn main() {
    let numbers = vec![1, 2, 3];

    let sum = process_v2(&numbers);
    println!("Sum: {}, Data: {:?}", sum, numbers);
}
```

### Debugging the Borrow Checker <a href="#debugging-the-borrow-checker-" class="header-link">🔗</a>

Before changing code, locate the owner, the first borrow or move, the conflicting operation, and the later use that keeps the borrow relevant. Then change the ownership plan: shorten a borrow, split independent fields, move a value to its true owner, or intentionally clone it. Adding `'static` or `RefCell` without a reason usually changes the contract instead of explaining the error.

**Understanding Error Messages**:

```rust
fn demonstrate_errors() {
    let mut data = vec![1, 2, 3];

    let first = &data;
    // data.push(4); // ERROR: cannot borrow as mutable while immutable borrow exists
    println!("{}", first.len());
    data.push(4); // Valid now: the shared borrow has no later use.

    let vec1 = vec![1, 2, 3];
    let vec2 = vec1;
    // println!("{:?}", vec1); // ERROR: value used after move
    println!("{:?}", vec2);
}

fn main() {
    demonstrate_errors();
}
```

***

## Part X: Advanced Topics <a href="#part-x-advanced-topics-" class="header-link">🔗</a>

### RPIT Capture Rules and Lifetime Control <a href="#rpit-capture-rules-and-lifetime-control-" class="header-link">🔗</a>

“Return-position `impl Trait`”, or RPIT, hides the concrete return type behind a trait. For a parser returning an iterator, the public contract must also say which input lifetimes that hidden type may depend on.

In edition 2024, RPIT captures all in-scope generic parameters by default. Use `use<...>` to specify the capture set. A bound such as `+ 'a` instead says the hidden type outlives `'a`; it is not a list of the only parameters captured. Precise capture was introduced in Rust 1.82 and is available across editions. [RPIT capture rules](https://doc.rust-lang.org/edition-guide/rust-2024/rpit-lifetime-capture.html).

```rust
fn letters<'a>(text: &'a str, _temporary_note: &str)
    -> impl Iterator<Item = char> + use<'a>
{
    text.chars()
}

fn main() {
    let text = String::from("Coffee");
    let note = String::from("Print in blue");
    let chars = letters(&text, &note);
    drop(note); // The returned iterator does not capture this borrow.
    println!("{}", chars.collect::<String>());
}
```

Here the iterator borrows `text`; its type does not depend on the note's lifetime. If the function is generic over types or constants too, current precise-capture syntax requires those in-scope type and const parameters in the `use<...>` list. The simple example has none.

**Do not unnecessarily tie a parser's lifetime to its output**

```rust
struct Processor;

impl Processor {
    fn process<'a>(&self, input: &'a str) -> impl Iterator<Item = char> + use<'a> {
        input.chars()
    }
}

fn main() {
    let input = String::from("Tea");
    let iter = {
        let processor = Processor;
        processor.process(&input)
    };
    println!("{}", iter.collect::<String>());
}
```

The processor is gone before iteration begins; the input remains alive. This is valid because the returned iterator depends on the input, not on the processor. Unlike closures, where capture describes stored values, RPIT capture describes which generic parameters a hidden **type** may depend on.

### Advanced Async Patterns <a href="#advanced-async-patterns-" class="header-link">🔗</a>

An async callback can borrow each input until its returned future completes. The `AsyncFn` family expresses that relationship, including async closures; it has been stable since Rust 1.85.

```rust
async fn inspect_twice<F>(inspect: F)
where
    F: for<'a> AsyncFn(&'a str) -> usize,
{
    let first = String::from("Coffee");
    let second = String::from("Tea");
    println!("{}", inspect(&first).await);
    println!("{}", inspect(&second).await);
}

#[tokio::main]
async fn main() {
    inspect_twice(async |order: &str| {
        tokio::task::yield_now().await;
        order.len()
    }).await;
}
```

The callback accepts a fresh borrow on each call, and the future may retain that borrow across its `await`. These calls are sequential and do not impose a spawned-task `'static` requirement. `AsyncFn`, `AsyncFnMut`, and `AsyncFnOnce` describe shared, exclusive, and consuming calls, respectively. [AsyncFn](https://doc.rust-lang.org/std/ops/trait.AsyncFn.html).

When a future must be `Send`, inspect both its initial captures and the state it retains while suspended. A captured `Rc` can make a future non-`Send` even before its first poll. An `Rc` created *inside* a future and confined to a block before any `await` need not make the future non-`Send`. Avoid holding blocking lock guards across `await`; release them in a short block or use an appropriate asynchronous synchronization design. [Tokio's Send explanation](https://tokio.rs/tokio/tutorial/spawning#send-bound).

Cancellation also has ownership consequences: dropping an ordinary future drops its captured state, but does not roll back already completed I/O. Dropping a spawned task's handle only detaches it, as Part VII explained. Decide explicitly whether unfinished work should continue, be cancelled, or be retried.

### Temporary Scope Behavior <a href="#temporary-scope-behavior-" class="header-link">🔗</a>

Two edition-2024 changes matter when temporary values own borrows or locks. First, edition 2024 normally ends a tail-expression temporary's scope at the end of that block, before its local variables are dropped. Separate temporary lifetime-extension rules still apply to certain expressions. This rule is not enabled merely by installing a new compiler. [Tail-expression changes](https://doc.rust-lang.org/edition-guide/rust-2024/temporary-tail-expr-scope.html).

**Edition 2024 — compiles; the same code is rejected in edition 2021:**

```rust
use std::cell::RefCell;

fn get_length() -> usize {
    let c = RefCell::new(String::from("hello"));
    c.borrow().len()
}

fn main() {
    println!("Length: {}", get_length());
}
```

`c.borrow()` returns a guard borrowing `c`. Edition 2024 drops that temporary guard before `c`. In earlier editions, assign the length to a local first, then return that number; the statement ends the temporary guard's scope.

**Edition 2024 — a failed `if let` drops its scrutinee temporary before `else`:**

Here “scrutinee” means the value examined by the pattern. The temporary read-lock guard is released before the write attempt on the `None` branch. With edition 2021, this example can deadlock trying to write while it still holds the read lock. [if-let changes](https://doc.rust-lang.org/edition-guide/rust-2024/temporary-if-let-scope.html).

```rust
use std::sync::RwLock;

fn check_and_update(value: &RwLock<Option<bool>>) {
    if let Some(x) = *value.read().unwrap() {
        println!("Value is {}", x);
    } else {
        let mut v = value.write().unwrap();
        if v.is_none() {
            *v = Some(true);
        }
    }
}

fn main() {
    let data = RwLock::new(None);
    check_and_update(&data);
    println!("Final value: {:?}", *data.read().unwrap());
}
```

### Scoped Borrowing in Complex Patterns <a href="#scoped-borrowing-in-complex-patterns-" class="header-link">🔗</a>

**The same edition distinction with `RefCell`**

In edition 2024, the empty case below releases the temporary shared-borrow guard before entering `else`. In edition 2021 it panics when `borrow_mut()` finds that guard still active. On the successful branch, the original borrow remains available for the borrowed element.

```rust
use std::cell::RefCell;

fn process_option(data: &RefCell<Vec<i32>>) {
    if let Some(first) = data.borrow().first() {
        let first_val = *first;
        println!("First element: {}", first_val);
    } else {
        data.borrow_mut().push(42);
    }
}

fn main() {
    let data = RefCell::new(vec![]);
    process_option(&data);
    println!("{:?}", data.borrow());
}
```

For code whose release point should be obvious in any edition, copy the needed value out in its own statement: `let first = data.borrow().first().copied();`, then match `first`. The guard ends at that statement; the `Option<i32>` owns its contents. The analogous lock pattern copies `*value.read().unwrap()` into a local before `if let`.

Do not replace `if let` with `match` and assume identical temporary scopes. A `match` scrutinee can keep its temporaries alive through the selected arm. Also, an `if let` successful branch can still hold its guard while you are inside that branch. Syntax and the path taken both matter.

**Slice patterns borrow their matched elements:**

```rust
fn main() {
    let data = vec![1, 2, 3];

    match data.as_slice() {
        [] => println!("Empty"),
        [first] => println!("Single: {}", first),
        [first, rest @ ..] => println!("First: {}, Rest: {:?}", first, rest),
    }
}
```

***

## Key Takeaways <a href="#key-takeaways-" class="header-link">🔗</a>

Mastery means predicting which design is valid before asking the compiler. Keep these distinctions clear:

1. **Ownership, access, and cleanup are separate questions.** Moving responsibility does not imply copying heap data. Dropping a borrow does not necessarily destroy its owner.
2. **A lifetime is a contract, not life support.** An annotation cannot keep a local value alive. Tie a result only to inputs it actually depends on.
3. **Use the smallest access the operation needs.** Read through `&T`, modify through `&mut T`, take ownership when keeping or consuming the value, and choose shared ownership deliberately.
4. **Interior mutability still enforces rules.** `RefCell` checks borrowing at runtime; thread-safe sharing needs the appropriate synchronization and trait bounds.
5. **Unsafe needs an explicit proof.** A compiling example is not evidence that arbitrary safe callers cannot cause undefined behavior.
6. **Async does not always require `'static`.** Independently spawned tasks often do; directly awaited borrowed work often does not. `Send` answers a different question.
7. **Performance is measured.** Know which operation allocates, clones, locks, or follows another pointer before choosing a design.

**Practise predicting the outcome**

For each change, explain the answer before compiling it:

- Add `Drop` to `Config`, then try moving its `api_key` field. Why does replacing the field with a valid value solve a different problem from cloning it?
- Change the printing `move` closure to `drop(data)`. Which call traits remain?
- Keep the first mutable window, call `next` again, then print the first. Which two borrows would overlap?
- Move a reference to a local `String` into a spawned task. Why does `move` not make the referent `'static`?
- Feed the deduplicator `[1, 2, 1]`. Why would `Vec::dedup` alone be wrong?
- Compile the final `RefCell` examples under both editions. Which behavior changes at compilation, and which changes at runtime?

The expected reasoning is in the corresponding sections. Use `rustc --explain E0505` for “move while borrowed” and `rustc --explain E0597` for “does not live long enough”; then redraw the ownership relationship instead of collecting compiler workarounds.

## Further Reading <a href="#further-reading-" class="header-link">🔗</a>

Use these as different kinds of authority: the Book teaches a model, the Reference specifies language behavior, the standard library documents API contracts, and the Nomicon explains the obligations behind unsafe implementations.

- [The Rust Book: ownership](https://doc.rust-lang.org/book/ch04-01-what-is-ownership.html)
- [The Rust Reference](https://doc.rust-lang.org/reference/)
- [The Rust 2024 Edition Guide](https://doc.rust-lang.org/edition-guide/rust-2024/index.html)
- [The Rustonomicon](https://doc.rust-lang.org/nomicon/)
- [Tokio: spawning and ownership](https://tokio.rs/tokio/tutorial/spawning)

The later posts on [Copy and Clone](/rust/concepts/2025/05/21/rust-copy-clone.html), [trait objects](/rust/concepts/2025/10/23/rust-dyn.html), and [Drop](/rust/concepts/2025/12/30/rust-drop.html) continue those individual topics. This post's job is to connect them through ownership.
