---
layout: post
title: "Linked Lists in Rust 1.90.0: Practice example"
date: 2025-05-07 21:23:00 +0530
categories: rust concepts
last_updated: 2025-10-22
---

# **Linked Lists in Rust 1.90.0: Practice example**

Let us pratice that what we have learned in the [basics of Rust](https://amritsingh183.github.io/rust/concepts/2025/01/01/rust-var-const-lifetimes.html)
## Introduction

Linked lists represent one of the most challenging data structures to implement correctly in Rust due to the language's strict ownership and borrowing rules. This document provides a complete technical reference for understanding and implementing various types of linked lists in safe Rust, with unsafe alternatives presented only at the end.

## Why Linked Lists Are Challenging in Rust

### The Ownership Dilemma

Rust's core ownership rule states that each value can have only one owner at a time. However, linked list structures inherently require multiple references to the same 

- **Doubly-linked lists**: Each node has two owners (previous and next nodes)
- **Circular lists**: The tail node references the head, creating shared ownership
- **Lists with cycles**: Multiple nodes may reference the same target node

This fundamental conflict between Rust's ownership model and linked list semantics requires careful use of smart pointers and interior mutability.

### Borrowing Rules and Mutability

Safe Rust enforces that you can have either multiple immutable references OR one mutable reference to a value, never both simultaneously. In linked lists, we often need to:

- Traverse the list (shared access)
- Modify nodes during traversal (mutable access)
- Maintain pointers to multiple nodes

This necessitates runtime-checked borrowing through `RefCell`.

## Singly-Linked List: The Foundation

### Basic Structure Using Box

The simplest linked list in Rust uses `Box<T>` for heap allocation and `Option` for representing the end of the list:

```rust
#[derive(Debug, Clone)]
struct Node<T> {
     T,
    next: Option<Box<Node<T>>>,
}

#[derive(Debug)]
pub struct SinglyLinkedList<T> {
    head: Option<Box<Node<T>>>,
}
```

### Core Implementation

```rust
impl<T> Node<T> {
    fn new( T) -> Self {
        Node { data, next: None }
    }
}

impl<T> SinglyLinkedList<T> {
    pub fn new() -> Self {
        SinglyLinkedList { head: None }
    }

    pub fn push_front(&mut self,  T) {
        let mut new_node = Box::new(Node::new(data));
        new_node.next = self.head.take();
        self.head = Some(new_node);
    }

    pub fn pop_front(&mut self) -> Option<T> {
        self.head.take().map(|node| {
            self.head = node.next;
            node.data
        })
    }

    pub fn peek(&self) -> Option<&T> {
        self.head.as_ref().map(|node| &node.data)
    }

    pub fn peek_mut(&mut self) -> Option<&mut T> {
        self.head.as_mut().map(|node| &mut node.data)
    }

    pub fn is_empty(&self) -> bool {
        self.head.is_none()
    }
}
```

### Iteration Support

```rust
impl<T> SinglyLinkedList<T> {
    pub fn iter(&self) -> Iter<T> {
        Iter {
            next: self.head.as_deref(),
        }
    }

    pub fn iter_mut(&mut self) -> IterMut<T> {
        IterMut {
            next: self.head.as_deref_mut(),
        }
    }
}

pub struct Iter<'a, T> {
    next: Option<&'a Node<T>>,
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        self.next.map(|node| {
            self.next = node.next.as_deref();
            &node.data
        })
    }
}

pub struct IterMut<'a, T> {
    next: Option<&'a mut Node<T>>,
}

impl<'a, T> Iterator for IterMut<'a, T> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        self.next.take().map(|node| {
            self.next = node.next.as_deref_mut();
            &mut node.data
        })
    }
}
```

### Drop Implementation

Recursive drop can cause stack overflow for very long lists. Here's an iterative approach:

```rust
impl<T> Drop for SinglyLinkedList<T> {
    fn drop(&mut self) {
        let mut current = self.head.take();
        while let Some(mut node) = current {
            current = node.next.take();
            // node is dropped here
        }
    }
}
```

## Sorted Singly-Linked List

### Structure and Constraints

A sorted linked list maintains elements in order, requiring `Ord` trait bounds:

```rust
#[derive(Debug)]
pub struct SortedLinkedList<T: Ord> {
    head: Option<Box<Node<T>>>,
}

impl<T: Ord> SortedLinkedList<T> {
    pub fn new() -> Self {
        SortedLinkedList { head: None }
    }

    pub fn insert(&mut self,  T) {
        let mut new_node = Box::new(Node::new(data));

        // Find insertion point
        match self.head {
            None => {
                self.head = Some(new_node);
            }
            Some(ref mut head) if new_node.data < head.data => {
                new_node.next = self.head.take();
                self.head = Some(new_node);
            }
            Some(ref mut head) => {
                let mut current = head.as_mut();
                loop {
                    match current.next {
                        None => {
                            current.next = Some(new_node);
                            break;
                        }
                        Some(ref mut next) if new_node.data < next.data => {
                            new_node.next = current.next.take();
                            current.next = Some(new_node);
                            break;
                        }
                        Some(ref mut next) => {
                            current = next;
                        }
                    }
                }
            }
        }
    }

    pub fn remove(&mut self,  &T) -> bool {
        match self.head {
            None => false,
            Some(ref mut head) if &head.data == data => {
                self.head = head.next.take();
                true
            }
            Some(ref mut head) => {
                let mut current = head.as_mut();
                loop {
                    match current.next {
                        None => return false,
                        Some(ref mut next) if &next.data == data => {
                            current.next = next.next.take();
                            return true;
                        }
                        Some(ref mut next) => {
                            current = next;
                        }
                    }
                }
            }
        }
    }
}
```

### Merging Two Sorted Lists

```rust
impl<T: Ord> SortedLinkedList<T> {
    pub fn merge(mut list1: Self, mut list2: Self) -> Self {
        let mut result = SortedLinkedList::new();
        let mut tail = &mut result.head;

        let mut l1 = list1.head.take();
        let mut l2 = list2.head.take();

        loop {
            match (l1.take(), l2.take()) {
                (Some(mut node1), Some(node2)) => {
                    if node1.data <= node2.data {
                        l2 = Some(node2);
                        l1 = node1.next.take();
                        *tail = Some(node1);
                    } else {
                        l1 = Some(node1);
                        l2 = node2.next.take();
                        *tail = Some(node2);
                    }
                }
                (Some(node), None) => {
                    *tail = Some(node);
                    break;
                }
                (None, Some(node)) => {
                    *tail = Some(node);
                    break;
                }
                (None, None) => break,
            }

            // Advance tail pointer
            tail = &mut tail.as_mut().unwrap().next;
        }

        result
    }
}
```

## Doubly-Linked List with Safe Rust

### Understanding Rc, RefCell, and Weak

To implement a doubly-linked list safely, we need three smart pointer types:

- **`Rc<T>`**: Reference counting for shared ownership
- **`RefCell<T>`**: Interior mutability with runtime borrow checking
- **`Weak<T>`**: Non-owning references to prevent reference cycles

### Structure Definition

```rust
use std::rc::{Rc, Weak};
use std::cell::RefCell;

type Link<T> = Option<Rc<RefCell<Node<T>>>>;
type WeakLink<T> = Option<Weak<RefCell<Node<T>>>>;

#[derive(Debug)]
struct Node<T> {
     T,
    next: Link<T>,
    prev: WeakLink<T>,
}

#[derive(Debug)]
pub struct DoublyLinkedList<T> {
    head: Link<T>,
    tail: WeakLink<T>,
}
```

### Core Implementation

```rust
impl<T> Node<T> {
    fn new( T) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Node {
            data,
            next: None,
            prev: None,
        }))
    }
}

impl<T> DoublyLinkedList<T> {
    pub fn new() -> Self {
        DoublyLinkedList {
            head: None,
            tail: None,
        }
    }

    pub fn push_front(&mut self,  T) {
        let new_node = Node::new(data);

        match self.head.take() {
            Some(old_head) => {
                old_head.borrow_mut().prev = Some(Rc::downgrade(&new_node));
                new_node.borrow_mut().next = Some(old_head);
            }
            None => {
                // First node - set as tail too
                self.tail = Some(Rc::downgrade(&new_node));
            }
        }

        self.head = Some(new_node);
    }

    pub fn push_back(&mut self,  T) {
        let new_node = Node::new(data);

        match self.tail.take() {
            Some(old_tail_weak) => {
                if let Some(old_tail) = old_tail_weak.upgrade() {
                    new_node.borrow_mut().prev = Some(old_tail_weak);
                    old_tail.borrow_mut().next = Some(new_node.clone());
                    self.tail = Some(Rc::downgrade(&new_node));
                }
            }
            None => {
                // First node
                self.tail = Some(Rc::downgrade(&new_node));
                self.head = Some(new_node);
            }
        }
    }

    pub fn pop_front(&mut self) -> Option<T> {
        self.head.take().map(|old_head| {
            match old_head.borrow_mut().next.take() {
                Some(new_head) => {
                    new_head.borrow_mut().prev = None;
                    self.head = Some(new_head);
                }
                None => {
                    // List is now empty
                    self.tail = None;
                }
            }

            Rc::try_unwrap(old_head)
                .ok()
                .expect("Multiple references to node")
                .into_inner()
                .data
        })
    }

    pub fn pop_back(&mut self) -> Option<T> {
        self.tail.take().and_then(|old_tail_weak| {
            old_tail_weak.upgrade().map(|old_tail| {
                match old_tail.borrow_mut().prev.take() {
                    Some(new_tail_weak) => {
                        if let Some(new_tail) = new_tail_weak.upgrade() {
                            new_tail.borrow_mut().next = None;
                            self.tail = Some(new_tail_weak);
                        }
                    }
                    None => {
                        // List is now empty
                        self.head = None;
                    }
                }

                Rc::try_unwrap(old_tail)
                    .ok()
                    .expect("Multiple references to node")
                    .into_inner()
                    .data
            })
        })
    }

    pub fn is_empty(&self) -> bool {
        self.head.is_none()
    }
}
```

### Traversal Methods

```rust
impl<T: std::fmt::Display> DoublyLinkedList<T> {
    pub fn display_forward(&self) {
        let mut current = self.head.clone();
        while let Some(node) = current {
            print!("{} <-> ", node.borrow().data);
            current = node.borrow().next.clone();
        }
        println!("None");
    }

    pub fn display_backward(&self) {
        if let Some(ref tail_weak) = self.tail {
            if let Some(tail) = tail_weak.upgrade() {
                let mut current = Some(tail);
                while let Some(node) = current {
                    print!("{} <-> ", node.borrow().data);
                    current = node.borrow()
                        .prev
                        .as_ref()
                        .and_then(|weak| weak.upgrade());
                }
                println!("None");
            }
        }
    }
}
```

## Detecting Cycles: Floyd's Algorithm

### The Tortoise and Hare Approach

Floyd's Cycle Detection Algorithm uses two pointers moving at different speeds to detect cycles efficiently:

```rust
impl<T> SinglyLinkedList<T> {
    pub fn has_cycle(&self) -> bool {
        if self.head.is_none() {
            return false;
        }

        let mut slow = self.head.as_deref();
        let mut fast = self.head.as_deref();

        while fast.is_some() && fast.unwrap().next.is_some() {
            slow = slow.unwrap().next.as_deref();
            fast = fast.unwrap().next.as_ref()
                .and_then(|n| n.next.as_deref());

            if slow.is_some() && fast.is_some() {
                if std::ptr::eq(
                    slow.unwrap() as *const Node<T>,
                    fast.unwrap() as *const Node<T>
                ) {
                    return true;
                }
            }
        }

        false
    }
}
```

### Finding Cycle Start Point

```rust
impl<T> SinglyLinkedList<T> {
    pub fn find_cycle_start(&self) -> Option<*const Node<T>> {
        if !self.has_cycle() {
            return None;
        }

        let mut slow = self.head.as_deref();
        let mut fast = self.head.as_deref();

        // Phase 1: Find meeting point
        while fast.is_some() && fast.unwrap().next.is_some() {
            slow = slow.unwrap().next.as_deref();
            fast = fast.unwrap().next.as_ref()
                .and_then(|n| n.next.as_deref());

            if slow.is_some() && fast.is_some() {
                if std::ptr::eq(
                    slow.unwrap() as *const Node<T>,
                    fast.unwrap() as *const Node<T>
                ) {
                    break;
                }
            }
        }

        // Phase 2: Find cycle start
        slow = self.head.as_deref();
        while slow.is_some() && fast.is_some() {
            if std::ptr::eq(
                slow.unwrap() as *const Node<T>,
                fast.unwrap() as *const Node<T>
            ) {
                return Some(slow.unwrap() as *const Node<T>);
            }
            slow = slow.unwrap().next.as_deref();
            fast = fast.unwrap().next.as_deref();
        }

        None
    }
}
```

### Mathematical Intuition

The algorithm works because if there's a cycle of length $$C$$ and the cycle starts $$M$$ nodes from the head:

1. When slow enters the cycle, fast is already $$k$$ steps into it
2. They meet after slow travels $$C - k$$ more steps
3. At meeting point, slow traveled $$M + C - k$$, fast traveled $$2(M + C - k)$$
4. The difference equals multiples of cycle length: $$M + C - k = nC$$
5. Therefore $$M = (n-1)C + k$$, meaning $$M$$ steps from meeting point equals $$M$$ steps from head

## Lists with Multiple References to Same Node

### Using Rc for Shared Nodes

When multiple nodes need to reference the same node, `Rc` provides shared ownership:

```rust
use std::rc::Rc;
use std::cell::RefCell;

#[derive(Debug)]
struct SharedNode<T> {
     T,
    refs: Vec<Weak<RefCell<SharedNode<T>>>>,
}

#[derive(Debug)]
pub struct MultiRefList<T> {
    nodes: Vec<Rc<RefCell<SharedNode<T>>>>,
}

impl<T> MultiRefList<T> {
    pub fn new() -> Self {
        MultiRefList { nodes: Vec::new() }
    }

    pub fn add_node(&mut self,  T) -> usize {
        let node = Rc::new(RefCell::new(SharedNode {
            data,
            refs: Vec::new(),
        }));
        self.nodes.push(node);
        self.nodes.len() - 1
    }

    pub fn add_reference(&mut self, from_idx: usize, to_idx: usize) -> Result<(), &'static str> {
        if from_idx >= self.nodes.len() || to_idx >= self.nodes.len() {
            return Err("Index out of bounds");
        }

        let to_node_weak = Rc::downgrade(&self.nodes[to_idx]);
        self.nodes[from_idx].borrow_mut().refs.push(to_node_weak);
        Ok(())
    }

    pub fn get_references(&self, idx: usize) -> Result<Vec<usize>, &'static str> {
        if idx >= self.nodes.len() {
            return Err("Index out of bounds");
        }

        let node = self.nodes[idx].borrow();
        let mut result = Vec::new();

        for weak_ref in &node.refs {
            if let Some(strong) = weak_ref.upgrade() {
                // Find index of this node
                for (i, n) in self.nodes.iter().enumerate() {
                    if Rc::ptr_eq(n, &strong) {
                        result.push(i);
                        break;
                    }
                }
            }
        }

        Ok(result)
    }

    pub fn reference_count(&self, idx: usize) -> Result<usize, &'static str> {
        if idx >= self.nodes.len() {
            return Err("Index out of bounds");
        }
        Ok(Rc::strong_count(&self.nodes[idx]))
    }
}
```

### Graph-Like Structures

For more complex scenarios resembling directed graphs:

```rust
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct GraphNode<T> {
    id: usize,
     T,
    edges: Vec<usize>,
}

#[derive(Debug)]
pub struct GraphList<T> {
    nodes: HashMap<usize, Rc<RefCell<GraphNode<T>>>>,
    next_id: usize,
}

impl<T> GraphList<T> {
    pub fn new() -> Self {
        GraphList {
            nodes: HashMap::new(),
            next_id: 0,
        }
    }

    pub fn add_node(&mut self,  T) -> usize {
        let id = self.next_id;
        self.next_id += 1;

        let node = Rc::new(RefCell::new(GraphNode {
            id,
            data,
            edges: Vec::new(),
        }));

        self.nodes.insert(id, node);
        id
    }

    pub fn add_edge(&mut self, from: usize, to: usize) -> Result<(), &'static str> {
        if !self.nodes.contains_key(&to) {
            return Err("Target node does not exist");
        }

        if let Some(node) = self.nodes.get(&from) {
            node.borrow_mut().edges.push(to);
            Ok(())
        } else {
            Err("Source node does not exist")
        }
    }

    pub fn get_neighbors(&self, id: usize) -> Result<Vec<usize>, &'static str> {
        if let Some(node) = self.nodes.get(&id) {
            Ok(node.borrow().edges.clone())
        } else {
            Err("Node does not exist")
        }
    }

    pub fn has_path_dfs(&self, from: usize, to: usize) -> bool {
        use std::collections::HashSet;

        fn dfs_helper<T>(
            graph: &GraphList<T>,
            current: usize,
            target: usize,
            visited: &mut HashSet<usize>,
        ) -> bool {
            if current == target {
                return true;
            }

            visited.insert(current);

            if let Ok(neighbors) = graph.get_neighbors(current) {
                for neighbor in neighbors {
                    if !visited.contains(&neighbor) {
                        if dfs_helper(graph, neighbor, target, visited) {
                            return true;
                        }
                    }
                }
            }

            false
        }

        let mut visited = HashSet::new();
        dfs_helper(self, from, to, &mut visited)
    }
}
```

## Advanced Operations

### Reversing a Linked List

```rust
impl<T> SinglyLinkedList<T> {
    pub fn reverse(&mut self) {
        let mut prev: Option<Box<Node<T>>> = None;
        let mut current = self.head.take();

        while let Some(mut node) = current {
            let next = node.next.take();
            node.next = prev;
            prev = Some(node);
            current = next;
        }

        self.head = prev;
    }
}
```

### Finding the Middle Element

```rust
impl<T: Clone> SinglyLinkedList<T> {
    pub fn find_middle(&self) -> Option<T> {
        if self.head.is_none() {
            return None;
        }

        let mut slow = self.head.as_deref();
        let mut fast = self.head.as_deref();

        while fast.is_some() && fast.unwrap().next.is_some() {
            slow = slow.unwrap().next.as_deref();
            fast = fast.unwrap().next.as_ref()
                .and_then(|n| n.next.as_deref());
        }

        slow.map(|node| node.data.clone())
    }
}
```

### Removing Duplicates from Sorted List

```rust
impl<T: PartialEq> SortedLinkedList<T> {
    pub fn remove_duplicates(&mut self) {
        if self.head.is_none() {
            return;
        }

        let mut current = self.head.as_mut();

        while let Some(node) = current {
            let mut next_different = node.next.as_mut();

            // Skip all nodes with same value
            while next_different.is_some() && 
                  next_different.as_ref().unwrap().data == node.data {
                next_different = next_different.unwrap().next.as_mut();
            }

            // Link to first different node
            node.next = next_different.and_then(|n| {
                Some(Box::new(Node {
                     std::mem::replace(&mut n.data, unsafe { std::mem::zeroed() }),
                    next: n.next.take(),
                }))
            });

            current = node.next.as_mut();
        }
    }
}
```

## Testing and Examples

### Basic Usage Examples

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_singly_linked_list() {
        let mut list = SinglyLinkedList::new();
        assert!(list.is_empty());

        list.push_front(1);
        list.push_front(2);
        list.push_front(3);

        assert_eq!(list.peek(), Some(&3));
        assert_eq!(list.pop_front(), Some(3));
        assert_eq!(list.pop_front(), Some(2));
        assert_eq!(list.pop_front(), Some(1));
        assert_eq!(list.pop_front(), None);
    }

    #[test]
    fn test_sorted_list() {
        let mut list = SortedLinkedList::new();
        list.insert(5);
        list.insert(2);
        list.insert(8);
        list.insert(1);
        list.insert(9);

        // Verify sorted order
        let values: Vec<i32> = list.iter().cloned().collect();
        assert_eq!(values, vec![1, 2, 5, 8, 9]);
    }

    #[test]
    fn test_doubly_linked_list() {
        let mut list = DoublyLinkedList::new();
        list.push_front(2);
        list.push_front(1);
        list.push_back(3);
        list.push_back(4);

        assert_eq!(list.pop_front(), Some(1));
        assert_eq!(list.pop_back(), Some(4));
        assert_eq!(list.pop_front(), Some(2));
        assert_eq!(list.pop_back(), Some(3));
        assert!(list.is_empty());
    }

    #[test]
    fn test_multi_ref_list() {
        let mut list = MultiRefList::new();
        let n0 = list.add_node("A");
        let n1 = list.add_node("B");
        let n2 = list.add_node("C");

        // Multiple nodes reference node 1
        list.add_reference(n0, n1).unwrap();
        list.add_reference(n2, n1).unwrap();

        let refs = list.get_references(n1).unwrap();
        assert_eq!(list.reference_count(n1).unwrap(), 3); // list + 2 refs
    }
}
```

## Performance Considerations

### Time Complexity Summary

| Operation | Singly-Linked | Doubly-Linked | Array/Vec |
|-----------|---------------|---------------|-----------|
| Access by index | O(n) | O(n) | O(1) |
| Insert at front | O(1) | O(1) | O(n) |
| Insert at back | O(n) | O(1) | O(1) amortized |
| Delete at front | O(1) | O(1) | O(n) |
| Delete at back | O(n) | O(1) | O(1) |
| Search | O(n) | O(n) | O(n) |

### Space Complexity

- **Singly-linked**: 1 pointer overhead per node
- **Doubly-linked**: 2 pointers (or 1 `Rc` + 1 `Weak`) per node
- **With `Rc<RefCell<>>`**: Additional allocation overhead for reference counting

### When to Use Linked Lists

Linked lists are appropriate when:

1. You need O(1) insertion/deletion at known positions
2. You're implementing lock-free concurrent structures
3. You can't afford amortized costs of dynamic arrays
4. You're working in embedded contexts with intrusive lists

For most use cases, `Vec` or `VecDeque` will be more efficient due to cache locality.

***

## ⚠️ UNSAFE RUST IMPLEMENTATIONS ⚠️

### Critical Warning

The following section contains `unsafe` code that bypasses Rust's safety guarantees. Use only when:

1. You fully understand Rust's memory model and aliasing rules
2. You've profiled and confirmed safe alternatives are insufficient
3. You can maintain invariants that the compiler cannot verify
4. You're prepared to handle undefined behavior risks

**Unsafe code can cause**:
- Memory corruption
- Data races
- Segmentation faults
- Undefined behavior that appears to work but fails unpredictably

### Raw Pointer-Based Singly-Linked List

```rust
use std::ptr;

