#![allow(non_snake_case)]
use regex::Regex;
use std::fs;
use std::io::{self, Write};

fn main() -> io::Result<()> {
    // Read the input file
    let input_path =
        "/Users/amritsingh/work/amritsingh183.github.io/_posts/2025-01-05-rust-mem-ref.md";
    let output_path = "output.md";

    let content = fs::read_to_string(input_path)?;
    let modified_content = process_markdown_headings(&content);
    let modifiedContentRef: &String = &modified_content;
    println!("{:?}", modifiedContentRef);
    let mustWrite = true;
    if mustWrite {
        // Write the modified content
        fs::write(output_path, modified_content)?;
        println!("Processing complete! Output written to {}", output_path);
    }
    Ok(())
}

fn process_markdown_headings(content: &str) -> String {
    // Regex to match lines that start with one or more # followed by space and text
    // Captures: (1) the # symbols, (2) the heading text
    let heading_regex = Regex::new(r"^(#{1,6})\s+(.+?)$").unwrap();

    content
        .lines()
        .map(|line| {
            if let Some(captures) = heading_regex.captures(line) {
                let hashes = captures.get(1).unwrap().as_str();
                let heading_text = captures.get(2).unwrap().as_str();

                // Generate the anchor from the heading text
                let anchor = generate_anchor(heading_text);

                // Construct the new line with the chain link appended
                format!("{} {} [chain](#{}-)", hashes, heading_text, anchor)
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<String>>()
        .join("\n")
}

fn generate_anchor(heading_text: &str) -> String {
    // Convert heading text to a GitHub/Markdown-style anchor
    // 1. Convert to lowercase
    // 2. Replace spaces with hyphens
    // 3. Remove special characters except hyphens
    // 4. Append "-chain" at the end

    heading_text
        .to_lowercase()
        .chars()
        .map(|c| {
            if c.is_alphanumeric() {
                c
            } else if c.is_whitespace() {
                '-'
            } else if c == ':' || c == '.' {
                // Remove colons and periods
                '\0'
            } else {
                // Keep other characters as hyphens or remove them
                '-'
            }
        })
        .filter(|&c| c != '\0')
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<&str>>()
        .join("-")
}
