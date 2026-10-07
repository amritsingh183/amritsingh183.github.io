//! Figures of Parts V and VI (chapters 25 to 33) of "Machine Learning, Drawn Out". The values these figures draw are
//! computed here from the book's shared data (the five blurbs, the three attention words) or from inputs stated beside
//! them, with three exceptions. Published values that a figure quotes, such as GloVe's Table 1, the model sizes and
//! SBERT's timings, are typed as literals, and the figure, its caption or the text beside it names their source. Labels
//! also type numbers as text, such as sizes ("11 rows, 4 columns") and simple counts ("8 words"). The shading of Figure
//! 29.1 stands for values but is chosen by hand, as the figure itself says.
use crate::Figure;
use crate::svg::{Anchor, Cell, Svg, cell, heat, num, numt, text_w, thousands, tok};

/// Every figure of these chapters, in chapter order.
pub fn figures() -> Vec<Figure> {
    vec![
        fig_25_1_one_hot(),
        fig_25_2_word_map(),
        fig_25_3_lookup(),
        fig_25_4_pipeline(),
        fig_26_1_counts(),
        fig_26_2_idf(),
        fig_26_3_cosine(),
        fig_26_4_order(),
        fig_27_1_window(),
        fig_27_2_two_nets(),
        fig_27_3_rows(),
        fig_27_4_cosines(),
        fig_27_5_analogy(),
        fig_28_1_cooccurrence(),
        fig_28_2_ratios(),
        fig_28_3_ice_steam(),
        fig_28_4_weighting(),
        fig_29_1_static_contextual(),
        fig_29_2_pca(),
        fig_29_3_autoencoder(),
        fig_30_1_bottleneck(),
        fig_30_2_route(),
        fig_30_3_alignment(),
        fig_31_1_three_steps(),
        fig_31_2_qkv(),
        fig_31_3_heads(),
        fig_31_4_positions(),
        fig_32_1_block(),
        fig_32_2_pretrain(),
        fig_33_1_cross_bi(),
        fig_33_2_cost(),
        fig_33_3_siamese(),
        fig_33_4_cosine(),
    ]
}

// =================================================================================================
// Shared data and arithmetic
// =================================================================================================

/// The five bookshop blurbs (SPEC section B), B1 to B5.
const BLURBS: [&str; 5] = [
    "a quiet mystery in a small town",
    "a small town detective and a quiet mystery",
    "recipes from a village kitchen",
    "the detective returns to the village",
    "how to repair a bicycle",
];

/// Hand-set feature vectors (crime, place, food, tools) for the content words of the blurbs, in order of first
/// appearance. Invented for teaching and labelled so in the text; words not listed (a, and, from, how, in, the, to)
/// carry no features.
const FEATURE_NAMES: [&str; 4] = ["crime", "place", "food", "tools"];
const FEATURES: [(&str, [f64; 4]); 11] = [
    ("quiet", [0.3, 0.3, 0.1, 0.0]),
    ("mystery", [0.9, 0.0, 0.0, 0.0]),
    ("small", [0.1, 0.4, 0.1, 0.1]),
    ("town", [0.1, 1.0, 0.0, 0.0]),
    ("detective", [0.9, 0.2, 0.0, 0.1]),
    ("recipes", [0.0, 0.0, 1.0, 0.3]),
    ("village", [0.1, 0.8, 0.3, 0.0]),
    ("kitchen", [0.0, 0.2, 0.9, 0.2]),
    ("returns", [0.2, 0.1, 0.0, 0.2]),
    ("repair", [0.0, 0.0, 0.0, 1.0]),
    ("bicycle", [0.0, 0.1, 0.0, 0.9]),
];

/// Formats a coordinate for path data with at most one decimal (the builder's own formatter is private to it).
fn xy(v: f64) -> String {
    if (v - v.round()).abs() < 1e-9 { format!("{}", v.round() as i64) } else { format!("{v:.1}") }
}

fn tokens(s: &str) -> Vec<&str> {
    s.split(' ').collect()
}