pub struct UnsafeList<T> {
    head: *mut Node<T>,
    tail: *mut Node<T>,
}

struct Node<T> {
     T,
    next: *mut Node<T>,
}

impl<T> UnsafeList<T> {
    pub fn new() -> Self {
        UnsafeList {
            head: ptr::null_mut(),
            tail: ptr::null_mut(),
        }
    }

    pub fn push_back(&mut self,  T) {
        unsafe {
            let new_node = Box::into_raw(Box::new(Node {
                data,
                next: ptr::null_mut(),
            }));

            if self.tail.is_null() {
                self.head = new_node;
                self.tail = new_node;
            } else {
                (*self.tail).next = new_node;
                self.tail = new_node;
            }
        }
    }

    pub fn pop_front(&mut self) -> Option<T> {
        unsafe {
            if self.head.is_null() {
                None
            } else {
                let old_head = Box::from_raw(self.head);
                self.head = old_head.next;

                if self.head.is_null() {
                    self.tail = ptr::null_mut();
                }

                Some(old_head.data)
            }
        }
    }
}

impl<T> Drop for UnsafeList<T> {
    fn drop(&mut self) {
        while self.pop_front().is_some() {}
    }
}

// SAFETY: UnsafeList owns its nodes exclusively
unsafe impl<T: Send> Send for UnsafeList<T> {}
```

### Doubly-Linked List with Raw Pointers

```rust
pub struct UnsafeDoublyLinkedList<T> {
    head: *mut DNode<T>,
    tail: *mut DNode<T>,
}

