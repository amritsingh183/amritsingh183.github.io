---
layout: post
title: "Mastering Ownership, Moves, Borrowing, and Lifetimes in Rust"
date: 2025-10-7 10:23:00 +0530
categories: rust concepts
---

# Mastering Ownership, Moves, Borrowing, and Lifetimes in Rust

This curriculum builds from first principles of ownership to advanced patterns like two‑phase borrows, reborrowing, and partial moves, then formalizes lifetime inference and annotations before covering 'static, const/static, and Rust 2024’s static_mut_refs behavior.

### Prerequisites and goals

- Assumes familiarity with variables, functions, and basic control flow, without presuming prior systems programming experience.
- Goals: unambiguous mental model, correct rule ordering, and mastery of edge cases so learners can design APIs and debug borrow checker errors with confidence.


### Mental model: aliasing XOR mutability

- Rust enforces “many readers or one writer” by separating shared borrows that permit aliasing from unique mutable borrows that require exclusivity for the borrow’s duration.
- This rule is realized by the borrow checker together with non‑lexical lifetimes that shorten borrows to their last use, increasing flexibility without sacrificing safety.


### Ownership basics

- Each value has a single owner, ownership moves on assignment for non‑Copy types, and values are dropped when the owner goes out of scope.
- Copy types (integers, bool, char, and composites of Copy fields) are duplicated on assignment, while non‑Copy types like String and Vec move by default.

```rust
let s1 = String::from("hi");
let s2 = s1;        // move: s1 invalid afterwards
let x = 5;
let y = x;          // copy: x and y remain valid
```

- Passing by value moves into a function and returning by value moves out, so ownership flows across function boundaries explicitly.


### Moves vs Copy in practice

- A move transfers ownership without cloning data, so using the source after a move is a compile error preventing use‑after‑free.
- Implementing Copy requires plain‑old‑data semantics; types with Drop or non‑Copy fields cannot be Copy to preserve correct destruction behavior.


### Borrowing fundamentals

- Shared borrows `&T` permit read‑only aliasing, and unique borrows `&mut T` permit mutation with exclusive access for the borrow’s lifetime.
- Non‑lexical lifetimes end a borrow at its last use, allowing later borrows in the same scope once earlier uses are completed.

```rust
let mut s = String::from("hi");
let r1 = &s;
let r2 = &s;
println!("{} {}", r1, r2); // last use of r1/r2
let r3 = &mut s;           // OK after last use of r1/r2
r3.push('!');
```


### Two‑phase borrows

- Two‑phase borrows create a “reservation” for a future `&mut` that becomes exclusive only when mutation occurs, enabling patterns like `v.push(v.len())`.
- This works because read‑only evaluation of arguments happens before the exclusive `&mut` activation, avoiding simultaneous aliasing with a unique borrow.

```rust
let mut v = vec![1, 2];
v.push(v.len()); // reserved then activated; compiles
```


### Reborrowing

- Reborrowing produces a shorter‑lived borrow from an existing borrow (e.g., `let b: &mut _ = &mut *a;` when `a: &mut T`), temporarily suspending the parent borrow.
- The parent borrow cannot be used while the child reborrow is active, but becomes usable again after the reborrow ends, modeling unique access transfer.

```rust
let mut n = 0u32;
let a = &mut n;          // unique borrow
let b: &mut _ = a;       // reborrow; a inactive
*b += 1;                 // use b
// b ends here
*a += 1;                 // a reactivated
```


### Partial moves

- Rust allows moving some fields out of a non‑Copy struct while leaving others, making the original binding “partially moved” and unusable as a whole.
- Pattern matching can move some fields and borrow others simultaneously; after such a pattern, only the still‑owned or borrowed parts remain accessible via their bindings.

```rust
#[derive(Debug)]
struct User { id: u64, name: String, note: Option<String> }

let u = User { id: 1, name: "Ada".into(), note: None };
// Move `name`, borrow `id`
let User { id: ref uid, name, .. } = u;
println!("{uid}");  // borrow ok
println!("{name}"); // moved field
// `u` as a whole is now unusable
```


### Lifetime essentials

- Lifetimes describe the validity of references so that borrows never outlive their sources, and most lifetimes are inferred by the compiler.
- When inference is insufficient, explicit lifetime parameters tie the output reference’s lifetime to one or more input references to prevent dangling pointers.

```rust
fn first<'a>(s: &'a str) -> &'a str { &s[..1] }
```


### Lifetime elision rules

- Elision rules: each elided input reference gets its own lifetime; if exactly one input lifetime exists, outputs get that lifetime; in methods, the receiver’s lifetime flows to elided outputs.
- These rules remove most annotations from function signatures while preserving precise relationships between input and output references.

```rust
// Elided
fn head(s: &str) -> &str { &s[..1] }
// Desugared
fn head<'a>(s: &'a str) -> &'a str { &s[..1] }
```


### 'static, const, and static

- `'static` denotes data available for the entire program duration, such as string literals, but requiring `'static` should be reserved for APIs that truly need it.
- `const` values are inlined and have no stable address, whereas `static` values have a fixed address and `'static` lifetime, making statics suitable for global state and FFI.


### Rust 2024: static_mut_refs

