//! Prints the three ML posts (`_posts/2026-09-28-rust-candle-ml-guide.md`, `_posts/2026-09-28-rust-ml-handbook.md` and
//! `_posts/2026-09-29-ml-drawn-out.md`) as double-sided A4 PDFs. Standard library only.
//!
//! Compile:  rustc --edition 2024 -O -o /tmp/make_print ml/print/make_print.rs
//! Run:      /tmp/make_print ml/print _posts/2026-09-28-rust-candle-ml-guide.md _posts/2026-09-28-rust-ml-handbook.md
//!               _posts/2026-09-29-ml-drawn-out.md
//!           (one command line; the first argument is the output folder, which holds print.css; each source becomes
//!           <stem>.pdf there, and any subset of the posts may be given)
//!
//! Each source is a Jekyll post directly under the site's `_posts/` folder; the site root is the folder holding `_config.yml`.
//! The build skips the front matter itself, resolves the posts' root-absolute image paths (`/ml/figures/…`) against the site
//! root, removes the web-only MathJax line that the handbook and the book carry, so that Chrome never touches the network,
//! and prints the post's published address under Jekyll's default permalink (`/:categories/:year/:month/:day/:title.html`,
//! in UTC as GitHub Pages builds) as the cover's Source line. It stops on any front matter, file name or `_config.yml`
//! setting it cannot map to that address.
//!
//! pandoc turns the remaining Markdown into HTML (maths as MathML, code highlighted); headless Google Chrome prints that HTML,
//! styled by print.css, to PDF. PRINT_PANDOC and PRINT_CHROME override the tool paths; PRINT_KEEP_TMP=1 keeps the
//! working folder with every pass's HTML and PDF.
//!
//! Chrome can neither start a part on a right-hand page nor print a heading's page number in the contents, so each
//! document is printed at least twice. After a pass the build reads, from the PDF's named destinations, the page on
//! which every part and every contents target landed; it writes those numbers into the contents and puts a blank
//! page before each part that would otherwise open on a left-hand page. It stops when a pass prints exactly what
//! that pass assumed, and fails with a message naming the entry otherwise.
//!
//! A part is a unit with its own running head: the `#` sections when a document has more than one H1 (the guide and the
//! book, together with the `##` sections before their first part), otherwise the `##` sections (the handbook). The first
//! part after the cover, parts whose titles begin with "Part " or with a number and a dot, and the first part
//! whose title begins with "Appendix" open on a right-hand page.
#![forbid(unsafe_code)]

use std::collections::{BTreeSet, HashMap, HashSet};
use std::path::{Path, PathBuf};

/// The message printed after `make_print: ` when the build stops.
struct Fail(String);

fn fail<T>(message: impl Into<String>) -> Result<T, Fail> {
    Err(Fail(message.into()))
}

/// The one web-only line a post may carry (MathJax for the maths of the handbook and the book on the site). It is
/// removed byte for byte before pandoc so that the print needs no network; any other <script> stops the build.
const WEB_SCRIPT: &str =
    r#"<script id="MathJax-script" async src="https://cdn.jsdelivr.net/npm/mathjax@3/es5/tex-svg.js"></script>"#;

/// Numbers that must agree with print.css, and the build's limits.
mod page {
    /// Room for one code line: the 172 mm text block less a code frame's 7 pt padding and 0.5 pt border each side.
    pub const CODE_LINE_PT: f64 = 172.0 * 72.0 / 25.4 - 2.0 * 7.0 - 2.0 * 0.5;
    /// print.css `pre { font-size }`.
    pub const CODE_PT: f64 = 8.5;
    /// Smallest size a program output or a text drawing is reduced to so that its lines fit unbroken.
    pub const FLOOR_PT: f64 = 6.5;
    /// Advance width of every Menlo glyph, in ems (1233/2048).
    pub const MENLO_EM: f64 = 0.6021;
    /// Height of the text block: A4's 297 mm less the 18 mm and 19 mm margins.
    pub const BLOCK_HEIGHT_PT: f64 = (297.0 - 18.0 - 19.0) * 72.0 / 25.4;
    /// A code block taller than this cannot share a page with even a four-line lead-in (11 pt × 1.4 lines and the
    /// paragraph's 0.75 em margin), so the lead-in is not asked to stay with it: Chrome would otherwise leave one
    /// near-empty page for the lead-in and another for its last line.
    pub const TALL_PT: f64 = BLOCK_HEIGHT_PT - 4.0 * 11.0 * 1.4 - 0.75 * 11.0;
    /// print.css `pre { line-height }` and the frame's padding and border, top and bottom.
    pub const CODE_LEADING: f64 = 1.35;
    pub const FRAME_PT: f64 = 2.0 * 5.0 + 2.0 * 0.5;
    /// Tables with at most this many body rows are kept on one page.
    pub const KEEP_ROWS: usize = 12;
    pub const MAX_PASSES: usize = 4;
    pub const PASS_SECONDS: u64 = 120;
}

/// Plain-text helpers for the HTML pandoc writes.
mod text {
    /// Text content of an HTML fragment: tags removed, entities decoded.
    pub fn plain(html: &str) -> String {
        decode(&strip_tags(html))
    }

    pub fn strip_tags(s: &str) -> String {
        let mut out = String::with_capacity(s.len());
        let mut in_tag = false;
        for ch in s.chars() {
            match (in_tag, ch) {
                (false, '<') => in_tag = true,
                (true, '>') => in_tag = false,
                (false, c) => out.push(c),
                (true, _) => {}
            }
        }
        out
    }

    /// Decodes the named entities pandoc writes and numeric character references; leaves anything else as it is.
    pub fn decode(s: &str) -> String {
        let mut out = String::with_capacity(s.len());
        let mut rest = s;
        while let Some(i) = rest.find('&') {
            out.push_str(&rest[..i]);
            let tail = &rest[i..];
            let decoded = tail.find(';').filter(|&end| end <= 10).and_then(|end| {
                let entity = &tail[1..end];
                let ch = match entity {
                    "amp" => Some('&'),
                    "lt" => Some('<'),
                    "gt" => Some('>'),
                    "quot" => Some('"'),
                    "apos" => Some('\''),
                    _ => entity
                        .strip_prefix("#x")
                        .or_else(|| entity.strip_prefix("#X"))
                        .and_then(|hex| u32::from_str_radix(hex, 16).ok())
                        .or_else(|| entity.strip_prefix('#').and_then(|dec| dec.parse().ok()))
                        .and_then(char::from_u32),
                };
                ch.map(|c| (c, end))
            });
            match decoded {
                Some((c, end)) => {
                    out.push(c);
                    rest = &tail[end + 1..];
                }
                None => {
                    out.push('&');
                    rest = &tail[1..];
                }
            }
        }
        out.push_str(rest);
        out
    }

    /// Text made safe for HTML content and attribute values.
    pub fn escape(s: &str) -> String {
        s.replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
    }

    /// A CSS string literal.
    pub fn css_string(s: &str) -> String {
        let mut out = String::from("\"");
        for ch in s.chars() {
            match ch {
                '\\' => out.push_str("\\\\"),
                '"' => out.push_str("\\\""),
                '\n' | '\r' => out.push(' '),
                c => out.push(c),
            }
        }
        out.push('"');
        out
    }

    /// A `file://` URL for an absolute path, percent-encoding every byte outside the unreserved set and `/`.
    pub fn file_url(path: &std::path::Path) -> String {
        let mut url = String::from("file://");
        for &b in path.as_os_str().as_encoded_bytes() {
            if b.is_ascii_alphanumeric() || b"-._~/".contains(&b) {
                url.push(b as char);
            } else {
                url.push_str(&format!("%{b:02X}"));
            }
        }
        url
    }
}

/// Reshapes pandoc's HTML for print and splits it into the cover and the parts.
mod doc {
    use crate::{Fail, fail, page, text};
    use std::collections::{BTreeMap, HashMap};
    use std::fmt::Write as _;
    use std::path::Path;

    pub struct Division {
        pub title: String,
        pub body: String,
        pub recto: bool,
    }

    /// A contents line that links to a heading and so gets a page number.
    pub struct Entry {
        pub id: String,
        pub text: String,
    }

    pub struct Doc {
        pub title: String,
        pub cover: String,
        pub divisions: Vec<Division>,
        pub entries: Vec<Entry>,
        pub summary: Vec<String>,
    }

    /// Private-use characters around a contents target's id; each pass replaces them with the page number.
    pub const NUM_OPEN: char = '\u{E000}';
    pub const NUM_CLOSE: char = '\u{E001}';