/// The vocabulary of a set of texts, sorted alphabetically (as a count vectoriser orders its columns).
fn vocabulary(texts: &[&'static str]) -> Vec<&'static str> {
    let mut v: Vec<&'static str> = texts.iter().flat_map(|t| t.split(' ')).collect();
    v.sort_unstable();
    v.dedup();
    v
}

/// Count matrix: one row per text, one column per vocabulary word.
fn count_matrix(texts: &[&'static str], vocab: &[&str]) -> Vec<Vec<f64>> {
    texts
        .iter()
        .map(|t| {
            let toks = tokens(t);
            vocab.iter().map(|w| toks.iter().filter(|x| *x == w).count() as f64).collect()
        })
        .collect()
}

/// Document frequency of each column: in how many rows the word appears.
fn doc_freq(counts: &[Vec<f64>]) -> Vec<usize> {
    (0..counts[0].len()).map(|j| counts.iter().filter(|r| r[j] > 0.0).count()).collect()
}

/// Inverse document frequency ln(N / df) (Manning, Raghavan and Schütze, Introduction to Information Retrieval, eq. 21).
fn idf(df: &[usize], n: usize) -> Vec<f64> {
    df.iter().map(|d| (n as f64 / *d as f64).ln()).collect()
}

/// tf-idf = count × idf (the same book, eq. 22).
fn tfidf(counts: &[Vec<f64>], idf: &[f64]) -> Vec<Vec<f64>> {
    counts.iter().map(|r| r.iter().zip(idf).map(|(c, w)| c * w).collect()).collect()
}

fn dot(a: &[f64], b: &[f64]) -> f64 {
    a.iter().zip(b).map(|(x, y)| x * y).sum()
}

fn norm(a: &[f64]) -> f64 {
    dot(a, a).sqrt()
}

fn cosine(a: &[f64], b: &[f64]) -> f64 {
    dot(a, b) / (norm(a) * norm(b))
}

fn cosine_matrix(rows: &[Vec<f64>]) -> Vec<Vec<f64>> {
    rows.iter().map(|a| rows.iter().map(|b| cosine(a, b)).collect()).collect()
}

/// Feature vector of a word (zeros for words without features).
fn feature(word: &str) -> [f64; 4] {
    FEATURES.iter().find(|(w, _)| *w == word).map(|(_, f)| *f).unwrap_or([0.0; 4])
}

/// Mean pooling: the average of the feature vectors of every word of a text (all words count).
fn mean_pool(text: &str) -> Vec<f64> {
    let toks = tokens(text);
    let mut acc = vec![0.0; 4];
    for t in &toks {
        for (a, v) in acc.iter_mut().zip(feature(t)) {
            *a += v;
        }
    }
    acc.iter().map(|a| a / toks.len() as f64).collect()
}

/// First two principal directions of a set of points (rows), found by power iteration on the covariance matrix,
/// and each point's coordinates along them. Returns (projections, share of variance on each direction, first direction).
fn pca2(points: &[Vec<f64>]) -> (Vec<[f64; 2]>, [f64; 2], Vec<f64>) {
    let n = points.len() as f64;
    let d = points[0].len();
    let mean: Vec<f64> = (0..d).map(|j| points.iter().map(|p| p[j]).sum::<f64>() / n).collect();
    let centred: Vec<Vec<f64>> = points.iter().map(|p| p.iter().zip(&mean).map(|(a, m)| a - m).collect()).collect();
    let mut cov = vec![vec![0.0; d]; d];
    for p in &centred {
        for i in 0..d {
            for j in 0..d {
                cov[i][j] += p[i] * p[j] / n;
            }
        }
    }
    let total: f64 = (0..d).map(|i| cov[i][i]).sum();
    let mut dirs: Vec<Vec<f64>> = Vec::new();
    let mut vals = [0.0; 2];
    let mut work = cov.clone();
    for (k, val) in vals.iter_mut().enumerate() {
        let mut v: Vec<f64> = (0..d).map(|i| 1.0 + i as f64 * 0.1 + k as f64 * 0.37).collect();
        for _ in 0..500 {
            let w: Vec<f64> = (0..d).map(|i| dot(&work[i], &v)).collect();
            let len = norm(&w);
            v = w.iter().map(|x| x / len).collect();
        }
        // a fixed sign so the picture does not flip: the largest component is positive
        let big = v.iter().cloned().fold(0.0f64, |m, x| if x.abs() > m.abs() { x } else { m });
        if big < 0.0 {
            v = v.iter().map(|x| -x).collect();
        }
        let lambda = dot(&v, &(0..d).map(|i| dot(&work[i], &v)).collect::<Vec<f64>>());
        *val = lambda / total;
        for i in 0..d {
            for j in 0..d {
                work[i][j] -= lambda * v[i] * v[j];
            }
        }
        dirs.push(v);
    }
    let proj = centred.iter().map(|p| [dot(p, &dirs[0]), dot(p, &dirs[1])]).collect();
    (proj, vals, dirs[0].clone())
}

// -------------------------------------------------------------------------------------------------
// Small drawing helpers
// -------------------------------------------------------------------------------------------------

/// Cells for a matrix of values with a heat fill; zero cells stay plain and use muted ink.
fn heat_cells(m: &[Vec<f64>], max: f64, decimals: usize) -> Vec<Vec<Cell>> {
    m.iter()
        .map(|r| {
            r.iter()
                .map(|v| {
                    if v.abs() < 1e-12 {
                        cell(numt(0.0, 0)).ink(tok::MUTED)
                    } else {
                        cell(if decimals == 0 { numt(*v, 0) } else { num(*v, decimals) }).fill(heat(*v, max))
                    }
                })
                .collect()
        })
        .collect()
}

/// Row labels right-aligned before a grid.
fn row_labels(s: &mut Svg, x_right: f64, y: f64, ch: f64, labels: &[&str], size: u8) {
    for (i, l) in labels.iter().enumerate() {
        s.text(x_right, y + i as f64 * ch + ch / 2.0 + 4.0, l, size, tok::INK, Anchor::End);
    }
}

/// Places a label beside each dot, trying four positions in turn (up-right, down-right, up-left, down-left) and taking
/// the first whose estimated box overlaps no earlier label and no dot. Returns (x, y, starts at x).
fn place_labels(dots: &[(f64, f64)], labels: &[&str], size: u8) -> Vec<(f64, f64, bool)> {
    let mut boxes: Vec<(f64, f64, f64, f64)> = Vec::new();
    let mut out = Vec::new();
    let h = f64::from(size) + 2.0;
    for (k, (x, y)) in dots.iter().enumerate() {
        let w = text_w(labels[k], size);
        let tries = [(8.0, -6.0, true), (8.0, 14.0, true), (-8.0, -6.0, false), (-8.0, 14.0, false)];
        let mut pick = tries[0];
        for t in tries {
            let bx = if t.2 { x + t.0 } else { x + t.0 - w };
            let by = y + t.1 - h + 3.0;
            let hits_label = boxes.iter().any(|b| bx < b.0 + b.2 && bx + w > b.0 && by < b.1 + b.3 && by + h > b.1);
            let hits_dot = dots.iter().any(|(dx, dy)| *dx > bx - 5.0 && *dx < bx + w + 5.0 && *dy > by - 5.0 && *dy < by + h + 5.0);
            if !hits_label && !hits_dot {
                pick = t;
                break;
            }
        }
        let bx = if pick.2 { x + pick.0 } else { x + pick.0 - w };
        boxes.push((bx, y + pick.1 - h + 3.0, w, h));
        out.push((x + pick.0, y + pick.1, pick.2));
    }
    out
}

/// Column labels centred above a grid.
fn col_labels(s: &mut Svg, x: f64, y: f64, cw: f64, labels: &[&str], size: u8) {
    for (j, l) in labels.iter().enumerate() {
        s.text(x + j as f64 * cw + cw / 2.0, y, l, size, tok::INK2, Anchor::Middle);
    }
}

// =================================================================================================
// Chapter 25. From one-hot to embeddings
// =================================================================================================

fn fig_25_1_one_hot() -> Figure {
    let mut s = Svg::new(
        720,
        392,
        "One-hot codes give each word its own column with a single 1; four made-up features describe the same words with fewer, fuller columns",
        "Left: an eleven by eleven grid, one row per content word of the blurbs, with a 1 on the diagonal and 0 elsewhere. Right: the same eleven words described by four hand-set features, crime, place, food and tools, shaded by value.",
    ).min_text(12);
    let words: Vec<&str> = FEATURES.iter().map(|(w, _)| *w).collect();
    let n = words.len();
    let ch = 24.0;
    let y0 = 76.0;
    // left: one-hot
    let x1 = 118.0;
    let onehot: Vec<Vec<f64>> = (0..n).map(|i| (0..n).map(|j| if i == j { 1.0 } else { 0.0 }).collect()).collect();
    let cells1 = heat_cells(&onehot, 1.0, 0);
    s.cells_wh(x1, y0, 22.0, ch, &cells1);
    row_labels(&mut s, x1 - 8.0, y0, ch, &words, 12);
    s.text_bold(x1 + n as f64 * 11.0, 30.0, "one-hot: one column per word", 13, tok::INK, Anchor::Middle);
    s.text(x1 + n as f64 * 11.0, 48.0, "11 columns, a single 1 in each row", 11, tok::INK2, Anchor::Middle);
    s.text(x1 + n as f64 * 11.0, 66.0, "columns in the same order as the rows", 11, tok::MUTED, Anchor::Middle);
    // right: features
    let x2 = 468.0;
    let cw2 = 56.0;
    let feats: Vec<Vec<f64>> = FEATURES.iter().map(|(_, f)| f.to_vec()).collect();
    let cells2: Vec<Vec<Cell>> = feats
        .iter()
        .map(|r| r.iter().map(|v| if *v == 0.0 { cell(numt(0.0, 0)).ink(tok::MUTED) } else { cell(num(*v, 1)).fill(heat(*v, 1.0)) }).collect())
        .collect();
    s.cells_wh(x2, y0, cw2, ch, &cells2);
    col_labels(&mut s, x2, y0 - 8.0, cw2, &FEATURE_NAMES, 12);
    s.text_bold(x2 + 2.0 * cw2, 30.0, "made-up features: four columns", 13, tok::INK, Anchor::Middle);
    s.text(x2 + 2.0 * cw2, 48.0, "how much of each quality a word has", 11, tok::INK2, Anchor::Middle);
    let ym = y0 + n as f64 * ch / 2.0;
    s.arrow(x1 + n as f64 * 22.0 + 14.0, ym, x2 - 14.0, ym, None);
    s.text((x1 + n as f64 * 22.0 + x2) / 2.0, ym - 10.0, "same words", 12, tok::INK2, Anchor::Middle);
    let zeros1 = onehot.iter().flatten().filter(|v| **v == 0.0).count();
    let zeros2 = feats.iter().flatten().filter(|v| **v == 0.0).count();
    let yb = y0 + n as f64 * ch + 24.0;
    s.text(
        360.0,
        yb,
        &format!("one-hot: {zeros1} of {} cells are 0 and no two rows share a 1; features: {zeros2} of {} cells are 0", n * n, n * 4),
        11,
        tok::INK2,
        Anchor::Middle,
    );
    s.text(360.0, yb + 16.0, "the feature values are invented for this book; a learned embedding picks its own columns", 11, tok::MUTED, Anchor::Middle);
    ("b25-1-one-hot".to_string(), s.finish())
}

fn fig_25_2_word_map() -> Figure {
    let mut s = Svg::new(
        720,
        400,
        "Flattened to two directions, the made-up feature vectors put crime words, place words, food words and tool words in four separate corners",
        "A scatter plot of the eleven content words at their coordinates along the two main directions of the four-feature table, with dashed lines joining three pairs and their cosine similarity written beside each line.",
    ).min_text(12);
    let pts: Vec<Vec<f64>> = FEATURES.iter().map(|(_, f)| f.to_vec()).collect();
    let (proj, share, _) = pca2(&pts);
    let (px0, py0, pw, ph) = (60.0, 40.0, 420.0, 320.0);
    let xs: Vec<f64> = proj.iter().map(|p| p[0]).collect();
    let ys: Vec<f64> = proj.iter().map(|p| p[1]).collect();
    let (xmin, xmax) = (xs.iter().cloned().fold(f64::INFINITY, f64::min), xs.iter().cloned().fold(f64::NEG_INFINITY, f64::max));
    let (ymin, ymax) = (ys.iter().cloned().fold(f64::INFINITY, f64::min), ys.iter().cloned().fold(f64::NEG_INFINITY, f64::max));
    let pad = 0.18;
    let sx = |x: f64| px0 + (x - xmin + pad) / (xmax - xmin + 2.0 * pad) * pw;
    let sy = |y: f64| py0 + ph - (y - ymin + pad) / (ymax - ymin + 2.0 * pad) * ph;
    s.rect(px0, py0, pw, ph, tok::SURFACE, Some(tok::GRID));
    s.line(sx(0.0), py0, sx(0.0), py0 + ph, tok::GRID, 1.0);
    s.line(px0, sy(0.0), px0 + pw, sy(0.0), tok::GRID, 1.0);
    s.text(px0 + pw / 2.0, py0 + ph + 20.0, &format!("first direction ({}% of the spread)", numt(share[0] * 100.0, 0)), 11, tok::INK2, Anchor::Middle);
    s.text(px0 - 10.0, py0 + 12.0, &format!("second ({}%)", numt(share[1] * 100.0, 0)), 11, tok::INK2, Anchor::Start);
    let idx = |w: &str| FEATURES.iter().position(|(x, _)| *x == w).unwrap_or(0);
    let pairs = [("mystery", "detective"), ("town", "village"), ("mystery", "recipes")];
    let mut notes: Vec<String> = Vec::new();
    for (a, b) in pairs {
        let (i, j) = (idx(a), idx(b));
        s.path(&format!("M{} {} L{} {}", xy(sx(xs[i])), xy(sy(ys[i])), xy(sx(xs[j])), xy(sy(ys[j]))), tok::AXIS, 1.0);
        notes.push(format!("{a} and {b}: {}", num(cosine(&pts[i], &pts[j]), 2)));
    }
    let dots: Vec<(f64, f64)> = (0..FEATURES.len()).map(|k| (sx(xs[k]), sy(ys[k]))).collect();
    let words: Vec<&str> = FEATURES.iter().map(|(w, _)| *w).collect();
    let spots = place_labels(&dots, &words, 12);
    for (k, (_, f)) in FEATURES.iter().enumerate() {
        let group = (0..4).fold(0, |b, c| if f[c] > f[b] { c } else { b });
        let colour = match group {
            0 => tok::S1,
            1 => tok::S3,
            2 => tok::S2,
            _ => tok::INK2,
        };
        s.dot(dots[k].0, dots[k].1, 5.0, colour, Some(tok::SURFACE));
        let (lx, ly, start) = spots[k];
        s.text(lx, ly, words[k], 12, tok::INK, if start { Anchor::Start } else { Anchor::End });
    }
    // legend and cosines
    let lx = 496.0;
    s.text_bold(lx, 60.0, "largest feature", 12, tok::INK, Anchor::Start);
    for (i, (name, colour)) in [("crime", tok::S1), ("place", tok::S3), ("food", tok::S2), ("tools", tok::INK2)].iter().enumerate() {
        let y = 84.0 + i as f64 * 20.0;
        s.dot(lx + 6.0, y - 4.0, 5.0, colour, Some(tok::SURFACE));
        s.text(lx + 18.0, y, name, 12, tok::INK2, Anchor::Start);
    }
    s.text_bold(lx, 190.0, "cosine in all four features", 12, tok::INK, Anchor::Start);
    for (i, n) in notes.iter().enumerate() {
        s.text(lx, 212.0 + i as f64 * 18.0, n, 11, tok::INK2, Anchor::Start);
    }
    s.text(lx, 290.0, "one-hot codes give 0 for", 11, tok::MUTED, Anchor::Start);
    s.text(lx, 304.0, "every pair of different words", 11, tok::MUTED, Anchor::Start);
    ("b25-2-word-map".to_string(), s.finish())
}

fn fig_25_3_lookup() -> Figure {
    let mut s = Svg::new(
        720,
        380,
        "Multiplying a one-hot row by the feature table picks out one row of the table: an embedding layer is a lookup",
        "A row of eleven cells with a single 1 at detective, times the eleven by four feature table with the detective row outlined, equals that row of four numbers.",
    ).min_text(12);
    let words: Vec<&str> = FEATURES.iter().map(|(w, _)| *w).collect();
    let n = words.len();
    let pick = words.iter().position(|w| *w == "detective").unwrap_or(0);
    let table: Vec<Vec<f64>> = FEATURES.iter().map(|(_, f)| f.to_vec()).collect();
    let onehot: Vec<f64> = (0..n).map(|i| if i == pick { 1.0 } else { 0.0 }).collect();
    // result = onehot × table, computed
    let result: Vec<f64> = (0..4).map(|j| (0..n).map(|i| onehot[i] * table[i][j]).sum()).collect();
    let ch = 24.0;
    let y0 = 62.0;
    // one-hot as a column so the row it selects lines up
    let xo = 110.0;
    let col: Vec<Vec<Cell>> = onehot
        .iter()
        .map(|v| vec![if *v == 1.0 { cell("1").fill(tok::FILL3).stroke(tok::S1) } else { cell("0").ink(tok::MUTED) }])
        .collect();
    s.cells_wh(xo, y0, 30.0, ch, &col);
    row_labels(&mut s, xo - 8.0, y0, ch, &words, 12);
    s.text(xo + 15.0, y0 - 12.0, "one-hot", 12, tok::INK, Anchor::Middle);
    s.text(xo + 15.0, y0 + n as f64 * ch + 18.0, "stood on end:", 11, tok::INK2, Anchor::Middle);
    s.text(xo + 15.0, y0 + n as f64 * ch + 32.0, "each entry meets the row beside it", 11, tok::INK2, Anchor::Middle);
    let xt = 230.0;
    let cw = 50.0;
    let cells_t: Vec<Vec<Cell>> = table
        .iter()
        .enumerate()
        .map(|(i, r)| {
            r.iter()
                .map(|v| {
                    let c0 = if *v == 0.0 { cell("0").ink(tok::MUTED) } else { cell(num(*v, 1)) };
                    if i == pick { c0.fill(tok::FILL2) } else { c0 }
                })
                .collect()
        })
        .collect();
    s.text(185.0, y0 + n as f64 * ch / 2.0 + 4.0, "×", 16, tok::INK2, Anchor::Middle);
    s.cells_wh(xt, y0, cw, ch, &cells_t);
    col_labels(&mut s, xt, y0 - 12.0, cw, &FEATURE_NAMES, 12);
    s.rect_bold(xt - 2.0, y0 + pick as f64 * ch - 2.0, 4.0 * cw + 4.0, ch + 4.0, "none", tok::S1);
    s.text(xt + 2.0 * cw, y0 + n as f64 * ch + 18.0, "the table: 11 rows, 4 columns", 11, tok::INK2, Anchor::Middle);
    let xr = 500.0;
    let yr = y0 + pick as f64 * ch;
    s.text(xr - 28.0, yr + ch / 2.0 + 5.0, "=", 16, tok::INK2, Anchor::Middle);
    let rc: Vec<Vec<Cell>> = vec![result.iter().map(|v| if *v == 0.0 { cell("0").ink(tok::MUTED) } else { cell(num(*v, 1)).fill(tok::FILL2) }).collect()];
    s.cells_wh(xr, yr, 46.0, ch, &rc);
    col_labels(&mut s, xr, yr - 8.0, 46.0, &FEATURE_NAMES, 11);
    s.text(xr + 92.0, yr + ch + 20.0, "detective's embedding", 12, tok::INK, Anchor::Middle);
    s.text(xr + 92.0, yr + ch + 38.0, "the 1 keeps its row;", 11, tok::INK2, Anchor::Middle);
    s.text(xr + 92.0, yr + ch + 52.0, "every 0 wipes out its row", 11, tok::INK2, Anchor::Middle);
    ("b25-3-lookup".to_string(), s.finish())
}

fn fig_25_4_pipeline() -> Figure {
    let mut s = Svg::new(
        720,
        260,
        "An embedding model turns texts of any length into vectors of one fixed length, and similarity between texts becomes similarity between vectors",
        "Three texts of different lengths, two blurbs and a review, pass through an embedding model box and come out as three equal-length rows of cells; two of the rows feed a cosine similarity box.",
    ).min_text(12);
    let inputs = [("B3: recipes from a village kitchen", "5 words"), ("B2: a small town detective", "and a quiet mystery: 8 words"), ("a customer's review", "about 100 words")];
    let (bx, bw, bh) = (286.0, 110.0, 150.0);
    let ys = [58.0, 118.0, 178.0];
    for (i, (label, detail)) in inputs.iter().enumerate() {
        let y = ys[i];
        s.text(14.0, y - 2.0, label, 11, tok::INK, Anchor::Start);
        s.text(14.0, y + 13.0, detail, 11, tok::MUTED, Anchor::Start);
        s.arrow(250.0, y + 3.0, bx - 4.0, y + 3.0, None);
    }
    s.rect_bold(bx, 40.0, bw, bh, tok::FILL1, tok::S1);
    s.text(bx + bw / 2.0, 108.0, "embedding", 12, tok::INK, Anchor::Middle);
    s.text(bx + bw / 2.0, 124.0, "model", 12, tok::INK, Anchor::Middle);
    let cw = 20.0;
    for (i, y) in ys.iter().enumerate() {
        s.arrow(bx + bw + 4.0, y + 3.0, 424.0, y + 3.0, None);
        let row: Vec<Cell> = (0..8).map(|k| if k == 6 { cell("…").ink(tok::INK2) } else { cell("").fill(tok::FILL1) }).collect();
        s.cells_wh(430.0, y - 8.0, cw, 22.0, &[row]);
        if i < 2 {
            s.line(430.0 + 8.0 * cw + 2.0, y + 3.0, 612.0, y + 3.0, tok::AXIS, 1.0);
        }
    }
    s.path("M612 61 L612 121", tok::AXIS, 1.0);
    s.arrow(612.0, 91.0, 626.0, 91.0, None);
    s.labelled_box2(628.0, 70.0, 84.0, 42.0, "cosine", "one number", tok::SURFACE, tok::AXIS);
    s.text(510.0, 222.0, "every row has the same length, whatever went in", 11, tok::INK2, Anchor::Middle);
    s.text(510.0, 238.0, "(384 numbers for the model all-MiniLM-L6-v2)", 11, tok::MUTED, Anchor::Middle);
    ("b25-4-pipeline".to_string(), s.finish())
}

// =================================================================================================
// Chapter 26. Counting words: bag of words and TF-IDF
// =================================================================================================

fn fig_26_1_counts() -> Figure {
    let mut s = Svg::new(
        720,
        350,
        "The five blurbs as a table of word counts: eighteen columns, most of them zero in any one blurb",
        "Two side-by-side grids, the eighteen words of the blurbs in alphabetical order as rows and the blurbs B1 to B5 as columns; non-zero counts are shaded, zeros are muted.",
    ).min_text(12);
    let vocab = vocabulary(&BLURBS);
    let counts = count_matrix(&BLURBS, &vocab);
    let (cw, ch) = (40.0, 25.0);
    let y0 = 64.0;
    let half = vocab.len().div_ceil(2);
    for (p, x0) in [(0usize, 130.0), (1usize, 470.0)] {
        let range = (p * half)..(vocab.len().min((p + 1) * half));
        let rows: Vec<Vec<f64>> = range.clone().map(|j| (0..5).map(|b| counts[b][j]).collect()).collect();
        let labels: Vec<&str> = range.clone().map(|j| vocab[j]).collect();
        s.cells_wh(x0, y0, cw, ch, &heat_cells(&rows, 2.0, 0));
        row_labels(&mut s, x0 - 8.0, y0, ch, &labels, 12);
        col_labels(&mut s, x0, y0 - 10.0, cw, &["B1", "B2", "B3", "B4", "B5"], 12);
    }
    let cells = counts.len() * vocab.len();
    let zeros = counts.iter().flatten().filter(|v| **v == 0.0).count();
    let yb = y0 + half as f64 * ch + 28.0;
    s.text(
        360.0,
        yb,
        &format!("{} words in the vocabulary × 5 blurbs = {cells} cells; {zeros} of them are 0 ({}%)", vocab.len(), numt(zeros as f64 / cells as f64 * 100.0, 0)),
        12,
        tok::INK,
        Anchor::Middle,
    );
    s.text(360.0, yb + 18.0, "each column of the table is one blurb's bag of words; the order of the words inside a blurb is gone", 11, tok::INK2, Anchor::Middle);
    s.text_bold(360.0, 30.0, "how often each word occurs in each blurb", 13, tok::INK, Anchor::Middle);
    ("b26-1-counts".to_string(), s.finish())
}

fn fig_26_2_idf() -> Figure {
    let mut s = Svg::new(
        720,
        300,
        "In blurb B1 the word a occurs twice but gets the smallest tf-idf weight, because four of the five blurbs contain it",
        "A table for the six different words of blurb B1: count in B1, number of blurbs containing the word, idf equal to ln of 5 over that number, and a bar for tf-idf equal to count times idf.",
    ).min_text(12);
    let vocab = vocabulary(&BLURBS);
    let counts = count_matrix(&BLURBS, &vocab);
    let df = doc_freq(&counts);
    let w_idf = idf(&df, BLURBS.len());
    let tf = tfidf(&counts, &w_idf);
    // the words of B1 in reading order, each once
    let mut seen: Vec<&str> = Vec::new();
    for t in tokens(BLURBS[0]) {
        if !seen.contains(&t) {
            seen.push(t);
        }
    }
    let heads = ["word", "count in B1", "blurbs with it", "idf = ln(5 ÷ that)", "tf-idf = count × idf"];
    let xs = [70.0, 160.0, 260.0, 380.0, 480.0];
    let y0 = 70.0;
    for (i, h) in heads.iter().enumerate() {
        s.text_bold(xs[i], y0 - 16.0, h, 12, tok::INK, if i == 0 { Anchor::End } else if i == 4 { Anchor::Start } else { Anchor::Middle });
    }
    let max_w = seen.iter().map(|w| tf[0][vocab.iter().position(|v| v == w).unwrap_or(0)]).fold(0.0, f64::max);
    for (r, w) in seen.iter().enumerate() {
        let j = vocab.iter().position(|v| v == w).unwrap_or(0);
        let y = y0 + r as f64 * 28.0;
        if r % 2 == 0 {
            s.rect(12.0, y - 2.0, 696.0, 28.0, tok::NEUTRAL, None);
        }
        s.text(xs[0], y + 16.0, w, 12, tok::INK, Anchor::End);
        s.text(xs[1], y + 16.0, &numt(counts[0][j], 0), 12, tok::INK, Anchor::Middle);
        s.text(xs[2], y + 16.0, &format!("{} of 5", df[j]), 12, tok::INK, Anchor::Middle);
        s.text(xs[3], y + 16.0, &num(w_idf[j], 3), 12, tok::INK, Anchor::Middle);
        let bw = tf[0][j] / max_w * 150.0;
        s.rect(xs[4], y + 3.0, bw, 18.0, tok::S1, None);
        s.text(xs[4] + bw + 6.0, y + 16.0, &num(tf[0][j], 3), 12, tok::INK, Anchor::Start);
    }
    let yb = y0 + seen.len() as f64 * 28.0 + 22.0;
    s.text(360.0, yb, "a word found in every blurb would get idf = ln(5 ÷ 5) = 0 and vanish; rare words weigh most", 11, tok::INK2, Anchor::Middle);
    s.text(360.0, yb + 16.0, "here the rarest word is the little word in, which is why real systems also drop stop words", 11, tok::INK2, Anchor::Middle);
    ("b26-2-idf".to_string(), s.finish())
}

fn fig_26_3_cosine() -> Figure {
    let mut s = Svg::new(
        720,
        330,
        "Measured by raw counts, B1 looks as close to the bicycle blurb as to the recipe blurb; tf-idf shrinks that false likeness and keeps the real one",
        "Two five by five heat maps of cosine similarity between the blurbs, the left from raw counts and the right from tf-idf weights, with the B1 and B5 cell outlined in both.",
    ).min_text(12);
    let vocab = vocabulary(&BLURBS);
    let counts = count_matrix(&BLURBS, &vocab);
    let w_idf = idf(&doc_freq(&counts), BLURBS.len());
    let tf = tfidf(&counts, &w_idf);
    let names = ["B1", "B2", "B3", "B4", "B5"];
    let cw = 50.0;
    let y0 = 76.0;
    for (m, x0, title) in [(cosine_matrix(&counts), 70.0, "cosine of raw counts"), (cosine_matrix(&tf), 420.0, "cosine of tf-idf weights")] {
        s.text_bold(x0 + 2.5 * cw, 34.0, title, 13, tok::INK, Anchor::Middle);
        s.cells_wh(x0, y0, cw, 34.0, &heat_cells(&m, 1.0, 2));
        row_labels(&mut s, x0 - 8.0, y0 - 4.5, 34.0, &names, 12);
        col_labels(&mut s, x0, y0 - 10.0, cw, &names, 12);
        s.rect_bold(x0 + 4.0 * cw, y0, cw, 34.0, "none", tok::S2);
        s.rect_bold(x0, y0 + 4.0 * 34.0, cw, 34.0, "none", tok::S2);
    }
    let cc = cosine_matrix(&counts);
    let ct = cosine_matrix(&tf);
    let yb = y0 + 5.0 * 34.0 + 30.0;
    s.text(
        360.0,
        yb,
        &format!("B1 with B5 (they share only the word a): {} by counts, {} by tf-idf", num(cc[0][4], 2), num(ct[0][4], 2)),
        12,
        tok::INK,
        Anchor::Middle,
    );
    s.text(
        360.0,
        yb + 18.0,
        &format!("B1 with B2 (same story, shared quiet, mystery, small, town): {} by counts, {} by tf-idf", num(cc[0][1], 2), num(ct[0][1], 2)),
        11,
        tok::INK2,
        Anchor::Middle,
    );
    ("b26-3-cosine".to_string(), s.finish())
}

fn fig_26_4_order() -> Figure {
    let mut s = Svg::new(
        720,
        250,
        "A blurb and the same words with detective and village swapped, which says something quite different, produce exactly the same bag of words",
        "Two sentences drawn as rows of word boxes, the second the first with detective and village swapped, each feeding an arrow into its count row over five words; the two count rows are identical.",
    ).min_text(12);
    let a: &'static str = BLURBS[3];
    let b: &'static str = "the village returns to the detective";
    let vocab = vocabulary(&[a, b]);
    let counts = count_matrix(&[a, b], &vocab);
    let same = counts[0] == counts[1];
    for (r, text) in [a, b].iter().enumerate() {
        let y = 50.0 + r as f64 * 90.0;
        let mut x = 14.0;
        for t in tokens(text) {
            let w = 12.0 + 7.2 * t.chars().count() as f64;
            s.rect(x, y, w, 26.0, if t == "detective" || t == "village" { tok::FILL2 } else { tok::SURFACE }, Some(tok::AXIS));
            s.text(x + w / 2.0, y + 17.0, t, 12, tok::INK, Anchor::Middle);
            x += w + 4.0;
        }
        s.arrow(x + 6.0, y + 13.0, 398.0, y + 13.0, None);
        let row: Vec<Cell> = counts[r].iter().map(|v| cell(numt(*v, 0)).fill(heat(*v, 2.0))).collect();
        s.cells_wh(404.0, y, 60.0, 26.0, &[row]);
        if r == 0 {
            col_labels(&mut s, 404.0, y - 8.0, 60.0, &vocab, 12);
        }
    }
    s.text(
        360.0,
        214.0,
        if same { "identical rows: the bag cannot tell the blurb from the same words with two swapped" } else { "the rows differ" },
        12,
        tok::INK,
        Anchor::Middle,
    );
    ("b26-4-order".to_string(), s.finish())
}

// =================================================================================================
// Chapter 27. Word2Vec: CBOW and skip-gram
// =================================================================================================

/// The LCG named in the book's shared data (x ← 6364136223846793005·x + 1442695040888963407 mod 2⁶⁴), here only to
/// draw small starting weights.
struct Lcg(u64);

impl Lcg {
    /// A value in [−0.5, 0.5).
    fn next(&mut self) -> f64 {
        self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        ((self.0 >> 11) as f64 / (1u64 << 53) as f64) - 0.5
    }
}

const SG_WINDOW: usize = 2;
const SG_DIM: usize = 2;
const SG_RATE: f64 = 0.05;
const SG_PASSES: usize = 3000;
const SG_SEED: u64 = 20260929;

/// (centre, neighbour) index pairs of every blurb for a window of `win` words on each side.
fn skipgram_pairs(vocab: &[&str], win: usize) -> Vec<(usize, usize)> {
    let mut pairs = Vec::new();
    for b in BLURBS {
        let t: Vec<usize> = tokens(b).iter().map(|w| vocab.iter().position(|v| v == w).unwrap_or(0)).collect();
        for i in 0..t.len() {
            for j in i.saturating_sub(win)..(i + win + 1).min(t.len()) {
                if j != i {
                    pairs.push((t[i], t[j]));
                }
            }
        }
    }
    pairs
}

/// Skip-gram with a full softmax over the vocabulary (Mikolov et al. 2013b, eq. 2), trained by plain stochastic
/// gradient descent on the five blurbs. Returns the input vectors (one row per vocabulary word).
fn skipgram(vocab: &[&str]) -> Vec<Vec<f64>> {
    let pairs = skipgram_pairs(vocab, SG_WINDOW);
    let v = vocab.len();
    let mut rng = Lcg(SG_SEED);
    let mut w_in: Vec<Vec<f64>> = (0..v).map(|_| (0..SG_DIM).map(|_| rng.next() * 0.2).collect()).collect();
    let mut w_out: Vec<Vec<f64>> = (0..v).map(|_| (0..SG_DIM).map(|_| rng.next() * 0.2).collect()).collect();
    for _ in 0..SG_PASSES {
        for &(c, o) in &pairs {
            let h = w_in[c].clone();
            let scores: Vec<f64> = (0..v).map(|k| dot(&h, &w_out[k])).collect();
            let p = softmax(&scores);
            let mut grad_h = vec![0.0; SG_DIM];
            for k in 0..v {
                let g = p[k] - if k == o { 1.0 } else { 0.0 };
                for t in 0..SG_DIM {
                    grad_h[t] += g * w_out[k][t];
                    w_out[k][t] -= SG_RATE * g * h[t];
                }
            }
            for t in 0..SG_DIM {
                w_in[c][t] -= SG_RATE * grad_h[t];
            }
        }
    }
    w_in
}

fn softmax(z: &[f64]) -> Vec<f64> {
    let m = z.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let e: Vec<f64> = z.iter().map(|x| (x - m).exp()).collect();
    let sum: f64 = e.iter().sum();
    e.iter().map(|x| x / sum).collect()
}

/// A row of word boxes; returns the x of each box's left edge and its width.
fn word_boxes(s: &mut Svg, x0: f64, y: f64, words: &[&str], fills: &[&'static str]) -> Vec<(f64, f64)> {
    let mut x = x0;
    let mut out = Vec::new();
    for (i, w) in words.iter().enumerate() {
        let bw = text_w(w, 12) + 16.0;
        s.rect(x, y, bw, 26.0, fills[i], Some(tok::AXIS));
        s.text(x + bw / 2.0, y + 17.0, w, 12, tok::INK, Anchor::Middle);
        out.push((x, bw));
        x += bw + 5.0;
    }
    out
}

fn fig_27_1_window() -> Figure {
    let mut s = Svg::new(
        720,
        300,
        "A window slides along the blurb; each position pairs the centre word with its neighbours, which become the training examples",
        "Blurb B2 drawn twice as a row of word boxes: first with detective as the centre and two neighbours on each side outlined, then with quiet as the centre near the end, where the window is cut short; the resulting pairs are listed beside each row.",
    ).min_text(12);
    let words = tokens(BLURBS[1]);
    for (row, centre) in [(0usize, 3usize), (1usize, 6usize)] {
        let y = 52.0 + row as f64 * 116.0;
        let lo = centre.saturating_sub(SG_WINDOW);
        let hi = (centre + SG_WINDOW).min(words.len() - 1);
        let fills: Vec<&'static str> = (0..words.len()).map(|i| if i == centre { tok::FILL3 } else if i >= lo && i <= hi { tok::FILL1 } else { tok::SURFACE }).collect();
        let boxes = word_boxes(&mut s, 24.0, y, &words, &fills);
        let (wx0, _) = boxes[lo];
        let (wx1, ww1) = boxes[hi];
        s.rect_dashed(wx0 - 4.0, y - 5.0, wx1 + ww1 - wx0 + 8.0, 36.0, tok::S2);
        s.text(wx0 - 4.0, y - 10.0, &format!("window: {} words each side", SG_WINDOW), 11, tok::S2, Anchor::Start);
        let neighbours: Vec<&str> = (lo..=hi).filter(|i| *i != centre).map(|i| words[i]).collect();
        let pairs: Vec<String> = neighbours.iter().map(|n| format!("({}, {n})", words[centre])).collect();
        s.text(24.0, y + 50.0, &format!("skip-gram examples: {}", pairs.join("  ")), 12, tok::INK, Anchor::Start);
        s.text(24.0, y + 68.0, &format!("CBOW example: ({}) → {}", neighbours.join(", "), words[centre]), 12, tok::INK2, Anchor::Start);
    }
    let vocab = vocabulary(&BLURBS);
    let total = skipgram_pairs(&vocab, SG_WINDOW).len();
    s.text(360.0, 288.0, &format!("sliding the window over all five blurbs gives {total} (centre, neighbour) pairs"), 12, tok::INK, Anchor::Middle);
    ("b27-1-window".to_string(), s.finish())
}

fn fig_27_2_two_nets() -> Figure {
    let mut s = Svg::new(
        720,
        344,
        "CBOW guesses the centre word from its neighbours; skip-gram guesses each neighbour from the centre word; both look words up in the same table",
        "Two left-to-right flows. Top: four neighbour words enter a lookup box, their rows are averaged, every vocabulary word is scored and a softmax gives probabilities, and the target is detective. Bottom: detective enters the lookup box, its row scores every vocabulary word, and the targets are the four neighbours in turn.",
    ).min_text(12);
    let vocab = vocabulary(&BLURBS);
    let v = vocab.len();
    let words = tokens(BLURBS[1]);
    let neighbours = [words[1], words[2], words[4], words[5]];
    let lookup = format!("{v} rows × {SG_DIM} numbers");
    // CBOW
    s.text_bold(16.0, 26.0, "CBOW", 13, tok::INK, Anchor::Start);
    for (i, n) in neighbours.iter().enumerate() {
        let y = 40.0 + i as f64 * 28.0;
        s.labelled_box(16.0, y, 90.0, 22.0, n, tok::FILL1, tok::AXIS);
        s.arrow(108.0, y + 11.0, 150.0, 92.0, None);
    }
    s.labelled_box2(152.0, 70.0, 150.0, 44.0, "look up 4 rows", &lookup, tok::SURFACE, tok::AXIS);
    s.arrow(304.0, 92.0, 320.0, 92.0, None);
    s.labelled_box2(322.0, 70.0, 98.0, 44.0, "average", "one vector", tok::SURFACE, tok::AXIS);
    s.arrow(422.0, 92.0, 438.0, 92.0, None);
    s.labelled_box2(440.0, 70.0, 158.0, 44.0, &format!("score all {v} words"), "softmax: probabilities", tok::SURFACE, tok::AXIS);
    s.arrow(600.0, 92.0, 620.0, 92.0, None);
    s.labelled_box(622.0, 81.0, 84.0, 22.0, words[3], tok::FILL3, tok::S3);
    s.line(16.0, 168.0, 704.0, 168.0, tok::GRID, 1.0);
    // skip-gram
    s.text_bold(16.0, 194.0, "skip-gram", 13, tok::INK, Anchor::Start);
    s.labelled_box(16.0, 249.0, 90.0, 22.0, words[3], tok::FILL3, tok::S3);
    s.arrow(108.0, 260.0, 150.0, 260.0, None);
    s.labelled_box2(152.0, 238.0, 150.0, 44.0, "look up 1 row", &lookup, tok::SURFACE, tok::AXIS);
    s.arrow(304.0, 260.0, 438.0, 260.0, None);
    s.labelled_box2(440.0, 238.0, 158.0, 44.0, &format!("score all {v} words"), "softmax: probabilities", tok::SURFACE, tok::AXIS);
    for (i, n) in neighbours.iter().enumerate() {
        let y = 208.0 + i as f64 * 28.0;
        s.arrow(600.0, 260.0, 620.0, y + 11.0, None);
        s.labelled_box(622.0, y, 84.0, 22.0, n, tok::FILL1, tok::AXIS);
    }
    s.text(380.0, 332.0, "one target at a time: four training examples from one window", 11, tok::INK2, Anchor::Middle);
    ("b27-2-two-nets".to_string(), s.finish())
}

fn fig_27_3_rows() -> Figure {
    let vocab = vocabulary(&BLURBS);
    let w = skipgram(&vocab);
    let pick = vocab.iter().position(|x| *x == "mystery").unwrap_or(0);
    let mut s = Svg::new(
        720,
        420,
        "After training, each row of the input table is a word's embedding; the one-hot code of mystery picks out its two numbers",
        "A column of eighteen one-hot entries with a 1 at mystery, beside the eighteen by two table learned by skip-gram on the five blurbs, with the mystery row outlined, and the two numbers of that row repeated on the right.",
    ).min_text(12);
    let (ch, y0) = (18.0, 44.0);
    let xo = 112.0;
    let col: Vec<Vec<Cell>> = (0..vocab.len()).map(|i| vec![if i == pick { cell("1").fill(tok::FILL3) } else { cell("0").ink(tok::MUTED) }]).collect();
    s.cells_wh(xo, y0, 28.0, ch, &col);
    row_labels(&mut s, xo - 8.0, y0, ch, &vocab, 11);
    s.text(xo + 14.0, y0 - 10.0, "one-hot", 11, tok::INK2, Anchor::Middle);
    let xt = 200.0;
    let cw = 64.0;
    let rows: Vec<Vec<Cell>> = w
        .iter()
        .enumerate()
        .map(|(i, r)| r.iter().map(|x| { let c0 = cell(num(*x, 2)); if i == pick { c0.fill(tok::FILL2) } else { c0 } }).collect())
        .collect();
    s.cells_wh(xt, y0, cw, ch, &rows);
    col_labels(&mut s, xt, y0 - 10.0, cw, &["number 1", "number 2"], 11);
    s.rect_bold(xt - 2.0, y0 + pick as f64 * ch - 2.0, 2.0 * cw + 4.0, ch + 4.0, "none", tok::S1);
    s.text(166.0, y0 + vocab.len() as f64 * ch / 2.0 + 4.0, "×", 16, tok::INK2, Anchor::Middle);
    let yr = y0 + pick as f64 * ch;
    s.text(372.0, yr + 14.0, "=", 16, tok::INK2, Anchor::Middle);
    let r: Vec<Vec<Cell>> = vec![w[pick].iter().map(|x| cell(num(*x, 2)).fill(tok::FILL2)).collect()];
    s.cells_wh(392.0, yr - 4.0, 70.0, 26.0, &r);
    s.text(462.0, yr + 44.0, "the embedding of mystery", 12, tok::INK, Anchor::Middle);
    let notes = [
        "trained here on the five blurbs:".to_string(),
        format!("skip-gram, window of {SG_WINDOW}, {SG_DIM} numbers per word,"),
        format!("{} passes over {} pairs, step size {SG_RATE},", thousands(SG_PASSES as u64), skipgram_pairs(&vocab, SG_WINDOW).len()),
        "small random starting values".to_string(),
        "the Word2Vec paper used up to 1,000 numbers".to_string(),
        "per word and up to 6 billion words of text".to_string(),
    ];
    for (i, n) in notes.iter().enumerate() {
        s.text(360.0, 312.0 + i as f64 * 16.0, n, 12, if i < 4 { tok::INK2 } else { tok::MUTED }, Anchor::Start);
    }
    ("b27-3-rows".to_string(), s.finish())
}

fn fig_27_4_cosines() -> Figure {
    let vocab = vocabulary(&BLURBS);
    let w = skipgram(&vocab);
    let at = |x: &str| w[vocab.iter().position(|v| *v == x).unwrap_or(0)].clone();
    let mut s = Svg::new(
        720,
        380,
        "Cosine similarity reads the angle between two vectors: 1 for the same direction, 0 for a right angle, −1 for opposite; in the trained table mystery and town point almost the same way",
        "Left: three small pairs of arrows at 0, 90 and 180 degrees with cosines 1, 0 and −1. Right: the trained two-number vectors of six words drawn as arrows from a common origin, with the cosines of three pairs listed underneath.",
    ).min_text(12);
    // left: the three reference angles
    s.text_bold(110.0, 30.0, "what the cosine says", 13, tok::INK, Anchor::Middle);
    for (i, (deg, label)) in [(0.0f64, "same direction"), (90.0, "right angle"), (180.0, "opposite")].iter().enumerate() {
        let cy = 90.0 + i as f64 * 96.0;
        let cx = 70.0;
        let r = 44.0;
        s.line(cx, cy, cx + r, cy, tok::S1, 2.5);
        let (dx, dy) = (deg.to_radians().cos() * r * 0.8, -deg.to_radians().sin() * r * 0.8);
        s.line(cx, cy + if *deg == 0.0 { 6.0 } else { 0.0 }, cx + dx, cy + dy + if *deg == 0.0 { 6.0 } else { 0.0 }, tok::S2, 2.5);
        s.dot(cx, cy, 3.0, tok::INK2, None);
        s.text(130.0, cy - 2.0, &format!("cos {}° = {}", numt(*deg, 0), numt(deg.to_radians().cos(), 0)), 12, tok::INK, Anchor::Start);
        s.text(130.0, cy + 14.0, label, 11, tok::INK2, Anchor::Start);
    }
    // right: trained vectors
    let chosen = ["mystery", "town", "quiet", "small", "recipes", "kitchen"];
    let colours = [tok::S1, tok::S1, tok::S3, tok::S3, tok::S2, tok::S2];
    let raw: Vec<Vec<f64>> = chosen.iter().map(|c| at(c)).collect();
    // turn the whole picture so the fan of arrows opens to the right; a common turn changes no angle
    let (mx, my) = raw.iter().fold((0.0, 0.0), |(a, b), v| (a + v[0] / norm(v), b + v[1] / norm(v)));
    let phi = -my.atan2(mx);
    let vecs: Vec<Vec<f64>> = raw.iter().map(|v| vec![v[0] * phi.cos() - v[1] * phi.sin(), v[0] * phi.sin() + v[1] * phi.cos()]).collect();
    let (ox, oy, r) = (316.0, 196.0, 104.0);
    // a faint unit circle: every arrow is drawn at the same length because only its direction matters to the cosine
    let mut circle = String::new();
    for k in 0..=72 {
        let a = k as f64 * 5.0f64.to_radians();
        circle.push_str(&format!("{}{} {}", if k == 0 { "M" } else { " L" }, xy(ox + r * a.cos()), xy(oy - r * a.sin())));
    }
    s.path(&circle, tok::GRID, 1.0);
    s.line(ox - r - 10.0, oy, ox + r + 10.0, oy, tok::GRID, 1.0);
    s.line(ox, oy - r - 10.0, ox, oy + r + 10.0, tok::GRID, 1.0);
    let tips: Vec<(f64, f64)> = vecs.iter().map(|v| (ox + v[0] / norm(v) * r, oy - v[1] / norm(v) * r)).collect();
    for (k, v) in tips.iter().enumerate() {
        s.line(ox, oy, v.0, v.1, colours[k], 2.0);
        s.dot(v.0, v.1, 4.0, colours[k], Some(tok::SURFACE));
    }
    // labels sit just outside the circle along each arrow; arrows closer than 4 degrees share one label
    let angles: Vec<f64> = vecs.iter().map(|v| v[1].atan2(v[0])).collect();
    let mut done = vec![false; chosen.len()];
    for k in 0..chosen.len() {
        if done[k] {
            continue;
        }
        let group: Vec<usize> = (k..chosen.len()).filter(|j| !done[*j] && (angles[*j] - angles[k]).abs() < 4f64.to_radians()).collect();
        let names: Vec<&str> = group.iter().map(|j| chosen[*j]).collect();
        for j in &group {
            done[*j] = true;
        }
        let a = angles[k];
        let (lx, ly) = (ox + (r + 12.0) * a.cos(), oy - (r + 12.0) * a.sin() + 4.0 + if a.sin() < -0.5 { 8.0 } else { 0.0 });
        let anchor = if a.cos() > 0.25 { Anchor::Start } else if a.cos() < -0.25 { Anchor::End } else { Anchor::Middle };
        s.text(lx, ly, &names.join(" and "), 12, tok::INK, anchor);
    }
    s.text(ox, 40.0, "six rows of the trained table, as directions", 11, tok::INK2, Anchor::Middle);
    s.text(ox, 352.0, "arrows drawn at one length and the picture turned as a whole:", 11, tok::MUTED, Anchor::Middle);
    s.text(ox, 366.0, "neither changes an angle, so neither changes a cosine", 11, tok::MUTED, Anchor::Middle);
    let pairs = [("mystery", "town"), ("quiet", "small"), ("recipes", "kitchen"), ("mystery", "kitchen")];
    for (i, (a, b)) in pairs.iter().enumerate() {
        s.text(544.0, 84.0 + i as f64 * 20.0, &format!("{a}, {b}: {}", num(cosine(&at(a), &at(b)), 3)), 12, tok::INK, Anchor::Start);
    }
    s.text_bold(544.0, 62.0, "cosines", 12, tok::INK, Anchor::Start);
    ("b27-4-cosines".to_string(), s.finish())
}

/// Figure 27.5, left: made-up positions, set by hand to show the idea and not learned from anything, for three pairs of
/// words that differ by one shared step. The figure prints each position beside its word and, like the text, says that
/// they are made up.
const ANALOGY_WORDS: [(&str, [f64; 2]); 6] = [
    ("man", [-2.5, -1.5]),
    ("woman", [-2.0, 0.5]),
    ("uncle", [-0.5, -1.0]),
    ("aunt", [0.0, 1.0]),
    ("king", [2.0, -1.5]),
    ("queen", [2.5, 0.5]),
];

/// Figure 27.5, right: the question put to the tiny trained table of Figure 27.3, "quiet is to mystery as small is to
/// what?", and its expected answer (the blurbs say "a quiet mystery" and "a small town").
const TOY_QUESTION: [&str; 4] = ["quiet", "mystery", "small", "town"];

/// The words of `vocab` other than the three question words, ranked by cosine with b − a + c (the Word2Vec papers' own
/// test): (word, cosine), best first.
fn analogy_ranking(vocab: &[&str], rows: &[Vec<f64>], a: &str, b: &str, c: &str) -> Vec<(String, f64)> {
    let at = |x: &str| rows[vocab.iter().position(|v| *v == x).unwrap_or(0)].clone();
    let target: Vec<f64> = (0..rows[0].len()).map(|k| at(b)[k] - at(a)[k] + at(c)[k]).collect();
    let mut ranked: Vec<(String, f64)> = vocab
        .iter()
        .filter(|v| **v != a && **v != b && **v != c)
        .map(|v| (v.to_string(), cosine(&target, &at(v))))
        .collect();
    ranked.sort_by(|p, q| q.1.partial_cmp(&p.1).unwrap_or(std::cmp::Ordering::Equal));
    ranked
}

/// An ordinal in words for the small ranks a figure names (first to tenth), in digits beyond.
fn ordinal(n: usize) -> String {
    const WORDS: [&str; 10] = ["first", "second", "third", "fourth", "fifth", "sixth", "seventh", "eighth", "ninth", "tenth"];
    if (1..=10).contains(&n) { WORDS[n - 1].to_string() } else { format!("{n}th") }
}

fn fig_27_5_analogy() -> Figure {
    let words: Vec<&str> = ANALOGY_WORDS.iter().map(|w| w.0).collect();
    let rows: Vec<Vec<f64>> = ANALOGY_WORDS.iter().map(|w| w.1.to_vec()).collect();
    let at = |x: &str| rows[words.iter().position(|v| *v == x).unwrap_or(0)].clone();
    let step = [at("woman")[0] - at("man")[0], at("woman")[1] - at("man")[1]];
    let target: Vec<f64> = (0..2).map(|k| at("king")[k] - at("man")[k] + at("woman")[k]).collect();
    let [ma, mb, mc] = ["man", "king", "woman"];
    let made_up = analogy_ranking(&words, &rows, ma, mb, mc);
    let vocab = vocabulary(&BLURBS);
    let trained = skipgram(&vocab);
    let [qa, qb, qc, want] = TOY_QUESTION;
    let toy = analogy_ranking(&vocab, &trained, qa, qb, qc);
    let want_rank = toy.iter().position(|r| r.0 == want).map_or(0, |i| i + 1);
    let pair = |v: &[f64]| format!("({}, {})", numt(v[0], 2), numt(v[1], 2));
    let rank_words = ordinal(want_rank);
    let labels: Vec<String> = ANALOGY_WORDS.iter().map(|(w, p)| format!("{w} {}", pair(p))).collect();
    let others: Vec<String> = made_up.iter().map(|r| format!("{} {}", r.0, num(r.1, 2))).collect();
    let mut s = Svg::new(
        720,
        360,
        "A relation between two words can be a direction: when man to woman is the same step as king to queen, king minus man plus woman lands on queen",
        &format!(
            "Left: six words at made-up positions set by hand and printed beside them, {}; three grey arrows, from man to woman, uncle to aunt and king to queen, are the same step {}, and king − man + woman = {}, where queen sits; ranked by cosine with it, leaving out the question words {ma}, {mb} and {mc}: {}. Right: the tiny trained table of Figure 27.3 asked which word is to {qc} as {qb} is to {qa}: its nearest word to {qb} − {qa} + {qc} is {} ({}), and {want} comes {rank_words}.",
            labels.join(", "),
            pair(&step),
            pair(&target),
            others.join(", "),
            toy[0].0,
            num(toy[0].1, 3)
        ),
    ).min_text(12);
    // left: the made-up positions
    s.text_bold(24.0, 30.0, "made-up positions: one step for three pairs", 12, tok::INK, Anchor::Start);
    let (x0, x1, y0, y1) = (-3.2, 3.2, -2.2, 1.6);
    let (left, top, k) = (40.0, 48.0, 46.0);
    let px = |x: f64| left + (x - x0) * k;
    let py = |y: f64| top + (y1 - y) * k;
    // the two axes only: the positions are made up, so only their differences matter
    s.line(px(0.0), py(y0), px(0.0), py(y1), tok::AXIS, 1.0);
    s.line(px(x0), py(0.0), px(x1), py(0.0), tok::AXIS, 1.0);
    for pairs in ANALOGY_WORDS.chunks(2) {
        let (m, w) = (pairs[0].1, pairs[1].1);
        s.arrow(px(m[0]), py(m[1]) - 5.0, px(w[0]), py(w[1]) + 5.0, None);
    }
    // each word with its made-up position, so that every value below can be worked out by hand: the blue words under
    // their dots and the orange words over theirs, away from the arrows between them; a label that would cross the
    // vertical axis ends just left of it instead
    for (i, ((_, p), label)) in ANALOGY_WORDS.iter().zip(&labels).enumerate() {
        let colour = if i % 2 == 0 { tok::S1 } else { tok::S2 };
        s.dot(px(p[0]), py(p[1]), 5.0, colour, Some(tok::SURFACE));
        let y = if i % 2 == 0 { py(p[1]) + 21.0 } else { py(p[1]) - 10.0 };
        let half = text_w(label, 12) / 2.0;
        let crosses_axis = (px(p[0]) - half..=px(p[0]) + half).contains(&px(0.0));
        let (x, anchor) = if crosses_axis { (px(0.0) - 8.0, Anchor::End) } else { (px(p[0]), Anchor::Middle) };
        s.text(x, y, label, 12, colour, anchor);
    }
    s.text(24.0, 250.0, &format!("each grey arrow is the same step, {}", pair(&step)), 12, tok::INK, Anchor::Start);
    s.text(24.0, 268.0, &format!("king − man + woman = {}: queen's own place", pair(&target)), 12, tok::INK, Anchor::Start);
    s.text(24.0, 286.0, &format!("nearest by cosine: {}", others.join(", ")), 12, tok::INK, Anchor::Start);
    s.text(24.0, 304.0, &format!("(the question words {ma}, {mb} and {mc} are left out)"), 11, tok::MUTED, Anchor::Start);
    s.text(24.0, 328.0, "the positions are made up, set by hand to show the idea;", 11, tok::MUTED, Anchor::Start);
    s.text(24.0, 344.0, "vectors learned from text show such steps only roughly", 11, tok::MUTED, Anchor::Start);
    // right: the same question put to the tiny trained table
    let xr = 412.0;
    s.text_bold(xr, 30.0, "the tiny table of Figure 27.3", 12, tok::INK, Anchor::Start);
    s.text(xr, 52.0, &format!("{qa} is to {qb} as {qc} is to ?"), 12, tok::INK, Anchor::Start);
    s.text(xr, 70.0, &format!("nearest words to {qb} − {qa} + {qc}:"), 12, tok::INK2, Anchor::Start);
    let mut cells: Vec<Vec<Cell>> = vec![vec![cell("rank").fill(tok::NEUTRAL), cell("word").fill(tok::NEUTRAL), cell("cosine").fill(tok::NEUTRAL)]];
    for (i, (w, c)) in toy.iter().take(want_rank.max(5)).enumerate() {
        let hi = if *w == want { tok::FILL1 } else { tok::SURFACE };
        cells.push(vec![cell((i + 1).to_string()).fill(hi), cell(w.clone()).fill(hi), cell(num(*c, 3)).fill(hi)]);
    }
    s.cells_wh(xr, 82.0, 90.0, 26.0, &cells);
    let ty = 82.0 + 26.0 * cells.len() as f64 + 22.0;
    s.text(xr, ty, &format!("the right word, {want}, comes {rank_words};"), 12, tok::INK, Anchor::Start);
    s.text(xr, ty + 18.0, "five blurbs are far too little text", 12, tok::INK, Anchor::Start);
    s.text(xr, ty + 36.0, "(the three question words are left out)", 11, tok::MUTED, Anchor::Start);
    ("b27-5-analogy".to_string(), s.finish())
}

// =================================================================================================
// Chapter 28. GloVe: co-occurrence and probability ratios
// =================================================================================================

/// Symmetric co-occurrence counts within `window` words on either side: X[i][j] is how often word j sits near word i.
fn cooccurrence(texts: &[&'static str], window: usize) -> (Vec<&'static str>, Vec<Vec<f64>>) {
    let vocab = vocabulary(texts);
    let mut x = vec![vec![0.0; vocab.len()]; vocab.len()];
    for t in texts {
        let ids: Vec<usize> = tokens(t).iter().map(|w| vocab.iter().position(|v| v == w).unwrap_or(0)).collect();
        for i in 0..ids.len() {
            for j in i.saturating_sub(window)..(i + window + 1).min(ids.len()) {
                if j != i {
                    x[ids[i]][ids[j]] += 1.0;
                }
            }
        }
    }
    (vocab, x)
}

/// Table 1 of Pennington, Socher and Manning (2014), from a 6 billion token corpus: probe word, P(k | ice),
/// P(k | steam), and the printed ratio. Quoted data, not computed here: the paper's ratios come from unrounded
/// probabilities, so dividing the rounded ones does not reproduce them exactly (1.9 ÷ 0.22 is 8.6, printed 8.9).
const GLOVE_TABLE1: [(&str, f64, f64, f64); 4] =
    [("solid", 1.9e-4, 2.2e-5, 8.9), ("gas", 6.6e-5, 7.8e-4, 8.5e-2), ("water", 3.0e-3, 2.2e-3, 1.36), ("fashion", 1.7e-5, 1.8e-5, 0.96)];

/// The GloVe weighting function f(x) = (x / x_max)^α below x_max and 1 above, with the paper's x_max = 100, α = 3/4.
fn glove_weight(x: f64) -> f64 {
    const X_MAX: f64 = 100.0;
    const ALPHA: f64 = 0.75;
    if x < X_MAX { (x / X_MAX).powf(ALPHA) } else { 1.0 }
}

/// Formats a probability as "1.9 × 10⁻⁴".
fn sci(v: f64) -> String {
    let e = v.log10().floor() as i32;
    let m = v / 10f64.powi(e);
    let sup: String = format!("{e}").chars().map(|c| match c {
        '-' => '⁻',
        '0' => '⁰',
        '1' => '¹',
        '2' => '²',
        '3' => '³',
        '4' => '⁴',
        '5' => '⁵',
        '6' => '⁶',
        '7' => '⁷',
        '8' => '⁸',
        _ => '⁹',
    }).collect();
    if e == 0 { num(m, 2) } else { format!("{} × 10{sup}", num(m, 1)) }
}

fn fig_28_1_cooccurrence() -> Figure {
    let (vocab, x) = cooccurrence(&BLURBS[0..2], 1);
    let mut s = Svg::new(
        720,
        360,
        "Counting which words sit next to which in blurbs B1 and B2 gives a symmetric table; each row total is how many neighbours that word had",
        "An eight by eight heat map of co-occurrence counts for the words of blurbs B1 and B2 with a window of one word on each side, rows and columns in alphabetical order, and a column of row totals on the right.",
    ).min_text(12);
    let (cw, ch) = (60.0, 28.0);
    let x0 = 104.0;
    let y0 = 70.0;
    let max = x.iter().flatten().cloned().fold(0.0, f64::max);
    s.cells_wh(x0, y0, cw, ch, &heat_cells(&x, max, 0));
    row_labels(&mut s, x0 - 8.0, y0, ch, &vocab, 12);
    col_labels(&mut s, x0, y0 - 10.0, cw, &vocab, 11);
    let totals: Vec<Vec<Cell>> = x.iter().map(|r| vec![cell(numt(r.iter().sum(), 0)).fill(tok::NEUTRAL)]).collect();
    let xt = x0 + vocab.len() as f64 * cw + 14.0;
    s.cells_wh(xt, y0, 60.0, ch, &totals);
    s.text(xt + 30.0, y0 - 10.0, "row total", 11, tok::INK2, Anchor::Middle);
    s.text_bold(360.0, 30.0, "neighbours one word away in B1 and B2", 13, tok::INK, Anchor::Middle);
    let pairs: f64 = x.iter().flatten().sum::<f64>() / 2.0;
    s.text(
        360.0,
        y0 + vocab.len() as f64 * ch + 26.0,
        &format!("{} neighbouring pairs, each counted from both ends: the table is its own mirror image", numt(pairs, 0)),
        11,
        tok::INK2,
        Anchor::Middle,
    );
    ("b28-1-cooccurrence".to_string(), s.finish())
}

fn fig_28_2_ratios() -> Figure {
    let (vocab, x) = cooccurrence(&BLURBS[0..2], 1);
    let row = |w: &str| x[vocab.iter().position(|v| *v == w).unwrap_or(0)].clone();
    let (qi, sm) = (row("quiet"), row("small"));
    let (tq, ts): (f64, f64) = (qi.iter().sum(), sm.iter().sum());
    let mut s = Svg::new(
        720,
        340,
        "Dividing the neighbour probabilities of quiet by those of small gives 1 for their shared neighbour a, and pulls apart the words each has alone",
        "A table with one row per word k of B1 and B2: bars for the probability of k next to quiet and next to small, and the ratio of the two with a plain reading of it.",
    ).min_text(12);
    let heads = ["word k", "P(k | quiet)", "P(k | small)", "ratio", "what the ratio says"];
    let xs = [80.0, 100.0, 240.0, 400.0, 470.0];
    let y0 = 62.0;
    s.text_bold(xs[0], y0 - 14.0, heads[0], 12, tok::INK, Anchor::End);
    s.text_bold(xs[1] + 60.0, y0 - 14.0, heads[1], 12, tok::INK, Anchor::Middle);
    s.text_bold(xs[2] + 60.0, y0 - 14.0, heads[2], 12, tok::INK, Anchor::Middle);
    s.text_bold(xs[3], y0 - 14.0, heads[3], 12, tok::INK, Anchor::Middle);
    s.text_bold(xs[4], y0 - 14.0, heads[4], 12, tok::INK, Anchor::Start);
    for (k, w) in vocab.iter().enumerate() {
        let y = y0 + k as f64 * 30.0;
        if k % 2 == 0 {
            s.rect(12.0, y - 2.0, 696.0, 30.0, tok::NEUTRAL, None);
        }
        let (pq, ps) = (qi[k] / tq, sm[k] / ts);
        s.text(xs[0], y + 17.0, w, 12, tok::INK, Anchor::End);
        for (p, bx, colour) in [(pq, xs[1], tok::S3), (ps, xs[2], tok::S1)] {
            if p > 0.0 {
                s.rect(bx, y + 5.0, p * 180.0, 16.0, colour, None);
            }
            s.text(bx + p * 180.0 + 6.0, y + 17.0, &numt(p, 2), 11, if p > 0.0 { tok::INK } else { tok::MUTED }, Anchor::Start);
        }
        let (ratio, reading) = if pq > 0.0 && ps > 0.0 {
            (numt(pq / ps, 2), "a neighbour they share")
        } else if pq > 0.0 {
            ("∞".to_string(), "seen only beside quiet")
        } else if ps > 0.0 {
            (numt(0.0, 0), "seen only beside small")
        } else {
            ("0 ÷ 0".to_string(), "never beside either: no evidence")
        };
        s.text(xs[3], y + 17.0, &ratio, 12, tok::INK, Anchor::Middle);
        s.text(xs[4], y + 17.0, reading, 11, tok::INK2, Anchor::Start);
    }
    let yb = y0 + vocab.len() as f64 * 30.0 + 20.0;
    s.text(360.0, yb, &format!("quiet had {} neighbours in all and small had {}; each probability is a count divided by that total", numt(tq, 0), numt(ts, 0)), 11, tok::INK2, Anchor::Middle);
    ("b28-2-ratios".to_string(), s.finish())
}

fn fig_28_3_ice_steam() -> Figure {
    let mut s = Svg::new(
        720,
        330,
        "In the GloVe paper's corpus the ratio of probabilities is large for solid, small for gas and close to 1 for water and fashion",
        "For four probe words, the two probabilities from Table 1 of the GloVe paper are listed, and the printed ratio is drawn as a bar on a logarithmic scale from 0.01 to 100 with a line at 1.",
    ).min_text(12);
    let (lo, hi) = (0.01f64.log10(), 100f64.log10());
    let (bx0, bw) = (330.0, 360.0);
    let sx = |v: f64| bx0 + (v.log10() - lo) / (hi - lo) * bw;
    let y0 = 88.0;
    s.text_bold(84.0, y0 - 22.0, "probe word k", 12, tok::INK, Anchor::Middle);
    s.text_bold(172.0, y0 - 22.0, "P(k | ice)", 12, tok::INK, Anchor::Middle);
    s.text_bold(262.0, y0 - 22.0, "P(k | steam)", 12, tok::INK, Anchor::Middle);
    s.text_bold(bx0 + bw / 2.0, y0 - 38.0, "ratio P(k | ice) ÷ P(k | steam), printed in the paper", 12, tok::INK, Anchor::Middle);
    for t in [0.01, 0.1, 1.0, 10.0, 100.0] {
        let x = sx(t);
        s.line(x, y0 - 12.0, x, y0 + 4.0 * 48.0 - 8.0, if t == 1.0 { tok::AXIS } else { tok::GRID }, if t == 1.0 { 1.5 } else { 1.0 });
        s.text(x, y0 - 18.0, &numt(t, 2), 11, tok::MUTED, Anchor::Middle);
    }
    for (i, (k, pi, ps, ratio)) in GLOVE_TABLE1.iter().enumerate() {
        let y = y0 + i as f64 * 48.0;
        s.text(84.0, y + 16.0, k, 12, tok::INK, Anchor::Middle);
        s.text(172.0, y + 16.0, &sci(*pi), 12, tok::INK, Anchor::Middle);
        s.text(262.0, y + 16.0, &sci(*ps), 12, tok::INK, Anchor::Middle);
        let (x1, x2) = (sx(1.0).min(sx(*ratio)), sx(1.0).max(sx(*ratio)));
        s.rect(x1, y + 4.0, (x2 - x1).max(2.0), 18.0, if *ratio > 1.2 { tok::S1 } else if *ratio < 0.8 { tok::S2 } else { tok::MUTED }, None);
        let label = if *ratio < 0.1 { sci(*ratio) } else { numt(*ratio, 2) };
        if *ratio >= 1.0 {
            s.text(x2 + 6.0, y + 17.0, &label, 12, tok::INK, Anchor::Start);
        } else {
            s.text(x1 - 6.0, y + 17.0, &label, 12, tok::INK, Anchor::End);
        }
    }
    let yb = y0 + 4.0 * 48.0 + 16.0;
    s.text(360.0, yb, "solid goes with ice and gas with steam; water (both) and fashion (neither) sit near 1", 12, tok::INK2, Anchor::Middle);
    s.text(360.0, yb + 16.0, "values from Table 1 of Pennington, Socher and Manning (2014), a corpus of 6 billion tokens", 11, tok::MUTED, Anchor::Middle);
    ("b28-3-ice-steam".to_string(), s.finish())
}

fn fig_28_4_weighting() -> Figure {
    let mut s = Svg::new(
        720,
        338,
        "GloVe weighs each pair by f(x): rare pairs count for little, and every pair seen 100 times or more counts the same",
        "A curve of the weighting function f of the co-occurrence count x from 0 to 150, rising as x to the power three quarters over 100 and flat at 1 from x equal to 100, with four marked points.",
    ).min_text(12);
    let (px0, py0, pw, ph) = (80.0, 40.0, 560.0, 210.0);
    let xmax = 150.0;
    let sx = |x: f64| px0 + x / xmax * pw;
    let sy = |y: f64| py0 + ph - y * ph;
    s.line(px0, sy(0.0), px0 + pw, sy(0.0), tok::AXIS, 1.0);
    s.line(px0, py0, px0, py0 + ph, tok::AXIS, 1.0);
    for t in [0.0, 0.5, 1.0] {
        s.line(px0, sy(t), px0 + pw, sy(t), tok::GRID, 1.0);
        s.text(px0 - 8.0, sy(t) + 4.0, &numt(t, 1), 11, tok::MUTED, Anchor::End);
    }
    for t in [0.0, 25.0, 50.0, 75.0, 100.0, 125.0, 150.0] {
        s.text(sx(t), sy(0.0) + 18.0, &numt(t, 0), 11, tok::MUTED, Anchor::Middle);
    }
    s.text(px0 + pw / 2.0, sy(0.0) + 36.0, "x: how many times the two words were seen together", 11, tok::INK2, Anchor::Middle);
    s.text(px0 + 8.0, py0 - 12.0, "weight f(x)", 11, tok::INK2, Anchor::Start);
    let mut d = String::new();
    for k in 0..=300 {
        let x = k as f64 * xmax / 300.0;
        d.push_str(&format!("{}{} {}", if k == 0 { "M" } else { " L" }, xy(sx(x)), xy(sy(glove_weight(x)))));
    }
    s.path(&d, tok::S1, 2.5);
    for x in [10.0, 50.0, 100.0] {
        let f = glove_weight(x);
        s.dot(sx(x), sy(f), 4.5, tok::S2, Some(tok::SURFACE));
        s.text(sx(x) + 8.0, sy(f) + 16.0, &format!("f({}) = {}", numt(x, 0), num(f, 2)), 11, tok::INK, Anchor::Start);
    }
    let (_, table) = cooccurrence(&BLURBS[0..2], 1);
    let biggest = table.iter().flatten().cloned().fold(0.0, f64::max);
    s.text(360.0, 308.0, "x_max = 100 and power 3/4, the values used in the GloVe paper;", 11, tok::MUTED, Anchor::Middle);
    s.text(
        360.0,
        324.0,
        &format!("the biggest count in Figure 28.1, {}, would get f = {}", numt(biggest, 0), num(glove_weight(biggest), 2)),
        11,
        tok::MUTED,
        Anchor::Middle,
    );
    ("b28-4-weighting".to_string(), s.finish())
}

// =================================================================================================
// Chapter 29. Contextual embeddings and dimensionality reduction
// =================================================================================================

/// The eight books of chapters 9 and 10 (SPEC section B): pages, and training price = 12·(1 − e^(−pages/180)) + noise,
/// in hundreds of rupees.
fn books_pages_price() -> Vec<Vec<f64>> {
    let pages: [f64; 8] = [60.0, 90.0, 120.0, 160.0, 200.0, 260.0, 320.0, 400.0];
    let noise: [f64; 8] = [0.6, -0.5, 0.8, -0.9, 0.4, -0.7, 0.5, -0.3];
    pages.iter().zip(noise).map(|(x, e)| vec![*x, 12.0 * (1.0 - (-x / 180.0).exp()) + e]).collect()
}

/// Standardises each column: subtract the mean, divide by the (population) standard deviation.
fn standardise(rows: &[Vec<f64>]) -> Vec<Vec<f64>> {
    let n = rows.len() as f64;
    let d = rows[0].len();
    let mean: Vec<f64> = (0..d).map(|j| rows.iter().map(|r| r[j]).sum::<f64>() / n).collect();
    let sd: Vec<f64> = (0..d).map(|j| (rows.iter().map(|r| (r[j] - mean[j]).powi(2)).sum::<f64>() / n).sqrt()).collect();
    rows.iter().map(|r| (0..d).map(|j| (r[j] - mean[j]) / sd[j]).collect()).collect()
}

fn fig_29_1_static_contextual() -> Figure {
    let mut s = Svg::new(
        720,
        306,
        "A static embedding looks up book on its own and returns one vector for every sentence; a contextual model reads each sentence and returns a different vector for each use",
        "Top: the single word book goes into a lookup table and one row of cells comes out, the same for any sentence. Bottom: two sentences, one using book as a noun and one as a verb, go into a model that reads the whole sentence, and two differently shaded rows come out.",
    ).min_text(12);
    let pattern_a = [tok::FILL1, tok::FILL3, tok::FILL2, tok::SURFACE, tok::FILL3, tok::FILL1];
    let pattern_b = [tok::FILL3, tok::FILL1, tok::SURFACE, tok::FILL3, tok::FILL2, tok::FILL2];
    let pattern_c = [tok::FILL2, tok::SURFACE, tok::FILL3, tok::FILL1, tok::FILL1, tok::FILL3];
    let vec_row = |s: &mut Svg, y: f64, pat: &[&'static str; 6]| {
        let row: Vec<Cell> = pat.iter().map(|f| cell("").fill(f)).collect();
        s.cells_wh(476.0, y, 22.0, 20.0, &[row]);
    };
    // static
    s.text_bold(16.0, 30.0, "static (Word2Vec, GloVe)", 13, tok::INK, Anchor::Start);
    word_boxes(&mut s, 180.0, 52.0, &["book"], &[tok::FILL3]);
    s.arrow(236.0, 65.0, 290.0, 65.0, None);
    s.labelled_box2(292.0, 43.0, 150.0, 44.0, "lookup table", "sees only the word", tok::SURFACE, tok::AXIS);
    s.arrow(444.0, 65.0, 472.0, 65.0, None);
    vec_row(&mut s, 55.0, &pattern_a);
    s.text(614.0, 69.0, "every sentence", 11, tok::INK2, Anchor::Start);
    s.line(16.0, 112.0, 704.0, 112.0, tok::GRID, 1.0);
    // contextual
    s.text_bold(16.0, 138.0, "contextual (BERT)", 13, tok::INK, Anchor::Start);
    let sentences = [("the detective reads a book", "noun"), ("please book a table", "verb")];
    for (i, (t, kind)) in sentences.iter().enumerate() {
        let y = 156.0 + i as f64 * 64.0;
        let words = tokens(t);
        let fills: Vec<&'static str> = words.iter().map(|w| if *w == "book" { tok::FILL3 } else { tok::SURFACE }).collect();
        let boxes = word_boxes(&mut s, 16.0, y, &words, &fills);
        let (lx, lw) = boxes[boxes.len() - 1];
        s.text(16.0, y + 42.0, &format!("book as a {kind}"), 11, tok::INK2, Anchor::Start);
        s.arrow(lx + lw + 6.0, y + 13.0, 290.0, 201.0 + (i as f64 - 0.5) * 16.0, None);
        vec_row(&mut s, 168.0 + i as f64 * 34.0, if i == 0 { &pattern_b } else { &pattern_c });
        s.text(614.0, 182.0 + i as f64 * 34.0, &format!("sentence {}", i + 1), 11, tok::INK2, Anchor::Start);
    }
    s.labelled_box2(292.0, 179.0, 150.0, 44.0, "contextual model", "reads the whole sentence", tok::FILL1, tok::S1);
    s.arrow(444.0, 195.0, 472.0, 180.0, None);
    s.arrow(444.0, 207.0, 472.0, 212.0, None);
    s.text(360.0, 290.0, "shading stands for values: the rows are drawn to show the idea, not computed", 11, tok::MUTED, Anchor::Middle);
    ("b29-1-static-contextual".to_string(), s.finish())
}

fn fig_29_2_pca() -> Figure {
    let books = books_pages_price();
    let z = standardise(&books);
    let (proj, share, u) = pca2(&z);
    let mut s = Svg::new(
        720,
        410,
        "The eight books of chapter 9, standardised, lie close to one line; projecting each onto that line keeps most of the spread in one number per book",
        "Left: a scatter of the eight books with standardised pages across and standardised price up, the first principal direction drawn through the centre, and a grey drop from each book to the line. Right: the eight projected positions on a single number line.",
    ).min_text(12);
    let (cx, cy, half) = (190.0, 206.0, 150.0);
    let lim = 2.2;
    let sx = |v: f64| cx + v / lim * half;
    let sy = |v: f64| cy - v / lim * half;
    s.rect(cx - half, cy - half, 2.0 * half, 2.0 * half, tok::SURFACE, Some(tok::GRID));
    s.line(cx - half, cy, cx + half, cy, tok::GRID, 1.0);
    s.line(cx, cy - half, cx, cy + half, tok::GRID, 1.0);
    // the first principal direction u through the centre; standardised columns have mean 0, so the centre is the origin
    let r: f64 = z.iter().map(|p| p[0] * p[1]).sum::<f64>() / z.len() as f64;
    s.line(sx(-lim * u[0]), sy(-lim * u[1]), sx(lim * u[0]), sy(lim * u[1]), tok::S2, 2.0);
    for (k, p) in z.iter().enumerate() {
        let t = proj[k][0];
        let (fx, fy) = (t * u[0], t * u[1]);
        s.path(&format!("M{} {} L{} {}", xy(sx(p[0])), xy(sy(p[1])), xy(sx(fx)), xy(sy(fy))), tok::MUTED, 1.0);
        s.dot(sx(fx), sy(fy), 3.0, tok::S2, None);
        s.dot(sx(p[0]), sy(p[1]), 5.0, tok::S1, Some(tok::SURFACE));
    }
    s.text(cx, cy + half + 18.0, "pages, standardised", 11, tok::INK2, Anchor::Middle);
    s.text(cx - half, cy - half - 8.0, "price, standardised", 11, tok::INK2, Anchor::Start);
    // number line of scores
    let (lx0, lx1, ly) = (420.0, 700.0, 150.0);
    let smax = proj.iter().map(|p| p[0].abs()).fold(0.0, f64::max) * 1.1;
    let lx = |v: f64| (lx0 + lx1) / 2.0 + v / smax * (lx1 - lx0) / 2.0;
    s.line(lx0, ly, lx1, ly, tok::AXIS, 1.0);
    for (k, p) in proj.iter().enumerate() {
        s.dot(lx(p[0]), ly, 5.0, tok::S2, Some(tok::SURFACE));
        s.text(lx(p[0]), if k % 2 == 0 { ly - 12.0 } else { ly + 22.0 }, &numt(books[k][0], 0), 11, tok::INK, Anchor::Middle);
    }
    s.text_bold((lx0 + lx1) / 2.0, 80.0, "one number per book", 13, tok::INK, Anchor::Middle);
    s.text((lx0 + lx1) / 2.0, 100.0, "each book's place on the line, by pages", 11, tok::INK2, Anchor::Middle);
    let notes = [
        format!("the line keeps {}% of the total spread;", numt(share[0] * 100.0, 1)),
        format!("the {}% left over is the grey drops", numt(share[1] * 100.0, 1)),
        format!("correlation of pages and price: {}", num(r, 3)),
        "for two standardised columns the line".to_string(),
        "always runs at 45 degrees, and its share".to_string(),
        "of the spread is (1 + r) ÷ 2".to_string(),
    ];
    for (i, n) in notes.iter().enumerate() {
        s.text(404.0, 230.0 + i as f64 * 18.0, n, 11, if i < 3 { tok::INK } else { tok::INK2 }, Anchor::Start);
    }
    ("b29-2-pca".to_string(), s.finish())
}

fn fig_29_3_autoencoder() -> Figure {
    let vocab = vocabulary(&BLURBS);
    let counts = count_matrix(&BLURBS, &vocab);
    let mut s = Svg::new(
        720,
        392,
        "An autoencoder squeezes a long input through a narrow code and is trained to rebuild the input from the code alone",
        "An hourglass drawn left to right: a column of eighteen cells holding the word counts of blurb B2, an encoder narrowing to a code of two empty cells, a decoder widening to eighteen empty reconstruction cells.",
    ).min_text(12);
    let (ch, y0) = (16.0, 48.0);
    let n = vocab.len() as f64;
    let col: Vec<Vec<Cell>> = counts[1].iter().map(|v| vec![if *v > 0.0 { cell(numt(*v, 0)).fill(heat(*v, 2.0)) } else { cell("") }]).collect();
    s.cells_wh(84.0, y0, 26.0, ch, &col);
    row_labels(&mut s, 78.0, y0 - 1.0, ch, &vocab, 12);
    s.text(97.0, y0 - 10.0, "input: B2", 12, tok::INK2, Anchor::Middle);
    let mid = y0 + n * ch / 2.0;
    s.path(&format!("M120 {} L304 {} L304 {} L120 {} Z", xy(y0), xy(mid - 18.0), xy(mid + 18.0), xy(y0 + n * ch)), tok::S1, 1.5);
    s.text(212.0, mid + 4.0, "encoder", 12, tok::INK, Anchor::Middle);
    let code: Vec<Vec<Cell>> = vec![vec![cell("").fill(tok::FILL3)], vec![cell("").fill(tok::FILL3)]];
    s.cells_wh(316.0, mid - 14.0, 26.0, 14.0, &code);
    s.text(329.0, mid - 24.0, "code", 12, tok::INK, Anchor::Middle);
    s.text(329.0, mid + 32.0, "2 numbers", 12, tok::INK2, Anchor::Middle);
    s.path(&format!("M354 {} L540 {} L540 {} L354 {} Z", xy(mid - 18.0), xy(y0), xy(y0 + n * ch), xy(mid + 18.0)), tok::S2, 1.5);
    s.text(447.0, mid + 4.0, "decoder", 12, tok::INK, Anchor::Middle);
    let out: Vec<Vec<Cell>> = (0..vocab.len()).map(|_| vec![cell("")]).collect();
    s.cells_wh(552.0, y0, 26.0, ch, &out);
    s.text(565.0, y0 - 10.0, "rebuilt input", 12, tok::INK2, Anchor::Middle);
    s.text(594.0, mid - 8.0, "training makes", 12, tok::INK2, Anchor::Start);
    s.text(594.0, mid + 8.0, "right match left", 12, tok::INK2, Anchor::Start);
    s.text(360.0, 358.0, "with a straight-line decoder and squared error it finds the same subspace as PCA;", 12, tok::INK2, Anchor::Middle);
    s.text(360.0, 376.0, "bent, non-linear layers can follow curves that PCA misses", 12, tok::INK2, Anchor::Middle);
    ("b29-3-autoencoder".to_string(), s.finish())
}

// =================================================================================================
// Chapter 30. From the bottleneck to attention
// =================================================================================================

/// Blurb B1 and a French rendering of it. Each French word's English partner (by position in B1) gives the toy
/// alignment scores of `alignment_weights`.
const FRENCH: [&str; 7] = ["un", "mystère", "tranquille", "dans", "une", "petite", "ville"];
const FRENCH_PARTNER: [usize; 7] = [0, 2, 1, 3, 4, 5, 6];

/// Toy alignment: score = 4 when the French word translates the English word, minus 0.5 for every step of distance
/// between their positions; softmax over each French word's row turns scores into weights.
fn alignment_weights() -> Vec<Vec<f64>> {
    let english = tokens(BLURBS[0]);
    (0..FRENCH.len())
        .map(|i| {
            let partner = english[FRENCH_PARTNER[i]];
            let scores: Vec<f64> = (0..english.len())
                .map(|j| (if english[j] == partner { 4.0 } else { 0.0 }) - 0.5 * (i as f64 - j as f64).abs())
                .collect();
            softmax(&scores)
        })
        .collect()
}

fn state_cell(s: &mut Svg, x: f64, y: f64, fill: &str, stroke: &str) {
    s.rect(x, y, 44.0, 30.0, fill, Some(stroke));
}

fn fig_30_1_bottleneck() -> Figure {
    let mut s = Svg::new(
        720,
        300,
        "An encoder reads the English blurb one word at a time and hands the decoder a single context vector; everything the decoder knows must pass through it",
        "Top row: seven encoder cells linked left to right, each fed one English word of blurb B1. The last cell feeds one outlined context vector box. Bottom row: seven decoder cells linked left to right, starting from the context vector and producing the French words.",
    ).min_text(12);
    let english = tokens(BLURBS[0]);
    let step = 62.0;
    for (i, w) in english.iter().enumerate() {
        let x = 30.0 + i as f64 * step;
        state_cell(&mut s, x, 56.0, tok::FILL1, tok::S1);
        s.text(x + 22.0, 108.0, w, 11, tok::INK, Anchor::Middle);
        if i + 1 < english.len() {
            s.arrow(x + 46.0, 71.0, x + step - 2.0, 71.0, None);
        }
    }
    s.text(30.0, 40.0, "encoder: reads the English, one word per step", 12, tok::INK, Anchor::Start);
    let xl = 30.0 + 6.0 * step + 44.0;
    s.arrow(xl + 2.0, 71.0, 512.0, 71.0, None);
    s.rect_bold(514.0, 50.0, 116.0, 44.0, tok::FILL3, tok::S2);
    s.text(572.0, 69.0, "context vector", 12, tok::INK, Anchor::Middle);
    s.text(572.0, 85.0, "one fixed size", 11, tok::INK2, Anchor::Middle);
    s.text(580.0, 116.0, "the bottleneck", 11, tok::S2, Anchor::Start);
    s.path("M572 94 L572 150 L52 150 L52 170", tok::INK2, 1.5);
    s.arrow(52.0, 168.0, 52.0, 184.0, None);
    for (i, w) in FRENCH.iter().enumerate() {
        let x = 30.0 + i as f64 * step;
        state_cell(&mut s, x, 186.0, tok::NEUTRAL, tok::S2);
        s.text(x + 22.0, 238.0, w, 11, tok::INK, Anchor::Middle);
        if i + 1 < FRENCH.len() {
            s.arrow(x + 46.0, 201.0, x + step - 2.0, 201.0, None);
        }
    }
    s.text(480.0, 206.0, "decoder: writes the French,", 12, tok::INK, Anchor::Start);
    s.text(480.0, 222.0, "starting from the context", 12, tok::INK, Anchor::Start);
    s.text(360.0, 280.0, "a long blurb and a short one must fit through the same fixed-size vector", 11, tok::INK2, Anchor::Middle);
    ("b30-1-bottleneck".to_string(), s.finish())
}

fn fig_30_2_route() -> Figure {
    let w = alignment_weights();
    let target = 2usize; // tranquille
    let mut s = Svg::new(
        720,
        340,
        "With attention, the decoder step that writes tranquille takes a weighted sum of all seven encoder states, and nearly all the weight goes to quiet",
        "Seven encoder cells in a row with the English words of B1 beneath; a line from each cell to a summing node, drawn thicker for a larger weight and labelled with the weight; the sum feeds the decoder cell that writes tranquille.",
    ).min_text(12);
    let english = tokens(BLURBS[0]);
    let step = 92.0;
    let (sx, sy) = (360.0, 214.0);
    for (j, word) in english.iter().enumerate() {
        let x = 38.0 + j as f64 * step;
        state_cell(&mut s, x, 50.0, tok::FILL1, tok::S1);
        s.text(x + 22.0, 42.0, word, 11, tok::INK, Anchor::Middle);
        let wt = w[target][j];
        s.line(x + 22.0, 80.0, sx, sy - 12.0, if wt > 0.5 { tok::S2 } else { tok::AXIS }, 0.6 + 9.0 * wt);
        let (mx, my) = (x + 22.0 + (sx - x - 22.0) * 0.28, 80.0 + (sy - 92.0) * 0.28);
        let off = if wt > 0.5 { 16.0 } else { 6.0 };
        s.text(mx + if j < 3 { -off } else { off }, my + 4.0, &num(wt, 3), 11, if wt > 0.5 { tok::INK } else { tok::INK2 }, if j < 3 { Anchor::End } else { Anchor::Start });
    }
    s.dot(sx, sy, 12.0, tok::SURFACE, Some(tok::INK2));
    s.text(sx, sy + 5.0, "Σ", 13, tok::INK, Anchor::Middle);
    s.text(sx + 20.0, sy + 4.0, "weighted sum of the seven states", 11, tok::INK2, Anchor::Start);
    s.arrow(sx, sy + 13.0, sx, 250.0, None);
    s.rect_bold(sx - 22.0, 252.0, 44.0, 30.0, tok::NEUTRAL, tok::S2);
    s.text(sx, 300.0, FRENCH[target], 12, tok::INK, Anchor::Middle);
    s.text(sx - 40.0, 271.0, "decoder step 3", 11, tok::INK2, Anchor::End);
    let total: f64 = w[target].iter().sum();
    s.text(360.0, 328.0, &format!("line width follows the weight; the seven weights add up to {}", numt(total, 0)), 11, tok::MUTED, Anchor::Middle);
    ("b30-2-route".to_string(), s.finish())
}

fn fig_30_3_alignment() -> Figure {
    let w = alignment_weights();
    let english = tokens(BLURBS[0]);
    let mut s = Svg::new(
        720,
        380,
        "The weights for every French word form an alignment map: mostly a diagonal, with a swap where French puts tranquille after its noun",
        "A seven by seven heat map: rows are the French words, columns the English words of B1, each cell the attention weight; the bright cells run down the diagonal except for mystère and tranquille, which cross.",
    ).min_text(12);
    let (cw, ch) = (62.0, 34.0);
    let (x0, y0) = (150.0, 56.0);
    s.cells_wh(x0, y0, cw, ch, &heat_cells(&w, 1.0, 2));
    row_labels(&mut s, x0 - 10.0, y0, ch, &FRENCH, 12);
    col_labels(&mut s, x0, y0 - 10.0, cw, &english, 12);
    s.rect_bold(x0 + cw, y0 + ch, 2.0 * cw, 2.0 * ch, "none", tok::S2);
    s.text(x0 + 3.5 * cw, y0 + 7.0 * ch + 26.0, "each row is one decoder step and adds up to 1", 11, tok::INK2, Anchor::Middle);
    s.text(x0 + 3.5 * cw, y0 + 7.0 * ch + 42.0, "made-up scores: 4 for a word and its translation, minus 0.5 per step of distance; softmax per row", 11, tok::MUTED, Anchor::Middle);
    ("b30-3-alignment".to_string(), s.finish())
}

// =================================================================================================
// Chapter 31. Self-attention in plain words
// =================================================================================================

/// The three attention words of SPEC section B, in the order they appear in blurb B1.
const ATT_WORDS: [&str; 3] = ["quiet", "mystery", "town"];
const ATT_X: [[f64; 2]; 3] = [[2.0, 0.0], [1.0, 1.0], [0.0, 2.0]];

fn mat_mul(a: &[Vec<f64>], b: &[Vec<f64>]) -> Vec<Vec<f64>> {
    a.iter().map(|r| (0..b[0].len()).map(|j| r.iter().zip(b).map(|(x, row)| x * row[j]).sum()).collect()).collect()
}

fn transpose(a: &[Vec<f64>]) -> Vec<Vec<f64>> {
    (0..a[0].len()).map(|j| a.iter().map(|r| r[j]).collect()).collect()
}

/// Self-attention on rows of `x` with projections (identity when `None`); `scale` divides the scores by √d.
/// Returns (scores, weights, outputs).
fn self_attention(x: &[Vec<f64>], wq: &[Vec<f64>], wk: &[Vec<f64>], wv: &[Vec<f64>], scale: bool) -> (Vec<Vec<f64>>, Vec<Vec<f64>>, Vec<Vec<f64>>) {
    let q = mat_mul(x, wq);
    let k = mat_mul(x, wk);
    let v = mat_mul(x, wv);
    let d = q[0].len() as f64;
    let scores: Vec<Vec<f64>> = mat_mul(&q, &transpose(&k)).iter().map(|r| r.iter().map(|s| if scale { s / d.sqrt() } else { *s }).collect()).collect();
    let weights: Vec<Vec<f64>> = scores.iter().map(|r| softmax(r)).collect();
    let out = mat_mul(&weights, &v);
    (scores, weights, out)
}

fn identity2() -> Vec<Vec<f64>> {
    vec![vec![1.0, 0.0], vec![0.0, 1.0]]
}

fn swap2() -> Vec<Vec<f64>> {
    vec![vec![0.0, 1.0], vec![1.0, 0.0]]
}

fn att_x() -> Vec<Vec<f64>> {
    ATT_X.iter().map(|r| r.to_vec()).collect()
}

fn plain_cells(m: &[Vec<f64>], decimals: usize) -> Vec<Vec<Cell>> {
    m.iter().map(|r| r.iter().map(|v| cell(if decimals == 0 { numt(*v, 0) } else { num(*v, decimals) })).collect()).collect()
}

fn fig_31_1_three_steps() -> Figure {
    let x = att_x();
    let (scores, weights, out) = self_attention(&x, &identity2(), &identity2(), &identity2(), false);
    // reorder the words: the outputs should come back reordered in the same way
    let order = [2usize, 0, 1];
    let xr: Vec<Vec<f64>> = order.iter().map(|i| x[*i].clone()).collect();
    let (_, _, out_r) = self_attention(&xr, &identity2(), &identity2(), &identity2(), false);
    let same = order.iter().enumerate().all(|(k, i)| out_r[k].iter().zip(&out[*i]).all(|(a, b)| (a - b).abs() < 1e-12));
    let mut s = Svg::new(
        720,
        360,
        "Simple self-attention in three steps: dot products score every pair of words, softmax turns each row into weights, and each output is the weighted mix of the word vectors",
        "Left: the three word vectors quiet, mystery and town drawn as arrows, with the three outputs as hollow dots. Middle: the three by three table of dot-product scores and the three by three table of softmax weights. Right: the output vectors.",
    ).min_text(12);
    // plot
    let (ox, oy, u) = (36.0, 250.0, 72.0);
    for t in 0..=2 {
        let f = t as f64;
        s.line(ox + f * u, oy - 2.4 * u, ox + f * u, oy, tok::GRID, 1.0);
        s.line(ox, oy - f * u, ox + 2.4 * u, oy - f * u, tok::GRID, 1.0);
        s.text(ox + f * u, oy + 16.0, &numt(f, 0), 11, tok::MUTED, Anchor::Middle);
        if t > 0 {
            s.text(ox - 6.0, oy - f * u + 4.0, &numt(f, 0), 11, tok::MUTED, Anchor::End);
        }
    }
    let colours = [tok::S1, tok::S3, tok::S2];
    for (k, v) in x.iter().enumerate() {
        s.line(ox, oy, ox + v[0] * u, oy - v[1] * u, colours[k], 2.5);
        s.dot(ox + v[0] * u, oy - v[1] * u, 4.0, colours[k], None);
        let (lx, ly, anchor) = match k {
            0 => (ox + v[0] * u + 6.0, oy - v[1] * u - 8.0, Anchor::Start),
            1 => (ox + v[0] * u + 8.0, oy - v[1] * u - 4.0, Anchor::Start),
            _ => (ox + v[0] * u + 8.0, oy - v[1] * u + 4.0, Anchor::Start),
        };
        s.text(lx, ly, ATT_WORDS[k], 12, tok::INK, anchor);
    }
    for (k, y) in out.iter().enumerate() {
        s.dot(ox + y[0] * u, oy - y[1] * u, 5.0, tok::SURFACE, Some(colours[k]));
    }
    s.text(ox, 40.0, "word vectors (lines) and", 11, tok::INK2, Anchor::Start);
    s.text(ox, 54.0, "their outputs (hollow dots)", 11, tok::INK2, Anchor::Start);
    // scores and weights
    let (gx, gy, cw, ch) = (292.0, 96.0, 30.0, 30.0);
    s.text_bold(gx + 1.5 * cw, 40.0, "1. score", 12, tok::INK, Anchor::Middle);
    s.text(gx + 1.5 * cw, 56.0, "dot product of each pair", 11, tok::INK2, Anchor::Middle);
    s.cells(gx, gy, cw, &plain_cells(&scores, 0));
    row_labels(&mut s, gx - 6.0, gy, ch, &ATT_WORDS, 11);
    col_labels(&mut s, gx, gy - 8.0, cw, &["q", "m", "t"], 11);
    let wx = gx + 3.0 * cw + 28.0;
    let wcw = 50.0;
    s.text_bold(wx + 1.5 * wcw, 40.0, "2. softmax each row", 12, tok::INK, Anchor::Middle);
    s.text(wx + 1.5 * wcw, 56.0, "weights that add up to 1", 11, tok::INK2, Anchor::Middle);
    s.cells_wh(wx, gy, wcw, ch, &heat_cells(&weights, 1.0, 3));
    col_labels(&mut s, wx, gy - 8.0, wcw, &["q", "m", "t"], 11);
    s.arrow(gx + 3.0 * cw + 4.0, gy + 1.5 * ch, wx - 4.0, gy + 1.5 * ch, None);
    // outputs
    let yx = wx + 3.0 * wcw + 22.0;
    s.text_bold(yx + 48.0, 40.0, "3. mix", 12, tok::INK, Anchor::Middle);
    s.text(yx + 48.0, 56.0, "blend the vectors", 11, tok::INK2, Anchor::Middle);
    s.cells_wh(yx, gy, 48.0, ch, &plain_cells(&out, 3));
    col_labels(&mut s, yx, gy - 8.0, 48.0, &["first", "second"], 11);
    s.arrow(wx + 3.0 * wcw + 4.0, gy + 1.5 * ch, yx - 4.0, gy + 1.5 * ch, None);
    s.text(gx - 6.0, gy + 3.0 * ch + 18.0, "q, m, t: quiet, mystery, town", 11, tok::MUTED, Anchor::Start);
    // worked row
    let terms: Vec<String> = (0..3).map(|j| format!("{} × ({}, {})", num(weights[0][j], 3), numt(x[j][0], 0), numt(x[j][1], 0))).collect();
    s.text(286.0, 228.0, "output for quiet:", 12, tok::INK, Anchor::Start);
    s.text(286.0, 246.0, &terms.join(" + "), 12, tok::INK2, Anchor::Start);
    s.text(286.0, 264.0, &format!("= ({}, {})", num(out[0][0], 3), num(out[0][1], 3)), 12, tok::INK2, Anchor::Start);
    s.text(286.0, 292.0, &format!("mystery scores {} with itself and {} with quiet: no bigger", numt(scores[1][1], 0), numt(scores[1][0], 0)), 12, tok::INK2, Anchor::Start);
    s.text(
        286.0,
        310.0,
        if same { "reorder the words: the same outputs return, reordered" } else { "reordering changed the outputs" },
        12,
        tok::INK2,
        Anchor::Start,
    );
    s.text(360.0, 344.0, "no weights are learned in this version: the word vectors alone decide who attends to whom", 11, tok::MUTED, Anchor::Middle);
    ("b31-1-three-steps".to_string(), s.finish())
}

fn fig_31_2_qkv() -> Figure {
    let x = att_x();
    let (wq, wk, wv) = (identity2(), swap2(), identity2());
    let q = mat_mul(&x, &wq);
    let k = mat_mul(&x, &wk);
    let v = mat_mul(&x, &wv);
    let (scores, weights, out) = self_attention(&x, &wq, &wk, &wv, true);
    let mut s = Svg::new(
        720,
        420,
        "Queries, keys and values are three learned re-descriptions of each word; here the keys swap the two numbers, and quiet now attends mostly to town instead of itself",
        "The three word vectors on the left feed three projections: queries unchanged, keys with their two numbers swapped, values unchanged. Queries meet keys in a table of scaled scores, softmax gives the weights, and the weights mix the values into the outputs.",
    ).min_text(12);
    let (cw, ch) = (36.0, 22.0);
    let y0 = 176.0;
    s.cells_wh(20.0, y0, cw, ch, &plain_cells(&x, 0));
    s.text(56.0, y0 - 10.0, "word vectors", 11, tok::INK2, Anchor::Middle);
    for (i, w) in ATT_WORDS.iter().enumerate() {
        s.text(20.0 + 2.0 * cw + 6.0, y0 + i as f64 * ch + 15.0, w, 11, tok::INK, Anchor::Start);
    }
    let blocks = [("Q = queries", "what each word looks for", &q, 40.0), ("K = keys: numbers swapped", "what each word offers", &k, 156.0), ("V = values", "what each word passes on", &v, 272.0)];
    for (name, note, m, y) in blocks.iter() {
        s.arrow(150.0, y0 + 1.5 * ch, 206.0, y + 55.0, None);
        s.cells_wh(210.0, *y + 22.0, cw, ch, &plain_cells(m, 0));
        s.text(210.0, *y, name, 12, tok::INK, Anchor::Start);
        s.text(210.0, *y + 14.0, note, 11, tok::INK2, Anchor::Start);
    }
    let gx = 440.0;
    let gw = 44.0;
    let gh = 26.0;
    s.text_bold(gx + 1.5 * gw, 40.0, "scores ÷ √2", 12, tok::INK, Anchor::Middle);
    s.text(gx + 1.5 * gw, 56.0, "each query against each key", 11, tok::INK2, Anchor::Middle);
    s.cells_wh(gx, 84.0, gw, gh, &plain_cells(&scores, 2));
    col_labels(&mut s, gx, 76.0, gw, &ATT_WORDS, 11);
    row_labels(&mut s, gx - 6.0, 84.0, gh, &ATT_WORDS, 11);
    s.arrow(gx + 1.5 * gw, 84.0 + 3.0 * gh + 6.0, gx + 1.5 * gw, 208.0, None);
    s.text(gx + 1.5 * gw + 8.0, 190.0, "softmax each row", 11, tok::INK2, Anchor::Start);
    s.text_bold(gx + 1.5 * gw, 226.0, "weights", 12, tok::INK, Anchor::Middle);
    s.cells_wh(gx, 250.0, gw, gh, &heat_cells(&weights, 1.0, 3));
    col_labels(&mut s, gx, 244.0, gw, &ATT_WORDS, 11);
    row_labels(&mut s, gx - 6.0, 250.0, gh, &ATT_WORDS, 11);
    let ox = gx + 3.0 * gw + 30.0;
    s.arrow(gx + 3.0 * gw + 4.0, 250.0 + 1.5 * gh, ox - 4.0, 250.0 + 1.5 * gh, None);
    s.text(gx + 3.0 * gw + 17.0, 250.0 + 1.5 * gh - 6.0, "× V", 11, tok::INK2, Anchor::Middle);
    s.cells_wh(ox, 250.0, 50.0, gh, &plain_cells(&out, 3));
    s.text_bold(ox + 50.0, 226.0, "outputs", 12, tok::INK, Anchor::Middle);
    s.text(
        360.0,
        392.0,
        &format!("quiet's weights: {} on itself, {} on mystery, {} on town", num(weights[0][0], 3), num(weights[0][1], 3), num(weights[0][2], 3)),
        11,
        tok::INK2,
        Anchor::Middle,
    );
    s.text(360.0, 408.0, "real layers learn the three matrices; here they are chosen by hand to show the effect", 11, tok::MUTED, Anchor::Middle);
    ("b31-2-qkv".to_string(), s.finish())
}

fn fig_31_3_heads() -> Figure {
    let x = att_x();
    let (_, w1, o1) = self_attention(&x, &identity2(), &identity2(), &identity2(), true);
    let (_, w2, o2) = self_attention(&x, &identity2(), &swap2(), &identity2(), true);
    let joined: Vec<Vec<f64>> = o1.iter().zip(&o2).map(|(a, b)| a.iter().chain(b.iter()).cloned().collect()).collect();
    let mut s = Svg::new(
        720,
        352,
        "Two heads look at the same three words in two different ways at the same time, and their outputs are placed side by side",
        "The three word vectors on the left feed two heads drawn one above the other. Head 1 uses keys as they are, so quiet and town attend mostly to themselves and mystery spreads its attention evenly; head 2 swaps the keys, so quiet and town attend to each other and mystery again spreads evenly. Each head's two-number outputs are joined into four numbers per word.",
    ).min_text(12);
    let (cw, ch) = (44.0, 26.0);
    let yx = 132.0;
    s.cells_wh(20.0, yx, 34.0, ch, &plain_cells(&x, 0));
    for (i, w) in ATT_WORDS.iter().enumerate() {
        s.text(94.0, yx + i as f64 * ch + 17.0, w, 11, tok::INK, Anchor::Start);
    }
    s.text(54.0, yx - 10.0, "word vectors", 11, tok::INK2, Anchor::Middle);
    for (h, (w, o, title, y)) in [(&w1, &o1, "head 1: keys as they are", 44.0), (&w2, &o2, "head 2: keys swapped", 208.0)].iter().enumerate() {
        let wx = 214.0;
        s.text_bold(wx, y - 12.0, title, 12, tok::INK, Anchor::Start);
        s.cells_wh(wx, *y + 10.0, cw, ch, &heat_cells(w, 1.0, 2));
        col_labels(&mut s, wx, *y + 4.0, cw, &["q", "m", "t"], 11);
        s.arrow(150.0, yx + 1.5 * ch, wx - 6.0, y + 10.0 + 1.5 * ch, None);
        s.arrow(wx + 3.0 * cw + 6.0, y + 10.0 + 1.5 * ch, 372.0, y + 10.0 + 1.5 * ch, Some("× V"));
        s.cells_wh(376.0, *y + 10.0, cw, ch, &plain_cells(o, 2));
        s.arrow(376.0 + 2.0 * cw + 6.0, y + 10.0 + 1.5 * ch, 520.0, yx + 1.5 * ch + if h == 0 { -8.0 } else { 8.0 }, None);
        let _ = h;
    }
    s.text_bold(610.0, yx - 26.0, "joined", 12, tok::INK, Anchor::Middle);
    s.text(610.0, yx - 10.0, "4 numbers per word", 11, tok::INK2, Anchor::Middle);
    let jc: Vec<Vec<Cell>> = joined.iter().map(|r| r.iter().enumerate().map(|(j, v)| { let c0 = cell(num(*v, 2)); if j < 2 { c0.fill(tok::FILL1) } else { c0.fill(tok::NEUTRAL) } }).collect()).collect();
    s.cells_wh(526.0, yx, 42.0, ch, &jc);
    s.text(610.0, yx + 3.0 * ch + 18.0, "blue: head 1, grey: head 2", 11, tok::INK2, Anchor::Middle);
    s.text(360.0, 320.0, "q, m, t: quiet, mystery, town; both heads divide their scores by √2;", 11, tok::MUTED, Anchor::Middle);
    s.text(360.0, 336.0, "a real layer then mixes the joined numbers with one more matrix", 11, tok::MUTED, Anchor::Middle);
    ("b31-3-heads".to_string(), s.finish())
}

/// The sinusoidal positional code of Vaswani et al. (2017), PE(pos, 2i) = sin(pos / 10000^(2i/d)),
/// PE(pos, 2i+1) = cos(pos / 10000^(2i/d)).
fn positional(pos: usize, d: usize) -> Vec<f64> {
    (0..d)
        .map(|dim| {
            let i = dim / 2;
            let a = pos as f64 / 10000f64.powf(2.0 * i as f64 / d as f64);
            if dim % 2 == 0 { a.sin() } else { a.cos() }
        })
        .collect()
}

fn fig_31_4_positions() -> Figure {
    let words = tokens(BLURBS[0]);
    let d = 4usize;
    let pe: Vec<Vec<f64>> = (0..words.len()).map(|p| positional(p, d)).collect();
    let mut s = Svg::new(
        720,
        360,
        "Each position gets its own pattern of sines and cosines, so the two copies of a in blurb B1 receive different position codes",
        "Four small line charts, one per number of a four-number position code, plotting its value at positions 0 to 6 under the words of B1; below, the codes of the two copies of a at positions 0 and 4.",
    ).min_text(12);
    let names = ["sin(pos)", "cos(pos)", "sin(pos ÷ 100)", "cos(pos ÷ 100)"];
    let (pw, ph) = (150.0, 120.0);
    for dim in 0..d {
        let px = 28.0 + dim as f64 * (pw + 20.0);
        let py = 50.0;
        let sx = |p: usize| px + 10.0 + p as f64 * (pw - 20.0) / (words.len() - 1) as f64;
        let sy = |v: f64| py + ph / 2.0 - v * (ph / 2.0 - 8.0);
        s.rect(px, py, pw, ph, tok::SURFACE, Some(tok::GRID));
        s.line(px, sy(0.0), px + pw, sy(0.0), tok::GRID, 1.0);
        s.text(px + pw / 2.0, py - 10.0, names[dim], 12, tok::INK, Anchor::Middle);
        let mut path = String::new();
        for p in 0..words.len() {
            path.push_str(&format!("{}{} {}", if p == 0 { "M" } else { " L" }, xy(sx(p)), xy(sy(pe[p][dim]))));
        }
        let colour = if dim < 2 { tok::S1 } else { tok::S2 };
        s.path(&path, colour, 2.0);
        for p in 0..words.len() {
            s.dot(sx(p), sy(pe[p][dim]), 3.5, colour, Some(tok::SURFACE));
        }
        if dim == 0 {
            s.text(px - 4.0, sy(1.0) + 4.0, "1", 11, tok::MUTED, Anchor::End);
            s.text(px - 4.0, sy(-1.0) + 4.0, "−1", 11, tok::MUTED, Anchor::End);
        }
        for (p, w) in words.iter().enumerate() {
            if dim == 0 || p % 2 == 0 {
                s.text(sx(p), py + ph + 14.0, &p.to_string(), 11, tok::MUTED, Anchor::Middle);
            }
            if dim == 0 && (p == 0 || p == 4) {
                s.text(sx(p), py + ph + 28.0, w, 11, tok::INK, Anchor::Middle);
            }
        }
    }
    s.text(28.0, 232.0, "position number under each chart; the word a sits at positions 0 and 4 of the blurb", 11, tok::INK2, Anchor::Start);
    let rows: Vec<Vec<Cell>> = [0usize, 4].iter().map(|p| pe[*p].iter().map(|v| cell(num(*v, 3)).fill(tok::FILL1)).collect()).collect();
    s.cells_wh(250.0, 256.0, 70.0, 28.0, &rows);
    s.text(240.0, 274.0, "a at position 0", 12, tok::INK, Anchor::End);
    s.text(240.0, 302.0, "a at position 4", 12, tok::INK, Anchor::End);
    s.text(360.0, 340.0, "added to each word's embedding, the code makes identical words at different places differ", 12, tok::INK2, Anchor::Middle);
    ("b31-4-positions".to_string(), s.finish())
}

// =================================================================================================
// Chapter 32. Transformers and pretrained models
// =================================================================================================

/// Model sizes quoted from the papers (not computed): Vaswani et al. (2017) Table 3, base model (encoder and decoder
/// together); Devlin et al. (2019) section 3, BERT-Base and BERT-Large. Columns: name, blocks, vector size, heads,
/// feed-forward inner size, parameters.
const MODEL_SIZES: [(&str, &str, &str, &str, &str, &str); 3] = [
    ("Transformer 2017", "6 + 6", "512", "8", "2,048", "65M"),
    ("BERT-Base 2018", "12", "768", "12", "3,072", "110M"),
    ("BERT-Large 2018", "24", "1,024", "16", "4,096", "340M"),
];

fn fig_32_1_block() -> Figure {
    let mut s = Svg::new(
        720,
        420,
        "A Transformer encoder block: self-attention, then a small feed-forward network, each wrapped in a skip connection and a normalisation; the block is stacked many times",
        "A vertical flow: word embeddings plus position codes, multi-head self-attention, add and normalise with a skip connection curving around the attention, a feed-forward network, add and normalise with a second skip connection, and the output; a bracket marks the repeated block. A table lists the sizes of three published models.",
    ).min_text(12);
    let cx = 184.0;
    let (bw, bh) = (216.0, 30.0);
    let stages = [
        ("word embeddings + position codes", tok::NEUTRAL, tok::AXIS),
        ("multi-head self-attention", tok::FILL1, tok::S1),
        ("add, then normalise", tok::SURFACE, tok::AXIS),
        ("feed-forward network", tok::FILL1, tok::S1),
        ("add, then normalise", tok::SURFACE, tok::AXIS),
        ("one vector per word, out", tok::NEUTRAL, tok::AXIS),
    ];
    let ys: Vec<f64> = (0..stages.len()).map(|i| 40.0 + i as f64 * 58.0).collect();
    for (i, (name, fill, stroke)) in stages.iter().enumerate() {
        s.labelled_box(cx - bw / 2.0, ys[i], bw, bh, name, fill, stroke);
        if i + 1 < stages.len() {
            s.arrow(cx, ys[i] + bh + 2.0, cx, ys[i + 1] - 2.0, None);
        }
    }
    // skip connections: the input of each sub-layer is added back after it
    s.skip(cx - bw / 2.0 - 2.0, ys[0] + bh + 10.0, cx - bw / 2.0 - 2.0, ys[2] + bh / 2.0, -40.0, Some("skip"));
    s.skip(cx - bw / 2.0 - 2.0, ys[2] + bh + 10.0, cx - bw / 2.0 - 2.0, ys[4] + bh / 2.0, -40.0, Some("skip"));
    s.brace_right(cx + bw / 2.0 + 8.0, ys[1] - 6.0, ys[4] + bh + 6.0, "one block,");
    s.text(cx + bw / 2.0 + 18.0, (ys[1] + ys[4] + bh) / 2.0 + 20.0, "repeated", 11, tok::INK2, Anchor::Start);
    // table of sizes
    let tx = 378.0;
    let heads = ["blocks", "vector", "heads", "inner", "params"];
    let cols = [tx + 140.0, tx + 184.0, tx + 224.0, tx + 264.0, tx + 306.0];
    s.text_bold(tx, 70.0, "sizes in the papers", 12, tok::INK, Anchor::Start);
    for (j, h) in heads.iter().enumerate() {
        s.text(cols[j], 96.0, h, 11, tok::INK2, Anchor::Middle);
    }
    for (i, row) in MODEL_SIZES.iter().enumerate() {
        let y = 120.0 + i as f64 * 30.0;
        if i % 2 == 0 {
            s.rect(tx - 6.0, y - 16.0, 328.0, 26.0, tok::NEUTRAL, None);
        }
        s.text(tx, y + 2.0, row.0, 11, tok::INK, Anchor::Start);
        for (j, v) in [row.1, row.2, row.3, row.4, row.5].iter().enumerate() {
            s.text(cols[j], y + 2.0, v, 11, tok::INK, Anchor::Middle);
        }
    }
    let notes = [
        "vector: numbers per word inside the model",
        "inner: width of the feed-forward layer",
        "params: parameters; M: million",
        "the 2017 model: 6 encoder and 6 decoder blocks;",
        "BERT keeps only the encoder side",
    ];
    for (i, n) in notes.iter().enumerate() {
        s.text(tx, 232.0 + i as f64 * 16.0, n, 12, tok::INK2, Anchor::Start);
    }
    s.text(tx, 330.0, "sources: Vaswani et al. 2017, Table 3;", 12, tok::MUTED, Anchor::Start);
    s.text(tx, 346.0, "Devlin et al. 2019, section 3", 12, tok::MUTED, Anchor::Start);
    ("b32-1-block".to_string(), s.finish())
}

fn fig_32_2_pretrain() -> Figure {
    let mut s = Svg::new(
        720,
        360,
        "BERT is first trained to fill in hidden words, then the same weights are fine-tuned with a small new head for a task such as sorting blurbs",
        "Top: the blurb B1 with the word mystery replaced by a mask token passes through BERT, which guesses the hidden word. Bottom: the same BERT, copied, reads a whole blurb and a small new head on the first position's output answers whether the book is a mystery.",
    ).min_text(12);
    // pre-training
    s.text_bold(16.0, 28.0, "1. pre-training: guess the hidden word", 13, tok::INK, Anchor::Start);
    let mut toks: Vec<&str> = vec!["[CLS]"];
    toks.extend(tokens(BLURBS[0]));
    toks.push("[SEP]");
    let masked = 3usize;
    toks[masked] = "[MASK]";
    let fills: Vec<&'static str> = (0..toks.len()).map(|i| if i == masked { tok::FILL3 } else { tok::SURFACE }).collect();
    let boxes = word_boxes(&mut s, 16.0, 136.0, &toks, &fills);
    let right = boxes[boxes.len() - 1].0 + boxes[boxes.len() - 1].1;
    s.rect_bold(16.0, 84.0, right - 16.0, 36.0, tok::FILL1, tok::S1);
    s.text((16.0 + right) / 2.0, 107.0, "BERT: 12 blocks that read the whole blurb in both directions", 12, tok::INK, Anchor::Middle);
    let (mx, mw) = boxes[masked];
    s.arrow(mx + mw / 2.0, 82.0, mx + mw / 2.0, 58.0, None);
    s.text(mx + mw / 2.0 + 8.0, 64.0, &format!("guess: {}", tokens(BLURBS[0])[masked - 1]), 12, tok::INK, Anchor::Start);
    s.text(right + 12.0, 94.0, "15% of word pieces", 11, tok::INK2, Anchor::Start);
    s.text(right + 12.0, 110.0, "are chosen for guessing;", 12, tok::INK2, Anchor::Start);
    s.text(right + 12.0, 126.0, "no labels needed", 11, tok::INK2, Anchor::Start);
    s.line(16.0, 184.0, 704.0, 184.0, tok::GRID, 1.0);
    // fine-tuning
    s.text_bold(16.0, 212.0, "2. fine-tuning: same weights, one small new part", 13, tok::INK, Anchor::Start);
    let mut toks2: Vec<&str> = vec!["[CLS]"];
    toks2.extend(tokens(BLURBS[3]));
    toks2.push("[SEP]");
    let fills2: Vec<&'static str> = (0..toks2.len()).map(|i| if i == 0 { tok::FILL3 } else { tok::SURFACE }).collect();
    let b2 = word_boxes(&mut s, 16.0, 316.0, &toks2, &fills2);
    let right2 = b2[b2.len() - 1].0 + b2[b2.len() - 1].1;
    s.rect_bold(16.0, 264.0, right2 - 16.0, 36.0, tok::FILL1, tok::S1);
    s.text((16.0 + right2) / 2.0, 287.0, "BERT, starting from the pre-trained weights", 12, tok::INK, Anchor::Middle);
    let (cx0, cw0) = b2[0];
    s.arrow(cx0 + cw0 / 2.0, 262.0, cx0 + cw0 / 2.0, 244.0, None);
    s.labelled_box(cx0, 222.0, 150.0, 22.0, "new head: 2 outputs", tok::FILL3, tok::S3);
    s.arrow(cx0 + 152.0, 233.0, cx0 + 180.0, 233.0, None);
    s.text(cx0 + 186.0, 237.0, "is this blurb a mystery? yes or no", 12, tok::INK, Anchor::Start);
    s.text(right2 + 12.0, 280.0, "the output at [CLS]", 11, tok::INK2, Anchor::Start);
    s.text(right2 + 12.0, 296.0, "stands for the whole input", 11, tok::INK2, Anchor::Start);
    ("b32-2-pretrain".to_string(), s.finish())
}

// =================================================================================================
// Chapter 33. Sentence embeddings and sentence transformers
// =================================================================================================

/// The SBERT paper's timing for 10,000 sentences (Reimers and Gurevych 2019, section 1), quoted: about 65 hours with a
/// BERT cross-encoder on a V100 GPU, about 5 seconds to embed with SBERT, about 0.01 seconds for the cosines.
const SBERT_HOURS: f64 = 65.0;
const SBERT_EMBED_SECONDS: f64 = 5.0;
const SBERT_COSINE_SECONDS: f64 = 0.01;
const SBERT_SENTENCES: u64 = 10_000;

fn fig_33_1_cross_bi() -> Figure {
    let mut s = Svg::new(
        720,
        330,
        "A cross-encoder reads both blurbs together and returns one score; a bi-encoder turns each blurb into its own vector once and compares vectors",
        "Left: blurbs B1 and B4 joined into one input with separator tokens pass through BERT and a small head that outputs a similarity score. Right: B1 and B4 each pass through the same BERT and a pooling step, giving vectors u and v whose cosine is the score.",
    ).min_text(12);
    // cross-encoder
    s.text_bold(20.0, 30.0, "cross-encoder", 13, tok::INK, Anchor::Start);
    s.text(20.0, 46.0, "both blurbs in one input", 11, tok::INK2, Anchor::Start);
    s.labelled_box(20.0, 250.0, 300.0, 26.0, "[CLS] B1 [SEP] B4 [SEP]", tok::NEUTRAL, tok::AXIS);
    s.arrow(170.0, 248.0, 170.0, 216.0, None);
    s.rect_bold(60.0, 170.0, 220.0, 44.0, tok::FILL1, tok::S1);
    s.text(170.0, 197.0, "BERT reads the pair", 12, tok::INK, Anchor::Middle);
    s.arrow(170.0, 168.0, 170.0, 138.0, None);
    s.labelled_box(110.0, 110.0, 120.0, 26.0, "small head", tok::SURFACE, tok::AXIS);
    s.arrow(170.0, 108.0, 170.0, 84.0, None);
    s.text(170.0, 76.0, "score for this pair", 12, tok::INK, Anchor::Middle);
    s.text(20.0, 304.0, "a new pass through BERT for every pair", 11, tok::INK2, Anchor::Start);
    s.line(360.0, 24.0, 360.0, 314.0, tok::GRID, 1.0);
    // bi-encoder
    s.text_bold(380.0, 30.0, "bi-encoder (SBERT)", 13, tok::INK, Anchor::Start);
    s.text(380.0, 46.0, "each blurb on its own, weights shared", 11, tok::INK2, Anchor::Start);
    for (k, (name, vec_name)) in [("B1", "u"), ("B4", "v")].iter().enumerate() {
        let x = 392.0 + k as f64 * 156.0;
        s.labelled_box(x, 250.0, 124.0, 26.0, name, tok::NEUTRAL, tok::AXIS);
        s.arrow(x + 62.0, 248.0, x + 62.0, 216.0, None);
        s.rect_bold(x, 170.0, 124.0, 44.0, tok::FILL1, tok::S1);
        s.text(x + 62.0, 190.0, "BERT", 12, tok::INK, Anchor::Middle);
        s.text(x + 62.0, 206.0, "then mean pooling", 12, tok::INK2, Anchor::Middle);
        s.arrow(x + 62.0, 168.0, x + 62.0, 140.0, None);
        s.labelled_box(x + 37.0, 114.0, 50.0, 24.0, vec_name, tok::FILL3, tok::S3);
    }
    s.path("M454 114 L454 96 L610 96 L610 114", tok::INK2, 1.0);
    s.arrow(532.0, 96.0, 532.0, 82.0, None);
    s.text(532.0, 74.0, "cosine(u, v)", 12, tok::INK, Anchor::Middle);
    s.text(380.0, 304.0, "vectors made once, compared by cosine", 12, tok::INK2, Anchor::Start);
    ("b33-1-cross-bi".to_string(), s.finish())
}

fn fig_33_2_cost() -> Figure {
    let n = SBERT_SENTENCES;
    let pairs = n * (n - 1) / 2;
    let blurb_pairs = (BLURBS.len() * (BLURBS.len() - 1) / 2) as u64;
    let cross_s = SBERT_HOURS * 3600.0;
    let mut s = Svg::new(
        720,
        300,
        "Finding the closest pair among 10,000 sentences: about 65 hours with a cross-encoder, about 5 seconds with SBERT plus a hundredth of a second of cosines",
        "Horizontal bars on a logarithmic time axis from a hundredth of a second to a million seconds: the cross-encoder bar reaches 234,000 seconds, the SBERT embedding bar 5 seconds and the cosine bar 0.01 seconds.",
    ).min_text(12);
    let (lo, hi) = (0.001f64.log10(), 1.0e6f64.log10());
    let (x0, w) = (250.0, 380.0);
    let sx = |v: f64| x0 + (v.log10() - lo) / (hi - lo) * w;
    let ticks = [(0.001, "0.001 s"), (1.0, "1 s"), (60.0, "1 min"), (3600.0, "1 h"), (86400.0, "1 day"), (1.0e6, "10⁶ s")];
    for (t, l) in ticks {
        s.line(sx(t), 60.0, sx(t), 196.0, tok::GRID, 1.0);
        s.text(sx(t), 212.0, l, 11, tok::MUTED, Anchor::Middle);
    }
    let bars = [
        (format!("cross-encoder: {} pairs", thousands(pairs)), cross_s, tok::S2, format!("about {} hours", numt(SBERT_HOURS, 0))),
        ("SBERT: embed 10,000 sentences".to_string(), SBERT_EMBED_SECONDS, tok::S1, format!("about {} s", numt(SBERT_EMBED_SECONDS, 0))),
        ("then all the cosines".to_string(), SBERT_COSINE_SECONDS, tok::S3, format!("about {} s", numt(SBERT_COSINE_SECONDS, 2))),
    ];
    for (i, (name, v, colour, label)) in bars.iter().enumerate() {
        let y = 70.0 + i as f64 * 42.0;
        s.text(x0 - 10.0, y + 17.0, name, 12, tok::INK, Anchor::End);
        s.rect(x0, y + 4.0, sx(*v) - x0, 20.0, colour, None);
        s.text(sx(*v) + 6.0, y + 18.0, label, 11, tok::INK, Anchor::Start);
    }
    s.text(
        360.0,
        244.0,
        &format!("n sentences make n × (n − 1) ÷ 2 pairs: {} sentences make {}; our five blurbs make {}", thousands(n), thousands(pairs), blurb_pairs),
        11,
        tok::INK2,
        Anchor::Middle,
    );
    s.text(
        360.0,
        262.0,
        &format!("65 hours is {} seconds, about {} times the 5 seconds", thousands(cross_s as u64), thousands((cross_s / SBERT_EMBED_SECONDS) as u64)),
        11,
        tok::INK2,
        Anchor::Middle,
    );
    s.text(360.0, 284.0, "times from Reimers and Gurevych (2019), measured on a V100 GPU", 11, tok::MUTED, Anchor::Middle);
    ("b33-2-cost".to_string(), s.finish())
}

fn fig_33_3_siamese() -> Figure {
    let u = mean_pool(BLURBS[0]);
    let v = mean_pool(BLURBS[3]);
    let d: Vec<f64> = u.iter().zip(&v).map(|(a, b)| (a - b).abs()).collect();
    let joined: Vec<f64> = u.iter().chain(v.iter()).chain(d.iter()).cloned().collect();
    let mut s = Svg::new(
        720,
        388,
        "SBERT's training path: two blurbs pass through one shared encoder, their mean-pooled vectors u and v are joined with |u − v|, and a small classifier names the relation",
        "Blurbs B1 and B4 each go through the same encoder and mean pooling, giving four-number vectors u and v computed from the made-up feature table; the element-wise difference is computed; the twelve joined numbers feed a classifier with three outputs: entailment, neutral, contradiction.",
    ).min_text(12);
    let fmt_row = |vals: &[f64], fill: &'static str| -> Vec<Vec<Cell>> { vec![vals.iter().map(|x| cell(num(*x, 2)).fill(fill)).collect()] };
    for (k, (name, text, vals, y)) in [("B1", BLURBS[0], &u, 40.0), ("B4", BLURBS[3], &v, 150.0)].iter().enumerate() {
        s.text(16.0, *y + 16.0, &format!("{name}: {text}"), 11, tok::INK, Anchor::Start);
        s.rect_bold(16.0, *y + 28.0, 196.0, 34.0, tok::FILL1, tok::S1);
        s.text(114.0, *y + 50.0, "encoder + mean pooling", 12, tok::INK, Anchor::Middle);
        s.arrow(214.0, *y + 45.0, 250.0, *y + 45.0, None);
        s.cells_wh(254.0, *y + 32.0, 44.0, 26.0, &fmt_row(vals, tok::FILL3));
        s.text(254.0 + 88.0, *y + 22.0, if k == 0 { "u" } else { "v" }, 12, tok::INK, Anchor::Middle);
    }
    s.text(114.0, 124.0, "one encoder, used twice:", 11, tok::S1, Anchor::Middle);
    s.text(114.0, 138.0, "the same weights", 11, tok::S1, Anchor::Middle);
    s.text(342.0, 262.0, "|u − v|", 12, tok::INK, Anchor::Middle);
    s.cells_wh(254.0, 270.0, 44.0, 26.0, &fmt_row(&d, tok::NEUTRAL));
    s.text(254.0, 316.0, "their gap, number by number", 12, tok::INK2, Anchor::Start);
    // joined vector
    let jx = 460.0;
    s.text_bold(jx + 60.0, 40.0, "(u, v, |u − v|)", 12, tok::INK, Anchor::Middle);
    let jcells: Vec<Vec<Cell>> = joined.iter().enumerate().map(|(i, x)| vec![cell(num(*x, 2)).fill(if i < 8 { tok::FILL3 } else { tok::NEUTRAL })]).collect();
    s.cells_wh(jx + 38.0, 52.0, 44.0, 22.0, &jcells);
    s.arrow(430.0, 90.0, jx + 32.0, 90.0, None);
    s.arrow(430.0, 196.0, jx + 32.0, 196.0, None);
    s.arrow(430.0, 283.0, jx + 32.0, 283.0, None);
    s.arrow(jx + 84.0, 184.0, 584.0, 184.0, None);
    s.labelled_box2(586.0, 150.0, 122.0, 68.0, "classifier", "3 scores, softmax", tok::SURFACE, tok::AXIS);
    for (i, l) in ["entailment", "neutral", "contradiction"].iter().enumerate() {
        s.text(647.0, 246.0 + i as f64 * 16.0, l, 11, tok::INK2, Anchor::Middle);
    }
    s.text(360.0, 356.0, "the encoder here is the made-up table of Figure 25.1; SBERT uses BERT.", 12, tok::MUTED, Anchor::Middle);
    s.text(360.0, 372.0, "After training, only u and v are kept and compared by cosine; the classifier is thrown away.", 12, tok::MUTED, Anchor::Middle);
    ("b33-3-siamese".to_string(), s.finish())
}

fn fig_33_4_cosine() -> Figure {
    let vocab = vocabulary(&BLURBS);
    let counts = count_matrix(&BLURBS, &vocab);
    let pooled: Vec<Vec<f64>> = BLURBS.iter().map(|b| mean_pool(b)).collect();
    let cc = cosine_matrix(&counts);
    let ce = cosine_matrix(&pooled);
    let mut s = Svg::new(
        720,
        340,
        "Word counts say B1 and B4 have nothing in common; averaged word vectors say they are close, because both are about crime in a small place",
        "Two five by five heat maps of cosine similarity between the blurbs: the left from word counts, the right from mean-pooled made-up feature vectors; the B1 and B4 cells are outlined in both.",
    ).min_text(12);
    let names = ["B1", "B2", "B3", "B4", "B5"];
    let cw = 50.0;
    let y0 = 76.0;
    for (m, x0, title) in [(&cc, 70.0, "cosine of word counts"), (&ce, 420.0, "cosine of averaged word vectors")] {
        s.text_bold(x0 + 2.5 * cw, 34.0, title, 13, tok::INK, Anchor::Middle);
        s.cells_wh(x0, y0, cw, 34.0, &heat_cells(m, 1.0, 2));
        row_labels(&mut s, x0 - 8.0, y0 - 4.5, 34.0, &names, 12);
        col_labels(&mut s, x0, y0 - 10.0, cw, &names, 12);
        s.rect_bold(x0 + 3.0 * cw, y0, cw, 34.0, "none", tok::S2);
        s.rect_bold(x0, y0 + 3.0 * 34.0, cw, 34.0, "none", tok::S2);
    }
    let yb = y0 + 5.0 * 34.0 + 30.0;
    s.text(360.0, yb, &format!("B1 and B4 share no word: {} by counts, {} by averaged vectors", num(cc[0][3], 2), num(ce[0][3], 2)), 12, tok::INK, Anchor::Middle);
    s.text(
        360.0,
        yb + 18.0,
        &format!("the bicycle blurb B5 stays apart: {} with B1 by averaged vectors", num(ce[0][4], 2)),
        11,
        tok::INK2,
        Anchor::Middle,
    );
    s.text(360.0, yb + 36.0, "word vectors: the made-up table of Figure 25.1, averaged over every word of a blurb", 11, tok::MUTED, Anchor::Middle);
    ("b33-4-cosine".to_string(), s.finish())
}
