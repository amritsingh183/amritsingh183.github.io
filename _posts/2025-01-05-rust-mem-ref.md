---
layout: post
title: "Comprehensive Rust Memory Layout Reference"
date: 2025-01-05 23:11:00 +0530
categories: rust concepts
last_updated: 2026-09-19
---
<a id="comprehensive-rust-memory-layout-reference-rust-1900-"></a>

# Comprehensive Rust Memory Layout Reference

This is the second stop after [variables, constants and lifetimes](/rust/concepts/2025/01/01/rust-var-const-lifetimes.html). That post explains who may use a value and when. Here we ask: **what does that value contain, what does it point to, and who cleans up the memory?**

The order matters. We start with values stored directly inside other values, then add owning pointers, then borrowed views. Only after that do we combine them into enums, trait objects, closures and a small game inventory. Later topics appear early where they prevent a wrong mental model: you do not need to master all their APIs yet.

**Checked with Rust 1.98.1, edition 2024, on 64-bit Apple Silicon (`aarch64-apple-darwin`), September 2026.** Rust 1.98.1 was [released on 3 September 2026](https://blog.rust-lang.org/2026/09/03/Rust-1.98.1/). A compiler version and an edition are different settings; edition 2024 does not freeze memory layouts. The original publication date above is retained.

<a id="index-"></a>

## Index

- [Overview: three separate questions](#overview-)
- [Table 1: values stored inline](#table-1-stack-only-types-no-indirection-)
- [Table 2: types that own separate storage](#table-2-heap-backed-types-owned-smart-pointers-)
- [Table 3: borrowed references, slices and raw pointers](#table-3-borrowed-references--slices-non-owning-)
- [Table 4: ownership, mutation and dispatch](#table-4-semantics--dispatch-matrix-)
- [Table 5: wrappers, enums and special types](#table-5-generic-wrappers-enums--special-types-)
- [Table 6: function pointers and closures](#table-6-function-pointers--closures-)
- [Memory breakdown: an array of string slices](#memory-breakdown-example-fixed-array-of-string-slices-)
- [Alignment and padding](#alignment--padding-rules-)
- [Copy, move and clone](#copy-trait-behavior-)
- [Thread safety: Send and Sync](#thread-safety--send--sync-traits-)
- [Putting it together: a game inventory](#example-to-demo-as-many-as-we-can-)
- [Practise explaining the memory](#practise)
- [Sources and the next lesson](#sources)

<a id="overview-"></a>

## Overview: three separate questions

Think of a library catalogue card. The card occupies space. The book occupies space somewhere else. Copying a card does not copy the book. Now add one more question: is this card responsible for returning the book, or does it merely help someone find it?

For a Rust value, ask these questions separately:

1. **Where is the value itself stored?** A local variable may use a stack slot, a register, or disappear during optimization. A field lives inside its enclosing value. A boxed value lives in an allocation. A static has storage lasting for the program.
2. **Does the value contain other values directly, or point elsewhere?** An array contains its elements directly. A `Vec` contains a small handle that manages a separate element buffer.
3. **Does it own what it points to?** A `String` owns its text buffer. An `&str` borrows text. An `Rc` shares ownership with other `Rc` handles.

**Inline means "inside the containing value." It does not mean "on the stack."** For example, `Box<[u32; 3]>` owns an allocation containing an array, and the three integers are inline in that allocation. A `Vec<String>` holds its `String` handles inside its element buffer; each nonempty string owns another buffer for its text.

Likewise, a known size does not decide the storage location. `String` has a known, fixed handle size even though its text can grow.

<a id="assumptions-"></a>

### How to read the sizes

The tables use bytes and ordinary standard-library types with their default allocators. A **word** here is 8 bytes, the size of `usize` on the checked target. Pointers to *sized* types have that size; pointers to slices and trait objects need additional information.

- **Guaranteed** means a language or library contract, under the stated conditions.
- **Observed** means a measurement with the compiler and target named above. Do not turn it into a file format, foreign-language interface or promise about another build.
- **Value size** means `size_of::<T>()`, including the value's padding. It does **not** walk through pointers and add the memory they reach. [The `size_of` documentation](https://doc.rust-lang.org/std/mem/fn.size_of.html) defines this precisely.

This complete program makes the distinction visible:

```rust
use std::mem::{align_of, size_of, size_of_val};

fn main() {
    let name = String::from("Amrit");
    let view: &str = &name;

    println!("String value: {} bytes", size_of::<String>()); // 24 here
    println!("String alignment: {}", align_of::<String>()); // 8 here
    println!("&str value: {} bytes", size_of_val(&view));    // 16 here
    println!("Borrowed text: {} bytes", size_of_val(view)); // 5
    println!("Text length: {}, capacity: {}", name.len(), name.capacity());
}
```

Notice the extra `&`: `size_of_val(&view)` measures the reference value; `size_of_val(view)` measures the `str` it refers to. Neither measures the whole process. Allocator bookkeeping, unused capacity and other allocations are separate questions. [`size_of_val`](https://doc.rust-lang.org/std/mem/fn.size_of_val.html) also works when a value's size is known only at runtime.

***

<a id="table-1-stack-only-types-no-indirection-"></a>

## Table 1: values stored inline

These types do not introduce an allocation merely by grouping their contents. Their fields or elements can still own allocations or borrow other data.

| Type | Value size on this target | What is inside? | Copy behavior |
| :-- | :-- | :-- | :-- |
| `i8`, `i16`, `i32`, `i64`, `i128` | 1, 2, 4, 8, 16; guaranteed | Signed integer | `Copy` |
| `u8`, `u16`, `u32`, `u64`, `u128` | 1, 2, 4, 8, 16; guaranteed | Unsigned integer | `Copy` |
| `isize`, `usize` | 8; target-dependent | Pointer-sized integer | `Copy` |
| `f32`, `f64` | 4, 8; guaranteed | Floating-point number | `Copy` |
| `bool` | 1; guaranteed | `false` or `true` | `Copy` |
| `char` | 4; guaranteed | One Unicode scalar value | `Copy` |
| `[T; N]` | `N * size_of::<T>()`; guaranteed | Exactly `N` inline elements | `Copy` when `T: Copy` |
| `[&str; N]` | `N * 16`; observed | `N` string-slice references; text elsewhere | `Copy`, even when the text is borrowed from a `String` |
| `(T, U, V)` | Field sizes plus layout-dependent padding | All three fields inline | `Copy` when all fields are `Copy` |
| A struct | Field sizes plus layout-dependent padding | Its fields inline | Eligible for `Copy` if all fields are `Copy`; requires an explicit implementation, often `#[derive(Clone, Copy)]` |

An array of three `String` values occupies **72 bytes here**, before counting text buffers: `3 * 24`. An array of three `&str` values occupies **48 bytes here**: `3 * 16`. The first owns its strings; the second borrows text. The array owns its reference *values*, but that does not make it the owner of the referenced text.

Practical choice: use `[u8; 4]` for exactly four colour channels. Use `Vec<u8>` when the number of channels or samples must grow. Both can contain elements inline; the difference is where those elements are stored and whether the collection can change length.

A Rust `char` is not necessarily one visible character on a screen: an accented letter can be composed of multiple Unicode scalar values. A `char` occupies 4 bytes; its UTF-8 encoding inside a string uses 1–4 bytes. [Rust's `char` documentation](https://doc.rust-lang.org/std/primitive.char.html) explains that distinction.

***

<a id="table-2-heap-backed-types-owned-smart-pointers-"></a>

## Table 2: types that own separate storage

The **handle** is the value you assign, pass to a function or store in a struct. Its location depends on where you put it. Moving the handle does not, by itself, clone its allocation.

| Type | Handle size here | Separately managed storage | What assignment and cloning do |
| :-- | :-- | :-- | :-- |
| `String` | 24; observed | UTF-8 text buffer, with length and capacity measured in bytes | Assignment moves; cloning makes an independent string |
| `Vec<T>` | 24; observed | Contiguous buffer containing `T` elements, with spare capacity | Assignment moves; cloning clones the elements when `T: Clone` |
| `Box<T>`, `T: Sized` | 8; single-pointer representation guaranteed | One `T`; `T` may itself own other storage | Assignment moves; cloning clones `T` into another box when `T: Clone` |
| `Box<[T]>` | 16; observed | A fixed-length owned slice; length travels with its pointer | Moves; has no capacity field or `push` method |
| `Rc<T>`, `T: Sized` | 8; observed | Shared allocation containing reference counts and `T` | Assignment moves; `Rc::clone` adds an owner without cloning `T` |
| `Arc<T>`, `T: Sized` | 8; observed | Shared allocation with atomic reference counts and `T` | Assignment moves; `Arc::clone` adds an owner without cloning `T` |
| `HashMap<u32, String>` | 48; observed for this exact type | Hash-table storage; strings may own further buffers | Assignment moves; cloning clones keys and values |
| `BTreeMap<u32, String>` | 24; observed for this exact type | Tree nodes containing keys and values; strings may own further buffers | Assignment moves; cloning clones keys and values |

The [`Box` single-pointer guarantee](https://doc.rust-lang.org/std/boxed/#memory-layout) requires a sized pointee. `Box<[u8; 100]>` points to an array whose length is part of the type; `Box<[u8]>` points to a slice whose length must accompany the pointer. That is why these two handles can have different sizes.

### Length, capacity and allocations

`Vec::len()` counts initialized elements; `capacity()` counts element slots available before growth is needed. For non-zero-sized `T`, the buffer has room for `capacity() * size_of::<T>()` bytes of elements. Text owned by those elements is additional.

`push` uses an available slot without reallocating the vector buffer; growing a full buffer requires reallocation for non-zero-sized elements. Reallocation may move the buffer. This is one reason Rust rejects keeping a reference to an element, pushing, and then using that old reference.

An empty `Vec::new()` does not allocate an element buffer. A `Vec<()>` needs no element allocation even when its length is large. Clearing a vector removes its elements but retains capacity. The familiar pointer/length/capacity picture describes its components, **not a guaranteed field order**. These are [documented `Vec` guarantees](https://doc.rust-lang.org/std/vec/struct.Vec.html#guarantees).

`String::new()` likewise starts without allocating text storage. Growing text may allocate; borrowing it does not copy it. [`String` documents its representation and capacity](https://doc.rust-lang.org/std/string/struct.String.html#representation). `Box::new(())` needs no allocation for its zero-sized payload. Do not infer "one allocation happened" merely from the presence of an owning type.

### Shared ownership is different from shared mutation

Imagine the game's quest log and screen both need the same quest. Two `Rc` handles can keep one quest alive. Cloning a handle changes the ownership count, not the quest. Ordinary assignment transfers a handle without adding an owner.

When the last strong owner goes away, the contained value is dropped. A `Weak` handle does not keep that value alive, but can keep the allocation reserved until the weak handle is gone. Strong-reference cycles can therefore leak; use `Weak` for appropriate back-links. The same ownership pattern applies to `Arc`. [The `Rc` module](https://doc.rust-lang.org/std/rc/) describes these rules.

The current implementation has strong and weak counters, but "16 bytes plus `T`" is not a portable allocation formula: alignment and implementation details matter. Do not count that overhead once per clone.

`Rc` and `Arc` do not generally grant mutation through all owners. `Rc<RefCell<T>>` supports checked shared mutation within one thread. `Arc<Mutex<T>>` is one option across threads when `T` meets the required bounds. `Arc` makes the ownership counts thread-safe; it does not make arbitrary `T` thread-safe. [See the `Arc` thread-safety contract](https://doc.rust-lang.org/std/sync/struct.Arc.html#thread-safety).

### Maps do not allocate one box per entry

Do not picture a map as "a control structure, then a separate allocation for every entry." [`HashMap`](https://doc.rust-lang.org/std/collections/struct.HashMap.html) stores entries in its table; [`BTreeMap`](https://doc.rust-lang.org/std/collections/struct.BTreeMap.html) stores several entries in each node. A stored `String` can add its own text allocation. Keys, values, unused space, alignment and implementation determine the total. Use `HashMap` for lookup by key; use `BTreeMap` when ordered keys and range traversal matter.

***

<a id="table-3-borrowed-references--slices-non-owning-"></a>

## Table 3: borrowed references, slices and raw pointers

Borrowing creates a view of existing data. It does not transfer ownership of that data or allocate a copy.

In the first two rows, `T` is **sized**: its size is known at compile time. `[T]`, `str` and `dyn Trait` are dynamically sized types, or **DSTs**; a pointer to one carries the information needed to use it.

| Type | Value size here | What it permits | What keeps the data valid? |
| :-- | :-- | :-- | :-- |
| `&T` | 8 for sized `T`; guaranteed | Shared access; no ordinary mutation through this reference | Borrowing rules and the referent's lifetime |
| `&mut T` | 8 for sized `T`; guaranteed | Exclusive access, including mutation | Borrowing rules and the referent's lifetime |
| `&[T]` | 16; observed | Shared view of consecutive elements, with an element count | The borrowed storage must stay valid |
| `&mut [T]` | 16; observed | Exclusive view of consecutive elements | The borrowed storage must stay valid |
| `&str` | 16; observed | Shared view of valid UTF-8, with a byte count | The borrowed text must stay valid |
| `*const T`, `*mut T` | 8 for sized `T`; guaranteed | Pointers without reference lifetime guarantees | Code using them must establish the required validity |

Slice references contain a data pointer and length. A trait-object pointer instead carries information about the hidden concrete type. The 16-byte wide-pointer measurements are useful, but Rust's [layout reference](https://doc.rust-lang.org/reference/type-layout.html#pointers-and-references-layout) warns against treating every DST pointer's current size as a permanent guarantee.

References must be non-null, aligned and point to a valid value, even when the value takes zero bytes. Shared references are `Copy`; mutable references are not. A lifetime annotation does not add a runtime timer or counter to either. [The reference type documentation](https://doc.rust-lang.org/std/primitive.reference.html) explains their validity and trait rules.

For the same borrowed data, ordinary access follows "many shared readers or one exclusive borrower." The restriction lasts while that borrow is needed, which may end before the closing brace. Reborrowing an `&mut T` temporarily hands its access to another reference. Interior-mutability types provide controlled mutation through shared references; they do not give arbitrary writes through any `&T`.

### A slice can borrow local data

You do not need a heap allocation to make an `&str`. This complete example borrows bytes from a local array:

```rust
fn main() {
    let bytes = [b'O', b'K'];
    let message = std::str::from_utf8(&bytes).expect("known ASCII bytes");
    println!("{message}"); // OK
}
```

The `expect` is justified by these fixed ASCII bytes. Untrusted input requires handling the possible UTF-8 error. A string literal has type `&'static str`; a slice borrowed from a local `String` does not become `'static` just because both are `&str`.

### A string length counts bytes

`"é".len()` is 2. Its UTF-8 bytes must stay together, so `&"é"[0..1]` panics, while `"é".get(0..1)` returns `None`. This is a boundary error, not an ownership error. Use `.chars()` to iterate Unicode scalar values; it still does not group every visible character. [The `str` API](https://doc.rust-lang.org/std/primitive.str.html) documents byte indexing and UTF-8 boundaries.

### Raw pointers are not references with the checks switched off

Creating a raw pointer from a reference, copying it or storing it is safe. Reading or writing its pointee requires `unsafe` and proof that the operation is valid. A raw pointer may be null or dangling; `*mut T` does not itself grant permission to mutate an immutable value. Turning a raw pointer into a reference must also meet that reference's rules. An `unsafe` block does not relax those rules. [See the raw-pointer API](https://doc.rust-lang.org/std/primitive.pointer.html).

***

<a id="table-4-semantics--dispatch-matrix-"></a>

## Table 4: ownership, mutation and dispatch

These are independent choices. A type's size cannot tell you all three.

| Question | Plain rule | Small example |
| :-- | :-- | :-- |
| Does assignment duplicate or transfer? | `Copy` permits implicit duplication; a non-`Copy` value moves | `u32` copies; `String` moves |
| Does the handle own the referent? | Inspect the type's contract | `Box<T>` owns; `&T` borrows; `Rc<T>` shares ownership |
| Does the operation allocate? | Inspect that operation and the current capacity | Borrowing a string does not; growing it may |
| Can this binding change its value? | `mut` permits reassignment or ordinary mutable borrowing | `let mut name = String::new();` |
| Can a shared reference change the contents? | Only through an appropriate interior-mutability API | `Cell::set`, `RefCell::borrow_mut`, `Mutex::lock` |
| Can it cross threads safely? | Check `Send` and `Sync`, separately from `Copy` | A raw pointer is `Copy` but neither `Send` nor `Sync` |
| Which implementation does a call use? | Concrete/generic calls can use static dispatch; trait-object calls use dynamic dispatch | `T: Display` versus `&dyn Display` |

**Dispatch** means deciding which function implementation to call. With a concrete type, the compiler knows the implementation. With `&dyn Display`, the handle carries a pointer to a **vtable**, a table used to find the concrete implementation at runtime. A generic parameter is a placeholder for a concrete type; it does not inherently mean dynamic dispatch.

Dynamic dispatch does not require the data to be on the heap: `&42u32 as &dyn std::fmt::Display` borrows an integer. Conversely, `Box<u32>` uses heap storage without requiring trait-object dispatch. Optimizers may simplify calls, so this distinction is not a promise about exact generated instructions. [Trait-object dispatch is described in the Reference](https://doc.rust-lang.org/reference/types/trait-object.html).

***

<a id="table-5-generic-wrappers-enums--special-types-"></a>

## Table 5: wrappers, enums and special types

An enum chooses one of several alternatives. A wrapper adds a rule around another type. Neither automatically implies a new heap allocation.

| Type | Value size here | Meaning and allocation behavior |
| :-- | :-- | :-- |
| `()` | 0; guaranteed, alignment 1 | One possible value, `()`. Returning normally with no useful result |
| `!` | No value can exist; do not treat it as an ordinary stored value | A computation that never returns, as in `fn stop() -> !` |
| `PhantomData<T>` | 0; guaranteed, alignment 1 | Compile-time relationship to `T`; contains no `T` and allocates nothing |
| `Option<bool>` | 1; observed | `Some(false)`, `Some(true)` or `None` |
| `Option<NonZeroU32>` | 4; guaranteed | Zero represents absence; a present number cannot be zero |
| `Option<&T>`, sized `T` | 8; guaranteed | Optional borrowed reference; null represents absence |
| `Option<String>` | 24; observed | Optional owned string; no extra allocation just for the `Option` |
| `Option<usize>` | 16; observed | Every `usize` value is valid, so absence needs an additional representation |
| `Result<T, E>` | Measure the concrete type | An `Ok(T)` or an `Err(E)` stored inline; payloads may own allocations |
| `Pin<P>` | Same layout as `P`; guaranteed | Restricts access through a pointer so address-sensitive pointees can be pinned |
| `&dyn Trait` | 16; observed | Borrowed wide pointer; concrete value may be local, static or heap-allocated |
| `Box<dyn Trait>` | 16; observed | Owning wide pointer; owns the concrete value |
| `Rc<dyn Trait>`, `Arc<dyn Trait>` | 16 each; observed | Shared owning wide pointers; counts and concrete value are in the shared allocation |
| `Cell<T>` | Same representation as `T`; guaranteed | Holds `T` inline and permits replacement through shared access |
| `RefCell<T>` | `RefCell<u32>` is 16; observed | Holds borrow-tracking state and `T` inline; checks borrows at runtime |

### A niche is a bit pattern the inner type cannot use

Suppose a ticket number must be nonzero. The number zero is then available to mean "no ticket." That is the idea behind `Option<NonZeroU32>`: it needs no separate tag. A **tag** or **discriminant** identifies the active enum variant.

Rust documents compact `Option` representations for certain types, including sized references, sized default-allocator boxes, function pointers and nonzero integers. `Option<String>` and `Option<bool>` are compact in this build, but their exact representation is not part of that general guarantee list. Do not guess which byte marks `None`, and do not manufacture it with raw bytes. [The `Option` representation contract](https://doc.rust-lang.org/std/option/#representation) lists the guarantees; [`NonZero` has its own layout contract](https://doc.rust-lang.org/std/num/struct.NonZero.html#layout).

For a general enum, **"largest payload plus one byte" is not a valid size formula**. Tag representation, alignment, padding and possible niches all matter. For example, `Result<u64, u8>` is 16 bytes in the checked build, not 9. The next lesson explores how to use these alternatives; here the lesson is that a type's valid states influence its layout.

### Zero-sized does not mean meaningless

`()` has one possible value; `!` has none. On Rust 1.98.1, using `!` as a function's return type is stable, while general uses such as `Option<!>` remain unstable. [`std::convert::Infallible`](https://doc.rust-lang.org/std/convert/enum.Infallible.html) is a stable empty error type for a result that cannot fail. [See the never-type documentation](https://doc.rust-lang.org/std/primitive.never.html).

A zero-sized type can still affect traits, alignment and cleanup. For example, a user-defined zero-sized type can implement `Drop`; zero bytes do not imply zero behavior. `PhantomData<T>` records a relationship used by compiler checks without storing a `T`. The marker's form influences those checks, so it is not interchangeable with every other zero-sized marker. [The `PhantomData` documentation](https://doc.rust-lang.org/std/marker/struct.PhantomData.html) explains its purpose and guaranteed size.

### Pin protects a pointee's address

`Pin` is useful when something relies on a value staying at its current address, as some future implementations do. Moving a `Pin<Box<T>>` handle can be fine: the box's pointee stays where it is. For an address-sensitive pinned value, the storage must stay at the same address and remain valid until its destructor has finished (or panicked). This is the [pinning drop guarantee](https://doc.rust-lang.org/std/pin/index.html#subtle-details-and-the-drop-guarantee).

When the pointee implements `Unpin`, the pinning restrictions can be lifted safely. `Pin` itself adds no allocation and does not make a type self-referential. `Pin<&mut T>` can pin a borrowed value; `Pin<Box<T>>` uses the allocation owned by the box. [The `Pin` documentation](https://doc.rust-lang.org/std/pin/struct.Pin.html) covers the distinction and its layout guarantee.

### Interior mutability changes the access rule, not the storage location

`Cell<u32>` is **4 bytes**, with no extra counter or allocation. Its `get()` copies out a `Copy` value; `set()` replaces the value. `Cell` does not hand out an unrestricted shared reference to its contents. [`Cell` guarantees the inner type's representation](https://doc.rust-lang.org/std/cell/struct.Cell.html#memory-layout).

`RefCell<u32>` is **16 bytes here**, including borrow bookkeeping and padding. `borrow()` produces a shared borrow guard; `borrow_mut()` produces an exclusive one. Conflicting borrows panic; `try_borrow()` and `try_borrow_mut()` report failure instead. **The runtime borrow ends when the guard is dropped; its last use does not itself drop the guard.** Use a smaller block or `drop(guard)` before requesting conflicting access. This differs from the borrow checker's ability to end an ordinary reference's borrow after its last use. Neither wrapper alone allocates separate storage for `T`. [See `RefCell`](https://doc.rust-lang.org/std/cell/struct.RefCell.html).

These distinctions matter in a single-threaded game: `Rc` can keep a quest alive for several screens, while `RefCell` controls their access to the same quest. Across threads, a `RefCell` is not a substitute for synchronization.

***

<a id="table-6-function-pointers--closures-"></a>

## Table 6: function pointers and closures

A function pointer identifies code. A closure combines behavior with a captured environment: the values or references it needs from outside its body. This section covers ordinary, non-async closures; async closures have additional rules for a later lesson.

| Form | Value size | What to remember |
| :-- | :-- | :-- |
| A named function's item value, `let f = add;` | 0; function-item types are zero-sized | The identity of `add` is part of this value's type |
| A function pointer, `let f: fn(i32) -> i32 = add;` | 8 here; observed | A pointer value that can refer to different compatible functions; `Copy` |
| A diverging function pointer, `fn() -> !` | 8 here; observed | Calling it does not return; this says nothing about capture storage |
| A closure with no captures | 0 here; observed, closure layout is unspecified | Can coerce to a compatible function pointer; implements `Fn`, `FnMut`, `FnOnce` and `Copy` |
| A closure borrowing a value to read it | Depends on captures and padding | Usually `Fn`; can be `Copy` when all captures meet the rules |
| A closure mutating captured state without moving out of it | Depends on captures and padding | `FnMut`, also `FnOnce`; a mutable-reference capture prevents `Copy` |
| A closure moving a non-`Copy` value out when called | Depends on captures and padding | `FnOnce` only: calling consumes that captured value |

[`Function items`](https://doc.rust-lang.org/reference/types/function-item.html) and [`function pointers`](https://doc.rust-lang.org/std/primitive.fn.html) are different types even when either can be called with `f(3)`.

The call traits form a hierarchy of permitted access: every closure implements `FnOnce`; a closure callable through mutable access also implements `FnMut`; one callable through shared access also implements `Fn`. **`move` says how values enter the closure. The body determines how calling uses them.** Closure layout is unspecified, so "sum of captured sizes" is only a starting intuition. [These are the Reference's closure rules](https://doc.rust-lang.org/reference/types/closure.html#call-traits-and-coercions).

This complete example uses three small jobs:

```rust
fn main() {
    // Own the label, but only read it on each call: Fn.
    let label = String::from("Score");
    let show = move || println!("{label}");
    show();
    show();

    // Mutably borrow the counter and return a copied number: FnMut.
    let mut clicks = 0;
    let mut click = || {
        clicks += 1;
        clicks
    };
    println!("click {}", click()); // 1
    println!("click {}", click()); // 2

    // Give away the captured String when called: FnOnce only.
    let receipt = String::from("Paid");
    let hand_over = move || receipt;
    let delivered = hand_over();
    println!("{delivered}");
    // hand_over(); // Would fail: the closure has been consumed.
}
```

Do not use `|| &mut clicks` as the ordinary `FnMut` example: it attempts to let a mutable reference to captured state escape the call and is rejected. Mutate inside the closure and return a value, as above.

`move` does not allocate by itself. It can move a `String` handle into the environment; that string's existing text allocation remains separate. Boxing the closure is another decision. Copying a closure is permitted only when its capture modes and captured types allow it; the `Fn`/`FnMut`/`FnOnce` label alone does not decide `Copy`.

***

<a id="memory-breakdown-example-fixed-array-of-string-slices-"></a>

## Memory breakdown: an array of string slices

For `let words = ["hello", "world", "rust"];`, the type is `[&str; 3]`:

```text
words: inline array, 48 bytes in this build
├── words[0]: &str = data pointer + length 5     (16 bytes)
├── words[1]: &str = data pointer + length 5     (16 bytes)
└── words[2]: &str = data pointer + length 4     (16 bytes)
                │
                └── borrowed UTF-8 bytes with static lifetime
                    "hello"   "world"   "rust"
                       5    +    5    +    4 = 14 bytes of text
```

This is a **logical breakdown**, not an exact stack-frame or executable-size measurement. The compiler can eliminate the local array or share literal storage. Rust string slices do not require a trailing null byte. Copying `words` copies the references, not these text bytes. Dropping it does not free the literals.

Now compare owned strings:

```text
[String; 3]: 72 bytes in this build
├── String handle ──> its owned text buffer
├── String handle ──> its owned text buffer
└── String handle ──> its owned text buffer
```

Move that array into a `Box`, and those 72 bytes are in the box's allocation. Its strings still own their respective buffers. This is the same ownership graph in a different location.

***

<a id="alignment--padding-rules-"></a>

## Alignment and padding

Alignment is an address requirement. If a type needs alignment 4, its stored values must start at addresses divisible by 4. **Padding** is space left between fields or after the final field so these requirements can be met. Size includes that padding; field sizes alone need not add up to it. Alignment depends on the target and type, not simply the number of bytes in the type.

Imagine labelled drawers that must start at every fourth position in a cabinet. Leaving a gap before one drawer lets it start at the required position. Tail padding makes the *next whole cabinet*, when cabinets form an array, start correctly too.

<a id="default-representation-reprrust-"></a>

### Default representation: `repr(Rust)`

Rust may reorder struct fields. It does not promise the smallest possible layout or a particular field order. Therefore, measure an ordinary struct when size matters; do not infer its memory order from source order. Tuples likewise have no general field-order guarantee.

<a id="fixed-representation-reprc-"></a>

### C representation: `repr(C)`

For a `repr(C)` struct, declaration order controls field placement, with padding for the target's alignment rules. This complete example prints offsets as well as sizes:

```rust
use std::mem::{align_of, offset_of, size_of};

#[repr(C)]
struct Reading {
    status: u8,
    count: u32,
    code: u8,
}

fn main() {
    println!("size={}, alignment={}", size_of::<Reading>(), align_of::<Reading>());
    println!("status={}, count={}, code={}",
        offset_of!(Reading, status),
        offset_of!(Reading, count),
        offset_of!(Reading, code));
}
```

Here the offsets are 0, 4 and 8, with size 12 and alignment 4: three padding bytes before `count`, and three after `code`. Moving `code` before `count` would make this `repr(C)` struct 8 bytes on this target.

`repr(C)` does not recursively give a Rust-layout field a C layout. It also does not make `String` a C string, define a portable network format or settle endianness. Use explicit serialization for saved files and network messages. [The Reference defines representation guarantees](https://doc.rust-lang.org/reference/type-layout.html#representations).

<a id="packed-representation-reprpacked-"></a>

### Packed representation: `repr(packed)`

Packing lowers alignment requirements and can remove padding between fields. Bare `repr(packed)` still uses Rust field-order rules; `repr(C, packed)` also fixes declaration order. Padding *inside* a nested field is not removed.

The main gotcha is correctness: **creating an unaligned reference is undefined behavior**, even if your processor can read unaligned bytes. Formatting can borrow a field implicitly. Copy a `Copy` field into an aligned local first:

```rust
#[repr(C, packed)]
struct Packet {
    tag: u8,
    count: u32,
}

fn main() {
    let packet = Packet { tag: 1, count: 42 };
    let count = packet.count; // By-value read; does not create &packet.count.
    println!("{count}");
    // println!("{}", packet.count); // Would fail: creates an unaligned reference.
}
```

This `Packet` is 5 bytes with alignment 1. `unsafe` around `&packet.count` would not make that reference valid. When raw access is required, APIs such as `read_unaligned` have their own safety obligations; packing is not a routine "make it faster" switch. [Compiler error E0793 explains the hazard](https://doc.rust-lang.org/error_codes/E0793.html).

***

<a id="copy-trait-behavior-"></a>

## Copy, move and clone

**Copy** means assignment may duplicate the value implicitly and leave the source usable. It does not mean the value lives on the stack, is small, or is thread-safe. `[u8; 1_000_000]` is `Copy`; copying that much data can be expensive if the compiler cannot eliminate the work.

**Move** transfers use of a value to another owner. A move may involve copying bytes or may be optimized away. A large inline array does not become a pointer merely because it moves. Moving a `String` transfers its handle and responsibility for its text; it does not clone the text.

**Clone** is an explicit operation defined by the type. `String::clone` creates independent text; `Rc::clone` shares the same value. A container clone follows its elements' clone behavior: cloning `Vec<Rc<T>>` adds shared owners rather than recursively cloning every `T`.

Types implementing `Drop` cannot implement `Copy`. Arrays and tuples receive `Copy` when their elements permit it; structs need an explicit implementation. Shared `&T` references are `Copy` even if `T` is not; `&mut T` is not `Copy` because duplicating exclusive access would break its contract. [The `Copy` trait documentation](https://doc.rust-lang.org/std/marker/trait.Copy.html) gives the rules.

Dropping an ordinary owning container drops the values it contains. For `Rc` and `Arc`, dropping one handle only releases that owner's claim; the last strong owner triggers value cleanup. Borrowed references do not destroy their referents. Automatic cleanup is not a universal leak-prevention promise: strong-reference cycles and intentionally forgotten values can retain resources.

***

<a id="thread-safety--send--sync-traits-"></a>

## Thread safety: Send and Sync

**`Send` asks:** can ownership of a value move safely to another thread?

**`Sync` asks:** can different threads safely share references to this value? More precisely, `T: Sync` means `&T: Send`.

These are safety permissions, not a request to start threads. They also do not remove lifetime requirements. [See `Send`](https://doc.rust-lang.org/std/marker/trait.Send.html) and [the exact `Sync` definition](https://doc.rust-lang.org/std/marker/trait.Sync.html).

The table uses the standard types with default allocators:

| Type | `Send` | `Sync` |
| :-- | :-- | :-- |
| Numeric primitives, `bool`, `char` | Yes | Yes |
| `String` | Yes | Yes |
| `Vec<T>`, `Box<T>` | When `T: Send` | When `T: Sync` |
| `Rc<T>` | No | No |
| `Arc<T>` | When `T: Send + Sync` | When `T: Send + Sync` |
| `&T` | When `T: Sync` | When `T: Sync` |
| `&mut T` | When `T: Send` | When `T: Sync` |
| `Cell<T>`, `RefCell<T>` | When `T: Send` | No |
| `Mutex<T>` | When `T: Send` | When `T: Send` |
| `*const T`, `*mut T` | No | No |

The surprising row is `&mut T`. It can be `Sync`: sharing a reference *to the mutable reference* gives `&&mut T`, which cannot provide unrestricted mutation of `T`. Exclusive access remains exclusive. This does not let two threads both use the same `&mut T` to write. [The `Sync` documentation explains this case](https://doc.rust-lang.org/std/marker/trait.Sync.html).

`Mutex<T>` can be shared when `T: Send` because the lock grants access to one holder at a time. [`Mutex` documents these bounds](https://doc.rust-lang.org/std/sync/struct.Mutex.html). `Arc<RefCell<T>>` does not become thread-safe: atomic ownership counts do not synchronize `RefCell`'s borrowing state.

Trait objects also need the relevant bounds: `Arc<dyn Trait>` alone does not promise them. `Arc<dyn Trait + Send + Sync>` states the additional requirements. You can move a `Vec<String>` to a worker thread without an `Arc`; shared ownership is useful when several owners need to retain the same allocation.

***

<a id="example-to-demo-as-many-as-we-can-"></a>

## Putting it together: a game inventory

The game keeps player statistics, a growable inventory, one quest visible in two places, and a large pixel buffer. This is intentionally single-threaded. Each type has a job:

- `PlayerStats` is small and `Copy`, so a previous state can be kept for comparison.
- `Vec<Item>` owns the inventory items; each item owns its name.
- `Rc<RefCell<Quest>>` lets the game and quest screen retain and update one quest with runtime-checked borrowing.
- `Box<[u8]>` owns pixels whose length will not change after loading.
- `HashMap<u32, String>` demonstrates lookup data with its own ownership.

This complete program prints measured value sizes instead of guessed stack totals:

```rust
use std::cell::RefCell;
use std::collections::HashMap;
use std::mem::{align_of, size_of, size_of_val};
use std::rc::Rc;

#[derive(Debug, Clone, Copy)]
struct PlayerStats {
    health: u32,
    level: u8,
    experience: u64,
    gold: u32,
}

struct Item {
    id: u32,
    name: String,
    damage: u16,
}

struct Quest {
    title: String,
    completed: bool,
}

struct GameState {
    player: PlayerStats,
    inventory: Vec<Item>,
    active_quest: Option<Rc<RefCell<Quest>>>,
    pixels: Box<[u8]>,
}

fn show_layout<T>(label: &str) {
    println!("{label}: size={}, alignment={}", size_of::<T>(), align_of::<T>());
}

// A borrowed slice lets this function read any consecutive group of items.
fn print_inventory(items: &[Item]) {
    for item in items {
        println!("  #{} {} (damage {})", item.id, item.name, item.damage);
    }
}

fn main() {
    let quest = Rc::new(RefCell::new(Quest {
        title: String::from("Find the lost key"),
        completed: false,
    }));
    let quest_screen = Rc::clone(&quest); // Another owner, same quest.

    let mut inventory = Vec::with_capacity(4);
    inventory.push(Item { id: 1, name: String::from("Sword"), damage: 15 });
    inventory.push(Item { id: 2, name: String::from("Shield"), damage: 0 });

    let mut game = GameState {
        player: PlayerStats { health: 100, level: 1, experience: 0, gold: 50 },
        inventory,                 // Move the inventory; do not clone its items.
        active_quest: Some(quest),  // Move this handle; two strong owners remain.
        pixels: vec![0u8; 1_000_000].into_boxed_slice(),
    };

    show_layout::<PlayerStats>("PlayerStats");
    show_layout::<Item>("Item");
    show_layout::<GameState>("GameState (excludes separate allocations)");
    show_layout::<Option<Rc<RefCell<Quest>>>>("Optional quest handle");
    println!("Pixel handle: {}; pixel payload: {}",
        size_of_val(&game.pixels), size_of_val(&*game.pixels));

    let coordinates = [10i32, 20, 30];
    let combat_roll = (6u8, 12u8, true);
    println!("Coordinates: {} bytes; combat tuple: {} bytes",
        size_of_val(&coordinates), size_of_val(&combat_roll));

    println!("Inventory before pickup:");
    print_inventory(&game.inventory); // Borrow; no inventory clone.
    let capacity_before = game.inventory.capacity();
    game.inventory.push(Item { id: 3, name: String::from("Potion"), damage: 0 });
    println!("Inventory after pickup:");
    print_inventory(&game.inventory);
    println!("Item count: {}; vector capacity: {} -> {}",
        game.inventory.len(), capacity_before, game.inventory.capacity());
    // Capacity was at least four, so this third push did not grow the buffer.
    // Constructing the Potion's String is a separate allocation question.

    let before_combat = game.player; // Explicitly useful Copy snapshot.
    game.player.health -= 30;
    game.player.experience += 100;
    game.player.gold += 25;
    println!("Before combat: {before_combat:?}");
    println!("Current player: {:?}; level {}", game.player, game.player.level);

    println!("Quest screen before update: {}", quest_screen.borrow().completed);
    if let Some(active) = &game.active_quest {
        {
            let mut quest = active.borrow_mut();
            quest.completed = true;
        } // End the mutable borrow before the screen reads.
        println!("Quest strong owners: {}", Rc::strong_count(active)); // 2
    }
    {
        let visible = quest_screen.borrow();
        println!("Quest screen after update: {} = {}", visible.title, visible.completed);
    }

    let player_names = HashMap::from([
        (1u32, String::from("Alice")),
        (2u32, String::from("Bob")),
    ]);
    println!("Player-name map handle: {} bytes", size_of_val(&player_names));
    if let Some(name) = player_names.get(&1) {
        println!("Player 1: {name}");
    }
}
```

Follow the program's changes: the inventory grows from two items to three; the current player's health goes from 100 to 70 while the copied snapshot stays at 100; and the quest screen sees `false` become `true` after the game updates the shared quest. These are actual changes to the state being displayed.

The pixel payload is 1,000,000 bytes; its `Box<[u8]>` handle is 16 bytes here. Building a vector buffer avoids writing a million-element local array that is then passed to `Box::new`. **`Box::new([0; 1_000_000])` is not a language guarantee against a large temporary stack allocation.** The vector approach is appropriate here; this program does not measure allocator call counts or peak stack usage.

To understand the inventory's memory, start with the `Vec` handle inside `GameState`, then the capacity-sized buffer of `Item` values, then the name buffers each item owns. For the quest, count one shared allocation, not two copies of the quest. `size_of::<GameState>()` alone cannot tell you the total retained memory.

***

<a id="practise"></a>

## Practise explaining the memory

Before memorizing sizes, answer these in your own words:

1. If `[String; 3]` moves into a `Box`, which values change location, and which text buffers remain separately owned?
2. If an `Rc<Quest>` is assigned to another name, does the strong count increase? What changes if you use `Rc::clone`?
3. Why can an `&str` refer to a local byte array, and why must its lifetime still be limited?
4. Why can `Option<NonZeroU32>` use four bytes, while `Option<usize>` needs more than one `usize` on this target?
5. Why does a `move` closure that only prints its captured `String` remain callable twice?
6. Why is `Arc<RefCell<T>>` insufficient for shared mutation across threads?

Answers: the boxed allocation holds the array and its string handles; moving those handles does not clone their text. Assignment transfers an `Rc` handle, while cloning adds an owner. A slice needs valid text and a valid borrow, not heap storage. Zero is unavailable to a `NonZeroU32` value and can represent absence; every `usize` bit pattern already represents a number. The closure keeps ownership and only borrows its captured text when printing. Finally, `Arc` protects its counts, while `RefCell` provides no thread-safe synchronization for its contents.

<a id="sources"></a>

## Sources and the next lesson

The source links beside each section are the authorities for the rules. For questions of layout, start with the [Rust Reference](https://doc.rust-lang.org/reference/type-layout.html), then the type's standard-library documentation. For an implementation-dependent size, measure with `size_of`, `size_of_val`, `align_of` and, where appropriate, `offset_of!` on your actual compiler and target. Never inspect padding or transmute a private layout merely to confirm a table.

Next comes [Option and Result](/rust/concepts/2025/01/09/rust-option-result.html): using alternatives to represent absence and failure. Later, revisit [ownership](/rust/concepts/2025/02/09/rust-ownership.html) and [Copy and Clone](/rust/concepts/2025/05/21/rust-copy-clone.html) for their deeper API rules. Here, the skill to keep practising is drawing a value's ownership and reference relationships before guessing its memory cost.