    /// Level and id of a line that starts with a heading such as `<h2 id="x">`.
    fn heading(line: &str) -> Option<(u8, &str)> {
        let b = line.as_bytes();
        if b.len() < 3 || b[0] != b'<' || b[1] != b'h' || !(b'1'..=b'6').contains(&b[2]) {
            return None;
        }
        let rest = line[3..].strip_prefix(" id=\"")?;
        let q = rest.find('"')?;
        rest[q..].starts_with("\">").then_some((b[2] - b'0', &rest[..q]))
    }

    /// The id of a paragraph that holds nothing but an anchor: `<p><a id="x"></a></p>`.
    fn anchor_paragraph(line: &str) -> Option<&str> {
        let id = line.strip_prefix("<p><a id=\"")?.strip_suffix("\"></a></p>")?;
        (!id.contains('"')).then_some(id)
    }

    pub fn prepare(pandoc: &str, src: &Path, site: &Path) -> Result<Doc, Fail> {
        let name = src.display().to_string();
        let shape = |line: usize, what: &str| Fail(format!("{name}: {what} (pandoc HTML line {})", line + 1));
        if pandoc.contains(NUM_OPEN) || pandoc.contains(NUM_CLOSE) {
            return fail(format!(
                "{name}: contains U+E000 or U+E001, which the build uses as markers"
            ));
        }
        let mut lines: Vec<String> = pandoc.lines().map(str::to_string).collect();
        while lines.last().is_some_and(|l| l.trim().is_empty()) {
            lines.pop();
        }
        let mut summary = Vec::new();

        // The title: the document must open with its H1.
        let first = lines.first().map(String::as_str).unwrap_or("");
        if heading(first).map(|h| h.0) != Some(1) || !first.ends_with("</h1>") {
            return Err(shape(0, "expected the document to open with an H1"));
        }
        let title = text::plain(first);

        // Anchors written as `<a id>` on the line before a heading become pandoc paragraphs of their own. Chrome places
        // such an empty anchor on the page before a heading that opens a new page, so links to it are pointed at the
        // heading's own id instead and the anchor is dropped.
        let mut alias: HashMap<String, String> = HashMap::new();
        let mut dropped = 0;
        let mut i = 0;
        while i < lines.len() {
            let Some(id) = anchor_paragraph(&lines[i]).map(str::to_string) else {
                i += 1;
                continue;
            };
            let not_before_heading = || {
                shape(
                    i,
                    &format!("anchor paragraph <p><a id=\"{id}\"></a></p> is not followed by a heading"),
                )
            };
            let j = (i + 1..lines.len())
                .find(|&j| !lines[j].trim().is_empty())
                .ok_or_else(not_before_heading)?;
            let heading_id = heading(&lines[j])
                .map(|h| h.1.to_string())
                .ok_or_else(not_before_heading)?;
            if heading_id == id {
                dropped += 1;
            } else {
                alias.insert(id, heading_id);
            }
            lines.remove(i);
        }

        let (mut html, retargeted) = retarget_links(&lines.join("\n"), &alias);
        html = code_blocks(&html, &name, &mut summary)?;
        html = figures(&html, src, site, &mut summary)?;
        html = mark_uri_links(&html, &mut summary);
        html = keep_short_tables(&html, &name, &mut summary)?;
        html = narrow_hats(&html, &mut summary);
        let mut lines: Vec<String> = html.lines().map(str::to_string).collect();

        let heads: Vec<(u8, String, String, usize)> = lines
            .iter()
            .enumerate()
            .filter_map(|(k, l)| heading(l).map(|(level, id)| (level, id.to_string(), text::plain(l), k)))
            .collect();
        let count = |level: u8| heads.iter().filter(|h| h.0 == level).count();
        summary.insert(
            0,
            format!(
                "headings: {} h1, {} h2, {} h3, {} deeper; anchors before headings: {} replaced by the heading's id ({retargeted} links), {dropped} duplicates dropped",
                count(1),
                count(2),
                count(3),
                heads.len() - count(1) - count(2) - count(3),
                alias.len()
            ),
        );

        // Contents: every entry that links to a heading gets a number slot; the bold Part lines of the guide and the
        // book are linked to their H1 so that they get one too.
        let contents_line = heads.iter().find(|h| h.2 == "Contents").map(|h| h.3);
        let ci = contents_line.ok_or_else(|| Fail(format!("{name}: no heading named Contents")))?;
        let ul = (ci + 1..lines.len())
            .find(|&k| !lines[k].trim().is_empty())
            .filter(|&k| lines[k] == "<ul>")
            .ok_or_else(|| shape(ci, "Contents heading not followed by a <ul>"))?;
        let mut depth = 0i64;
        let mut end = None;
        for (k, l) in lines.iter().enumerate().skip(ul) {
            depth += l.matches("<ul>").count() as i64 - l.matches("</ul>").count() as i64;
            if depth == 0 {
                end = Some(k);
                break;
            }
        }
        let end = end.ok_or_else(|| shape(ul, "unbalanced <ul> in Contents"))?;
        let h1_by_text: HashMap<&str, &str> = heads
            .iter()
            .filter(|h| h.0 == 1)
            .map(|h| (h.2.as_str(), h.1.as_str()))
            .collect();
        let mut entries = Vec::new();
        let (mut linked_parts, mut unmatched) = (0, Vec::new());
        for k in ul..=end {
            let slot = |id: &str| {
                format!("<span class=\"ld\"></span><span class=\"pg\">{NUM_OPEN}{id}{NUM_CLOSE}</span></span>")
            };
            let replacement = if let Some((prefix, anchor, id, rest)) = toc_link(&lines[k]) {
                entries.push(Entry {
                    id: id.to_string(),
                    text: text::plain(anchor),
                });
                Some(format!("{prefix}<span class=\"e\">{anchor}{}{rest}", slot(id)))
            } else if let Some((strong, inner, rest)) = toc_bold(&lines[k]) {
                let label = text::plain(inner);
                match h1_by_text.get(label.as_str()) {
                    Some(&id) => {
                        linked_parts += 1;
                        entries.push(Entry {
                            id: id.to_string(),
                            text: label,
                        });
                        Some(format!(
                            "<li><span class=\"e\"><a href=\"#{id}\">{strong}</a>{}{rest}",
                            slot(id)
                        ))
                    }
                    None => {
                        unmatched.push(label);
                        None
                    }
                }
            } else {
                None
            };
            if let Some(r) = replacement {
                lines[k] = r;
            }
        }
        if entries.is_empty() {
            return Err(shape(ul, "Contents list has no entries"));
        }
        lines[ul] = "<ul class=\"toc\">".to_string();
        summary.push(format!(
            "contents: {} entries with page numbers ({linked_parts} bold part lines linked to their H1)",
            entries.len()
        ));
        for label in unmatched {
            summary.push(format!(
                "warning: Contents line \"{label}\" matches no heading; printed without a number"
            ));
        }

        // Parts.
        let h1_lines: Vec<usize> = heads.iter().filter(|h| h.0 == 1).map(|h| h.3).collect();
        let first_part = if h1_lines.len() > 1 { h1_lines[1] } else { usize::MAX };
        let by_h1 = first_part != usize::MAX;
        let starts: Vec<usize> = heads
            .iter()
            .filter(|h| {
                h.3 != 0
                    && if by_h1 {
                        h.0 == 1 || (h.0 == 2 && h.3 < first_part)
                    } else {
                        h.0 == 2
                    }
            })
            .map(|h| h.3)
            .collect();
        if starts.is_empty() {
            return fail(format!(
                "{name}: no {} headings to divide the document into parts",
                if by_h1 { "H1" } else { "H2" }
            ));
        }
        let cover_end = if lines.get(1).is_some_and(|l| l.starts_with("<p>")) {
            2
        } else {
            1
        };
        if starts[0] < cover_end {
            return Err(shape(starts[0], "a part heading sits inside the cover"));
        }
        let mut divisions = Vec::new();
        if lines[cover_end..starts[0]].iter().any(|l| !l.trim().is_empty()) {
            divisions.push(Division {
                title: title.clone(),
                body: lines[cover_end..starts[0]].join("\n"),
                recto: true,
            });
        }
        let mut appendix_seen = false;
        for (n, &k) in starts.iter().enumerate() {
            let end = starts.get(n + 1).copied().unwrap_or(lines.len());
            let t = text::plain(&lines[k]);
            let numbered = t
                .split_once('.')
                .is_some_and(|(d, _)| !d.is_empty() && d.bytes().all(|b| b.is_ascii_digit()));
            let mut recto = t.starts_with("Part ") || numbered;
            if t.starts_with("Appendix") && !appendix_seen {
                appendix_seen = true;
                recto = true;
            }
            divisions.push(Division {
                title: t,
                body: lines[k..end].join("\n"),
                recto,
            });
        }
        divisions[0].recto = true;
        summary.push(format!(
            "parts: {} with their own running head, {} opening on a right-hand page",
            divisions.len(),
            divisions.iter().filter(|d| d.recto).count()
        ));
        let cover = lines[..cover_end].join("\n");
        Ok(Doc {
            title,
            cover,
            divisions,
            entries,
            summary,
        })
    }

