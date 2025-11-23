---
layout: post
title: "When to Use (and Avoid) `collect()`"
date: 2025-23-14 15:41:48 +0530
categories: rust concepts
last_updated: 2025-23-14
---

# When to Use (and Avoid) `collect()` in Rust 1.91.1

I spent an embarrassing amount of time debugging a lifetime error last week. Let me save you the trouble.

I was building a parser for OCR responses from my llamacpp server running LightOCR. The HTTP endpoint returns raw text with line-by-line confidence scores, and I needed to extract just the successful parses. Simple enough, right? Wrong.

### The Error That Made Me Question Everything

Here's what I wrote first:

```rust
let lines = ocr_response
    .lines()
    .map(|line| self.parse_line(line))
    .collect::<Vec<Result<_, _>>>()  // Creates temporary Vec
    .iter()                           // Borrows the Vec
    .filter(|d| d.is_ok());          // Vec dropped, reference dangling!
```

The compiler rejected this immediately. The `Vec` gets created and dropped in the same expression, but `.iter()` tries to borrow from it. Once I understood what was happening, the error message made perfect sense: you can't borrow from something that ceased to exist.

### The Fix That Actually Compiled

I switched to `flatten()` and everything worked:

```rust
let lines: Vec<_> = ocr_response
    .lines()
    .map(|line| self.parse_line(line))
    .flatten()
    .collect();
```

This compiles perfectly because `Result<T, E>` implements `IntoIterator`. When you call `flatten()` on an iterator of Results, it yields the `Ok` values and discards the `Err` values. No intermediate collection needed.

### Then Clippy Made Me Feel Silly

Running `cargo clippy` pointed me toward the more idiomatic solution:

```rust
fn parse_lines(&self, ocr_response: &str) {
    let successful_lines: Vec<_> = ocr_response
        .lines()
        .filter_map(|line| self.parse_line(line).ok())
        .collect();
    println!("{:?}", successful_lines);
}
```

This is what you'll see in production Rust code. The `filter_map()` combinator with `.ok()` makes the intent explicit: filter for successful results and extract their values in one pass. When processing batch OCR responses with thousands of lines, this pattern handles everything efficiently while remaining immediately readable.

### How I Think About `collect()` Now

After working on this OCR pipeline for a few months, I've developed some instincts about when to materialize an iterator:

**Use `collect()` when you actually need a collection:**

- Storing parsed document data for JSON serialization back to the client
- Running multiple passes over preprocessed text for different extraction patterns
- Converting to `HashSet` to deduplicate repeated text blocks from multi-page scans
- Calculating statistics on confidence scores (mean, median, percentiles for quality assessment)

**Skip the intermediate `collect()` when:**

- You're still chaining operations—keep the iterator lazy
- Processing happens in a single pass (filtering, transforming, building responses)
- You'd immediately call `.iter()` again (that's literally the mistake I made)
- Working with `Result` or `Option` types—use `filter_map()` or `flatten()` instead

I used to collect entire OCR batch responses into `Vec<String>` just to count high-confidence lines. Switching to `.filter(|line| line.confidence > 0.8).count()` eliminated the allocation entirely. Same answer, zero memory pressure.

### The Lesson

Both `flatten()` and `filter_map()` solve my original problem correctly, but `filter_map()` communicates intent more clearly when working with `Result` or `Option` types. Think of iterators as pipelines: keep the data flowing until you genuinely need to materialize it. Your code will be faster, use less memory, and any Rust developer reviewing your pull request will know exactly what you meant.
