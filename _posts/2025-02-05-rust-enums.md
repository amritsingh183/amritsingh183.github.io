---
layout: post
title: "Mastering Rust's `Enum` type: A Complete Guide"
date: 2025-02-05 23:23:00 +0530
categories: rust concepts
last_updated: 2026-09-19
---
# Mastering Rust's Enum Type: A Complete Guide

At the café till the same kind of question comes up all day. Which size is this coffee: small, medium or large? How is the customer paying: cash, card or voucher? Where is order number 7 right now: received, brewing, ready, collected or cancelled? Each question has a short list of possible answers, and exactly one of them is true at a time.

An **enum** is how Rust writes that kind of question down as a type. This post shows how to define enums, what data their variants can carry, what they cost in memory, and how `match` and its relatives take them apart. Along the way it covers the mistakes the compiler catches for you, and the few it does not.

## Prerequisites

This is the fourth post in the series. It assumes:

- [Variables, constants and lifetimes](/rust/concepts/2025/01/01/rust-var-const-lifetimes.html): bindings, `mut`, scope, moves and `Drop`.
- [The memory layout reference](/rust/concepts/2025/01/05/rust-mem-ref.html): what a `String` contains and what a niche is. You need only the idea that every value has a size in bytes.
- [Option and Result](/rust/concepts/2025/01/09/rust-option-result.html): the first meeting with `match`, `if let` and `let ... else`, on the two enums the standard library gives you.

Post 3 used enums that the standard library wrote. This post is about the enums **you** write, and about everything `match` can do with them. The [ownership post](/rust/concepts/2025/02/09/rust-ownership.html) that follows relies on the borrowing habits practised here.

**Checked in September 2026 against Rust 1.98.1 (1.98.0 shipped in August 2026), edition 2024.** Every code block is a complete program that compiles and runs as shown, except the ones whose first line starts with ✗. Those are meant to fail so you can see the exact error the compiler gives. Two blocks marked ✎ are the two halves of a two-crate example and are explained where they appear. Where a rule depends on the edition or on a Rust version, the text says which. Sizes reported by `size_of` were observed on this machine (64-bit, Apple silicon); the text says when a number is guaranteed and when it is merely what this compiler chose.

**The running example.** Three enums from the café till appear throughout:

| Café question | Enum | What its variants carry |
| :-- | :-- | :-- |
| Which size? | `Size`: `Small`, `Medium`, `Large` | Nothing. A label is enough. |
| How is the customer paying? | `Payment`: `Cash`, `Card { .. }`, `Voucher(..)` | Different data per variant: nothing, card details, a voucher code |
| Where is this order? | `OrderStatus`: `Received`, `Brewing`, `Ready { .. }`, `Collected`, `Cancelled { .. }` | The state of an order, with data only where that state needs it |

When a rule feels abstract, ask what it would mean at the till.

**Contents**

