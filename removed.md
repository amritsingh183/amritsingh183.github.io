`2025-01-01-revised-rust-var-const-lifetimes.md` is a full revised article for about Rust that covers `var`, `const` and `static` and `borrowing and ownership` without covering `lifetimes for references`. You must critique the revised document as we want a perfect and completely (technically, practically, theoretically) valid(valid in all dimensions) document for Rust. We must ensure everything is valid for Rust against reddit, stack overflow and https://doc.rust-lang.org/edition-guide/rust-2024/index.html. `lifetimes` are a separate blog post in `https://amritsingh183.github.io/rust/concepts/2025/02/09/rust-ownership.html`. The article must look like it is written by a human software engineer


- We must not cover `lifetimes for references` in this article
- we already have comprehensive memory layout reference in `https://amritsingh183.github.io/rust/concepts/2025/01/05/rust-mem-ref.html`
- lifetimes are covered here `https://amritsingh183.github.io/rust/concepts/2025/02/09/rust-ownership.html`
- We must not cover Interior Mutability, Mutex, RwLock, RefCell at all. Since we are not promoting `unsafe` code
- WE must cover `Safe Global State Patterns` only and not the unsafe ones

If you have improvement to suggest, give me markdown patches.. the rust code should be in ```rust``` and everything else in ```text```.