    /// Points every `href="#x"` whose `x` is an alias at the aliased id; returns the new HTML and the count.
    fn retarget_links(html: &str, alias: &HashMap<String, String>) -> (String, usize) {
        let mut out = String::with_capacity(html.len());
        let mut rest = html;
        let mut n = 0;
        while let Some(i) = rest.find("href=\"#") {
            let start = i + "href=\"#".len();
            out.push_str(&rest[..start]);
            let end = rest[start..].find('"').map_or(rest.len(), |q| start + q);
            match alias.get(&rest[start..end]) {
                Some(target) => {
                    out.push_str(target);
                    n += 1;
                }
                None => out.push_str(&rest[start..end]),
            }
            rest = &rest[end..];
        }
        out.push_str(rest);
        (out, n)
    }

    /// `<li>` or `<li><p>`, then `<a href="#id">…</a>`, then the rest of the line.
    fn toc_link(l: &str) -> Option<(&str, &str, &str, &str)> {
        let body = l.strip_prefix("<li>")?;
        let (prefix_len, body) = match body.strip_prefix("<p>") {
            Some(b) => (7, b),
            None => (4, body),
        };
        let after = body.strip_prefix("<a href=\"#")?;
        let q = after.find('"')?;
        if !after[q..].starts_with("\">") {
            return None;
        }
        let close = body.find("</a>")? + 4;
        Some((&l[..prefix_len], &body[..close], &after[..q], &body[close..]))
    }

    /// `<li><strong>…</strong>` then the rest of the line.
    fn toc_bold(l: &str) -> Option<(&str, &str, &str)> {
        let body = l.strip_prefix("<li>")?;
        let inner = body.strip_prefix("<strong>")?;
        let end = inner.find("</strong>")?;
        let strong_len = "<strong>".len() + end + "</strong>".len();
        Some((&body[..strong_len], &inner[..end], &body[strong_len..]))
    }

    /// Every code block: pandoc's per-line ids and anchors go; each line becomes `<span class="l" style="--i:K">`
    /// with its K leading spaces turned into padding, so a wrapped line continues further in than it began.
    /// Program output and text drawings too wide for the column are set smaller, down to a floor, to stay unbroken.
    fn code_blocks(html: &str, name: &str, summary: &mut Vec<String>) -> Result<String, Fail> {
        let shape = |what: String| Fail(format!("{name}: {what}"));
        let mut out = String::with_capacity(html.len());
        let mut rest = html;
        let mut kinds: BTreeMap<String, usize> = BTreeMap::new();
        let (mut shrunk, mut too_wide, mut wrapping_lines, mut block_no, mut tall) = (0, 0, 0, 0, 0);
        let per_line_at = |pt: f64| (page::CODE_LINE_PT / (pt * page::MENLO_EM)).floor() as usize;
        loop {
            let at = match (rest.find("<div class=\"sourceCode\""), rest.find("<pre")) {
                (None, None) => break,
                (Some(a), Some(b)) => a.min(b),
                (Some(a), None) | (None, Some(a)) => a,
            };
            out.push_str(&rest[..at]);
            let block = &rest[at..];
            block_no += 1;
            // Opener, its length, the highlighted block's id, the kind, and the closing tags.
            let (opener_len, id, kind, close) =
                if let Some(after_id) = block.strip_prefix("<div class=\"sourceCode\" id=\"") {
                    let q = after_id
                        .find('"')
                        .ok_or_else(|| shape(format!("code block {block_no}: unterminated id")))?;
                    let lang_start = after_id[q..]
                        .strip_prefix("\"><pre class=\"sourceCode ")
                        .ok_or_else(|| shape(format!("code block {block_no}: unexpected highlighted-block opener")))?;
                    let lq = lang_start
                        .find('"')
                        .ok_or_else(|| shape(format!("code block {block_no}: unterminated class")))?;
                    let lang = &lang_start[..lq];
                    let code_open = format!("\"><code class=\"sourceCode {lang}\">");
                    if !lang_start[lq..].starts_with(&code_open) {
                        return Err(shape(format!("code block {block_no}: <pre> and <code> classes differ")));
                    }
                    let opener_len = block.len() - lang_start[lq + code_open.len()..].len();
                    (
                        opener_len,
                        Some(after_id[..q].to_string()),
                        lang.to_string(),
                        "</code></pre></div>",
                    )
                } else if let Some(after) = block.strip_prefix("<pre class=\"") {
                    let q = after
                        .find('"')
                        .ok_or_else(|| shape(format!("code block {block_no}: unterminated class")))?;
                    if !after[q..].starts_with("\"><code>") {
                        return Err(shape(format!(
                            "code block {block_no}: <pre class> not followed by <code>"
                        )));
                    }
                    (
                        "<pre class=\"".len() + q + "\"><code>".len(),
                        None,
                        after[..q].to_string(),
                        "</code></pre>",
                    )
                } else if block.starts_with("<pre><code>") {
                    ("<pre><code>".len(), None, "unlabelled".to_string(), "</code></pre>")
                } else {
                    return Err(shape(format!("code block {block_no}: unrecognised <pre> opener")));
                };
            let body_len = block[opener_len..]
                .find(close)
                .ok_or_else(|| shape(format!("code block {block_no}: no {close}")))?;
            let body = &block[opener_len..opener_len + body_len];
            *kinds.entry(kind.clone()).or_default() += 1;

            let mut lines_html = String::with_capacity(body.len() + 64);
            let mut widths = Vec::new();
            let mut indents = Vec::new();
            for (m, line) in body.split('\n').enumerate() {
                let content = match &id {
                    Some(id) => {
                        let wrapper = format!(
                            "<span id=\"{id}-{}\"><a href=\"#{id}-{}\" aria-hidden=\"true\" tabindex=\"-1\"></a>",
                            m + 1,
                            m + 1
                        );
                        line.strip_prefix(wrapper.as_str())
                            .and_then(|c| c.strip_suffix("</span>"))
                            .ok_or_else(|| {
                                shape(format!(
                                    "code line {} of block {id} is not in pandoc's <span id> wrapper",
                                    m + 1
                                ))
                            })?
                    }
                    None => {
                        if line.contains('<') {
                            return Err(shape(format!(
                                "plain code block {block_no}, line {}: contains markup",
                                m + 1
                            )));
                        }
                        line
                    }
                };
                // Leading tags stay; the spaces after them become padding.
                let mut tags_end = 0;
                while content[tags_end..].starts_with('<') {
                    match content[tags_end..].find('>') {
                        Some(e) => tags_end += e + 1,
                        None => break,
                    }
                }
                let mut indent = 0;
                let mut text_start = tags_end;
                for ch in content[tags_end..].chars() {
                    match ch {
                        ' ' => indent += 1,
                        '\t' => indent += 4,
                        _ => break,
                    }
                    text_start += 1;
                }
                widths.push(text::plain(content).chars().count());
                indents.push(indent);
                write!(
                    lines_html,
                    "<span class=\"l\" style=\"--i:{indent}\">{}{}</span>",
                    &content[..tags_end],
                    &content[text_start..]
                )
                .ok();
            }
            let max_width = widths.iter().copied().max().unwrap_or(0);
            let mut size = page::CODE_PT;
            if id.is_none() && max_width > per_line_at(page::CODE_PT) {
                let fit = (page::CODE_LINE_PT / (max_width as f64 * page::MENLO_EM) * 10.0).floor() / 10.0;
                if fit >= page::FLOOR_PT {
                    size = fit;
                    shrunk += 1;
                } else {
                    too_wide += 1;
                }
            }
            let per_line = per_line_at(size);
            wrapping_lines += widths.iter().filter(|&&w| w > per_line).count();
            // Printed lines: a wrapped line continues 4 characters further in than its own indentation.
            let printed: usize = widths
                .iter()
                .zip(&indents)
                .map(|(&w, &k)| {
                    if w <= per_line {
                        1
                    } else {
                        1 + (w - per_line).div_ceil(per_line.saturating_sub(k + 4).max(8))
                    }
                })
                .sum();
            let height = printed as f64 * size * page::CODE_LEADING + page::FRAME_PT;
            let is_tall = height > page::TALL_PT;
            tall += usize::from(is_tall);
            let style = if size < page::CODE_PT {
                format!(" style=\"font-size: {size:.1}pt\"")
            } else {
                String::new()
            };
            let opener = &block[..opener_len];
            let pre_at = opener
                .find("<pre")
                .ok_or_else(|| shape(format!("code block {block_no}: no <pre> in its opener")))?;
            // The <pre> carries the reduced size; the outermost element carries class "tall".
            let mut new_opener = opener.to_string();
            new_opener.insert_str(pre_at + "<pre".len(), &style);
            if is_tall {
                let first_tag_end = new_opener.find('>').unwrap_or(0);
                match new_opener.find(" class=\"").filter(|&c| c < first_tag_end) {
                    Some(c) => new_opener.insert_str(c + " class=\"".len(), "tall "),
                    None => new_opener.insert_str("<pre".len(), " class=\"tall\""),
                }
            }
            out.push_str(&new_opener);
            out.push_str(&lines_html);
            out.push_str(close);
            rest = &block[opener_len + body_len + close.len()..];
        }
        out.push_str(rest);
        let listed: Vec<String> = kinds.iter().map(|(k, n)| format!("{k} {n}")).collect();
        summary.push(format!(
            "code blocks: {}; {shrunk} reduced to fit unbroken, {too_wide} too wide even at {} pt; {wrapping_lines} lines wrap; {tall} too tall to share a page with a lead-in",
            listed.join(", "),
            page::FLOOR_PT
        ));
        Ok(out)
    }

