#![allow(non_snake_case)]
use regex::Regex;
use std::fs;
use std::io::{self, Write};

fn main() -> io::Result<()> {
    // Read the input file
    let input_path =
        "/Users/amritsingh/work/amritsingh183.github.io/_posts/2025-02-09-rust-ownership.md";
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
                format!(
                    "{} {} <a href=\"#{}-\" class=\"header-link\">🔗</a>",
                    hashes, heading_text, anchor,
                )
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<String>>()
        .join("\n")
}

fn generate_anchor(heading_text: &str) -> String {
    heading_text
        .to_lowercase()
        .chars()
        .map(|c| match c {
            // Keep alphanumeric and hyphens
            'a'..='z' | '0'..='9' | '-' => c,
            // Convert spaces to hyphens
            ' ' => '-',
            // Remove all other characters (punctuation, symbols)
            _ => '\0',
        })
        .filter(|&c| c != '\0')
        .collect::<String>()
        .trim_matches('-')
        .to_string()
}