struct DNode<T> {
     T,
    next: *mut DNode<T>,
    prev: *mut DNode<T>,
}

impl<T> UnsafeDoublyLinkedList<T> {
    pub fn new() -> Self {
        UnsafeDoublyLinkedList {
            head: ptr::null_mut(),
            tail: ptr::null_mut(),
        }
    }

    pub fn push_front(&mut self,  T) {
        unsafe {
            let new_node = Box::into_raw(Box::new(DNode {
                data,
                next: self.head,
                prev: ptr::null_mut(),
            }));

            if self.head.is_null() {
                self.tail = new_node;
            } else {
                (*self.head).prev = new_node;
            }

            self.head = new_node;
        }
    }

    pub fn push_back(&mut self,  T) {
        unsafe {
            let new_node = Box::into_raw(Box::new(DNode {
                data,
                next: ptr::null_mut(),
                prev: self.tail,
            }));

            if self.tail.is_null() {
                self.head = new_node;
            } else {
                (*self.tail).next = new_node;
            }

            self.tail = new_node;
        }
    }

    pub fn pop_front(&mut self) -> Option<T> {
        unsafe {
            if self.head.is_null() {
                None
            } else {
                let old_head = Box::from_raw(self.head);
                self.head = old_head.next;

                if self.head.is_null() {
                    self.tail = ptr::null_mut();
                } else {
                    (*self.head).prev = ptr::null_mut();
                }

                Some(old_head.data)
            }
        }
    }

