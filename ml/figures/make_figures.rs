//! Draws the concept figures of the two ML posts (`_posts/2026-09-28-rust-candle-ml-guide.md` and
//! `_posts/2026-09-28-rust-ml-handbook.md`) as SVG files. Standard library only.
//!
//! Compile:  rustc --edition 2024 -O -o /tmp/make_figures make_figures.rs
//! Run:      /tmp/make_figures ml/figures        (the argument is the output folder)
//!
//! Every number drawn is computed in `calc` from the same inputs the documents use (softmax, attention,
//! positional encodings, receptive fields, dilation offsets, transposed-convolution stamp counts,
//! convolution output sizes and window positions, normalisation statistics, the BatchNorm fold, the backward examples). Two runs write byte-identical files.
#![forbid(unsafe_code)]

/// Colour and font tokens: the only colour literals in this program. Validated as a set (see the documents'
/// verification notes): categorical slots in fixed order, one-hue blue ramp for magnitude, ink for text.
mod tok {
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

/// A minimal SVG builder: native shapes and text only, explicit colours, one arrow marker.
mod svg {
    use crate::tok;

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
            Svg { w, h, title: title.to_string(), desc: desc.to_string(), body: String::new(), arrow_used: false }
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
            let attrs = Self::text_attrs(size, fill, &anchor);
            self.body.push_str(&format!("<text x=\"{}\" y=\"{}\"{attrs}>{}</text>\n", c(x), c(y), esc(s)));
        }

        pub fn text_bold(&mut self, x: f64, y: f64, s: &str, size: u8, fill: &str, anchor: Anchor) {
            let attrs = Self::text_attrs(size, fill, &anchor);
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
}

/// The arithmetic behind every drawn number. Pure functions on f64; no result is typed as a literal elsewhere.
mod calc {
    pub fn softmax(z: &[f64]) -> Vec<f64> {
        let m = z.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        let exps: Vec<f64> = z.iter().map(|&v| (v - m).exp()).collect();
        let sum: f64 = exps.iter().sum();
        exps.iter().map(|e| e / sum).collect()
    }

    /// The textbook formula: overflows to ∞ and then NaN exactly as floating point does.
    pub fn naive_softmax(z: &[f64]) -> (Vec<f64>, f64, Vec<f64>) {
        let exps: Vec<f64> = z.iter().map(|v| v.exp()).collect();
        let sum: f64 = exps.iter().sum();
        let probs = exps.iter().map(|e| e / sum).collect();
        (exps, sum, probs)
    }

    /// The shifted stages: the maximum, the shifted logits, their exponentials, the sum, the probabilities.
    pub fn shifted_softmax(z: &[f64]) -> (f64, Vec<f64>, Vec<f64>, f64, Vec<f64>) {
        let m = z.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        let shifted: Vec<f64> = z.iter().map(|v| v - m).collect();
        let exps: Vec<f64> = shifted.iter().map(|v| v.exp()).collect();
        let sum: f64 = exps.iter().sum();
        let probs = exps.iter().map(|e| e / sum).collect();
        (m, shifted, exps, sum, probs)
    }

    pub fn matmul(a: &[Vec<f64>], b: &[Vec<f64>]) -> Vec<Vec<f64>> {
        let n = b[0].len();
        a.iter()
            .map(|row| (0..n).map(|j| row.iter().zip(b).map(|(x, brow)| x * brow[j]).sum()).collect())
            .collect()
    }

    pub fn transpose(a: &[Vec<f64>]) -> Vec<Vec<f64>> {
        (0..a[0].len()).map(|j| a.iter().map(|row| row[j]).collect()).collect()
    }

    pub struct Attn {
        pub q: Vec<Vec<f64>>,
        pub k: Vec<Vec<f64>>,
        pub v: Vec<Vec<f64>>,
        pub scores: Vec<Vec<f64>>,
        pub scaled: Vec<Vec<f64>>,
        pub weights: Vec<Vec<f64>>,
        pub out: Vec<Vec<f64>>,
    }

    /// Scaled dot-product attention with projections P (features in rows): Q = X·P_Q etc.; `mask[i][j]` true = forbidden.
    pub fn attention(
        x: &[Vec<f64>],
        p_q: &[Vec<f64>],
        p_k: &[Vec<f64>],
        p_v: &[Vec<f64>],
        mask: Option<&[Vec<bool>]>,
    ) -> Attn {
        let q = matmul(x, p_q);
        let k = matmul(x, p_k);
        let v = matmul(x, p_v);
        let d_k = q[0].len() as f64;
        let scores = matmul(&q, &transpose(&k));
        let scaled: Vec<Vec<f64>> = scores
            .iter()
            .enumerate()
            .map(|(i, row)| {
                row.iter()
                    .enumerate()
                    .map(|(j, s)| {
                        let forbidden = mask.is_some_and(|m| m[i][j]);
                        if forbidden { f64::NEG_INFINITY } else { s / d_k.sqrt() }
                    })
                    .collect()
            })
            .collect();
        let weights: Vec<Vec<f64>> = scaled.iter().map(|row| softmax(row)).collect();
        let out = matmul(&weights, &v);
        Attn { q, k, v, scores, scaled, weights, out }
    }

    /// The sinusoidal table of the Transformer paper: PE(pos, 2i) = sin(pos / 10000^(2i/d)), PE(pos, 2i+1) = cos(…).
    pub fn positional(seq: usize, d_model: usize) -> Vec<Vec<f64>> {
        (0..seq)
            .map(|pos| {
                (0..d_model)
                    .map(|dim| {
                        let i = dim / 2;
                        let angle = pos as f64 / 10000f64.powf(2.0 * i as f64 / d_model as f64);
                        if dim % 2 == 0 { angle.sin() } else { angle.cos() }
                    })
                    .collect()
            })
            .collect()
    }

    /// Receptive field after each layer for (kernel, dilation, stride) triples: rf += (k−1)·d·jump; jump *= s.
    pub fn receptive_fields(layers: &[(usize, usize, usize)]) -> Vec<usize> {
        let (mut rf, mut jump) = (1usize, 1usize);
        layers
            .iter()
            .map(|&(k, d, s)| {
                rf += (k - 1) * d * jump;
                jump *= s;
                rf
            })
            .collect()
    }

    /// The offsets one output reads after each successive 3-tap layer with the given dilations (offset sets add).
    pub fn offset_rows(dilations: &[i64]) -> Vec<Vec<i64>> {
        let mut rows = Vec::new();
        let mut set: Vec<i64> = vec![0];
        for &d in dilations {
            let mut next: Vec<i64> = Vec::new();
            for &o in &set {
                for step in [-d, 0, d] {
                    next.push(o + step);
                }
            }
            next.sort_unstable();
            next.dedup();
            set = next;
            rows.push(set.clone());
        }
        rows
    }

    /// Stamp counts of a transposed 1-D convolution: `n` inputs, kernel width `k`, stride `s`, all-ones kernel.
    pub fn tconv_counts(n: usize, k: usize, s: usize) -> Vec<usize> {
        let len = (n - 1) * s + k;
        let mut counts = vec![0usize; len];
        for i in 0..n {
            for j in 0..k {
                counts[i * s + j] += 1;
            }
        }
        counts
    }

    /// Valid cross-correlation (no flip, no padding, stride 1) of a 2-D input with a 2-D kernel.
    pub fn conv_valid(input: &[Vec<f64>], kernel: &[Vec<f64>]) -> Vec<Vec<f64>> {
        let (h, w) = (input.len(), input[0].len());
        let (kh, kw) = (kernel.len(), kernel[0].len());
        (0..=h - kh)
            .map(|oy| {
                (0..=w - kw)
                    .map(|ox| {
                        let mut acc = 0.0;
                        for (ky, krow) in kernel.iter().enumerate() {
                            for (kx, kv) in krow.iter().enumerate() {
                                acc += input[oy + ky][ox + kx] * kv;
                            }
                        }
                        acc
                    })
                    .collect()
            })
            .collect()
    }

    /// The (kernel weight, input value) pairs one output multiplies, in kernel order.
    pub fn window_terms(input: &[Vec<f64>], kernel: &[Vec<f64>], oy: usize, ox: usize) -> Vec<(f64, f64)> {
        let mut terms = Vec::new();
        for (ky, krow) in kernel.iter().enumerate() {
            for (kx, kv) in krow.iter().enumerate() {
                terms.push((*kv, input[oy + ky][ox + kx]));
            }
        }
        terms
    }

    /// Row of the convolution matrix for output (oy, ox) over an h×w input read row by row.
    pub fn conv_matrix_row(kernel: &[Vec<f64>], h: usize, w: usize, oy: usize, ox: usize) -> Vec<f64> {
        let mut row = vec![0.0; h * w];
        for (ky, krow) in kernel.iter().enumerate() {
            for (kx, kv) in krow.iter().enumerate() {
                row[(oy + ky) * w + (ox + kx)] = *kv;
            }
        }
        row
    }

    /// The full (flipped, textbook) 1-D convolution matrix of the guide's section 7.2: c[r][j] = h[r − j].
    pub fn toeplitz_full(x_len: usize, h: &[f64]) -> Vec<Vec<f64>> {
        let rows = x_len + h.len() - 1;
        (0..rows)
            .map(|r| (0..x_len).map(|j| if r >= j && r - j < h.len() { h[r - j] } else { 0.0 }).collect())
            .collect()
    }

    /// The valid, unflipped 1-D convolution matrix (the handbook's section 6.4): row r holds the kernel at columns r….
    pub fn conv_matrix_1d(x_len: usize, kernel: &[f64]) -> Vec<Vec<f64>> {
        let rows = x_len + 1 - kernel.len();
        (0..rows)
            .map(|r| (0..x_len).map(|j| if j >= r && j - r < kernel.len() { kernel[j - r] } else { 0.0 }).collect())
            .collect()
    }

    pub fn matvec(a: &[Vec<f64>], x: &[f64]) -> Vec<f64> {
        a.iter().map(|row| row.iter().zip(x).map(|(p, q)| p * q).sum()).collect()
    }

    pub fn dot(a: &[f64], b: &[f64]) -> f64 {
        a.iter().zip(b).map(|(p, q)| p * q).sum()
    }

    /// Folds a fixed BatchNorm into the scalar map w·x + b: returns (α, w', b').
    pub fn bn_fold(w: f64, b: f64, mu: f64, var: f64, eps: f64, gamma: f64, beta: f64) -> (f64, f64, f64) {
        let alpha = gamma / (var + eps).sqrt();
        (alpha, alpha * w, alpha * (b - mu) + beta)
    }

    pub fn sigmoid(x: f64) -> f64 {
        1.0 / (1.0 + (-x).exp())
    }

    pub fn relu(x: f64) -> f64 {
        x.max(0.0)
    }

    pub struct SeExample {
        pub maps: [Vec<Vec<f64>>; 2],
        pub averages: [f64; 2],
        pub hidden: f64,
        pub logits: [f64; 2],
        pub gates: [f64; 2],
        pub scaled: [Vec<Vec<f64>>; 2],
    }

    /// The handbook's two-channel SE example: maps [[1,3],[1,3]] and [[0,2],[0,2]], hidden weights [1, −1], second-layer weights ±ln 3.
    pub fn se_two_channel() -> SeExample {
        let maps = [vec![vec![1.0, 3.0], vec![1.0, 3.0]], vec![vec![0.0, 2.0], vec![0.0, 2.0]]];
        let avg = |m: &Vec<Vec<f64>>| m.iter().flatten().sum::<f64>() / m.iter().flatten().count() as f64;
        let averages = [avg(&maps[0]), avg(&maps[1])];
        let hidden = relu(1.0 * averages[0] + (-1.0) * averages[1]);
        let ln3 = 3f64.ln();
        let logits = [ln3 * hidden, -ln3 * hidden];
        let gates = [sigmoid(logits[0]), sigmoid(logits[1])];
        let scale = |m: &Vec<Vec<f64>>, g: f64| m.iter().map(|row| row.iter().map(|v| v * g).collect()).collect();
        let scaled = [scale(&maps[0], gates[0]), scale(&maps[1], gates[1])];
        SeExample { maps, averages, hidden, logits, gates, scaled }
    }

    pub struct Step {
        pub output: f64,
        pub loss: f64,
        pub delta: f64,
        pub grad: Vec<f64>,
        pub new_w: Vec<f64>,
        pub new_output: f64,
    }

    /// The guide's section 4.6: weight [1, 1], input [1, 2], target 1, squared error, SGD with rate 0.1.
    pub fn one_layer_step() -> Step {
        let w = vec![1.0, 1.0];
        let x = vec![1.0, 2.0];
        let target = 1.0;
        let lr = 0.1;
        let output = dot(&w, &x);
        let loss = (output - target) * (output - target);
        let delta = 2.0 * (output - target);
        let grad: Vec<f64> = x.iter().map(|xi| delta * xi).collect();
        let new_w: Vec<f64> = w.iter().zip(&grad).map(|(wi, g)| wi - lr * g).collect();
        let new_output = dot(&new_w, &x);
        Step { output, loss, delta, grad, new_w, new_output }
    }

    pub struct Backprop {
        pub x: Vec<f64>,
        pub z1: Vec<f64>,
        pub h: Vec<f64>,
        pub y_hat: f64,
        pub loss: f64,
        pub d_yhat: f64,
        pub d_w2: Vec<f64>,
        pub d_b2: f64,
        pub d_h: Vec<f64>,
        pub d_z1: Vec<f64>,
        pub d_w1: Vec<Vec<f64>>,
        pub d_b1: Vec<f64>,
        pub w1_new: Vec<Vec<f64>>,
        pub b1_new: Vec<f64>,
        pub w2_new: Vec<f64>,
        pub b2_new: f64,
        pub h_new: Vec<f64>,
        pub y_hat_new: f64,
        pub loss_new: f64,
    }

    /// The handbook's section 4.4: x = [1, 2], W1 = I, b1 = 0, w2 = [1, −1], b2 = 0, target 0, L = ½(ŷ − y)², rate 0.1.
    pub fn two_layer_backward() -> Backprop {
        let x = vec![1.0, 2.0];
        let w1 = vec![vec![1.0, 0.0], vec![0.0, 1.0]];
        let b1 = vec![0.0, 0.0];
        let w2 = vec![1.0, -1.0];
        let b2 = 0.0;
        let y = 0.0;
        let lr = 0.1;
        let z1: Vec<f64> = w1.iter().zip(&b1).map(|(row, b)| dot(row, &x) + b).collect();
        let h: Vec<f64> = z1.iter().map(|v| relu(*v)).collect();
        let y_hat = dot(&w2, &h) + b2;
        let loss = 0.5 * (y_hat - y) * (y_hat - y);
        let d_yhat = y_hat - y;
        let d_w2: Vec<f64> = h.iter().map(|hj| d_yhat * hj).collect();
        let d_b2 = d_yhat;
        let d_h: Vec<f64> = w2.iter().map(|wj| d_yhat * wj).collect();
        let d_z1: Vec<f64> = d_h.iter().zip(&z1).map(|(g, z)| if *z > 0.0 { *g } else { 0.0 }).collect();
        let d_w1: Vec<Vec<f64>> = d_z1.iter().map(|g| x.iter().map(|xi| g * xi).collect()).collect();
        let d_b1 = d_z1.clone();
        let w1_new: Vec<Vec<f64>> = w1
            .iter()
            .zip(&d_w1)
            .map(|(row, grow)| row.iter().zip(grow).map(|(wv, g)| wv - lr * g).collect())
            .collect();
        let b1_new: Vec<f64> = b1.iter().zip(&d_b1).map(|(b, g)| b - lr * g).collect();
        let w2_new: Vec<f64> = w2.iter().zip(&d_w2).map(|(wv, g)| wv - lr * g).collect();
        let b2_new = b2 - lr * d_b2;
        let z1_new: Vec<f64> = w1_new.iter().zip(&b1_new).map(|(row, b)| dot(row, &x) + b).collect();
        let h_new: Vec<f64> = z1_new.iter().map(|v| relu(*v)).collect();
        let y_hat_new = dot(&w2_new, &h_new) + b2_new;
        let loss_new = 0.5 * (y_hat_new - y) * (y_hat_new - y);
        Backprop {
            x, z1, h, y_hat, loss, d_yhat, d_w2, d_b2, d_h, d_z1, d_w1, d_b1, w1_new, b1_new, w2_new, b2_new, h_new,
            y_hat_new, loss_new,
        }
    }

    /// A 1×1 convolution at one pixel: the channel vector times the (C_out, C_in) matrix.
    pub fn pointwise(vector: &[f64], matrix: &[Vec<f64>]) -> Vec<f64> {
        matvec(matrix, vector)
    }

    /// Output length of a forward convolution: floor((l + 2p − d(k − 1) − 1) / s) + 1.
    pub fn conv_out(l: usize, k: usize, s: usize, p: usize, d: usize) -> usize {
        (l + 2 * p - d * (k - 1) - 1) / s + 1
    }

    /// Output length of a transposed convolution, the documents' formula: (l − 1)s − 2p + d(k − 1) + output_padding + 1.
    pub fn tconv_out(l: usize, k: usize, s: usize, p: usize, d: usize, op: usize) -> usize {
        (l - 1) * s + d * (k - 1) + op + 1 - 2 * p
    }

    /// The input indices each forward window reads (0-based; below 0 or at least `l` means a padding cell):
    /// window i covers i·s − p + j·d for j = 0..k.
    pub fn conv_windows(l: usize, k: usize, s: usize, p: usize, d: usize) -> Vec<Vec<i64>> {
        (0..conv_out(l, k, s, p, d))
            .map(|i| (0..k).map(|j| (i * s + j * d) as i64 - p as i64).collect())
            .collect()
    }

    /// The output positions each input of a transposed convolution stamps (0-based after the padding crop; a
    /// negative position is cropped by the padding, one at or beyond the output length is kept only if
    /// `output_padding` extends the axis that far): input i reaches i·s + j·d − p for j = 0..k.
    pub fn tconv_stamps(lin: usize, k: usize, s: usize, p: usize, d: usize) -> Vec<Vec<i64>> {
        (0..lin).map(|i| (0..k).map(|j| (i * s + j * d) as i64 - p as i64).collect()).collect()
    }
}

/// One function per figure. Each draws the mechanism with the document's own numbers, computed in `calc`.
mod figs {
    use crate::calc;
    use crate::svg::{self, Anchor, Cell, Svg, cell, num, numt};
    use crate::tok;

    #[derive(Clone, Copy, PartialEq, Eq)]
    pub enum Variant {
        Guide,
        Handbook,
    }

    impl Variant {
        fn suffix(self) -> &'static str {
            match self {
                Variant::Guide => "guide",
                Variant::Handbook => "handbook",
            }
        }
    }

    /// A shape in the document's notation: the guide writes `(1, 2, 3, 4)`, the handbook `[1,2,3,4]`.
    fn shape(dims: &[&str], v: Variant) -> String {
        match v {
            Variant::Guide => format!("({})", dims.join(", ")),
            Variant::Handbook => format!("[{}]", dims.join(",")),
        }
    }

    fn vec_str(v: &[f64], decimals: usize) -> String {
        format!("[{}]", v.iter().map(|x| numt(*x, decimals)).collect::<Vec<_>>().join(", "))
    }

    /// One product term of a convolution window, with a negative weight in parentheses.
    fn term(w: f64, x: f64) -> String {
        let ws = numt(w, 2);
        let ws = if w < 0.0 { format!("({ws})") } else { ws };
        format!("{ws}·{}", numt(x, 2))
    }

    /// Cells of a matrix at a fixed number of decimals; zeros in muted ink when `zero_muted`.
    fn grid_cells(mat: &[Vec<f64>], decimals: usize, zero_muted: bool) -> Vec<Vec<Cell>> {
        mat.iter()
            .map(|row| {
                row.iter()
                    .map(|v| {
                        let c = cell(numt(*v, decimals));
                        if zero_muted && *v == 0.0 { c.ink(tok::MUTED) } else { c }
                    })
                    .collect()
            })
            .collect()
    }

    /// Cell width that fits the longest formatted entry of a matrix (at least `min`).
    fn grid_w(mat: &[Vec<f64>], decimals: usize, min: f64) -> f64 {
        let longest = mat.iter().flatten().map(|v| svg::text_w(&numt(*v, decimals), 11)).fold(0.0, f64::max);
        (longest + 10.0).max(min).ceil()
    }

    // ---------------------------------------------------------------------------------------------
    // F01 broadcasting: shapes align from the right
    // ---------------------------------------------------------------------------------------------
    fn shape_row(s: &mut Svg, x_right: f64, y: f64, dims: &[&str], label: &str, marks: &[(usize, &str, bool)]) {
        let (bw, bh, gap) = (46.0, 26.0, 6.0);
        let n = dims.len() as f64;
        let x0 = x_right - n * bw - (n - 1.0) * gap;
        s.text(x0 - 10.0, y + bh / 2.0 + 4.0, label, 12, tok::INK2, Anchor::End);
        for (i, d) in dims.iter().enumerate() {
            let x = x0 + i as f64 * (bw + gap);
            let mark = marks.iter().find(|m| m.0 == i);
            match mark {
                Some((_, colour, true)) => {
                    s.rect(x, y, bw, bh, tok::SURFACE, Some(tok::GRID));
                    s.rect_dashed(x, y, bw, bh, colour);
                }
                Some((_, colour, false)) => s.rect_bold(x, y, bw, bh, tok::SURFACE, colour),
                None => s.rect(x, y, bw, bh, tok::SURFACE, Some(tok::GRID)),
            }
            s.text(x + bw / 2.0, y + bh / 2.0 + 4.0, d, 12, tok::INK, Anchor::Middle);
        }
    }