    /// Figures are printed from the site's own files: every `<img src>` must be a root-absolute path such as
    /// /ml/figures/x.svg, resolved against the folder holding _config.yml.
    fn figures(html: &str, src: &Path, site: &Path, summary: &mut Vec<String>) -> Result<String, Fail> {
        let mut out = String::with_capacity(html.len());
        let mut rest = html;
        let mut n = 0;
        while let Some(i) = rest.find("<img src=\"") {
            let start = i + "<img src=\"".len();
            out.push_str(&rest[..start]);
            let q = rest[start..]
                .find('"')
                .ok_or_else(|| Fail(format!("{}: unterminated <img src>", src.display())))?;
            let reference = text::decode(&rest[start..start + q]);
            if reference.contains("://") || reference.starts_with("data:") || reference.starts_with("//") {
                return fail(format!(
                    "image {reference} in {} is not a path on this site",
                    src.display()
                ));
            }
            let Some(inside) = reference.strip_prefix('/') else {
                return fail(format!(
                    "image {reference} in {} is not root-absolute (/…): in a post a relative path would point at the \
                     post's address, not at its folder",
                    src.display()
                ));
            };
            let path = site.join(inside);
            let absolute = std::fs::canonicalize(&path)
                .ok()
                .filter(|p| p.is_file())
                .ok_or_else(|| {
                    Fail(format!(
                        "figure file missing: {} (referenced from {})",
                        path.display(),
                        src.display()
                    ))
                })?;
            if !absolute.starts_with(site) {
                return fail(format!(
                    "image {reference} in {} resolves outside the site {}",
                    src.display(),
                    site.display()
                ));
            }
            out.push_str(&text::escape(&text::file_url(&absolute)));
            n += 1;
            rest = &rest[start + q..];
        }
        out.push_str(rest);
        summary.push(format!("figures: {n}"));
        Ok(out)
    }

    /// A link whose text is its own address gets class "uri", so print.css does not print the address twice.
    fn mark_uri_links(html: &str, summary: &mut Vec<String>) -> String {
        let mut out = String::with_capacity(html.len());
        let mut rest = html;
        let (mut external, mut marked) = (0, 0);
        while let Some(i) = rest.find("<a href=\"http") {
            out.push_str(&rest[..i]);
            let tag = &rest[i..];
            let open = "<a href=\"".len();
            let parts = tag[open..].find('"').and_then(|q| {
                let href = &tag[open..open + q];
                let inner_start = open + q + 2;
                tag[open + q..].starts_with("\">").then_some(())?;
                let close = tag[inner_start..].find("</a>")?;
                Some((
                    href,
                    &tag[inner_start..inner_start + close],
                    inner_start + close + "</a>".len(),
                ))
            });
            match parts {
                Some((href, inner, consumed)) => {
                    external += 1;
                    if text::plain(inner) == text::decode(href) {
                        marked += 1;
                        write!(out, "<a class=\"uri\" href=\"{href}\">{inner}</a>").ok();
                    } else {
                        out.push_str(&tag[..consumed]);
                    }
                    rest = &tag[consumed..];
                }
                None => {
                    out.push_str(&tag[..open]);
                    rest = &tag[open..];
                }
            }
        }
        out.push_str(rest);
        summary.push(format!(
            "external links: {external} ({marked} whose text is the address)"
        ));
        out
    }

    /// Short tables stay on one page; long ones continue with their heading row repeated.
    fn keep_short_tables(html: &str, name: &str, summary: &mut Vec<String>) -> Result<String, Fail> {
        if html.contains("<table ") {
            return fail(format!(
                "{name}: a <table> carries attributes; the build expects pandoc's plain <table>"
            ));
        }
        let mut out = String::with_capacity(html.len());
        let mut rest = html;
        let (mut tables, mut kept) = (0, 0);
        while let Some(i) = rest.find("<table>") {
            out.push_str(&rest[..i]);
            let table = &rest[i..];
            let end = table
                .find("</table>")
                .ok_or_else(|| Fail(format!("{name}: unterminated <table>")))?;
            let body = &table[..end];
            let rows = match (body.find("<tbody>"), body.find("</tbody>")) {
                (Some(a), Some(b)) if a < b => body[a..b].matches("<tr").count(),
                _ => 0,
            };
            tables += 1;
            if rows <= page::KEEP_ROWS {
                kept += 1;
                out.push_str("<table class=\"keep\">");
            } else {
                out.push_str("<table>");
            }
            rest = &table["<table>".len()..];
        }
        out.push_str(rest);
        summary.push(format!(
            "tables: {tables} ({kept} of at most {} rows kept on one page)",
            page::KEEP_ROWS
        ));
        Ok(out)
    }

    /// `\hat` is a narrow accent in TeX; MathML's default would stretch it over its neighbours.
    fn narrow_hats(html: &str, summary: &mut Vec<String>) -> String {
        let hat = "<mo accent=\"true\">\u{302}</mo>";
        let n = html.matches(hat).count();
        let maths = html.matches("<math ").count();
        summary.push(format!("maths: {maths} expressions, {n} hat accents kept narrow"));
        html.replace(hat, "<mo accent=\"true\" stretchy=\"false\">\u{302}</mo>")
    }

    /// The page Chrome prints in one pass: the cover, then each part in its own named page, with blanks and
    /// contents numbers as planned by the previous pass.
    pub fn page(
        d: &Doc,
        css: &str,
        source_url: &str,
        numbers: &HashMap<String, usize>,
        blanks: &std::collections::BTreeSet<usize>,
    ) -> Result<String, Fail> {
        let mut rules = String::new();
        writeln!(
            rules,
            "@page :left {{ @top-left {{ content: {}; }} }}",
            text::css_string(&d.title)
        )
        .ok();
        for (n, division) in d.divisions.iter().enumerate() {
            writeln!(
                rules,
                "@page d{}:right {{ @top-right {{ content: {}; }} }}",
                n + 1,
                text::css_string(&division.title)
            )
            .ok();
        }
        let mut body = String::new();
        write!(
            body,
            "<section class=\"cover\" style=\"page: cover\">\n{}\n<p class=\"src\">Source: {}</p>\n<nav class=\"map\" aria-hidden=\"true\">",
            d.cover,
            text::escape(source_url)
        )
        .ok();
        for n in 1..=d.divisions.len() {
            write!(body, "<a href=\"#pd-{n}\"></a>").ok();
        }
        body.push_str("</nav>\n</section>\n");
        for (n, division) in d.divisions.iter().enumerate() {
            if blanks.contains(&(n + 1)) {
                body.push_str("<div class=\"blank\" style=\"page: blank\"></div>\n");
            }
            let class = if division.recto { "division recto" } else { "division" };
            write!(
                body,
                "<section class=\"{class}\" id=\"pd-{}\" style=\"page: d{}\">\n{}\n</section>\n",
                n + 1,
                n + 1,
                division.body
            )
            .ok();
        }
        let body = fill_numbers(&body, numbers)?;
        unique_ids(&body)?;
        Ok(format!(
            "<!doctype html>\n<html lang=\"en\">\n<head>\n<meta charset=\"utf-8\">\n<title>{}</title>\n<style>\n{css}\n{rules}</style>\n</head>\n<body>\n{body}</body>\n</html>\n",
            text::escape(&d.title)
        ))
    }

