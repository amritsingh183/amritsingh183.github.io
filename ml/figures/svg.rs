//! A minimal SVG builder and the colour and font tokens shared by the blog's figure generators
//! (`ml/figures/make_figures.rs` and `ml/book/make_book_figures.rs`): native shapes and text only, explicit colours,
//! one arrow marker. Standard library only. Each generator includes this file with `mod svg;` (the book generator
//! through `#[path]`) and reaches the tokens as `svg::tok`.

/// Colour and font tokens: the only colour literals in this program. Validated as a set (see the documents'
/// verification notes): categorical slots in fixed order, one-hue blue ramp for magnitude, ink for text.
pub mod tok {
    pub const SURFACE: &str = "#fcfcfb";
    pub const INK: &str = "#0b0b0b";
    pub const INK2: &str = "#52514e";
    pub const MUTED: &str = "#898781";
    pub const GRID: &str = "#e1e0d9";
    pub const AXIS: &str = "#c3c2b7";
    pub const NEUTRAL: &str = "#f0efec";
    pub const S1: &str = "#2a78d6";
    pub const S2: &str = "#eb6834";
    pub const S3: &str = "#1baf7a";
    pub const RAMP: [&str; 13] = [
        "#cde2fb", "#b7d3f6", "#9ec5f4", "#86b6ef", "#6da7ec", "#5598e7", "#3987e5", "#2a78d6", "#256abf",
        "#1c5cab", "#184f95", "#104281", "#0d366b",
    ];
    /// Light fills that keep ink text readable (ramp steps 100, 150, 250).
    pub const FILL1: &str = RAMP[0];
    pub const FILL2: &str = RAMP[1];
    pub const FILL3: &str = RAMP[3];
    pub const FONT: &str = "system-ui, -apple-system, 'Segoe UI', Helvetica, Arial, sans-serif";
}

pub enum Anchor {
    Start,
    Middle,
    End,
}

pub struct Svg {
    w: u32,
    h: u32,
    title: String,
    desc: String,
    body: String,
    arrow_used: bool,
    /// Smallest text size this figure prints; 0 leaves every requested size as it is.
    min_text: u8,
}

/// Formats a coordinate with at most one decimal.
pub fn c(v: f64) -> String {
    if (v - v.round()).abs() < 1e-9 {
        format!("{}", v.round() as i64)
    } else {
        format!("{v:.1}")
    }
}

/// Escapes text for XML content and attribute values.
pub fn esc(s: &str) -> String {
    let mut o = String::with_capacity(s.len() + 8);
    for ch in s.chars() {
        match ch {
            '&' => o.push_str("&amp;"),
            '<' => o.push_str("&lt;"),
            '>' => o.push_str("&gt;"),
            '"' => o.push_str("&quot;"),
            '\'' => o.push_str("&apos;"),
            _ => o.push(ch),
        }
    }
    o
}

/// Formats a number with a fixed number of decimals, a typographic minus, no negative zero, and ∞ / NaN.
pub fn num(v: f64, decimals: usize) -> String {
    if v.is_nan() {
        return "NaN".to_string();
    }
    if v.is_infinite() {
        return if v > 0.0 { "∞".to_string() } else { "−∞".to_string() };
    }
    let s = format!("{v:.decimals$}");
    let s = match s.strip_prefix('-') {
        Some(rest) if rest.chars().all(|ch| ch == '0' || ch == '.') => rest.to_string(),
        _ => s,
    };
    s.replace('-', "−")
}

/// Formats a number with up to `decimals` decimals, dropping trailing zeros (2.50 → 2.5, 4.00 → 4).
pub fn numt(v: f64, decimals: usize) -> String {
    let s = num(v, decimals);
    if s.contains('.') {
        let t = s.trim_end_matches('0').trim_end_matches('.');
        if t.is_empty() || t == "−" { "0".to_string() } else { t.to_string() }
    } else {
        s
    }
}

