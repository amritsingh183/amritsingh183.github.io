## 5. Borrowing and References 🔗

### Overview 🔗

A **reference** is a way to access data without taking ownership. Instead of moving a value into a function, you can lend it temporarily through a reference. The compiler enforces strict rules about references at compile time, preventing data races and use-after-free bugs entirely.

References come in two varieties: **shared references** (read-only) and **mutable references** (exclusive write access), corresponding to Rust's core ownership principle: **multiple readers OR one writer, but never both simultaneously**.

***

### Shared References (`&T`) 🔗

A **shared reference** lets you read a value without taking ownership. You create one using the `&` operator. The `&` operator **always** creates an immutable (shared) reference, regardless of whether the binding itself is mutable.

```rust
fn main() {
    let s1 = String::from("hello");
    let len = calculate_length(&s1); // Borrow s1 via shared reference
    println!("Length of '{}' is {}", s1, len); // s1 is still valid
}

fn calculate_length(s: &String) -> usize {
    s.len()
} // s goes out of scope, but it doesn't own the data, so nothing is dropped
```

The key insight: you can create **multiple shared references** to the same data simultaneously:

```rust
fn main() {
    let s = String::from("hello");
    let r1 = &s;   // first shared reference
    let r2 = &s;   // second shared reference (allowed)
    println!("{} and {}", r1, r2); // ✅ Multiple readers are safe
}
```

**Why shared references are always read-only**: The `&` operator is designed to create read-only access. If you want mutation, you must use `&mut` (discussed next). This design prevents accidental mutations through borrowed references.

***

### Mutable References (`&mut T`) 🔗

A **mutable reference** gives exclusive write access to borrowed data. You create one with `&mut`. However, mutable references have a crucial restriction: **at most one mutable reference can exist to a value at any given time**.

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

This restriction prevents data races at compile time:

```rust
fn main() {
    let mut s = String::from("hello");
    let r1 = &mut s; // first mutable reference
    // let r2 = &mut s; // ❌ COMPILE ERROR: cannot borrow s as mutable more than once
    println!("{}", r1);
}
```

**Why this restriction exists**: If two mutable references existed simultaneously, both could modify the same data unpredictably, causing data races—a class of bugs that Rust prevents entirely at compile time.

***

### Binding Mutability vs Reference Mutability 🔗

**Binding mutability** (`let mut`) and **reference mutability** (`&` vs `&mut`) are orthogonal concerns. Binding mutability determines whether you can reassign a binding; reference mutability determines whether a reference can modify the data it points to.

```rust
let mut data = String::from("hello");
let immutable_ref = &data;        // Shared reference, read-only
let mutable_ref = &mut data;      // Mutable reference, can modify
```

Critically, you cannot create a mutable reference from an immutable binding:

```rust
let s = String::from("hello");
// let r = &mut s; // ❌ ERROR: binding s is not mutable
```

But a mutable binding can have shared references:

```rust
let mut s = String::from("hello");
let r = &s; // ✅ OK: binding is mutable, but this reference is shared
```

The `&` operator always creates a shared reference regardless of binding mutability. To get mutability, you must explicitly use `&mut` **and** have a mutable binding.

***

### The Borrowing Rules 🔗

Rust enforces two fundamental rules that the compiler checks automatically:

**Rule 1: Exclusivity**: At any given time, you can have **either** one mutable reference **or** any number of shared references, **but not both**.

```rust
fn main() {
    let mut s = String::from("hello");
    let r1 = &s;     // shared reference OK
    let r2 = &s;     // another shared reference OK
    // let r3 = &mut s; // ❌ ERROR: cannot borrow as mutable while shared references exist
    println!("{} and {}", r1, r2);
}
```

**Rule 2: Validity**: All references must always be valid. The compiler prevents **dangling references** (references to freed memory) entirely at compile time—this is discussed in detail in the next section.

These rules prevent entire categories of memory safety bugs that plague languages without borrow checking.

***