    fn fill_numbers(body: &str, numbers: &HashMap<String, usize>) -> Result<String, Fail> {
        let mut out = String::with_capacity(body.len());
        let mut rest = body;
        while let Some(i) = rest.find(NUM_OPEN) {
            out.push_str(&rest[..i]);
            let after = &rest[i + NUM_OPEN.len_utf8()..];
            let j = after
                .find(NUM_CLOSE)
                .ok_or_else(|| Fail("internal: unclosed page-number marker".to_string()))?;
            if let Some(number) = numbers.get(&after[..j]) {
                write!(out, "{number}").ok();
            }
            rest = &after[j + NUM_CLOSE.len_utf8()..];
        }
        out.push_str(rest);
        Ok(out)
    }

    fn unique_ids(body: &str) -> Result<(), Fail> {
        let mut seen = std::collections::HashSet::new();
        let mut rest = body;
        while let Some(i) = rest.find(" id=\"") {
            let after = &rest[i + 5..];
            let q = after
                .find('"')
                .ok_or_else(|| Fail("internal: unterminated id attribute".to_string()))?;
            if !seen.insert(&after[..q]) {
                return fail(format!("duplicate id \"{}\"", &after[..q]));
            }
            rest = &after[q..];
        }
        Ok(())
    }
}

/// Just enough of Chrome's PDF to know which page each named destination points at: the classic cross-reference
/// table, the catalog, the page tree and the `/Dests` dictionary. Streams are never read.
mod pdf {
    use crate::{Fail, fail};
    use std::collections::HashMap;
    use std::path::Path;

    pub struct Layout {
        pub pages: usize,
        /// Destination name to page, the first page being 1.
        pub dests: HashMap<String, usize>,
    }

    fn find(hay: &[u8], needle: &[u8]) -> Option<usize> {
        hay.windows(needle.len()).position(|w| w == needle)
    }

    fn rfind(hay: &[u8], needle: &[u8]) -> Option<usize> {
        hay.windows(needle.len()).rposition(|w| w == needle)
    }

    /// The unsigned integer at the start of `b` after optional whitespace, and the index after it.
    fn number(b: &[u8]) -> Option<(usize, usize)> {
        let start = b.iter().position(|c| !c.is_ascii_whitespace())?;
        let len = b[start..].iter().take_while(|c| c.is_ascii_digit()).count();
        let n = std::str::from_utf8(&b[start..start + len]).ok()?.parse().ok()?;
        Some((n, start + len))
    }

    /// The object number in `key N 0 R`.
    fn reference(object: &str, key: &str) -> Option<usize> {
        let at = object.find(key)? + key.len();
        let rest = object[at..].trim_start();
        let digits: String = rest.chars().take_while(char::is_ascii_digit).collect();
        rest[digits.len()..]
            .trim_start()
            .starts_with("0 R")
            .then(|| digits.parse().ok())?
    }

    /// A destination's id: Chrome names it after the link's URL fragment, percent-encoded, and the PDF name escapes
    /// that again with `#xx` (an id `café` arrives as `caf#25C3#25A9`).
    fn name(raw: &str) -> String {
        String::from_utf8_lossy(&unescape(&unescape(raw.as_bytes(), b'#'), b'%')).into_owned()
    }

    /// Replaces every `mark` followed by two hexadecimal digits with the byte they spell.
    fn unescape(b: &[u8], mark: u8) -> Vec<u8> {
        let hex = |c: u8| (c as char).to_digit(16).map(|d| d as u8);
        let mut out = Vec::with_capacity(b.len());
        let mut i = 0;
        while i < b.len() {
            if b[i] == mark && i + 2 < b.len() {
                if let (Some(hi), Some(lo)) = (hex(b[i + 1]), hex(b[i + 2])) {
                    out.push(hi * 16 + lo);
                    i += 3;
                    continue;
                }
            }
            out.push(b[i]);
            i += 1;
        }
        out
    }

    pub fn read(path: &Path) -> Result<Layout, Fail> {
        let shown = path.display();
        let b = std::fs::read(path).map_err(|e| Fail(format!("cannot read {shown}: {e}")))?;
        let sx = rfind(&b, b"startxref").ok_or_else(|| Fail(format!("no startxref in {shown}")))?;
        let (xref, _) =
            number(&b[sx + "startxref".len()..]).ok_or_else(|| Fail(format!("no startxref offset in {shown}")))?;
        if !b.get(xref..).is_some_and(|t| t.starts_with(b"xref")) {
            return fail(format!("xref table not found at offset {xref} in {shown}"));
        }
        let mut offsets: HashMap<usize, usize> = HashMap::new();
        let mut pos = xref + 4;
        loop {
            let rest = &b[pos..];
            let skip = rest.iter().take_while(|c| c.is_ascii_whitespace()).count();
            if rest[skip..].starts_with(b"trailer") {
                pos += skip;
                break;
            }
            let (first, a) = number(rest).ok_or_else(|| Fail(format!("bad xref subsection in {shown}")))?;
            let (count, c) = number(&rest[a..]).ok_or_else(|| Fail(format!("bad xref subsection in {shown}")))?;
            let eol = rest[a + c..].iter().take_while(|c| c.is_ascii_whitespace()).count();
            pos += a + c + eol;
            for k in 0..count {
                let entry = b
                    .get(pos..pos + 20)
                    .ok_or_else(|| Fail(format!("truncated xref table in {shown}")))?;
                if entry[17] == b'n' {
                    let (offset, _) = number(&entry[..10]).ok_or_else(|| Fail(format!("bad xref entry in {shown}")))?;
                    offsets.insert(first + k, offset);
                }
                pos += 20;
            }
        }
        let trailer = String::from_utf8_lossy(&b[pos..]);
        let root = reference(&trailer, "/Root").ok_or_else(|| Fail(format!("trailer has no /Root in {shown}")))?;
        let object = |n: usize| -> Result<String, Fail> {
            let at = *offsets
                .get(&n)
                .ok_or_else(|| Fail(format!("object {n} is not in the xref table of {shown}")))?;
            let from = b
                .get(at..)
                .ok_or_else(|| Fail(format!("object {n} lies beyond the end of {shown}")))?;
            let end = find(from, b"endobj").ok_or_else(|| Fail(format!("object {n} has no endobj in {shown}")))?;
            Ok(String::from_utf8_lossy(&from[..end]).into_owned())
        };
        let catalog = object(root)?;
        let pages_root =
            reference(&catalog, "/Pages").ok_or_else(|| Fail(format!("catalog has no /Pages in {shown}")))?;
        let dests_obj = reference(&catalog, "/Dests").ok_or_else(|| {
            Fail(format!(
                "catalog has no /Dests in {shown} (no internal links were rendered)"
            ))
        })?;

        // Page order: depth-first through the (nested) page tree.
        let mut order = Vec::new();
        let mut stack = vec![pages_root];
        let mut visited = 0;
        while let Some(n) = stack.pop() {
            visited += 1;
            if visited > 1_000_000 {
                return fail(format!("page tree of {shown} does not terminate"));
            }
            let o = object(n)?;
            if o.contains("/Type /Pages") {
                let kids_at = o
                    .find("/Kids [")
                    .ok_or_else(|| Fail(format!("page tree node {n} has no /Kids in {shown}")))?;
                let kids_end = o[kids_at..]
                    .find(']')
                    .ok_or_else(|| Fail(format!("unterminated /Kids in {shown}")))?
                    + kids_at;
                let kids: Vec<usize> = o[kids_at + "/Kids [".len()..kids_end]
                    .split(" R")
                    .filter_map(|r| r.split_whitespace().next().and_then(|d| d.parse().ok()))
                    .collect();
                stack.extend(kids.into_iter().rev());
            } else if o.contains("/Type /Page") {
                order.push(n);
            } else {
                return fail(format!("page tree: object {n} is neither /Pages nor /Page in {shown}"));
            }
        }
        let page_of: HashMap<usize, usize> = order.iter().enumerate().map(|(i, &n)| (n, i + 1)).collect();

        // `/name [P 0 R /XYZ x y z]` entries.
        let d = object(dests_obj)?;
        let open = d
            .find("<<")
            .ok_or_else(|| Fail(format!("/Dests of {shown} is not a dictionary")))?;
        let mut rest = &d[open + 2..];
        let mut dests = HashMap::new();
        loop {
            rest = rest.trim_start();
            if rest.starts_with(">>") || rest.is_empty() {
                break;
            }
            let Some(after_slash) = rest.strip_prefix('/') else {
                return fail(format!("unexpected text in /Dests of {shown}"));
            };
            let name_len = after_slash
                .find(|c: char| c.is_whitespace() || "[]/<>()".contains(c))
                .unwrap_or(after_slash.len());
            let key = name(&after_slash[..name_len]);
            let value = after_slash[name_len..].trim_start();
            let Some(inner) = value.strip_prefix('[') else {
                return fail(format!("destination \"{key}\" in {shown} is not an array"));
            };
            let close = inner
                .find(']')
                .ok_or_else(|| Fail(format!("unterminated destination \"{key}\" in {shown}")))?;
            let target: usize = inner[..close]
                .split_whitespace()
                .next()
                .and_then(|t| t.parse().ok())
                .ok_or_else(|| Fail(format!("destination \"{key}\" in {shown} has no page reference")))?;
            let pg = *page_of.get(&target).ok_or_else(|| {
                Fail(format!(
                    "destination \"{key}\" points at object {target}, which is not a page"
                ))
            })?;
            dests.insert(key, pg);
            rest = &inner[close + 1..];
        }
        Ok(Layout {
            pages: order.len(),
            dests,
        })
    }
}