    pub fn fig_broadcast(v: Variant) -> (String, String) {
        let mut s = Svg::new(
            720,
            250,
            "Broadcasting aligns shapes from the right",
            "Two panels of shape rulers. Left: a size-1 axis stretches to match the other operand. Right: a one-dimensional per-channel vector aligned from the right lands on the last axis unless it is reshaped to put its length on the channel axis.",
        );
        let (bw, gap) = (46.0, 6.0);
        // Left panel
        let xr = 330.0;
        s.text_bold(20.0, 24.0, "asked for: broadcast", 13, tok::INK, Anchor::Start);
        let (a, b, r, la, lb, axes) = match v {
            Variant::Guide => (
                ["1", "2", "2", "2"],
                ["1", "2", "1", "1"],
                ["1", "2", "2", "2"],
                "image",
                "gate",
                ["batch", "channels", "height", "width"],
            ),
            Variant::Handbook => (
                ["B", "C", "H", "W"],
                ["1", "C", "1", "1"],
                ["B", "C", "H", "W"],
                "image",
                "bias",
                ["B", "C", "H", "W"],
            ),
        };
        let x0 = xr - 4.0 * bw - 3.0 * gap;
        for (i, name) in axes.iter().enumerate() {
            s.text(x0 + i as f64 * (bw + gap) + bw / 2.0, 46.0, name, 11, tok::MUTED, Anchor::Middle);
        }
        shape_row(&mut s, xr, 54.0, &a, la, &[]);
        shape_row(&mut s, xr, 96.0, &b, lb, &[(2, tok::S1, true), (3, tok::S1, true)]);
        // stretched arrows from the two size-1 boxes to the result row
        for i in [2usize, 3] {
            let xc = x0 + i as f64 * (bw + gap) + bw / 2.0;
            s.arrow(xc, 124.0, xc, 148.0, None);
        }
        s.text(x0 + 2.5 * (bw + gap), 140.0, "stretched", 11, tok::INK2, Anchor::Middle);
        shape_row(&mut s, xr, 152.0, &r, "result", &[]);
        let note_l = match v {
            Variant::Guide => "broadcast_mul: sizes 1 grow to 2 and 2; plain * refuses",
            Variant::Handbook => "compare from the right: equal, or one of them is 1",
        };
        s.text(x0 + 2.0 * (bw + gap) - gap / 2.0, 206.0, note_l, 11, tok::INK2, Anchor::Middle);
        // divider
        s.line(356.0, 16.0, 356.0, 236.0, tok::GRID, 1.0);
        // Right panel: the trap
        let xr2 = 690.0;
        s.text_bold(380.0, 24.0, "the trap: a bare per-channel vector", 13, tok::INK, Anchor::Start);
        let (w, alpha, alpha_r, lw, lal, lar, axes2) = match v {
            Variant::Guide => (
                ["3", "2", "3", "3"],
                ["3"],
                ["3", "1", "1", "1"],
                "weight",
                "α (3,)",
                "α reshaped",
                ["out", "in", "kh", "kw"],
            ),
            Variant::Handbook => (
                ["B", "C", "H", "W"],
                ["C"],
                ["1", "C", "1", "1"],
                "image",
                "bias [C]",
                "bias [1,C,1,1]",
                ["B", "C", "H", "W"],
            ),
        };
        let x2 = xr2 - 4.0 * bw - 3.0 * gap;
        for (i, name) in axes2.iter().enumerate() {
            s.text(x2 + i as f64 * (bw + gap) + bw / 2.0, 46.0, name, 11, tok::MUTED, Anchor::Middle);
        }
        shape_row(&mut s, xr2, 54.0, &w, lw, &[]);
        shape_row(&mut s, xr2, 96.0, &alpha, lal, &[(0, tok::S2, false)]);
        let lands = match v {
            Variant::Guide => "lands on kw: the kernel columns",
            Variant::Handbook => "lands on W: the width",
        };
        s.text(xr2, 136.0, lands, 11, tok::S2, Anchor::End);
        s.text(xr2, 149.0, "(aligned from the right)", 11, tok::INK2, Anchor::End);
        let ch_idx = match v {
            Variant::Guide => 0,
            Variant::Handbook => 1,
        };
        shape_row(&mut s, xr2, 160.0, &alpha_r, lar, &[(ch_idx, tok::S1, false)]);
        let fix = match v {
            Variant::Guide => "reshape first: one α per output channel",
            Variant::Handbook => "reshape first: one bias per channel C",
        };
        s.text(x2 + 2.0 * (bw + gap) - gap / 2.0, 206.0, fix, 11, tok::INK2, Anchor::Middle);
        let title_a = match v {
            Variant::Guide => "size 1 stretches; a (3,) vector aligns with the last axis",
            Variant::Handbook => "size 1 stretches; a [C] vector aligns with the last axis",
        };
        s.text(360.0, 236.0, title_a, 11, tok::MUTED, Anchor::Middle);
        (format!("f01-broadcast-{}.svg", v.suffix()), s.finish())
    }

    // ---------------------------------------------------------------------------------------------
    // F02 reshape regroups, transpose re-strides
    // ---------------------------------------------------------------------------------------------
    pub fn fig_layout(v: Variant) -> (String, String) {
        let (storage, view_a, strides_a, view_b, strides_b, label_a, label_b): (Vec<i64>, Vec<Vec<i64>>, &str, Vec<Vec<i64>>, &str, &str, &str) = match v {
            Variant::Guide => {
                let st: Vec<i64> = (0..24).collect();
                let a: Vec<Vec<i64>> = (0..2).map(|r| (0..12).map(|c| 12 * r + c).collect()).collect();
                let b: Vec<Vec<i64>> = (0..12).map(|r| vec![r, 12 + r]).collect();
                (st, a, "strides (24, 12, 1)", b, "strides (24, 1, 12)", "reshape → (1, 2, 12)", "transpose(1, 2) → (1, 12, 2)")
            }
            Variant::Handbook => {
                let st: Vec<i64> = (1..=6).collect();
                let a: Vec<Vec<i64>> = (0..3).map(|r| vec![2 * r + 1, 2 * r + 2]).collect();
                let b: Vec<Vec<i64>> = (0..3).map(|r| vec![r + 1, r + 4]).collect();
                (st, a, "strides (2, 1)", b, "strides (1, 3)", "reshape to [3,2]", "transpose to [3,2]")
            }
        };
        let copy_cells: Vec<i64> = view_b.iter().flatten().copied().collect();
        let cw = match v {
            Variant::Guide => 22.0,
            Variant::Handbook => 30.0,
        };
        // rows of view B actually drawn (the guide shows 0, 1, 2, ⋮, 11)
        let (rows_b, drawn_b): (Vec<Vec<Cell>>, usize) = match v {
            Variant::Guide => {
                let mut rows: Vec<Vec<Cell>> = (0..3).map(|r| view_b[r].iter().map(|x| cell(x.to_string())).collect()).collect();
                rows.push(vec![cell("⋮").stroke(tok::SURFACE), cell("⋮").stroke(tok::SURFACE)]);
                rows.push(view_b[11].iter().map(|x| cell(x.to_string())).collect());
                (rows, 5)
            }
            Variant::Handbook => (view_b.iter().map(|r| r.iter().map(|x| cell(x.to_string())).collect()).collect(), 3),
        };
        let ay = 84.0;
        let a_h = view_a.len() as f64 * cw;
        let b_h = drawn_b as f64 * cw;
        let bottom = (ay + a_h + 30.0).max(ay + b_h + 44.0);
        let cy = bottom + 30.0;
        let height = (cy + cw + 18.0).ceil() as u32;
        let mut s = Svg::new(
            720,
            height,
            "Reshape regroups the row-major sequence; transpose changes strides over the same storage",
            "A storage strip of numbers in memory order, a reshaped view that reads them in order with its strides, a transposed view that reads the same storage with swapped strides, and the new storage that contiguous() copies.",
        );
        let n = storage.len() as f64;
        let sx = (720.0 - n * cw) / 2.0;
        let head = match v {
            Variant::Guide => "storage of the (1, 2, 3, 4) tensor, row-major: 24 numbers that never move",
            Variant::Handbook => "storage of A [2,3], row-major: the six numbers never move",
        };
        s.text(360.0, 22.0, head, 12, tok::INK2, Anchor::Middle);
        let strip: Vec<Vec<Cell>> = vec![storage.iter().map(|x| cell(x.to_string())).collect()];
        s.cells(sx, 32.0, cw, &strip);
        // view A (left)
        let ax = 40.0;
        let rows_a: Vec<Vec<Cell>> = view_a.iter().map(|r| r.iter().map(|x| cell(x.to_string())).collect()).collect();
        s.cells(ax, ay, cw, &rows_a);
        s.text(ax + view_a[0].len() as f64 * cw + 12.0, ay + cw / 2.0 + 4.0, strides_a, 11, tok::INK2, Anchor::Start);
        s.text(ax, ay + a_h + 16.0, label_a, 12, tok::INK, Anchor::Start);
        let read_a = match v {
            Variant::Guide => "view[0][r][c] = storage[12·r + 1·c]: still in order",
            Variant::Handbook => "view[r][c] = storage[2·r + 1·c]: still in order",
        };
        s.text(ax, ay + a_h + 30.0, read_a, 11, tok::INK2, Anchor::Start);
        // view B (right)
        let bx = 440.0;
        s.cells(bx, ay, cw, &rows_b);
        s.rect_bold(bx, ay, 2.0 * cw, cw, "none", tok::S1);
        s.text(bx + 2.0 * cw + 12.0, ay + cw / 2.0 + 4.0, strides_b, 11, tok::INK2, Anchor::Start);
        s.text(bx, ay + b_h + 16.0, label_b, 12, tok::INK, Anchor::Start);
        let (read_b, read_b2) = match v {
            Variant::Guide => ("view[0][r][c] = storage[1·r + 12·c]", "token 0 = [0, 12]; not contiguous"),
            Variant::Handbook => ("view[r][c] = storage[1·r + 3·c]", "not contiguous until copied"),
        };
        s.text(bx, ay + b_h + 30.0, read_b, 11, tok::INK2, Anchor::Start);
        s.text(bx, ay + b_h + 44.0, read_b2, 11, tok::INK2, Anchor::Start);
        // thin lines from storage cells to their positions in both views
        let sy = 32.0 + cw;
        let pick: Vec<(usize, f64, f64)> = match v {
            Variant::Guide => vec![(0, ax + cw / 2.0, ay), (12, ax + 12.0 * cw + 1.0, ay + 1.5 * cw), (0, bx + cw / 2.0, ay), (12, bx + cw + cw / 2.0, ay)],
            Variant::Handbook => vec![(0, ax + cw / 2.0, ay), (1, ax + cw + cw / 2.0, ay), (0, bx + cw / 2.0, ay), (3, bx + cw + cw / 2.0, ay)],
        };
        for (idx, tx, ty) in pick {
            let from_x = sx + idx as f64 * cw + cw / 2.0;
            s.line(from_x, sy + 1.0, tx, ty - 1.0, tok::S1, 1.0);
        }
        // contiguous copy strip
        let shown: Vec<i64> = copy_cells.iter().take(match v { Variant::Guide => 8, Variant::Handbook => 6 }).copied().collect();
        let mut copy_row: Vec<Cell> = shown.iter().map(|x| cell(x.to_string()).fill(tok::FILL1)).collect();
        if v == Variant::Guide {
            copy_row.push(cell("…").stroke(tok::SURFACE));
        }
        s.text(ax, cy - 10.0, "after contiguous(): a new storage, copied in the transposed order", 12, tok::INK, Anchor::Start);
        s.cells(ax, cy, cw, &vec![copy_row]);
        let ncols = shown.len() as f64 + if v == Variant::Guide { 1.0 } else { 0.0 };
        s.text(ax + ncols * cw + 12.0, cy + cw / 2.0 + 4.0, "the only step that moves numbers", 11, tok::INK2, Anchor::Start);
        (format!("f02-reshape-transpose-{}.svg", v.suffix()), s.finish())
    }

    // ---------------------------------------------------------------------------------------------
    // F03 a feature map becomes a sequence of pixel tokens
    // ---------------------------------------------------------------------------------------------
    pub fn fig_tokens(v: Variant) -> (String, String) {
        let mut s = Svg::new(
            720,
            260,
            "flatten_from(2).transpose(1, 2) turns (b, c, h, w) into (b, h·w, c): one token per pixel",
            "Stacked channel planes of a 3 by 4 map on the left, the flattened c by 12 strip in the middle, and the 12 token rows on the right; one pixel is highlighted through all three stages with its index arithmetic.",
        );
        let (h, w) = (3usize, 4usize);
        let cellw = 26.0;
        let (planes_label, hl, front_indices, token_label, strip_label, tokens_shape, tail_note) = match v {
            Variant::Guide => (
                "channel 0 (front): 0 … 11; channel 1 (behind): 12 … 23",
                (0usize, 0usize),
                (0..12).map(|i| i.to_string()).collect::<Vec<_>>(),
                "token 0 = [0, 12]: pixel (0, 0) through both channels",
                "flatten_from(2) → (1, 2, 12)",
                "transpose(1, 2) → (1, 12, 2)",
                "12 tokens × 2 features",
            ),
            Variant::Handbook => (
                "64 feature planes, each 3 × 4 (two drawn)",
                (1usize, 2usize),
                (0..12).map(|i| i.to_string()).collect::<Vec<_>>(),
                "token 6 = row 1 · 4 + col 2: all 64 features of that location",
                "flatten_from(2) → [2,64,12]",
                "transpose(1, 2) → [2,12,64]",
                "12 tokens × 64 features",
            ),
        };
        // planes (back plane offset up-right)
        let (px, py) = (40.0, 70.0);
        s.rect(px + 12.0, py - 12.0, w as f64 * cellw, h as f64 * cellw, tok::SURFACE, Some(tok::AXIS));
        if v == Variant::Handbook {
            s.rect(px + 24.0, py - 24.0, w as f64 * cellw, h as f64 * cellw, tok::SURFACE, Some(tok::AXIS));
        }
        let front: Vec<Vec<Cell>> = (0..h)
            .map(|r| {
                (0..w)
                    .map(|c| {
                        let i = r * w + c;
                        let cl = cell(front_indices[i].clone()).ink(tok::MUTED);
                        if (r, c) == hl { cl.fill(tok::FILL2).ink(tok::INK) } else { cl }
                    })
                    .collect()
            })
            .collect();
        s.cells(px, py, cellw, &front);
        s.rect_bold(px + hl.1 as f64 * cellw, py + hl.0 as f64 * cellw, cellw, cellw, "none", tok::S1);
        s.text(px, py + h as f64 * cellw + 18.0, planes_label, 11, tok::INK2, Anchor::Start);
        let in_shape = match v {
            Variant::Guide => "(1, 2, 3, 4) = (b, c, h, w)",
            Variant::Handbook => "[2,64,3,4] = [B,C,H,W]",
        };
        s.text(px, 36.0, in_shape, 12, tok::INK, Anchor::Start);
        // strip c × (h·w)
        let (sx, sy, scw) = (250.0, 84.0, 18.0);
        s.arrow(px + w as f64 * cellw + 16.0, py + 30.0, sx - 8.0, sy + 20.0, None);
        s.text((px + w as f64 * cellw + sx) / 2.0 + 4.0, py + 18.0, "flatten_from(2)", 11, tok::INK2, Anchor::Middle);
        let hl_index = hl.0 * w + hl.1;
        let strip_rows: Vec<Vec<Cell>> = (0..2)
            .map(|ch| {
                (0..12)
                    .map(|i| {
                        let text = match v {
                            Variant::Guide => (ch * 12 + i).to_string(),
                            Variant::Handbook => String::new(),
                        };
                        let cl = cell(text).ink(tok::MUTED);
                        if i == hl_index { cl.fill(tok::FILL2) } else { cl }
                    })
                    .collect()
            })
            .collect();
        s.cells(sx, sy, scw, &strip_rows);
        if v == Variant::Handbook {
            s.text(sx + 6.0 * scw, sy + 2.0 * scw + 14.0, "⋮ 64 rows", 11, tok::MUTED, Anchor::Middle);
        }
        s.text(sx, 36.0, strip_label, 12, tok::INK, Anchor::Start);
        s.text(sx, sy - 8.0, "rows: channels; columns: pixel index r·w + c", 11, tok::INK2, Anchor::Start);
        s.rect_bold(sx + hl_index as f64 * scw, sy, scw, 2.0 * scw, "none", tok::S1);
        // tokens column
        let (tx, ty, tcw) = (548.0, 56.0, 18.0);
        s.arrow(sx + 12.0 * scw + 12.0, sy + scw, tx - 8.0, sy + scw, None);
        s.text((sx + 12.0 * scw + tx) / 2.0, sy + scw - 8.0, "transpose(1, 2)", 11, tok::INK2, Anchor::Middle);
        s.text(tx - 20.0, 36.0, tokens_shape, 12, tok::INK, Anchor::Start);
        let shown: Vec<usize> = vec![0, 1, 2, usize::MAX, 6, usize::MAX, 11];
        let mut rows_t: Vec<Vec<Cell>> = Vec::new();
        for (k, &t) in shown.iter().enumerate() {
            if t == usize::MAX {
                rows_t.push(vec![cell("⋮").stroke(tok::SURFACE), cell("").stroke(tok::SURFACE)]);
            } else {
                let texts: [String; 2] = match v {
                    Variant::Guide => [t.to_string(), (12 + t).to_string()],
                    Variant::Handbook => [String::new(), String::new()],
                };
                let fill = if t == hl_index { tok::FILL2 } else { tok::SURFACE };
                rows_t.push(vec![cell(texts[0].clone()).fill(fill).ink(tok::MUTED), cell(texts[1].clone()).fill(fill).ink(tok::MUTED)]);
                s.text(tx + 2.0 * tcw + 6.0, ty + k as f64 * tcw + tcw / 2.0 + 4.0, &format!("token {t}"), 11, tok::INK2, Anchor::Start);
            }
        }
        s.cells(tx, ty, tcw, &rows_t);
        let hl_row = shown.iter().position(|&t| t == hl_index).unwrap_or(0);
        s.rect_bold(tx, ty + hl_row as f64 * tcw, 2.0 * tcw, tcw, "none", tok::S1);
        s.text(tx + tcw, ty + 7.0 * tcw + 16.0, tail_note, 11, tok::INK2, Anchor::Middle);
        s.text(tx + tcw, ty + 7.0 * tcw + 30.0, "each row: c features", 11, tok::MUTED, Anchor::Middle);
        s.text(360.0, 236.0, token_label, 12, tok::INK, Anchor::Middle);
        (format!("f03-image-to-tokens-{}.svg", v.suffix()), s.finish())
    }

    // ---------------------------------------------------------------------------------------------
    // F04 one storage, three handles
    // ---------------------------------------------------------------------------------------------
    pub fn fig_handles(v: Variant) -> (String, String) {
        let mut s = Svg::new(
            720,
            300,
            "The layer's tensor, the VarMap's Var and the optimizer's Var are three handles to one storage",
            "A storage box in the centre with three handle boxes pointing into it, and below it the forward output before and after one SGD step overwrites the storage in place.",
        );
        let st = calc::one_layer_step();
        let (sx, sy, sw, sh) = (280.0, 100.0, 160.0, 64.0);
        s.rect_bold(sx, sy, sw, sh, tok::FILL1, tok::S1);
        let (l1, l2) = match v {
            Variant::Guide => ("one storage".to_string(), format!("weight = {}", vec_str(&st.new_w.iter().map(|_| 1.0).collect::<Vec<_>>(), 0))),
            Variant::Handbook => ("one storage".to_string(), "readout.weight [1, 2]".to_string()),
        };
        s.text(sx + sw / 2.0, sy + 26.0, &l1, 13, tok::INK, Anchor::Middle);
        s.text(sx + sw / 2.0, sy + 46.0, &l2, 12, tok::INK2, Anchor::Middle);
        // handles
        let (hl1, hl2, ht1, ht2, hr1, hr2, a1, a2, a3) = match v {
            Variant::Guide => (
                "layer field",
                "weight: Var (same storage)",
                "VarMap entry",
                "\"weight\" → Var",
                "optimizer",
                "all_vars(): Vec<Var>",
                "same id",
                "same id",
                "same id",
            ),
            Variant::Handbook => (
                "model field",
                "Tensor from get_with_hints",
                "VarMap entry",
                "\"readout.weight\" → Var",
                "optimizer",
                "all_vars(): Vec<Var>",
                "get_with_hints",
                "all_vars()",
                "backward_step writes",
            ),
        };
        s.labelled_box2(40.0, sy, 180.0, sh, hl1, hl2, tok::SURFACE, tok::AXIS);
        s.labelled_box2(sx, 16.0, sw, 52.0, ht1, ht2, tok::SURFACE, tok::AXIS);
        s.labelled_box2(500.0, sy, 180.0, sh, hr1, hr2, tok::SURFACE, tok::AXIS);
        s.arrow(220.0, sy + sh / 2.0, sx - 4.0, sy + sh / 2.0, Some(a1));
        s.arrow(sx + sw / 2.0, 68.0, sx + sw / 2.0, sy - 4.0, None);
        s.text(sx + sw / 2.0 + 8.0, 88.0, a2, 11, tok::INK2, Anchor::Start);
        s.arrow(500.0, sy + sh / 2.0, sx + sw + 4.0, sy + sh / 2.0, Some(a3));
        // before / after
        let y2 = 220.0;
        match v {
            Variant::Guide => {
                let before = format!("forward({}) = {}", vec_str(&[1.0, 2.0], 0), numt(st.output, 2));
                let after = format!("forward({}) = {}", vec_str(&[1.0, 2.0], 0), numt(st.new_output, 2));
                s.labelled_box2(40.0, y2, 200.0, 48.0, "before the step", &before, tok::SURFACE, tok::GRID);
                s.labelled_box2(480.0, y2, 200.0, 48.0, "after the step", &after, tok::SURFACE, tok::GRID);
                let step = format!("SGD: w ← w − 0.1·{} = {}", vec_str(&st.grad, 0), vec_str(&st.new_w, 1));
                s.arrow(240.0, y2 + 24.0, 476.0, y2 + 24.0, Some(&step));
                s.text(360.0, y2 + 66.0, "written into the one storage; nothing is copied", 11, tok::INK2, Anchor::Middle);
                s.text(sx + sw / 2.0, sy + sh + 20.0, "layer.weight.id() == varmap.all_vars()[0].id()", 11, tok::INK2, Anchor::Middle);
            }
            Variant::Handbook => {
                s.labelled_box2(40.0, y2, 200.0, 48.0, "before the step", "model.forward(x) reads the storage", tok::SURFACE, tok::GRID);
                s.labelled_box2(480.0, y2, 200.0, 48.0, "after the step", "the same handle, new values", tok::SURFACE, tok::GRID);
                s.arrow(240.0, y2 + 24.0, 476.0, y2 + 24.0, Some("optimizer.backward_step(&loss)"));
                s.text(360.0, y2 + 66.0, "the map, the model and the optimizer see the update at once", 11, tok::INK2, Anchor::Middle);
                s.text(sx + sw / 2.0, sy + sh + 20.0, "moving vb into build_model moved a handle, not the map", 11, tok::INK2, Anchor::Middle);
            }
        }
        (format!("f04-one-storage-{}.svg", v.suffix()), s.finish())
    }