1. [What an enum is](#enums-1)
2. [The three kinds of variant](#enums-2)
3. [A variant is not a type](#enums-3)
4. [Creating and using enum values](#enums-4)
5. [Pattern matching in depth](#enums-5)
6. [Exhaustiveness and wildcards](#enums-6)
7. [Best practices and pitfalls](#enums-7)
8. [Summary and reference](#enums-8)

<a id="enums-1"></a>

## 1. What an enum is

A `struct` bundles several things that are all present at once: a receipt has a ticket number *and* a total *and* a time. An `enum` is the other shape: a value that is exactly **one** of several named possibilities. A payment is cash *or* card *or* voucher, never two of them. Rust calls each possibility a **variant**.

(If you meet the type-theory names elsewhere: a struct is a *product type*, an enum is a *sum type*. You do not need the names to use them.)

The simplest enum is a list of labels:

```rust
#[derive(Debug)]
enum Size {
    Small,
    Medium,
    Large,
}

fn main() {
    let first = Size::Small;
    let second = Size::Large;
    println!("{first:?} and {second:?}");
    let refill = Size::Medium;
    println!("{refill:?}");
}
```

```text
Small and Large
Medium
```

Three things to notice:

- `Size` is the type. `Size::Small` is a value of that type: you name a variant through its enum, with `::`.
- `#[derive(Debug)]` lets `{:?}` print the variant's name. Without it, `println!` has no idea how to show a `Size`.
- Nothing in `Size` is a number or a string. A variant can be a pure label, and the compiler still knows the complete list of labels.

What makes Rust enums different from the enums of C or Java, or from Go's lists of constants, is that variants can **carry data**, and different variants can carry different data:

```rust
#[derive(Debug)]
enum Payment {
    Cash,
    Card { last4: u16, contactless: bool },
    Voucher(String),
}

fn main() {
    let payments = [
        Payment::Cash,
        Payment::Card { last4: 4242, contactless: true },
        Payment::Voucher(String::from("FREE1")),
    ];
    for payment in &payments {
        let note = match payment {
            Payment::Cash => String::from("exact change, please"),
            Payment::Card { last4, contactless } => format!("card ending {last4}, contactless: {contactless}"),
            Payment::Voucher(code) => format!("voucher code {code}"),
        };
        println!("{payment:?}: {note}");
    }
}
```

```text
Cash: exact change, please
Card { last4: 4242, contactless: true }: card ending 4242, contactless: true
Voucher("FREE1"): voucher code FREE1
```

Every element of `payments` has the same type, `Payment`. One holds nothing, one holds a number and a flag, one holds a `String`. The array does not care which: it stores three `Payment` values, all the same size. A `Payment` value holds the data of **one** variant at a time, never of all three. The `match` names a variant and, in the same pattern, the data inside it: `last4`, `contactless`, `code`. Post 3 introduced this; section 5 goes much deeper.

### You have been using enums since post 3

`Option` and `Result` are ordinary enums defined in the standard library. Their definitions are short enough to read in full:

```text
enum Option<T> {
    None,
    Some(T),
}

enum Result<T, E> {
    Ok(T),
    Err(E),
}
```

You write `Some(3)` rather than `Option::Some(3)` only because the [standard prelude](https://doc.rust-lang.org/std/prelude/index.html) imports the variants into every module; `std::option::Option::{self, Some, None}` is the exact line. Section 3 shows how to do the same for your own enums, and why doing it carelessly bites.

<a id="enums-2"></a>

## 2. The three kinds of variant

A variant can carry nothing, positional values, or named fields. [The Reference](https://doc.rust-lang.org/reference/items/enumerations.html) calls these unit-like, tuple-like and struct-like variants; this post says unit, tuple and struct variants.

| Kind | Declaration | Building one | Use it when |
| :-- | :-- | :-- | :-- |
| Unit | `Cash` | `Payment::Cash` | A label is enough |
| Tuple | `Voucher(String)` | `Payment::Voucher(code)` | One or two values whose meaning the variant name makes obvious |
| Struct | `Card { last4: u16, contactless: bool }` | `Payment::Card { last4: 4242, contactless: true }` | The fields need names to be understood |

One enum can mix all three kinds, as `Payment` does. Here is each kind being built, with two details that save typing:

```rust
#[derive(Debug)]
enum Payment {
    Cash,
    Card { last4: u16, contactless: bool },
    Voucher(String),
}

fn describe(payment: &Payment) -> String {
    match payment {
        Payment::Cash => String::from("cash"),
        Payment::Card { last4, contactless } => format!("card ending {last4} (contactless: {contactless})"),
        Payment::Voucher(code) => format!("voucher {code}"),
    }
}

fn main() {
    let cash = Payment::Cash;

    // Field shorthand: a variable named `last4` fills the field named `last4`.
    let last4 = 4242;
    let card = Payment::Card { last4, contactless: false };

    let voucher = Payment::Voucher(String::from("FREE1"));

    // A tuple variant is also a function that builds the variant.
    let codes = vec![String::from("A1"), String::from("B2")];
    let vouchers: Vec<Payment> = codes.into_iter().map(Payment::Voucher).collect();

    println!("{cash:?}");
    println!("{card:?}");
    println!("{voucher:?}");
    println!("{vouchers:?}");
    for payment in [cash, card, voucher].into_iter().chain(vouchers) {
        println!("{}", describe(&payment));
    }
}
```

```text
Cash
Card { last4: 4242, contactless: false }
Voucher("FREE1")
[Voucher("A1"), Voucher("B2")]
cash
card ending 4242 (contactless: false)
voucher FREE1
voucher A1
voucher B2
```

**A tuple variant is a function.** `Payment::Voucher` on its own, without parentheses, behaves as a function of type `fn(String) -> Payment`. That is why `.map(Payment::Voucher)` works, and why `Some` can be passed around as a function; post 3 called `Some` a *variant constructor*. A unit variant is a plain value. A struct variant is neither: you must write the braces and every field.

**Choosing the kind.** Two numbers called `4242, true` mean little on their own; `last4` and `contactless` explain themselves. Reach for a struct variant as soon as a reader would have to guess what a position means, and for a unit variant whenever the name is the whole message. Tuple variants suit a single obvious payload: `Voucher(String)`, `Some(T)`, `Err(E)`.

<a id="enums-3"></a>

## 3. A variant is not a type

This is a point that trips up many people coming from other languages, so it gets its own section.

### The enum is the type; the variant is one of its values

`Payment::Card` is a way to build a `Payment`, and a way to recognise one. It is **not** a type of its own. You cannot write a function that accepts only cards:

```rust
// ✗ Does not compile. Error: E0573 expected type, found variant `Payment::Card`
enum Payment {
    Cash,
    Card { last4: u16, contactless: bool },
}

fn charge(card: Payment::Card) {
    let _ = card;
}

fn main() {
    charge(Payment::Card { last4: 4242, contactless: true });
    let _ = Payment::Cash;
}
```

The compiler even suggests the fix it thinks you meant: "try using the variant's enum". When some code genuinely needs *only* the card details, give those details their own struct and let the variant hold it:

```rust
#[derive(Debug)]
struct CardDetails {
    last4: u16,
    contactless: bool,
}

#[derive(Debug)]
enum Payment {
    Cash,
    Card(CardDetails),
}

fn charge(card: &CardDetails) -> String {
    let how = if card.contactless { "by tap" } else { "by chip and PIN" };
    format!("charging card ending {:04} {how}", card.last4)
}

fn main() {
    let payments = [Payment::Card(CardDetails { last4: 4242, contactless: true }), Payment::Cash];
    for payment in &payments {
        match payment {
            Payment::Card(details) => println!("{}", charge(details)),
            Payment::Cash => println!("cash, nothing to charge"),
        }
    }
    println!("{payments:?}");
}
```

```text
charging card ending 4242 by tap
cash, nothing to charge
[Card(CardDetails { last4: 4242, contactless: true }), Cash]
```

Now `charge` has a real type to name, and `Payment` still answers the "which one?" question. This struct-inside-a-variant shape is common in real code. The standard library's `IpAddr` is exactly this: `enum IpAddr { V4(Ipv4Addr), V6(Ipv6Addr) }`, two structs behind two variants.

### Variants live inside their enum's name

`Card` on its own means nothing to the compiler; `Payment::Card` does. That is true when you build a value **and** when you match one. The prefix is needed inside a pattern just as much as outside it:

```rust
// ✗ Does not compile. Error: E0422 cannot find struct, variant or union type `Card` in this scope
enum Payment {
    Cash,
    Card { last4: u16, contactless: bool },
}

fn main() {
    let payment = Payment::Card { last4: 4242, contactless: true };
    match payment {
        Card { last4, .. } => println!("card ending {last4:04}"),
        Payment::Cash => println!("cash"),
    }
}
```

What *does* let you drop the prefix is an import. `use Size::*;` brings every variant into scope, exactly as the prelude does for `Some` and `None`; `use Size::{Small, Large};` brings named ones:

```rust
#[derive(Debug)]
enum Size {
    Small,
    Medium,
    Large,
}

use Size::*;

fn cup_volume_ml(size: Size) -> u32 {
    match size {
        Small => 240,
        Medium => 350,
        Large => 470,
    }
}

fn main() {
    let order = Large;
    println!("{order:?}");
    println!("{} ml", cup_volume_ml(order));
    println!("{} ml", cup_volume_ml(Small) + cup_volume_ml(Medium));
}
```

```text
Large
470 ml
590 ml
```

Inside an `impl Size` block there is a third spelling, `Self::Small`, which section 4 uses. Two variants of one enum may share a field name, and they stay distinct, because the variant name is always part of the pattern:

```rust
#[derive(Debug)]
enum TillEvent {
    Opened { till: u8 },
    Closed { till: u8 },
}

fn main() {
    for event in [TillEvent::Opened { till: 1 }, TillEvent::Closed { till: 1 }] {
        match event {
            TillEvent::Opened { till } => println!("till {till} opened"),
            TillEvent::Closed { till } => println!("till {till} closed"),
        }
    }
}
```

```text
till 1 opened
till 1 closed
```

### The bare-name trap

Imports make the next mistake possible, so learn it now. In a pattern, a bare name that Rust cannot find as a variant or constant becomes a **new variable that matches anything**. If you forget the prefix and forget the import, you get a catch-all where you meant a specific variant. When the name is spelled exactly like a variant of the matched type, the compiler refuses:

```rust
// ✗ Does not compile. Error: E0170 pattern binding `Small` is named the same as one of the variants of the type `Size`
#[derive(Debug)]
enum Size {
    Small,
    Large,
}

fn main() {
    let size = Size::Large;
    match size {
        Small => println!("small"),
        _ => println!("something else"),
    }
}
```

The error ends with "help: to match on the variant, qualify the path: `Size::Small`" and notes that the lint `bindings_with_variant_name` is `deny` by default. So far so good. Now misspell the variant:

```rust
enum Size {
    Small,
    Large,
}

fn main() {
    let size = Size::Large;
    match size {
        Smol => println!("small?"),
        Size::Large => println!("large"),
    }
    let _ = Size::Small;
}
```

```text
small?
```

This **compiles**, and the arm for `Large` can never run: `Smol` is a fresh variable that matches every `Size`, including `Large`. The compiler does complain, but only with warnings, three of them:

```text
warning: unreachable pattern
  |         Smol => println!("small?"),
  |         ---- matches any value
  |         Size::Large => println!("large"),
  |         ^^^^^^^^^^^ no value can reach this
warning: unused variable: `Smol`
warning: variable `Smol` should have a snake case name
```

Any one of those warnings is telling you the truth. Treat "unreachable pattern" on an enum match as an error until you have read the arm it points at. The rule behind both examples is in [the Reference](https://doc.rust-lang.org/reference/patterns.html#identifier-patterns): a single name in a pattern is first looked up as a constant, a unit struct or a variant in scope, and only if nothing is found does it become a binding. The same rule is what lets a constant work as a pattern:

```rust
const FREE_CODE: &str = "FREE1";

fn main() {
    let code = String::from("FREE1");
    match code.as_str() {
        FREE_CODE => println!("free coffee"),
        other => println!("voucher {other}"),
    }
}
```

```text
free coffee
```

`FREE_CODE` is found, so it is compared; `other` is not found, so it binds. (The `.as_str()` is needed because string patterns match `&str`, not `String`; section 5 shows the error you get without it.)

<a id="enums-4"></a>

## 4. Creating and using enum values

### An enum value is an ordinary value: it moves

Everything post 1 said about ownership applies to enums unchanged. A `Payment::Voucher` owns its `String` the way a struct field would: move the `Payment` and the `String` moves with it; when the `Payment` goes out of scope, the `String` is freed. And a value of a plain-label enum is **not** copied just because it is small:

```rust
// ✗ Does not compile. Error: E0382 borrow of moved value: `first`
#[derive(Debug)]
enum Size {
    Small,
    Large,
}

fn main() {
    let first = Size::Small;
    let second = first;
    println!("{first:?} {second:?}");
    let _ = Size::Large;
}
```

"move occurs because `first` has type `Size`, which does not implement the `Copy` trait." Rust never assumes `Copy`; you ask for it, and the request is granted only when every field of every variant is itself `Copy`. A `String` field makes the derive fail with E0204, "this field does not implement `Copy`". For labels like `Size` the derive is the normal thing to do:

```rust
#[derive(Debug, Clone, Copy)]
enum Size {
    Small,
    Large,
}

fn main() {
    let first = Size::Small;
    let second = first;
    println!("{first:?} {second:?}");
    let _ = Size::Large;
}
```

```text
Small Small
```

The [Copy and Clone post](/rust/concepts/2025/05/21/rust-copy-clone.html) covers when copying is the right call. From here on `Size` derives `Clone, Copy` and `Payment` does not, so a `Size` copies and a `Payment` moves.

### Every variant has a number

Behind every variant is an integer, its **discriminant**. By default the first variant is 0 and each one after it counts up by one. You can set the numbers yourself, and for an enum whose variants carry no data you can read them with `as`:

```rust
#[derive(Debug, Clone, Copy)]
enum Size {
    Small,
    Medium,
    Large,
}

#[derive(Debug, Clone, Copy)]
#[repr(u16)]
enum CupMl {
    Small = 240,
    Medium = 350,
    Large = 470,
}

const PRICES_IN_PENCE: [u32; 3] = [250, 300, 350];

fn main() {
    println!("{} {} {}", Size::Small as u8, Size::Medium as u8, Size::Large as u8);
    println!("{} {} {} ml", CupMl::Small as u16, CupMl::Medium as u16, CupMl::Large as u16);
    println!("{} p", PRICES_IN_PENCE[Size::Medium as usize]);
    println!("{} byte, {} bytes", std::mem::size_of::<Size>(), std::mem::size_of::<CupMl>());
}
```

```text
0 1 2
240 350 470 ml
300 p
1 byte, 2 bytes
```

The rules, each checked on Rust 1.98.1:

- Unset discriminants continue from the previous one: `A = 5, B, C = 10, D` gives 5, 6, 10, 11. Two variants with the same number are an error (E0081).
- `as` goes from enum to integer only, and only for enums whose variants all carry no data. `Payment as u8` is E0605: "an `as` expression can be used to convert enum types to numeric types only if the enum type is unit-only or field-less". Casting to a float is E0606, and the compiler says "cast through an integer first".
- `#[repr(u16)]` fixes how the number is stored: as a `u16`, so `CupMl` is two bytes and every discriminant must fit in a `u16`. Use a `repr` when the number leaves the program: written to a file, sent over a socket, or handed to C. Without one the compiler chooses, and here it chose one byte for `Size`.
- With a `repr`, even variants that carry data may take explicit numbers, as the `TaggedPayment` enum in the next subsection shows; without one that is E0732. Rust 1.66 allowed this. Reading such a discriminant back still needs `unsafe`, so use `match` instead.

`as` has no reverse. Turning a number from a file back into a `Size` is a conversion that can fail, and Rust makes you say what happens to a number that fits no variant:

```rust
// ✗ Does not compile. Error: E0605 non-primitive cast: `i32` as `Size`
#[derive(Debug, Clone, Copy)]
enum Size {
    Small,
    Medium,
    Large,
}

fn main() {
    let size = 2 as Size;
    println!("{size:?}");
    let _ = (Size::Small, Size::Medium, Size::Large);
}
```

The idiom is `TryFrom`, which gives you `try_from` and `try_into` in one go:

```rust
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u8)]
enum Size {
    Small = 1,
    Medium = 2,
    Large = 3,
}

impl TryFrom<u8> for Size {
    type Error = String;

    fn try_from(code: u8) -> Result<Self, Self::Error> {
        match code {
            1 => Ok(Size::Small),
            2 => Ok(Size::Medium),
            3 => Ok(Size::Large),
            other => Err(format!("no size has code {other}")),
        }
    }
}

fn main() {
    println!("{:?}", Size::try_from(2));
    println!("{:?}", Size::try_from(9));
    let from_file: Result<Size, _> = 3u8.try_into();
    println!("{from_file:?}");
    println!("{}", Size::Large as u8);
}
```

```text
Ok(Medium)
Err("no size has code 9")
Ok(Large)
3
```

### How big is an enum?

Post 2 measured `Option` and `Result`. Here are the café enums, measured on this machine:

```rust
use std::mem::size_of;

#[allow(dead_code)]
enum Size {
    Small,
    Medium,
    Large,
}

#[allow(dead_code)]
enum Payment {
    Cash,
    Card { last4: u16, contactless: bool },
    Voucher(String),
}

#[allow(dead_code)]
#[repr(u8)]
enum TaggedPayment {
    Cash = 1,
    Card { last4: u16, contactless: bool } = 2,
    Voucher(String) = 3,
}

#[allow(dead_code)]
enum Big {
    Photo([u8; 1024]), // a latte-art photo attached to an order
    Nothing,
}

#[allow(dead_code)]
enum Boxed {
    Photo(Box<[u8; 1024]>),
    Nothing,
}

fn main() {
    println!("Size            {}", size_of::<Size>());
    println!("Option<Size>    {}", size_of::<Option<Size>>());
    println!("String          {}", size_of::<String>());
    println!("Payment         {}", size_of::<Payment>());
    println!("TaggedPayment   {}", size_of::<TaggedPayment>());
    println!("Option<Payment> {}", size_of::<Option<Payment>>());
    println!("Big             {}", size_of::<Big>());
    println!("Boxed           {}", size_of::<Boxed>());
}
```

```text
Size            1
Option<Size>    1
String          24
Payment         24
TaggedPayment   32
Option<Payment> 24
Big             1025
Boxed           8
```

Reading the numbers:

- **`Size` is one byte.** Something has to record which of the three labels a value holds. That record is the **tag**. The number it stores is the variant's *discriminant* from the previous subsection; with three variants one byte is plenty. A niche, explained in the next bullet, can hold the discriminant without a tag of its own, as `Payment` shows two bullets down.
- **`Option<Size>` is also one byte.** A byte has 256 values and `Size` uses three, so `None` can take a fourth. Post 2 called a spare bit pattern like this a *niche*.
- **`Payment` is 24 bytes, the same as a `String`.** You might expect 24 for the string plus a tag. There is no separate tag. When the value is `Cash` or `Card` there is no string, so the compiler writes a number no real string could have (a capacity above `isize::MAX`) into the capacity slot; that impossible number *is* the tag. The card number and flag go into bytes the string's other fields would have used. `Option<Payment>` fits in the same 24 bytes for the same reason.
- **`TaggedPayment` is 32 bytes.** `#[repr(u8)]` asks for an explicit one-byte tag. The tag then needs a slot of its own, and because the `String` inside must start at an 8-byte boundary, that slot costs eight bytes.
- **`Big` is 1025 bytes even when it holds `Nothing`.** An enum is always at least as large as its largest variant, because any variable of that type must be able to hold any variant. If one variant is huge and rare, put its payload in a `Box`, so the enum stores an 8-byte pointer instead. Clippy's `large_enum_variant` lint warns exactly here, when the two largest variants differ by more than 200 bytes (the default threshold): "large size difference between variants".

The only guaranteed rule in this list is the last one: at least as large as the largest variant. Where the tag lives and how a niche is used are the compiler's choices, and they may change between versions. If a layout must be stable, because the bytes cross into C or onto disk, ask for it with `#[repr(...)]`, as the previous subsection showed. The [`Option` representation guarantees](https://doc.rust-lang.org/std/option/index.html#representation) that post 2 listed are the documented exception: `Option<Box<T>>` and `Option<&T>` are promised to be the same size as the plain pointer.

### Same variant, ignoring the data?

Sometimes the question is "are these two payments the same kind?" with no interest in the card numbers. `std::mem::discriminant` (Rust 1.21) answers it with an opaque handle that supports `==` and hashing:

```rust
use std::mem::discriminant;

#[derive(Debug)]
enum Payment {
    Cash,
    Card { last4: u16 },
    Voucher(String),
}

fn same_method(a: &Payment, b: &Payment) -> bool {
    discriminant(a) == discriminant(b)
}

fn main() {
    let a = Payment::Voucher(String::from("FREE1"));
    let b = Payment::Voucher(String::from("B2"));
    let c = Payment::Card { last4: 4242 };
    println!("{}", same_method(&a, &b));
    println!("{}", same_method(&a, &c));
    println!("{}", matches!(c, Payment::Card { .. }));
    for payment in [&a, &b, &c, &Payment::Cash] {
        match payment {
            Payment::Cash => println!("cash"),
            Payment::Card { last4 } => println!("card ending {last4:04}"),
            Payment::Voucher(code) => println!("voucher {code}"),
        }
    }
}
```

```text
true
false
true
voucher FREE1
voucher B2
card ending 4242
cash
```

The handle is a `Discriminant<Payment>`, not a number, and it works for enums with data. For a yes-or-no question about one variant, `matches!` on the last line is the readable choice; section 5 covers it.

### Enums have methods, constants and traits

An `impl` block attaches behaviour to an enum exactly as it does to a struct. Inside it, `Self::Received` names a variant without repeating the enum's name. This is where `OrderStatus` earns its keep:

```rust
#[derive(Debug, PartialEq)]
enum OrderStatus {
    Received,
    Brewing,
    Ready { counter: u8 },
    Collected,
    Cancelled { reason: String },
}

impl OrderStatus {
    const COUNTERS: u32 = 3;

    fn label(&self) -> &'static str {
        match self {
            Self::Received => "received",
            Self::Brewing => "brewing",
            Self::Ready { .. } => "ready",
            Self::Collected => "collected",
            Self::Cancelled { .. } => "cancelled",
        }
    }

    fn is_open(&self) -> bool {
        !matches!(self, Self::Collected | Self::Cancelled { .. })
    }

    fn advance(self, ticket: u32) -> Self {
        match self {
            Self::Received => Self::Brewing,
            Self::Brewing => Self::Ready { counter: (ticket % Self::COUNTERS) as u8 + 1 },
            Self::Ready { .. } => Self::Collected,
            finished => finished,
        }
    }

    fn cancel(&mut self, reason: &str) {
        if self.is_open() {
            *self = Self::Cancelled { reason: reason.to_string() };
        }
    }
}

fn main() {
    let mut status = OrderStatus::Received;
    println!("{} (open: {})", status.label(), status.is_open());

    status = status.advance(7).advance(7);
    println!("{} (open: {})", status.label(), status.is_open());
    if let OrderStatus::Ready { counter } = &status {
        println!("collect at counter {counter}");
    }

    status.cancel("customer left");
    status.cancel("again"); // already finished: nothing happens
    if let OrderStatus::Cancelled { reason } = &status {
        println!("{}: {reason}", status.label());
    }
    println!("{}", status == OrderStatus::Collected);
}
```

```text
received (open: true)
ready (open: true)
collect at counter 2
cancelled: customer left
false
```

Four things happen here that a struct method would not show you:

- `label(&self)` matches on a **reference**. Rust lets a non-reference pattern such as `Self::Ready { .. }` look through the `&`, and any field it bound would come out as a reference, `&u8`. Section 5 explains this rule and what edition 2024 changed about it.
- `advance(self, ..)` takes the value **by value**, so the match may consume it and build the next state. The last arm, `finished => finished`, is a named catch-all: it binds the whole value, `Collected` or `Cancelled { .. }`, and hands it back unchanged. Unlike `_`, a name keeps the value. Section 6 explains when a catch-all like this becomes a trap.
- `cancel(&mut self, ..)` replaces the entire value with `*self = ...`. That is how an enum changes variant: you do not "switch the tag", you assign a new value.
- `COUNTERS` is an associated constant, read as `Self::COUNTERS`. Post 1's constant rules apply.

`#[derive]` works on enums too. The common set:

```rust
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
enum Size {
    Small,
    #[default]
    Medium,
    Large,
}

fn main() {
    println!("default: {:?}", Size::default());
    println!("{}", Size::Small < Size::Large);
    let mut sizes = vec![Size::Large, Size::Small, Size::Medium];
    sizes.sort();
    println!("{sizes:?}");
    let mut prices = HashMap::new();
    prices.insert(Size::Small, 250);
    prices.insert(Size::Large, 350);
    println!("{:?}", prices.get(&Size::Large));
}
```

```text
default: Medium
true
[Small, Medium, Large]
Some(350)
```

- `PartialEq` gives `==`; without it `Size::Small == Size::Large` is E0369, and the compiler suggests the derive. `Eq` adds the promise that every value equals itself, which `HashMap` keys need.
- `PartialOrd` and `Ord` order values **by discriminant first**, which is declaration order unless you set the numbers yourself, then by the fields inside the same variant: `Small < Medium < Large` here, and `Cash < Card { .. } < Voucher(..)` for `Payment` whatever the data. Reorder the variants, or renumber them, and every comparison changes, silently.
- `Hash` together with `Eq` lets the enum be a `HashMap` key.
- `Default` needs `#[default]` on exactly one variant, and that variant must be a unit variant (Rust 1.62). Putting it on `Voucher(String)` is an error: "the `#[default]` attribute may only be used on unit enum variants".
- `Copy` needs `Clone` and every field `Copy`, as this section began.

For text the customer sees, implement `Display` by hand. `Debug` output is for you, not for the receipt:

```rust
use std::fmt;

#[derive(Debug, Clone, Copy)]
enum Size {
    Small,
    Medium,
    Large,
}

impl fmt::Display for Size {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let word = match self {
            Size::Small => "small",
            Size::Medium => "medium",
            Size::Large => "large",
        };
        write!(f, "{word}")
    }
}

fn main() {
    let size = Size::Large;
    println!("One {size} latte, please");
    let label: String = size.to_string();
    println!("{label} / {size:?}");
    let _ = (Size::Small, Size::Medium);
}
```

```text
One large latte, please
large / Large
```

<a id="enums-5"></a>

## 5. Pattern matching in depth

Post 3 introduced `match`, `if let` and `let ... else` on `Option` and `Result`. Everything there carries over. This section adds what your own enums need: how a match takes data out, what that does to ownership, and the full toolbox of patterns.

### Four rules of `match`

1. The value being matched (the Reference calls it the *scrutinee*) is compared with the arms **from top to bottom**, and the first pattern that fits wins.
2. `match` is an **expression**. Every arm must produce the same type, or something that fits any type, such as `return`, `panic!()` or `continue`.
3. Every possible value must be covered. Section 6 is about this rule.
4. A pattern can **bind** names to the data inside a variant, and an arm can carry an extra condition, the **guard**.

```rust
#[derive(Debug, Clone, Copy)]
enum Size {
    Small,
    Medium,
    Large,
}

fn price_in_pence(size: Size, large_cups_in_stock: bool) -> u32 {
    let base = match size {
        Size::Small => 250,
        Size::Medium => 300,
        Size::Large if large_cups_in_stock => 350,
        Size::Large => return 0,
    };
    base + 20
}

fn main() {
    println!("{}", price_in_pence(Size::Small, true));
    println!("{}", price_in_pence(Size::Medium, true));
    println!("{}", price_in_pence(Size::Large, true));
    println!("{}", price_in_pence(Size::Large, false));
}
```

```text
270
320
370
0
```

`Size::Large if large_cups_in_stock` comes before the plain `Size::Large` arm because the first fit wins; swap them and large coffees are always free. `return 0` leaves the function, so it never has to produce a `u32` for `base`. Had one arm produced a number and another a string, the compiler would have refused with E0308, "`match` arms have incompatible types".

### Taking the data out, or only looking at it

A pattern that binds a name **moves** the data into that name, unless the data is `Copy`. For `Voucher(String)` that means the string leaves the enum:

```rust
// ✗ Does not compile. Error: E0382 borrow of partially moved value: `payment`
#[derive(Debug)]
enum Payment {
    Cash,
    Voucher(String),
}

fn main() {
    let payment = Payment::Voucher(String::from("FREE1"));
    match payment {
        Payment::Voucher(code) => println!("voucher {code}"),
        Payment::Cash => println!("cash"),
    }
    println!("{payment:?}");
}
```

The compiler's own suggestion, "borrow this binding in the pattern to avoid moving the value", is the `ref` keyword: `Payment::Voucher(ref code)`. The more common fix is to match a **reference**. Then every name the pattern binds becomes a reference automatically:

```rust
#[derive(Debug)]
enum Payment {
    Cash,
    Voucher(String),
}

fn main() {
    for payment in [Payment::Voucher(String::from("FREE1")), Payment::Cash] {
        match &payment {
            Payment::Voucher(code) => {
                let code: &String = code; // the binding is a reference, so nothing moved
                println!("voucher {code}");
            }
            Payment::Cash => println!("cash"),
        }
        println!("still here: {payment:?}");
    }
}
```

```text
voucher FREE1
still here: Voucher("FREE1")
cash
still here: Cash
```

Two more facts about moving:

- **A pattern moves only what it names.** `Payment::Voucher(_)` and `Payment::Card { .. }` bind nothing, so they move nothing.
- **The move is decided when the program is compiled, not when it runs.** `if let Payment::Voucher(code) = payment` moves the string on the path where the pattern matches, so `payment` counts as partially moved afterwards even if at run time it held `Cash`:

```rust
// ✗ Does not compile. Error: E0382 borrow of partially moved value: `payment`
#[derive(Debug)]
enum Payment {
    Cash,
    Voucher(String),
}

fn main() {
    let payment = Payment::Cash;
    if let Payment::Voucher(code) = payment {
        println!("{code}");
    }
    println!("{payment:?}");
}
```

Matching `&mut` works the same way and lets you change the data in place:

```rust
#[derive(Debug)]
enum OrderStatus {
    Ready { counter: u8 },
    Cancelled { reason: String },
}

fn main() {
    let mut status = OrderStatus::Ready { counter: 3 };
    if let OrderStatus::Ready { counter } = &mut status {
        *counter += 1; // counter: &mut u8
    }
    println!("{status:?}");

    status = OrderStatus::Cancelled { reason: String::from("out of oat milk") };
    match &mut status {
        OrderStatus::Cancelled { reason } => reason.push('!'), // reason: &mut String
        OrderStatus::Ready { .. } => {}
    }
    println!("{status:?}");
}
```

```text
Ready { counter: 4 }
Cancelled { reason: "out of oat milk!" }
```

One trap remains, and it is a very common enum error in real code. Inside a function that receives `&Payment` you cannot move the string out, because you do not own it. Writing `match *p` to "get at the value" is E0507:

```rust
// ✗ Does not compile. Error: E0507 cannot move out of `p` as enum variant `Voucher` which is behind a shared reference
enum Payment {
    Cash,
    Voucher(String),
}

fn voucher_code(p: &Payment) -> String {
    match *p {
        Payment::Voucher(code) => code,
        Payment::Cash => String::new(),
    }
}

fn main() {
    let payment = Payment::Voucher(String::from("FREE1"));
    println!("{}", voucher_code(&payment));
}
```

Either return a borrow, `&str`, or pay for a copy with `.clone()`:

```rust
enum Payment {
    Cash,
    Voucher(String),
}

fn voucher_code(p: &Payment) -> &str {
    match p {
        Payment::Voucher(code) => code,
        Payment::Cash => "",
    }
}

fn voucher_code_owned(p: &Payment) -> String {
    match p {
        Payment::Voucher(code) => code.clone(),
        Payment::Cash => String::new(),
    }
}

fn main() {
    let payment = Payment::Voucher(String::from("FREE1"));
    println!("{} {}", voucher_code(&payment), voucher_code_owned(&payment));
    println!("{:?}", voucher_code(&Payment::Cash));
}
```

```text
FREE1 FREE1
""
```

### Match ergonomics, and what edition 2024 reserves

The rule that made `match &payment` bind `code` as `&String` is called **match ergonomics**, or *default binding modes*, and arrived in Rust 1.26. Before it you wrote `&Payment::Voucher(ref code)`. The mechanism: when a non-reference pattern meets a reference, the compiler looks through the reference and switches the *default binding mode* from "move" to "ref" (or to "ref mut" for `&mut`); every plain name bound from then on becomes a reference. [The Reference](https://doc.rust-lang.org/reference/patterns.html#binding-modes) spells out the rules.

Edition 2024 reserves some syntax inside such patterns. Once the compiler is implicitly borrowing, you may no longer write `&`, `mut`, `ref` or `ref mut` deeper in the pattern. The edition guide's own examples use arrays; these two are adapted from them:

```rust
// ✗ Does not compile in edition 2024. Error: cannot explicitly dereference within an implicitly-borrowing pattern (edition 2021 compiles and prints 0)
fn main() {
    let [&x] = &[&0];
    println!("{x}");
}
```

```rust
// ✗ Does not compile in edition 2024. Error: cannot mutably bind by value within an implicitly-borrowing pattern (edition 2021 compiles and prints 1)
fn main() {
    let [mut x] = &[0];
    x += 1;
    println!("{x}");
}
```

Under edition 2021 both compile, and they print `0` and `1`, because in that edition `&` and `mut` silently reset the binding mode to "move" and changed the bound type. Edition 2024 asks you to be explicit instead, and the explicit form compiles in every edition:

```rust
fn main() {
    let &[&x] = &[&0];
    println!("{x}");
    let &[mut y] = &[0];
    y += 1;
    println!("{y}");
}
```

```text
0
1
```

For enums this rarely bites, because the everyday form is already fine: match the reference, name the fields, and take the references you are given. Write the leading `&` yourself only when you want the value copied out:

```rust
#[derive(Debug)]
enum Payment {
    Card { last4: u16 },
}

fn main() {
    let payment = Payment::Card { last4: 4242 };

    let Payment::Card { last4 } = &payment; // last4: &u16, borrowed
    let borrowed: &u16 = last4;

    let &Payment::Card { last4: copied } = &payment; // copied: u16, a copy
    let copy: u16 = copied;

    println!("{borrowed} {copy} {payment:?}");
}
```

```text
4242 4242 Card { last4: 4242 }
```

The `let` here is legal because `Payment` has exactly one variant, so the pattern cannot fail. Section 6 shows what happens when it can.

### The pattern toolbox

One function shows most of what a pattern can say. Read the arms top to bottom; the comments give each feature its name:

```rust
#[derive(Debug)]
enum Payment {
    Cash,
    Card { last4: u16, contactless: bool },
    Voucher(String),
}

fn describe(payment: &Payment, pence: u32) -> String {
    match (payment, pence) {
        // Range pattern with an exclusive end (Rust 1.80): 0 to 99.
        (Payment::Cash, 0..100) => String::from("small cash"),
        // `n @ pattern` binds the value and tests it at the same time.
        (Payment::Cash, n @ 100..=1000) => format!("cash {n}"),
        (Payment::Cash, n) => format!("big cash {n}"),
        // A literal inside a struct pattern, `..` for the rest, and a guard.
        (Payment::Card { contactless: true, .. }, n) if n <= 5000 => String::from("tap"),
        // `card @` keeps the whole value while `last4` names one field.
        (card @ Payment::Card { last4, .. }, _) => format!("chip and PIN {last4:04} {card:?}"),
        // A guard may read the bound name; `code` is a `&String` here.
        (Payment::Voucher(code), _) if code == "FREE1" => String::from("free"),
        // An or-pattern: any of the three.
        (Payment::Voucher(_), 1 | 2 | 3) => String::from("tiny voucher"),
        (Payment::Voucher(code), _) => format!("voucher {code}"),
    }
}

fn main() {
    println!("{}", describe(&Payment::Cash, 50));
    println!("{}", describe(&Payment::Cash, 1000));
    println!("{}", describe(&Payment::Cash, 1001));
    println!("{}", describe(&Payment::Card { last4: 4242, contactless: true }, 5000));
    println!("{}", describe(&Payment::Card { last4: 4242, contactless: true }, 5001));
    println!("{}", describe(&Payment::Voucher(String::from("FREE1")), 2));
    println!("{}", describe(&Payment::Voucher(String::from("X9")), 2));
    println!("{}", describe(&Payment::Voucher(String::from("X9")), 4));
}
```

```text
small cash
cash 1000
big cash 1001
tap
chip and PIN 4242 Card { last4: 4242, contactless: true }
free
tiny voucher
voucher X9
```

Notes on the toolbox:

- **Matching several things at once** is a tuple scrutinee, `(payment, pence)`. There is nothing special about it: a tuple pattern takes the tuple apart.
- **Ranges**: `a..=b` is inclusive, `a..b` excludes `b` (allowed in patterns since Rust 1.80), and `..b` or `a..` leave one end open. A range must not be empty.
- **Or-patterns** with `|` can sit anywhere inside a pattern, `Some(1 | 2)` included, since Rust 1.53.
- **`..`** stands for "the fields I did not mention" in struct and tuple patterns. It is required when you skip fields.
- **Guards** run after the pattern matched and may read the names it bound. Section 6 has a warning about them.
- **Patterns nest.** `Some(Payment::Card { .. })` reaches through an `Option` into a `Payment` in one step:

```rust
#[derive(Debug)]
enum Payment {
    Cash,
    Card { last4: u16 },
}

fn main() {
    let payments = [None, Some(Payment::Cash), Some(Payment::Card { last4: 4242 })];
    for payment in &payments {
        match payment {
            Some(Payment::Card { last4 }) => println!("card ending {last4:04}"),
            Some(Payment::Cash) => println!("cash"),
            None => println!("not paid yet"),
        }
    }
}
```

```text
not paid yet
cash
card ending 4242
```

**Strings in patterns.** A string literal pattern has type `&str`. Matching a `String` against it does not compile:

```rust
// ✗ Does not compile. Error: E0308 mismatched types, expected `String`, found `&str`
fn main() {
    let drink = String::from("latte");
    match drink {
        "latte" => println!("milk"),
        _ => println!("something else"),
    }
}
```

Match `drink.as_str()` instead, and remember that strings, like numbers, have no complete list of values, so a `_` or a name is required at the end:

```rust
fn main() {
    let drink = String::from("latte");
    match drink.as_str() {
        "latte" | "flat white" => println!("milk"),
        "espresso" => println!("no milk"),
        other => println!("unknown drink {other}"),
    }
}
```

```text
milk
```

### Shorter forms

Every one of these is a `match` in disguise, and everything above about moving and borrowing applies to them.

```rust
#[derive(Debug)]
enum Payment {
    Cash,
    Card { last4: u16 },
    Voucher(String),
}

/// `let ... else`: bind one variant or leave. The else block must diverge.
fn voucher_code(payment: &Payment) -> Option<&str> {
    let Payment::Voucher(code) = payment else {
        return None;
    };
    Some(code)
}

fn main() {
    // `if let` with `else if let`: a match written as a chain of questions.
    let payment = Payment::Card { last4: 4242 };
    if let Payment::Cash = payment {
        println!("cash");
    } else if let Payment::Card { last4 } = &payment {
        println!("card ending {last4:04}");
    } else {
        println!("voucher");
    }

    // `matches!`: a yes-or-no answer, with an optional guard.
    let tip = Some(20_u32);
    println!("{}", matches!(payment, Payment::Card { .. }));
    println!("{}", matches!(tip, Some(t) if t >= 10));

    // `while let`: keep taking while the pattern fits.
    let mut queue = vec![
        Payment::Cash,
        Payment::Voucher(String::from("A1")),
        Payment::Cash,
    ];
    while let Some(next) = queue.pop() {
        println!("{next:?} -> {:?}", voucher_code(&next));
    }
}
```

```text
card ending 4242
true
true
Cash -> None
Voucher("A1") -> Some("A1")
Cash -> None
```

`let ... else` (Rust 1.65) needs an `else` block that cannot fall through: `return`, `break`, `continue` or a panic. `matches!` (Rust 1.42) expands to a `match` with a `true` arm and a `false` arm, so it takes the same patterns and guards. Since Rust 1.96 there is also `std::assert_matches!`, for tests; it is not in the prelude, so write `use std::assert_matches;` first, and when it fails it prints the actual value.

**Let chains** (Rust 1.88) let one `if` ask several questions, mixing `let` patterns and plain conditions with `&&`. They are available only in edition 2024:

```rust
// Needs edition 2024 (Rust 1.88 or newer). Under edition 2021 the compiler says: "let chains are only allowed in Rust 2024 or later".
enum Payment {
    Cash,
    Card { last4: u16 },
}

fn main() {
    let payment = Some(Payment::Card { last4: 4242 });
    let tip = Some(20_u32);
    if let Some(Payment::Card { last4 }) = &payment
        && let Some(t) = tip
        && t > 10
    {
        println!("card ending {last4:04} with a {t} p tip");
    }
    let _ = Payment::Cash;
}
```

```text
card ending 4242 with a 20 p tip
```

Before let chains you nested two `if let`s, or wrote `matches!` twice. The [1.88 release notes](https://blog.rust-lang.org/2025/06/26/Rust-1.88.0/) explain why the feature depends on the edition-2024 temporary-scope rule that post 3 mentioned.

<a id="enums-6"></a>

## 6. Exhaustiveness and wildcards

### The compiler counts your arms

A `match` must cover every value the scrutinee can hold. Miss a variant and the program does not compile. This is an error, never a warning:

```rust
// ✗ Does not compile. Error: E0004 non-exhaustive patterns: `Payment::Card { .. }` not covered
enum Payment {
    Cash,
    Card { last4: u16 },
    Voucher(String),
}

fn main() {
    let payment = Payment::Card { last4: 4242 };
    match payment {
        Payment::Cash => println!("cash"),
        Payment::Voucher(code) => println!("voucher {code}"),
    }
}
```

The message names exactly what is missing and offers an arm to paste in, `Payment::Card { .. } => todo!()`. This check is the reason to model choices as enums at all: add a variant and the compiler walks you to every `match` that must now decide what to do with it.

### `_` hides the future

A wildcard arm `_` matches everything, and a match that ends with one is exhaustive by definition. It is also blind to change:

```rust
#[derive(Debug)]
enum Payment {
    Cash,
    Card { last4: u16 },
    Voucher(String),
    Refund { original_ticket: u32 }, // added months after the match below was written
}

fn takings_line(payment: &Payment, pence: u32) -> String {
    match payment {
        Payment::Cash => format!("+{pence} p in the drawer"),
        _ => format!("+{pence} p by other means"),
    }
}

fn describe(payment: &Payment) -> String {
    match payment {
        Payment::Cash => String::from("cash"),
        Payment::Card { last4 } => format!("card {last4:04}"),
        Payment::Voucher(code) => format!("voucher {code}"),
        Payment::Refund { original_ticket } => format!("refund of #{original_ticket}"),
    }
}

fn main() {
    let refund = Payment::Refund { original_ticket: 17 };
    println!("{}: {}", describe(&refund), takings_line(&refund, 350));
    let others = [Payment::Cash, Payment::Card { last4: 4242 }, Payment::Voucher(String::from("B2"))];
    for payment in &others {
        println!("{}: {}", describe(payment), takings_line(payment, 350));
    }
}
```

```text
refund of #17: +350 p by other means
cash: +350 p in the drawer
card 4242: +350 p by other means
voucher B2: +350 p by other means
```

A refund is counted as income, and nothing complained. The `_` arm was written when it meant "card or voucher"; it silently grew to mean "or refund". A named catch-all such as `other => ...` is the same trap with a name.

The habit that avoids this: on enums you own, list the variants. When several need the same treatment, group them with `|` rather than reaching for `_`:

```text
Payment::Card { .. } | Payment::Voucher(_) => format!("+{pence} p by other means"),
```

Now adding `Refund` makes this match fail to compile, which is exactly the reminder you wanted. Keep `_` for values that genuinely have no complete list, such as numbers and strings, and for enums from other crates marked `#[non_exhaustive]`, below.

### Guards do not count

An arm with a guard covers nothing as far as the exhaustiveness check is concerned, because the compiler does not reason about the condition:

```rust
// ✗ Does not compile. Error: E0004 non-exhaustive patterns: `i32::MIN..=i32::MAX` not covered
fn main() {
    let change = 5_i32;
    let sign = match change {
        n if n > 0 => "customer is owed change",
        n if n <= 0 => "nothing owed",
    };
    println!("{sign}");
}
```

The note in the message says it plainly: "match arms with guards don't count towards exhaustivity". You and I can see that the two guards cover every integer; the compiler does not try. Make the last arm unguarded, `n => "nothing owed"`, and it compiles.

### `#[non_exhaustive]`: promising that variants will be added

A library that expects to add variants later marks the enum `#[non_exhaustive]` (Rust 1.40). Inside the crate that defines the enum, the attribute changes nothing:

```rust
#[non_exhaustive]
#[derive(Debug, Clone, Copy)]
enum Size {
    Small,
    Medium,
    Large,
}

fn main() {
    for size in [Size::Small, Size::Medium, Size::Large] {
        let ml = match size {
            Size::Small => 240,
            Size::Medium => 350,
            Size::Large => 470,
        };
        println!("{size:?} {ml} ml");
    }
}
```

```text
Small 240 ml
Medium 350 ml
Large 470 ml
```

Every *other* crate must add a wildcard arm, because the next version of the library may add a `Size::ExtraLarge` and their code has to keep compiling. Here is the same enum in a library crate, and a program that uses it without the wildcard:

```rust
// ✎ menu_lib.rs: compiled as a library crate with `rustc --edition 2024 --crate-type lib --crate-name menu_lib menu_lib.rs`
#[non_exhaustive]
#[derive(Debug, Clone, Copy)]
pub enum Size {
    Small,
    Medium,
    Large,
}
```

```rust
// ✎ main.rs: compiled against that library with `rustc --edition 2024 --extern menu_lib=libmenu_lib.rlib main.rs`
use menu_lib::Size;

fn main() {
    let size = Size::Small;
    match size {
        Size::Small => println!("small"),
        Size::Medium => println!("medium"),
        Size::Large => println!("large"),
    }
}
```

Compiling `main.rs` gives this, abbreviated:

```text
error[E0004]: non-exhaustive patterns: `_` not covered
  |     match size {
  |           ^^^^ pattern `_` not covered
  = note: `Size` is marked as non-exhaustive, so a wildcard `_` is necessary to match exhaustively
```

Adding `_ => println!("a size added later")` makes it compile. Use the attribute on enums you publish and expect to grow. Do not use it on enums that only your own code matches: in the crate that defines the enum it does nothing, and in your other crates it only switches off the check that this section began with. (In Cargo, `src/lib.rs` and `src/main.rs` of one package are already two crates, so the attribute bites between them.)

### An enum with no variants

`enum Never {}` has no variants, so no value of that type can ever exist. That sounds useless, and it is exactly what you want for "this cannot happen". A `Result<u32, Never>` can only ever be `Ok`, and since Rust 1.82 the compiler lets you leave out the impossible `Err` arm when you match by value:

```rust
enum Never {}

fn parse_ticket(text: &str) -> Result<u32, Never> {
    Ok(text.len() as u32)
}

fn main() {
    let Ok(n) = parse_ticket("latte");
    println!("{n}");

    let r: Result<u32, std::convert::Infallible> = Ok(3);
    let Ok(m) = r;
    println!("{m}");

    let never_made: Vec<Never> = Vec::new();
    println!("{}", never_made.len());
}
```

```text
5
3
0
```

The standard library's version of `Never` is `std::convert::Infallible`, used by conversions that cannot fail. Two limits: a `Vec<Never>` is fine, but it will always be empty; and when the empty type sits behind a reference you must still write the arm, as `Err(e) => match *e {}`. Post 2 explained that the never type `!` is stable only as a function's return type on Rust 1.98.1, so an empty enum is the stable way to say "no value" in a type parameter.

### `let` needs a pattern that cannot fail

`let` takes a value apart without asking "did it match?", so its pattern must always fit. A pattern that could fail is *refutable*, and `let` refuses it:

```rust
// ✗ Does not compile. Error: E0005 refutable pattern in local binding
enum Payment {
    Cash,
    Voucher(String),
}

fn main() {
    let payment = Payment::Cash;
    let Payment::Voucher(code) = payment;
    println!("{code}");
}
```

The message ends with the fix: "you might want to use `let...else` to handle the variant that isn't matched". For a single-variant enum, a tuple, a struct, or `Result<T, Never>`, the plain `let` is fine, as sections 5 and 6 showed.

<a id="enums-7"></a>

## 7. Best practices and pitfalls

### Make impossible states impossible

The payment could have been a struct of flags. Watch what that allows:

```rust
#[derive(Debug)]
struct PaymentFlags {
    is_cash: bool,
    card_last4: Option<u16>,
    voucher: Option<String>,
}

fn main() {
    let nonsense = PaymentFlags {
        is_cash: true,
        card_last4: Some(4242),
        voucher: Some(String::from("FREE1")),
    };
    if nonsense.is_cash && nonsense.card_last4.is_some() && nonsense.voucher.is_some() {
        println!("paid three ways at once: {nonsense:?}");
    }
}
```

```text
paid three ways at once: PaymentFlags { is_cash: true, card_last4: Some(4242), voucher: Some("FREE1") }
```

A payment that is cash *and* card *and* voucher compiles and prints. Every function that receives a `PaymentFlags` must decide what that combination means, and each will decide differently. The enum `Payment` cannot express the combination at all, so no function needs a rule for it. That is the whole design lesson of this post: when the possibilities are a list, write the list as an enum and let the compiler rule out everything else.

The same idea drove `OrderStatus`: a `Ready` order has a counter and a `Cancelled` order has a reason, and neither field exists on the other states, so there is no "counter of a cancelled order" to get wrong.

### A recursive enum needs a `Box`

A receipt is a line followed by the rest of the receipt, until the end. Written directly, that definition does not compile:

```rust
// ✗ Does not compile. Error: E0072 recursive type `Receipt` has infinite size
enum Receipt {
    End,
    Line { item: String, pence: u32, rest: Receipt },
}

fn main() {
    let _ = Receipt::End;
    let _ = Receipt::Line { item: String::new(), pence: 0, rest: Receipt::End };
}
```

Section 4 said an enum is at least as large as its largest variant. A `Line` contains a `Receipt`, which may be a `Line`, which contains a `Receipt`: the size never settles. The compiler says how to stop that: "insert some indirection (e.g., a `Box`, `Rc`, or `&`) to break the cycle". A `Box` is a fixed-size pointer to storage elsewhere, so the enum's size settles:

```rust
enum Receipt {
    End,
    Line { item: String, pence: u32, rest: Box<Receipt> },
}

fn total(receipt: &Receipt) -> u32 {
    match receipt {
        Receipt::End => 0,
        Receipt::Line { pence, rest, .. } => pence + total(rest),
    }
}

fn print_lines(receipt: &Receipt) {
    if let Receipt::Line { item, pence, rest } = receipt {
        println!("{item}: {pence} p");
        print_lines(rest);
    }
}

fn main() {
    let receipt = Receipt::Line {
        item: String::from("latte"),
        pence: 320,
        rest: Box::new(Receipt::Line {
            item: String::from("croissant"),
            pence: 250,
            rest: Box::new(Receipt::End),
        }),
    };
    print_lines(&receipt);
    println!("total {} p", total(&receipt));
}
```

```text
latte: 320 p
croissant: 250 p
total 570 p
```

`total(rest)` and `print_lines(rest)` pass a `&Box<Receipt>` where a `&Receipt` is expected; the automatic dereference of `Box` handles that. The [linked-list post](/rust/concepts/2025/05/07/rust-linked-list.html) builds on exactly this shape.

### Keep each decision in one place

Every `match` on `OrderStatus` is a place that must change when a variant is added. Ten matches scattered across the program are ten places. The methods in section 4, `label`, `is_open` and `advance`, make the enum answer its own questions, so the rest of the program asks `status.is_open()` and never repeats the list of finished states. When you find the same `match` in a second file, move it into an `impl`.

### Enum or trait object? Enum with generics?

An enum is a **closed** list: every possibility is known when you compile, and the compiler checks every match against that list. A trait object, `Box<dyn Trait>`, is an **open** list: other modules and other crates can add kinds you never see, and you cannot match on them. Choose the enum when you own the list and want exhaustiveness; choose the trait object when the point is that the list is extensible. The [trait objects post](/rust/concepts/2025/10/23/rust-dyn.html) develops the second half.

Enums take type parameters and lifetimes exactly as structs do. `Option<T>` is the everyday example; here are two more, plus the standard library's `Cow`, an enum that says "borrowed or owned":

```rust
use std::borrow::Cow;

#[derive(Debug)]
enum Either<L, R> {
    Left(L),
    Right(R),
}

#[derive(Debug)]
enum CupLabel<'a> {
    Printed(&'a str),
    Custom(String),
}

fn describe(value: &Either<u32, String>) -> String {
    match value {
        Either::Left(n) => format!("number {n}"),
        Either::Right(s) => format!("text {s}"),
    }
}

fn label_for(name: Option<&str>) -> Cow<'_, str> {
    match name {
        Some(n) => Cow::Borrowed(n),
        None => Cow::Owned(String::from("Customer")),
    }
}

fn main() {
    let a: Either<u32, String> = Either::Left(3);
    let b: Either<u32, String> = Either::Right(String::from("three"));
    println!("{a:?} {b:?} / {} / {}", describe(&a), describe(&b));

    let name = String::from("Amrit");
    let printed = CupLabel::Printed(&name);
    let custom = CupLabel::Custom(String::from("Birthday girl"));
    for label in [&printed, &custom] {
        let shown: &str = match label {
            CupLabel::Printed(s) => s,
            CupLabel::Custom(s) => s,
        };
        println!("{label:?} says {shown}");
    }

    println!("{} / {}", label_for(Some("Amrit")), label_for(None));
}
```

```text
Left(3) Right("three") / number 3 / text three
Printed("Amrit") says Amrit
Custom("Birthday girl") says Birthday girl
Amrit / Customer
```

`CupLabel<'a>` carries a lifetime because one variant borrows; the [ownership post](/rust/concepts/2025/02/09/rust-ownership.html) explains such annotations. `Cow` avoids allocating a `String` when a borrowed name will do, and matches like any other enum.

### Pitfalls at a glance

| You wrote | What went wrong | Do this instead |
| :-- | :-- | :-- |
| `Small => ...` without `use Size::*` | A new variable that matches anything; E0170 if the name is a variant, only warnings if misspelt | `Size::Small`, or `Self::Small` inside an `impl`; read "unreachable pattern" warnings |
| `_ => ...` on your own enum | New variants are swallowed silently | List the variants; group those that share treatment with an or-pattern |
| `match payment { Voucher(code) => ... }` then use `payment` | The `String` moved out; E0382 | `match &payment`, or `ref code` |
| `match *p { .. }` with `p: &Payment` | Moving out from behind a reference; E0507 | `match p`, return `&str` or `.clone()` |
| `let b = a;` for a unit-only enum, then use `a` | Enums are not `Copy` by default; E0382 | `#[derive(Clone, Copy)]` when every field is `Copy` |
| `a == b` on an enum | No `PartialEq`; E0369 | `#[derive(PartialEq)]` |
| `3 as Size` | No integer-to-enum cast; E0605 | `impl TryFrom<u8> for Size` |
| `payment as u8` with data-carrying variants | Only data-free enums cast; E0605 | `std::mem::discriminant`, or a method that returns a code |
| `n if n > 0 => .., n if n <= 0 => ..` | Guards do not count; E0004 | Make the last arm unguarded |
| `match name { "latte" => .. }` on a `String` | Literal patterns are `&str`; E0308 | `match name.as_str()` |
| `let Payment::Voucher(code) = payment;` | Refutable pattern; E0005 | `let ... else`, `if let` or `match` |
| `Line { rest: Receipt }` | Infinite size; E0072 | `Box<Receipt>` |
| `#[default]` on `Voucher(String)` | Only unit variants may be the default | Choose a unit variant or write `impl Default` |
| `pub Large` inside an enum | Variants always share the enum's visibility; E0449 | Make the enum `pub`; you cannot hide one variant |
| `Payment::Cheque` | No such variant; E0599 | Check the declaration |

<a id="enums-8"></a>

## 8. Summary and reference

### Variant kinds

| Variant | Kind | Fields | Build | Match |
| :-- | :-- | :-- | :-- | :-- |
| `Cash` | Unit | None | `Payment::Cash` | `Payment::Cash` |
| `Voucher(String)` | Tuple | Positional | `Payment::Voucher(code)` | `Payment::Voucher(code)`, `Payment::Voucher(_)` |
| `Card { last4: u16, contactless: bool }` | Struct | Named | `Payment::Card { last4, contactless: true }` | `Payment::Card { last4, .. }` |

All three are values or constructors of the one type `Payment`. None of them is a type.

### Which form of matching?

| You want to | Write | Notes |
| :-- | :-- | :-- |
| Handle every variant | `match value { ... }` | Exhaustive; an expression; first fit wins |
| Handle one variant, ignore the rest | `if let Pat = value { ... }` | May take `else` and `else if let` |
| Bind one variant or leave the function | `let Pat = value else { return ... };` | The `else` must diverge (Rust 1.65) |
| Loop while a pattern fits | `while let Some(x) = stack.pop() { ... }` | |
| A yes-or-no answer | `matches!(value, Pat)` | Accepts guards (Rust 1.42) |
| Several patterns and conditions in one `if` | `if let A = x && let B = y && cond { ... }` | Edition 2024 only (Rust 1.88) |
| Look without moving | `match &value`, `if let Pat = &value` | Bindings become references |
| Change in place | `match &mut value` | Bindings become `&mut` |

### The complete café example

Everything above in one program: sizes with a price table, payments with rules, orders that move through their states, and a receipt at the end. Read it top to bottom and check that you can predict the output before running it.

```rust
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum Size {
    Small,
    #[default]
    Medium,
    Large,
}

impl Size {
    const PRICES_IN_PENCE: [u32; 3] = [250, 300, 350];

    fn price_in_pence(self) -> u32 {
        Self::PRICES_IN_PENCE[self as usize]
    }
}

impl fmt::Display for Size {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let word = match self {
            Size::Small => "small",
            Size::Medium => "medium",
            Size::Large => "large",
        };
        write!(f, "{word}")
    }
}

#[derive(Debug)]
enum Payment {
    Cash,
    Card { last4: u16, contactless: bool },
    Voucher(String),
}

#[derive(Debug)]
enum OrderStatus {
    Received,
    Brewing,
    Ready { counter: u8 },
    Collected,
    Cancelled { reason: String },
}

impl OrderStatus {
    /// Move one step forward. Finished orders stay where they are.
    fn advance(&mut self, ticket: u32) {
        *self = match self {
            Self::Received => Self::Brewing,
            Self::Brewing => Self::Ready { counter: (ticket % 3) as u8 + 1 },
            Self::Ready { .. } => Self::Collected,
            Self::Collected | Self::Cancelled { .. } => return,
        };
    }
}

struct Order {
    ticket: u32,
    size: Size,
    payment: Payment,
    status: OrderStatus,
}

/// What the till charges, in pence, after the payment rules.
fn amount_due(order: &Order) -> u32 {
    let price = order.size.price_in_pence();
    match &order.payment {
        Payment::Cash => price,
        Payment::Card { contactless: true, .. } => price,
        Payment::Card { .. } => price + 20,
        Payment::Voucher(code) if code == "FREE1" => 0,
        Payment::Voucher(_) => price.saturating_sub(100),
    }
}

fn paid_with(payment: &Payment) -> String {
    match payment {
        Payment::Cash => String::from("cash"),
        Payment::Card { last4, contactless } => {
            let how = if *contactless { "tap" } else { "chip and PIN" };
            format!("card ending {last4:04} ({how})")
        }
        Payment::Voucher(code) => format!("voucher {code}"),
    }
}

fn main() {
    let mut orders = vec![
        Order { ticket: 1, size: Size::Large, payment: Payment::Cash, status: OrderStatus::Received },
        Order {
            ticket: 2,
            size: Size::default(),
            payment: Payment::Card { last4: 4242, contactless: false },
            status: OrderStatus::Received,
        },
        Order {
            ticket: 3,
            size: Size::Small,
            payment: Payment::Voucher(String::from("FREE1")),
            status: OrderStatus::Received,
        },
    ];

    // The barista moves every order forward twice: Received -> Brewing -> Ready.
    for order in &mut orders {
        order.status.advance(order.ticket);
        order.status.advance(order.ticket);
    }

    // Order 2 is cancelled before it is collected.
    let Some(second) = orders.iter_mut().find(|order| order.ticket == 2) else {
        panic!("order 2 must exist");
    };
    second.status = OrderStatus::Cancelled { reason: String::from("customer left") };

    let mut takings = 0;
    for order in &orders {
        match &order.status {
            OrderStatus::Ready { counter } => {
                let due = amount_due(order);
                takings += due;
                println!(
                    "#{} {} coffee, {due} p, {}: collect at counter {counter}",
                    order.ticket,
                    order.size,
                    paid_with(&order.payment)
                );
            }
            OrderStatus::Cancelled { reason } => println!("#{} cancelled: {reason}", order.ticket),
            other => println!("#{} still {other:?}", order.ticket),
        }
    }
    println!("Takings: {takings} p");
}
```

```text
#1 large coffee, 350 p, cash: collect at counter 2
#2 cancelled: customer left
#3 small coffee, 0 p, voucher FREE1: collect at counter 1
Takings: 350 p
```

Things worth noticing in the program:

- `advance(&mut self, ..)` matches on `self` to *read* the current state and then assigns the new state with `*self = ...`. The `return` in the last arm leaves finished orders untouched; without it every arm would have to produce a value.
- `order.status.advance(order.ticket)` borrows one field mutably while reading another. Different fields of one struct may be borrowed independently.
- `amount_due` matches `&order.payment` and lets the guard read `code` as a `&String`.
- The final `match` ends with a named catch-all, `other`. That is acceptable here because the three remaining states really do get the same treatment, and `other` is printed rather than ignored. It is still a place to revisit when `OrderStatus` grows.

### Version notes: a version is not an edition

Post 1 explained the difference: a version is the compiler you have, an edition is a setting in `Cargo.toml`. For enums and patterns these are the ones that matter:

| Feature | Available since | Edition requirement |
| :-- | :-- | :-- |
| `std::mem::discriminant` | Rust 1.21 | None |
| Match ergonomics: `match &value` binds references | Rust 1.26 | None |
| `#[non_exhaustive]` | Rust 1.40 | None |
| `matches!` | Rust 1.42 | None |
| Or-patterns nested inside other patterns, as in `Some(1 or 2)` written with the vertical bar | Rust 1.53 | None |
| Sub-bindings after `@`, as in `card @ Payment::Card { last4, .. }` | Rust 1.56 | None |
| `#[default]` on a unit variant with `derive(Default)` | Rust 1.62 | None |
| `let ... else` | Rust 1.65 | None |
| Explicit discriminants on variants with fields, under a `#[repr]` | Rust 1.66 | None |
| Exclusive range patterns, `0..100` | Rust 1.80 | None |
| Omitting arms for empty types, `let Ok(x) = result;` | Rust 1.82 | None |
| Reserved pattern syntax: no `&`, `mut`, `ref` while implicitly borrowing | Rust 1.85 | Edition 2024 |
| Let chains, `if let A = x && let B = y` | Rust 1.88 | Edition 2024 |
| `std::assert_matches!` | Rust 1.96 | None |

Each entry above was checked against Rust 1.98.1 in September 2026; the release notes linked from [releases.rs](https://releases.rs/) give the original announcements.

### Practise predicting the behaviour

Decide what each snippet does before reading the answers underneath. Every one of them uses only the rules in this post.

1. `let a = Size::Small; let b = a;` where `Size` derives `Debug` only. Can you print `a` afterwards?
2. `match &payment { Payment::Voucher(code) => code.len(), _ => 0 }`. What is the type of `code`?
3. `match size { Small => 1, Medium => 2, Large => 3 }` with no `use Size::*`. Does it compile?
4. `enum Size { Small = 1, Medium, Large = 10 }`. What is `Medium as u8`?
5. An enum has variants `A`, `B` and `C`; a `match` covers `A`, `B`, and `x if x_is_fine(&x)`. Does it compile?
6. `if let Some(Payment::Card { last4 }) = &maybe_payment && *last4 > 4000 { .. }` in a crate on edition 2021. Does it compile?
7. `Option<Size>` where `Size` has three unit variants. Is it bigger than `Size`?

Answers: (1) No: `Size` is not `Copy` unless you derive it, so `b = a` moved it; E0382. (2) `&String`: matching a reference switches the default binding mode to "ref". (3) No: `Small` is read as a binding, and its name is a variant of `Size`, so E0170 stops it; without that lint the second and third arms would be unreachable. (4) 2: an unset discriminant is one more than the previous one. (5) No: the guarded arm does not count, so `C` is uncovered; E0004. (6) No: let chains are edition 2024 only, "let chains are only allowed in Rust 2024 or later". (7) No, on this compiler: both are one byte, because `None` uses a spare tag value; that is a layout choice, not a promise, unlike `Option<Box<T>>` and `Option<&T>`.

### What comes next

You can now define a type that lists its possibilities, attach data to each, and take it apart safely with `match`. The [ownership post](/rust/concepts/2025/02/09/rust-ownership.html) picks up where the borrowing in section 5 left off: what it means to hold `&T` or `&mut T` over time, how lifetimes describe that, and the patterns that keep large programs honest.
