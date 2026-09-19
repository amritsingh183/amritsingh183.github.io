---
layout: post
title: "Mastering Variables, Constants and Lifetimes in Rust: A Complete Guide"
date: 2025-01-01 11:23:00 +0530
categories: rust concepts
last_updated: 2026-09-18
---
# Mastering Variables, Constants and Lifetimes in Rust: A Complete Guide

**What "lifetime" means in this post.** Every value in a Rust program is born somewhere and dies somewhere. A local variable is born at its `let` and dies at the closing brace of its block. A constant is baked into the program when it is compiled. A static is ready before `main` runs and is never destroyed. That everyday meaning of *lifetime*, how long a value lives and who may touch it while it lives, is what this post teaches. The `'a` annotations you write in function signatures describe how long *borrows* must stay valid, which is bounded by these value lifetimes; they get their own treatment in the [ownership and lifetimes guide](/rust/concepts/2025/02/09/rust-ownership.html), which assumes you have the mental model from here.

> Memory layout details (how many bytes a `String` takes, where padding goes) are in the [memory layout guide](/rust/concepts/2025/01/05/rust-mem-ref.html). You do not need it to read this post.

**Checked in September 2026 against Rust 1.98.1 (1.98.0 shipped in August 2026), edition 2024.** Every code block compiles and runs as shown, except the ones whose first line starts with ✗. Those are meant to fail so you can see the exact error the compiler gives.

**A version is not an edition.** Rust ships a new *version* every six weeks: 1.80, 1.81, ... 1.98. An *edition* (2015, 2018, 2021, 2024) is a setting in `Cargo.toml` that switches on a small set of language rule changes. A compiler supports every edition that existed when it shipped, so a new compiler still builds old-edition crates, while edition 2024 needs Rust 1.85 or newer. So `LazyLock` "needs Rust 1.80" (a version), while "references to `static mut` are an error" is an edition-2024 rule that even Rust 1.98 does not apply to a 2021-edition crate. This post says which one it means every time.

**The running example: a café till.** Throughout this post the code models a small café's cash register. It has a fixed tax rate (a constant), one ticket counter shared by every till (a static), orders that change while the customer is still deciding (mutable variables), receipts that are handed to customers (ownership moves), and an order board that everyone reads but only one person rewrites at a time (borrowing). When a rule feels abstract, ask "what would this mean at the café?"

## Foundation: One Rule Behind Everything

### Many readers or one writer

The café has an order board on the wall. Any number of baristas can *read* it at the same time. When somebody *rewrites* it, nobody else may read or write until they finish; otherwise a barista reads half an old order and half a new one.

Rust applies this rule to every piece of data. At any moment you may have either

- any number of **shared references** (`&T`), which can only read, or
- exactly one **exclusive reference** (`&mut T`), which can write,

but never both at once. The borrow checker enforces the rule before your program runs. The formal name is **aliasing XOR mutability**: data may be aliased (reachable under several names) or mutable (changed through a name), but not both at the same moment.

The rule is not only about threads. In a single thread it stops you from adding to a list while you are in the middle of walking through it, a classic crash in C++ and a classic surprise in Java and Python. It also lets the compiler optimise hard, because an exclusive reference really is exclusive: nothing else can change the data behind your back.

**Where raw pointers fit.** `*const T` and `*mut T` are exempt from the compile-time check and may alias freely; that is what `unsafe` code uses them for. What must still never happen is contradicting a *reference* that is alive: writing to memory that a live `&T` points at, or touching memory that a live `&mut T` owns through any other path. Do that and the program has undefined behaviour, whether the write came through a raw pointer or not. The one sanctioned exception is memory that a type has wrapped in `UnsafeCell`; that is the foundation of interior mutability, which Part Nine explains, and it is how a `Mutex` can change data behind a shared reference without breaking the rule. Keep this in mind for the `static mut` section in Part Three.

### How this guide is organized

You need basic Rust syntax (variables, functions, `if`, loops) and nothing else. The goal is a mental model good enough to write safe code with confidence, design types that cannot be misused, and read a borrow-checker error as information instead of an insult.

***

## Part One: Variable Bindings

### Immutable by default

A `let` creates a **binding**: a name attached to a value. Bindings are immutable unless you say otherwise. Once a receipt is printed its total should not change, and Rust makes that the default for every name.

```rust
fn main() {
    let printed_total = 4.50;
    println!("Receipt total: {printed_total}");
    // printed_total = 5.00; // ✗ error[E0384]: cannot assign twice to immutable variable
}
```

This is the compiler catching a whole family of "who changed this?" bugs for free.

### Mutable bindings

Add `mut` when the value genuinely needs to change, such as a running order total while the customer is still ordering:

```rust
fn main() {
    let mut running_total = 0.0;
    running_total += 3.00; // espresso
    running_total += 1.50; // extra shot
    println!("So far: {running_total:.2}");
}
```

`mut` lets you change the *value*, never the *type*. A binding that starts as a number stays a number:

```rust
// ✗ Does not compile. Error: E0308 mismatched types
fn main() {
    let mut quantity = 2;
    quantity = "two";
    println!("{quantity}");
}
```

### `mut` belongs to the name, not the value

`mut` on a binding is a promise about the *name*, not about the value. You can move a value out of an immutable binding into a mutable one and change it there; the value was never "immutable", only its first name was:

```rust
fn main() {
    let order = String::from("latte");   // immutable name
    let mut order = order;               // same value, mutable name
    order.push_str(", oat milk");
    println!("{order}");
}
```

This matters later, when we look at how values are destroyed.

### An immutable binding can be given its value later

You can declare a binding and fill it in on every path before it is used. It is still immutable: exactly one assignment happens, and the compiler checks that every branch performs it.

```rust
fn main() {
    let is_member = true;
    let discount;              // declared, not yet given a value
    if is_member {
        discount = 0.10;
    } else {
        discount = 0.0;
    }
    println!("Discount: {discount}");
}
```

Assign a second time and you get the same error as for any other immutable binding:

```rust
// ✗ Does not compile. Error: E0384 cannot assign twice to immutable variable
fn main() {
    let discount;
    discount = 0.10;
    discount = 0.15;
    println!("{discount}");
}
```

### Shadowing

Shadowing declares a *new* binding with the same name. The old one is hidden, not changed. Because it is a new binding, the type can change, which is exactly what you want when a quantity typed at the till arrives as text and must become a number:

```rust
fn main() {
    let quantity = "2";                                                   // text from the keypad
    let quantity: u32 = quantity.trim().parse().expect("a whole number"); // now a number
    let quantity = quantity * 2;                                          // double-shot day
    println!("{quantity}");                                               // 4
}
```

A shadow inside a block disappears at the block's end, like any other binding:

```rust
fn main() {
    let x = 5;
    let x = x + 1;            // shadow: 6
    {
        let x = x * 2;        // shadow again, only inside this block: 12
        println!("inner x: {x}");
    }
    println!("outer x: {x}"); // 6
}
```

### Shadowing: two things to know

**It hurts readability when overused.** Four transformations all called `config` bury what is happening; distinct names show the flow. Shadow when a value changes type or meaning once and the old binding is not needed again nearby.

**The shadowed value is not dropped.** Hiding a name does not end the value's life; it lives until the end of its scope. With a lock guard that means the lock stays held:

```rust
use std::sync::Mutex;

fn main() {
    let till = Mutex::new(100);
    let safe = Mutex::new(5000);
    let drawer = till.lock().unwrap();       // till open, lock held
    println!("till holds {}", *drawer);
    let drawer = safe.lock().unwrap();       // name reused for the safe; the till guard is hidden, NOT dropped
    println!("safe holds {}", *drawer);
    println!("till still locked? {}", till.try_lock().is_err()); // true
} // both guards die here, newest first
```

If you meant to release the lock, drop it explicitly with `drop(drawer)` before shadowing, or end the block.

### Scope and automatic cleanup

Every binding lives inside a scope, usually a pair of braces. When the scope ends, the value is destroyed and anything it owns (heap memory, a file handle, a lock) is released. There is no garbage collector and no manual `free`: the release happens at a point the compiler can name.

```rust
struct Drawer;

impl Drop for Drawer {
    fn drop(&mut self) {
        println!("drawer locked");
    }
}

fn main() {
    {
        let _morning = Drawer;
        println!("serving customers");
    } // _morning goes out of scope: "drawer locked" prints now
    println!("shop closed");
}
```

#### The Drop trait

`Drop` is how a type says what "being destroyed" means for it. The signature is fixed:

```rust
// fragment: the trait as declared in the standard library
pub trait Drop {
    fn drop(&mut self);
}
```

`Vec`, `Box` and `MutexGuard` implement it. `String` and `File` do not implement it themselves: they clean up through the fields they contain, which is the usual case. Most of your own types will not need it either, because a struct whose fields clean up after themselves needs nothing extra.

#### In what order do things drop?

Three rules cover almost everything:

1. Local variables drop in **reverse** order of declaration (last in, first out).
2. The fields of a struct drop in **declaration** order.
3. A temporary (a value that was never given a name) drops at the end of the statement that created it.