    // ---------------------------------------------------------------------------------------------
    // F05 what backward multiplies at each layer
    // ---------------------------------------------------------------------------------------------
    fn chain_box(s: &mut Svg, x: f64, y: f64, w: f64, name: &str, value: &str) {
        s.rect(x, y, w, 44.0, tok::SURFACE, Some(tok::AXIS));
        s.text(x + w / 2.0, y + 18.0, name, 12, tok::INK, Anchor::Middle);
        s.text(x + w / 2.0, y + 34.0, value, 11, tok::INK2, Anchor::Middle);
    }

    pub fn fig_backward(v: Variant) -> (String, String) {
        match v {
            Variant::Guide => {
                let st = calc::one_layer_step();
                let mut s = Svg::new(
                    720,
                    320,
                    "backward walks the chain in reverse: δ at the output, δ·xᵀ for the weight, then one SGD step",
                    "A left-to-right chain x, linear layer, output, loss with forward values above and gradients below, and a final row with the SGD update.",
                );
                let y = 70.0;
                let boxes = [
                    (30.0, 100.0, "x", vec_str(&[1.0, 2.0], 0)),
                    (180.0, 170.0, "Linear, no bias", format!("W = {}", vec_str(&[1.0, 1.0], 0))),
                    (400.0, 110.0, "ŷ = W·x", numt(st.output, 0)),
                    (560.0, 130.0, "L = (ŷ − 1)²", numt(st.loss, 0)),
                ];
                for (x, w, name, val) in &boxes {
                    chain_box(&mut s, *x, y, *w, name, val);
                }
                s.arrow(130.0, y + 22.0, 176.0, y + 22.0, Some("forward"));
                s.arrow(350.0, y + 22.0, 396.0, y + 22.0, None);
                s.arrow(510.0, y + 22.0, 556.0, y + 22.0, None);
                // backward row
                let yb = 150.0;
                s.arrow(556.0, yb, 510.0, yb, Some("backward"));
                s.arrow(396.0, yb, 350.0, yb, None);
                s.arrow(176.0, yb, 130.0, yb, None);
                s.text(625.0, yb + 26.0, &format!("δ = ∂L/∂ŷ = 2·({} − 1) = {}", numt(st.output, 0), numt(st.delta, 0)), 11, tok::INK2, Anchor::Middle);
                s.text(455.0, yb + 26.0, "δ arrives at ŷ", 11, tok::INK2, Anchor::Middle);
                s.text(265.0, yb + 26.0, &format!("∂L/∂W = δ·xᵀ = {}·{} = {}", numt(st.delta, 0), vec_str(&[1.0, 2.0], 0), vec_str(&st.grad, 0)), 11, tok::INK2, Anchor::Middle);
                s.text(265.0, yb + 42.0, &format!("∂L/∂x = Wᵀ·δ = {} (passed on if x had a producer)", vec_str(&[st.delta, st.delta], 0)), 11, tok::MUTED, Anchor::Middle);
                s.text(92.0, yb + 26.0, "x is data: no gradient kept", 11, tok::MUTED, Anchor::Middle);
                // GradStore + step
                let ys = 240.0;
                s.rect(30.0, ys, 660.0, 52.0, tok::FILL1, Some(tok::S1));
                s.text(360.0, ys + 20.0, &format!("GradStore: {{ W ↦ {} }} — computed with the forward-pass values; nothing updated yet", vec_str(&st.grad, 0)), 12, tok::INK, Anchor::Middle);
                let upd = format!(
                    "then one SGD step (η = 0.1): W ← {} − 0.1·{} = {}, and forward({}) = {}",
                    vec_str(&[1.0, 1.0], 0),
                    vec_str(&st.grad, 0),
                    vec_str(&st.new_w, 1),
                    vec_str(&[1.0, 2.0], 0),
                    numt(st.new_output, 0)
                );
                s.text(360.0, ys + 40.0, &upd, 12, tok::INK2, Anchor::Middle);
                (format!("f05-backward-{}.svg", v.suffix()), s.finish())
            }
            Variant::Handbook => {
                let b = calc::two_layer_backward();
                let mut s = Svg::new(
                    720,
                    380,
                    "Backpropagation through a two-unit ReLU network: every gradient uses the forward-pass values, and the update comes last",
                    "A left-to-right chain x, first layer, ReLU, second layer, prediction, loss with forward values above and gradients below, then a row with the updated parameters and the recomputed prediction.",
                );
                let y = 70.0;
                let boxes: [(f64, f64, &str, String); 6] = [
                    (20.0, 70.0, "x", vec_str(&b.x, 0)),
                    (110.0, 120.0, "x W₁ᵀ + b₁ = z₁", vec_str(&b.z1, 0)),
                    (250.0, 100.0, "ReLU → h", vec_str(&b.h, 0)),
                    (370.0, 120.0, "h·w₂ + b₂ = ŷ", numt(b.y_hat, 0)),
                    (510.0, 80.0, "target y", "0".to_string()),
                    (610.0, 90.0, "L = ½(ŷ − y)²", numt(b.loss, 1)),
                ];
                for (x, w, name, val) in &boxes {
                    chain_box(&mut s, *x, y, *w, name, val);
                }
                s.text(170.0, y - 12.0, "W₁ = I, b₁ = [0, 0]", 11, tok::INK2, Anchor::Middle);
                s.text(430.0, y - 12.0, "w₂ = [1, −1], b₂ = 0", 11, tok::INK2, Anchor::Middle);
                s.arrow(90.0, y + 22.0, 106.0, y + 22.0, None);
                s.arrow(230.0, y + 22.0, 246.0, y + 22.0, None);
                s.arrow(350.0, y + 22.0, 366.0, y + 22.0, None);
                s.arrow(490.0, y + 22.0, 506.0, y + 22.0, None);
                s.arrow(590.0, y + 22.0, 606.0, y + 22.0, None);
                s.text(360.0, y + 60.0, "forward →", 11, tok::INK2, Anchor::Middle);
                // backward texts
                let yb = 160.0;
                s.arrow(700.0, yb, 40.0, yb, None);
                s.text(360.0, yb - 6.0, "← backward: each box multiplies the gradient it receives by its own local derivative", 11, tok::INK2, Anchor::Middle);
                s.text(655.0, yb + 22.0, &format!("∂L/∂ŷ = ŷ − y = {}", numt(b.d_yhat, 0)), 11, tok::INK2, Anchor::Middle);
                s.text(430.0, yb + 22.0, "∂L/∂w₂ = δ·hᵀ", 11, tok::INK2, Anchor::Middle);
                s.text(430.0, yb + 36.0, &format!("= {}", vec_str(&b.d_w2, 0)), 11, tok::INK2, Anchor::Middle);
                s.text(430.0, yb + 50.0, &format!("∂L/∂b₂ = {}", numt(b.d_b2, 0)), 11, tok::INK2, Anchor::Middle);
                s.text(430.0, yb + 64.0, &format!("∂L/∂h = w₂ᵀ·δ = {}", vec_str(&b.d_h, 0)), 11, tok::INK2, Anchor::Middle);
                s.text(300.0, yb + 22.0, "ReLU: both z₁ > 0,", 11, tok::INK2, Anchor::Middle);
                s.text(300.0, yb + 36.0, "so δ passes unchanged:", 11, tok::INK2, Anchor::Middle);
                s.text(300.0, yb + 50.0, &vec_str(&b.d_z1, 0), 11, tok::INK2, Anchor::Middle);
                let d_w1 = format!(
                    "= [[{}, {}], [{}, {}]]",
                    numt(b.d_w1[0][0], 0),
                    numt(b.d_w1[0][1], 0),
                    numt(b.d_w1[1][0], 0),
                    numt(b.d_w1[1][1], 0)
                );
                s.text(170.0, yb + 22.0, "∂L/∂W₁ = δ·xᵀ", 11, tok::INK2, Anchor::Middle);
                s.text(170.0, yb + 36.0, &d_w1, 11, tok::INK2, Anchor::Middle);
                s.text(170.0, yb + 50.0, &format!("∂L/∂b₁ = {}", vec_str(&b.d_b1, 0)), 11, tok::INK2, Anchor::Middle);
                s.text(55.0, yb + 22.0, "x: data", 11, tok::MUTED, Anchor::Middle);
                // update row
                let ys = 268.0;
                s.rect(20.0, ys, 680.0, 92.0, tok::FILL1, Some(tok::S1));
                s.text(360.0, ys + 20.0, "all gradients first, then one SGD step with η = 0.1:", 12, tok::INK, Anchor::Middle);
                let w1n = format!(
                    "W₁' = [[{}, {}], [{}, {}]]   b₁' = {}",
                    numt(b.w1_new[0][0], 1),
                    numt(b.w1_new[0][1], 1),
                    numt(b.w1_new[1][0], 1),
                    numt(b.w1_new[1][1], 1),
                    vec_str(&b.b1_new, 1)
                );
                s.text(360.0, ys + 40.0, &w1n, 12, tok::INK2, Anchor::Middle);
                s.text(360.0, ys + 58.0, &format!("w₂' = {}   b₂' = {}", vec_str(&b.w2_new, 1), numt(b.b2_new, 1)), 12, tok::INK2, Anchor::Middle);
                s.text(
                    360.0,
                    ys + 78.0,
                    &format!("recomputed: h' = {}, ŷ' = {}, L' = {}", vec_str(&b.h_new, 1), numt(b.y_hat_new, 2), numt(b.loss_new, 4)),
                    12,
                    tok::INK2,
                    Anchor::Middle,
                );
                (format!("f05-backward-{}.svg", v.suffix()), s.finish())
            }
        }
    }

    // ---------------------------------------------------------------------------------------------
    // F06 softmax: subtract the maximum first
    // ---------------------------------------------------------------------------------------------
    fn stage(s: &mut Svg, x: f64, y: f64, w: f64, name: &str, value: &str, bad: bool) {
        if bad {
            s.rect_bold(x, y, w, 44.0, tok::SURFACE, tok::S2);
        } else {
            s.rect(x, y, w, 44.0, tok::SURFACE, Some(tok::AXIS));
        }
        s.text(x + w / 2.0, y + 17.0, name, 11, tok::INK2, Anchor::Middle);
        s.text(x + w / 2.0, y + 34.0, value, 12, tok::INK, Anchor::Middle);
    }

    pub fn fig_softmax(v: Variant) -> (String, String) {
        let mut s = Svg::new(
            720,
            260,
            "Subtracting the maximum before exponentiating leaves the probabilities unchanged and prevents overflow",
            "Two lanes of stage boxes: the textbook formula overflows to infinity and ends in NaN; the shifted lane subtracts the maximum, so every exponent is at most zero and the denominator is at least one.",
        );
        let big: Vec<f64> = match v {
            Variant::Guide => vec![1000.0, 1005.0, 1002.0],
            Variant::Handbook => vec![1001.0, 1005.0, 1002.0],
        };
        let (n_exps, n_sum, n_probs) = calc::naive_softmax(&big);
        let (m, shifted, exps, sum, probs) = calc::shifted_softmax(&big);
        let xs = [20.0, 150.0, 280.0, 410.0, 560.0];
        let w = 118.0;
        let y1 = 56.0;
        let y2 = 166.0;
        s.text_bold(20.0, 30.0, "textbook: exp(z) / Σ exp(z)", 13, tok::INK, Anchor::Start);
        stage(&mut s, xs[0], y1, w, "logits z", &vec_str(&big, 0), false);
        s.rect_dashed(xs[1], y1, w, 44.0, tok::GRID);
        s.text(xs[1] + w / 2.0, y1 + 26.0, "(no shift)", 11, tok::MUTED, Anchor::Middle);
        let exps_txt = format!("[{}]", n_exps.iter().map(|e| num(*e, 0)).collect::<Vec<_>>().join(", "));
        stage(&mut s, xs[2], y1, w, "exp", &exps_txt, true);
        stage(&mut s, xs[3], y1, w, "Σ", &num(n_sum, 0), true);
        stage(&mut s, xs[4], y1, w + 20.0, "÷", &format!("∞/∞ = {}", num(n_probs[0], 0)), true);
        s.text(xs[2] + w / 2.0, y1 + 58.0, "exp(709.8) is already ∞ in f64", 11, tok::S2, Anchor::Middle);
        for i in [0usize, 2, 3] {
            let x_from = xs[i] + w;
            let x_to = if i == 0 { xs[2] } else { xs[i + 1] };
            s.arrow(x_from + 2.0, y1 + 22.0, x_to - 4.0, y1 + 22.0, None);
        }
        s.text_bold(20.0, 142.0, &format!("shifted: exp(z − m) / Σ exp(z − m), m = max(z) = {}", numt(m, 0)), 13, tok::INK, Anchor::Start);
        let logits_label = match v {
            Variant::Guide => vec_str(&big, 0),
            Variant::Handbook => vec_str(&big, 0),
        };
        stage(&mut s, xs[0], y2, w, "logits z", &logits_label, false);
        if v == Variant::Handbook {
            s.text(xs[0] + w / 2.0 + 12.0, y2 + 58.0, "or [1, 5, 2] − 5: the same", 11, tok::INK2, Anchor::Middle);
        }
        stage(&mut s, xs[1], y2, w, "− m", &vec_str(&shifted, 0), false);
        let exps_s = format!("[{}]", exps.iter().map(|e| numt(*e, 4)).collect::<Vec<_>>().join(", "));
        stage(&mut s, xs[2], y2, w, "exp", &exps_s, false);
        stage(&mut s, xs[3], y2, w, "Σ", &num(sum, 4), false);
        stage(&mut s, xs[4], y2, w + 20.0, "÷", &vec_str(&probs, 4), false);
        for i in 0..4 {
            s.arrow(xs[i] + w + 2.0, y2 + 22.0, xs[i + 1] - 4.0, y2 + 22.0, None);
        }
        s.text(xs[1] + w / 2.0, y2 + 58.0, "all ≤ 0", 11, tok::INK2, Anchor::Middle);
        s.text(xs[2] + w / 2.0, y2 + 58.0, "all in (0, 1]", 11, tok::INK2, Anchor::Middle);
        s.text(xs[3] + w / 2.0, y2 + 58.0, "≥ 1, never 0", 11, tok::INK2, Anchor::Middle);
        let same = match v {
            Variant::Guide => "what the textbook defines",
            Variant::Handbook => "same result for both inputs",
        };
        s.text(xs[4] + (w + 20.0) / 2.0, y2 + 58.0, same, 11, tok::INK2, Anchor::Middle);
        (format!("f06-softmax-shift-{}.svg", v.suffix()), s.finish())
    }

    // ---------------------------------------------------------------------------------------------
    // F07 one output is one window, multiplied and summed
    // ---------------------------------------------------------------------------------------------
    pub fn fig_conv_window(v: Variant) -> (String, String) {
        let (input, kernel): (Vec<Vec<f64>>, Vec<Vec<f64>>) = match v {
            Variant::Guide => (
                (0..4).map(|r| (0..4).map(|c| (r * 4 + c + 1) as f64).collect()).collect(),
                vec![vec![0.0, -1.0, 0.0], vec![-1.0, 5.0, -1.0], vec![0.0, -1.0, 0.0]],
            ),
            Variant::Handbook => (
                (0..3).map(|r| (0..3).map(|c| (r * 3 + c + 1) as f64).collect()).collect(),
                vec![vec![1.0, 2.0], vec![0.0, -1.0]],
            ),
        };
        let height = match v {
            Variant::Guide => 320,
            Variant::Handbook => 230,
        };
        let mut s = Svg::new(
            720,
            height,
            "One output value is the kernel laid on one window, multiplied cell by cell and summed, without flipping",
            "Input grid with the top-left window highlighted and a dashed next window, the kernel grid, the product line with every term, and the output grid with its first cell highlighted; the guide variant adds row 1 of the convolution matrix.",
        );
        let out = calc::conv_valid(&input, &kernel);
        let terms = calc::window_terms(&input, &kernel, 0, 0);
        let (kh, kw) = (kernel.len(), kernel[0].len());
        let cw = 30.0;
        let (ix, iy) = (30.0, 50.0);
        let in_rows: Vec<Vec<Cell>> = input
            .iter()
            .enumerate()
            .map(|(r, row)| {
                row.iter()
                    .enumerate()
                    .map(|(c, val)| {
                        let cl = cell(numt(*val, 0));
                        if r < kh && c < kw { cl.fill(tok::FILL2) } else { cl }
                    })
                    .collect()
            })
            .collect();
        s.cells(ix, iy, cw, &in_rows);
        s.rect_bold(ix, iy, kw as f64 * cw, kh as f64 * cw, "none", tok::S1);
        s.rect_dashed(ix + cw, iy, kw as f64 * cw, kh as f64 * cw, tok::S2);
        let in_label = match v {
            Variant::Guide => "input 4 × 4 (one channel)",
            Variant::Handbook => "input 3 × 3",
        };
        s.text(ix, iy - 12.0, in_label, 12, tok::INK, Anchor::Start);
        let iw = input[0].len() as f64 * cw;
        s.text(ix, iy + input.len() as f64 * cw + 16.0, "solid: this output's window", 11, tok::S1, Anchor::Start);
        s.text(ix, iy + input.len() as f64 * cw + 30.0, "dashed: the next output's window", 11, tok::S2, Anchor::Start);
        // kernel
        let (kx, ky) = (ix + iw + 60.0, iy + 8.0);
        s.text(kx, iy - 12.0, "kernel (not flipped)", 12, tok::INK, Anchor::Start);
        s.cells(kx, ky, cw, &grid_cells(&kernel, 0, false));
        s.text((ix + iw + kx) / 2.0, iy + kh as f64 * cw / 2.0 + 12.0, "×", 13, tok::INK2, Anchor::Middle);
        // output
        let (ox, oy) = (560.0, iy + 8.0);
        s.text(ox, iy - 12.0, &format!("output {} × {}", out.len(), out[0].len()), 12, tok::INK, Anchor::Start);
        let out_rows: Vec<Vec<Cell>> = out
            .iter()
            .enumerate()
            .map(|(r, row)| {
                row.iter()
                    .enumerate()
                    .map(|(c, val)| {
                        let cl = cell(numt(*val, 0));
                        if r == 0 && c == 0 { cl.fill(tok::FILL2) } else { cl }
                    })
                    .collect()
            })
            .collect();
        s.cells(ox, oy, cw, &out_rows);
        s.rect_bold(ox, oy, cw, cw, "none", tok::S1);
        // product line
        let py = iy + input.len() as f64 * cw + 62.0;
        let line: Vec<String> = terms.iter().map(|(w, x)| term(*w, *x)).collect();
        let total = numt(out[0][0], 0);
        let product = format!("{} = {}", line.join(" + "), total);
        s.text(360.0, py, &product, 12, tok::INK, Anchor::Middle);
        s.arrow(kx + kw as f64 * cw / 2.0, ky + kh as f64 * cw + 6.0, 300.0, py - 16.0, None);
        s.text(kx + kw as f64 * cw + 46.0, ky + kh as f64 * cw + 22.0, "multiply matching cells", 11, tok::INK2, Anchor::Start);
        let prod_end = 360.0 + svg::text_w(&product, 12) / 2.0;
        s.arrow(prod_end + 6.0, py - 10.0, ox - 4.0, oy + cw / 2.0 + 6.0, None);
        s.text((prod_end + ox) / 2.0 + 10.0, py - 30.0, "add → one output", 11, tok::INK2, Anchor::Middle);
        if v == Variant::Guide {
            // row 1 of C
            let row = calc::conv_matrix_row(&kernel, 4, 4, 0, 0);
            let cy = py + 44.0;
            s.text(30.0, cy - 10.0, "row 1 of the 4 × 16 matrix C: the nine weights at this window's pixels, zeros elsewhere", 12, tok::INK, Anchor::Start);
            let window: Vec<usize> = (0..3).flat_map(|ky| (0..3).map(move |kx| ky * 4 + kx)).collect();
            let cells_row: Vec<Cell> = row
                .iter()
                .enumerate()
                .map(|(i, val)| {
                    let cl = cell(numt(*val, 0));
                    if window.contains(&i) { cl.fill(tok::FILL2) } else { cl.ink(tok::MUTED) }
                })
                .collect();
            let cwid = 40.0;
            s.cells_wh(30.0, cy, cwid, 24.0, &vec![cells_row]);
            for i in 0..16 {
                s.text(30.0 + i as f64 * cwid + cwid / 2.0, cy + 38.0, &format!("{}", i + 1), 11, tok::MUTED, Anchor::Middle);
            }
            s.text(30.0 + 16.0 * cwid + 8.0, cy + 38.0, "pixel", 11, tok::MUTED, Anchor::Start);
        }
        (format!("f07-conv-window-{}.svg", v.suffix()), s.finish())
    }

    // ---------------------------------------------------------------------------------------------
    // F08 rows gather, columns scatter
    // ---------------------------------------------------------------------------------------------
    fn matrix_with_vectors(
        s: &mut Svg,
        x: f64,
        y: f64,
        cw: f64,
        mat: &[Vec<f64>],
        xvec: &[f64],
        yvec: &[f64],
        name_m: &str,
        name_x: &str,
        name_y: &str,
        hl_row: Option<usize>,
        hl_col: Option<usize>,
    ) -> f64 {
        let mut rows = grid_cells(mat, 0, true);
        if let Some(r) = hl_row {
            for c in rows[r].iter_mut() {
                c.fill = tok::FILL2;
            }
        }
        s.cells(x, y, cw, &rows);
        if let Some(c) = hl_col {
            s.rect_bold(x + c as f64 * cw, y, cw, mat.len() as f64 * cw, "none", tok::S2);
        }
        let mw = mat[0].len() as f64 * cw;
        s.text(x + mw / 2.0, y - 10.0, name_m, 12, tok::INK, Anchor::Middle);
        let xx = x + mw + 26.0;
        s.text(xx - 13.0, y + mat.len() as f64 * cw / 2.0 + 4.0, "·", 13, tok::INK2, Anchor::Middle);
        let xrows: Vec<Vec<Cell>> = xvec.iter().map(|v| vec![cell(numt(*v, 0))]).collect();
        s.cells(xx, y, cw, &xrows);
        s.text(xx + cw / 2.0, y - 10.0, name_x, 12, tok::INK, Anchor::Middle);
        let yx = xx + cw + 30.0;
        s.text(yx - 15.0, y + mat.len() as f64 * cw / 2.0 + 4.0, "=", 13, tok::INK2, Anchor::Middle);
        let yrows: Vec<Vec<Cell>> = yvec
            .iter()
            .enumerate()
            .map(|(i, v)| {
                let cl = cell(numt(*v, 0));
                vec![if Some(i) == hl_row { cl.fill(tok::FILL2) } else { cl }]
            })
            .collect();
        s.cells(yx, y, cw, &yrows);
        s.text(yx + cw / 2.0, y - 10.0, name_y, 12, tok::INK, Anchor::Middle);
        yx + cw
    }