### Dangling Reference Prevention 🔗

A **dangling reference** would point to memory that has been freed or is no longer valid. Rust makes this **impossible** by checking that data outlives references to it.

```rust
// ❌ This code will NOT compile
fn dangle() -> &String {
    let s = String::from("hello");
    &s  // ERROR: returns a reference to local data that will be dropped
}
// When dangle() returns, s is dropped, but the reference would point to freed memory
```

The compiler rejects this with:

```
error[E0515]: cannot return reference to local variable `s`
```

**The solution**: Return the owned value instead, transferring ownership:

```rust
fn no_dangle() -> String {
    let s = String::from("hello");
    s  // Ownership moves to the caller
}

fn main() {
    let s = no_dangle(); // s receives ownership
    println!("{}", s); // ✅ Works: s is valid here
}
```

By moving ownership instead of returning a reference, the string remains valid in the caller's scope.

***

### Non-Lexical Lifetimes (NLL) 🔗

#### What NLL Solves

Before NLL, Rust used **lexical scoping** (determined by curly braces `{}`) to determine how long borrows lasted. This was overly conservative—a borrow would last until the end of the block, even if it was never used again.

```rust
// This code was rejected in pre-2018 Rust, despite being safe:
fn main() {
    let mut scores = vec![1, 2, 3];
    let score = &scores[0];
    println!("{}", score); // last use of score
    scores.push(4); // ERROR (before NLL): cannot modify while borrowed
}
```

**How NLL solves this**: Non-Lexical Lifetimes track borrows more precisely. **A borrow ends at its last use, not at the end of the lexical scope**.

```rust
// This now compiles (Rust 2018+):
fn main() {
    let mut scores = vec![1, 2, 3];
    let score = &scores[0];
    println!("{}", score); // ← borrow ends here (last use)
    scores.push(4); // ✅ OK: score is no longer borrowed
}
```


#### NLL in Practice

```rust
fn main() {
    let mut data = vec![1, 2, 3];
    let first = &data[0];           // shared borrow begins
    println!("First element: {}", first); // ← borrow ends here (last use)
    data.push(4);  // ✅ OK: first is no longer used, so mutation is allowed
    println!("{:?}", data);
}
```


***

### Two-Phase Borrows 🔗

**Two-phase borrows** apply specifically to method calls on `&mut self` when the arguments also contain shared references to the same data. This solves a specific pattern that would otherwise require verbose "take and return" code.

```rust
fn main() {
    let mut v = vec![1, 2, 3];
    v.push(v.len()); // ✅ Works due to two-phase borrows
    println!("{:?}", v); // prints [1, 2, 3, 3]
}
```

**Why this is safe**: During the evaluation of `v.push(v.len())`:

1. **Phase 1 (Shared borrow)**: Arguments are evaluated first. `v.len()` requires only a shared borrow of `v`.
2. **Phase 2 (Mutable borrow)**: After all arguments are evaluated, `push`'s `&mut self` becomes active.

The shared read (phase 1) completes **before** the mutable write (phase 2) begins, so there's no actual overlap. This special rule applies only to method calls with `&mut self` and shared borrows in arguments.

***

### Advanced Borrowing Patterns 🔗

#### Implicit Reborrowing in Function Calls

**Mutable references do NOT implement `Copy`**, meaning they normally move when assigned. However, the compiler **implicitly performs reborrowing** when passing mutable references to functions:

```rust
fn modify(v: &mut Vec<i32>) {
    v.push(42);
}

fn main() {
    let mut data = vec![1, 2];
    let r = &mut data;
    modify(r);  // Compiler inserts: modify(&mut *r) — creates temporary reborrow
    modify(r);  // ✅ r is still valid; reborrow from previous call already ended
    println!("{:?}", r); // ✅ Works: r still owns the mutable reference
}
```

**Key distinction**: This implicit reborrowing happens **only in function call contexts** (parameter passing and method calls). For other operations, mutable references move:

```rust
fn main() {
    let mut x = 5;
    let mut_ref1 = &mut x;
    let mut_ref2 = mut_ref1; // ASSIGNMENT (not function call) — moves ownership
    // println!("{}", *mut_ref1); // ❌ ERROR: mut_ref1 was moved
    println!("{}", *mut_ref2); // ✅ OK
}
```


#### Deref Coercion

References implement automatic deref coercion, allowing methods to be called without explicit dereferencing:

```rust
let s = String::from("hello");
let r = &s;
println!("{}", r.len()); // ✅ Compiler automatically dereferences: (*r).len()

// This works because &String coerces to &str via deref, and String has len()
```


#### Partial Moves 🔗

A partial move occurs when you move some fields out of a struct while leaving other fields in place:

```rust
#[derive(Debug)]
struct Point {
    x: i32,
    y: String,
}

fn main() {
    let p = Point {
        x: 10,
        y: String::from("hello"),
    };

    let x_val = p.x;  // Copy: x is i32, implements Copy
    let y_val = p.y;  // Move: y is String, does not implement Copy

    println!("{}", p.x); // ✅ OK: x was copied, not moved
    // println!("{:?}", p); // ❌ ERROR: p partially moved (y was moved)
    // println!("{}", p.y); // ❌ ERROR: y was moved out
}
```

After a partial move, you cannot use the entire struct, but you can still use fields that weren't moved (and implement `Copy`).

***

### Common Pitfalls 🔗

#### Pitfall 1: Fighting NLL by Limiting Scope Unnecessarily

```rust
// ❌ Unnecessary scope limitation
fn process_data(numbers: &[i32]) {
    let first = {
        let first_ref = &numbers[0];
        *first_ref
    }; // unnecessary braces
    
    // This pattern was needed before NLL to end borrows early
}

// ✅ Modern code (Rust 2018+): Let NLL handle scope
fn process_data(numbers: &[i32]) {
    let first_ref = &numbers[0];
    println!("{}", first_ref); // last use here
    // Borrow naturally ends, no need for explicit scoping
}
```


#### Pitfall 2: Assuming Mutable References Implement Copy

```rust
// ❌ This won't work:
fn main() {
    let mut x = 5;
    let r1 = &mut x;
    let r2 = r1; // MOVES, not copies
    // println!("{}", *r1); // ERROR: r1 was moved
}

// ✅ Function parameters handle this transparently:
fn modify(x: &mut i32) { *x += 1; }

fn main() {
    let mut x = 5;
    let r = &mut x;
    modify(r); // Implicit reborrow: modify(&mut *r)
    modify(r); // ✅ r still valid
}
```


***

## Key Takeaways 🔗

1. **Shared references (`&T`)** enable **read-only borrowing**; unlimited references allowed simultaneously
2. **Mutable references (`&mut T`)** enable **exclusive write access**; at most one can exist at a time
3. **Binding and reference mutability are independent**: whether a reference is mutable depends only on `&` vs `&mut`, not on whether the binding is `mut`
4. **NLL (Rust 2018+)** allows borrows to end at their last use, not block end, making code more ergonomic
5. **Dangling references are impossible**: the compiler guarantees all references are valid
6. **Reborrowing is implicit in function calls**: mutable references don't move when passed to functions; the compiler handles temporary reborrows automatically
7. **No reference can be created after data is dropped**: Rust enforces this at compile time

***

## Why This Section Matters 🔗

The borrowing rules represent Rust's solution to a fundamental problem that causes bugs in nearly all other languages: **how to share data safely without garbage collection**. By enforcing exclusivity (one writer XOR many readers) at compile time, Rust prevents:

- **Data races**: Multiple threads modifying the same data simultaneously
- **Use-after-free**: References to freed memory
- **Dangling pointers**: References to deallocated data
- **Iterator invalidation**: Modifying a collection while iterating

All of these are **impossible** in Rust, checked at compile time, with **zero runtime cost**.

***