```rust
struct Noisy(&'static str);

impl Drop for Noisy {
    fn drop(&mut self) {
        println!("dropped {}", self.0);
    }
}

#[allow(dead_code)]
struct Order {
    cup: Noisy,
    lid: Noisy,
}

fn main() {
    let _first = Noisy("first local");
    let _second = Noisy("second local");
    let _order = Order { cup: Noisy("cup"), lid: Noisy("lid") };
    let count = { Noisy("temporary"); 1 }; // the temporary dies at the end of its statement
    println!("count = {count}, end of main");
}
// Output:
// dropped temporary
// count = 1, end of main
// dropped cup
// dropped lid
// dropped second local
// dropped first local
```

#### `let _ = value` is not `let _x = value`

The underscore pattern `_` binds nothing, which has three consequences people mix up:

- `let _ = existing_variable;` does **not** move the variable and does **not** drop it. Nothing happens.
- `let _ = make_something();` creates a temporary that nothing holds, so it is dropped **immediately**.
- `let _name = make_something();` binds a real name, so the value lives to the end of the scope.

```rust
struct Noisy(&'static str);

impl Drop for Noisy {
    fn drop(&mut self) {
        println!("dropped {}", self.0);
    }
}

fn main() {
    let receipt = Noisy("receipt");
    let _ = receipt;                       // nothing moves, nothing drops
    println!("receipt still here: {}", receipt.0);

    let _ = Noisy("temporary");            // dropped right away
    println!("after the temporary");

    let _kept = Noisy("kept");             // a real name: lives until main ends
    println!("end of main");
}
// Output:
// receipt still here: receipt
// dropped temporary
// after the temporary
// end of main
// dropped kept
// dropped receipt
```

The lock guard is where this bites so often that rustc refuses it outright. `let _ = till.lock()` would release the lock in the same statement and protect nothing, so the deny-by-default lint `let_underscore_lock` turns it into a compile error:

```rust
// ✗ Does not compile. Error: non-binding let on a synchronization lock (lint let_underscore_lock)
use std::sync::Mutex;

fn main() {
    let till = Mutex::new(0);
    let _ = till.lock().unwrap();   // the guard would be dropped in this very statement
    println!("locked? {}", till.try_lock().is_err());
}
```

Name the guard (`let _held = till.lock().unwrap();`) and it lives to the end of the scope, which is what you meant.

#### Why `drop` takes `&mut self`, and why you cannot call it

Destroying a value is its last use. The compiler consumes it and hands the destructor exclusive access (`&mut self`) so cleanup can change internal state, such as returning heap memory or closing a file descriptor. That the binding was not `mut` does not matter: `mut` belongs to the name, and by now the name is gone. Nothing else can observe the value at that moment, so the many-readers-or-one-writer rule is respected without any special exception. The signature is `&mut self` rather than `self` because taking the value by value would move it into `drop`, where it would need dropping again, forever. The [Drop post](/rust/2025/12/30/rust-drop.html) walks through that.

You cannot call the destructor yourself:

```rust
// ✗ Does not compile. Error: E0040 explicit use of destructor method
struct Drawer;

impl Drop for Drawer {
    fn drop(&mut self) {
        println!("drawer locked");
    }
}

fn main() {
    let d = Drawer;
    d.drop();
}
```

To end a value early, give it to `std::mem::drop`, available everywhere as plain `drop`. It takes ownership, so the value dies inside that call:

```rust
struct Drawer;

impl Drop for Drawer {
    fn drop(&mut self) {
        println!("drawer locked");
    }
}

fn main() {
    let d = Drawer;
    drop(d);                 // "drawer locked" prints here
    println!("after drop");
}
```

`Drop` is the only trait whose method the compiler runs for you when a value dies. Other traits get compiler support for their *syntax* (`Deref` for `*`, `IntoIterator` for `for`, `Fn` for calls, `Future` for `.await`), but none of them runs code automatically at the end of a scope, and you cannot write a trait that does.

### Edition 2024 changed when two kinds of temporaries die

A temporary is a value with no name, like the guard returned by `orders.borrow()` before you store it anywhere. Temporaries live until the end of the statement that made them. Two places used to keep them alive longer than anyone expected, and edition 2024 shortened both. This is the most practical "lifetimes" change in years, and it is a pure edition rule: the same compiler behaves differently depending on your crate's edition.

**Tail expressions.** In edition 2021 a temporary created in the final expression of a block lived *past the block*, until the enclosing statement ended, so it died after the block's own locals. A guard that borrowed a local therefore outlived the local:

```rust
use std::cell::RefCell;

fn count_orders() -> usize {
    let orders = RefCell::new(vec![String::from("latte")]);
    orders.borrow().len()   // edition 2021: error[E0597] `orders` does not live long enough
}                            // edition 2024: the temporary guard dies before `orders`, so this compiles

fn main() {
    println!("{}", count_orders());
}
```

**`if let`.** In edition 2021 the temporary from the `if let` scrutinee stayed alive through the `else` branch. With a `RefCell` that meant a panic; with a `Mutex` it meant a deadlock:

```rust
use std::cell::RefCell;

fn main() {
    let queue = RefCell::new(Vec::<String>::new());
    if let Some(next) = queue.borrow().first() {
        println!("next up: {next}");
    } else {
        // edition 2021: panics "already borrowed", because the read guard is still alive here
        // edition 2024: the read guard died before `else`, so this is fine
        queue.borrow_mut().push(String::from("latte"));
    }
    println!("{:?}", queue.borrow());
}
```

Separately from editions, Rust 1.98 (August 2026) tightened one more corner in every edition: temporaries created inside `assert_eq!` and `assert_ne!` now get their own scope, so a guard made inside the assertion is released as soon as the assertion finishes.

***

## Part Two: Constants

### Declaring constants

A `const` is a value the compiler works out completely while compiling. It must have a type annotation, and it can live at module level or inside a function:

```rust
const TAX_RATE: f64 = 0.05;            // 5 percent, the same at every till
const MAX_ITEMS_PER_ORDER: usize = 20;

fn main() {
    const OPENING_HOUR: u32 = 7;       // constants can be local too
    let subtotal = 10.0;
    println!("total {:.2}", subtotal * (1.0 + TAX_RATE));
    println!("max {MAX_ITEMS_PER_ORDER} items, opens at {OPENING_HOUR}");
}
```

Leave the type off and the compiler refuses:

```rust
// ✗ Does not compile. Error: missing type for `const` item
const MAX_ITEMS_PER_ORDER = 20;

fn main() {
    println!("{MAX_ITEMS_PER_ORDER}");
}
```

### What may go into a constant

The rule is simple: anything the compiler can finish evaluating before the program runs. That includes literals, arithmetic, arrays and structs, associated constants like `u32::MAX`, calls to functions marked `const fn`, and `const { ... }` blocks. It excludes anything that depends on the world at run time: the clock, files, the network, random numbers, and any ordinary function.

```rust
const SECONDS_PER_SHIFT: u32 = 60 * 60 * 8;        // arithmetic
const MAX_TICKET: u32 = u32::MAX;                   // an associated constant
const CUP_SIZES_ML: [u32; 3] = [240, 350, 470];     // an array
const BIGGEST_CUP_ML: u32 = 2u32.pow(9);            // a const fn call: 512
const SHOP_NAME: &str = "Corner Café";              // a string literal
const _: () = assert!(CUP_SIZES_ML.len() == 3);     // a compile-time check; the build fails if it is false

fn main() {
    println!("{SECONDS_PER_SHIFT} {MAX_TICKET} {CUP_SIZES_ML:?} {BIGGEST_CUP_ML} {SHOP_NAME}");
}
```

```rust
// ✗ Does not compile. Error: E0015 cannot call non-const associated function `SystemTime::now` in constants
use std::time::SystemTime;

const OPENED_AT: SystemTime = SystemTime::now();

fn main() {
    println!("{OPENED_AT:?}");
}
```

Two things worth knowing about `const fn`: most standard-library functions are **not** const, and a `const fn` is an ordinary function too, so you can also call it at run time with run-time values. Since Rust 1.83 a `const fn` may take `&mut` parameters, and a `const` may hold a reference to a `static`, as long as that static is immutable and has no interior mutability.

**Inline `const { }` blocks (Rust 1.79).** A block marked `const` is evaluated at compile time wherever it appears. The everyday use is building an array of something that is not `Copy`, because the block is evaluated afresh for every element:

```rust
use std::sync::Mutex;

// Eight independent queues, one per till. `Mutex` is not Copy, so `[Mutex::new(Vec::new()); 8]`
// is rejected; the `const { }` block is evaluated once per element instead.
static QUEUES: [Mutex<Vec<String>>; 8] = [const { Mutex::new(Vec::new()) }; 8];

fn main() {
    QUEUES[2].lock().unwrap().push(String::from("latte"));
    println!("till 2 has {} order(s)", QUEUES[2].lock().unwrap().len());
}
```

### Constants vs variables

| | `const` | `let` |
| :-- | :-- | :-- |
| **Mutability** | never; `mut` is not allowed | immutable unless `mut` |
| **Type annotation** | required | optional, usually inferred |
| **Value** | must be computable at compile time | computed at run time |
| **Memory** | no address of its own; every use gets a fresh copy of the value | one place in memory while it is alive |
| **Scope** | anywhere, including module level | the block it is declared in |