/// Formats a non-negative integer with thousands separators (8320 → "8,320").
pub fn thousands(n: u64) -> String {
    let digits = n.to_string();
    let mut out = String::new();
    for (i, ch) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i) % 3 == 0 {
            out.push(',');
        }
        out.push(ch);
    }
    out
}

/// Maps a magnitude in `0..=max` to a light-to-mid ramp step (100…450), so ink text stays readable.
pub fn heat(v: f64, max: f64) -> &'static str {
    let t = if max > 0.0 { (v / max).clamp(0.0, 1.0) } else { 0.0 };
    let i = (t * 7.0).round() as usize;
    tok::RAMP[i.min(7)]
}

#[derive(Clone)]
pub struct Cell {
    pub text: String,
    pub fill: &'static str,
    pub stroke: &'static str,
    pub ink: &'static str,
}

/// A plain cell: surface fill, grid border, ink text.
pub fn cell(text: impl Into<String>) -> Cell {
    Cell { text: text.into(), fill: tok::SURFACE, stroke: tok::GRID, ink: tok::INK }
}

impl Cell {
    pub fn fill(mut self, f: &'static str) -> Self {
        self.fill = f;
        self
    }
    pub fn stroke(mut self, s: &'static str) -> Self {
        self.stroke = s;
        self
    }
    pub fn ink(mut self, i: &'static str) -> Self {
        self.ink = i;
        self
    }
}

/// Estimated text width at a font size (checked visually; used to size boxes around labels).
pub fn text_w(s: &str, size: u8) -> f64 {
    0.56 * f64::from(size) * s.chars().count() as f64
}

impl Svg {
    pub fn new(w: u32, h: u32, title: &str, desc: &str) -> Svg {
        Svg { w, h, title: title.to_string(), desc: desc.to_string(), body: String::new(), arrow_used: false, min_text: 0 }
    }

    /// Raises every text size below `size` to `size`, helpers included. The book's figures use 12: their 720-unit
    /// canvas prints 488 pt wide, so 12 units is 8.1 pt on paper and 11 would be 7.5 pt.
    #[allow(dead_code)] // used by the book generator only; make_figures.rs keeps every size as requested
    pub fn min_text(mut self, size: u8) -> Svg {
        self.min_text = size;
        self
    }

    pub fn rect(&mut self, x: f64, y: f64, w: f64, h: f64, fill: &str, stroke: Option<&str>) {
        let st = match stroke {
            Some(s) => format!(" stroke=\"{s}\" stroke-width=\"1\""),
            None => String::new(),
        };
        self.body.push_str(&format!(
            "<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" rx=\"2\" fill=\"{fill}\"{st}/>\n",
            c(x), c(y), c(w), c(h)
        ));
    }

    /// A box outlined at 1.5px, used where the outline carries meaning (a highlighted entity).
    pub fn rect_bold(&mut self, x: f64, y: f64, w: f64, h: f64, fill: &str, stroke: &str) {
        self.body.push_str(&format!(
            "<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" rx=\"2\" fill=\"{fill}\" stroke=\"{stroke}\" stroke-width=\"1.5\"/>\n",
            c(x), c(y), c(w), c(h)
        ));
    }

    pub fn rect_dashed(&mut self, x: f64, y: f64, w: f64, h: f64, stroke: &str) {
        self.body.push_str(&format!(
            "<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" rx=\"2\" fill=\"none\" stroke=\"{stroke}\" stroke-width=\"1.5\" stroke-dasharray=\"4 2\"/>\n",
            c(x), c(y), c(w), c(h)
        ));
    }

    fn text_attrs(size: u8, fill: &str, anchor: &Anchor) -> String {
        let mut attrs = String::new();
        if size != 12 {
            attrs.push_str(&format!(" font-size=\"{size}\""));
        }
        if fill != tok::INK {
            attrs.push_str(&format!(" fill=\"{fill}\""));
        }
        match anchor {
            Anchor::Middle => attrs.push_str(" text-anchor=\"middle\""),
            Anchor::End => attrs.push_str(" text-anchor=\"end\""),
            Anchor::Start => {}
        }
        attrs
    }