    pub fn pop_back(&mut self) -> Option<T> {
        unsafe {
            if self.tail.is_null() {
                None
            } else {
                let old_tail = Box::from_raw(self.tail);
                self.tail = old_tail.prev;

                if self.tail.is_null() {
                    self.head = ptr::null_mut();
                } else {
                    (*self.tail).next = ptr::null_mut();
                }

                Some(old_tail.data)
            }
        }
    }
}

impl<T> Drop for UnsafeDoublyLinkedList<T> {
    fn drop(&mut self) {
        while self.pop_front().is_some() {}
    }
}

unsafe impl<T: Send> Send for UnsafeDoublyLinkedList<T> {}
```

### Why Unsafe Can Be Faster

Unsafe implementations avoid:

1. **Reference counting overhead**: No `Rc::clone()` or atomic operations
2. **Borrow checking at runtime**: No `RefCell::borrow_mut()` panics
3. **Indirection**: Direct pointer manipulation vs wrapped smart pointers
4. **Memory allocation**: Can use arena allocators or custom allocation strategies

**However**: The performance gain is often negligible compared to using `Vec` or `VecDeque` for most workloads due to cache effects.

### Safety Invariants You Must Maintain

When writing unsafe linked list code, you must ensure:

1. **Exclusive access**: No aliasing mutable pointers
2. **Valid pointers**: All non-null pointers point to valid, initialized memory
3. **Proper ownership**: Clear ownership transfer points to prevent double-frees
4. **No dangling references**: Drop order prevents use-after-free
5. **Thread safety**: Proper `Send`/`Sync` implementations if used across threads

***

## Conclusion

Implementing linked lists in Rust demonstrates the language's memory safety guarantees and ownership model. While safe implementations using `Rc<RefCell<>>` are verbose, they provide runtime-verified correctness. For production code, prefer standard library collections like `Vec` and `VecDeque` unless you have specific requirements that necessitate linked structures.

The patterns shown here—combining `Rc` for shared ownership, `RefCell` for interior mutability, and `Weak` for breaking cycles—apply broadly to other data structures like trees and graphs.

***