### Where a constant lives: a recipe, not a cake

A `const` is a recipe. Every place you use it, the compiler bakes a fresh copy of the value right there. A `static` (Part Three) is one cake on the counter that everybody points at. Three consequences follow:

1. Two uses of the same constant are **not guaranteed** to share an address. In practice rustc usually gives them the same read-only location, but nothing promises it.
2. For a constant whose value has no destructor and no interior mutability, taking `&SOME_CONST` gives you a `'static` reference anyway, because the compiler *promotes* the value into read-only memory. The same happens for `&5` or `&[1, 2, 3]` written directly in code.
3. If the value is large and used in many places, or if something outside Rust needs one stable address (C code, hardware), use a `static`.

```rust
static TICKET_PREFIX: &str = "CC-";
const TAX_RATE: f64 = 0.05;

fn main() {
    let a = &TICKET_PREFIX as *const &str;
    let b = &TICKET_PREFIX as *const &str;
    println!("static: same address? {}", a == b);   // always true

    let c = &TAX_RATE as *const f64;
    let d = &TAX_RATE as *const f64;
    println!("const: same address? {}", c == d);    // usually true, but not promised

    let forever: &'static f64 = &TAX_RATE;          // promoted into read-only memory
    println!("{forever}");
}
```

### It is the expression that must be constant, not the type

A constant may own heap-typed values, as long as the initializer itself is constant. `Vec::new()` and `String::new()` are `const fn` (they allocate nothing), so this works, and each use is a brand-new, independent value:

```rust
const EMPTY_ORDER: Vec<String> = Vec::new();   // Vec::new is a const fn

fn main() {
    let mut order_a = EMPTY_ORDER;             // a fresh Vec
    order_a.push(String::from("latte"));
    let order_b = EMPTY_ORDER;                 // another fresh Vec, still empty
    println!("{} {}", order_a.len(), order_b.len()); // 1 0
}
```

There is one limit hiding here: compile-time evaluation cannot hand you a heap allocation, so a constant can never own a heap buffer. `String::new()` passes because it allocates nothing. `String::from("welcome")` is rejected, first because `From::from` is not a `const fn`, and it could not work anyway, because the result would need a heap allocation:

```rust
// ✗ Does not compile. Error: E0015 cannot call non-const associated function `<String as From<&str>>::from` in constants
const GREETING: String = String::from("welcome");

fn main() {
    println!("{GREETING}");
}
```

For values that genuinely need run time (a menu parsed from a file, a database handle), use a `static` with `LazyLock`, covered in Part Three and Part Ten.

### The counter that never counts: interior mutability in a `const`

Because every use of a constant is a fresh copy, a constant that can be changed from the inside (an atomic, a `Mutex`, a `Cell`) is useless as shared state. Each call changes a temporary copy and throws it away. It is not dangerous, just a silent no-op, and rustc warns about it by default:

```rust
use std::sync::atomic::{AtomicU32, Ordering};

const TICKETS: AtomicU32 = AtomicU32::new(0); // rustc warns: const_item_interior_mutations

fn main() {
    TICKETS.fetch_add(1, Ordering::Relaxed);   // adds 1 to a fresh temporary copy
    TICKETS.fetch_add(1, Ordering::Relaxed);   // adds 1 to another fresh copy
    println!("{}", TICKETS.load(Ordering::Relaxed)); // 0: a third fresh copy
}
```

Make it a `static` and it counts. The one legitimate use of an interior-mutable `const` is as a *template* for building arrays, which is what `[const { ... }; N]` now does more directly.

***

## Part Three: Static Items

### What a static is

A `static` is a value with **one fixed address for the whole program**. Every reference to it points at the same bytes. The initializer is evaluated at compile time, exactly like a constant, and the bytes are stored inside your executable. There is no "run this before `main`" step in Rust.

```rust
static SHOP_NAME: &str = "Corner Café";   // one copy, one address, for the whole program

fn main() {
    println!("Welcome to {SHOP_NAME}");
}
```

**Statics are never dropped.** When the program ends the operating system reclaims the memory, but no `Drop` code runs. A static holding a buffered writer (`BufWriter`) never flushes it at exit; a static holding a lock never unlocks. If the last write matters, do it explicitly before `main` returns.

```rust
struct Drawer(&'static str);

impl Drop for Drawer {
    fn drop(&mut self) {
        println!("locking {}", self.0);
    }
}

static MAIN_DRAWER: Drawer = Drawer("main drawer");   // allowed, but its Drop never runs

fn main() {
    let spare = Drawer("spare drawer");
    println!("closing time for {} and {}", MAIN_DRAWER.0, spare.0);
}
// Output:
// closing time for main drawer and spare drawer
// locking spare drawer          <- the local is dropped
//                               <- nothing is printed for the static
```

### Static vs const

| | `const` | `static` |
| :-- | :-- | :-- |
| **Memory address** | none of its own; fresh copy per use | one fixed address |
| **Initialisation** | compile time | compile time; or on first use with `LazyLock` (Rust 1.80+) |
| **Dropped at exit** | each copy is dropped like any value | never |
| **Mutability** | never | `static mut` (unsafe) or interior mutability (`Mutex`, atomics) |
| **Thread safety** | not applicable | the type must be `Sync` |
| **Use for** | fixed numbers, small tables, anything you want inlined | shared state, large tables, calling into C |

### Two ways to fill a static

**At compile time**, with a constant expression:

```rust
static PORT: u16 = 8080;

fn main() {
    println!("listening on {PORT}");
}
```

**On first use**, with `LazyLock` (Rust 1.80, any edition). The closure runs at run time, once, the first time anybody touches the value; every later use reuses the result. This is how you give a static something that needs run time to build, like a menu parsed from a file:

```rust
use std::collections::HashMap;
use std::sync::LazyLock;

static MENU: LazyLock<HashMap<&'static str, f64>> = LazyLock::new(|| {
    println!("building the menu");
    HashMap::from([("espresso", 2.50), ("latte", 3.50)])
});

fn main() {
    println!("program started");
    println!("latte costs {}", MENU["latte"]);        // "building the menu" prints here
    println!("espresso costs {}", MENU["espresso"]);  // reused, nothing is rebuilt
}
```

Prefer `LazyLock` over `OnceLock` when the initialisation code is known where the static is declared; `OnceLock` is for values that arrive from somewhere else at run time (Part Ten).

### When to use a static

- You need one stable address: FFI (calling into C code), a device register that lives at a fixed address, anything outside Rust that keeps a pointer.
- You need program-wide shared state: a ticket counter, a cache, a connection pool. Use interior mutability (`Mutex`, `RwLock`, atomics, `OnceLock`, `LazyLock`).
- The data is large and read-only and you do not want a copy at every use site.

### The `Sync` requirement

A static is visible from every thread, so its type must be `Sync`. Two words that are easy to confuse:

- **`Send`**: the value may be *handed over* to another thread.
- **`Sync`**: the value may be *looked at from several threads at once* through shared references.

Plain data (`u32`, `[i32; 3]`, `&str`, `String`) is both. `Cell` and `RefCell` are `Send` whenever what they hold is `Send` (you may move one to another thread) but **never** `Sync`, whatever they contain, because they let you mutate through a shared reference with no synchronisation. So they cannot be statics:

```rust
// ✗ Does not compile. Error: E0277 `Cell<u32>` cannot be shared between threads safely
use std::cell::Cell;

static TICKETS: Cell<u32> = Cell::new(0);

fn main() {
    TICKETS.set(1);
}
```

Their thread-safe siblings are atomics (for `Cell`) and `Mutex` or `RwLock` (for `RefCell`):

```rust
use std::sync::atomic::{AtomicU32, Ordering};

static TICKETS: AtomicU32 = AtomicU32::new(0);   // AtomicU32 is Sync

fn main() {
    let mine = TICKETS.fetch_add(1, Ordering::Relaxed) + 1;
    println!("ticket {mine}");
}
```

### `static mut`: what is actually wrong with it

Before `Mutex::new` and the atomics could be used in statics, people wrote `static mut` for global counters. Edition 2024 made the compiler refuse most uses of it: taking **any reference** to a `static mut`, including the hidden references made by `println!` or by a method call, is rejected by the deny-by-default lint `static_mut_refs`. In editions 2021 and earlier the same lint is only a warning. A lint can be switched off with `#[allow]`, but doing that is a written statement that you are taking over the compiler's job.

```rust
// ✗ Does not compile in edition 2024. Error: creating a shared reference to mutable static
static mut TICKETS: u32 = 0;

fn main() {
    unsafe {
        TICKETS += 1;              // a plain read or write is still allowed (inside unsafe)
        println!("{}", TICKETS);   // println! takes a reference: rejected by the lint
    }
}
```

Here is the honest version of the rule, because the short version ("any reference is instantly undefined behaviour") is not quite right. Creating a reference to a `static mut` is undefined behaviour **if** the static is written while your shared reference is alive, or if anyone holds a second reference while your exclusive one is alive. That is the many-readers-or-one-writer rule from the Foundation, and the compiler says so in the error itself: shared references to mutable statics are undefined behaviour *if the static is mutated or a mutable reference is created while the shared reference lives*. A lone `&TICKETS` in a program with one thread and no other reference is not, by itself, broken. The problem is that with a global you cannot prove that by looking at one function: any function, in any thread, might be writing at the same time. So the lint refuses references outright, because nobody can check the rule locally.