/// Runs headless Chrome and stops it once the PDF is complete: it does not exit by itself after printing.
mod chrome {
    use crate::{Fail, text};
    use std::io::{BufRead, BufReader, Read};
    use std::path::Path;
    use std::process::{Child, Command, Stdio};
    use std::sync::mpsc::{self, RecvTimeoutError};
    use std::time::{Duration, Instant};

    fn ends_with_eof(pdf: &Path) -> bool {
        let Ok(mut f) = std::fs::File::open(pdf) else {
            return false;
        };
        let mut b = Vec::new();
        if f.read_to_end(&mut b).is_err() {
            return false;
        }
        let trimmed = b.trim_ascii_end();
        trimmed.ends_with(b"%%EOF")
    }

    fn stop(child: &mut Child) {
        let _ = Command::new("kill").args(["-TERM", &child.id().to_string()]).status();
        let until = Instant::now() + Duration::from_secs(5);
        while Instant::now() < until {
            if let Ok(Some(_)) = child.try_wait() {
                return;
            }
            std::thread::sleep(Duration::from_millis(50));
        }
        let _ = child.kill();
        let _ = child.wait();
    }

    /// Prints `html` to `pdf` with a fresh profile; returns the time taken and which completion rule fired.
    pub fn print(
        chrome: &str,
        html: &Path,
        pdf: &Path,
        profile: &Path,
        seconds: u64,
    ) -> Result<(Duration, &'static str), Fail> {
        let start = Instant::now();
        let mut child = Command::new(chrome)
            .args([
                "--headless=new",
                "--disable-gpu",
                "--no-first-run",
                "--no-default-browser-check",
                "--disable-extensions",
                "--no-pdf-header-footer",
                "--virtual-time-budget=2000",
            ])
            .arg(format!("--user-data-dir={}", profile.display()))
            .arg(format!("--print-to-pdf={}", pdf.display()))
            .arg(text::file_url(html))
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| Fail(format!("cannot start Chrome ({chrome}): {e}")))?;
        let stderr = child
            .stderr
            .take()
            .ok_or_else(|| Fail("Chrome's stderr is not captured".to_string()))?;
        let (tx, rx) = mpsc::channel::<String>();
        std::thread::spawn(move || {
            for line in BufReader::new(stderr).lines() {
                let Ok(line) = line else { break };
                if tx.send(line).is_err() {
                    break;
                }
            }
        });
        let expected = pdf.display().to_string();
        let mut tail: std::collections::VecDeque<String> = std::collections::VecDeque::new();
        let (mut last_size, mut unchanged_since) = (0u64, Instant::now());
        let outcome: Result<&'static str, Fail> = loop {
            match rx.recv_timeout(Duration::from_millis(250)) {
                Ok(line) => {
                    // "N bytes written to file <path>"
                    let reported = line
                        .trim()
                        .split_once(" bytes written to file ")
                        .filter(|(_, path)| *path == expected)
                        .and_then(|(n, _)| n.parse::<u64>().ok());
                    if let Some(n) = reported {
                        // The file is normally complete when the line appears; allow a moment for the last write.
                        let until = Instant::now() + Duration::from_secs(2);
                        loop {
                            let size = std::fs::metadata(pdf).map(|m| m.len()).unwrap_or(0);
                            if size == n && ends_with_eof(pdf) {
                                break;
                            }
                            if Instant::now() > until {
                                stop(&mut child);
                                return Err(Fail(format!(
                                    "PDF is incomplete: {} has {size} bytes, Chrome reported {n}, or it lacks %%EOF",
                                    pdf.display()
                                )));
                            }
                            std::thread::sleep(Duration::from_millis(50));
                        }
                        break Ok("Chrome reported the file written");
                    }
                    tail.push_back(line);
                    if tail.len() > 40 {
                        tail.pop_front();
                    }
                }
                Err(RecvTimeoutError::Disconnected) => std::thread::sleep(Duration::from_millis(250)),
                Err(RecvTimeoutError::Timeout) => {}
            }
            // Second rule, for a Chrome that words its message differently: the file ends with %%EOF and its size
            // has not changed for a second.
            let size = std::fs::metadata(pdf).map(|m| m.len()).unwrap_or(0);
            if size != last_size {
                (last_size, unchanged_since) = (size, Instant::now());
            } else if size > 0 && unchanged_since.elapsed() >= Duration::from_secs(1) && ends_with_eof(pdf) {
                break Ok("the file was complete and unchanged for one second");
            }
            if let Ok(Some(status)) = child.try_wait() {
                let lines: Vec<String> = tail.iter().cloned().collect();
                break Err(Fail(format!(
                    "Chrome exited with {status} before writing the PDF; last stderr:\n{}",
                    lines.join("\n")
                )));
            }
            if start.elapsed() > Duration::from_secs(seconds) {
                let lines: Vec<String> = tail.iter().cloned().collect();
                break Err(Fail(format!(
                    "Chrome did not report a written PDF within {seconds} s ({}); last stderr:\n{}",
                    html.display(),
                    lines.join("\n")
                )));
            }
        };
        stop(&mut child);
        outcome.map(|rule| (start.elapsed(), rule))
    }
}

struct Tools {
    pandoc: String,
    chrome: String,
}

fn find_tools() -> Result<(Tools, String, String), Fail> {
    let pandoc = std::env::var("PRINT_PANDOC").unwrap_or_else(|_| "pandoc".to_string());
    let out = std::process::Command::new(&pandoc)
        .arg("--version")
        .output()
        .map_err(|e| {
            Fail(if e.kind() == std::io::ErrorKind::NotFound {
                format!("pandoc not found (looked for \"{pandoc}\"); set PRINT_PANDOC")
            } else {
                format!("cannot run {pandoc}: {e}")
            })
        })?;
    let pandoc_version = String::from_utf8_lossy(&out.stdout)
        .lines()
        .next()
        .unwrap_or("")
        .trim()
        .to_string();
    let chrome = std::env::var("PRINT_CHROME")
        .unwrap_or_else(|_| "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome".to_string());
    if !Path::new(&chrome).is_file() {
        return fail(format!("Chrome not found at \"{chrome}\"; set PRINT_CHROME"));
    }
    let out = std::process::Command::new(&chrome)
        .arg("--version")
        .output()
        .map_err(|e| Fail(format!("cannot run {chrome}: {e}")))?;
    let chrome_version = String::from_utf8_lossy(&out.stdout).trim().to_string();
    Ok((Tools { pandoc, chrome }, pandoc_version, chrome_version))
}