    pub fn text(&mut self, x: f64, y: f64, s: &str, size: u8, fill: &str, anchor: Anchor) {
        let attrs = Self::text_attrs(size.max(self.min_text), fill, &anchor);
        self.body.push_str(&format!("<text x=\"{}\" y=\"{}\"{attrs}>{}</text>\n", c(x), c(y), esc(s)));
    }

    pub fn text_bold(&mut self, x: f64, y: f64, s: &str, size: u8, fill: &str, anchor: Anchor) {
        let attrs = Self::text_attrs(size.max(self.min_text), fill, &anchor);
        self.body.push_str(&format!(
            "<text x=\"{}\" y=\"{}\"{attrs} font-weight=\"600\">{}</text>\n",
            c(x), c(y), esc(s)
        ));
    }

    pub fn line(&mut self, x1: f64, y1: f64, x2: f64, y2: f64, stroke: &str, width: f64) {
        self.body.push_str(&format!(
            "<line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" stroke=\"{stroke}\" stroke-width=\"{}\"/>\n",
            c(x1), c(y1), c(x2), c(y2), c(width)
        ));
    }

    /// An arrow in annotation ink with an optional 11px label centred above its midpoint.
    pub fn arrow(&mut self, x1: f64, y1: f64, x2: f64, y2: f64, label: Option<&str>) {
        self.arrow_used = true;
        self.body.push_str(&format!(
            "<line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" stroke=\"{}\" stroke-width=\"1.5\" marker-end=\"url(#arrow)\"/>\n",
            c(x1), c(y1), c(x2), c(y2), tok::INK2
        ));
        if let Some(l) = label {
            self.text((x1 + x2) / 2.0, (y1 + y2) / 2.0 - 5.0, l, 11, tok::INK2, Anchor::Middle);
        }
    }

    pub fn path(&mut self, d: &str, stroke: &str, width: f64) {
        self.body.push_str(&format!(
            "<path d=\"{d}\" fill=\"none\" stroke=\"{stroke}\" stroke-width=\"{}\"/>\n",
            c(width)
        ));
    }

    /// A curved skip connection from (x1, y1) to (x2, y2) bulging by `bulge` to the right (positive) or left.
    pub fn skip(&mut self, x1: f64, y1: f64, x2: f64, y2: f64, bulge: f64, label: Option<&str>) {
        self.arrow_used = true;
        let cx = x1 + bulge;
        self.body.push_str(&format!(
            "<path d=\"M{} {} C{} {} {} {} {} {}\" fill=\"none\" stroke=\"{}\" stroke-width=\"1.5\" marker-end=\"url(#arrow)\"/>\n",
            c(x1), c(y1), c(cx), c(y1), c(cx), c(y2), c(x2), c(y2), tok::INK2
        ));
        if let Some(l) = label {
            let dir = if bulge >= 0.0 { 1.0 } else { -1.0 };
            self.text(x1 + bulge * 0.75 + dir * 4.0, (y1 + y2) / 2.0 + 4.0, l, 11, tok::INK2, if bulge >= 0.0 { Anchor::Start } else { Anchor::End });
        }
    }

    pub fn dot(&mut self, cx: f64, cy: f64, r: f64, fill: &str, stroke: Option<&str>) {
        let st = match stroke {
            Some(s) => format!(" stroke=\"{s}\" stroke-width=\"1.5\""),
            None => String::new(),
        };
        self.body.push_str(&format!(
            "<circle cx=\"{}\" cy=\"{}\" r=\"{}\" fill=\"{fill}\"{st}/>\n",
            c(cx), c(cy), c(r)
        ));
    }