If you truly must keep a `static mut` (some C interfaces demand a plain mutable global), use raw pointers (`&raw const` and `&raw mut`, Rust 1.82), which the lint accepts because a raw pointer makes no aliasing promise. The responsibility for the many-readers-or-one-writer rule is now yours:

```rust
static mut TICKETS: u32 = 0;

fn main() {
    let p = &raw mut TICKETS;      // a raw pointer, not a reference: the lint is satisfied
    unsafe { *p += 1; }            // your promise: no other thread or reference touches TICKETS now
    let now = unsafe { *p };       // copy the value out
    println!("ticket {now}");
}
```

This is the pattern the edition guide itself recommends. It is not undefined behaviour on its own; it becomes so the moment another thread, or a live reference, uses the same memory at the same time. For everything that is not C interop, use the types in Part Ten:

- **atomics** for counters and flags,
- **`Mutex`** or **`RwLock`** for anything bigger,
- **`OnceLock`** for set-once values that arrive at run time,
- **`LazyLock`** for values built on first use.

***

## Part Four: Ownership Fundamentals

### The three ownership rules

1. Every value has exactly one owner at a time.
2. When the owner goes out of scope, the value is dropped.
3. Ownership can be handed over (moved) to another binding, to a function, or back to a caller.

At the café a receipt is a physical thing. Whoever holds it owns it; hand it to the customer and you no longer have it. There is never a moment when two people both hold the same receipt.

### Stack vs heap: where does data live?

Local variables live in the current function's stack frame. Data that can grow, or whose size is not known at compile time, lives on the heap, and something on the stack holds its address. Needing to outlive a function is not a reason on its own: a value leaves a function by being moved out, heap or not. In Rust you do not choose this per variable; the *type* chooses. `Box`, `Vec`, `String` and `HashMap` are the everyday heap types.

**Stack only:**

```rust
struct Point {
    x: f64,
    y: f64,
}

fn main() {
    let table = Point { x: 3.0, y: 4.0 };                // lives in main's stack frame
    println!("({:?}, {:?}) takes {} bytes", table.x, table.y, std::mem::size_of_val(&table)); // 16
}
```

**Heap, explicitly:**

```rust
struct Point {
    x: f64,
    y: f64,
}

fn main() {
    let boxed = Box::new(Point { x: 3.0, y: 4.0 }); // the Point is on the heap; `boxed` holds its address
    println!("{} {}", boxed.x, boxed.y);
}
```

**Both at once, the common case.** An `Order` is a small fixed-size struct on the stack whose `String` and `Vec` fields point at heap buffers:

```rust
struct Order {
    customer: String,     // 24 bytes here; the letters live on the heap
    ticket: u32,          // 4 bytes here
    items: Vec<String>,   // 24 bytes here; the list lives on the heap
}

fn main() {
    let order = Order {
        customer: String::from("Asha"),
        ticket: 17,
        items: vec![String::from("latte")],
    };
    println!("{} bytes in the Order itself", std::mem::size_of::<Order>()); // 56 on 64-bit
    println!("{} #{} {}", order.customer, order.ticket, order.items.len());
}
```

**When does it matter?** Rarely, at first. Primitives and small structs are on the stack; growable things are on the heap because they must be; `thread_local!` values live in per-thread storage; statics live in the executable's data section. Reach for `Box` only when you need it (a recursive type, a trait object, a huge value you do not want to copy around), and let a profiler, not a hunch, tell you when allocation is your problem. The [memory layout guide](/rust/concepts/2025/01/05/rust-mem-ref.html) has the byte-level detail.

### Move: the default for every type

When you assign a value to another binding, pass it to a function, or return it, ownership **moves**. The old name becomes invalid and the compiler will not let you use it. This is true for every type, stack or heap, unless the type opts into `Copy` (next section).

```rust
#[derive(Debug)]
struct Receipt {
    ticket: u32,
}

fn main() {
    let r1 = Receipt { ticket: 17 };
    let r2 = r1;                  // the receipt changes hands
    // println!("{r1:?}");        // ✗ error[E0382]: borrow of moved value: `r1`
    println!("{r2:?} for ticket {}", r2.ticket);
}
```

The same happens on a function call and on a return:

```rust
fn hand_to_customer(receipt: String) {
    println!("customer takes: {receipt}");
} // `receipt` is dropped here: the function owned it

fn print_receipt() -> String {
    String::from("latte 3.50")   // ownership goes to the caller
}

fn main() {
    let receipt = print_receipt();
    hand_to_customer(receipt);
    // println!("{receipt}");    // ✗ error[E0382]: borrow of moved value: `receipt`
}
```

### Copy: permission to duplicate the bytes

`Copy` is a permission a type gives the compiler: *"duplicating my bytes produces a complete, independent value, because I own nothing outside those bytes."* With that permission, `let b = a;` copies instead of moving, and `a` stays usable.

Numbers, `bool`, `char`, shared references, raw pointers, function pointers, and tuples, arrays and closures made only of `Copy` things all qualify. A `String` does not: its bytes contain the address of a heap buffer it owns, so two bit-for-bit copies would both think they own the same buffer and both would free it.

The compiler enforces three rules for `#[derive(Copy)]`:

1. every field must be `Copy`,
2. the type must not implement `Drop`, and
3. the type must also be `Clone`, because `Copy` is a promise that `clone()` is just a bit copy, so the two are always derived together.

```rust
#[derive(Clone, Copy, Debug)]
struct Price {
    cents: u32,
}

fn charge(p: Price) {
    println!("charging {} cents", p.cents);
}

fn main() {
    let p1 = Price { cents: 350 };
    let p2 = p1;          // copied, both usable
    charge(p1);           // copied again
    println!("{p1:?} {p2:?}");
}
```

```rust
// ✗ Does not compile. Error: E0204 the trait `Copy` cannot be implemented for this type
#[derive(Clone, Copy)]
struct Order {
    customer: String,   // String is not Copy
}

fn main() {}
```

```rust
// ✗ Does not compile. Error: E0184 the trait `Copy` cannot be implemented for this type; the type has a destructor
#[derive(Clone, Copy)]
struct Drawer;

impl Drop for Drawer {
    fn drop(&mut self) {}
}

fn main() {}
```

Notice what the rules do **not** say. They do not say "no heap": a struct holding a raw pointer to heap memory can be `Copy`, and a `Copy` value can sit on the heap inside a `Box`. They do not say "small": a `[u8; 4096]` is `Copy`. The only question is whether copying the bytes copies any *ownership*.

#### Why `&T` is `Copy` and `&mut T` is not

A shared reference is a promise that nobody writes while it exists. Duplicating that promise is harmless: more readers are always allowed. An exclusive reference is a promise that *nothing else* can reach the data. Duplicating it would create two "exclusive" paths, which is the one thing the Foundation rule forbids. So `&mut T` moves:

```rust
fn main() {
    let mut total = 10;
    let m1 = &mut total;
    let m2 = m1;          // this line compiles: m1 is MOVED into m2, not copied
    *m2 += 1;
    // *m1 += 1;          // ✗ error[E0382]: use of moved value: `m1`
    println!("{total}");
}
```

A reference is small, so copying one is cheap, but it is not always one machine word:

| Reference | Size on 64-bit |
| :-- | :-- |
| `&i32`, `&String`, `&Order` | 8 bytes (an address) |
| `&str`, `&[T]` | 16 bytes (address + length) |
| `&dyn Trait` | 16 bytes (address + a pointer to the type's method table) |

### Non-Copy types move, and `clone()` copies on request

Anything that owns heap memory or has a destructor is a move-only type: `String`, `Vec<T>`, `Box<T>`, `HashMap<K, V>`, `File`, and any struct containing one of them. When you genuinely want two independent copies of such a value, ask for it with `clone()`. It is explicit because it can be expensive:

```rust
fn main() {
    let original = String::from("latte");
    let duplicate = original.clone();   // a second heap buffer with the same letters
    println!("{original} {duplicate}"); // both usable
}
```

### Move and Drop cooperate

A moved value is dropped exactly once, where its *final* owner goes out of scope. The old binding owns nothing, so nothing happens when it dies. This is what makes resource handling automatic and leak-free:

```rust
struct Drawer {
    id: u32,
}

impl Drop for Drawer {
    fn drop(&mut self) {
        println!("locking drawer {}", self.id);
    }
}

fn main() {
    let morning_shift = Drawer { id: 3 };
    let evening_shift = morning_shift;   // the drawer changes hands; morning_shift owns nothing now
    println!("evening shift has drawer {}", evening_shift.id);
} // "locking drawer 3" prints exactly once
```

***

## Part Five: Non-Lexical Lifetimes (NLL)

> This part is about how long a **borrow** lasts. Explicit lifetime annotations (`'a`) are in the [ownership and lifetimes guide](/rust/concepts/2025/02/09/rust-ownership.html).

### The problem NLL solved

Until Rust 1.31 (December 2018) the borrow checker decided that a borrow lasted until the closing brace of the block, even when the reference was never used again. That rejected obviously fine code:

```rust
fn main() {
    let mut board = vec![String::from("latte")];
    let first = &board[0];
    println!("next: {first}");             // last use of `first`
    board.push(String::from("espresso"));  // rejected before NLL: `first` was "still borrowed"
    println!("{board:?}");
}
```

A human sees that `first` is dead after the `println!`. The old checker could not.

### How NLL works

Since Rust 1.31 for edition 2018, and 1.36 for edition 2015 (at first as warnings only; the old checker was removed for good in 1.63), the borrow checker follows the control flow and ends a borrow at the **last point the reference can be used**. It is not an optimisation and it changes nothing about the compiled program; it changes which programs are accepted.

```rust
fn main() {
    let mut s = String::from("hello");

    let r1 = &s;
    let r2 = &s;
    println!("{r1} and {r2}");  // last use of r1 and r2

    let r3 = &mut s;            // fine: no shared borrow is alive any more
    r3.push_str(" world");
    println!("{r3}");
}
```

One precise detail: the borrows behind `r1` and `r2` last through the *whole* `println!` call, not just the moment the arguments are read. The macro hands references into a function, and a reference passed to a function counts as alive for the entire call. They end when that statement finishes, which is still before `r3` is created.

Most of the time you never think about NLL. It is why the borrow checker feels reasonable instead of pedantic.

***

## Part Six: Borrowing and References

> Lifetime annotations and reference semantics in depth: the [ownership and lifetimes guide](/rust/concepts/2025/02/09/rust-ownership.html). Part Eight covers two-phase borrows, which explain why `v.push(v.len())` is allowed.

### Binding mutability and reference mutability are two different dials

`let mut` decides whether the **name** can be reassigned or borrowed mutably. `&` versus `&mut` decides what a **reference** may do with the data. Knowing one tells you nothing about the other:

```rust
fn main() {
    let s = String::from("hi");
    let r = &s;                     // immutable binding, shared reference: read only
    println!("{r}");

    let mut s = String::from("hi");
    let r = &s;                     // mutable binding, still a shared reference: read only
    println!("{r}");

    let r = &mut s;                 // mutable binding, exclusive reference: may modify
    r.push('!');
    println!("{r}");
}
```

| Binding | Reference | Result |
| :-- | :-- | :-- |
| immutable | `&` | read-only view; the name cannot be reassigned |
| immutable | `&mut` | **compile error E0596**: cannot borrow as mutable |
| mutable | `&` | read-only view; the name can be reassigned later |
| mutable | `&mut` | the view may modify the value |

```rust
// ✗ Does not compile. Error: E0596 cannot borrow `s` as mutable, as it is not declared as mutable
fn main() {
    let s = String::from("hi");
    let r = &mut s;
    r.push('!');
}
```

### Shared references (`&T`)

A shared reference lets a function read a value without taking it. The caller keeps ownership, and the value is not dropped when the function returns:

```rust
fn count_items(order: &Vec<String>) -> usize {
    order.len()
}

fn main() {
    let order = vec![String::from("latte"), String::from("scone")];
    let n = count_items(&order);
    println!("{n} items: {order:?}");  // order is still ours
}
```

Any number of shared references may exist at once, and because `&T` is `Copy`, passing one to a function does not use it up: the function receives a copy of the reference.

### Cloning through a reference: which thing gets cloned?

Calling `.clone()` on a reference does one of two things depending on the type behind it, and it is worth knowing which:

```rust
fn main() {
    let orders = vec![String::from("latte")];
    let orders_ref: &Vec<String> = &orders;

    let copy = orders_ref.clone();          // clones the Vec, not the reference
    let _: Vec<String> = copy;

    let double_ref: &&Vec<String> = &orders_ref;
    let just_a_ref = double_ref.clone();    // clones the reference; rustc warns: suspicious_double_ref_op
    let _: &Vec<String> = just_a_ref;
}
```

When Rust resolves `orders_ref.clone()`, it first looks for a `clone` whose receiver is exactly `&Vec<String>`. `Vec::clone(&self)` is exactly that, so the list is cloned. The reference type's own `clone` would need `&&Vec<String>`, which comes later in the search. That is method resolution, not deref coercion. The trap runs the other way: if the type behind the reference is **not** `Clone`, `.clone()` quietly copies the reference and hands you another `&T`, and rustc warns with `noop_method_call`. Read that warning as "you did not clone what you think you cloned".

### Exclusive references (`&mut T`)

An exclusive reference lets a function modify a value it does not own:

```rust
fn add_shot(order: &mut String) {
    order.push_str(" + extra shot");
}

fn main() {
    let mut order = String::from("latte");
    add_shot(&mut order);
    println!("{order}");
}
```

`&mut T` is not `Copy`. When you pass one to a function it is *reborrowed* for the duration of the call, which is why you can pass the same `&mut` twice in a row. Part Eight explains reborrowing.

### The borrowing rules

1. **Many readers or one writer**, checked on every path through the code: any number of `&T` or exactly one `&mut T`, never both alive at once.
2. **No dangling references**: a reference can never outlive the value it points at.

```rust
fn main() {
    let mut s = String::from("hello");
    let r1 = &s;
    let r2 = &s;
    // let r3 = &mut s;   // ✗ error[E0502]: cannot borrow `s` as mutable because it is also borrowed as immutable
    println!("{r1} and {r2}");
    s.push('!');          // fine here: r1 and r2 are no longer used
    println!("{s}");
}
```

### Dangling references

Returning a reference to a local is refused. The compiler stops at the signature: a returned reference has to borrow from *something*, and with no parameters there is nothing to borrow from:

```rust
// ✗ Does not compile. Error: E0106 missing lifetime specifier
fn todays_special() -> &String {
    let s = String::from("flat white");
    &s
}

fn main() {
    println!("{}", todays_special());
}
```

If you "help" the compiler by writing `-> &'static String`, the error changes to E0515, *cannot return reference to local variable*, which is the one people expect to see first. The fix in both cases is to return the owned value:

```rust
fn todays_special() -> String {
    let s = String::from("flat white");
    s   // ownership goes to the caller
}

fn main() {
    println!("{}", todays_special());
}
```

### Two exclusive references into one collection

You cannot hold `&mut v[0]` and `&mut v[2]` at the same time by indexing twice; the compiler cannot see that the indexes differ. The standard library provides the safe ways to split a collection into non-overlapping exclusive views:

```rust
fn main() {
    let mut queue = vec![String::from("latte"), String::from("mocha"), String::from("espresso")];

    let [first, last] = queue.get_disjoint_mut([0, 2]).unwrap();   // Rust 1.86+
    std::mem::swap(first, last);                                   // the last order jumps to the front

    let (front, back) = queue.split_at_mut(1);                     // front: [0], back: [1, 2]
    front[0].push_str(" (rush)");
    back[0].push_str(" (regular)");

    println!("{queue:?}");   // ["espresso (rush)", "mocha (regular)", "latte"]
}
```

***

## Part Seven: Passing Values to Functions

Rust passes every argument **by value**. What that means depends on the type, and it is all decided at compile time with no run-time cost:

| What you pass | Type | What happens | The caller's binding afterwards |
| :-- | :-- | :-- | :-- |
| `x` | a `Copy` type | the bytes are duplicated | still valid, unchanged |
| `x` | any other type | ownership moves into the function | invalid; using it is a compile error |
| `&x` or `&mut x` | a reference | the reference itself is passed by value (copied for `&T`, reborrowed for `&mut T`); the borrow lasts as long as the callee can use it | valid; usable again once the borrow ends |

"By reference" is not a third mechanism in the machine's eyes: a reference is a small value that is copied or reborrowed like any other. It only *feels* different because the borrow checker tracks what the callee may do with it.

```rust
fn square(x: i32) -> i32 {                 // i32 is Copy: `n` is copied
    x * x
}

fn hand_over(receipt: String) {            // String is not Copy: `receipt` is moved
    println!("{receipt}");
}

fn add_item(order: &mut Vec<String>) {     // borrowed exclusively for the call
    order.push(String::from("scone"));
}

fn main() {
    let n = 5;
    println!("{} {}", square(n), square(n));   // n still usable

    let receipt = String::from("latte 3.50");
    hand_over(receipt);
    // hand_over(receipt);                     // ✗ error[E0382]: use of moved value

    let mut order = vec![String::from("latte")];
    add_item(&mut order);                      // borrow starts and ends inside the call
    add_item(&mut order);                      // so we can do it again
    println!("{order:?}");
}
```

***

## Part Eight: Advanced Borrowing Patterns

### Two-phase borrows: why `v.push(v.len())` works

Read literally, `queue.push(queue.len())` takes an exclusive borrow of `queue` for `push` *and* a shared borrow for `len()` at the same time, which the Foundation rule forbids. It compiles anyway:

```rust
fn main() {
    let mut queue: Vec<usize> = vec![];
    queue.push(queue.len());   // number each ticket by its position in the queue
    queue.push(queue.len());
    println!("{queue:?}");     // [0, 1]
}
```

The borrow checker treats the `&mut` it created for the method receiver as a **two-phase borrow**. Phase one *reserves* it: from then on nobody may take another `&mut`, but shared reads are still fine. Phase two *activates* it at the moment `push` is actually called, after all the arguments have been evaluated. `queue.len()` runs during phase one, so the two borrows never truly overlap.

Two-phase treatment applies to three shapes, all of them borrows the compiler inserts *for* you:

1. the automatic `&mut self` of a method call (`queue.push(...)`),
2. the automatic reborrow of a `&mut` you pass as a function argument,
3. the hidden `&mut` of a compound assignment on an overloaded operator (`x += x` on a `Wrapping`, the integer wrapper whose arithmetic wraps around instead of overflowing).

It does **not** apply to a `&mut queue` that you write out by hand, because that is an ordinary borrow that starts the moment it is evaluated:

```rust
// ✗ Does not compile. Error: E0502 cannot borrow `queue` as immutable because it is also borrowed as mutable
fn number_ticket(queue: &mut Vec<usize>, n: usize) {
    queue.push(n);
}

fn main() {
    let mut queue: Vec<usize> = vec![];
    number_ticket(&mut queue, queue.len());
}
```

The difference is not "method call versus free function". The same free function is happy when the `&mut` is *reborrowed* for you (shape 2):

```rust
fn number_ticket(queue: &mut Vec<usize>, n: usize) {
    queue.push(n);
}

fn main() {
    let mut queue: Vec<usize> = vec![];
    let q = &mut queue;
    number_ticket(q, q.len());   // `q` is implicitly reborrowed, and that reborrow is two-phase
    println!("{queue:?}");
}
```

And the method-call form breaks as soon as you write the borrow by hand: `Vec::push(&mut queue, queue.len())` gives the same error as the free function. Rust always evaluates a function's arguments **left to right**; that order is guaranteed by the language. The explicit `&mut queue` is evaluated first and is already active when `queue.len()` runs, which is the conflict.

**Practical rule:** if a call complains about a borrow, evaluate the argument into a local first:

```rust
fn number_ticket(queue: &mut Vec<usize>, n: usize) {
    queue.push(n);
}

fn main() {
    let mut queue: Vec<usize> = vec![];
    let n = queue.len();           // the shared borrow ends here
    number_ticket(&mut queue, n);  // now the exclusive borrow has no competition
    println!("{queue:?}");
}
```

### Reborrowing

A reborrow is a new reference created *through* an existing one, written `&*r` or `&mut *r`. While the reborrow is alive the original is paused; once the reborrow's last use has passed, the original is usable again. You can write one by hand:

```rust
fn main() {
    let mut total = 5;
    let r1 = &mut total;
    let r2 = &mut *r1;   // explicit reborrow
    *r2 += 1;            // r1 is paused while r2 is in use
    *r1 += 1;            // r2's last use has passed, so r1 is back
    println!("{total}"); // 7
}
```

You rarely write that, because the compiler does it for you **whenever an expression of type `&mut T` is used where exactly `&mut T` is expected**, such as a function parameter. That is why an exclusive reference can be passed to a function again and again:

```rust
fn add_shot(order: &mut String) {
    order.push_str(" +shot");
}

fn main() {
    let mut order = String::from("latte");
    let r = &mut order;
    add_shot(r);   // implicitly reborrowed as `&mut *r` for the call
    add_shot(r);   // so `r` is still ours afterwards
    println!("{order}");
}
```

**The trap: implicit reborrowing needs a known `&mut T` target.** When the parameter is generic, the compiler does not know it is looking for `&mut T`, so it moves the reference instead:

```rust
fn add_shot(order: &mut String) {
    order.push_str(" +shot");
}

fn log<T: std::fmt::Debug>(item: T) {   // generic parameter: no implicit reborrow
    println!("{item:?}");
}

fn main() {
    let mut order = String::from("latte");
    let r = &mut order;
    add_shot(r);      // reborrowed
    log(r);           // MOVED into `log`
    // add_shot(r);   // ✗ error[E0382]: borrow of moved value: `r`
    println!("{order}");
}
```

The `for` loop has the same shape. `for item in r` calls `IntoIterator::into_iter(r)`, whose parameter is the generic `Self`, so `r` is moved and gone after the loop. Iterate through a method call or an explicit reborrow instead:

```rust
fn main() {
    let mut orders = vec![String::from("latte"), String::from("mocha")];
    let r = &mut orders;

    for o in r.iter_mut() { o.push('!'); }   // method call: reborrowed, r survives
    for o in &mut *r { o.push('?'); }        // explicit reborrow: r survives
    // for o in r { }                        // would MOVE r; using r afterwards is E0382

    r.push(String::from("espresso"));
    println!("{orders:?}");
}
```

If you ever need the paused-original behaviour on purpose, for instance to hand a `&mut` to a helper and keep your own afterwards, write `&mut *r` explicitly; it always works.

### Closures and the `move` keyword

A closure looks at how its body uses each variable and captures it in the **gentlest way that works**: a shared borrow if it only reads, an exclusive borrow if it changes it, by value if it consumes it. The compiler tries the modes in that order (the Reference lists one more in between, a "unique immutable borrow", which only matters when a closure writes through a `&mut` it merely borrowed).

```rust
fn main() {
    let special = String::from("flat white");
    let announce = || println!("today: {special}");   // only reads: borrows `special`
    announce();
    println!("{special}");                            // still ours

    let mut served = 0;
    let mut tick = || served += 1;                    // changes it: exclusive borrow
    tick();
    tick();
    println!("{served}");

    let sold_out = || drop(special);                  // consumes it: captured by value, no `move` needed
    sold_out();
    // println!("{special}");                         // ✗ error[E0382]: borrow of moved value: `special`
}
```

`move` forces by-value capture of everything the closure mentions. You need it when the closure will **outlive the current function** and the body only *borrows*, because a borrow of a local cannot leave the function alive. Threads are the classic case:

```rust
use std::thread;

fn main() {
    let orders = vec![String::from("latte"), String::from("mocha")];

    // This body consumes `orders` (a `for` loop takes it by value), so the closure
    // owns it even without `move`.
    let kitchen = thread::spawn(|| {
        for o in orders {
            println!("making {o}");
        }
    });
    kitchen.join().unwrap();

    let orders = vec![String::from("espresso")];

    // This body only borrows `orders`. Without `move` the compiler refuses (E0373): the
    // thread could outlive main's stack frame. `move` hands the whole Vec to the thread.
    let kitchen = thread::spawn(move || {
        for o in &orders {
            println!("making {o}");
        }
    });
    kitchen.join().unwrap();
    // println!("{orders:?}");   // ✗ error[E0382]: borrow of moved value: `orders`
}
```

Two details experts rely on: `move` on a `Copy` value copies it, so the original stays usable; and since edition 2021 a closure captures individual *fields* (`order.items`) rather than whole variables, which makes many more closures compile.

### Partial moves

Moving one field out of a struct leaves the struct **partially moved**. The fields that are still there can be used; the struct as a whole cannot. This is entirely a compile-time matter: code with a partial-move mistake is refused, never run.

```rust
#[derive(Debug)]
struct Order {
    ticket: u32,
    receipt: String,
}

fn main() {
    let order = Order { ticket: 17, receipt: String::from("paid 3.50") };
    let receipt = order.receipt;          // moved out: `order` is now partially moved
    println!("{}", order.ticket);         // fine: `ticket` is Copy and still there
    // println!("{order:?}");             // ✗ error[E0382]: borrow of partially moved value: `order`
    // println!("{}", order.receipt);     // ✗ error[E0382]: borrow of moved value: `order.receipt`
    println!("{receipt}");
}
```

**Fix one: destructure**, when you own the struct. Every field gets a name, ownership is explicit, and the old name is gone rather than half-gone:

```rust
struct Order {
    ticket: u32,
    receipt: String,
}

fn file_receipt(receipt: String) {
    println!("filed: {receipt}");
}

fn main() {
    let order = Order { ticket: 17, receipt: String::from("paid 3.50") };
    let Order { ticket, receipt } = order;   // ticket is copied, receipt is moved, `order` no longer exists
    file_receipt(receipt);
    println!("ticket {ticket}");
}
```

**Fix two: `mem::take` or `Option::take`**, when you only have `&mut self` and cannot move anything out. `take` swaps in an empty value and hands you the old one, so the struct stays whole:

```rust
#[derive(Debug)]
struct Order {
    ticket: u32,
    receipt: String,
}

impl Order {
    fn hand_over_receipt(&mut self) -> String {
        std::mem::take(&mut self.receipt)   // leaves an empty String behind
    }
}

fn main() {
    let mut order = Order { ticket: 17, receipt: String::from("paid 3.50") };
    let receipt = order.hand_over_receipt();
    println!("{receipt} / ticket {} / {order:?}", order.ticket); // order is still whole; its receipt is now ""
}
```

**Fix three: rebuild.** Read the copyable fields from the partially moved value and construct a fresh struct:

```rust
#[derive(Debug)]
struct Order {
    ticket: u32,
    receipt: String,
}

fn main() {
    let order = Order { ticket: 17, receipt: String::from("paid 3.50") };
    let old_receipt = order.receipt;                                                 // move out
    let order = Order { ticket: order.ticket, receipt: String::from("refunded") };   // ticket still readable
    println!("{old_receipt} -> {order:?}");
}
```

***

## Part Nine: Interior Mutability, Briefly

**Interior mutability** means changing data through a *shared* reference, with the many-readers-or-one-writer rule checked at run time instead of compile time. Everything in Part Ten is built on it: an atomic, a `Mutex`, a `RwLock`, a `OnceLock` and a `LazyLock` all let you mutate through `&`, which is exactly what a `static` hands out. They are entirely safe to use; the check has simply moved from the compiler to a lock, an atomic instruction, or a "set exactly once" rule.

Each thread-safe type has a cheaper single-thread sibling that a `static` cannot hold (they are not `Sync`), but that is perfect inside one thread or inside `thread_local!`:

| Single thread | Across threads | What it gives you |
| :-- | :-- | :-- |
| `Cell<T>` | atomics | get and set a small value |
| `RefCell<T>` | `Mutex<T>`, `RwLock<T>` | borrow a value mutably, checked at run time |
| `OnceCell<T>` | `OnceLock<T>` | set once, read many times |
| `LazyCell<T>` | `LazyLock<T>` | build on first use |

A `RefCell` panics if you break the rule; a `Mutex` makes the second thread wait. Same rule, different response. The single-thread column, and the `UnsafeCell` underneath all of it, get their own post.

***

## Part Ten: Safe Global State Patterns

### Atomics for counters and flags

For a number or a flag shared by every thread, an atomic gives you correct updates without a lock. The café's ticket counter:

```rust
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread;

static TICKETS_SOLD: AtomicU64 = AtomicU64::new(0);

fn main() {
    let tills: Vec<_> = (0..4)
        .map(|_| {
            thread::spawn(|| {
                for _ in 0..10_000 {
                    TICKETS_SOLD.fetch_add(1, Ordering::Relaxed);
                }
            })
        })
        .collect();
    for t in tills {
        t.join().unwrap();
    }
    println!("{}", TICKETS_SOLD.load(Ordering::Relaxed)); // exactly 40000, every run
}
```

Note the result: **exactly** 40 000. `Relaxed` does not mean "approximately". Every atomic operation is indivisible whatever ordering you pick, and no increment is ever lost. What the `Ordering` controls is something else: whether the operation also *publishes or waits for other memory*.

**The orderings in plain words:**

- **`Relaxed`**: "count correctly, promise nothing about any other memory." Right for counters and statistics that nothing else depends on. Wrong the moment you use the value to decide that some *other* data is ready.
- **`Release`** on a store and **`Acquire`** on a load: a hand-off note. Everything the writer did *before* the `Release` store is visible to a reader *after* its `Acquire` load sees that value. This is the pattern for "the data is ready" flags and for most coordination.
- **`SeqCst`**: `Acquire` and `Release` plus one extra promise: every thread agrees on a single global order of all `SeqCst` operations. You need this only when several threads reason about the *combined* order of several atomics. As a replacement for a weaker ordering it is never wrong, only slower; it does not repair an algorithm that is wrong for other reasons.

The hand-off pattern, with the "shop open" sign publishing today's price:

```rust
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::thread;

static PRICE_CENTS: AtomicU32 = AtomicU32::new(0);
static OPEN: AtomicBool = AtomicBool::new(false);

fn main() {
    let manager = thread::spawn(|| {
        PRICE_CENTS.store(350, Ordering::Relaxed);   // 1. set today's price
        OPEN.store(true, Ordering::Release);         // 2. flip the sign: publishes everything written before it
    });

    while !OPEN.load(Ordering::Acquire) {            // 3. a till waits for the sign
        std::hint::spin_loop();
    }
    println!("{}", PRICE_CENTS.load(Ordering::Relaxed)); // 4. guaranteed 350, because of Release/Acquire
    manager.join().unwrap();
}
```

Make the sign `Relaxed` on both sides and step 4 may print 0 on a phone or a server CPU with a weak memory model, and even on x86 the *compiler* is free to reorder the two stores. That is not undefined behaviour, it is a wrong answer, and it is wrong per the language rules on every architecture, not only on ARM.

**Choosing:**

1. A counter or statistic that nothing else depends on: `Relaxed`.
2. A flag or handle that tells other threads "this data is ready": `Release` to publish, `Acquire` to observe.
3. Several atomics whose *combined* ordering matters across threads: `SeqCst`, and write down why.
4. Not sure? `SeqCst` is always correct. Measure before you weaken it.

On x86-64, loads and read-modify-write operations compile to the same instructions under every ordering, though a weaker ordering still gives the compiler more freedom to reorder around them; a `SeqCst` *store* becomes an `xchg` (or a move plus a fence) where a `Release` store is a plain move. On ARM and RISC-V the differences are larger. Busy-waiting as in step 3 is for illustration; real code parks the thread or uses a condition variable (`Condvar`), which lets a thread sleep until it is woken.

### Mutex and RwLock

For anything bigger than a number, a `Mutex` lets one thread at a time work on the data; the guard it returns unlocks when dropped. A `RwLock` allows many readers *or* one writer, which is the Foundation rule enforced at run time.

```rust
use std::sync::Mutex;

static TODAYS_ORDERS: Mutex<Vec<String>> = Mutex::new(Vec::new());

fn main() {
    {
        let mut orders = TODAYS_ORDERS.lock().unwrap();   // waits until the lock is free
        orders.push(String::from("latte"));
        orders.push(String::from("mocha"));
    } // guard dropped: unlocked

    let orders = TODAYS_ORDERS.lock().unwrap();          // lock it again
    println!("{orders:?}");
} // dropped again
```

**Poisoning.** If a thread panics while holding the lock, the mutex becomes *poisoned*, because the data may be half-updated. `lock()` then returns `Err`; it does not panic by itself. The usual `.unwrap()` is what turns that into a panic, deliberately, so that a broken invariant spreads no further. You can also choose to look inside:

```rust
use std::sync::Mutex;
use std::thread;

static TILL: Mutex<u32> = Mutex::new(100);

fn main() {
    let _ = thread::spawn(|| {
        let _cash = TILL.lock().unwrap();
        panic!("printer jammed while the drawer was open");
    })
    .join();                                   // that thread panicked while holding the lock

    match TILL.lock() {
        Ok(cash) => println!("clean: {cash}"),
        Err(poisoned) => {                      // lock() returned Err; no panic happened here
            let cash = poisoned.into_inner();   // you may still take the guard
            println!("poisoned, but the drawer holds {cash}");
        }
    }
}
```

When you run this, the spawned thread's panic message also appears on stderr; that is the point of the example.

Three more things to know:

- **Locking a mutex you already hold**, on the same thread, is a bug. The documentation promises only that the second call never returns; in practice it deadlocks (observed on macOS). Keep guards short-lived, and never call out to code that might lock the same mutex.
- `try_lock()` returns immediately with `Err` if the lock is busy, for the cases where waiting is wrong.
- Rust 1.98 still ships only the poisoning `Mutex` on stable. A non-poisoning variant exists in the standard library as an unstable `std::sync::nonpoison` module.

### OnceLock: set once, from wherever the value comes from

`OnceLock` starts empty and accepts exactly one value. It is the tool for a global that is filled in at start-up from something outside the code, such as a config file or a command-line flag:

```rust
use std::sync::OnceLock;

static SHOP_ID: OnceLock<String> = OnceLock::new();

fn main() {
    let from_config = String::from("corner-cafe-01");   // imagine: read from a file at start-up
    println!("{:?}", SHOP_ID.set(from_config));         // Ok(())
    println!("{:?}", SHOP_ID.set(String::from("x")));   // Err("x"): already set; no panic
    println!("{}", SHOP_ID.get().unwrap());
}
```

`get_or_init` combines the two steps and is safe to race: if several threads call it at once, one closure runs and the others wait for its result. If that closure panics, the cell stays empty and the next caller's closure gets its turn:

```rust
use std::sync::OnceLock;

static DB_URL: OnceLock<String> = OnceLock::new();

fn db_url() -> &'static str {
    DB_URL.get_or_init(|| {
        println!("reading the connection string once");
        String::from("postgres://localhost/cafe")
    })
}

fn main() {
    println!("{}", db_url());
    println!("{}", db_url());   // cached
}
```

Do not call `get_or_init` on the same cell from inside its own initialiser: the documentation says the outcome is unspecified, and the current implementation deadlocks.

### LazyLock: build on first use

`LazyLock` (Rust 1.80, any edition) is a `OnceLock` whose initialiser is written right where the static is declared. The closure runs at **run time, on first access**, never at compile time:

```rust
use std::sync::LazyLock;

static PRICE_LIST: LazyLock<Vec<(&'static str, f64)>> = LazyLock::new(|| {
    println!("building the price list");
    vec![("espresso", 2.50), ("latte", 3.50)]
});

fn main() {
    println!("before first use");
    println!("{:?}", *PRICE_LIST);   // "building the price list" prints here
    println!("{:?}", *PRICE_LIST);   // reused
}
```

Use `LazyLock` when the initialisation code is self-contained, and `OnceLock` when the value is handed in from elsewhere. Since Rust 1.94, `LazyLock::get` tells you whether it has been initialised without forcing it, and `LazyLock::force_mut` gives mutable access through a `&mut` to the lock.

### LazyCell and `thread_local!`

`LazyCell` is the single-thread twin of `LazyLock`. Because it is not `Sync`, the compiler simply refuses to put it in a `static`; there is no run-time hazard to worry about, only a compile error:

```rust
// ✗ Does not compile. Error: E0277 `UnsafeCell<...>` cannot be shared between threads safely; the help line points at LazyCell, which is not Sync
use std::cell::LazyCell;

static MENU: LazyCell<Vec<&'static str>> = LazyCell::new(|| vec!["latte"]);

fn main() {
    println!("{:?}", *MENU);
}
```

It is `Send` whenever both its value and its initialiser closure are `Send`, so *moving* one into a thread is fine. Its natural home is inside a single thread's own data. For per-thread globals, `thread_local!` already initialises its value lazily the first time each thread touches it, so a `LazyCell` inside is rarely needed. What you often want instead is the `const { }` initialiser, which lets the implementation skip the lazy-initialisation check on most platforms:

```rust
use std::cell::Cell;

thread_local! {
    // One counter per thread; with a `const` initialiser the lazy-initialisation check can be skipped.
    static SERVED_HERE: Cell<u32> = const { Cell::new(0) };
}

fn main() {
    SERVED_HERE.with(|n| n.set(n.get() + 1));
    let on_other_thread = std::thread::spawn(|| SERVED_HERE.with(|n| n.get())).join().unwrap();
    println!("this thread {}, other thread {}", SERVED_HERE.with(|n| n.get()), on_other_thread); // 1, 0
}
```

### Arc<Mutex<T>> for shared ownership across threads

A `static` is one value for the whole program. When a group of threads needs to share something created at run time, wrap it in `Arc` (shared ownership with an atomic reference count) around a `Mutex` (one writer at a time):

```rust
use std::sync::{Arc, Mutex};
use std::thread;

fn main() {
    let tips = Arc::new(Mutex::new(0));
    let mut baristas = vec![];

    for _ in 0..3 {
        let tips = Arc::clone(&tips);   // another handle to the same jar; the jar itself is not copied
        baristas.push(thread::spawn(move || {
            let mut jar = tips.lock().expect("tip jar mutex poisoned");
            *jar += 5;
        }));
    }
    for b in baristas {
        b.join().unwrap();
    }
    println!("tips: {}", *tips.lock().unwrap());   // 15
}
```

Writing `Arc::clone(&tips)` rather than `tips.clone()` is a convention that tells the reader "this copies a handle, not the data". Keep guards short, take locks in one fixed order everywhere, and remember that `lock()` gives you `Err` on poison; `expect` with a message is the clearest way to turn that into a panic.

***

## Part Eleven: Best Practices and Decision Guide

### Const or static?

**Use `const` when** the value is fixed at compile time, is small, and you are happy for every use to get its own copy: the tax rate, a maximum, a lookup table.

```rust
const TAX_RATE: f64 = 0.05;
const MAX_ITEMS_PER_ORDER: usize = 20;
const LOYALTY_STAMPS: [u8; 5] = [1, 2, 3, 4, 5];

fn total_with_tax(subtotal: f64) -> f64 {
    subtotal * (1.0 + TAX_RATE)   // TAX_RATE is copied in right here
}

fn main() {
    println!("{:.2} {MAX_ITEMS_PER_ORDER} {LOYALTY_STAMPS:?}", total_with_tax(10.0));
}
```

**Use `static` when** you need one address (calling into C code), one instance of something large, or shared state that changes at run time:

```rust
use std::sync::{Mutex, OnceLock};

struct AppConfig {
    database_url: String,
}

static CONFIG: OnceLock<AppConfig> = OnceLock::new();
static TICKETS: Mutex<u64> = Mutex::new(0);

fn config() -> &'static AppConfig {
    CONFIG.get_or_init(|| AppConfig { database_url: String::from("postgres://localhost/cafe") })
}

fn main() {
    *TICKETS.lock().unwrap() += 1;
    println!("{} / tickets {}", config().database_url, TICKETS.lock().unwrap());
}
```

Anything with interior mutability (`AtomicU32`, `Mutex`, `Cell`) that is meant to be *shared* belongs in a `static`, never in a `const`: in a `const` every use is a fresh copy and the mutation is lost (Part Two). rustc warns about it by default.

### Move or borrow?

**Move** when the callee should own the value from now on: the caller is done with it, the value carries a resource with cleanup (a file, a connection), or the function turns it into something new.

**Borrow** when the caller still needs it afterwards, the callee only reads it, or the callee needs temporary write access. Library functions usually borrow, and take the most general type they can (`&str` rather than `&String`, `&[T]` rather than `&Vec<T>`).

```rust
use std::io::Read;

// Returns an owned String: the caller becomes the owner.
fn read_menu(path: &str) -> std::io::Result<String> {
    let mut file = std::fs::File::open(path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(contents)
}

// Borrows: only needs to look.
fn count_lines(text: &str) -> usize {
    text.lines().count()
}

fn main() {
    match read_menu("menu.txt") {
        Ok(menu) => println!("{} lines", count_lines(&menu)),
        Err(e) => println!("no menu file: {e}"),
    }
}
```

Run as shown there is no `menu.txt`, so you will see the error branch; the point is the two signatures.

### Common pitfalls

**Pitfall 1: cloning to make the compiler go away.** A `.clone()` sprinkled in to silence an error usually means the function should borrow:

```rust
// Wasteful: takes the whole list by value although it only reads it, so every caller
// that still needs the list has to clone it first.
fn print_receipts_wasteful(orders: Vec<String>) -> Vec<String> {
    let mut receipts = Vec::new();
    for order in &orders {
        receipts.push(format!("receipt: {order}"));
    }
    receipts
}

// Right: borrow the slice; the caller keeps its list and nothing is copied but the output.
fn print_receipts(orders: &[String]) -> Vec<String> {
    orders.iter().map(|order| format!("receipt: {order}")).collect()
}

fn main() {
    let orders = vec![String::from("latte"), String::from("mocha")];
    let a = print_receipts_wasteful(orders.clone());   // the clone exists only to keep `orders`
    let b = print_receipts(&orders);                   // no clone needed
    println!("{a:?} {b:?} {}", orders.len());
}
```

**Pitfall 2: holding a borrow across a change.** The borrow checker is describing a real bug here: pushing may move the whole vector to a new heap buffer, and `first` would point at freed memory.

```rust
// ✗ Does not compile. Error: E0502 cannot borrow `board` as mutable because it is also borrowed as immutable
fn main() {
    let mut board = vec![String::from("latte")];
    let first = &board[0];
    board.push(String::from("mocha"));
    println!("{first}");
}
```

Take what you need out of the borrow first (clone the item, or copy its length), or do the push before you take the reference.

**Pitfall 3: `static mut` when a safe type exists.** Since Rust 1.63 `Mutex::new` is usable in a static, and the atomics' constructors have been since Rust 1.24; there is no reason left for a `static mut` outside C interop (Part Three).

```rust
use std::sync::atomic::{AtomicU64, Ordering};

static TICKETS: AtomicU64 = AtomicU64::new(0);

fn next_ticket() -> u64 {
    TICKETS.fetch_add(1, Ordering::Relaxed) + 1   // a counter nothing else depends on: Relaxed is exact
}

fn main() {
    println!("{} {}", next_ticket(), next_ticket());
}
```

***

## Part Twelve: Choosing the Right Global State Pattern

| Pattern | Use it for | Example |
| :-- | :-- | :-- |
| plain `static` | a fixed value with one address, known at compile time | `static PORT: u16 = 8080;` |
| **atomics** | counters and flags | `static HITS: AtomicU64 = AtomicU64::new(0);` |
| **`Mutex` / `RwLock`** | shared state that changes at run time | `static ORDERS: Mutex<Vec<String>> = Mutex::new(Vec::new());` |
| **`LazyLock`** | a value built on first use, initialiser known at the declaration | `static MENU: LazyLock<Menu> = LazyLock::new(load_menu);` |
| **`OnceLock`** | a value set once at run time from somewhere else | `static ID: OnceLock<String> = OnceLock::new();` then `ID.set(v)` |
| **`Arc<Mutex<T>>`** | shared ownership of run-time data between threads | `let shared = Arc::new(Mutex::new(data));` |

**Quick decision:**

- A number or a flag: an atomic.
- Anything bigger that changes: `Mutex` or `RwLock`.
- Built once, on first use, from code you can write at the declaration: `LazyLock`.
- Set once, from a value that arrives at run time: `OnceLock`.
- Not global at all, but shared by some threads: `Arc<Mutex<T>>`.

## Conclusion

Everything in this post follows from two ideas. **One owner at a time**: every value has exactly one owner, so it is dropped exactly once, at a point the compiler can name. **Many readers or one writer**: while a value is shared it cannot change, and while it is being changed nobody else can see it. Constants are values copied into place at compile time; statics are single, never-dropped values with one address; bindings own values for a scope; borrows lend them out under the one rule.

Learn where each value is born and where it dies, and the borrow checker stops being an obstacle and becomes the colleague who reads your code more carefully than you do.

For byte-level layout, see the [memory layout guide](/rust/concepts/2025/01/05/rust-mem-ref.html). For lifetime annotations, variance and the advanced ownership patterns, continue with the [ownership and lifetimes guide](/rust/concepts/2025/02/09/rust-ownership.html).