/// pandoc on a post's body, fed through stdin. The reader is GitHub-flavoured Markdown without YAML metadata blocks, so a
/// front matter block that ever reached pandoc would print (and stop the build at the H1 rule) instead of vanishing.
fn pandoc(tools: &Tools, body: &str, src: &Path) -> Result<String, Fail> {
    use std::io::Write as _;
    let mut child = std::process::Command::new(&tools.pandoc)
        .args([
            "-f",
            "gfm-yaml_metadata_block",
            "-t",
            "html5",
            "--wrap=none",
            "--math-method=mathml",
            "--syntax-highlighting=default",
        ])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| Fail(format!("cannot run pandoc on {}: {e}", src.display())))?;
    // pandoc reads all of its input before it writes anything, so writing the whole body first and collecting the output
    // afterwards cannot block on a full pipe.
    let mut stdin = child
        .stdin
        .take()
        .ok_or_else(|| Fail(format!("pandoc's stdin is not captured for {}", src.display())))?;
    let written = stdin.write_all(body.as_bytes());
    drop(stdin);
    let out = child
        .wait_with_output()
        .map_err(|e| Fail(format!("cannot run pandoc on {}: {e}", src.display())))?;
    let stderr = String::from_utf8_lossy(&out.stderr);
    if !out.status.success() {
        return fail(format!("pandoc failed on {} ({}): {stderr}", src.display(), out.status));
    }
    written.map_err(|e| Fail(format!("cannot pass {} to pandoc: {e}", src.display())))?;
    if !stderr.trim().is_empty() {
        return fail(format!("pandoc printed warnings for {}: {stderr}", src.display()));
    }
    String::from_utf8(out.stdout).map_err(|_| Fail(format!("pandoc wrote invalid UTF-8 for {}", src.display())))
}

/// The site root: the nearest folder above the source that holds `_config.yml`.
fn site_root(absolute: &Path) -> Result<PathBuf, Fail> {
    absolute
        .ancestors()
        .skip(1)
        .find(|dir| dir.join("_config.yml").is_file())
        .map(Path::to_path_buf)
        .ok_or_else(|| {
            Fail(format!(
                "no _config.yml above {}, so the site address is unknown",
                absolute.display()
            ))
        })
}

/// The key and the raw value of a YAML line that sets a top-level key: `key: value` from column 0, where YAML ignores
/// spaces before the colon (`key : value` sets `key`). An indented line belongs to a nested mapping and gives nothing.
fn top_level_key(line: &str) -> Option<(&str, &str)> {
    if line.starts_with(char::is_whitespace) {
        return None;
    }
    line.split_once(':').map(|(key, value)| (key.trim_end(), value))
}

/// The value of a top-level `key:` line of `_config.yml`, cut at ` #`, quotes trimmed.
fn config_value(config: &str, key: &str) -> Option<String> {
    config
        .lines()
        .find_map(|l| top_level_key(l).filter(|&(k, _)| k == key).map(|(_, v)| v))
        .map(|v| {
            let v = v.split(" #").next().unwrap_or("").trim();
            v.trim_matches(|c| c == '"' || c == '\'').to_string()
        })
}

/// What the build reads from a post's front matter.
struct Front {
    date: String,
    categories: Vec<String>,
    /// Lines of the block, both `---` lines included.
    lines: usize,
}

/// Splits a post into its front matter and the body after the closing `---`. Keys are `key:` lines at column 0; a key
/// that gives the post another address, a missing date, or categories that are not words on one line stop the build.
fn front_matter<'a>(text: &'a str, name: &str) -> Result<(Front, &'a str), Fail> {
    let Some(block) = text.strip_prefix("---\n") else {
        return fail(format!(
            "{name}: expected the post to open with a front matter block (---)"
        ));
    };
    let mut offset = "---\n".len();
    let (mut date, mut categories) = (None, Vec::new());
    // `lines` is the 1-based line number, the opening `---` being line 1.
    for (lines, line) in (2..).zip(block.split_inclusive('\n')) {
        offset += line.len();
        let line = line.strip_suffix('\n').unwrap_or(line);
        if line == "---" {
            let date = date.ok_or_else(|| Fail(format!("{name}: front matter has no date:")))?;
            return Ok((
                Front {
                    date,
                    categories,
                    lines,
                },
                &text[offset..],
            ));
        }
        let Some((key, value)) = top_level_key(line) else {
            continue;
        };
        match key {
            "permalink" | "slug" | "category" => {
                return fail(format!(
                    "{name}: front matter sets {key}, which the build cannot map to Jekyll's default post address"
                ));
            }
            "categories" => {
                let value = value.trim();
                if value.is_empty() || value.starts_with('[') {
                    return fail(format!(
                        "{name}: categories must be words on one line, as in `categories: rust concepts`"
                    ));
                }
                categories = value.split_whitespace().map(str::to_string).collect();
            }
            "date" => date = Some(value.trim().to_string()),
            _ => {}
        }
    }
    fail(format!("{name}: front matter has no closing ---"))
}

/// Year, month and day of a `YYYY-MM-DD HH:MM:SS +HHMM` date that falls on the same day in UTC, where GitHub Pages builds.
fn same_utc_day<'a>(date: &'a str, name: &str) -> Result<(&'a str, &'a str, &'a str), Fail> {
    let b = date.as_bytes();
    let shape = b.len() == 25
        && [0, 1, 2, 3, 5, 6, 8, 9, 11, 12, 14, 15, 17, 18, 21, 22, 23, 24]
            .iter()
            .all(|&i| b[i].is_ascii_digit())
        && [(4, b'-'), (7, b'-'), (10, b' '), (13, b':'), (16, b':'), (19, b' ')]
            .iter()
            .all(|&(i, c)| b[i] == c)
        && (b[20] == b'+' || b[20] == b'-');
    if !shape {
        return fail(format!("{name}: date \"{date}\" is not YYYY-MM-DD HH:MM:SS +HHMM"));
    }
    let number = |at: usize| i32::from(b[at] - b'0') * 10 + i32::from(b[at + 1] - b'0');
    let sign = if b[20] == b'+' { 1 } else { -1 };
    let minutes = number(11) * 60 + number(14) - sign * (number(21) * 60 + number(23));
    if !(0..1440).contains(&minutes) {
        return fail(format!(
            "{name}: date {date} falls on another day in UTC, where GitHub Pages builds, so the address would carry \
             another day"
        ));
    }
    Ok((&date[0..4], &date[5..7], &date[8..10]))
}

/// The post's published address under Jekyll's default permalink `/:categories/:year/:month/:day/:title.html`, as GitHub
/// Pages builds it (in UTC). Anything that would make Jekyll compute another address stops the build.
fn post_url(site: &Path, absolute: &Path, front: &Front, name: &str) -> Result<String, Fail> {
    let posts = site.join("_posts");
    if absolute.parent() != Some(posts.as_path()) {
        return fail(format!(
            "{name}: not directly under the site's _posts folder ({})",
            posts.display()
        ));
    }
    let stem = absolute
        .file_name()
        .and_then(|f| f.to_str())
        .and_then(|f| f.strip_suffix(".md"))
        .unwrap_or("");
    let b = stem.as_bytes();
    let named = b.len() > 11
        && [0, 1, 2, 3, 5, 6, 8, 9].iter().all(|&i| b[i].is_ascii_digit())
        && [4, 7, 10].iter().all(|&i| b[i] == b'-')
        && b[11..]
            .iter()
            .all(|&c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'-');
    if !named {
        return fail(format!(
            "{name}: file name is not YYYY-MM-DD-slug.md with a slug of [a-z0-9-]"
        ));
    }
    let (year, month, day) = same_utc_day(&front.date, name)?;
    let file_day = &stem[..10];
    if format!("{year}-{month}-{day}") != file_day {
        return fail(format!(
            "{name}: date {} does not fall on the file name's day {file_day}",
            front.date
        ));
    }
    let config =
        std::fs::read_to_string(site.join("_config.yml")).map_err(|e| Fail(format!("cannot read _config.yml: {e}")))?;
    for key in ["permalink", "collections", "timezone", "defaults"] {
        if config_value(&config, key).is_some() {
            return fail(format!(
                "{}/_config.yml sets {key}; the build knows only Jekyll's default post address in UTC",
                site.display()
            ));
        }
    }
    let url = config_value(&config, "url")
        .filter(|u| !u.is_empty())
        .ok_or_else(|| Fail(format!("no url: in {}/_config.yml", site.display())))?;
    let base = config_value(&config, "baseurl").unwrap_or_default();
    // Jekyll lower-cases the categories and keeps each once, in order of first appearance (a set).
    let (mut categories, mut seen) = (String::new(), Vec::new());
    for category in &front.categories {
        let c = category.to_lowercase();
        if !c
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b"._-".contains(&b))
        {
            return fail(format!(
                "{name}: category \"{category}\" has characters the build cannot map"
            ));
        }
        if seen.contains(&c) {
            continue;
        }
        categories.push_str(&c);
        categories.push('/');
        seen.push(c);
    }
    let slug = &stem[11..];
    Ok(format!(
        "{}{}/{categories}{year}/{month}/{day}/{slug}.html",
        url.trim_end_matches('/'),
        base.trim_end_matches('/')
    ))
}