- In the 2024 edition, taking any reference to a `static mut` is denied by default because such a reference is instant undefined behavior even if unused.
- Prefer atomics, Mutex/OnceLock/LazyLock, or thread_local for global mutable state, or use raw pointers when unavoidable to avoid creating references to `static mut`.

```rust
use std::sync::atomic::{AtomicU64, Ordering};
static COUNTER: AtomicU64 = AtomicU64::new(0);
COUNTER.fetch_add(1, Ordering::SeqCst);
```


### Comparison at a glance

| Topic | Definition | Key property | Typical use |
| :-- | :-- | :-- | :-- |
| Move | Transfer of ownership for non‑Copy values  | Invalidates the source binding  | Resource‑owning types like String/Vec  |
| Copy | Bitwise duplication for POD‑like values  | Both bindings remain valid  | Primitives and Copy composites  |
| \&T | Shared aliasable borrow  | Read‑only access by many readers  | Passing read‑only views  |
| \&mut T | Unique mutable borrow  | Exclusive, write access  | In‑place mutation without moving  |
| NLL | Borrow ends at last use  | Frees later code to borrow mutably  | Flexible in‑block patterns  |
| Two‑phase | Deferred `&mut` exclusivity  | Enables `v.push(v.len())`  | Builder‑style APIs and methods  |
| Reborrow | Borrow of a borrow  | Parent paused during child  | Nested unique access patterns  |
| Partial move | Move some fields from a struct  | Whole is unusable afterward  | Avoid clones on selected fields  |
| 'static | Longest possible lifetime  | Lives for program duration  | String literals and global data  |
| static_mut_refs | Deny refs to `static mut`  | Prevent instant UB  | Use atomics/Mutex/thread_local  |

### Patterns and anti‑patterns

- Prefer borrowing over moving in function parameters when ownership transfer is not semantically required to avoid unnecessary clones and invalidations.
- Return owned data when the callee constructs or takes ownership, but return references tied to inputs when the data must remain owned by the caller.
- Avoid `'static` bounds unless the data truly must outlive all tasks or threads, as overly broad constraints reduce flexibility.


### Practice prompts

- Rewrite functions to accept `&T` or `&mut T` instead of moving `T`, explaining how this changes caller obligations and borrow lifetimes.
- Transform code that fails due to overlapping borrows by applying NLL understanding or reborrowing to narrow the active lifetime of references.
- Demonstrate a `Vec` method call that leverages two‑phase borrows and explain reservation vs activation in evaluation order.


### Appendix: disjoint fields and borrowing

- The borrow checker is place‑ and field‑sensitive, but borrows taken through the same binding can conservatively prevent overlapping access even to disjoint fields during an active unique borrow.
- Destructure into independent bindings or scope borrows more tightly to make disjointness visible to the compiler and reduce conservative overlap errors.


### Suggested syllabus order for a course

- Ownership and moves → Copy vs non‑Copy → Shared vs mutable borrows and NLL → Two‑phase borrows → Reborrowing → Partial moves → Lifetime elision and explicit annotations → 'static, const/static, and 2024 static_mut_refs with safer patterns.

This structure corrects the earlier interleaving of variables/constants with ownership and adds the missing rules and edge cases so learners progress from mental models to expert‑level patterns confidently.
<span style="display:none"></span>

<div align="center">⁂</div>

: https://doc.rust-lang.org/book/ch04-00-understanding-ownership.html

: https://doc.rust-lang.org/book/ch04-02-references-and-borrowing.html

: https://doc.rust-lang.org/reference/lifetime-elision.html

: https://doc.rust-lang.org/rust-by-example/scope/move/partial_move.html

: https://www.ralfj.de/blog/2023/06/02/tree-borrows.html

: https://github.com/rust-lang/rust/issues/49434

: https://doc.rust-lang.org/edition-guide/rust-2024/static-mut-references.html

: https://doc.rust-lang.org/book/ch04-01-what-is-ownership.html

: https://haibane-tenshi.github.io/rust-reborrowing/

: https://web.mit.edu/rust-lang_v1.26.0/arch/amd64_ubuntu1404/share/doc/rust/html/reference/lifetime-elision.html

: 2025-10-8-var-const-lifetimes.md

: https://www.reddit.com/r/rust/comments/yibdpi/ownership_as_explained_in_the_rust_book/

: https://rust-book.cs.brown.edu/ch04-00-understanding-ownership.html

: https://jasonwalton.ca/rust-book-abridged/ch04-ownership/

: https://www.w3schools.com/rust/rust_borrowing_references.php

: https://masteringbackend.com/posts/understanding-lifetime-elision-in-rust

: https://joeprevite.com/rust-lang-book-chapter-4-notes/

: https://www.programiz.com/rust/references-and-borrowing

: https://earthly.dev/blog/rust-lifetimes-ownership-burrowing/

: https://rust-book.cs.brown.edu/ch04-01-what-is-ownership.html

: https://www.youtube.com/watch?v=qIhoi-74IXc

: https://www.reddit.com/r/rust/comments/13qcj0x/questions_about_ownership_rule/

: https://www.youtube.com/watch?v=K682q8p-YHg

: https://web.mit.edu/rust-lang_v1.25/arch/amd64_ubuntu1404/share/doc/rust/html/book/first-edition/references-and-borrowing.html

: https://doc.rust-lang.org/nomicon/lifetime-elision.html

: https://github.com/rust-lang/book/issues/3961