    pub fn fig_conv_matrix(v: Variant) -> (String, String) {
        let mut s = Svg::new(
            720,
            240,
            "As a matrix, each row gathers the inputs one output reads and each column lists the outputs one input feeds",
            "The convolution matrix times the input vector equals the output vector, with one row shaded and one column outlined; the handbook variant adds the transposed matrix times the output, showing where two contributions add.",
        );
        let cw = 34.0;
        match v {
            Variant::Guide => {
                let h = [10.0, 1.0];
                let x = [1.0, 2.0, 3.0];
                let c = calc::toeplitz_full(3, &h);
                let y = calc::matvec(&c, &x);
                let end = matrix_with_vectors(&mut s, 60.0, 44.0, cw, &c, &x, &y, "C (4 × 3)", "x", "y = C·x", Some(1), Some(1));
                let row_txt = format!("row 2 gathers x₁ and x₂: y₂ = {}·{} + {}·{} = {}", numt(c[1][0], 0), numt(x[0], 0), numt(c[1][1], 0), numt(x[1], 0), numt(y[1], 0));
                s.text(end + 30.0, 44.0 + cw + cw / 2.0 + 4.0, &row_txt, 12, tok::INK, Anchor::Start);
                s.text(end + 30.0, 44.0 + 3.0 * cw + 4.0, "column 2: x₂ feeds y₂ and y₃, the outputs whose window holds it", 11, tok::S2, Anchor::Start);
                s.text(end + 30.0, 44.0 + 3.0 * cw + 22.0, &format!("kernel h = {}: the same two weights on every diagonal", vec_str(&h, 0)), 11, tok::INK2, Anchor::Start);
                s.text(360.0, 216.0, "Cᵀ (section 7.3) reads the same entries column by column: a column scatters one input to its outputs", 11, tok::INK2, Anchor::Middle);
            }
            Variant::Handbook => {
                let k = [1.0, 2.0];
                let x = [1.0, 2.0, 3.0];
                let a = calc::conv_matrix_1d(3, &k);
                let y = calc::matvec(&a, &x);
                let end = matrix_with_vectors(&mut s, 30.0, 50.0, cw, &a, &x, &y, "A (2 × 3)", "x", "y", Some(1), Some(1));
                s.text(30.0, 50.0 + 3.0 * cw + 22.0, &format!("row 2 gathers x₂, x₃: {}·{} + {}·{} = {}", numt(a[1][1], 0), numt(x[1], 0), numt(a[1][2], 0), numt(x[2], 0), numt(y[1], 0)), 11, tok::INK2, Anchor::Start);
                s.text(30.0, 50.0 + 3.0 * cw + 38.0, "column 2: x₂ feeds both outputs", 11, tok::S2, Anchor::Start);
                let at = calc::transpose(&a);
                let back = calc::matvec(&at, &y);
                let x2 = end + 70.0;
                let mut rows = grid_cells(&at, 0, true);
                for c in rows[1].iter_mut() {
                    c.fill = tok::FILL2;
                }
                s.cells(x2, 50.0, cw, &rows);
                s.text(x2 + cw, 40.0, "Aᵀ (3 × 2)", 12, tok::INK, Anchor::Middle);
                let yx = x2 + 2.0 * cw + 26.0;
                s.text(yx - 13.0, 50.0 + 1.5 * cw + 4.0, "·", 13, tok::INK2, Anchor::Middle);
                let yrows: Vec<Vec<Cell>> = y.iter().map(|val| vec![cell(numt(*val, 0))]).collect();
                s.cells(yx, 50.0, cw, &yrows);
                s.text(yx + cw / 2.0, 40.0, "y", 12, tok::INK, Anchor::Middle);
                let bx = yx + cw + 30.0;
                s.text(bx - 15.0, 50.0 + 1.5 * cw + 4.0, "=", 13, tok::INK2, Anchor::Middle);
                let brows: Vec<Vec<Cell>> = back
                    .iter()
                    .enumerate()
                    .map(|(i, val)| vec![if i == 1 { cell(numt(*val, 0)).fill(tok::FILL2) } else { cell(numt(*val, 0)) }])
                    .collect();
                s.cells(bx, 50.0, cw, &brows);
                s.text(bx + cw / 2.0, 40.0, "Aᵀ·y", 12, tok::INK, Anchor::Middle);
                s.text(x2, 50.0 + 3.0 * cw + 22.0, &format!("middle entry: {}·{} + {}·{} = {}: two outputs contribute", numt(at[1][0], 0), numt(y[0], 0), numt(at[1][1], 0), numt(y[1], 0), numt(back[1], 0)), 11, tok::INK2, Anchor::Start);
                s.text(x2, 50.0 + 3.0 * cw + 38.0, "the same weights, used column-wise: scatter and add", 11, tok::INK2, Anchor::Start);
                s.text(360.0, 226.0, &format!("Aᵀ·y = {} is not x = {}: the transpose reverses connections, not information", vec_str(&back, 0), vec_str(&x, 0)), 11, tok::MUTED, Anchor::Middle);
            }
        }
        (format!("f08-conv-matrix-{}.svg", v.suffix()), s.finish())
    }

    // ---------------------------------------------------------------------------------------------
    // F09 transposed convolution: uneven overlap under stride 2
    // ---------------------------------------------------------------------------------------------
    fn stamp_panel(s: &mut Svg, x: f64, y: f64, cw: f64, n: usize, k: usize, stride: usize, title: &str, labels: Option<&[Vec<String>]>, counts: &[usize]) -> f64 {
        s.text_bold(x, y, title, 13, tok::INK, Anchor::Start);
        let rh = 16.0;
        let names = ["a", "b", "c"];
        for i in 0..n {
            let ry = y + 10.0 + i as f64 * (rh + 3.0);
            let x0 = x + (i * stride) as f64 * cw;
            s.rect_bold(x0, ry, k as f64 * cw, rh, tok::FILL2, tok::S1);
            let label = match labels {
                Some(_) => format!("input {} × kernel", names[i]),
                None => format!("stamp of input {}", i + 1),
            };
            s.text(x0 + k as f64 * cw / 2.0, ry + 12.0, &label, 11, tok::INK2, Anchor::Middle);
        }
        let oy = y + 10.0 + n as f64 * (rh + 3.0) + 6.0;
        let out_len = counts.len();
        let cells_row: Vec<Cell> = (0..out_len)
            .map(|j| {
                let text = match labels {
                    Some(l) => l[0][j].clone(),
                    None => counts[j].to_string(),
                };
                let cl = cell(text);
                if counts[j] > 1 { cl.stroke(tok::S2).fill(tok::NEUTRAL) } else { cl }
            })
            .collect();
        s.cells_wh(x, oy, cw, 26.0, &vec![cells_row]);
        s.text(x + out_len as f64 * cw + 10.0, oy + 17.0, &format!("{out_len} outputs"), 11, tok::INK2, Anchor::Start);
        oy + 26.0
    }

    pub fn fig_tconv_overlap(v: Variant) -> (String, String) {
        match v {
            Variant::Guide => {
                let mut s = Svg::new(
                    720,
                    430,
                    "A transposed convolution stamps its kernel once per input, stride positions apart, and adds: uneven overlap is the checkerboard",
                    "Three panels for kernel widths 3, 4 and 2 at stride 2 with three inputs: each panel shows one stamp bar per input and the output strip with the number of stamps each position receives.",
                );
                let cw = 44.0;
                let mut y = 24.0;
                for (k, note) in [(3usize, "1, 1, 2, 1, 2, 1, 1: the pattern behind the checkerboard"), (4, "even away from the borders; the 2-D middle row printed above is 2 × these: 2 2 4 4 4 4 2 2"), (2, "every output exactly once")] {
                    let counts = calc::tconv_counts(3, k, 2);
                    let title = format!("kernel {k}, stride 2, 3 inputs");
                    let end = stamp_panel(&mut s, 30.0, y, cw, 3, k, 2, &title, None, &counts);
                    s.text(30.0, end + 16.0, note, 11, tok::INK2, Anchor::Start);
                    y = end + 36.0;
                }
                s.text(600.0, 40.0, "cells outlined in orange", 11, tok::S2, Anchor::Middle);
                s.text(600.0, 54.0, "receive two stamps", 11, tok::S2, Anchor::Middle);
                (format!("f09-tconv-overlap-{}.svg", v.suffix()), s.finish())
            }
            Variant::Handbook => {
                let mut s = Svg::new(
                    720,
                    262,
                    "Upsampling two samples at stride 2 stamps the kernel twice: the middle output collects two terms, its neighbours one",
                    "Two panels: kernel [u, v, w] at stride 2 gives five outputs whose middle entry is a·w + b·u; kernel [u, v] at stride 2 gives four outputs with one term each.",
                );
                let cw = 92.0;
                let counts3 = calc::tconv_counts(2, 3, 2);
                let syms3 = vec![vec!["a·u".to_string(), "a·v".to_string(), "a·w + b·u".to_string(), "b·v".to_string(), "b·w".to_string()]];
                let end = stamp_panel(&mut s, 30.0, 24.0, cw, 2, 3, 2, "kernel [u, v, w], stride 2, inputs a, b", Some(&syms3), &counts3);
                s.text(30.0, end + 16.0, "five outputs from (2 − 1)·2 + 3; the middle one adds two stamps (outlined)", 11, tok::INK2, Anchor::Start);
                let counts2 = calc::tconv_counts(2, 2, 2);
                let syms2 = vec![vec!["a·u".to_string(), "a·v".to_string(), "b·u".to_string(), "b·v".to_string()]];
                let end2 = stamp_panel(&mut s, 30.0, end + 40.0, cw, 2, 2, 2, "kernel [u, v], stride 2 (width divisible by the stride)", Some(&syms2), &counts2);
                s.text(30.0, end2 + 16.0, "four outputs, one term each: no uneven overlap", 11, tok::INK2, Anchor::Start);
                (format!("f09-tconv-overlap-{}.svg", v.suffix()), s.finish())
            }
        }
    }

    // ---------------------------------------------------------------------------------------------
    // F10 a 1×1 convolution is a Linear at every pixel
    // ---------------------------------------------------------------------------------------------
    fn planes(s: &mut Svg, x: f64, y: f64, w: f64, h: f64, n: usize, mark: (f64, f64), label: &str) {
        for i in (0..n).rev() {
            let off = i as f64 * 9.0;
            s.rect(x + off, y - off, w, h, tok::SURFACE, Some(tok::AXIS));
        }
        s.rect_bold(x + mark.0, y + mark.1, 12.0, 12.0, tok::FILL2, tok::S1);
        s.text(x + w / 2.0, y + h + 18.0, label, 11, tok::INK2, Anchor::Middle);
    }

    pub fn fig_pointwise(v: Variant) -> (String, String) {
        let mut s = Svg::new(
            720,
            260,
            "A 1×1 convolution reads one pixel at a time but all of its channels, and applies the same matrix at every pixel",
            "Input channel planes with one pixel marked, that pixel's channel vector, the shared weight matrix, the output vector and the output planes with the same pixel marked; height and width are unchanged.",
        );
        let (pw, ph) = (110.0, 90.0);
        let mark = (40.0, 34.0);
        let (in_label, out_label) = match v {
            Variant::Guide => ("C_in = 64 channels, 32 × 32", "C_out = 128 channels, 32 × 32"),
            Variant::Handbook => ("3 channels R, G, B", "2 output channels"),
        };
        planes(&mut s, 30.0, 70.0, pw, ph, 3, mark, in_label);
        planes(&mut s, 560.0, 70.0, pw, ph, 3, mark, out_label);
        // pixel vector
        let (vx, vy, cw) = (215.0, 60.0, 34.0);
        s.arrow(30.0 + mark.0 + 12.0, 70.0 + mark.1 + 6.0, vx - 6.0, vy + 8.0, None);
        s.text(150.0, 58.0, "one pixel,", 11, tok::INK2, Anchor::Middle);
        s.text(150.0, 72.0, "all channels", 11, tok::INK2, Anchor::Middle);
        match v {
            Variant::Guide => {
                let rows: Vec<Vec<Cell>> = vec![vec![cell("c₁")], vec![cell("c₂")], vec![cell("⋮").stroke(tok::SURFACE)], vec![cell("c₆₄")]];
                s.cells(vx, vy, cw, &rows);
                s.text(vx + cw / 2.0, vy + 4.0 * cw + 16.0, "(64,)", 11, tok::INK2, Anchor::Middle);
                let (mx, my, mw, mh) = (300.0, 60.0, 190.0, 136.0);
                s.rect(mx, my, mw, mh, tok::FILL1, Some(tok::S1));
                s.text(mx + mw / 2.0, my + 40.0, "weight (128, 64, 1, 1)", 12, tok::INK, Anchor::Middle);
                s.text(mx + mw / 2.0, my + 60.0, "= a (128, 64) matrix", 12, tok::INK, Anchor::Middle);
                s.text(mx + mw / 2.0, my + 84.0, "the same matrix", 11, tok::INK2, Anchor::Middle);
                s.text(mx + mw / 2.0, my + 98.0, "at every pixel", 11, tok::INK2, Anchor::Middle);
                s.text(mx + mw / 2.0, my + 120.0, "+ bias (128,)", 11, tok::INK2, Anchor::Middle);
                s.arrow(vx + cw + 4.0, vy + 2.0 * cw, mx - 4.0, vy + 2.0 * cw, None);
                s.arrow(mx + mw + 4.0, vy + 2.0 * cw, 556.0, 70.0 + mark.1 + 6.0, None);
                s.text(360.0, 226.0, &format!("(1, 64, 32, 32) → (1, 128, 32, 32): the map stays 32 × 32; parameters 128·64 + 128 = {}", svg::thousands(128 * 64 + 128)), 12, tok::INK, Anchor::Middle);
                s.text(360.0, 244.0, "a 3×3 kernel would also read the eight neighbours; a 1×1 never does", 11, tok::INK2, Anchor::Middle);
            }
            Variant::Handbook => {
                let vector = [8.0, 2.0, 1.0];
                let matrix = vec![vec![0.5, 0.25, 0.25], vec![1.0, -1.0, 0.0]];
                let result = calc::pointwise(&vector, &matrix);
                let rows: Vec<Vec<Cell>> = vector.iter().map(|x| vec![cell(numt(*x, 0)).fill(tok::FILL2)]).collect();
                s.cells(vx, vy, cw, &rows);
                s.text(vx + cw / 2.0, vy + 3.0 * cw + 16.0, "[R, G, B]", 11, tok::INK2, Anchor::Middle);
                let mw = grid_w(&matrix, 2, 40.0);
                let (mx, my) = (300.0, 60.0);
                s.cells_wh(mx, my, mw, cw, &grid_cells(&matrix, 2, false));
                s.text(mx + 1.5 * mw, my - 10.0, "the same 2 × 3 matrix at every location", 12, tok::INK, Anchor::Middle);
                s.arrow(vx + cw + 4.0, vy + 1.5 * cw, mx - 4.0, vy + 1.5 * cw, None);
                let rx = mx + 3.0 * mw + 40.0;
                let rrows: Vec<Vec<Cell>> = result.iter().map(|x| vec![cell(numt(*x, 2)).fill(tok::FILL2)]).collect();
                s.cells_wh(rx, my, 46.0, cw, &rrows);
                s.text(rx - 18.0, my + cw + 4.0, "=", 13, tok::INK2, Anchor::Middle);
                s.arrow(rx + 50.0, my + cw, 556.0, 70.0 + mark.1 + 6.0, None);
                let l1 = format!("{} = {}", [term(matrix[0][0], vector[0]), term(matrix[0][1], vector[1]), term(matrix[0][2], vector[2])].join(" + "), numt(result[0], 2));
                let l2 = format!("{} = {}", [term(matrix[1][0], vector[0]), term(matrix[1][1], vector[1]), term(matrix[1][2], vector[2])].join(" + "), numt(result[1], 0));
                s.text(mx, my + cw * 2.0 + 30.0, &l1, 11, tok::INK2, Anchor::Start);
                s.text(mx, my + cw * 2.0 + 46.0, &l2, 11, tok::INK2, Anchor::Start);
                let second = calc::pointwise(&[1.0, 6.0, 3.0], &matrix);
                s.text(360.0, 236.0, &format!("another location [1, 6, 3] through the same matrix → {}; no neighbouring pixel is consulted", vec_str(&second, 2)), 11, tok::INK2, Anchor::Middle);
            }
        }
        (format!("f10-pointwise-{}.svg", v.suffix()), s.finish())
    }