/// The body without the lines equal to `WEB_SCRIPT`, and how many were removed; any other `<script` stops the build.
fn drop_web_script(body: &str, name: &str) -> Result<(String, usize), Fail> {
    let mut removed = 0;
    let kept: Vec<&str> = body
        .split('\n')
        .filter(|line| {
            let web = *line == WEB_SCRIPT;
            removed += usize::from(web);
            !web
        })
        .collect();
    let rest = kept.join("\n");
    if rest.to_ascii_lowercase().contains("<script") {
        return fail(format!(
            "{name}: contains a <script> the build does not know; only the MathJax line {WEB_SCRIPT} is removed"
        ));
    }
    Ok((rest, removed))
}

/// One document: pandoc, reshaping, then passes until the printed numbers and blank pages settle.
fn build(tools: &Tools, css: &str, src: &Path, out_dir: &Path, work: &Path) -> Result<(), Fail> {
    let stem = src
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default();
    println!("{}", src.display());
    let name = src.display().to_string();
    let text = std::fs::read_to_string(src).map_err(|e| Fail(format!("cannot read {}: {e}", src.display())))?;
    let absolute = std::fs::canonicalize(src).map_err(|e| Fail(format!("cannot resolve {}: {e}", src.display())))?;
    let site = site_root(&absolute)?;
    let (front, body) = front_matter(&text, &name)?;
    let (body, removed) = drop_web_script(body, &name)?;
    let url = post_url(&site, &absolute, &front, &name)?;
    let d = doc::prepare(&pandoc(tools, &body, src)?, src, &site)?;
    for line in &d.summary {
        println!("  {line}");
    }
    println!(
        "  post: {} front-matter lines skipped, {removed} web script line(s) removed; Source: {url}",
        front.lines
    );
    let mut numbers: HashMap<String, usize> = HashMap::new();
    let mut blanks: BTreeSet<usize> = BTreeSet::new();
    for pass in 1..=page::MAX_PASSES {
        let html_path = work.join(format!("{stem}-pass{pass}.html"));
        let pdf_path = work.join(format!("{stem}-pass{pass}.pdf"));
        let profile = work.join(format!("chrome-{stem}-pass{pass}"));
        let html = doc::page(&d, css, &url, &numbers, &blanks)?;
        std::fs::write(&html_path, html).map_err(|e| Fail(format!("cannot write {}: {e}", html_path.display())))?;
        let (took, rule) = chrome::print(&tools.chrome, &html_path, &pdf_path, &profile, page::PASS_SECONDS)?;
        let layout = pdf::read(&pdf_path)?;
        let starts: Vec<usize> = (1..=d.divisions.len())
            .map(|n| {
                layout.dests.get(&format!("pd-{n}")).copied().ok_or_else(|| {
                    Fail(format!(
                        "part \"{}\" (pd-{n}) has no destination in {}",
                        d.divisions[n - 1].title,
                        pdf_path.display()
                    ))
                })
            })
            .collect::<Result<_, _>>()?;
        let targets: Vec<usize> = d
            .entries
            .iter()
            .map(|e| {
                layout.dests.get(&e.id).copied().ok_or_else(|| {
                    Fail(format!(
                        "contents entry \"{}\" links to #{}, but the PDF has no destination of that name",
                        e.text, e.id
                    ))
                })
            })
            .collect::<Result<_, _>>()?;
        println!(
            "  pass {pass}: {} pages in {:.1} s ({rule})",
            layout.pages,
            took.as_secs_f64()
        );
        let even_start = d
            .divisions
            .iter()
            .zip(&starts)
            .position(|(div, &p)| div.recto && p % 2 == 0);
        let wrong_number = d
            .entries
            .iter()
            .zip(&targets)
            .position(|(e, &p)| numbers.get(&e.id) != Some(&p));
        if pass > 1 && even_start.is_none() && wrong_number.is_none() {
            let blank_pages: Vec<String> = blanks.iter().map(|&n| (starts[n - 1] - 1).to_string()).collect();
            let target = out_dir.join(format!("{stem}.pdf"));
            std::fs::copy(&pdf_path, &target).map_err(|e| Fail(format!("cannot write {}: {e}", target.display())))?;
            let bytes = std::fs::metadata(&target).map(|m| m.len()).unwrap_or(0);
            println!(
                "  settled: {} pages ({} sheets double-sided), blank pages {}, {} contents numbers match their headings",
                layout.pages,
                layout.pages.div_ceil(2),
                if blank_pages.is_empty() {
                    "none".to_string()
                } else {
                    blank_pages.join(", ")
                },
                d.entries.len()
            );
            println!("  wrote {} ({bytes} bytes)", target.display());
            return Ok(());
        }
        if pass == page::MAX_PASSES {
            if let Some(i) = wrong_number {
                return fail(format!(
                    "page numbers did not settle after {pass} passes: entry \"{}\" prints {} but its heading is on page {}",
                    d.entries[i].text,
                    numbers
                        .get(&d.entries[i].id)
                        .map(|n| n.to_string())
                        .unwrap_or_else(|| "nothing".to_string()),
                    targets[i]
                ));
            }
            if let Some(n) = even_start {
                return fail(format!(
                    "part \"{}\" still starts on even page {} after {pass} passes",
                    d.divisions[n].title, starts[n]
                ));
            }
        }
        // Plan the next pass. Blanks are only ever added: a blank page and a filled number slot move later pages
        // on without reflowing anything, so every page after an added blank is simply one higher.
        let mut added = Vec::new();
        for (n, division) in d.divisions.iter().enumerate() {
            let now = starts[n] + added.len();
            if division.recto && now % 2 == 0 && !blanks.contains(&(n + 1)) {
                blanks.insert(n + 1);
                added.push(starts[n]);
            }
        }
        numbers = d
            .entries
            .iter()
            .zip(&targets)
            .map(|(e, &p)| (e.id.clone(), p + added.iter().filter(|&&a| a <= p).count()))
            .collect();
    }
    fail(format!("{}: no pass was attempted", src.display()))
}

fn run() -> Result<(), Fail> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.len() < 2 {
        return fail("usage: make_print <out-dir> <_posts/YYYY-MM-DD-slug.md>...");
    }
    let out_dir = PathBuf::from(&args[0]);
    let css_path = out_dir.join("print.css");
    let css =
        std::fs::read_to_string(&css_path).map_err(|e| Fail(format!("no print.css in {}: {e}", out_dir.display())))?;
    let sources: Vec<PathBuf> = args[1..].iter().map(PathBuf::from).collect();
    let mut stems = HashSet::new();
    for s in &sources {
        if !s.is_file() {
            return fail(format!("source {} not found", s.display()));
        }
        if !stems.insert(s.file_stem().map(|x| x.to_os_string())) {
            return fail(format!(
                "two sources would both print to {}.pdf",
                s.file_stem().unwrap_or_default().to_string_lossy()
            ));
        }
    }
    let (tools, pandoc_version, chrome_version) = find_tools()?;
    println!("{pandoc_version}; {chrome_version}");
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let work = std::env::temp_dir().join(format!("make_print-{}-{nanos}", std::process::id()));
    std::fs::create_dir_all(&work)
        .map_err(|e| Fail(format!("cannot create temporary directory {}: {e}", work.display())))?;
    let mut result = Ok(());
    for src in &sources {
        if let Err(e) = build(&tools, &css, src, &out_dir, &work) {
            result = Err(e);
            break;
        }
    }
    let keep = std::env::var("PRINT_KEEP_TMP").is_ok_and(|v| v == "1");
    if result.is_ok() && !keep {
        std::fs::remove_dir_all(&work).map_err(|e| Fail(format!("cannot remove {}: {e}", work.display())))?;
    } else {
        eprintln!("make_print: working files kept in {}", work.display());
    }
    result
}

fn main() -> std::process::ExitCode {
    match run() {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(Fail(message)) => {
            eprintln!("make_print: {message}");
            std::process::ExitCode::FAILURE
        }
    }
}
