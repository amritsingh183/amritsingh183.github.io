//! Draws the figures of the post `_posts/2026-09-29-ml-drawn-out.md` ("Machine Learning, Drawn Out") as SVG files.
//! Standard library only.
//!
//! Compile:  rustc --edition 2024 -O -o /tmp/make_book_figures ml/book/make_book_figures.rs
//! Run:      /tmp/make_book_figures ml/book/figures        (the argument is the output folder)
//!
//! The SVG builder and the colour tokens are `ml/figures/svg.rs`, shared with `ml/figures/make_figures.rs`. Each part
//! module draws the figures of its chapters from numbers it computes itself; nothing drawn is typed as a literal result.
//! Two runs write byte-identical files. A figure's file stem is `b<chapter>-<n>-<slug>`, and two figures with one stem
//! stop the run.
#![forbid(unsafe_code)]

#[path = "../figures/svg.rs"]
mod svg;
mod part_a;
mod part_b;
mod part_c;

/// A figure: its file stem (`b<chapter>-<n>-<slug>`, chapter as two digits) and its SVG text.
pub type Figure = (String, String);

fn main() {
    let out = match std::env::args().nth(1) {
        Some(p) => std::path::PathBuf::from(p),
        None => {
            eprintln!("usage: make_book_figures <output folder>");
            std::process::exit(2);
        }
    };
    if let Err(e) = std::fs::create_dir_all(&out) {
        eprintln!("make_book_figures: cannot create {}: {e}", out.display());
        std::process::exit(1);
    }
    let mut figures: Vec<Figure> = Vec::new();
    figures.extend(part_a::figures());
    figures.extend(part_b::figures());
    figures.extend(part_c::figures());
    figures.sort_by(|a, b| a.0.cmp(&b.0));
    for pair in figures.windows(2) {
        if pair[0].0 == pair[1].0 {
            eprintln!("make_book_figures: two figures share the stem {}", pair[0].0);
            std::process::exit(1);
        }
    }
    for (stem, text) in &figures {
        let path = out.join(format!("{stem}.svg"));
        if let Err(e) = std::fs::write(&path, text) {
            eprintln!("make_book_figures: cannot write {}: {e}", path.display());
            std::process::exit(1);
        }
    }
    println!("{} figures written to {}", figures.len(), out.display());
}