    // ---------------------------------------------------------------------------------------------
    // F11 grouped and depthwise wiring
    // ---------------------------------------------------------------------------------------------
    fn wiring(s: &mut Svg, x: f64, y: f64, n_in: usize, n_out: usize, groups: usize, colours: &[&'static str], height: f64) {
        let (xi, xf, xo) = (x + 20.0, x + 110.0, x + 200.0);
        let pitch_in = height / n_in as f64;
        let pitch_out = height / n_out as f64;
        let in_y = |i: usize| y + pitch_in * (i as f64 + 0.5);
        let out_y = |o: usize| y + pitch_out * (o as f64 + 0.5);
        let ins_per_group = n_in / groups;
        let outs_per_group = n_out / groups;
        for o in 0..n_out {
            let g = o / outs_per_group;
            let colour = colours[g.min(colours.len() - 1)];
            for i in g * ins_per_group..(g + 1) * ins_per_group {
                s.line(xi, in_y(i), xf, out_y(o), colour, 1.0);
            }
            s.rect(xf - 8.0, out_y(o) - 6.0, 16.0, 12.0, tok::SURFACE, Some(colour));
            s.line(xf + 8.0, out_y(o), xo - 5.0, out_y(o), colour, 1.0);
            s.dot(xo, out_y(o), 4.0, colour, None);
        }
        for i in 0..n_in {
            let g = i / ins_per_group;
            s.dot(xi, in_y(i), 4.0, colours[g.min(colours.len() - 1)], None);
        }
        s.text(xi, y - 8.0, "in", 11, tok::MUTED, Anchor::Middle);
        s.text(xf, y - 8.0, "filters", 11, tok::MUTED, Anchor::Middle);
        s.text(xo, y - 8.0, "out", 11, tok::MUTED, Anchor::Middle);
    }

    pub fn fig_groups(v: Variant) -> (String, String) {
        let mut s = Svg::new(
            720,
            280,
            "Groups decide which input channels a filter can read; depthwise means one channel each, and only the 1×1 that follows mixes again",
            "Three wiring panels: every input to every filter; two groups with separate colours; one channel per filter followed by a pointwise stage. The weight shape is written under each panel.",
        );
        let titles: [&str; 3] = match v {
            Variant::Guide => ["groups = 1", "groups = 2", "groups = 4 (depthwise)"],
            Variant::Handbook => ["g = 1: every input to every filter", "g = 2: two isolated groups", "depthwise, multiplier 2"],
        };
        let shapes: [String; 3] = match v {
            Variant::Guide => ["weight (4, 4, k, k)".to_string(), "weight (4, 2, k, k)".to_string(), "weight (4, 1, k, k)".to_string()],
            Variant::Handbook => [
                format!("[12,8,3,3] = {} weights", 12 * 8 * 3 * 3),
                format!("[12,4,3,3] = {} weights", 12 * 4 * 3 * 3),
                format!("[6,1,3,3] = {} weights", 6 * 3 * 3),
            ],
        };
        let px = [20.0, 250.0, 480.0];
        for (p, x) in px.iter().enumerate() {
            s.text_bold(*x + 110.0, 30.0, titles[p], 13, tok::INK, Anchor::Middle);
            match (v, p) {
                (Variant::Guide, 0) => wiring(&mut s, *x, 60.0, 4, 4, 1, &[tok::S1], 130.0),
                (Variant::Guide, 1) => wiring(&mut s, *x, 60.0, 4, 4, 2, &[tok::S1, tok::S2], 130.0),
                (Variant::Guide, _) => wiring(&mut s, *x, 60.0, 4, 4, 4, &[tok::S1, tok::S1, tok::S1, tok::S1], 130.0),
                (Variant::Handbook, 0) => wiring(&mut s, *x, 60.0, 8, 12, 1, &[tok::S1], 130.0),
                (Variant::Handbook, 1) => wiring(&mut s, *x, 60.0, 8, 12, 2, &[tok::S1, tok::S2], 130.0),
                (Variant::Handbook, _) => wiring(&mut s, *x, 60.0, 3, 6, 3, &[tok::S1, tok::S1, tok::S1], 130.0),
            }
            s.text(*x + 110.0, 214.0, &shapes[p], 12, tok::INK, Anchor::Middle);
        }
        let sub: [&str; 3] = match v {
            Variant::Guide => ["each filter reads all 4 channels", "each filter reads its own pair", "each filter reads one channel"],
            Variant::Handbook => ["8 → 12: all channels meet", "0..3 → outputs 0..5, 4..7 → 6..11", "3 channels, 2 filters each"],
        };
        for (p, x) in px.iter().enumerate() {
            s.text(*x + 110.0, 232.0, sub[p], 11, tok::INK2, Anchor::Middle);
        }
        let tail = match v {
            Variant::Guide => "after a depthwise layer the channels have not met: the pointwise 1×1, weight (M, 4, 1, 1), is where they mix",
            Variant::Handbook => "channels in different groups never meet; a later 1×1 layer (or a channel shuffle) is what mixes them again",
        };
        s.text(360.0, 262.0, tail, 11, tok::INK2, Anchor::Middle);
        (format!("f11-groups-{}.svg", v.suffix()), s.finish())
    }

    // ---------------------------------------------------------------------------------------------
    // F12 dilation spaces the weights; equal dilations leave holes (shared)
    // ---------------------------------------------------------------------------------------------
    pub fn fig_dilation() -> (String, String) {
        let mut s = Svg::new(
            720,
            300,
            "Dilation spaces a kernel's weights apart; stacked equal dilations reach only multiples of d and leave holes, while 1, 2, 5 cover every offset",
            "A 3-weight kernel with dilation 2 drawn on five positions, then two panels of offset number lines showing which input positions one output reads after each of three layers: dilations 4, 4, 4 leave holes; dilations 1, 2, 5 do not.",
        );
        // top strip: k_eff
        let cw = 24.0;
        let kx = 40.0;
        let ky = 26.0;
        let taps = ["w₁", ".", "w₂", ".", "w₃"];
        let row: Vec<Cell> = taps
            .iter()
            .map(|t| if *t == "." { cell("·").ink(tok::MUTED) } else { cell(*t).fill(tok::FILL2) })
            .collect();
        s.cells(kx, ky, cw, &vec![row]);
        let (k, d) = (3usize, 2usize);
        s.text(kx + 5.0 * cw + 14.0, ky + cw / 2.0 + 4.0, &format!("{k} weights, dilation {d}: k_eff = {d}·({k} − 1) + 1 = {} positions", d * (k - 1) + 1), 12, tok::INK, Anchor::Start);
        s.text(kx + 5.0 * cw + 14.0, ky + cw / 2.0 + 20.0, "the two skipped positions between the weights are not read by this layer", 11, tok::INK2, Anchor::Start);
        // panels
        let panel = |s: &mut Svg, x0: f64, dil: &[i64], title: &str| {
            let rows = calc::offset_rows(dil);
            let reach = 12i64;
            let step = 10.0;
            let axis_y = 100.0;
            s.text_bold(x0, 74.0, title, 13, tok::INK, Anchor::Start);
            let xof = |o: i64| x0 + (o + reach) as f64 * step;
            s.line(xof(-reach), axis_y, xof(reach), axis_y, tok::AXIS, 1.0);
            for o in -reach..=reach {
                let tick = if o % 4 == 0 { 4.0 } else { 2.0 };
                s.line(xof(o), axis_y - tick, xof(o), axis_y + tick, tok::AXIS, 1.0);
                if o % 4 == 0 {
                    s.text(xof(o), axis_y - 8.0, &o.to_string(), 11, tok::MUTED, Anchor::Middle);
                }
            }
            s.text(x0 - 4.0, axis_y + 4.0, "offset", 11, tok::MUTED, Anchor::End);
            let last = rows.len() - 1;
            for (li, set) in rows.iter().enumerate() {
                let y = axis_y + 26.0 + li as f64 * 30.0;
                let row_label = if x0 < 300.0 { format!("after layer {} (d = {})", li + 1, dil[li]) } else { format!("d = {}", dil[li]) };
                s.text(x0 - 4.0, y + 4.0, &row_label, 11, tok::INK2, Anchor::End);
                s.line(xof(-reach), y, xof(reach), y, tok::GRID, 1.0);
                let span_min = *set.first().unwrap_or(&0);
                let span_max = *set.last().unwrap_or(&0);
                if li == last {
                    for o in span_min..=span_max {
                        if !set.contains(&o) {
                            s.dot(xof(o), y, 4.0, tok::SURFACE, Some(tok::S2));
                        }
                    }
                }
                for o in set {
                    s.dot(xof(*o), y, 4.0, tok::S1, None);
                }
            }
            let set = &rows[last];
            let span = (set.last().unwrap() - set.first().unwrap() + 1) as usize;
            let summary = format!("reads {} of {} positions in its span", set.len(), span);
            s.text(x0 + 12.0 * step, axis_y + 26.0 + 3.0 * 30.0 + 2.0, &summary, 12, tok::INK, Anchor::Middle);
        };
        panel(&mut s, 140.0, &[4, 4, 4], "d = 4, 4, 4: holes (hollow orange)");
        panel(&mut s, 450.0, &[1, 2, 5], "d = 1, 2, 5: every offset reached");
        s.line(420.0, 78.0, 420.0, 250.0, tok::GRID, 1.0);
        s.text(360.0, 280.0, "the dots are the input positions one output unit reads; a neighbouring unit reads a shifted set", 11, tok::INK2, Anchor::Middle);
        ("f12-dilation-holes.svg".to_string(), s.finish())
    }

    // ---------------------------------------------------------------------------------------------
    // F13 receptive-field growth (the one chart)
    // ---------------------------------------------------------------------------------------------
    pub fn fig_rf(v: Variant) -> (String, String) {
        let mut s = Svg::new(
            720,
            320,
            "The receptive field grows by (k − 1)·d·jump per layer: linear for a plain stack, doubling with doubled dilations, multiplied by every stride",
            "A line chart of receptive field against layer index for three stacks, each series direct-labelled with its final value.",
        );
        let series: Vec<(&str, Vec<usize>, &'static str)> = match v {
            Variant::Guide => vec![
                ("plain 3×3", calc::receptive_fields(&[(3, 1, 1), (3, 1, 1), (3, 1, 1)]), tok::S1),
                ("dilations 1, 2, 4, 8, 16", calc::receptive_fields(&[(3, 1, 1), (3, 2, 1), (3, 4, 1), (3, 8, 1), (3, 16, 1)]), tok::S2),
                ("strides 1, 2, 2, 2", calc::receptive_fields(&[(3, 1, 1), (3, 1, 2), (3, 1, 2), (3, 1, 2)]), tok::S3),
            ],
            Variant::Handbook => vec![
                ("dilations 1, 2, 4, 8, 16", calc::receptive_fields(&[(3, 1, 1), (3, 2, 1), (3, 4, 1), (3, 8, 1), (3, 16, 1)]), tok::S1),
                ("d = 1, 2, 4, 8, stride 2 in layer 2", calc::receptive_fields(&[(3, 1, 1), (3, 2, 2), (3, 4, 1), (3, 8, 1)]), tok::S2),
                ("dilation 2 repeated", calc::receptive_fields(&[(3, 2, 1), (3, 2, 1), (3, 2, 1)]), tok::S3),
            ],
        };
        let (x0, x1, y0, y1) = (70.0, 560.0, 262.0, 50.0);
        let xs = |layer: usize| x0 + (layer as f64 - 1.0) / 4.0 * (x1 - x0);
        let ys = |rf: f64| y0 - rf / 65.0 * (y0 - y1);
        // grid and axes
        for g in [10.0, 20.0, 30.0, 40.0, 50.0, 60.0] {
            s.line(x0, ys(g), x1, ys(g), tok::GRID, 1.0);
            s.text(x0 - 8.0, ys(g) + 4.0, &numt(g, 0), 11, tok::MUTED, Anchor::End);
        }
        s.line(x0, y0, x1, y0, tok::AXIS, 1.0);
        s.line(x0, y0, x0, y1, tok::AXIS, 1.0);
        for layer in 1..=5 {
            s.text(xs(layer), y0 + 16.0, &layer.to_string(), 11, tok::MUTED, Anchor::Middle);
        }
        s.text((x0 + x1) / 2.0, y0 + 34.0, "layer", 11, tok::INK2, Anchor::Middle);
        s.text(x0 - 8.0, y1 - 6.0, "receptive field (input pixels)", 11, tok::INK2, Anchor::Start);
        // legend row
        let mut lx = 70.0;
        for (name, _, colour) in &series {
            s.line(lx, 18.0, lx + 18.0, 18.0, colour, 2.0);
            s.dot(lx + 9.0, 18.0, 4.0, colour, None);
            s.text(lx + 24.0, 22.0, name, 11, tok::INK2, Anchor::Start);
            lx += 24.0 + svg::text_w(name, 11) + 26.0;
        }
        // series
        for (_, values, colour) in &series {
            let mut d = String::new();
            for (i, rf) in values.iter().enumerate() {
                let (px, py) = (xs(i + 1), ys(*rf as f64));
                if i == 0 {
                    d.push_str(&format!("M{} {}", svg::c(px), svg::c(py)));
                } else {
                    d.push_str(&format!(" L{} {}", svg::c(px), svg::c(py)));
                }
            }
            s.path(&d, colour, 2.0);
            for (i, rf) in values.iter().enumerate() {
                s.dot(xs(i + 1), ys(*rf as f64), 4.0, colour, Some(tok::SURFACE));
            }
            let last = values.len();
            let value_list = values.iter().map(|r| r.to_string()).collect::<Vec<_>>().join(", ");
            let dy = if last < 5 { 16.0 } else { 4.0 };
            s.text(xs(last) + 10.0, ys(*values.last().unwrap() as f64) + dy, &value_list, 11, tok::INK, Anchor::Start);
        }
        ("f13-receptive-field-".to_string() + v.suffix() + ".svg", s.finish())
    }

    // ---------------------------------------------------------------------------------------------
    // F14 which cells share a statistic (shared)
    // ---------------------------------------------------------------------------------------------
    pub fn fig_norm_axes() -> (String, String) {
        let mut s = Svg::new(
            720,
            260,
            "BatchNorm, LayerNorm, GroupNorm and InstanceNorm differ only in which cells are averaged together",
            "Four copies of a block of two images, each a grid of four channels by six positions; the cells that share one mean and variance are shaded: one channel across both images, one position of one image, a channel group of one image, one channel of one image.",
        );
        let (c, hw, cw) = (4usize, 6usize, 15.0);
        let block = |s: &mut Svg, x: f64, y: f64, shade: &dyn Fn(usize, usize, usize) -> bool, name: &str, note: &[&str]| {
            // back image (n = 1) offset up-right, front image (n = 0)
            for n in [1usize, 0] {
                let off = if n == 1 { 8.0 } else { 0.0 };
                let rows: Vec<Vec<Cell>> = (0..c)
                    .map(|ch| {
                        (0..hw)
                            .map(|p| if shade(n, ch, p) { cell("").fill(tok::FILL3).stroke(tok::S1) } else { cell("").stroke(tok::GRID) })
                            .collect()
                    })
                    .collect();
                s.cells(x + off, y - off, cw, &rows);
            }
            s.text_bold(x + hw as f64 * cw / 2.0 + 4.0, y - 22.0, name, 13, tok::INK, Anchor::Middle);
            for (i, line) in note.iter().enumerate() {
                s.text(x + hw as f64 * cw / 2.0 + 4.0, y + c as f64 * cw + 18.0 + i as f64 * 14.0, line, 11, tok::INK2, Anchor::Middle);
            }
        };
        let y = 70.0;
        block(&mut s, 30.0, y, &|_, ch, _| ch == 0, "BatchNorm", &["one statistic per channel,", "over all N·H·W cells", "(both images)"]);
        block(&mut s, 205.0, y, &|n, _, p| n == 0 && p == 2, "LayerNorm", &["one per token: its C features", "(the transformer use; a CNN", "LayerNorm over C,H,W pools differently)"]);
        block(&mut s, 380.0, y, &|n, ch, _| n == 0 && ch < 2, "GroupNorm", &["a group of channels", "of one image,", "all positions"]);
        block(&mut s, 555.0, y, &|n, ch, _| n == 0 && ch == 0, "InstanceNorm", &["one channel", "of one image", ""]);
        // axis labels on the first block
        s.text(22.0, y + c as f64 * cw / 2.0 + 4.0, "C", 11, tok::MUTED, Anchor::End);
        s.text(30.0 + hw as f64 * cw / 2.0, y + c as f64 * cw + 6.0, "H·W →", 11, tok::MUTED, Anchor::Middle);
        s.text(30.0 + hw as f64 * cw + 14.0, y - 4.0, "N = 2 images", 11, tok::MUTED, Anchor::Start);
        s.text(360.0, 236.0, "the shaded cells are averaged together; a learned scale and shift per channel (or per feature) follow in every case", 11, tok::INK2, Anchor::Middle);
        ("f14-norm-axes.svg".to_string(), s.finish())
    }

    // ---------------------------------------------------------------------------------------------
    // F15 a fixed BatchNorm folds into the convolution
    // ---------------------------------------------------------------------------------------------
    pub fn fig_bn_fold(v: Variant) -> (String, String) {
        let mut s = Svg::new(
            720,
            260,
            "With fixed statistics BatchNorm is a per-channel scale and shift, so it folds into the convolution before it and both routes give the same output",
            "Two lanes: convolution then BatchNorm, and the fused convolution with scaled weights and a new bias; a brace joins the two equal outputs.",
        );
        let (y1, y2) = (46.0, 150.0);
        match v {
            Variant::Handbook => {
                let (w, b, mu, var, eps, gamma, beta) = (2.0, 1.0, 3.0, 3.0, 1.0, 4.0, -2.0);
                let (alpha, wf, bf) = calc::bn_fold(w, b, mu, var, eps, gamma, beta);
                let x_in = 5.0;
                let z = w * x_in + b;
                let y_bn = gamma * (z - mu) / (var + eps).sqrt() + beta;
                let y_fused = wf * x_in + bf;
                s.text_bold(20.0, 30.0, "conv, then BN (inference statistics)", 13, tok::INK, Anchor::Start);
                s.labelled_box(20.0, y1, 70.0, 44.0, &format!("x = {}", numt(x_in, 0)), tok::SURFACE, tok::AXIS);
                s.labelled_box2(120.0, y1, 140.0, 44.0, &format!("conv: z = {}x + {}", numt(w, 0), numt(b, 0)), &format!("z = {}", numt(z, 0)), tok::SURFACE, tok::AXIS);
                s.labelled_box2(290.0, y1, 310.0, 44.0, &format!("BN: mu {}, v {}, epsilon {}, gamma {}, beta {}", numt(mu, 0), numt(var, 0), numt(eps, 0), numt(gamma, 0), numt(beta, 0)), &format!("y = {}·({} − {})/√({} + {}) + ({}) = {}", numt(gamma, 0), numt(z, 0), numt(mu, 0), numt(var, 0), numt(eps, 0), numt(beta, 0), numt(y_bn, 0)), tok::SURFACE, tok::AXIS);
                s.labelled_box(610.0, y1, 80.0, 44.0, &format!("y = {}", numt(y_bn, 0)), tok::FILL2, tok::S1);
                s.arrow(90.0, y1 + 22.0, 116.0, y1 + 22.0, None);
                s.arrow(260.0, y1 + 22.0, 286.0, y1 + 22.0, None);
                s.arrow(600.0, y1 + 22.0, 606.0, y1 + 22.0, None);
                s.text(340.0, 108.0, &format!("a = gamma/√(v + epsilon) = {}/√({} + {}) = {}", numt(gamma, 0), numt(var, 0), numt(eps, 0), numt(alpha, 0)), 12, tok::INK, Anchor::Middle);
                s.text(340.0, 124.0, &format!("W' = a·W = {};   b' = beta + a·(b − mu) = {} + {}·({} − {}) = {}", numt(wf, 0), numt(beta, 0), numt(alpha, 0), numt(b, 0), numt(mu, 0), numt(bf, 0)), 12, tok::INK, Anchor::Middle);
                s.text_bold(20.0, y2 - 8.0, "fused: one convolution", 13, tok::INK, Anchor::Start);
                s.labelled_box(20.0, y2, 70.0, 44.0, &format!("x = {}", numt(x_in, 0)), tok::SURFACE, tok::AXIS);
                s.labelled_box2(120.0, y2, 480.0, 44.0, &format!("conv: y = {}x − {}", numt(wf, 0), numt(-bf, 0)), &format!("{}·{} − {} = {}", numt(wf, 0), numt(x_in, 0), numt(-bf, 0), numt(y_fused, 0)), tok::FILL1, tok::S1);
                s.labelled_box(610.0, y2, 80.0, 44.0, &format!("y = {}", numt(y_fused, 0)), tok::FILL2, tok::S1);
                s.arrow(90.0, y2 + 22.0, 116.0, y2 + 22.0, None);
                s.arrow(600.0, y2 + 22.0, 606.0, y2 + 22.0, None);
                s.brace_right(696.0, y1, y2 + 44.0, "");
                s.text(650.0, (y1 + y2 + 44.0) / 2.0 + 4.0, "same output", 11, tok::INK2, Anchor::Middle);
                s.text(360.0, 232.0, "keeping epsilon inside the square root is what makes the two routes agree; with √3 instead of √4 they would not", 11, tok::INK2, Anchor::Middle);
            }
            Variant::Guide => {
                s.text_bold(20.0, 30.0, "conv, then BN (inference statistics)", 13, tok::INK, Anchor::Start);
                s.labelled_box(20.0, y1, 60.0, 44.0, "x", tok::SURFACE, tok::AXIS);
                s.labelled_box2(120.0, y1, 170.0, 44.0, "conv: W (3, 2, 3, 3), b (3,)", "z = W∗x + b", tok::SURFACE, tok::AXIS);
                s.labelled_box2(330.0, y1, 240.0, 44.0, "BN: μ, σ², γ, β, each (3,)", "y = γ·(z − μ)/√(σ² + ε) + β", tok::SURFACE, tok::AXIS);
                s.labelled_box(610.0, y1, 80.0, 44.0, "y", tok::FILL2, tok::S1);
                s.arrow(80.0, y1 + 22.0, 116.0, y1 + 22.0, None);
                s.arrow(290.0, y1 + 22.0, 326.0, y1 + 22.0, None);
                s.arrow(570.0, y1 + 22.0, 606.0, y1 + 22.0, None);
                s.text(340.0, 108.0, "α = γ/√(σ² + ε), one number per output channel", 12, tok::INK, Anchor::Middle);
                s.text(340.0, 124.0, "W' = α·W with α reshaped to (3, 1, 1, 1);   b' = α·(b − μ) + β", 12, tok::INK, Anchor::Middle);
                s.text_bold(20.0, y2 - 8.0, "fused: one convolution", 13, tok::INK, Anchor::Start);
                s.labelled_box(20.0, y2, 60.0, 44.0, "x", tok::SURFACE, tok::AXIS);
                s.labelled_box2(120.0, y2, 450.0, 44.0, "conv: W' = α·W (3, 2, 3, 3), b' (3,)", "one pass over the activations instead of two", tok::FILL1, tok::S1);
                s.labelled_box(610.0, y2, 80.0, 44.0, "y", tok::FILL2, tok::S1);
                s.arrow(80.0, y2 + 22.0, 116.0, y2 + 22.0, None);
                s.arrow(570.0, y2 + 22.0, 606.0, y2 + 22.0, None);
                s.brace_right(696.0, y1, y2 + 44.0, "");
                s.text(650.0, (y1 + y2 + 44.0) / 2.0 + 4.0, "same output", 11, tok::INK2, Anchor::Middle);
                // alpha per output channel: three slabs
                let (sx, sy) = (20.0, 212.0);
                for cch in 0..3 {
                    let x = sx + cch as f64 * 96.0;
                    s.rect(x, sy, 60.0, 30.0, tok::FILL2, Some(tok::S1));
                    s.text(x + 30.0, sy + 19.0, &format!("W[{cch}] × α{}", ["₀", "₁", "₂"][cch]), 11, tok::INK, Anchor::Middle);
                }
                s.text(316.0, sy + 12.0, "α_c scales every weight of output channel c;", 11, tok::INK2, Anchor::Start);
                s.text(316.0, sy + 26.0, "a bare (3,) α would scale the kernel columns instead (12.2)", 11, tok::INK2, Anchor::Start);
                s.text(360.0, 254.0, "a convolution created without a bias gets one from the fold: b' = β − α·μ", 11, tok::INK2, Anchor::Middle);
            }
        }
        (format!("f15-bn-fold-{}.svg", v.suffix()), s.finish())
    }

    // ---------------------------------------------------------------------------------------------
    // F16 squeeze, excite, scale
    // ---------------------------------------------------------------------------------------------
    pub fn fig_se(v: Variant) -> (String, String) {
        let mut s = Svg::new(
            720,
            300,
            "Squeeze averages each channel to one number, a tiny network turns those numbers into one gate per channel, and every position of a channel is multiplied by its gate",
            "Feature maps on the left, the squeezed column, the hidden column, the gate bars with their values, and the scaled output maps on the right; each stage carries its formula beneath it; no pixel moves.",
        );
        let ex = calc::se_two_channel();
        let mid = 128.0; // the y of the arrows
        match v {
            Variant::Handbook => {
                let cw = 26.0;
                let (mx, my) = (20.0, 64.0);
                for (ch, map) in ex.maps.iter().enumerate() {
                    let y = my + ch as f64 * 74.0;
                    s.text(mx, y - 8.0, &format!("channel {}", ch + 1), 11, tok::INK2, Anchor::Start);
                    s.cells(mx, y, cw, &grid_cells(map, 0, false));
                }
                s.text(mx + cw, 36.0, "X [1, 2, 2, 2]", 12, tok::INK, Anchor::Middle);
                // squeeze column
                let sx = 116.0;
                s.arrow(mx + 2.0 * cw + 6.0, mid, sx - 6.0, mid, None);
                let srows: Vec<Vec<Cell>> = ex.averages.iter().map(|a| vec![cell(numt(*a, 0)).fill(tok::FILL2)]).collect();
                s.cells_wh(sx, mid - 30.0, 40.0, 30.0, &srows);
                s.text(sx + 20.0, 36.0, "s [1, 2]", 12, tok::INK, Anchor::Middle);
                s.text(sx + 20.0, mid + 50.0, "mean over", 11, tok::INK2, Anchor::Middle);
                s.text(sx + 20.0, mid + 64.0, "H, W", 11, tok::INK2, Anchor::Middle);
                // hidden
                let hx = 220.0;
                s.arrow(sx + 46.0, mid, hx - 6.0, mid, None);
                s.cells_wh(hx, mid - 15.0, 40.0, 30.0, &vec![vec![cell(numt(ex.hidden, 0)).fill(tok::FILL2)]]);
                s.text(hx + 20.0, 36.0, "hidden", 12, tok::INK, Anchor::Middle);
                s.text(hx + 20.0, mid + 50.0, "ReLU(W₁·s), W₁ = [1, −1]:", 11, tok::INK2, Anchor::Middle);
                s.text(hx + 20.0, mid + 64.0, &format!("ReLU({} − {}) = {}", numt(ex.averages[0], 0), numt(ex.averages[1], 0), numt(ex.hidden, 0)), 11, tok::INK2, Anchor::Middle);
                // gates
                let gx = 330.0;
                s.arrow(hx + 46.0, mid, gx - 6.0, mid, None);
                s.text(gx + 50.0, 36.0, "gates in (0, 1)", 12, tok::INK, Anchor::Middle);
                for (ch, g) in ex.gates.iter().enumerate() {
                    let y = mid - 34.0 + ch as f64 * 34.0;
                    s.rect(gx, y, 100.0, 22.0, tok::SURFACE, Some(tok::GRID));
                    s.rect(gx, y, 100.0 * g, 22.0, tok::S1, None);
                    s.text(gx + 106.0, y + 15.0, &format!("{} = σ({}ln 3)", numt(*g, 2), if ex.logits[ch] >= 0.0 { "+" } else { "−" }), 11, tok::INK2, Anchor::Start);
                }
                s.text(gx + 90.0, mid + 50.0, "σ(W₂·h), W₂ = ±ln 3", 11, tok::INK2, Anchor::Middle);
                s.text(gx + 90.0, mid + 64.0, "one gate per channel", 11, tok::INK2, Anchor::Middle);
                // outputs
                let (ox, oy) = (614.0, 64.0);
                s.arrow(gx + 200.0, mid, ox - 6.0, mid, Some("× per channel"));
                for (ch, map) in ex.scaled.iter().enumerate() {
                    let y = oy + ch as f64 * 74.0;
                    s.text(ox, y - 8.0, &format!("× {}", numt(ex.gates[ch], 2)), 11, tok::INK2, Anchor::Start);
                    s.cells(ox, y, cw, &grid_cells(map, 2, false));
                }
                s.text(ox + cw, 36.0, "Y [1, 2, 2, 2]", 12, tok::INK, Anchor::Middle);
                s.text(360.0, 276.0, "one number per channel scales every position of that channel; the gates need not sum to one", 11, tok::INK2, Anchor::Middle);
            }
            Variant::Guide => {
                let cw = 12.0;
                let (mx, my) = (20.0, 70.0);
                for ch in 0..4 {
                    let y = my + ch as f64 * 34.0;
                    let rows: Vec<Vec<Cell>> = (0..2).map(|_| (0..2).map(|_| cell("").stroke(tok::GRID)).collect()).collect();
                    s.cells(mx + 8.0 * ch as f64, y, cw, &rows);
                }
                s.text(mx + 30.0, my + 4.0 * 34.0 + 10.0, "… 64 channels", 11, tok::INK2, Anchor::Middle);
                s.text(mx + 30.0, 36.0, "X (B, 64, H, W)", 12, tok::INK, Anchor::Middle);
                let col = |s: &mut Svg, x: f64, n: usize, label: &str| {
                    let mut rows: Vec<Vec<Cell>> = (0..n).map(|_| vec![cell("").fill(tok::FILL2)]).collect();
                    rows.push(vec![cell("⋮").stroke(tok::SURFACE)]);
                    s.cells_wh(x, mid - 50.0, 30.0, 20.0, &rows);
                    s.text(x + 15.0, 36.0, label, 12, tok::INK, Anchor::Middle);
                };
                let sx = 130.0;
                s.arrow(mx + 70.0, mid, sx - 6.0, mid, None);
                col(&mut s, sx, 4, "s (B, 64)");
                s.text(sx + 15.0, mid + 64.0, "mean over", 11, tok::INK2, Anchor::Middle);
                s.text(sx + 15.0, mid + 78.0, "H, W", 11, tok::INK2, Anchor::Middle);
                let hx = 230.0;
                s.arrow(sx + 36.0, mid, hx - 6.0, mid, None);
                let hrows: Vec<Vec<Cell>> = (0..4).map(|_| vec![cell("").fill(tok::FILL2)]).collect();
                s.cells_wh(hx, mid - 40.0, 30.0, 20.0, &hrows);
                s.text(hx + 15.0, 36.0, "(B, 4)", 12, tok::INK, Anchor::Middle);
                s.text(hx + 15.0, mid + 64.0, "ReLU(W₁·s),", 11, tok::INK2, Anchor::Middle);
                s.text(hx + 15.0, mid + 78.0, "W₁ (4, 64)", 11, tok::INK2, Anchor::Middle);
                let gx = 330.0;
                s.arrow(hx + 36.0, mid, gx - 6.0, mid, None);
                s.text(gx + 60.0, 36.0, "64 gates in (0, 1)", 12, tok::INK, Anchor::Middle);
                let lengths = [0.62, 0.15, 0.88, 0.40, 0.71, 0.27, 0.95, 0.52];
                for (i, l) in lengths.iter().enumerate() {
                    let y = 50.0 + i as f64 * 18.0;
                    s.rect(gx, y, 120.0, 12.0, tok::SURFACE, Some(tok::GRID));
                    s.rect(gx, y, 120.0 * l, 12.0, tok::S1, None);
                }
                s.text(gx + 60.0, 50.0 + 8.0 * 18.0 + 8.0, "⋮ (lengths illustrative)", 11, tok::MUTED, Anchor::Middle);
                s.text(gx + 60.0, mid + 92.0, "σ(W₂·h), W₂ (64, 4)", 11, tok::INK2, Anchor::Middle);
                let (ox, oy) = (600.0, 70.0);
                s.arrow(gx + 130.0, mid, ox - 6.0, mid, None);
                s.text((gx + 130.0 + ox) / 2.0, mid - 20.0, "reshape (B, 64, 1, 1),", 11, tok::INK2, Anchor::Middle);
                s.text((gx + 130.0 + ox) / 2.0, mid - 8.0, "broadcast_mul", 11, tok::INK2, Anchor::Middle);
                for ch in 0..4 {
                    let y = oy + ch as f64 * 34.0;
                    let rows: Vec<Vec<Cell>> = (0..2).map(|_| (0..2).map(|_| cell("").stroke(tok::GRID).fill(tok::FILL1)).collect()).collect();
                    s.cells(ox + 8.0 * ch as f64, y, cw, &rows);
                }
                s.text(ox + 30.0, oy + 4.0 * 34.0 + 10.0, "… each × its gate", 11, tok::INK2, Anchor::Middle);
                s.text(ox + 30.0, 36.0, "Y (B, 64, H, W)", 12, tok::INK, Anchor::Middle);
                s.text(360.0, 262.0, &format!("{} parameters = 2·64·4 weights + 4 + 64 biases; one number per channel scales every pixel of that channel", 2 * 64 * 4 + 4 + 64), 11, tok::INK2, Anchor::Middle);
                s.text(360.0, 280.0, "attention over channels: no tokens, no pairwise scores, only a learned gate per channel", 11, tok::INK2, Anchor::Middle);
            }
        }
        (format!("f16-se-gates-{}.svg", v.suffix()), s.finish())
    }

    // ---------------------------------------------------------------------------------------------
    // F17 attention: scores, weights, blend
    // ---------------------------------------------------------------------------------------------
    pub fn fig_attention(v: Variant) -> (String, String) {
        let mut s = Svg::new(
            720,
            300,
            "Q·Kᵀ/√d_k scores every query against every key, softmax over each row gives weights that sum to 1, and each output row blends the value rows",
            "Left to right: Q, K transposed, the score matrix, the scaled scores, the weight heatmap with row sums, V and the output; one row is outlined through every stage and its blend is written out.",
        );
        let x = vec![vec![1.0, 0.0], vec![0.0, 1.0], vec![1.0, 1.0]];
        let ident = vec![vec![1.0, 0.0], vec![0.0, 1.0]];
        let (a, dec, hl) = match v {
            Variant::Guide => (calc::attention(&x, &ident, &ident, &ident, None), 3usize, 0usize),
            Variant::Handbook => {
                let p_k = vec![vec![0.0, 1.0], vec![1.0, 0.0]];
                let p_v = vec![vec![2.0, 0.0], vec![0.0, 1.0]];
                (calc::attention(&x, &ident, &p_k, &p_v, None), 6usize, 0usize)
            }
        };
        let ch = 26.0;
        let y = 70.0;
        let mut cx = 16.0;
        let put = |s: &mut Svg, cx: &mut f64, mat: &[Vec<f64>], d: usize, name: &str, heat_on: bool, min_w: f64| -> f64 {
            let fmt = |val: f64| if d == 0 { numt(val, 0) } else { num(val, d) };
            let longest = mat.iter().flatten().map(|val| fmt(*val).chars().count()).max().unwrap_or(1) as f64;
            let w = (0.62 * 11.0 * longest + 8.0).max(min_w).ceil();
            let max = mat.iter().flatten().copied().fold(0.0, f64::max);
            let rows: Vec<Vec<Cell>> = mat
                .iter()
                .map(|r| r.iter().map(|val| {
                    let c0 = cell(fmt(*val));
                    if heat_on { c0.fill(svg::heat(*val, max)) } else { c0 }
                }).collect())
                .collect();
            s.cells_wh(*cx, y, w, ch, &rows);
            s.text(*cx + mat[0].len() as f64 * w / 2.0, y - 10.0, name, 12, tok::INK, Anchor::Middle);
            let start = *cx;
            *cx += mat[0].len() as f64 * w;
            start
        };
        let sep = |s: &mut Svg, cx: &mut f64, sym: &str, width: f64| {
            s.text(*cx + width / 2.0, y + 1.5 * ch + 4.0, sym, 12, tok::INK2, Anchor::Middle);
            *cx += width;
        };
        let q0 = put(&mut s, &mut cx, &a.q, 0, "Q", false, 22.0);
        sep(&mut s, &mut cx, "·", 16.0);
        let kt = calc::transpose(&a.k);
        put(&mut s, &mut cx, &kt, 0, "Kᵀ", false, 22.0);
        sep(&mut s, &mut cx, "=", 16.0);
        put(&mut s, &mut cx, &a.scores, 0, "scores", false, 22.0);
        sep(&mut s, &mut cx, "÷√2 →", 40.0);
        let wx = put(&mut s, &mut cx, &a.weights, dec, "weights", true, 26.0);
        s.text(wx + 1.5 * ((0.62 * 11.0 * num(a.weights[0][0], dec).chars().count() as f64 + 8.0).max(26.0).ceil()), y + 3.0 * ch + 14.0, "softmax over each row", 11, tok::INK2, Anchor::Middle);
        // row sums
        let sums: Vec<Vec<f64>> = a.weights.iter().map(|r| vec![r.iter().sum::<f64>()]).collect();
        cx += 4.0;
        put(&mut s, &mut cx, &sums, 0, "Σ", false, 22.0);
        sep(&mut s, &mut cx, "×", 16.0);
        put(&mut s, &mut cx, &a.v, 0, "V", false, 22.0);
        sep(&mut s, &mut cx, "=", 16.0);
        let ox = put(&mut s, &mut cx, &a.out, dec, "output", false, 26.0);
        let end_x = cx;
        // highlight row hl across stages
        s.rect_bold(q0 - 3.0, y + hl as f64 * ch - 3.0, end_x - q0 + 6.0, ch + 6.0, "none", tok::S1);
        let _ = (wx, ox);
        // blend line
        let wrow = &a.weights[hl];
        let terms: Vec<String> = wrow.iter().zip(&a.v).map(|(w, vr)| format!("{}·{}", num(*w, dec), vec_str(vr, 0))).collect();
        let out_row = format!("[{}]", a.out[hl].iter().map(|x| num(*x, dec)).collect::<Vec<_>>().join(", "));
        let blend = format!("row {}: {} = {}", hl + if v == Variant::Guide { 0 } else { 1 }, terms.join(" + "), out_row);
        s.text(360.0, 196.0, &blend, 12, tok::INK, Anchor::Middle);
        let note = match v {
            Variant::Guide => "Q = K = V = X here so every number can be checked; a real layer projects X three ways (14.3)",
            Variant::Handbook => "Q = X·P_Q, K = X·P_K, V = X·P_V with P_Q = I, P_K = [[0,1],[1,0]], P_V = diag(2, 1); no mask, no dropout",
        };
        s.text(360.0, 224.0, note, 11, tok::INK2, Anchor::Middle);
        let claim = match v {
            Variant::Guide => "each output row is a weighted blend of the value rows, not a lookup; the weights of a row sum to 1",
            Variant::Handbook => "softmax runs along the last axis (over the keys); each output row is a convex blend of the value rows",
        };
        s.text(360.0, 244.0, claim, 11, tok::INK2, Anchor::Middle);
        s.text(360.0, 274.0, "darker cell = larger weight (one hue, light to dark)", 11, tok::MUTED, Anchor::Middle);
        (format!("f17-attention-flow-{}.svg", v.suffix()), s.finish())
    }

    // ---------------------------------------------------------------------------------------------
    // F18 masks: where −∞ becomes 0
    // ---------------------------------------------------------------------------------------------
    pub fn fig_masks(v: Variant) -> (String, String) {
        match v {
            Variant::Guide => {
                let mut s = Svg::new(
                    720,
                    392,
                    "A mask names the keys a query may not read: their scores become −∞ before softmax, so their weights are exactly 0 and each row still sums to 1",
                    "Three 4 by 4 grids: the causal U8 mask with 1 above the diagonal, the scores with −∞ in the forbidden cells, and the weights with 0 there; a second strip shows the padding mask zeroing the last key column.",
                );
                let n = 4usize;
                let cw = 30.0;
                let y = 56.0;
                let forbidden = |r: usize, c: usize| c > r;
                let mask_rows: Vec<Vec<Cell>> = (0..n)
                    .map(|r| (0..n).map(|c| if forbidden(r, c) { cell("1").fill(tok::NEUTRAL).stroke(tok::S2) } else { cell("0") }).collect())
                    .collect();
                let sub = ["₀", "₁", "₂", "₃"];
                let score_rows: Vec<Vec<Cell>> = (0..n)
                    .map(|r| (0..n).map(|c| if forbidden(r, c) { cell("−∞").fill(tok::NEUTRAL).stroke(tok::S2) } else { cell(format!("s{}{}", sub[r], sub[c])) }).collect())
                    .collect();
                let weight_rows: Vec<Vec<Cell>> = (0..n)
                    .map(|r| (0..n).map(|c| if forbidden(r, c) { cell("0").fill(tok::NEUTRAL).stroke(tok::S2) } else { cell(format!("w{}{}", sub[r], sub[c])) }).collect())
                    .collect();
                let xs = [40.0, 270.0, 500.0];
                let names = ["mask (U8): 1 = forbidden", "scores after where_cond", "weights after softmax"];
                for (i, rows) in [&mask_rows, &score_rows, &weight_rows].iter().enumerate() {
                    s.cells(xs[i], y, cw, rows);
                    s.text(xs[i] + 2.0 * cw, y - 10.0, names[i], 12, tok::INK, Anchor::Middle);
                }
                s.text(xs[0] + 2.0 * cw, y + 4.0 * cw + 14.0, "rows: queries; columns: keys", 11, tok::MUTED, Anchor::Middle);
                s.arrow(xs[0] + 4.0 * cw + 6.0, y + 2.0 * cw, xs[1] - 6.0, y + 2.0 * cw, Some("mask → −∞"));
                s.arrow(xs[1] + 4.0 * cw + 6.0, y + 2.0 * cw, xs[2] - 6.0, y + 2.0 * cw, Some("softmax"));
                s.rect_bold(xs[0] - 3.0, y + cw - 3.0, 4.0 * cw + 6.0, cw + 6.0, "none", tok::S1);
                s.rect_bold(xs[2] - 3.0, y + cw - 3.0, 4.0 * cw + 6.0, cw + 6.0, "none", tok::S1);
                s.text(xs[2] + 2.0 * cw, y + 4.0 * cw + 14.0, "row 1: w₁₀ + w₁₁ = 1; every row sums to 1", 11, tok::INK2, Anchor::Middle);
                // padding strip
                let y2 = 244.0;
                s.text(40.0, y2 - 10.0, "padding mask (1, 1, 1, 4) = [0, 0, 0, 1]: the last word is padding", 12, tok::INK, Anchor::Start);
                let pad: Vec<Cell> = [0, 0, 0, 1].iter().map(|m| if *m == 1 { cell("1").fill(tok::NEUTRAL).stroke(tok::S2) } else { cell("0") }).collect();
                s.cells(40.0, y2, cw, &vec![pad]);
                s.text(40.0 + 4.0 * cw + 10.0, y2 + 19.0, "broadcasts over batch, heads and queries", 11, tok::INK2, Anchor::Start);
                let wrows: Vec<Vec<Cell>> = (0..n)
                    .map(|r| (0..n).map(|c| if c == 3 { cell("0.0").fill(tok::NEUTRAL).stroke(tok::S2) } else { cell(format!("w{}{}", sub[r], sub[c])) }).collect())
                    .collect();
                s.cells(440.0, y2 - 6.0, cw, &wrows);
                s.text(440.0 + 2.0 * cw, y2 - 16.0, "nobody attends to the padded word", 11, tok::INK2, Anchor::Middle);
                s.text(360.0, 384.0, "a row of nothing but −∞ gives NaN (−∞ − (−∞)): never mask every key of a query", 11, tok::S2, Anchor::Middle);
                (format!("f18-masks-{}.svg", v.suffix()), s.finish())
            }
            Variant::Handbook => {
                let mut s = Svg::new(
                    720,
                    262,
                    "Adding the causal mask replaces forbidden scores with −∞: their weights are exactly 0 and the rest of the row renormalises to 1",
                    "Three 3 by 3 grids: the additive mask, the masked scaled scores, and the weights; row 2 is outlined and its arithmetic written beneath.",
                );
                let x = vec![vec![1.0, 0.0], vec![0.0, 1.0], vec![1.0, 1.0]];
                let ident = vec![vec![1.0, 0.0], vec![0.0, 1.0]];
                let p_k = vec![vec![0.0, 1.0], vec![1.0, 0.0]];
                let p_v = vec![vec![2.0, 0.0], vec![0.0, 1.0]];
                let mask: Vec<Vec<bool>> = (0..3).map(|r| (0..3).map(|c| c > r).collect()).collect();
                let a = calc::attention(&x, &ident, &p_k, &p_v, Some(&mask));
                let cw = 46.0;
                let y = 60.0;
                let mask_rows: Vec<Vec<Cell>> = (0..3)
                    .map(|r| (0..3).map(|c| if mask[r][c] { cell("−∞").fill(tok::NEUTRAL).stroke(tok::S2) } else { cell("0") }).collect())
                    .collect();
                let score_rows: Vec<Vec<Cell>> = a
                    .scaled
                    .iter()
                    .enumerate()
                    .map(|(r, row)| {
                        row.iter()
                            .enumerate()
                            .map(|(c, val)| {
                                if mask[r][c] {
                                    cell("−∞").fill(tok::NEUTRAL).stroke(tok::S2)
                                } else {
                                    let unit = 1.0 / 2f64.sqrt();
                                    let sym = if *val == 0.0 { "0".to_string() } else if (*val - unit).abs() < 1e-12 { "a".to_string() } else { "2a".to_string() };
                                    cell(sym)
                                }
                            })
                            .collect()
                    })
                    .collect();
                let weight_rows: Vec<Vec<Cell>> = a
                    .weights
                    .iter()
                    .enumerate()
                    .map(|(r, row)| row.iter().enumerate().map(|(c, val)| if mask[r][c] { cell("0").fill(tok::NEUTRAL).stroke(tok::S2) } else if (*val - val.round()).abs() < 1e-12 { cell(numt(*val, 0)) } else { cell(num(*val, 6)) }).collect())
                    .collect();
                let xs = [30.0, 230.0, 470.0];
                s.cells(xs[0], y, cw, &mask_rows);
                s.text(xs[0] + 1.5 * cw, y - 10.0, "additive mask M", 12, tok::INK, Anchor::Middle);
                s.cells(xs[1], y, cw, &score_rows);
                s.text(xs[1] + 1.5 * cw, y - 10.0, "S = QKᵀ/√2 + M   (a = 1/√2)", 12, tok::INK, Anchor::Middle);
                let ww = grid_w(&a.weights, 6, 46.0);
                s.cells_wh(xs[2], y, ww, cw, &weight_rows);
                s.text(xs[2] + 1.5 * ww, y - 10.0, "weights: softmax over each row", 12, tok::INK, Anchor::Middle);
                s.arrow(xs[0] + 3.0 * cw + 6.0, y + 1.5 * cw, xs[1] - 6.0, y + 1.5 * cw, None);
                s.text((xs[0] + 3.0 * cw + xs[1]) / 2.0, y + 3.0 * cw + 14.0, "add to the scores", 11, tok::INK2, Anchor::Middle);
                s.arrow(xs[1] + 3.0 * cw + 6.0, y + 1.5 * cw, xs[2] - 6.0, y + 1.5 * cw, None);
                s.text((xs[1] + 3.0 * cw + xs[2]) / 2.0, y + 3.0 * cw + 14.0, "softmax", 11, tok::INK2, Anchor::Middle);
                s.rect_bold(xs[0] - 3.0, y + cw - 3.0, 3.0 * cw + 6.0, cw + 6.0, "none", tok::S1);
                s.rect_bold(xs[2] - 3.0, y + cw - 3.0, 3.0 * ww + 6.0, cw + 6.0, "none", tok::S1);
                let row2 = &a.weights[1];
                let out2 = &a.out[1];
                let w6 = format!("[{}, {}, 0]", num(row2[0], 6), num(row2[1], 6));
                let o6 = format!("[{}, {}]", num(out2[0], 6), num(out2[1], 6));
                s.text(360.0, 226.0, &format!("row 2: [a, 0, −∞] → weights {} → output {}·{} + {}·{} = {}", w6, num(row2[0], 6), vec_str(&a.v[0], 0), num(row2[1], 6), vec_str(&a.v[1], 0), o6), 11, tok::INK, Anchor::Middle);
                s.text(360.0, 241.0, "U8 1 = blocked in this handbook's Candle fragment; PyTorch's boolean SDPA mask uses True = allowed", 11, tok::INK2, Anchor::Middle);
                s.text(360.0, 254.0, "a row of only −∞ gives NaN: never mask every key of a query", 11, tok::S2, Anchor::Middle);
                (format!("f18-masks-{}.svg", v.suffix()), s.finish())
            }
        }
    }

    // ---------------------------------------------------------------------------------------------
    // F19 heads by reshape and transpose
    // ---------------------------------------------------------------------------------------------
    pub fn fig_heads(v: Variant) -> (String, String) {
        let mut s = Svg::new(
            720,
            284,
            "reshape splits the feature axis into (heads, d_k) and transpose(1, 2) moves the head axis in front so one batched matmul runs every head",
            "A row of shape drawings: the projected tensor with head 0's columns shaded and head 1's outlined, the reshaped tensor with the columns regrouped per token, the transposed tensor as two stacked per-head blocks, and the shapes under each stage.",
        );
        let (n, d, h) = match v {
            Variant::Guide => (4usize, 8usize, 2usize),
            Variant::Handbook => (3usize, 8usize, 2usize),
        };
        let dk = d / h;
        let cw = 16.0;
        let y = 70.0;
        let colour_for = |c: usize| if c < dk { Some(tok::FILL2) } else { None };
        // stage 1
        let x1 = 50.0;
        let rows1: Vec<Vec<Cell>> = (0..n).map(|_| (0..d).map(|c| match colour_for(c) { Some(f) => cell("").fill(f), None => cell("") }).collect()).collect();
        s.cells(x1, y, cw, &rows1);
        s.rect_bold(x1 + dk as f64 * cw, y, dk as f64 * cw, n as f64 * cw, "none", tok::S2);
        s.text(x1 + d as f64 * cw / 2.0, y - 12.0, "after one Linear", 12, tok::INK, Anchor::Middle);
        s.text(x1 + dk as f64 * cw / 2.0, y + n as f64 * cw + 14.0, "head 0", 11, tok::INK2, Anchor::Middle);
        s.text(x1 + 1.5 * dk as f64 * cw, y + n as f64 * cw + 14.0, "head 1", 11, tok::S2, Anchor::Middle);
        let sh1 = match v { Variant::Guide => shape(&["1", "4", "8"], v), Variant::Handbook => shape(&["2", "3", "8"], v) };
        s.text(x1 + d as f64 * cw / 2.0, y + n as f64 * cw + 32.0, &sh1, 12, tok::INK, Anchor::Middle);
        s.text(x1 - 4.0, y + n as f64 * cw / 2.0 + 4.0, "tokens", 11, tok::MUTED, Anchor::End);
        // stage 2: reshape
        let x2 = x1 + d as f64 * cw + 92.0;
        s.arrow(x1 + d as f64 * cw + 8.0, y + n as f64 * cw / 2.0, x2 - 8.0, y + n as f64 * cw / 2.0, Some("reshape"));
        let gap = 8.0;
        for head in 0..h {
            let hx = x2 + head as f64 * (dk as f64 * cw + gap);
            let rows: Vec<Vec<Cell>> = (0..n).map(|_| (0..dk).map(|_| if head == 0 { cell("").fill(tok::FILL2) } else { cell("") }).collect()).collect();
            s.cells(hx, y, cw, &rows);
            if head == 1 {
                s.rect_bold(hx, y, dk as f64 * cw, n as f64 * cw, "none", tok::S2);
            }
        }
        let w2 = h as f64 * dk as f64 * cw + (h - 1) as f64 * gap;
        s.text(x2 + w2 / 2.0, y - 12.0, "(…, heads, d_k)", 12, tok::INK, Anchor::Middle);
        let sh2 = match v { Variant::Guide => shape(&["1", "4", "2", "4"], v), Variant::Handbook => shape(&["2", "3", "2", "4"], v) };
        s.text(x2 + w2 / 2.0, y + n as f64 * cw + 32.0, &sh2, 12, tok::INK, Anchor::Middle);
        s.text(x2 + w2 / 2.0, y + n as f64 * cw + 14.0, "same numbers, regrouped", 11, tok::INK2, Anchor::Middle);
        // stage 3: transpose
        let x3 = x2 + w2 + 110.0;
        s.arrow(x2 + w2 + 8.0, y + n as f64 * cw / 2.0, x3 - 8.0, y + n as f64 * cw / 2.0, Some("transpose(1, 2)"));
        for head in 0..h {
            let hy = y - 10.0 + head as f64 * (n as f64 * cw + 14.0);
            let rows: Vec<Vec<Cell>> = (0..n).map(|_| (0..dk).map(|_| if head == 0 { cell("").fill(tok::FILL2) } else { cell("") }).collect()).collect();
            s.cells(x3, hy, cw, &rows);
            if head == 1 {
                s.rect_bold(x3, hy, dk as f64 * cw, n as f64 * cw, "none", tok::S2);
            }
            s.text(x3 + dk as f64 * cw + 6.0, hy + n as f64 * cw / 2.0 + 4.0, &format!("head {head}: {n} × {dk}"), 11, tok::INK2, Anchor::Start);
        }
        s.text(x3 + dk as f64 * cw / 2.0 + 30.0, y - 24.0, "(…, heads, tokens, d_k)", 12, tok::INK, Anchor::Middle);
        let sh3 = match v { Variant::Guide => shape(&["1", "2", "4", "4"], v), Variant::Handbook => shape(&["2", "2", "3", "4"], v) };
        s.text(x3 + dk as f64 * cw / 2.0 + 30.0, y + 2.0 * n as f64 * cw + 22.0, &sh3, 12, tok::INK, Anchor::Middle);
        // tail
        let (tail1, tail2, claim) = match v {
            Variant::Guide => (
                "one batched matmul per stage: scores (1, 2, 4, 4) = Q·Kᵀ/√4 for both heads at once, then × V,",
                "then transpose(1, 2) and reshape back to (1, 4, 8)",
                "head 1 is columns 4..8 of the projection: the same 8 numbers grouped differently, no second Linear",
            ),
            Variant::Handbook => (
                "Kᵀ [2,2,4,3] → scores [2,2,3,3] → softmax → A@V [2,2,3,4] → transpose(1, 2) [2,3,2,4] → merge [2,3,8]",
                "reshape separates a feature index into (head, feature in head);",
                "transpose puts heads where batched matrix multiplication expects them",
            ),
        };
        s.text(360.0, 244.0, tail1, 11, tok::INK2, Anchor::Middle);
        s.text(360.0, 258.0, tail2, 11, tok::INK2, Anchor::Middle);
        s.text(360.0, 272.0, claim, 11, tok::INK2, Anchor::Middle);
        (format!("f19-heads-{}.svg", v.suffix()), s.finish())
    }

    // ---------------------------------------------------------------------------------------------
    // F20 positional encoding: fast and slow dimension pairs
    // ---------------------------------------------------------------------------------------------
    pub fn fig_positional(v: Variant) -> (String, String) {
        let mut s = Svg::new(
            720,
            272,
            "Each dimension pair of the sinusoidal encoding is a sine and a cosine of pos / 10000^(2i/d): low pairs change quickly with position, high pairs slowly",
            "Small multiples, one panel per dimension pair, plotting the sine and cosine values at positions 0 to 4, with the position-1 values labelled.",
        );
        let (d, dec) = match v {
            Variant::Guide => (8usize, 3usize),
            Variant::Handbook => (4usize, 6usize),
        };
        let seq = 5usize;
        let pe = calc::positional(seq, d);
        let pairs = d / 2;
        let panel_w = if pairs == 4 { 160.0 } else { 300.0 };
        let gap = if pairs == 4 { 14.0 } else { 60.0 };
        let total = pairs as f64 * panel_w + (pairs - 1) as f64 * gap;
        let x_start = (720.0 - total) / 2.0;
        let (py0, py1) = (200.0, 60.0);
        let ys = |val: f64| (py0 + py1) / 2.0 - val * (py0 - py1) / 2.0;
        // legend
        s.line(x_start, 26.0, x_start + 18.0, 26.0, tok::S1, 2.0);
        s.dot(x_start + 9.0, 26.0, 4.0, tok::S1, None);
        s.text(x_start + 24.0, 30.0, "sin: dimension 2i", 11, tok::INK2, Anchor::Start);
        s.line(x_start + 150.0, 26.0, x_start + 168.0, 26.0, tok::S2, 2.0);
        s.dot(x_start + 159.0, 26.0, 4.0, tok::S2, None);
        s.text(x_start + 174.0, 30.0, "cos: dimension 2i + 1", 11, tok::INK2, Anchor::Start);
        for i in 0..pairs {
            let px = x_start + i as f64 * (panel_w + gap);
            let xs = |pos: usize| px + 20.0 + pos as f64 * (panel_w - 40.0) / (seq - 1) as f64;
            s.line(px, ys(0.0), px + panel_w, ys(0.0), tok::GRID, 1.0);
            s.line(px, ys(1.0), px + panel_w, ys(1.0), tok::GRID, 1.0);
            s.line(px, ys(-1.0), px + panel_w, ys(-1.0), tok::GRID, 1.0);
            s.text(px - 4.0, ys(1.0) + 4.0, "1", 11, tok::MUTED, Anchor::End);
            s.text(px - 4.0, ys(0.0) + 4.0, "0", 11, tok::MUTED, Anchor::End);
            s.text(px - 4.0, ys(-1.0) + 4.0, "−1", 11, tok::MUTED, Anchor::End);
            for pos in 0..seq {
                s.text(xs(pos), py0 + 16.0, &pos.to_string(), 11, tok::MUTED, Anchor::Middle);
            }
            let divisor = 10000f64.powf(2.0 * i as f64 / d as f64);
            s.text(px + panel_w / 2.0, py0 + 30.0, &format!("pair i = {i}: pos / {}", numt(divisor, 0)), 11, tok::INK2, Anchor::Middle);
            for (dim_off, colour) in [(0usize, tok::S1), (1usize, tok::S2)] {
                let mut dpath = String::new();
                for pos in 0..seq {
                    let val = pe[pos][2 * i + dim_off];
                    let (x, yv) = (xs(pos), ys(val));
                    dpath.push_str(&format!("{}{} {}", if pos == 0 { "M" } else { " L" }, svg::c(x), svg::c(yv)));
                }
                s.path(&dpath, colour, 2.0);
                for pos in 0..seq {
                    s.dot(xs(pos), ys(pe[pos][2 * i + dim_off]), 4.0, colour, Some(tok::SURFACE));
                }
                let v1 = pe[1][2 * i + dim_off];
                if dim_off == 0 {
                    s.text(xs(1) + 6.0, ys(v1) + 15.0, &num(v1, dec), 11, tok::INK, Anchor::Start);
                } else {
                    s.text(xs(1) - 7.0, ys(v1) - 7.0, &num(v1, dec), 11, tok::INK, Anchor::End);
                }
            }
        }
        let (note1, note2) = match v {
            Variant::Guide => ("position 0 is 0, 1, 0, 1, … in every pair (sin 0 = 0, cos 0 = 1);", "pair 0 moves fastest, pair 3 barely moves across these five positions"),
            Variant::Handbook => ("position 0 gives [0, 1, 0, 1]; position 1 gives the four labelled values;", "the slower pair separates far-apart positions, the faster one separates neighbours"),
        };
        s.text(360.0, 250.0, note1, 11, tok::INK2, Anchor::Middle);
        s.text(360.0, 264.0, note2, 11, tok::INK2, Anchor::Middle);
        (format!("f20-positional-{}.svg", v.suffix()), s.finish())
    }

    // ---------------------------------------------------------------------------------------------
    // F21 pre-LN and post-LN (shared)
    // ---------------------------------------------------------------------------------------------
    pub fn fig_block() -> (String, String) {
        let mut s = Svg::new(
            720,
            360,
            "Post-LN and pre-LN blocks contain the same four parts; only where the layer normalisation sits differs",
            "Two vertical flows: post-LN normalises after each skip addition; pre-LN normalises the input of each sub-layer. Skip connections are drawn as curves into the addition nodes; the LN boxes are shaded.",
        );
        let flow = |s: &mut Svg, cx: f64, title: &str, order: &[&str], skips: &[(usize, usize)]| {
            s.text_bold(cx, 28.0, title, 13, tok::INK, Anchor::Middle);
            let (bw, bh, step) = (140.0, 26.0, 36.0);
            let mut ys: Vec<f64> = Vec::new();
            for (i, name) in order.iter().enumerate() {
                let y = 44.0 + i as f64 * step;
                ys.push(y);
                match *name {
                    "x" | "y" | "u" => s.text(cx, y + 17.0, name, 12, tok::INK, Anchor::Middle),
                    "⊕" => {
                        s.dot(cx, y + 13.0, 11.0, tok::SURFACE, Some(tok::INK2));
                        s.text(cx, y + 17.0, "+", 13, tok::INK, Anchor::Middle);
                    }
                    "LN" => s.labelled_box(cx - bw / 2.0, y, bw, bh, "LayerNorm", tok::FILL2, tok::S1),
                    other => s.labelled_box(cx - bw / 2.0, y, bw, bh, other, tok::SURFACE, tok::AXIS),
                }
                if i + 1 < order.len() {
                    s.arrow(cx, y + bh + 1.0, cx, y + step - 2.0, None);
                }
            }
            // skips: (stage whose output is added, ⊕ that receives it), stated per flow
            for &(from, plus) in skips {
                debug_assert_eq!(order[plus], "⊕");
                let y_from = ys[from] + 26.0;
                let y_to = ys[plus] + 13.0;
                s.skip(cx + 6.0, y_from, cx + 11.0, y_to, 70.0, Some("skip"));
            }
        };
        // post-LN: u = LN(x + Attention(x)); y = LN(u + FFN(u)): the skips carry x and u (the first LN's output)
        flow(&mut s, 200.0, "post-LN (the original paper)", &["x", "Attention", "⊕", "LN", "FFN", "⊕", "LN", "y"], &[(0, 2), (3, 5)]);
        // pre-LN: u = x + Attention(LN(x)); y = u + FFN(LN(u)): the skips carry x and u (the first ⊕)
        flow(&mut s, 520.0, "pre-LN (GPT-2 and most later models)", &["x", "LN", "Attention", "⊕", "LN", "FFN", "⊕", "y"], &[(0, 3), (3, 6)]);
        s.line(360.0, 40.0, 360.0, 330.0, tok::GRID, 1.0);
        s.text(360.0, 340.0, "same four parts: attention, a per-token feed-forward network, two skip additions, two layer normalisations;", 11, tok::INK2, Anchor::Middle);
        s.text(360.0, 354.0, "only the position of LayerNorm (shaded) differs", 11, tok::INK2, Anchor::Middle);
        ("f21-transformer-block.svg".to_string(), s.finish())
    }

    // ---------------------------------------------------------------------------------------------
    // F22 partial self-attention
    // ---------------------------------------------------------------------------------------------
    pub fn fig_psa(v: Variant) -> (String, String) {
        let mut s = Svg::new(
            720,
            if v == Variant::Handbook { 362 } else { 300 },
            "Partial self-attention splits the channels after a 1×1 convolution, runs attention and a feed-forward network on one half, and fuses the halves with another 1×1",
            "A left-to-right flow with two channel bands, the lower one passing through attention and a feed-forward stage with skips, then concatenation and a 1×1 convolution; an inset states the score-matrix size.",
        );
        let y = 90.0;
        let (in_label, a_label, b_label, out_label, inset1, inset2, inset3) = match v {
            Variant::Guide => (
                "x (B, C, H, W)".to_string(),
                "a: (B, C/2, H, W), untouched".to_string(),
                "b: (B, C/2, H, W)".to_string(),
                "out (B, C, H, W)".to_string(),
                "scores per head: N × N with N = H·W".to_string(),
                "cost grows with N²: hence only after the last,".to_string(),
                "lowest-resolution stage, and only on half the channels".to_string(),
            ),
            Variant::Handbook => {
                let n = 20usize * 20;
                let per_head = n * n;
                let two_heads = 2 * per_head;
                let n80 = 80usize * 80;
                let per_head80 = n80 * n80;
                (
                    "x [1,256,20,20]".to_string(),
                    "a [1,128,20,20], untouched".to_string(),
                    "b [1,128,20,20]".to_string(),
                    "out [1,256,20,20]".to_string(),
                    format!("2 heads, d_k 32, d_v 64, N = {n}: {} entries per head, {} for two", svg::thousands(per_head as u64), svg::thousands(two_heads as u64)),
                    format!("at 80×80, N = {}: {} per head", svg::thousands(n80 as u64), svg::thousands(per_head80 as u64)),
                    format!("{}× more for 4× the height and width", per_head80 / per_head),
                )
            }
        };
        s.labelled_box(16.0, y + 20.0, 110.0, 40.0, &in_label, tok::SURFACE, tok::AXIS);
        s.labelled_box(150.0, y + 20.0, 70.0, 40.0, "1×1 conv", tok::SURFACE, tok::AXIS);
        s.arrow(126.0, y + 40.0, 146.0, y + 40.0, None);
        // split point
        s.arrow(220.0, y + 40.0, 248.0, y + 40.0, Some("split"));
        // band a (upper)
        s.rect_bold(250.0, y - 30.0, 300.0, 32.0, tok::FILL1, tok::S1);
        s.text(400.0, y - 10.0, &a_label, 12, tok::INK, Anchor::Middle);
        // band b (lower): attention and FFN with skips
        s.rect_bold(250.0, y + 50.0, 300.0, 100.0, tok::SURFACE, tok::S2);
        s.text(262.0, y + 66.0, &format!("{b_label} — BatchNorm, not LayerNorm"), 11, tok::INK2, Anchor::Start);
        s.labelled_box(262.0, y + 80.0, 110.0, 30.0, "MHSA", tok::SURFACE, tok::AXIS);
        s.dot(392.0, y + 95.0, 9.0, tok::SURFACE, Some(tok::INK2));
        s.text(392.0, y + 99.0, "+", 12, tok::INK, Anchor::Middle);
        s.labelled_box(414.0, y + 80.0, 80.0, 30.0, "FFN", tok::SURFACE, tok::AXIS);
        s.dot(516.0, y + 95.0, 9.0, tok::SURFACE, Some(tok::INK2));
        s.text(516.0, y + 99.0, "+", 12, tok::INK, Anchor::Middle);
        s.arrow(372.0, y + 95.0, 381.0, y + 95.0, None);
        s.arrow(401.0, y + 95.0, 410.0, y + 95.0, None);
        s.arrow(494.0, y + 95.0, 505.0, y + 95.0, None);
        s.path(&format!("M262 {} C262 {} 392 {} 392 {}", svg::c(y + 80.0), svg::c(y + 120.0), svg::c(y + 130.0), svg::c(y + 104.0)), tok::INK2, 1.0);
        s.path(&format!("M414 {} C414 {} 516 {} 516 {}", svg::c(y + 80.0), svg::c(y + 120.0), svg::c(y + 130.0), svg::c(y + 104.0)), tok::INK2, 1.0);
        s.text(327.0, y + 140.0, "skip", 11, tok::INK2, Anchor::Middle);
        s.text(465.0, y + 140.0, "skip", 11, tok::INK2, Anchor::Middle);
        // lines from the split to the bands
        s.line(248.0, y + 40.0, 248.0, y - 14.0, tok::INK2, 1.5);
        s.arrow(248.0, y - 14.0, 250.0, y - 14.0, None);
        s.line(248.0, y + 40.0, 248.0, y + 100.0, tok::INK2, 1.5);
        s.arrow(248.0, y + 100.0, 250.0, y + 100.0, None);
        // concat and fuse
        s.arrow(550.0, y - 14.0, 578.0, y + 30.0, None);
        s.arrow(550.0, y + 100.0, 578.0, y + 50.0, None);
        s.labelled_box(580.0, y + 20.0, 60.0, 40.0, "concat", tok::SURFACE, tok::AXIS);
        s.labelled_box(650.0, y + 20.0, 60.0, 40.0, "1×1 conv", tok::SURFACE, tok::AXIS);
        s.arrow(640.0, y + 40.0, 646.0, y + 40.0, None);
        s.text(712.0, y + 76.0, &out_label, 11, tok::INK2, Anchor::End);
        // inset
        s.text(360.0, 258.0, &inset1, 12, tok::INK, Anchor::Middle);
        s.text(360.0, 274.0, &inset2, 11, tok::INK2, Anchor::Middle);
        s.text(360.0, 288.0, &inset3, 11, tok::INK2, Anchor::Middle);
        if v == Variant::Handbook {
            // two bars: 20×20 and 80×80 score sizes per head, with a break mark
            let (bx, by) = (40.0, 306.0);
            s.rect(bx, by, 6.0, 10.0, tok::S1, None);
            s.text(bx + 12.0, by + 9.0, &format!("20×20: {} per head", svg::thousands((20u64 * 20) * (20 * 20))), 11, tok::INK2, Anchor::Start);
            s.rect(bx, by + 20.0, 600.0, 10.0, tok::S1, None);
            s.rect(bx + 300.0, by + 18.0, 6.0, 14.0, tok::SURFACE, None);
            s.line(bx + 298.0, by + 32.0, bx + 308.0, by + 18.0, tok::INK2, 1.0);
            s.text(bx + 12.0, by + 46.0, &format!("80×80: {} per head (the bar is cut; it would be {}× longer)", svg::thousands((80u64 * 80) * (80 * 80)), ((80u64 * 80) * (80 * 80)) / ((20 * 20) * (20 * 20))), 11, tok::INK2, Anchor::Start);
        }
        (format!("f22-psa-split-{}.svg", v.suffix()), s.finish())
    }

    // ---------------------------------------------------------------------------------------------
    // F23 the output-size ambiguity that output_padding resolves
    // ---------------------------------------------------------------------------------------------
    /// One cell of a position strip: a real position with its label, a padding or cropped position, a position
    /// no window reads, the position that only `output_padding` keeps, or a declared position nothing reaches.
    enum Pos {
        Real(String),
        Pad(&'static str),
        Unread(String),
        Extra(String),
        Declared,
    }

    /// A strip of `cw` × `ch` cells at (x, y): padding cells dashed with a muted label, never-read cells neutral
    /// with an orange outline, the extra cell tinted with an orange outline, a declared cell dashed orange and empty.
    fn pos_strip(s: &mut Svg, x: f64, y: f64, cw: f64, ch: f64, cells: &[Pos]) {
        for (j, p) in cells.iter().enumerate() {
            let cx = x + j as f64 * cw;
            let (tx, ty) = (cx + cw / 2.0, y + ch / 2.0 + 4.0);
            match p {
                Pos::Real(t) => {
                    s.rect(cx, y, cw, ch, tok::SURFACE, Some(tok::GRID));
                    s.text(tx, ty, t, 11, tok::INK, Anchor::Middle);
                }
                Pos::Pad(t) => {
                    s.rect_dashed(cx, y, cw, ch, tok::AXIS);
                    if !t.is_empty() {
                        s.text(tx, ty, t, 11, tok::MUTED, Anchor::Middle);
                    }
                }
                Pos::Unread(t) => {
                    s.rect_bold(cx, y, cw, ch, tok::NEUTRAL, tok::S2);
                    s.text(tx, ty, t, 11, tok::MUTED, Anchor::Middle);
                }
                Pos::Extra(t) => {
                    s.rect_bold(cx, y, cw, ch, tok::FILL2, tok::S2);
                    s.text(tx, ty, t, 11, tok::INK, Anchor::Middle);
                }
                Pos::Declared => s.rect_dashed(cx, y, cw, ch, tok::S2),
            }
        }
    }

    /// One window or stamp bar, `width` cells wide from column `col` of a strip whose cells are `cw` wide.
    fn bar_row(s: &mut Svg, x: f64, y: f64, cw: f64, col: i64, width: usize, label: &str, outline: &str) {
        let x0 = x + col as f64 * cw;
        let w = width as f64 * cw;
        s.rect_bold(x0, y, w, 16.0, tok::FILL2, outline);
        s.text(x0 + w / 2.0, y + 12.0, label, 11, tok::INK2, Anchor::Middle);
    }

    /// A square bracket under columns `col0..col1` of a strip, with its label beneath.
    fn bracket_under(s: &mut Svg, x: f64, y: f64, cw: f64, col0: usize, col1: usize, label: &str) {
        let x0 = x + col0 as f64 * cw + 2.0;
        let x1 = x + col1 as f64 * cw - 2.0;
        s.path(&format!("M{} {} v5 H{} v-5", svg::c(x0), svg::c(y), svg::c(x1)), tok::INK2, 1.0);
        s.text((x0 + x1) / 2.0, y + 17.0, label, 11, tok::INK2, Anchor::Middle);
    }

    /// The number of cells a window or stamp covers, from its first to its last tap.
    fn span(taps: &[i64]) -> usize {
        (taps[taps.len() - 1] - taps[0] + 1) as usize
    }

    /// The real cells (1-based) that no forward window reads.
    fn unread(windows: &[Vec<i64>], l: usize) -> Vec<usize> {
        (0..l).filter(|c| !windows.iter().any(|w| w.contains(&(*c as i64)))).map(|c| c + 1).collect()
    }

    /// Where the last forward window ends: on the padding, or on a real cell (1-based, named by `unit`).
    fn last_window_ends(windows: &[Vec<i64>], l: usize, unit: &str) -> String {
        let last = windows[windows.len() - 1][windows[0].len() - 1];
        if last >= l as i64 { format!("window {} ends on the padding", windows.len()) } else { format!("window {} ends on {unit} {}", windows.len(), last + 1) }
    }

    /// Which samples no window reads, as a footer.
    fn unread_footer(windows: &[Vec<i64>], l: usize) -> String {
        let u = unread(windows, l);
        if u.is_empty() { "every sample is read".to_string() } else { format!("sample {} is read by no window", u.iter().map(|c| c.to_string()).collect::<Vec<_>>().join(", ")) }
    }

    /// A forward panel: the windows as stacked bars over the padded input strip, an arrow to the output strip,
    /// and a footer line. `out_name` is the document's name for the output length. Returns the y below the footer.
    fn forward_panel(s: &mut Svg, x: f64, y: f64, cw: f64, l: usize, k: usize, st: usize, p: usize, title: &str, mark_last: bool, out_name: &str, footer: &str) -> f64 {
        s.text(x, y, title, 12, tok::INK, Anchor::Start);
        let windows = calc::conv_windows(l, k, st, p, 1);
        let n = windows.len();
        for (i, w) in windows.iter().enumerate() {
            let outline = if mark_last && i + 1 == n { tok::S2 } else { tok::S1 };
            bar_row(s, x, y + 10.0 + i as f64 * 19.0, cw, w[0] + p as i64, span(w), &format!("window {}", i + 1), outline);
        }
        let sy = y + 10.0 + n as f64 * 19.0 + 6.0;
        let mut cells: Vec<Pos> = (0..p).map(|_| Pos::Pad("pad")).collect();
        for c in 0..l {
            let read = windows.iter().any(|w| w.contains(&(c as i64)));
            let t = (c + 1).to_string();
            cells.push(if read { Pos::Real(t) } else { Pos::Unread(t) });
        }
        cells.extend((0..p).map(|_| Pos::Pad("pad")));
        pos_strip(s, x, sy, cw, 26.0, &cells);
        let total = cells.len() as f64 * cw;
        let oy = sy + 26.0 + 30.0;
        s.arrow(x + total / 2.0, sy + 28.0, x + total / 2.0, oy - 4.0, None);
        let out_cells: Vec<Pos> = (0..n).map(|c| Pos::Real((c + 1).to_string())).collect();
        let ox = x + (total - n as f64 * cw) / 2.0;
        pos_strip(s, ox, oy, cw, 26.0, &out_cells);
        s.text(ox + n as f64 * cw + 8.0, oy + 17.0, &format!("{out_name} = {n}"), 12, tok::INK, Anchor::Start);
        s.text(x, oy + 26.0 + 16.0, footer, 11, tok::INK2, Anchor::Start);
        oy + 26.0 + 16.0
    }

    pub fn fig_output_padding(v: Variant) -> (String, String) {
        match v {
            Variant::Guide => {
                let (k, st, p) = (3usize, 2usize, 1usize);
                let (a, b) = (7usize, 8usize);
                let mut s = Svg::new(
                    720,
                    462,
                    "A stride-2 convolution gives the same output size for two input sizes; output_padding tells the transposed convolution which one to return",
                    "Two forward panels (inputs 7 and 8, kernel 3, stride 2, padding 1) with their four windows and the shared output of 4, then the way back: four stamps into a strip of nine columns, where the padding crops the first and output_padding keeps the last. One axis of the photo is drawn; the other behaves the same, so in two dimensions the added positions are the bottom row and the right column.",
                );
                let cw = 28.0;
                let (na, nb) = (calc::conv_out(a, k, st, p, 1), calc::conv_out(b, k, st, p, 1));
                let (wa, wb) = (calc::conv_windows(a, k, st, p, 1), calc::conv_windows(b, k, st, p, 1));
                debug_assert_eq!((wa.len(), wb.len()), (na, nb));
                let end_a = forward_panel(&mut s, 30.0, 24.0, cw, a, k, st, p, &format!("H_in = {a}, padding {p} each side"), true, "H_out", &last_window_ends(&wa, a, "pixel"));
                forward_panel(&mut s, 380.0, 24.0, cw, b, k, st, p, &format!("H_in = {b}, padding {p} each side"), true, "H_out", &last_window_ends(&wb, b, "pixel"));
                let ly = end_a + 20.0;
                s.rect_bold(30.0, ly - 9.0, 10.0, 10.0, tok::FILL2, tok::S2);
                s.text(46.0, ly, "orange: the window that ends differently", 11, tok::INK2, Anchor::Start);
                // the way back: the forward outputs are now the inputs of a transposed convolution
                let lin = na;
                let (l0, l1) = (calc::tconv_out(lin, k, st, p, 1, 0), calc::tconv_out(lin, k, st, p, 1, 1));
                let by = ly + 30.0;
                s.text_bold(30.0, by, &format!("back from {lin}: transposed convolution, stride {st}, padding {p}"), 13, tok::INK, Anchor::Start);
                let bcw = 36.0;
                let stamps = calc::tconv_stamps(lin, k, st, p, 1);
                s.text(30.0, by + 18.0, &format!("the {lin} outputs, now the inputs"), 12, tok::INK, Anchor::Start);
                for (i, st_i) in stamps.iter().enumerate() {
                    let outline = if i + 1 == lin { tok::S2 } else { tok::S1 };
                    bar_row(&mut s, 30.0, by + 26.0 + i as f64 * 19.0, bcw, st_i[0] + p as i64, span(st_i), &format!("stamp of {}", i + 1), outline);
                }
                let sy = by + 26.0 + lin as f64 * 19.0 + 6.0;
                // full columns 0..=l1: column c is position c − p; negative positions are cropped, positions ≥ l0 kept only with output_padding
                let mut cells: Vec<Pos> = Vec::new();
                for c in 0..(l1 + p) {
                    let pos = c as i64 - p as i64;
                    cells.push(if pos < 0 {
                        Pos::Pad("")
                    } else if (pos as usize) < l0 {
                        Pos::Real((pos + 1).to_string())
                    } else {
                        Pos::Extra((pos + 1).to_string())
                    });
                }
                pos_strip(&mut s, 30.0, sy, bcw, 26.0, &cells);
                s.text(30.0 + bcw / 2.0, sy - 6.0, "cropped", 11, tok::MUTED, Anchor::Middle);
                bracket_under(&mut s, 30.0, sy + 30.0, bcw, p, p + l0, &format!("output_padding = 0 → H_out = {l0}"));
                bracket_under(&mut s, 30.0, sy + 54.0, bcw, p, p + l1, &format!("output_padding = 1 → H_out = {l1}"));
                let lines = [
                    format!("H_out = ({lin} − 1)·{st} − 2·{p} + 1·({k} − 1)"),
                    format!("+ output_padding + 1 = {l0} + output_padding"),
                    format!("column {l1} is the tail of stamp {lin}:"),
                    "cropped when output_padding = 0,".to_string(),
                    "kept when output_padding = 1: one side only".to_string(),
                    "(in 2-D: the bottom row and the right column)".to_string(),
                ];
                for (i, l) in lines.iter().enumerate() {
                    s.text(400.0, by + 26.0 + i as f64 * 15.0, l, 11, tok::INK2, Anchor::Start);
                }
                (format!("f23-output-padding-{}.svg", v.suffix()), s.finish())
            }
            Variant::Handbook => {
                let (k, st, p) = (3usize, 2usize, 0usize);
                let (a, b) = (5usize, 6usize);
                let mut s = Svg::new(
                    720,
                    380,
                    "A forward convolution with kernel 3, stride 2 and no padding maps 5 and 6 samples to 2; output_padding tells the transposed convolution to return 5 or 6",
                    "Two forward panels (inputs 5 and 6) with their two windows and the shared output of 2, the sixth sample marked as never read, then the way back: two stamps cover five positions and output_padding 1 declares a sixth that no stamp reaches.",
                );
                let cw = 36.0;
                let (na, nb) = (calc::conv_out(a, k, st, p, 1), calc::conv_out(b, k, st, p, 1));
                debug_assert_eq!(na, nb);
                let (wa, wb) = (calc::conv_windows(a, k, st, p, 1), calc::conv_windows(b, k, st, p, 1));
                let end_a = forward_panel(&mut s, 30.0, 24.0, cw, a, k, st, p, &format!("Lin = {a}, k = {k}, s = {st}, p = {p}"), false, "Lout", &unread_footer(&wa, a));
                forward_panel(&mut s, 330.0, 24.0, cw, b, k, st, p, &format!("Lin = {b}, k = {k}, s = {st}, p = {p}"), false, "Lout", &unread_footer(&wb, b));
                let dropped = unread(&wb, b);
                debug_assert_eq!(dropped, vec![b]);
                let ly = end_a + 20.0;
                s.rect_bold(30.0, ly - 9.0, 10.0, 10.0, tok::NEUTRAL, tok::S2);
                s.text(46.0, ly, "orange: the sample no window reads", 11, tok::INK2, Anchor::Start);
                let lin = na;
                let (l0, l1) = (calc::tconv_out(lin, k, st, p, 1, 0), calc::tconv_out(lin, k, st, p, 1, 1));
                let by = ly + 30.0;
                s.text_bold(30.0, by, &format!("back from {lin}: transposed convolution, stride {st}, kernel [u, v, w]"), 13, tok::INK, Anchor::Start);
                let bcw = 60.0;
                let names = ["a", "b"];
                let taps = ["u", "v", "w"];
                let stamps = calc::tconv_stamps(lin, k, st, p, 1);
                s.text(30.0, by + 18.0, &format!("the {lin} outputs, now the inputs a and b"), 12, tok::INK, Anchor::Start);
                for (i, st_i) in stamps.iter().enumerate() {
                    bar_row(&mut s, 30.0, by + 26.0 + i as f64 * 19.0, bcw, st_i[0] + p as i64, span(st_i), &format!("stamp of {}", names[i]), tok::S1);
                }
                let sy = by + 26.0 + lin as f64 * 19.0 + 6.0;
                // each position collects one term per stamp that reaches it: input name · kernel weight
                let mut cells: Vec<Pos> = Vec::new();
                for pos in 0..l1 {
                    let mut terms: Vec<String> = Vec::new();
                    for (i, st_i) in stamps.iter().enumerate() {
                        for (j, &q) in st_i.iter().enumerate() {
                            if q == pos as i64 {
                                terms.push(format!("{}·{}", names[i], taps[j]));
                            }
                        }
                    }
                    cells.push(if terms.is_empty() { Pos::Declared } else { Pos::Real(terms.join(" + ")) });
                }
                pos_strip(&mut s, 30.0, sy, bcw, 26.0, &cells);
                bracket_under(&mut s, 30.0, sy + 30.0, bcw, 0, l0, &format!("output_padding = 0 → Lout = {l0}"));
                bracket_under(&mut s, 30.0, sy + 54.0, bcw, 0, l1, &format!("output_padding = 1 → Lout = {l1}"));
                let lines = [
                    format!("Lout = ({lin} − 1)·{st} − 2·{p} + 1·({k} − 1)"),
                    format!("+ output_padding + 1 = {l0} + output_padding"),
                    format!("output_padding 1 declares a {}th position:", l1),
                    format!("a place for unread sample {}, not its value", dropped[0]),
                    format!("(admissible: output_padding 1 < stride {st})"),
                ];
                for (i, l) in lines.iter().enumerate() {
                    s.text(412.0, by + 26.0 + i as f64 * 15.0, l, 11, tok::INK2, Anchor::Start);
                }
                (format!("f23-output-padding-{}.svg", v.suffix()), s.finish())
            }
        }
    }

    // ---------------------------------------------------------------------------------------------
    // F24 where the SE block sits in a residual block
    // ---------------------------------------------------------------------------------------------
    /// One stage of a left-to-right flow: a plain label, a one- or two-line box (highlighted or not), or ⊕.
    enum Stage {
        Label(&'static str),
        Box1(&'static str, f64, bool),
        Box2(&'static str, &'static str, f64, bool),
        Plus,
    }

    /// Draws the stages left to right from `x` with boxes `bh` tall whose top is `y`, arrows between them;
    /// returns the x centre of every stage.
    fn flow_row(s: &mut Svg, x: f64, y: f64, bh: f64, stages: &[Stage]) -> Vec<f64> {
        let gap = 22.0;
        let mid = y + bh / 2.0;
        let mut cx = x;
        let mut centres = Vec::new();
        for (i, st) in stages.iter().enumerate() {
            let w = match st {
                Stage::Label(t) => svg::text_w(t, 12) + 4.0,
                Stage::Box1(_, w, _) | Stage::Box2(_, _, w, _) => *w,
                Stage::Plus => 22.0,
            };
            match st {
                Stage::Label(t) => s.text(cx + w / 2.0, mid + 4.0, t, 12, tok::INK, Anchor::Middle),
                Stage::Box1(t, _, hi) => {
                    let (fill, stroke) = if *hi { (tok::FILL2, tok::S1) } else { (tok::SURFACE, tok::AXIS) };
                    if *hi {
                        s.rect_bold(cx, y, w, bh, fill, stroke);
                        s.text(cx + w / 2.0, mid + 4.0, t, 12, tok::INK, Anchor::Middle);
                    } else {
                        s.labelled_box(cx, y, w, bh, t, fill, stroke);
                    }
                }
                Stage::Box2(t1, t2, _, hi) => {
                    let (fill, stroke) = if *hi { (tok::FILL2, tok::S1) } else { (tok::SURFACE, tok::AXIS) };
                    if *hi {
                        s.rect_bold(cx, y, w, bh, fill, stroke);
                        s.text(cx + w / 2.0, mid - 3.0, t1, 12, tok::INK, Anchor::Middle);
                        s.text(cx + w / 2.0, mid + 12.0, t2, 11, tok::INK2, Anchor::Middle);
                    } else {
                        s.labelled_box2(cx, y, w, bh, t1, t2, fill, stroke);
                    }
                }
                Stage::Plus => {
                    s.dot(cx + w / 2.0, mid, 11.0, tok::SURFACE, Some(tok::INK2));
                    s.text(cx + w / 2.0, mid + 4.0, "+", 13, tok::INK, Anchor::Middle);
                }
            }
            centres.push(cx + w / 2.0);
            if i + 1 < stages.len() {
                s.arrow(cx + w + 2.0, mid, cx + w + gap - 2.0, mid, None);
            }
            cx += w + gap;
        }
        centres
    }

    /// A skip connection from the arrow after stage `from` (x coordinate) down and along to the bottom of the
    /// stage centred at `to`, entering it from below; the label sits under the curve.
    fn skip_below(s: &mut Svg, x_from: f64, x_to: f64, mid: f64, bottom: f64, depth: f64, label: &str) {
        let low = mid + depth;
        s.path(&format!("M{} {} C{} {} {} {} {} {}", svg::c(x_from), svg::c(mid), svg::c(x_from), svg::c(low), svg::c(x_to), svg::c(low), svg::c(x_to), svg::c(bottom + 6.0)), tok::INK2, 1.5);
        s.arrow(x_to, bottom + 6.0, x_to, bottom + 1.0, None);
        s.text((x_from + x_to) / 2.0, low + 4.0, label, 11, tok::INK2, Anchor::Middle);
    }

    pub fn fig_se_placement(v: Variant) -> (String, String) {
        let (y, bh) = (60.0, 40.0);
        let mid = y + bh / 2.0;
        match v {
            Variant::Guide => {
                let mut s = Svg::new(
                    720,
                    210,
                    "In a residual block the SE gate scales the branch after conv-BN-ReLU 1×1, conv-BN-ReLU 3×3 and conv-BN 1×1, before the skip is added and the final ReLU applied",
                    "Left-to-right flow of the guide's residual block: three convolution stages, the highlighted SE box, the addition with the skip (identity or 1×1 projection), then ReLU; the skip bypasses SE.",
                );
                let stages = [
                    Stage::Label("x"),
                    Stage::Box2("conv-BN-ReLU", "1×1", 96.0, false),
                    Stage::Box2("conv-BN-ReLU", "3×3", 96.0, false),
                    Stage::Box2("conv-BN", "1×1", 96.0, false),
                    Stage::Box1("SE", 56.0, true),
                    Stage::Plus,
                    Stage::Box1("ReLU", 56.0, false),
                    Stage::Label("y"),
                ];
                let c = flow_row(&mut s, 48.0, y, bh, &stages);
                s.text(c[4], 36.0, "one gate per channel, z (B, C, 1, 1), each in (0, 1)", 11, tok::INK2, Anchor::Middle);
                s.text(c[4], 50.0, "multiplies the branch (13.2); the skip is not gated", 11, tok::INK2, Anchor::Middle);
                skip_below(&mut s, c[0] + 16.0, c[5], mid, mid + 11.0, 56.0, "identity or 1×1 projection");
                s.text(360.0, 184.0, "SE is the last stage of the branch, after the last conv-BN and before the addition;", 11, tok::INK2, Anchor::Middle);
                s.text(360.0, 198.0, "the ReLU follows the addition, and the skip joins at the addition without passing the gates", 11, tok::INK2, Anchor::Middle);
                (format!("f24-se-placement-{}.svg", v.suffix()), s.finish())
            }
            Variant::Handbook => {
                let mut s = Svg::new(
                    720,
                    200,
                    "SE sits on the residual branch after its convolution stack and multiplies it by one gate per channel before the skip path is added",
                    "Left-to-right flow of the handbook's residual integration: the convolution branch, the highlighted SE box with its per-channel gates, the addition with the ungated skip path, the output; the footer keeps the order of BN and activations open.",
                );
                let stages = [
                    Stage::Label("input"),
                    Stage::Box2("convolution branch", "[N,C,H,W]", 160.0, false),
                    Stage::Box2("SE", "gates [N,C,1,1]", 110.0, true),
                    Stage::Box1("add", 56.0, false),
                    Stage::Label("output"),
                ];
                let c = flow_row(&mut s, 110.0, y, bh, &stages);
                s.text(c[2], 44.0, "one gate per channel from the pooled descriptor (8.1)", 11, tok::INK2, Anchor::Middle);
                skip_below(&mut s, c[0] + 26.0, c[3], mid, y + bh, 56.0, "skip path, not gated");
                s.text(360.0, 174.0, "an actual block may also contain BN and activations;", 11, tok::INK2, Anchor::Middle);
                s.text(360.0, 188.0, "their order belongs to the architecture, not to SE", 11, tok::INK2, Anchor::Middle);
                (format!("f24-se-placement-{}.svg", v.suffix()), s.finish())
            }
        }
    }

    pub fn figures() -> Vec<(String, String)> {
        let mut out = Vec::new();
        for v in [Variant::Guide, Variant::Handbook] {
            out.push(fig_broadcast(v));
        }
        for v in [Variant::Guide, Variant::Handbook] {
            out.push(fig_layout(v));
        }
        for v in [Variant::Guide, Variant::Handbook] {
            out.push(fig_tokens(v));
        }
        for v in [Variant::Guide, Variant::Handbook] {
            out.push(fig_handles(v));
        }
        for v in [Variant::Guide, Variant::Handbook] {
            out.push(fig_backward(v));
        }
        for v in [Variant::Guide, Variant::Handbook] {
            out.push(fig_softmax(v));
        }
        for v in [Variant::Guide, Variant::Handbook] {
            out.push(fig_conv_window(v));
        }
        for v in [Variant::Guide, Variant::Handbook] {
            out.push(fig_conv_matrix(v));
        }
        for v in [Variant::Guide, Variant::Handbook] {
            out.push(fig_tconv_overlap(v));
        }
        for v in [Variant::Guide, Variant::Handbook] {
            out.push(fig_pointwise(v));
        }
        for v in [Variant::Guide, Variant::Handbook] {
            out.push(fig_groups(v));
        }
        out.push(fig_dilation());
        for v in [Variant::Guide, Variant::Handbook] {
            out.push(fig_rf(v));
        }
        out.push(fig_norm_axes());
        for v in [Variant::Guide, Variant::Handbook] {
            out.push(fig_bn_fold(v));
        }
        for v in [Variant::Guide, Variant::Handbook] {
            out.push(fig_se(v));
        }
        for v in [Variant::Guide, Variant::Handbook] {
            out.push(fig_attention(v));
        }
        for v in [Variant::Guide, Variant::Handbook] {
            out.push(fig_masks(v));
        }
        for v in [Variant::Guide, Variant::Handbook] {
            out.push(fig_heads(v));
        }
        for v in [Variant::Guide, Variant::Handbook] {
            out.push(fig_positional(v));
        }
        out.push(fig_block());
        for v in [Variant::Guide, Variant::Handbook] {
            out.push(fig_psa(v));
        }
        for v in [Variant::Guide, Variant::Handbook] {
            out.push(fig_output_padding(v));
        }
        for v in [Variant::Guide, Variant::Handbook] {
            out.push(fig_se_placement(v));
        }
        out
    }
}

fn main() -> std::process::ExitCode {
    let dir = std::env::args().nth(1).unwrap_or_else(|| ".".to_string());
    if let Err(e) = std::fs::create_dir_all(&dir) {
        eprintln!("cannot create {dir}: {e}");
        return std::process::ExitCode::FAILURE;
    }
    let files = figs::figures();
    for (name, body) in &files {
        let path = std::path::Path::new(&dir).join(name);
        if let Err(e) = std::fs::write(&path, body) {
            eprintln!("cannot write {}: {e}", path.display());
            return std::process::ExitCode::FAILURE;
        }
        println!("{name} {} bytes", body.len());
    }
    println!("{} files written to {dir}", files.len());
    std::process::ExitCode::SUCCESS
}