    /// A grid of cells; text is centred in each cell (12px for cells of 28px or more, otherwise 11px).
    pub fn cells(&mut self, x: f64, y: f64, cell: f64, rows: &[Vec<Cell>]) {
        let size: u8 = if cell >= 28.0 { 12 } else { 11 };
        for (r, row) in rows.iter().enumerate() {
            for (j, cl) in row.iter().enumerate() {
                let cx = x + j as f64 * cell;
                let cy = y + r as f64 * cell;
                self.rect(cx, cy, cell, cell, cl.fill, Some(cl.stroke));
                if !cl.text.is_empty() {
                    self.text(cx + cell / 2.0, cy + cell / 2.0 + 4.0, &cl.text, size, cl.ink, Anchor::Middle);
                }
            }
        }
    }

    /// Cells that are `cw` wide and `ch` high (for value strips whose entries are wider than tall).
    pub fn cells_wh(&mut self, x: f64, y: f64, cw: f64, ch: f64, rows: &[Vec<Cell>]) {
        let size: u8 = if ch >= 26.0 { 12 } else { 11 };
        for (r, row) in rows.iter().enumerate() {
            for (j, cl) in row.iter().enumerate() {
                let cx = x + j as f64 * cw;
                let cy = y + r as f64 * ch;
                self.rect(cx, cy, cw, ch, cl.fill, Some(cl.stroke));
                if !cl.text.is_empty() {
                    self.text(cx + cw / 2.0, cy + ch / 2.0 + 4.0, &cl.text, size, cl.ink, Anchor::Middle);
                }
            }
        }
    }

    /// A bracket to the right of a span with a label beside it.
    pub fn brace_right(&mut self, x: f64, y1: f64, y2: f64, label: &str) {
        let d = format!("M{} {} h6 V{} h-6", c(x), c(y1), c(y2));
        self.path(&d, tok::INK2, 1.0);
        self.text(x + 10.0, (y1 + y2) / 2.0 + 4.0, label, 11, tok::INK2, Anchor::Start);
    }

    /// A rounded box with a centred one-line label.
    pub fn labelled_box(&mut self, x: f64, y: f64, w: f64, h: f64, label: &str, fill: &str, stroke: &str) {
        self.rect(x, y, w, h, fill, Some(stroke));
        self.text(x + w / 2.0, y + h / 2.0 + 4.0, label, 12, tok::INK, Anchor::Middle);
    }

    /// A rounded box with two centred lines (a name and a detail line in annotation ink).
    pub fn labelled_box2(&mut self, x: f64, y: f64, w: f64, h: f64, l1: &str, l2: &str, fill: &str, stroke: &str) {
        self.rect(x, y, w, h, fill, Some(stroke));
        self.text(x + w / 2.0, y + h / 2.0 - 3.0, l1, 12, tok::INK, Anchor::Middle);
        self.text(x + w / 2.0, y + h / 2.0 + 12.0, l2, 11, tok::INK2, Anchor::Middle);
    }

    pub fn finish(self) -> String {
        let mut out = String::with_capacity(self.body.len() + 600);
        out.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
        out.push_str(&format!(
            "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 {w} {h}\" width=\"{w}\" height=\"{h}\" role=\"img\" aria-labelledby=\"title desc\">\n",
            w = self.w,
            h = self.h
        ));
        out.push_str(&format!("<title id=\"title\">{}</title>\n", esc(&self.title)));
        out.push_str(&format!("<desc id=\"desc\">{}</desc>\n", esc(&self.desc)));
        if self.arrow_used {
            out.push_str(&format!(
                "<defs><marker id=\"arrow\" viewBox=\"0 0 10 10\" refX=\"9\" refY=\"5\" markerWidth=\"7\" markerHeight=\"7\" orient=\"auto-start-reverse\"><path d=\"M0,0 L10,5 L0,10 z\" fill=\"{}\"/></marker></defs>\n",
                tok::INK2
            ));
        }
        out.push_str(&format!(
            "<rect width=\"{}\" height=\"{}\" fill=\"{}\"/>\n",
            self.w,
            self.h,
            tok::SURFACE
        ));
        out.push_str(&format!(
            "<g font-family=\"{}\" font-size=\"12\" fill=\"{}\">\n",
            tok::FONT,
            tok::INK
        ));
        out.push_str(&self.body);
        out.push_str("</g>\n</svg>\n");
        out
    }
}
