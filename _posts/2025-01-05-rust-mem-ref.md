---
layout: post
title: "Comprehensive Rust Memory Layout Reference"
date: 2025-01-05 23:11:00 +0530
categories: rust concepts
last_updated: 2025-10-14
---
# Comprehensive Rust Memory Layout Reference (Rust 1.90.0) <a href="#comprehensive-rust-memory-layout-reference-rust-1900-" class="header-link">🔗</a>

## Index <a href="#index-" class="header-link">🔗</a>

- [Comprehensive Rust Memory Layout Reference (Rust 1.90.0)](#comprehensive-rust-memory-layout-reference-rust-1900-)
  - [Overview](#overview-)
    - [Assumptions](#assumptions-)
  - [Table 1: Stack-Only Types (No Indirection)](#table-1-stack-only-types-no-indirection-)
  - [Table 2: Heap-Backed Types (Owned Smart Pointers)](#table-2-heap-backed-types-owned-smart-pointers-)
  - [Table 3: Borrowed References & Slices (Non-Owning)](#table-3-borrowed-references--slices-non-owning-)
  - [Table 4: Semantics & Dispatch Matrix](#table-4-semantics--dispatch-matrix-)
  - [Table 5: Generic Wrappers, Enums & Special Types](#table-5-generic-wrappers-enums--special-types-)
  - [Table 6: Function Pointers & Closures](#table-6-function-pointers--closures-)
  - [Memory Breakdown Example: Fixed Array of String Slices](#memory-breakdown-example-fixed-array-of-string-slices-)
  - [Alignment & Padding Rules](#alignment--padding-rules-)
    - [Default Representation (`repr(Rust)`)](#default-representation-reprrust-)
    - [Fixed Representation (`repr(C)`)](#fixed-representation-reprc-)
    - [Packed Representation (`repr(packed)`)](#packed-representation-reprpacked-)
  - [Copy Trait Behavior](#copy-trait-behavior-)
  - [Thread Safety & Send + Sync Traits](#thread-safety--send--sync-traits-)
  - [Example to demo as many as we can](#example-to-demo-as-many-as-we-can-)

This guide is assumes that you have gone through [basics of Rust](https://amritsingh183.github.io/rust/concepts/2025/01/01/rust-var-const-lifetimes.html)

## Overview <a href="#overview-" class="header-link">🔗</a>

This document provides complete coverage of Rust's memory layout, addressing all primitive, heap-backed, borrowed, and special types as of Rust 1.90.0.

### Assumptions <a href="#assumptions-" class="header-link">🔗</a>

- **Architecture**: 64-bit target (typical: x86_64, ARM64)
- **Pointer/reference size**: 8 bytes
- **Alignment**: Natural alignment for all types

***

## Table 1: Stack-Only Types (No Indirection) <a href="#table-1-stack-only-types-no-indirection-" class="header-link">🔗</a>

| Type<svg width="100" height="1"></svg>  | Stack Size | Container Location | Value(s) Location<svg width="100" height="1"></svg>  | Ownership Semantics | Copy/Move<svg width="100" height="1"></svg> | Example<svg width="350" height="1"></svg> |
| :-- | :-- | :-- | :-- | :-- | :-- | :-- |
| `i8`, `i16`, `i32`, `i64`, `i128` | 1–16 bytes | Stack | Stack (inline) | Owns; exclusive | Copy duplicated | `let x = -42i32;` → 4 bytes |
| `u8`, `u16`, `u32`, `u64`, `u128` | 1–16 bytes | Stack | Stack (inline) | Owns; exclusive | Copy duplicated | `let x = 42u64;` → 8 bytes |
| `isize`, `usize` | 8 bytes (64-bit) | Stack | Stack (inline) | Owns; exclusive | Copy duplicated | Platform-dependent; typically 8 bytes |
| `f32` | 4 bytes | Stack | Stack (inline) | Owns; exclusive | Copy duplicated | `let pi = 3.14f32;` → 4 bytes |
| `f64` | 8 bytes | Stack | Stack (inline) | Owns; exclusive | Copy duplicated | `let pi = 3.14;` → 8 bytes (default) |
| `bool` | 1 byte | Stack | Stack (inline) | Owns; exclusive | Copy duplicated | `let b = true;` → 1 byte |
| `char` | 4 bytes | Stack | Stack (inline) | Owns; exclusive | Copy duplicated | `let c = 'A';` → 4 bytes (UTF-32) |
| `[T; N]` fixed array | N×sizeof(T) | Stack | Stack (inline); Heap (if T owns heap data) | Owns all N elements | Copies if `T: Copy`; moves if `T: !Copy` | `let arr = [1, 2, 3];` → 12 bytes; each element may own heap data |
| `[&str; N]` | N × 16 bytes (fat ptrs) | Stack | Stack (fat pointers); Data in Binary/Heap (referenced) | Owns references; borrows string data | Copy (refs are Copy) | `["hello", "world"]` → 32 bytes container; data elsewhere |
| `(T, U, V)` tuple | Sum+padding | Stack | Stack (inline); Heap (if any field owns heap data) | Owns all elements | Copies if all fields `Copy`; moves otherwise | `(1i32, 2.0f64, 'a')` → 16+pad bytes; fields may own heap data |
| Struct (inline fields) | Sum+padding | Stack | Stack (inline); Heap (if any field owns heap data) | Owns all fields | Copies if all fields `Copy`; moves otherwise | `Point { x: 0, y: 0 }` → 8+pad bytes; fields may own heap data |

**Key Notes:**

- **"Stack-Only (No Indirection)"** means the **container structure itself** is allocated on the stack without using Box, Rc, Arc, or other pointer types to store its elements/fields. Elements/fields are embedded directly into the container's memory layout.

- **Container Location**: Where the container metadata/structure resides (always stack for this table). For `[&str; N]`, the fat pointers themselves are stack-allocated.

- **Value(s) Location**: Where the actual data lives. Primitives are inline in the container. For types owning heap-allocated data (like String) or references (like &str), actual data may reside on the heap or in the binary's data section. Fat pointers point to this data but are not themselves the data.

- **Critical distinction**: The container structure has no indirection, but its elements/fields **may own** heap data:
  - `[String; 3]`: Container on stack (48 bytes); the array **owns** each of the three String elements, which in turn own heap data
  - `[&str; N]`: Container on stack (N×16 bytes of fat pointers); **owns** the references themselves, which reference data elsewhere (heap or binary section)
  - `(i32, String)`: Container on stack; **owns** all fields, including the String which owns heap data

- **Not "all data on stack"**: Use this table for types whose container structure is stack-allocated with no indirection, regardless of whether elements own heap data or are references.

- Stack size may vary due to struct field alignment padding. For tuples and structs, actual size = sum of field sizes + alignment padding.

***

## Table 2: Heap-Backed Types (Owned Smart Pointers) <a href="#table-2-heap-backed-types-owned-smart-pointers-" class="header-link">🔗</a>

| Type | Stack Size | Container Location | Value(s) Location | Ownership Semantics | Copy/Move | Example<svg width="350" height="1"></svg> |
| :-- | :-- | :-- | :-- | :-- | :-- | :-- |
| String | 24 bytes (ptr+len+cap) | Stack | Heap (UTF-8 bytes) | Single owner; exclusive | Move | `String::from("hello")` |
| `Vec<T>` | 24 bytes (ptr+len+cap) | Stack | Heap (elements) | Single owner; exclusive | Move | `vec![1, 2, 3]` |
| `Box<T>` | 8 bytes (pointer) | Stack | Heap (value T) | Single owner; exclusive | Move | `Box::new(42i32)` |
| `Rc<T>` | 8 bytes/clone (pointer) | Stack (pointer only) | Heap (16B control block + value T) | Shared ownership; ref-counted | Clone only | `Rc::new(value)` with multiple clones |
| `Arc<T>` | 8 bytes/clone (pointer) | Stack (pointer only) | Heap (16B control block + value T) | Thread-safe shared; atomic ref-counted | Clone only | `Arc::new(value)` for thread safety |
| `HashMap<K,V>` | 48–56 bytes (metadata) | Stack | Heap (hash buckets + entries) | Single owner; exclusive | Move | Empty: ~48 bytes; 3 entries: ~72–96 bytes |
| `BTreeMap<K,V>` | 48 bytes (metadata) | Stack | Heap (tree nodes + entries) | Single owner; exclusive | Move | Ordered keys; efficient iteration |

**Key Notes:**

- **"Heap-Backed Types (Owned Smart Pointers)"** means the container uses **indirection**—it stores pointers on the stack that point to heap-allocated data. This differs from Table 1 where elements are embedded inline.
- **Memory split**:
    - **Stack portion**: Container metadata (pointer, length, capacity for String/Vec; hash table metadata for HashMap/BTreeMap)
    - **Heap portion**: Actual data (string bytes, vector elements, key-value entries) freed when owner drops
- **All are Move types by default**—ownership transferred on assignment unless explicitly cloned. Cloning increases reference count (for Rc/Arc) or duplicates data (for Box).
- **Reference-counted types** (`Rc<T>`, `Arc<T>`):
  - Control block (allocated once on heap): 8-byte strong count + 8-byte weak count = 16 bytes total
  - Control block is **shared** across all clones; each clone holds only an 8-byte pointer
  - Strong count tracks ownership; when it reaches 0, data is freed
- **HashMap/BTreeMap difference**: These allocate both a control structure (hash buckets or tree nodes) AND individual entries on the heap, making their memory overhead less predictable than String/Vec.
- **Version specificity**: All sizes and details assume 64-bit systems with Rust 1.90.0. Control block sizes and metadata may vary across Rust versions; verify using `std::mem::size_of()` on your platform.

***

## Table 3: Borrowed References & Slices (Non-Owning) <a href="#table-3-borrowed-references--slices-non-owning-" class="header-link">🔗</a>

| Type | Stack Size | Points To | Lifetime | Mutability | Allocation Responsibility | Example<svg width="350" height="1"></svg>  |
| :-- | :-- | :-- | :-- | :-- | :-- | :-- |
| `&T` immutable reference | 8 bytes (pointer) | Stack/heap/static value | Cannot outlive referent | Read-only | Referent owner responsible | `&x` (x: i32) |
| `&mut T` mutable reference | 8 bytes (pointer) | Stack/heap mutable; exclusive | Cannot outlive referent; exclusive | Read+write; exclusive | Referent owner responsible | `&mut s` (s: String) |
| `&[T]` immutable slice | 16 bytes (ptr+len) | Contiguous array/Vec/stack/heap/static | Cannot outlive source | Read-only | Referent owner responsible | `&arr[0..3]` (fat pointer) |
| `&mut [T]` mutable slice | 16 bytes (ptr+len) | Contiguous mutable data; exclusive | Cannot outlive source; exclusive | Read+write; exclusive | Referent owner responsible | `&mut vec![..]` (fat pointer) |
| `&str` string slice | 16 bytes (ptr+len) | UTF-8 String/literal/static/heap | Cannot outlive source | Read-only | Referent owner responsible | `"hello"` or `&my_string[0..5]` (fat pointer) |
| `*const T` raw pointer (unsafe) | 8 bytes (pointer) | Arbitrary memory | Programmer responsible | None (unsafe) | Programmer responsible | `ptr as *const i32` (unsafe block) |
| `*mut T` mutable raw pointer (unsafe) | 8 bytes (pointer) | Arbitrary mutable memory | Programmer responsible | None (unsafe) | Programmer responsible | `ptr as *mut i32` (unsafe block) |

**Key Notes:** References don't allocate or manage memory; they borrow existing data. Fat pointers (`&[T]`, `&str`) are 16 bytes (pointer+length) because compiler cannot statically know size of dynamically-sized types (DSTs). Borrowing rules enforced at compile-time: at most one `&mut` **OR** multiple `&` simultaneously.

***

## Table 4: Semantics & Dispatch Matrix <a href="#table-4-semantics--dispatch-matrix-" class="header-link">🔗</a>

| Trait | Stack-Only | Heap-Backed | References | Notes |
| :-- | :-- | :-- | :-- | :-- |
| **Owns data?** | Yes | Yes | No | References borrow; no ownership transfer |
| **Allocates memory?** | No | Yes | No | Heap types manage allocation on drop |
| **Move on assignment?** | Copy: No; Move: Yes | Yes (default) | Yes (ptr copied only) | Ownership semantics differ by type |
| **Thread-safe sharing?** | Copy types: Yes | Arc<T>: Yes; Others: Requires Arc<Mutex<T>> | &T: Yes; &mut T: Single-threaded exclusive | Send + Sync traits govern thread safety |
| **Mutability control** | Via mut binding | mut binding or interior mutability | &: immutable; &mut: exclusive | No mutable sharing via & at compile-time |


***

## Table 5: Generic Wrappers, Enums & Special Types <a href="#table-5-generic-wrappers-enums--special-types-" class="header-link">🔗</a>

| Type<svg width="250" height="1"></svg> | Stack Size | Heap Allocation | Data Location | Semantics | Example & Notes |
| :-- | :-- | :-- | :-- | :-- | :-- |
| `()` unit type | 0 bytes (ZST) | None | N/A | Empty value; single instance | `let x: () = ();` used in `Result<(), Error>` |
| `!` never type | 0 bytes (diverges) | None | N/A | Never returns; unreachable code | `fn panic() -> ! { ... }` indicates divergence |
| `PhantomData<T>` | 0 bytes (ZST) | None | N/A | Zero-sized marker; no runtime cost | `struct Invariant<T>(PhantomData<T>)` for type safety |
| `Option<bool>` niche-optimized | 1 byte | None (inlined) | Stack | Discriminant fits in invalid bits | `Option<bool>`: 1 byte; None uses spare bit pattern |
| `Option<NonZeroU32>` niche-optimized | 4 bytes | None (inlined) | Stack | Niche optimized: 0 invalid, encodes None | 4 bytes (no overhead vs NonZeroU32) |
| `Option<String>` niche-optimized | 24 bytes | String data on heap (if Some) | Stack (metadata); Heap (data if Some) | Null pointer optimization (NPO) | 24 bytes; same size as String! |
| `Option<&T>` niche-optimized | 8 bytes | None (ptr only) | Stack (ptr only) | Niche optimized: null encodes None | No extra size vs `&T` |
| `Option<usize>` non-optimized | 16 bytes (2× usize) | None (inlined) | Stack | No niche optimization possible | `usize::MAX` not usable for encoding |
| `Result<T, E>` enum | max(T,E)+1 byte | Depends on T & E | Stack/heap (varies) | Largest variant + 1-byte discriminant | Often niche-optimized away |
| `Pin<P>` self-referential marker | Same as P | Same as P | Same as P | Prevents T from being moved | `Pin<&mut T>`: 8 bytes; `Pin<Box<T>>`: 8 bytes |
| `&dyn Trait` trait object ref | 16 bytes (ptr+vtable) | Data on heap (if owned) | Stack (fat ptr); Heap (data) | Runtime polymorphism; fat pointer | `&dyn Display`: 16 bytes (pointer+vtable ptr) |
| `Box<dyn Trait>` owned trait object | 8 bytes (pointer) | Data on heap + vtable reference | Stack (ptr only); Heap (data+vtable) | Single owner; trait object on heap | 8 bytes stack + 16 bytes heap (ptr+vtable) |
| `Rc<dyn Trait>` shared trait object | 8 bytes (pointer) | Control block (16B) + data + vtable | Stack (ptr only); Heap (ctrl+data) | Shared; reference-counted | Control block shared across clones |
| `Arc<dyn Trait>` thread-safe trait object | 8 bytes (pointer) | Control block (16B) + data + vtable | Stack (ptr only); Heap (ctrl+data) | Thread-safe shared trait object | Atomic reference counting for threads |
| `Cell<T>` shared mutability | 8+sizeof(T) bytes | Value T inlined; no separate alloc | Stack (no separate allocation) | Interior mutability; no runtime checks | `Cell<u32>`: 8 bytes; mut not required |
| `RefCell<T>` runtime borrow checking | 8+sizeof(T)+8 bytes | Borrow count + value T inlined | Stack (no separate allocation) | Interior mutability; runtime borrow checks | `RefCell<u32>`: 12 bytes; panics on violation |

**Key Notes:** Niche optimization reduces enum size by using invalid values to encode discriminants. ZST (zero-sized type) has no runtime representation. Fat pointers for trait objects hold pointer+vtable for runtime polymorphism.

***

## Table 6: Function Pointers & Closures <a href="#table-6-function-pointers--closures-" class="header-link">🔗</a>

| Type <svg width="250" height="1"></svg>| Stack Size | Heap Allocation | Semantics | Example<svg width="300" height="1"></svg> | Notes |
| :-- | :-- | :-- | :-- | :-- | :-- |
| `fn(i32) -> i32` function pointer | 8 bytes | None | Copy; function code in binary | `let f: fn(i32) -> i32 = add;` | Points to machine code address |
| `fn() -> !` diverging function pointer | 8 bytes | None | Copy; never returns | `let f: fn() -> ! = panic;` | Type-safe divergence |
| Closure `|x| x+1` (no captures) | 0 bytes (ZST) | None | Copy; statically known | `let add = |x| x+1;` if all captured vars are Copy | Captures nothing or only Copy values |
| Closure `|x: i32| x+1` (FnOnce) | Captured vars size | None (stack) | Move semantics; consumes env | `move || x` where x is moved | Consumes captured variables |
| Closure `|| &x` (Fn) | Size of `&x` | None (stack) | Shared reference to env | `|| &x` where x is borrowed | Multiple calls; shared borrow |
| Closure `|| &mut x` (FnMut) | Size of `&mut x` | None (stack) | Mutable reference to env | `|| &mut x` where x is exclusive | Multiple calls; exclusive borrow |

**Key Notes:** Function pointers are Copy types. Closures capturing no values or only Copy values are themselves Copy. Closures that capture mutable references cannot be Copy. Closure trait (`Fn`, `FnMut`, `FnOnce`) is determined at compile-time.

***

## Memory Breakdown Example: Fixed Array of String Slices <a href="#memory-breakdown-example-fixed-array-of-string-slices-" class="header-link">🔗</a>

```
Stack (48 bytes total):
├── arr[0]: &str = (ptr → "hello", len=5)     [16 bytes: 8 ptr + 8 len]
├── arr[1]: &str = (ptr → "world", len=5)     [16 bytes: 8 ptr + 8 len]
└── arr[2]: &str = (ptr → "rust", len=4)      [16 bytes: 8 ptr + 8 len]

Binary/Static Memory (read-only section):
├── "hello" (5 bytes UTF-8)
├── "world" (5 bytes UTF-8)
└── "rust"  (4 bytes UTF-8)

Total: 48 bytes stack + 14 bytes read-only section
```


***

## Alignment & Padding Rules <a href="#alignment--padding-rules-" class="header-link">🔗</a>

### Default Representation (`repr(Rust)`) <a href="#default-representation-reprrust-" class="header-link">🔗</a>

Rust applies compiler optimizations to field ordering, minimizing padding and improving cache locality.

```rust
struct Example { a: u32, b: u8, c: u16 }  // 8 bytes (not 7)
// Compiler may reorder: u8(1) + padding(1) + u16(2) + u32(4) = 8 bytes
```


### Fixed Representation (`repr(C)`) <a href="#fixed-representation-reprc-" class="header-link">🔗</a>

Fields maintain declaration order; ensures FFI compatibility with C.

```rust
#[repr(C)]
struct CCompatible { a: u32, b: u8, c: u16 }  // 8 bytes with guaranteed layout
```


### Packed Representation (`repr(packed)`) <a href="#packed-representation-reprpacked-" class="header-link">🔗</a>

Removes padding; trades speed for size. Unaligned access can harm performance.

```rust
#[repr(packed)]
struct Compact { a: u32, b: u8, c: u16 }  // 7 bytes, potentially slower access
```


***

## Copy Trait Behavior <a href="#copy-trait-behavior-" class="header-link">🔗</a>

**Copy types** (numeric, bool, char, arrays/tuples of Copy types):

- Implicit duplication on assignment
- Stack-only; cannot contain non-Copy types
- No `Drop` implementation allowed

**Non-Copy (Move) types** (String, Vec, Box, Rc, Arc):

- Ownership transferred on assignment
- Heap data freed when owner drops

***

## Thread Safety & Send + Sync Traits <a href="#thread-safety--send--sync-traits-" class="header-link">🔗</a>

| Type Category | Send | Sync | Notes |
| :-- | :-- | :-- | :-- |
| Copy types (i32, bool, etc.) | ✓ | ✓ | Can be shared safely across threads |
| `Arc<T>` | ✓ if T: Send | ✓ if T: Sync | Atomic reference counting for thread-safe sharing |
| `Rc<T>` | ✗ | ✗ | Not thread-safe; single-threaded only |
| `&T` | ✓ if T: Sync | ✓ if T: Sync | Immutable reference; safe for shared access |
| `&mut T` | ✓ if T: Send | ✗ | Exclusive mutable access; cannot be Sync |
| `Mutex<T>` | ✓ if T: Send | ✓ if T: Send | Runtime mutual exclusion for safe shared mutation |


***


## Example to demo as many as we can <a href="#example-to-demo-as-many-as-we-can-" class="header-link">🔗</a>

```rust
use std::collections::HashMap;
use std::rc::Rc;

fn main() {
    println!("=== GAME INVENTORY SYSTEM - MEMORY LAYOUT DEMO ===\n");

    // ====================================================================
    // TABLE 1: Stack-Only Types (No Indirection)
    // ====================================================================

    println!("STACK-ONLY GAME DATA\n");

    // Player stats: Simple primitives stored directly on stack
    #[derive(Debug, Copy, Clone)]
    struct PlayerStats {
        health: u32,           // 4 bytes stack
        level: u8,             // 1 byte stack
        experience: u64,       // 8 bytes stack
        gold: u32,             // 4 bytes stack
    }                          // Total: ~17 bytes + padding = ~24 bytes stack

    let mut player = PlayerStats {
        health: 100,
        level: 5,
        experience: 1250,
        gold: 500,
    };
    println!("Player Stats: {:?}", player);
    println!("  -> ~24 bytes on stack (Copy type, can duplicate cheaply)\n");

    // Game coordinates: Fixed array of primitives
    let spawn_points: [i32; 6] = [10, 20, 30, 40, 50, 60];  // 24 bytes stack
    println!("Spawn Points: {:?}", spawn_points);
    println!("  -> 24 bytes on stack (fixed-size array)\n");

    // Quick stats tuple
    let combat_roll: (u8, u8, bool) = (6, 12, true);
    println!("Combat Roll (attack, defense, critical): {:?}", combat_roll);
    println!("  -> ~4 bytes on stack (tuple of primitives)\n");

    // ====================================================================
    // TABLE 2: Heap-Backed Types (Owned Smart Pointers)
    // ====================================================================

    println!("HEAP-BACKED GAME DATA\n");

    // Item definition: Needs heap storage for variable-length name
    #[derive(Debug, Clone)]
    struct Item {
        id: u32,                    // 4 bytes stack
        name: String,               // 24 bytes stack (metadata); heap for string
        damage: u16,                // 2 bytes stack
        durability: u8,             // 1 byte stack
    }                               // Total: ~31 bytes stack + heap allocation

    let sword = Item {
        id: 101,
        name: String::from("Excalibur"),
        damage: 75,
        durability: 100,
    };
    println!("Weapon: ID={}, Name={}, Damage={}, Durability={}", 
             sword.id, sword.name, sword.damage, sword.durability);
    println!("  -> ~32 bytes stack (struct); \"Excalibur\" on heap\n");

    let shield = Item {
        id: 102,
        name: String::from("Guardian Shield"),
        damage: 0,
        durability: 150,
    };
    println!("Shield: ID={}, Name={}, Damage={}, Durability={}", 
             shield.id, shield.name, shield.damage, shield.durability);
    println!("  -> ~32 bytes stack (struct); \"Guardian Shield\" on heap\n");

    // Player inventory: Dynamic array of items
    let mut inventory: Vec<Item> = vec![sword.clone(), shield.clone()];
    println!("Inventory: {} items", inventory.len());
    println!("  -> 24 bytes stack (Vec metadata); Item structs + strings on heap\n");

    // Add consumable to inventory
    let potion = Item {
        id: 201,
        name: String::from("Health Potion"),
        damage: 0,
        durability: 1,
    };
    inventory.push(potion.clone());
    println!("Added potion: ID={}, Name={}, Durability={}", 
             potion.id, potion.name, potion.durability);
    println!("  -> New heap allocation for Item + string\n");

    // Shared quest reference: Multiple systems need to read same data
    #[derive(Debug)]
    struct Quest {
        id: u32,
        title: String,
        objectives: Vec<String>,
        reward_gold: u32,
    }

    let shared_quest = Rc::new(Quest {
        id: 1,
        title: String::from("Slay the Dragon"),
        objectives: vec![
            String::from("Find dragon's lair"),
            String::from("Defeat dragon"),
            String::from("Return to king"),
        ],
        reward_gold: 1000,
    });
    
    // UI, Quest Log, and Achievement systems all reference same quest
    let quest_in_ui = Rc::clone(&shared_quest);
    let _quest_in_log = Rc::clone(&shared_quest);
    let _quest_for_achievements = Rc::clone(&shared_quest);

    println!("Shared Quest: ID={}, Title=\"{}\"", 
             shared_quest.id, shared_quest.title);
    println!("  -> 4 owners (UI, Log, Achievements, Original)");
    println!("  -> Each: 8 bytes stack pointer");
    println!("  -> One control block on heap (16 bytes: strong + weak counts)");
    println!("  -> One Quest struct on heap (shared by all)");
    println!("  -> Strong count: {}\n", Rc::strong_count(&shared_quest));

    // Player name cache: HashMap for fast lookup
    let mut player_names: HashMap<u32, String> = HashMap::new();
    player_names.insert(1, String::from("Alice"));
    player_names.insert(2, String::from("Bob"));
    player_names.insert(3, String::from("Charlie"));

    println!("Online Players: {:?}", player_names);
    println!("  -> ~48 bytes stack (HashMap metadata)");
    println!("  -> Hash buckets + entries on heap\n");

    // Large texture buffer: Too big for stack, use Box
    struct TextureData {
        pixels: Box<[u8; 1_000_000]>,  // 1MB texture
    }
    
    let player_texture = TextureData {
        pixels: Box::new([0; 1_000_000]),
    };
    println!("Player Texture (1MB): Loaded {} bytes", 
             player_texture.pixels.len());
    println!("  -> 8 bytes stack (Box pointer)");
    println!("  -> 1,000,000 bytes on heap (prevents stack overflow)\n");

    // ====================================================================
    // HYBRID: Real-World Game State
    // ====================================================================

    println!("COMPLETE GAME STATE\n");

    struct GameState {
        player_stats: PlayerStats,         // Stack-only (24 bytes)
        inventory: Vec<Item>,              // Heap-backed (24 bytes stack + heap)
        active_quest: Option<Rc<Quest>>,   // Shared ownership (16 bytes stack)
        save_file_path: String,            // Heap-backed (24 bytes stack + heap)
        frame_count: u64,                  // Stack-only (8 bytes)
    }                                      // Total: ~96 bytes stack + multiple heap regions

    let game = GameState {
        player_stats: player,
        inventory: inventory.clone(),
        active_quest: Some(quest_in_ui),
        save_file_path: String::from("/saves/game1.dat"),
        frame_count: 12450,
    };

    println!("Game State Loaded:");
    println!("  Player: Level {}, {} HP, {} gold", 
             game.player_stats.level, 
             game.player_stats.health, 
             game.player_stats.gold);
    println!("  Inventory: {} items", game.inventory.len());
    if let Some(ref quest) = game.active_quest {
        println!("  Active Quest: ID={}, Title=\"{}\"", quest.id, quest.title);
        println!("  Objectives: {} total", quest.objectives.len());
        println!("  Reward: {} gold", quest.reward_gold);
    }
    println!("  Save Path: {}", game.save_file_path);
    println!("  Frame: #{}\n", game.frame_count);

    println!("MEMORY BREAKDOWN:");
    println!("  Stack: ~96 bytes (GameState container)");
    println!("  Heap: Inventory items, quest data, save path string");
    println!("  Shared: Quest owned by UI, Log, and Achievement systems\n");

    // ====================================================================
    // GAME LOOP SIMULATION
    // ====================================================================

    println!("SIMULATING GAME ACTIONS\n");

    // Stack-based computation: Fast, no allocation
    player.experience += 100;
    player.gold += 50;
    println!("Gained 100 XP and 50 gold (stack operations only)\n");

    // Heap allocation: Pickup new item
    let scroll = Item {
        id: 301,
        name: String::from("Magic Scroll"),
        damage: 25,
        durability: 3,
    };
    inventory.push(scroll.clone());
    println!("Picked up: {} (Damage: {}, Durability: {})", 
             scroll.name, scroll.damage, scroll.durability);
    println!("  New heap allocation - Inventory size: {} items\n", inventory.len());

    // Quest progress: Shared data updated
    println!("Quest objective completed: \"{}\"", 
             shared_quest.objectives[0]);
    println!("  All {} owners see the same quest data (Rc sharing)\n",
             Rc::strong_count(&shared_quest));

    // Combat: Copy types duplicated cheaply
    let player_before_combat = player;  // Copy entire PlayerStats (stack copy)
    player.health -= 30;
    println!("Combat! Health: {} -> {} (Copy semantics allowed rollback)", 
             player_before_combat.health, 
             player.health);
    println!("  Previous state still valid: {:?}\n", player_before_combat);

    // ====================================================================
    // MEMORY EFFICIENCY LESSON
    // ====================================================================

    println!("KEY TAKEAWAYS:\n");
    println!("1. Stack-only types (PlayerStats, coordinates): Fast, Copy-able");
    println!("2. Heap types (String, Vec): Flexible size, Move semantics");
    println!("3. Rc<T>: Share read-only data across systems efficiently");
    println!("4. Box<T>: Move large data to heap to prevent stack overflow");
    println!("5. HashMap: Fast lookup for dynamic key-value data");
    println!("\nChoose the right type for the right job!");
}

```