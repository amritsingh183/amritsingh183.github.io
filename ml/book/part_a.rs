//! Figures of Parts I and II of "Machine Learning, Drawn Out": chapters 1 to 11. The values these figures draw are
//! computed here, from the shared inputs of the book's specification (its section B) or from the small inputs named in
//! this file. Labels also type numbers as text, such as names ("row 1", "frame 3"), sizes ("1 × 2") and the ranges of
//! the activation functions ("between 0 and 1").
use crate::Figure;
use crate::svg::{Anchor, Cell, Svg, cell, num, numt, text_w, thousands, tok};

type V2 = (f64, f64);
type M2 = [[f64; 2]; 2];
type V3 = [f64; 3];

// ------------------------------------------------------------------------------------------------------------------
// Shared inputs (the specification's section B). Everything else in this file is derived from these or named below.
// ------------------------------------------------------------------------------------------------------------------
/// Her basis: the columns are where her î and ĵ sit in our grid.
const HER: M2 = [[2.0, -1.0], [1.0, 1.0]];
/// The question vector (−1, 2).
const QV: V2 = (-1.0, 2.0);
/// A quarter turn anticlockwise.
const ROT: M2 = [[0.0, -1.0], [1.0, 0.0]];
/// The eigen example.
const EIG_M: M2 = [[3.0, 1.0], [0.0, 2.0]];
/// Cramer's system 3x + 2y = −4 and −x + 2y = −2.
const CR_A: M2 = [[3.0, 2.0], [-1.0, 2.0]];
const CR_B: V2 = (-4.0, -2.0);
/// The cross-product pair.
const XV: V3 = [2.0, 0.0, 1.0];
const XW: V3 = [0.0, 3.0, 1.0];
/// The computation graph: u = b·c, v = a + u, J = K·v.
const G_A: f64 = 5.0;
const G_B: f64 = 3.0;
const G_C: f64 = 2.0;
const G_K: f64 = 3.0;
/// The price-versus-pages sample (price in hundreds of rupees).
const PAGES: [f64; 8] = [60.0, 90.0, 120.0, 160.0, 200.0, 260.0, 320.0, 400.0];
const TRAIN_NOISE: [f64; 8] = [0.6, -0.5, 0.8, -0.9, 0.4, -0.7, 0.5, -0.3];
const TEST_NOISE: [f64; 8] = [-0.4, 0.7, -0.6, 0.5, -0.8, 0.3, -0.5, 0.6];
/// The two ridge points and the three penalties.
const RIDGE: [V2; 2] = [(1.0, 2.0), (3.0, 5.0)];
const LAMBDAS: [f64; 3] = [0.0, 1.0, 3.0];

// ------------------------------------------------------------------------------------------------------------------
// Inputs chosen in this file (the specification leaves them to the chapter's author).
// ------------------------------------------------------------------------------------------------------------------
/// Chapter 1: a 300-page book priced at 200 rupees, both axes counted in hundreds.
const BOOK: V2 = (3.0, 2.0);
/// Chapter 3: a matrix that swaps orientation, and one that flattens the plane.
const FLIP: M2 = [[1.0, 2.0], [2.0, 1.0]];
const FLAT: M2 = [[2.0, 4.0], [1.0, 2.0]];
/// Chapter 3: the boxes' length, depth, height and the sideways slant of the second one.
const PRISM: [f64; 4] = [3.0, 2.0, 2.0, 1.2];
/// Chapter 4: the projection example, the three sign cases, the pair for the rotation test and the shear.
const PW: V2 = (3.0, 4.0);
const PV: V2 = (4.0, 2.0);
const SIGN_CASES: [V2; 3] = [(4.0, 2.0), (4.0, -3.0), (-4.0, -1.0)];
const PAIR: [V2; 2] = [(2.0, 1.0), (1.0, 2.0)];
const SHEAR: M2 = [[1.0, 1.0], [0.0, 1.0]];
/// Chapter 4: the third edge of the box built on the cross-product pair.
const XX: V3 = [0.0, 0.0, 2.0];
/// Chapter 6: an ordinary vector to contrast with the eigenvectors; the λ window of the determinant plot and the two
/// outer λ values of its parallelogram panels; three vectors for the quarter turn; how many times M is applied.
const EIG_PROBE: V2 = (1.0, 1.0);
const EIG_RANGE: (f64, f64) = (0.0, 4.0);
const EIG_PANEL_ENDS: (f64, f64) = (1.0, 4.0);
const ROT_PROBES: [V2; 3] = [(2.0, 0.0), (1.0, 1.0), (-1.0, 2.0)];
const EIG_POWER: u32 = 10;
/// Chapter 7: the neuron's weights and bias, two books (pages in hundreds, age in decades).
const N_W: V2 = (0.9, -1.2);
const N_B: f64 = 0.3;
const N_BOOKS: [V2; 2] = [(3.0, 2.0), (1.0, 2.0)];
/// Chapter 7: two straight layers and the input used to check that they collapse into one.
const L1_W: M2 = [[1.0, -1.0], [2.0, 1.0]];
const L1_B: V2 = (0.0, -1.0);
const L2_W: V2 = (1.0, 2.0);
const L2_B: f64 = 0.5;
const L_X: V2 = (1.0, 2.0);
/// Chapter 7: where the bent line meets the price curve (pages); it bends at all but the last.
const KNOTS: [f64; 4] = [0.0, 100.0, 250.0, 400.0];
/// Chapter 7: the activation plots run from −Z_EDGE to Z_EDGE and are read off at ±Z_PROBE.
const Z_EDGE: f64 = 4.0;
const Z_PROBE: f64 = 2.0;
/// Chapter 8: the nudge.
const NUDGE: f64 = 0.001;
/// Chapter 9: the degree of the squiggle, one less than the number of training books.
const SQUIGGLE: usize = 7;
/// Chapter 10: the polynomial degrees compared by cross-validation, and the number of blocks.
const CV_DEGREES: [usize; 5] = [1, 2, 3, 5, 7];
const FOLDS: usize = 4;
/// Chapter 11: six new books around a gentler true line (intercept, slope) plus fixed noise.
const RIDGE_TEST_X: [f64; 6] = [0.5, 1.5, 2.0, 2.5, 3.5, 4.0];
const RIDGE_TEST_NOISE: [f64; 6] = [0.2, -0.3, 0.1, -0.2, 0.3, -0.1];
const RIDGE_TRUE: V2 = (1.9, 0.8);

/// Every figure of these chapters, in chapter order.
pub fn figures() -> Vec<Figure> {
    vec![
        fig_1_1(),
        fig_1_2(),
        fig_2_1(),
        fig_2_2(),
        fig_2_3(),
        fig_2_4(),
        fig_3_1(),
        fig_3_2(),
        cramer_fig(1),
        cramer_fig(0),
        fig_3_5(),
        fig_4_1(),
        fig_4_2(),
        fig_4_3(),
        fig_4_4(),
        fig_4_5(),
        fig_5_1(),
        fig_5_2(),
        fig_6_1(),
        fig_6_2(),
        fig_6_3(),
        fig_6_4(),
        fig_7_1(),
        fig_7_2(),
        fig_7_3(),
        fig_7_4(),
        fig_8_1(),
        fig_8_2(),
        fig_8_3(),
        fits_fig(false),
        fits_fig(true),
        fig_9_3(),
        fig_10_1(),
        fig_10_2(),
        fig_11_1(),
        fig_11_2(),
        fig_11_3(),
        fig_11_4(),
    ]
}

// ------------------------------------------------------------------------------------------------------------------
// Arithmetic
// ------------------------------------------------------------------------------------------------------------------
fn mv(m: M2, v: V2) -> V2 {
    (m[0][0] * v.0 + m[0][1] * v.1, m[1][0] * v.0 + m[1][1] * v.1)
}

fn mm(a: M2, b: M2) -> M2 {
    let mut r = [[0.0; 2]; 2];
    for (i, row) in r.iter_mut().enumerate() {
        for (j, e) in row.iter_mut().enumerate() {
            *e = a[i][0] * b[0][j] + a[i][1] * b[1][j];
        }
    }
    r
}

fn det2(m: M2) -> f64 {
    m[0][0] * m[1][1] - m[0][1] * m[1][0]
}

fn col(m: M2, j: usize) -> V2 {
    (m[0][j], m[1][j])
}

/// The matrix whose columns are `a` and `b`.
fn cols(a: V2, b: V2) -> M2 {
    [[a.0, b.0], [a.1, b.1]]
}

fn dot2(a: V2, b: V2) -> f64 {
    a.0 * b.0 + a.1 * b.1
}

fn add2(a: V2, b: V2) -> V2 {
    (a.0 + b.0, a.1 + b.1)
}

fn sc2(k: f64, a: V2) -> V2 {
    (k * a.0, k * a.1)
}

fn cross(a: V3, b: V3) -> V3 {
    [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]]
}

fn dot3(a: V3, b: V3) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn add3(a: V3, b: V3) -> V3 {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

fn sc3(k: f64, a: V3) -> V3 {
    [k * a[0], k * a[1], k * a[2]]
}

/// The determinant of the 3×3 matrix with columns c0, c1, c2, by the rule of Sarrus (independent of `cross`).
fn det3(c0: V3, c1: V3, c2: V3) -> f64 {
    c0[0] * c1[1] * c2[2] + c1[0] * c2[1] * c0[2] + c2[0] * c0[1] * c1[2]
        - c2[0] * c1[1] * c0[2]
        - c1[0] * c0[1] * c2[2]
        - c0[0] * c2[1] * c1[2]
}

fn relu(z: f64) -> f64 {
    z.max(0.0)
}

fn sigmoid(z: f64) -> f64 {
    1.0 / (1.0 + (-z).exp())
}

fn true_price(x: f64) -> f64 {
    12.0 * (1.0 - (-x / 180.0).exp())
}

fn train_prices() -> Vec<f64> {
    PAGES.iter().zip(TRAIN_NOISE).map(|(x, n)| true_price(*x) + n).collect()
}

fn test_prices() -> Vec<f64> {
    PAGES.iter().zip(TEST_NOISE).map(|(x, n)| true_price(*x) + n).collect()
}

/// Pages rescaled to about −1…1 so that high polynomial powers stay well conditioned.
fn tscale(x: f64) -> f64 {
    (x - 230.0) / 170.0
}

/// Least squares by Householder QR: the coefficients c minimising |A c − y|².
fn lstsq(rows: &[Vec<f64>], y: &[f64]) -> Vec<f64> {
    let m = rows.len();
    let n = rows[0].len();
    let mut a: Vec<Vec<f64>> = rows.to_vec();
    let mut b = y.to_vec();
    for k in 0..n {
        let norm = (k..m).map(|i| a[i][k] * a[i][k]).sum::<f64>().sqrt();
        if norm == 0.0 {
            continue;
        }
        let alpha = if a[k][k] > 0.0 { -norm } else { norm };
        let mut v = vec![0.0; m];
        for i in k..m {
            v[i] = a[i][k];
        }
        v[k] -= alpha;
        let vn: f64 = (k..m).map(|i| v[i] * v[i]).sum();
        if vn == 0.0 {
            continue;
        }
        for j in k..n {
            let s: f64 = (k..m).map(|i| v[i] * a[i][j]).sum();
            for i in k..m {
                a[i][j] -= 2.0 * v[i] * s / vn;
            }
        }
        let s: f64 = (k..m).map(|i| v[i] * b[i]).sum();
        for i in k..m {
            b[i] -= 2.0 * v[i] * s / vn;
        }
    }
    let mut c = vec![0.0; n];
    for i in (0..n).rev() {
        let s: f64 = ((i + 1)..n).map(|j| a[i][j] * c[j]).sum();
        c[i] = (b[i] - s) / a[i][i];
    }
    c
}

/// A polynomial of the given degree in rescaled pages, fitted by least squares.
fn polyfit(xs: &[f64], ys: &[f64], deg: usize) -> Vec<f64> {
    let rows: Vec<Vec<f64>> = xs.iter().map(|x| (0..=deg).map(|k| tscale(*x).powi(k as i32)).collect()).collect();
    lstsq(&rows, ys)
}

fn polyval(c: &[f64], x: f64) -> f64 {
    let t = tscale(x);
    c.iter().rev().fold(0.0, |acc, k| acc * t + k)
}

fn sse(c: &[f64], xs: &[f64], ys: &[f64]) -> f64 {
    xs.iter().zip(ys).map(|(x, y)| (y - polyval(c, *x)).powi(2)).sum()
}

/// All sixteen priced books sorted by pages (the training copy of a page count first) and dealt into blocks like cards.
fn cv_books() -> Vec<(f64, f64, usize)> {
    let (tr, te) = (train_prices(), test_prices());
    let mut all = Vec::new();
    for i in 0..PAGES.len() {
        all.push((PAGES[i], tr[i]));
        all.push((PAGES[i], te[i]));
    }
    all.iter().enumerate().map(|(i, (x, y))| (*x, *y, i % FOLDS)).collect()
}

/// The mean squared error of a polynomial of degree `deg` on each held-out block.
fn cv_scores(deg: usize) -> Vec<f64> {
    let books = cv_books();
    (0..FOLDS)
        .map(|k| {
            let (xs, ys): (Vec<f64>, Vec<f64>) = books.iter().filter(|b| b.2 != k).map(|b| (b.0, b.1)).unzip();
            let c = polyfit(&xs, &ys, deg);
            let held: Vec<&(f64, f64, usize)> = books.iter().filter(|b| b.2 == k).collect();
            held.iter().map(|b| (b.1 - polyval(&c, b.0)).powi(2)).sum::<f64>() / held.len() as f64
        })
        .collect()
}

/// Means of the ridge points' x and y.
fn ridge_means() -> V2 {
    let n = RIDGE.len() as f64;
    (RIDGE.iter().map(|p| p.0).sum::<f64>() / n, RIDGE.iter().map(|p| p.1).sum::<f64>() / n)
}

/// Ridge line (intercept, slope) for the two points: the intercept is not penalised.
fn ridge_line(lambda: f64) -> V2 {
    let (xm, ym) = ridge_means();
    let sxx: f64 = RIDGE.iter().map(|p| (p.0 - xm).powi(2)).sum();
    let sxy: f64 = RIDGE.iter().map(|p| (p.0 - xm) * (p.1 - ym)).sum();
    let b = sxy / (sxx + lambda);
    (ym - b * xm, b)
}

/// Sum of squared residuals of the line (a, b) on the two ridge points.
fn ridge_ssr(line: V2) -> f64 {
    RIDGE.iter().map(|p| (p.1 - (line.0 + line.1 * p.0)).powi(2)).sum()
}

/// The ridge cost of a slope when the intercept is the best one for that slope.
fn ridge_cost_at_slope(b: f64, lambda: f64) -> f64 {
    let (xm, ym) = ridge_means();
    ridge_ssr((ym - b * xm, b)) + lambda * b * b
}

fn ridge_test() -> Vec<V2> {
    RIDGE_TEST_X.iter().zip(RIDGE_TEST_NOISE).map(|(x, n)| (*x, RIDGE_TRUE.0 + RIDGE_TRUE.1 * x + n)).collect()
}

fn ridge_test_sse(line: V2) -> f64 {
    ridge_test().iter().map(|p| (p.1 - (line.0 + line.1 * p.0)).powi(2)).sum()
}

/// An exact fraction, for the change-of-basis chapter where the inverse has thirds.
#[derive(Clone, Copy)]
struct Q {
    n: i64,
    d: i64,
}

fn gcd(a: i64, b: i64) -> i64 {
    let (mut a, mut b) = (a.abs(), b.abs());
    while b != 0 {
        let t = a % b;
        a = b;
        b = t;
    }
    a
}

impl Q {
    fn new(n: i64, d: i64) -> Q {
        let g = gcd(n, d).max(1);
        let s = if d < 0 { -1 } else { 1 };
        Q { n: s * n / g, d: s * d / g }
    }
    fn int(v: f64) -> Q {
        Q::new(v.round() as i64, 1)
    }
    fn add(self, o: Q) -> Q {
        Q::new(self.n * o.d + o.n * self.d, self.d * o.d)
    }
    fn mul(self, o: Q) -> Q {
        Q::new(self.n * o.n, self.d * o.d)
    }
    fn neg(self) -> Q {
        Q::new(-self.n, self.d)
    }
    fn val(self) -> f64 {
        self.n as f64 / self.d as f64
    }
    fn s(self) -> String {
        let t = if self.d == 1 { format!("{}", self.n.abs()) } else { format!("{}/{}", self.n.abs(), self.d) };
        if self.n < 0 { format!("−{t}") } else { t }
    }
}

type QM = [[Q; 2]; 2];
type QV = (Q, Q);

fn qm(m: M2) -> QM {
    [[Q::int(m[0][0]), Q::int(m[0][1])], [Q::int(m[1][0]), Q::int(m[1][1])]]
}

fn qmv(m: QM, v: QV) -> QV {
    (m[0][0].mul(v.0).add(m[0][1].mul(v.1)), m[1][0].mul(v.0).add(m[1][1].mul(v.1)))
}

fn qmm(a: QM, b: QM) -> QM {
    let e = |i: usize, j: usize| a[i][0].mul(b[0][j]).add(a[i][1].mul(b[1][j]));
    [[e(0, 0), e(0, 1)], [e(1, 0), e(1, 1)]]
}

fn qinv(m: QM) -> QM {
    let det = m[0][0].mul(m[1][1]).add(m[0][1].mul(m[1][0]).neg());
    let r = Q::new(det.d, det.n);
    [[m[1][1].mul(r), m[0][1].neg().mul(r)], [m[1][0].neg().mul(r), m[0][0].mul(r)]]
}

fn qvs(v: QV) -> String {
    format!("({}, {})", v.0.s(), v.1.s())
}

// ------------------------------------------------------------------------------------------------------------------
// Formatting and drawing helpers
// ------------------------------------------------------------------------------------------------------------------
/// Up to two decimals, trailing zeros dropped, typographic minus.
fn f(v: f64) -> String {
    numt(v, 2)
}

/// A factor in a written product: negative numbers in parentheses.
fn fp(v: f64) -> String {
    if v < 0.0 { format!("({})", f(v)) } else { f(v) }
}

fn v2s(v: V2) -> String {
    format!("({}, {})", f(v.0), f(v.1))
}

fn v3s(v: V3) -> String {
    format!("({}, {}, {})", f(v[0]), f(v[1]), f(v[2]))
}

/// A sum of named terms plus a constant in plain form: coefficients ±1 are not written, zero terms are dropped.
fn lin(terms: &[(f64, &str)], k: f64) -> String {
    let mut out = String::new();
    for (c, name) in terms {
        if *c == 0.0 {
            continue;
        }
        let mag = c.abs();
        let body = if (mag - 1.0).abs() < 1e-12 { name.to_string() } else { format!("{}{}", f(mag), name) };
        if out.is_empty() {
            out = if *c < 0.0 { format!("−{body}") } else { body };
        } else {
            out.push_str(if *c < 0.0 { " − " } else { " + " });
            out.push_str(&body);
        }
    }
    if out.is_empty() {
        return f(k);
    }
    if k != 0.0 {
        out.push_str(if k < 0.0 { " − " } else { " + " });
        out.push_str(&f(k.abs()));
    }
    out
}

/// "y = a + bx" for a line given as (intercept, slope).
fn line_label(l: V2) -> String {
    let slope = if (l.1 - 1.0).abs() < 1e-12 { "x".to_string() } else { format!("{}x", f(l.1)) };
    format!("y = {} + {slope}", f(l.0))
}

/// A canvas coordinate for path data: at most one decimal, ASCII minus.
fn pc(v: f64) -> String {
    let r = (v * 10.0).round() / 10.0;
    if (r - r.round()).abs() < 1e-9 { format!("{}", r.round() as i64) } else { format!("{r:.1}") }
}

/// A label on a light patch of the surface colour, so grid lines do not run through it.
fn tag(s: &mut Svg, x: f64, y: f64, t: &str, size: u8, colour: &str, anchor: Anchor) {
    let w = text_w(t, size) + 6.0;
    let x0 = match &anchor {
        Anchor::Start => x - 3.0,
        Anchor::Middle => x - w / 2.0,
        Anchor::End => x - w + 3.0,
    };
    s.rect(x0, y - f64::from(size), w, f64::from(size) + 5.0, tok::SURFACE, None);
    s.text(x, y, t, size, colour, anchor);
}

/// Labels the tip of the arrow from the origin to `v`, just beyond the tip along the arrow's own direction, so the
/// label does not sit on the shaft; near the window's right edge it moves below the tip instead.
fn tip_label(s: &mut Svg, fr: &Frame, v: V2, t: &str, size: u8, colour: &str) {
    let (tip, o) = (fr.pt(v), fr.pt((0.0, 0.0)));
    let (dx, dy) = (tip.0 - o.0, tip.1 - o.1);
    let len = (dx * dx + dy * dy).sqrt().max(1e-9);
    let (ux, uy) = (dx / len, dy / len);
    let (x, y) = (tip.0 + 12.0 * ux, tip.1 + 12.0 * uy + 4.0);
    let w = text_w(t, size);
    if ux > 0.3 && x + w > fr.right() {
        tag(s, tip.0, tip.1 + 18.0, t, size, colour, Anchor::Middle);
    } else if ux > 0.3 {
        tag(s, x, y, t, size, colour, Anchor::Start);
    } else if ux < -0.3 && x - w < fr.left {
        tag(s, tip.0, tip.1 - 10.0, t, size, colour, Anchor::Middle);
    } else if ux < -0.3 {
        tag(s, x, y, t, size, colour, Anchor::End);
    } else {
        tag(s, x, y + if uy < 0.0 { -4.0 } else { 8.0 }, t, size, colour, Anchor::Middle);
    }
}

/// A data window drawn at a fixed place on the canvas; `sx` and `sy` are pixels per unit.
#[derive(Clone, Copy)]
struct Frame {
    left: f64,
    top: f64,
    sx: f64,
    sy: f64,
    x0: f64,
    x1: f64,
    y0: f64,
    y1: f64,
}

impl Frame {
    /// Equal scales on both axes (geometry).
    fn square(left: f64, top: f64, s: f64, x: (f64, f64), y: (f64, f64)) -> Frame {
        Frame { left, top, sx: s, sy: s, x0: x.0, x1: x.1, y0: y.0, y1: y.1 }
    }
    /// A plot of the given size on the canvas.
    fn fit(left: f64, top: f64, w: f64, h: f64, x: (f64, f64), y: (f64, f64)) -> Frame {
        Frame { left, top, sx: w / (x.1 - x.0), sy: h / (y.1 - y.0), x0: x.0, x1: x.1, y0: y.0, y1: y.1 }
    }
    fn px(&self, x: f64) -> f64 {
        self.left + (x - self.x0) * self.sx
    }
    fn py(&self, y: f64) -> f64 {
        self.top + (self.y1 - y) * self.sy
    }
    fn pt(&self, p: V2) -> V2 {
        (self.px(p.0), self.py(p.1))
    }
    fn right(&self) -> f64 {
        self.px(self.x1)
    }
    fn bottom(&self) -> f64 {
        self.py(self.y0)
    }
    fn cx(&self) -> f64 {
        (self.left + self.right()) / 2.0
    }
}

/// Clips the segment a–b (data coordinates) to the frame's window (Liang–Barsky).
fn clip(fr: &Frame, a: V2, b: V2) -> Option<(V2, V2)> {
    let (dx, dy) = (b.0 - a.0, b.1 - a.1);
    let (mut t0, mut t1) = (0.0_f64, 1.0_f64);
    for (p, q) in [(-dx, a.0 - fr.x0), (dx, fr.x1 - a.0), (-dy, a.1 - fr.y0), (dy, fr.y1 - a.1)] {
        if p == 0.0 {
            if q < 0.0 {
                return None;
            }
        } else {
            let r = q / p;
            if p < 0.0 {
                t0 = t0.max(r);
            } else {
                t1 = t1.min(r);
            }
        }
    }
    if t0 > t1 {
        return None;
    }
    Some(((a.0 + t0 * dx, a.1 + t0 * dy), (a.0 + t1 * dx, a.1 + t1 * dy)))
}

/// A data-space segment clipped to the window.
fn seg(s: &mut Svg, fr: &Frame, a: V2, b: V2, colour: &str, width: f64) {
    if let Some((p, q)) = clip(fr, a, b) {
        let (p, q) = (fr.pt(p), fr.pt(q));
        s.line(p.0, p.1, q.0, q.1, colour, width);
    }
}

/// The integer grid of a window, with the two axes a shade darker, and optional tick numbers.
fn grid(s: &mut Svg, fr: &Frame, numbers: bool) {
    let (xa, xb) = (fr.x0.ceil() as i64, fr.x1.floor() as i64);
    let (ya, yb) = (fr.y0.ceil() as i64, fr.y1.floor() as i64);
    for k in xa..=xb {
        let x = k as f64;
        s.line(fr.px(x), fr.py(fr.y0), fr.px(x), fr.py(fr.y1), if k == 0 { tok::AXIS } else { tok::GRID }, 1.0);
    }
    for k in ya..=yb {
        let y = k as f64;
        s.line(fr.px(fr.x0), fr.py(y), fr.px(fr.x1), fr.py(y), if k == 0 { tok::AXIS } else { tok::GRID }, 1.0);
    }
    if numbers {
        let y_lab = if fr.y0 <= 0.0 && fr.y1 >= 0.0 { fr.py(0.0) + 13.0 } else { fr.bottom() + 13.0 };
        for k in xa..=xb {
            if k != 0 {
                s.text(fr.px(k as f64), y_lab, &f(k as f64), 11, tok::MUTED, Anchor::Middle);
            }
        }
        let x_lab = if fr.x0 <= 0.0 && fr.x1 >= 0.0 { fr.px(0.0) - 5.0 } else { fr.left - 5.0 };
        for k in ya..=yb {
            if k != 0 {
                s.text(x_lab, fr.py(k as f64) + 4.0, &f(k as f64), 11, tok::MUTED, Anchor::End);
            }
        }
    }
}

/// The lines of a moved grid: images of the lines x = k and y = k under `m`, clipped to the window; the images of
/// the two axes are a shade darker.
fn tgrid(s: &mut Svg, fr: &Frame, m: M2) {
    let (a, b) = (col(m, 0), col(m, 1));
    for k in -24..=24 {
        let k = k as f64;
        for (base, dir) in [(sc2(k, a), b), (sc2(k, b), a)] {
            let p = add2(base, sc2(-60.0, dir));
            let q = add2(base, sc2(60.0, dir));
            if k == 0.0 {
                seg(s, fr, p, q, tok::MUTED, 1.5);
            } else {
                seg(s, fr, p, q, tok::AXIS, 1.0);
            }
        }
    }
}

/// A coloured arrow between two canvas points with an open head.
fn arrow_c(s: &mut Svg, a: V2, b: V2, colour: &str, width: f64) {
    let (dx, dy) = (b.0 - a.0, b.1 - a.1);
    let len = (dx * dx + dy * dy).sqrt();
    if len < 1e-9 {
        return;
    }
    let (ux, uy) = (dx / len, dy / len);
    let head = 9.0_f64.min(len * 0.45);
    let (c, sn) = (0.42_f64.cos(), 0.42_f64.sin());
    let l = (b.0 - head * (ux * c - uy * sn), b.1 - head * (uy * c + ux * sn));
    let r = (b.0 - head * (ux * c + uy * sn), b.1 - head * (uy * c - ux * sn));
    s.line(a.0, a.1, b.0 - ux, b.1 - uy, colour, width);
    s.path(&format!("M{} {} L{} {} L{} {}", pc(l.0), pc(l.1), pc(b.0), pc(b.1), pc(r.0), pc(r.1)), colour, width);
}

/// A coloured arrow between two data points.
fn arrow_d(s: &mut Svg, fr: &Frame, a: V2, b: V2, colour: &str, width: f64) {
    arrow_c(s, fr.pt(a), fr.pt(b), colour, width);
}

/// A dashed canvas line drawn as short segments.
fn dash(s: &mut Svg, a: V2, b: V2, colour: &str, width: f64) {
    let (dx, dy) = (b.0 - a.0, b.1 - a.1);
    let len = (dx * dx + dy * dy).sqrt();
    let (on, off) = (5.0, 3.5);
    let mut t = 0.0;
    while t < len {
        let e = (t + on).min(len);
        s.line(a.0 + dx * t / len, a.1 + dy * t / len, a.0 + dx * e / len, a.1 + dy * e / len, colour, width);
        t += on + off;
    }
}

fn dash_d(s: &mut Svg, fr: &Frame, a: V2, b: V2, colour: &str, width: f64) {
    if let Some((p, q)) = clip(fr, a, b) {
        dash(s, fr.pt(p), fr.pt(q), colour, width);
    }
}

/// Fills a convex polygon (canvas points) with light diagonal hatching and outlines it.
fn hatch(s: &mut Svg, pts: &[V2], fill: &str, edge: &str) {
    let (nx, ny) = (0.5_f64.sqrt(), 0.5_f64.sqrt());
    let proj: Vec<f64> = pts.iter().map(|p| p.0 * nx + p.1 * ny).collect();
    let lo = proj.iter().cloned().fold(f64::INFINITY, f64::min);
    let hi = proj.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let gap = 4.0;
    let mut t = (lo / gap).ceil() * gap;
    while t <= hi {
        let mut hits: Vec<V2> = Vec::new();
        for i in 0..pts.len() {
            let (a, b) = (pts[i], pts[(i + 1) % pts.len()]);
            let (fa, fb) = (a.0 * nx + a.1 * ny - t, b.0 * nx + b.1 * ny - t);
            if (fa <= 0.0 && fb > 0.0) || (fa > 0.0 && fb <= 0.0) {
                let u = fa / (fa - fb);
                hits.push((a.0 + (b.0 - a.0) * u, a.1 + (b.1 - a.1) * u));
            }
        }
        if hits.len() >= 2 {
            hits.sort_by(|p, q| (p.0 - p.1).partial_cmp(&(q.0 - q.1)).unwrap_or(std::cmp::Ordering::Equal));
            let (p, q) = (hits[0], hits[hits.len() - 1]);
            s.line(p.0, p.1, q.0, q.1, fill, 1.0);
        }
        t += gap;
    }
    let mut d = format!("M{} {}", pc(pts[0].0), pc(pts[0].1));
    for p in &pts[1..] {
        d.push_str(&format!(" L{} {}", pc(p.0), pc(p.1)));
    }
    d.push_str(" Z");
    s.path(&d, edge, 1.2);
}

fn hatch_d(s: &mut Svg, fr: &Frame, pts: &[V2], fill: &str, edge: &str) {
    let c: Vec<V2> = pts.iter().map(|p| fr.pt(*p)).collect();
    hatch(s, &c, fill, edge);
}

/// A sampled curve y = g(x), clipped to the window (pieces outside are skipped).
fn curve(s: &mut Svg, fr: &Frame, g: &dyn Fn(f64) -> f64, x: (f64, f64), n: usize, colour: &str, width: f64) {
    let pts: Vec<V2> = (0..=n).map(|i| x.0 + (x.1 - x.0) * i as f64 / n as f64).map(|x| (x, g(x))).collect();
    let mut d = String::new();
    let mut open = false;
    for w in pts.windows(2) {
        match clip(fr, w[0], w[1]) {
            Some((p, q)) => {
                let (p, q) = (fr.pt(p), fr.pt(q));
                if !open {
                    d.push_str(&format!("M{} {} ", pc(p.0), pc(p.1)));
                    open = true;
                }
                d.push_str(&format!("L{} {} ", pc(q.0), pc(q.1)));
                if !(fr.y0..=fr.y1).contains(&w[1].1) {
                    open = false;
                }
            }
            None => open = false,
        }
    }
    s.path(d.trim_end(), colour, width);
}

/// A dashed curve, drawn as every other short piece of a sampled curve.
fn dashed_curve(s: &mut Svg, fr: &Frame, g: &dyn Fn(f64) -> f64, x: (f64, f64), n: usize, colour: &str, width: f64) {
    let pts: Vec<V2> = (0..=n).map(|i| x.0 + (x.1 - x.0) * i as f64 / n as f64).map(|x| (x, g(x))).collect();
    for w in pts.windows(2).step_by(2) {
        seg(s, fr, w[0], w[1], colour, width);
    }
}

/// A 2×2 matrix as cells; with `tint`, the first column's numbers are green and the second's orange.
fn mat2(s: &mut Svg, x: f64, y: f64, cw: f64, m: M2, tint: bool) {
    let rows: Vec<Vec<Cell>> = (0..2)
        .map(|i| {
            (0..2)
                .map(|j| {
                    let c = cell(f(m[i][j]));
                    if tint { c.ink(if j == 0 { tok::S3 } else { tok::S2 }) } else { c }
                })
                .collect()
        })
        .collect();
    s.cells_wh(x, y, cw, 24.0, &rows);
}

/// A table whose columns have their own widths.
fn table(s: &mut Svg, x: f64, y: f64, widths: &[f64], h: f64, rows: &[Vec<Cell>]) {
    for (r, row) in rows.iter().enumerate() {
        let mut xx = x;
        for (c, cl) in row.iter().enumerate() {
            s.cells_wh(xx, y + r as f64 * h, widths[c], h, &[vec![cl.clone()]]);
            xx += widths[c];
        }
    }
}

/// Text lines starting at (x, y), 17 units apart, each with its own colour.
fn lines(s: &mut Svg, x: f64, y: f64, items: &[(&str, &str)]) {
    for (i, (t, c)) in items.iter().enumerate() {
        s.text(x, y + 17.0 * i as f64, t, 12, c, Anchor::Start);
    }
}

// ------------------------------------------------------------------------------------------------------------------
// Chapter 1
// ------------------------------------------------------------------------------------------------------------------
fn fig_1_1() -> Figure {
    let (ih, jh) = ((1.0, 0.0), (0.0, 1.0));
    let (vx, vy) = (dot2(BOOK, ih), dot2(BOOK, jh));
    let mut s = Svg::new(
        720,
        320,
        "A vector's coordinates are its shadows on the axes",
        &format!(
            "An arrow from the origin to {} with dashed lines down to the x-axis at {} and across to the y-axis at {}; each coordinate equals the dot product with a unit vector.",
            v2s(BOOK),
            f(vx),
            f(vy)
        ),
    ).min_text(12);
    let fr = Frame::square(58.0, 28.0, 62.0, (-0.6, 4.4), (-0.6, 3.6));
    grid(&mut s, &fr, false);
    let off = 0.2;
    for k in 1..=4 {
        s.text(fr.px(k as f64), fr.py(0.0) + 32.0, &k.to_string(), 11, tok::MUTED, Anchor::Middle);
    }
    for k in 1..=3 {
        s.text(fr.px(-off) - 9.0, fr.py(k as f64) + 4.0, &k.to_string(), 11, tok::MUTED, Anchor::End);
    }
    // the two shadows, just outside the axes
    s.line(fr.px(0.0), fr.py(-off), fr.px(vx), fr.py(-off), tok::S3, 5.0);
    s.line(fr.px(-off), fr.py(0.0), fr.px(-off), fr.py(vy), tok::S2, 5.0);
    dash_d(&mut s, &fr, BOOK, (vx, -off), tok::INK2, 1.0);
    dash_d(&mut s, &fr, BOOK, (-off, vy), tok::INK2, 1.0);
    arrow_d(&mut s, &fr, (0.0, 0.0), BOOK, tok::S1, 2.5);
    arrow_d(&mut s, &fr, (0.0, 0.0), ih, tok::S3, 2.5);
    arrow_d(&mut s, &fr, (0.0, 0.0), jh, tok::S2, 2.5);
    s.text(fr.px(1.0) - 2.0, fr.py(0.0) - 8.0, "î", 13, tok::S3, Anchor::Middle);
    s.text(fr.px(0.0) + 9.0, fr.py(1.0) + 4.0, "ĵ", 13, tok::S2, Anchor::Start);
    s.text_bold(fr.px(vx) + 8.0, fr.py(vy) - 6.0, &format!("v = {}", v2s(BOOK)), 12, tok::S1, Anchor::Start);
    tag(&mut s, fr.right(), fr.py(0.0) - 8.0, "pages (hundreds)", 12, tok::INK2, Anchor::End);
    s.text(fr.px(0.0) + 8.0, fr.top + 10.0, "price (hundreds of rupees)", 11, tok::INK2, Anchor::Start);
    let x = 430.0;
    s.text_bold(x, 56.0, &format!("v = {}", v2s(BOOK)), 13, tok::INK, Anchor::Start);
    s.text(x, 76.0, &format!("a {}-page book priced at {} rupees,", f(100.0 * BOOK.0), f(100.0 * BOOK.1)), 11, tok::INK2, Anchor::Start);
    s.text(x, 91.0, "with both axes counted in hundreds", 11, tok::INK2, Anchor::Start);
    s.text_bold(x, 128.0, &format!("shadow on the x-axis = {}", f(vx)), 12, tok::S3, Anchor::Start);
    s.text(x, 147.0, &format!("v · î = {} × {} + {} × {} = {}", f(BOOK.0), f(ih.0), f(BOOK.1), f(ih.1), f(vx)), 12, tok::INK, Anchor::Start);
    s.text_bold(x, 186.0, &format!("shadow on the y-axis = {}", f(vy)), 12, tok::S2, Anchor::Start);
    s.text(x, 205.0, &format!("v · ĵ = {} × {} + {} × {} = {}", f(BOOK.0), f(jh.0), f(BOOK.1), f(jh.1), f(vy)), 12, tok::INK, Anchor::Start);
    s.text(x, 246.0, "each coordinate is a shadow on an axis,", 11, tok::INK2, Anchor::Start);
    s.text(x, 261.0, "and each shadow is a dot product", 11, tok::INK2, Anchor::Start);
    s.text(x, 276.0, "with a unit vector", 11, tok::INK2, Anchor::Start);
    ("b01-1-projections".to_string(), s.finish())
}

/// Draws x·î + y·ĵ as unit steps, tip to tail, then the arrow itself.
fn steps(s: &mut Svg, fr: &Frame, v: V2) {
    let sx = if v.0 < 0.0 { -1.0 } else { 1.0 };
    let n = v.0.abs().round() as i64;
    for i in 0..n {
        let a = (sx * i as f64, 0.0);
        arrow_d(s, fr, a, (a.0 + sx, 0.0), tok::S3, 2.2);
    }
    let m = v.1.abs().round() as i64;
    let sy = if v.1 < 0.0 { -1.0 } else { 1.0 };
    for j in 0..m {
        let a = (v.0, sy * j as f64);
        arrow_d(s, fr, a, (v.0, a.1 + sy), tok::S2, 2.2);
    }
    arrow_d(s, fr, (0.0, 0.0), v, tok::S1, 2.5);
}

fn fig_1_2() -> Figure {
    let mut s = Svg::new(
        720,
        300,
        "A vector is a combination of the unit vectors",
        &format!(
            "Left: {} green steps along î and {} orange steps along ĵ reach {}. Right: one step backwards along î and {} along ĵ reach {}.",
            f(BOOK.0),
            f(BOOK.1),
            v2s(BOOK),
            f(QV.1),
            v2s(QV)
        ),
    ).min_text(12);
    let book_words = format!("the book: {} pages, {} rupees", f(100.0 * BOOK.0), f(100.0 * BOOK.1));
    let left = Frame::square(40.0, 44.0, 56.0, (-0.5, 4.0), (-0.5, 2.8));
    let right = Frame::square(440.0, 44.0, 56.0, (-2.0, 1.5), (-0.5, 2.8));
    for (fr, v, l1, l2) in [
        (&left, BOOK, "three steps along î, then two along ĵ", book_words.as_str()),
        (&right, QV, "one step back along î, then two along ĵ", "the vector chapters 2 and 5 ask about"),
    ] {
        grid(&mut s, fr, true);
        steps(&mut s, fr, v);
        s.text_bold(fr.cx(), 26.0, &format!("{} = {} î + {} ĵ", v2s(v), f(v.0), f(v.1)), 13, tok::INK, Anchor::Middle);
        s.text(fr.cx(), fr.bottom() + 34.0, l1, 12, tok::INK2, Anchor::Middle);
        s.text(fr.cx(), fr.bottom() + 52.0, l2, 11, tok::INK2, Anchor::Middle);
    }
    ("b01-2-combination".to_string(), s.finish())
}

// ------------------------------------------------------------------------------------------------------------------
// Chapter 2
// ------------------------------------------------------------------------------------------------------------------
fn fig_2_1() -> Figure {
    let (a1, a2) = (col(HER, 0), col(HER, 1));
    let img = mv(HER, QV);
    let mut s = Svg::new(
        720,
        264,
        "A matrix moves the whole grid",
        &format!(
            "Left: the usual grid with î, ĵ and the vector {}. Right: the same grid after the matrix A; î lands on {}, ĵ on {} and the vector on {}.",
            v2s(QV),
            v2s(a1),
            v2s(a2),
            v2s(img)
        ),
    ).min_text(12);
    let l = Frame::square(20.0, 40.0, 32.0, (-5.2, 3.2), (-2.2, 3.4));
    let r = Frame::square(400.0, 40.0, 32.0, (-5.2, 3.2), (-2.2, 3.4));
    grid(&mut s, &l, false);
    arrow_d(&mut s, &l, (0.0, 0.0), (1.0, 0.0), tok::S3, 2.5);
    arrow_d(&mut s, &l, (0.0, 0.0), (0.0, 1.0), tok::S2, 2.5);
    arrow_d(&mut s, &l, (0.0, 0.0), QV, tok::S1, 2.5);
    tag(&mut s, l.px(QV.0) - 6.0, l.py(QV.1) - 6.0, &v2s(QV), 12, tok::S1, Anchor::End);
    grid(&mut s, &r, false);
    tgrid(&mut s, &r, HER);
    arrow_d(&mut s, &r, (0.0, 0.0), a1, tok::S3, 2.5);
    arrow_d(&mut s, &r, (0.0, 0.0), a2, tok::S2, 2.5);
    arrow_d(&mut s, &r, (0.0, 0.0), img, tok::S1, 2.5);
    tag(&mut s, r.px(a1.0) + 6.0, r.py(a1.1) + 16.0, &v2s(a1), 12, tok::S3, Anchor::Start);
    tag(&mut s, r.px(a2.0) - 4.0, r.py(a2.1) - 8.0, &v2s(a2), 12, tok::S2, Anchor::End);
    tag(&mut s, r.px(img.0), r.py(img.1) - 10.0, &v2s(img), 12, tok::S1, Anchor::Middle);
    s.text_bold(l.cx(), 26.0, "before: the usual grid", 13, tok::INK, Anchor::Middle);
    s.text_bold(r.cx(), 26.0, "after A: the grid follows î and ĵ", 13, tok::INK, Anchor::Middle);
    let mx = (l.right() + r.left) / 2.0 - 30.0;
    s.text(mx + 30.0, 120.0, "A", 13, tok::INK, Anchor::Middle);
    mat2(&mut s, mx, 128.0, 30.0, HER, true);
    s.arrow(mx + 2.0, 196.0, mx + 58.0, 196.0, None);
    s.text(l.cx(), l.bottom() + 22.0, &format!("î = (1, 0), ĵ = (0, 1), v = {}", v2s(QV)), 12, tok::INK2, Anchor::Middle);
    s.text(r.cx(), r.bottom() + 22.0, &format!("î lands on {}, ĵ on {}: the columns of A", v2s(a1), v2s(a2)), 12, tok::INK2, Anchor::Middle);
    ("b02-1-grid".to_string(), s.finish())
}

fn fig_2_2() -> Figure {
    let (a1, a2) = (col(HER, 0), col(HER, 1));
    let p1 = sc2(QV.0, a1);
    let p2 = sc2(QV.1, a2);
    let img = mv(HER, QV);
    let mut s = Svg::new(
        720,
        262,
        "Matrix times vector is a combination of the columns",
        &format!(
            "In the moved grid, {} times the first column {} followed by {} times the second column {} lands on {}, the matrix times {}.",
            f(QV.0),
            v2s(a1),
            f(QV.1),
            v2s(a2),
            v2s(img),
            v2s(QV)
        ),
    ).min_text(12);
    let fr = Frame::square(24.0, 30.0, 40.0, (-5.2, 2.8), (-2.0, 2.6));
    grid(&mut s, &fr, false);
    tgrid(&mut s, &fr, HER);
    arrow_d(&mut s, &fr, (0.0, 0.0), p1, tok::S3, 2.5);
    arrow_d(&mut s, &fr, p1, add2(p1, p2), tok::S2, 2.5);
    arrow_d(&mut s, &fr, (0.0, 0.0), img, tok::S1, 2.5);
    tag(&mut s, fr.px(p1.0 / 2.0) + 10.0, fr.py(p1.1 / 2.0) + 18.0, &format!("{} × {}", f(QV.0), v2s(a1)), 12, tok::S3, Anchor::Start);
    tag(&mut s, fr.px(p1.0 + p2.0 / 2.0) - 12.0, fr.py(p1.1 + p2.1 / 2.0) + 16.0, &format!("{} × {}", f(QV.1), v2s(a2)), 12, tok::S2, Anchor::End);
    tag(&mut s, fr.px(img.0), fr.py(img.1) - 10.0, &v2s(img), 12, tok::S1, Anchor::Middle);
    let x = 384.0;
    s.text_bold(x, 50.0, &format!("A · {}", v2s(QV)), 13, tok::INK, Anchor::Start);
    lines(
        &mut s,
        x,
        76.0,
        &[
            (&format!("= {} × {} + {} × {}", f(QV.0), v2s(a1), f(QV.1), v2s(a2)), tok::INK),
            (&format!("= {} + {}", v2s(p1), v2s(p2)), tok::INK),
            (&format!("= {}", v2s(img)), tok::S1),
        ],
    );
    s.text(x, 150.0, "the same numbers, row by row:", 12, tok::INK2, Anchor::Start);
    lines(
        &mut s,
        x,
        170.0,
        &[
            (&format!("row 1: {} × {} + {} × {} = {}", f(HER[0][0]), fp(QV.0), fp(HER[0][1]), fp(QV.1), f(img.0)), tok::INK),
            (&format!("row 2: {} × {} + {} × {} = {}", f(HER[1][0]), fp(QV.0), fp(HER[1][1]), fp(QV.1), f(img.1)), tok::INK),
        ],
    );
    s.text(x, 226.0, "the columns say where î and ĵ go;", 11, tok::INK2, Anchor::Start);
    s.text(x, 241.0, "the vector says how much of each to take", 11, tok::INK2, Anchor::Start);
    ("b02-2-combination".to_string(), s.finish())
}

fn fig_2_3() -> Figure {
    let p = mm(ROT, HER);
    let mut s = Svg::new(
        720,
        262,
        "Row by column: how two matrices multiply",
        &format!(
            "Three frames. In each of the first two, one column of A is turned on its side and laid over each row of R; multiplying the stacked pairs and adding gives one column of R·A. The last frame shows the product, with columns {} and {}.",
            v2s(col(p, 0)),
            v2s(col(p, 1))
        ),
    ).min_text(12);
    let cw = 30.0;
    for j in 0..2 {
        let fx = 14.0 + j as f64 * 236.0;
        s.text_bold(fx, 24.0, &format!("frame {}: column {} of A", j + 1, j + 1), 12, tok::INK, Anchor::Start);
        s.text(fx, 40.0, "on its side, over each row of R", 11, tok::INK2, Anchor::Start);
        for i in 0..2 {
            let y = 62.0 + i as f64 * 70.0;
            s.text(fx, y - 6.0, &format!("row {} of R", i + 1), 11, tok::INK2, Anchor::Start);
            let over: Vec<Vec<Cell>> = vec![vec![cell(f(HER[0][j])).fill(tok::FILL1), cell(f(HER[1][j])).fill(tok::FILL1)]];
            s.cells_wh(fx + 4.0, y, cw, 24.0, &over);
            s.rect_bold(fx + 4.0, y, 2.0 * cw, 24.0, "none", tok::S1);
            let row: Vec<Vec<Cell>> = vec![vec![cell(f(ROT[i][0])), cell(f(ROT[i][1]))]];
            s.cells_wh(fx + 4.0, y + 24.0, cw, 24.0, &row);
            s.text(
                fx + 4.0 + 2.0 * cw + 10.0,
                y + 29.0,
                &format!("{}×{} + {}×{} = {}", fp(ROT[i][0]), fp(HER[0][j]), fp(ROT[i][1]), fp(HER[1][j]), f(p[i][j])),
                12,
                tok::INK,
                Anchor::Start,
            );
        }
        s.text(fx, 222.0, &format!("column {} of R·A: {}", j + 1, v2s(col(p, j))), 12, tok::S1, Anchor::Start);
    }
    let fx = 14.0 + 2.0 * 236.0;
    s.text_bold(fx, 24.0, "frame 3: the product R·A", 12, tok::INK, Anchor::Start);
    s.text(fx, 40.0, "first A, then R", 11, tok::INK2, Anchor::Start);
    let rows: Vec<Vec<Cell>> = (0..2).map(|i| (0..2).map(|j| cell(f(p[i][j])).fill(if j == 0 { tok::FILL1 } else { tok::FILL2 })).collect()).collect();
    s.cells_wh(fx + 8.0, 90.0, 44.0, 32.0, &rows);
    s.text(fx, 180.0, "each entry is one row of R", 11, tok::INK2, Anchor::Start);
    s.text(fx, 195.0, "times one column of A", 11, tok::INK2, Anchor::Start);
    s.line(14.0 + 236.0 - 10.0, 12.0, 14.0 + 236.0 - 10.0, 236.0, tok::GRID, 1.0);
    s.line(14.0 + 472.0 - 10.0, 12.0, 14.0 + 472.0 - 10.0, 236.0, tok::GRID, 1.0);
    ("b02-3-row-by-column".to_string(), s.finish())
}

fn fig_2_4() -> Figure {
    let ra = mm(ROT, HER);
    let ar = mm(HER, ROT);
    let mut s = Svg::new(
        720,
        300,
        "The order of two moves matters",
        &format!(
            "Left: A first, then the quarter turn R; î lands on {} and ĵ on {}. Right: R first, then A; î lands on {} and ĵ on {}. The two unit squares end in different places.",
            v2s(col(ra, 0)),
            v2s(col(ra, 1)),
            v2s(col(ar, 0)),
            v2s(col(ar, 1))
        ),
    ).min_text(12);
    let l = Frame::square(44.0, 44.0, 42.0, (-3.6, 1.6), (-1.6, 2.6));
    let r = Frame::square(404.0, 44.0, 42.0, (-3.6, 1.6), (-1.6, 2.6));
    for (fr, m, head) in [(&l, ra, "first A, then R: R·A"), (&r, ar, "first R, then A: A·R")] {
        grid(&mut s, fr, true);
        let (c1, c2) = (col(m, 0), col(m, 1));
        hatch_d(&mut s, fr, &[(0.0, 0.0), c1, add2(c1, c2), c2], tok::RAMP[2], tok::S1);
        arrow_d(&mut s, fr, (0.0, 0.0), c1, tok::S3, 2.5);
        arrow_d(&mut s, fr, (0.0, 0.0), c2, tok::S2, 2.5);
        s.text_bold(fr.cx(), 26.0, head, 13, tok::INK, Anchor::Middle);
        let ty = fr.bottom() + 24.0;
        mat2(&mut s, fr.left, ty - 10.0, 30.0, m, true);
        s.text(fr.left + 72.0, ty + 7.0, &format!("î → {}, ĵ → {}", v2s(c1), v2s(c2)), 12, tok::INK, Anchor::Start);
        s.text(fr.left + 72.0, ty + 24.0, &format!("the unit square's area becomes {}", f(det2(m).abs())), 11, tok::INK2, Anchor::Start);
    }
    ("b02-4-order".to_string(), s.finish())
}

// ------------------------------------------------------------------------------------------------------------------
// Chapter 3
// ------------------------------------------------------------------------------------------------------------------
fn fig_3_1() -> Figure {
    let id: M2 = [[1.0, 0.0], [0.0, 1.0]];
    let mats = [id, HER, FLIP];
    let mut s = Svg::new(
        720,
        300,
        "The determinant is the factor by which areas grow",
        &format!(
            "Three panels: the unit square has area {}; after A it is a parallelogram of area {}; after S it also has area {} but green and orange have swapped sides, so the determinant is {}.",
            f(det2(id)),
            f(det2(HER)),
            f(det2(FLIP).abs()),
            f(det2(FLIP))
        ),
    ).min_text(12);
    let heads = ["the unit square", "after A", "after S"];
    let notes = ["", "", "green and orange swap sides"];
    for k in 0..3 {
        let fr = Frame::square(24.0 + k as f64 * 240.0, 40.0, 40.0, (-1.4, 3.4), (-0.4, 3.4));
        let m = mats[k];
        let (c1, c2) = (col(m, 0), col(m, 1));
        grid(&mut s, &fr, false);
        hatch_d(&mut s, &fr, &[(0.0, 0.0), c1, add2(c1, c2), c2], tok::RAMP[2], tok::S1);
        arrow_d(&mut s, &fr, (0.0, 0.0), c1, tok::S3, 2.5);
        arrow_d(&mut s, &fr, (0.0, 0.0), c2, tok::S2, 2.5);
        s.text_bold(fr.cx(), 26.0, heads[k], 13, tok::INK, Anchor::Middle);
        let d = det2(m);
        mat2(&mut s, fr.left + 4.0, fr.bottom() + 12.0, 30.0, m, true);
        s.text(
            fr.left + 72.0,
            fr.bottom() + 28.0,
            &format!("det = {}×{} − {}×{} = {}", fp(m[0][0]), fp(m[1][1]), fp(m[0][1]), fp(m[1][0]), f(d)),
            12,
            tok::INK,
            Anchor::Start,
        );
        s.text(fr.left + 72.0, fr.bottom() + 45.0, &format!("area {}", f(d.abs())), 11, tok::INK2, Anchor::Start);
        if !notes[k].is_empty() {
            s.text(fr.left + 4.0, fr.bottom() + 80.0, notes[k], 11, tok::INK2, Anchor::Start);
        }
    }
    ("b03-1-area".to_string(), s.finish())
}

fn fig_3_2() -> Figure {
    let (c1, c2) = (col(FLAT, 0), col(FLAT, 1));
    let d = det2(FLAT);
    let p1 = (2.0, 0.0);
    let p2 = (0.0, 1.0);
    let (q1, q2) = (mv(FLAT, p1), mv(FLAT, p2));
    // with parallel columns, (second column's x, −first column's x) is sent to zero
    let null = (c2.0, -c1.0);
    let zero = mv(FLAT, null);
    let mut s = Svg::new(
        720,
        300,
        "A determinant of zero flattens the plane",
        &format!(
            "Left: the unit square and two different points, {} and {}. Right: after a matrix with determinant {} the whole plane lies on one line, the square is a segment, and both points land on {}, so the move cannot be undone.",
            v2s(p1),
            v2s(p2),
            f(d),
            v2s(q1)
        ),
    ).min_text(12);
    let l = Frame::square(24.0, 44.0, 40.0, (-1.2, 3.2), (-1.6, 1.8));
    let r = Frame::square(340.0, 44.0, 40.0, (-1.2, 7.2), (-1.2, 3.8));
    grid(&mut s, &l, true);
    hatch_d(&mut s, &l, &[(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)], tok::RAMP[2], tok::S1);
    dash_d(&mut s, &l, sc2(-3.0, null), sc2(3.0, null), tok::S3, 1.5);
    s.dot(l.px(p1.0), l.py(p1.1), 5.0, tok::S1, None);
    s.dot(l.px(p2.0), l.py(p2.1), 5.0, tok::S2, None);
    tag(&mut s, l.px(p1.0) + 8.0, l.py(p1.1) - 7.0, &v2s(p1), 12, tok::S1, Anchor::Start);
    tag(&mut s, l.px(p2.0) + 8.0, l.py(p2.1) - 6.0, &v2s(p2), 12, tok::S2, Anchor::Start);
    grid(&mut s, &r, true);
    seg(&mut s, &r, sc2(-5.0, c1), sc2(5.0, c1), tok::INK2, 2.0);
    let corner = add2(c1, c2);
    s.line(r.px(0.0), r.py(0.0), r.px(corner.0), r.py(corner.1), tok::S1, 5.0);
    s.dot(r.px(q1.0), r.py(q1.1), 7.0, tok::S1, None);
    s.dot(r.px(q2.0), r.py(q2.1), 4.0, tok::S2, None);
    s.dot(r.px(zero.0), r.py(zero.1), 4.0, tok::S3, None);
    tag(&mut s, r.px(q1.0) + 10.0, r.py(q1.1) + 18.0, &format!("both land on {}", v2s(q1)), 12, tok::INK, Anchor::Start);
    s.text_bold(l.cx(), 26.0, "before", 13, tok::INK, Anchor::Middle);
    s.text_bold(r.cx(), 26.0, &format!("after a matrix with det {}", f(d)), 13, tok::INK, Anchor::Middle);
    s.text(r.left, r.bottom() + 24.0, &format!("columns {} and {} lie on one line:", v2s(c1), v2s(c2)), 12, tok::INK, Anchor::Start);
    s.text(
        r.left,
        r.bottom() + 41.0,
        &format!("det = {}×{} − {}×{} = {}", fp(FLAT[0][0]), fp(FLAT[1][1]), fp(FLAT[0][1]), fp(FLAT[1][0]), f(d)),
        12,
        tok::INK,
        Anchor::Start,
    );
    s.text(r.left + 190.0, r.bottom() + 41.0, "the square is now a segment", 11, tok::INK2, Anchor::Start);
    s.text(l.left, l.bottom() + 24.0, "two different inputs;", 12, tok::INK2, Anchor::Start);
    s.text(l.left, l.bottom() + 41.0, "every point on the green dashed", 11, tok::S3, Anchor::Start);
    s.text(l.left, l.bottom() + 56.0, &format!("line lands on {}", v2s(zero)), 11, tok::S3, Anchor::Start);
    ("b03-2-collapse".to_string(), s.finish())
}

/// The Cramer figure for one unknown: `which` is 1 for y (area built on î) or 0 for x (area built on ĵ).
fn cramer_fig(which: usize) -> Figure {
    let det = det2(CR_A);
    let v = (det2(cols(CR_B, col(CR_A, 1))) / det, det2(cols(col(CR_A, 0), CR_B)) / det);
    let (base, base_img, name, stem) = if which == 1 {
        ((1.0, 0.0), col(CR_A, 0), "y", "b03-3-cramer-y")
    } else {
        ((0.0, 1.0), col(CR_A, 1), "x", "b03-4-cramer-x")
    };
    // the parallelogram's two edges in the order that makes its signed area equal the unknown
    let (e1, e2, f1, f2) = if which == 1 { (base, v, base_img, CR_B) } else { (v, base, CR_B, base_img) };
    let area_before = det2(cols(e1, e2));
    let area_after = det2(cols(f1, f2));
    let unknown = area_after / det;
    let base_c = if which == 1 { tok::S3 } else { tok::S2 };
    let mut s = Svg::new(
        720,
        330,
        &format!("Cramer's rule for {name}, drawn as areas"),
        &format!(
            "Left: the parallelogram built on the unknown vector and a unit vector has signed area {name}. Right: after the matrix it is built on the output {} and a column of the matrix, {}; its area is {} = {} × {name}, so {name} = {}.",
            v2s(CR_B),
            v2s(base_img),
            f(area_after),
            f(det),
            f(unknown)
        ),
    ).min_text(12);
    let l = Frame::square(28.0, 44.0, 44.0, (-1.3, 1.6), (-1.9, 1.3));
    let r = if which == 1 {
        Frame::square(330.0, 44.0, 44.0, (-4.6, 3.6), (-3.5, 1.3))
    } else {
        Frame::square(350.0, 40.0, 44.0, (-4.6, 3.0), (-2.5, 2.5))
    };
    grid(&mut s, &l, true);
    hatch_d(&mut s, &l, &[(0.0, 0.0), e1, add2(e1, e2), e2], tok::RAMP[2], tok::S1);
    arrow_d(&mut s, &l, (0.0, 0.0), base, base_c, 2.5);
    arrow_d(&mut s, &l, (0.0, 0.0), v, tok::S1, 2.5);
    tag(&mut s, l.px(v.0) - 4.0, l.py(v.1) + 18.0, "(x, y): unknown", 12, tok::S1, Anchor::Middle);
    s.text_bold(l.cx(), 26.0, "before", 13, tok::INK, Anchor::Middle);
    grid(&mut s, &r, true);
    tgrid(&mut s, &r, CR_A);
    hatch_d(&mut s, &r, &[(0.0, 0.0), f1, add2(f1, f2), f2], tok::RAMP[2], tok::S1);
    arrow_d(&mut s, &r, (0.0, 0.0), base_img, base_c, 2.5);
    arrow_d(&mut s, &r, (0.0, 0.0), CR_B, tok::S1, 2.5);
    tag(&mut s, r.px(CR_B.0) + 4.0, r.py(CR_B.1) + 20.0, &format!("output {}: known", v2s(CR_B)), 12, tok::S1, Anchor::Start);
    tag(&mut s, r.px(base_img.0) - 8.0, r.py(base_img.1) + (if which == 1 { 20.0 } else { -8.0 }), &v2s(base_img), 12, base_c, Anchor::End);
    s.text_bold(r.cx(), 26.0, "after the matrix", 13, tok::INK, Anchor::Middle);
    s.arrow(l.right() + 10.0, 120.0, r.left - 10.0, 120.0, None);
    let ty = 280.0;
    s.text(l.left, ty, &format!("signed area = {name}"), 12, tok::INK, Anchor::Start);
    s.text(l.left, ty + 17.0, &format!("(here {name} = {})", f(area_before)), 11, tok::INK2, Anchor::Start);
    s.text(r.left, ty, &format!("signed area = det × {name} = {} × {name}", f(det)), 12, tok::INK, Anchor::Start);
    s.text(r.left, ty + 17.0, &format!("= {}×{} − {}×{} = {}", fp(f1.0), fp(f2.1), fp(f2.0), fp(f1.1), f(area_after)), 12, tok::INK, Anchor::Start);
    s.text_bold(r.left, ty + 35.0, &format!("so {name} = {} ÷ {} = {}", f(area_after), f(det), f(unknown)), 12, tok::S1, Anchor::Start);
    (stem.to_string(), s.finish())
}

fn fig_3_5() -> Figure {
    let [pl, pw, ph, slant] = PRISM;
    let vol = pl * pw * ph;
    let mut s = Svg::new(
        720,
        300,
        "A slanted box has the same volume as an upright one",
        &format!(
            "Two boxes with the same {} by {} base and the same height {}: one upright, one slanted sideways. Both have volume base area times height, {}.",
            f(pl),
            f(pw),
            f(ph),
            f(vol)
        ),
    ).min_text(12);
    // oblique projection: length to the right, depth up and to the right, height up
    let proj = |o: V2, p: V3| -> V2 { (o.0 + 52.0 * (p[0] + 0.45 * p[1]), o.1 - 52.0 * (p[2] + 0.3 * p[1])) };
    for (k, o) in [(0usize, (60.0, 214.0)), (1usize, (370.0, 214.0))] {
        let sh = if k == 0 { 0.0 } else { slant };
        let base = [[0.0, 0.0, 0.0], [pl, 0.0, 0.0], [pl, pw, 0.0], [0.0, pw, 0.0]];
        let top: Vec<V3> = base.iter().map(|b| [b[0] + sh, b[1], b[2] + ph]).collect();
        let bp: Vec<V2> = base.iter().map(|b| proj(o, *b)).collect();
        let tp: Vec<V2> = top.iter().map(|b| proj(o, *b)).collect();
        hatch(&mut s, &[bp[0], bp[1], bp[2], bp[3]], tok::RAMP[1], tok::AXIS);
        dash(&mut s, bp[3], tp[3], tok::AXIS, 1.2);
        for i in 0..4 {
            let j = (i + 1) % 4;
            s.line(tp[i].0, tp[i].1, tp[j].0, tp[j].1, tok::INK2, 1.5);
        }
        for i in [0usize, 1, 2] {
            s.line(bp[i].0, bp[i].1, tp[i].0, tp[i].1, tok::INK2, 1.5);
        }
        // the height, measured straight up beside the box
        let hx = pl + sh.max(0.0) + 0.5;
        let foot = proj(o, [hx, pw, 0.0]);
        let head = proj(o, [hx, pw, ph]);
        s.line(foot.0, foot.1, head.0, head.1, tok::S2, 2.0);
        s.line(foot.0 - 5.0, foot.1, foot.0 + 5.0, foot.1, tok::S2, 2.0);
        s.line(head.0 - 5.0, head.1, head.0 + 5.0, head.1, tok::S2, 2.0);
        dash(&mut s, (bp[2].0, bp[2].1), (foot.0 - 6.0, foot.1), tok::AXIS, 1.0);
        dash(&mut s, (tp[2].0, tp[2].1), (head.0 - 6.0, head.1), tok::AXIS, 1.0);
        s.text(head.0 + 8.0, (foot.1 + head.1) / 2.0 + 4.0, &format!("h = {}", f(ph)), 12, tok::S2, Anchor::Start);
        s.text((bp[0].0 + bp[1].0) / 2.0, bp[0].1 + 18.0, &format!("l = {}", f(pl)), 12, tok::S3, Anchor::Middle);
        s.text((bp[1].0 + bp[2].0) / 2.0 + 10.0, (bp[1].1 + bp[2].1) / 2.0 + 14.0, &format!("w = {}", f(pw)), 12, tok::S1, Anchor::Start);
        let head_txt = if k == 0 { "upright box" } else { "slanted box" };
        s.text_bold(o.0 + 110.0, 26.0, head_txt, 13, tok::INK, Anchor::Middle);
        s.text(o.0 + 110.0, 258.0, &format!("volume = ({} × {}) × {} = {}", f(pl), f(pw), f(ph), f(vol)), 12, tok::INK, Anchor::Middle);
    }
    s.text(360.0, 284.0, "height is measured straight up, not along the slanted edge", 11, tok::INK2, Anchor::Middle);
    ("b03-5-prism".to_string(), s.finish())
}

// ------------------------------------------------------------------------------------------------------------------
// Chapter 4
// ------------------------------------------------------------------------------------------------------------------
fn fig_4_1() -> Figure {
    let d = dot2(PV, PW);
    let wl = dot2(PW, PW).sqrt();
    let proj_len = d / wl;
    let foot = sc2(d / dot2(PW, PW), PW);
    let normal = sc2(1.0 / wl, (-PW.1, PW.0));
    let mut s = Svg::new(
        720,
        330,
        "The dot product is a shadow times a length",
        &format!(
            "Left: v = {} casts a shadow of length {} on the line of w = {}, which is {} long, so v · w = {} × {} = {}. Right: three small panels show the sign: positive when v points along w, zero when perpendicular, negative when it points away.",
            v2s(PV),
            f(proj_len),
            v2s(PW),
            f(wl),
            f(proj_len),
            f(wl),
            f(d)
        ),
    ).min_text(12);
    let fr = Frame::square(34.0, 40.0, 44.0, (-0.8, 5.0), (-0.8, 4.8));
    grid(&mut s, &fr, true);
    dash_d(&mut s, &fr, sc2(-0.2, PW), sc2(1.5, PW), tok::AXIS, 1.0);
    // the shadow, drawn beside w's line so the arrow does not hide it
    let off = sc2(0.16, normal);
    s.line(fr.px(off.0), fr.py(off.1), fr.px(foot.0 + off.0), fr.py(foot.1 + off.1), tok::S3, 6.0);
    dash_d(&mut s, &fr, PV, foot, tok::INK2, 1.2);
    // a right-angle mark where the dashed line meets w's line
    let u = sc2(1.0 / wl, PW);
    let k = 0.16;
    let c1 = add2(foot, sc2(-k, u));
    let c2 = add2(c1, sc2(-k, normal));
    let c3 = add2(foot, sc2(-k, normal));
    let (p1, p2, p3) = (fr.pt(c1), fr.pt(c2), fr.pt(c3));
    s.path(&format!("M{} {} L{} {} L{} {}", pc(p1.0), pc(p1.1), pc(p2.0), pc(p2.1), pc(p3.0), pc(p3.1)), tok::INK2, 1.0);
    arrow_d(&mut s, &fr, (0.0, 0.0), PW, tok::S2, 2.5);
    arrow_d(&mut s, &fr, (0.0, 0.0), PV, tok::S1, 2.5);
    tag(&mut s, fr.px(PW.0) + 8.0, fr.py(PW.1) + 4.0, &format!("w = {}", v2s(PW)), 12, tok::S2, Anchor::Start);
    tag(&mut s, fr.px(PV.0) + 6.0, fr.py(PV.1) + 18.0, &format!("v = {}", v2s(PV)), 12, tok::S1, Anchor::Start);
    tag(&mut s, fr.px(foot.0 + 2.0 * off.0) - 8.0, fr.py(foot.1 + 2.0 * off.1) + 2.0, &format!("shadow {}", f(proj_len)), 12, tok::S3, Anchor::End);
    s.text_bold(fr.cx(), 26.0, &format!("v · w = {} × {} = {}", f(proj_len), f(wl), f(d)), 13, tok::INK, Anchor::Middle);
    s.text(fr.left, fr.bottom() + 24.0, &format!("by numbers: {}×{} + {}×{} = {}", fp(PV.0), fp(PW.0), fp(PV.1), fp(PW.1), f(d)), 12, tok::INK, Anchor::Start);
    let words = ["points along w: positive", "perpendicular: zero", "points away: negative"];
    for (k, v) in SIGN_CASES.iter().enumerate() {
        let m = Frame::square(372.0, 30.0 + k as f64 * 100.0, 10.0, (-4.8, 4.8), (-3.6, 4.6));
        s.rect(m.left, m.top, m.right() - m.left, m.bottom() - m.top, tok::SURFACE, Some(tok::GRID));
        seg(&mut s, &m, (m.x0, 0.0), (m.x1, 0.0), tok::GRID, 1.0);
        seg(&mut s, &m, (0.0, m.y0), (0.0, m.y1), tok::GRID, 1.0);
        arrow_d(&mut s, &m, (0.0, 0.0), PW, tok::S2, 2.0);
        arrow_d(&mut s, &m, (0.0, 0.0), *v, tok::S1, 2.0);
        let dv = dot2(*v, PW);
        s.text_bold(m.right() + 14.0, m.top + 30.0, &format!("v = {}", v2s(*v)), 12, tok::S1, Anchor::Start);
        s.text(m.right() + 14.0, m.top + 48.0, &format!("v · w = {}×{} + {}×{} = {}", fp(v.0), fp(PW.0), fp(v.1), fp(PW.1), f(dv)), 12, tok::INK, Anchor::Start);
        s.text(m.right() + 14.0, m.top + 65.0, words[k], 11, tok::INK2, Anchor::Start);
    }
    ("b04-1-projection".to_string(), s.finish())
}

fn fig_4_2() -> Figure {
    let wl = dot2(PW, PW).sqrt();
    let u = sc2(1.0 / wl, PW);
    let perp = (-u.1, u.0);
    let fv = |p: V2| dot2(PW, p);
    let mut s = Svg::new(
        720,
        330,
        "A 1 × 2 matrix is a vector lying on its side",
        &format!(
            "The plane with lines of equal value for the rule {}. A number line runs along the vector {}; î lands on {}, ĵ on {} and {} on {}, exactly the dot products with {}.",
            lin(&[(PW.0, "x"), (PW.1, "y")], 0.0),
            v2s(PW),
            f(fv((1.0, 0.0))),
            f(fv((0.0, 1.0))),
            v2s(PV),
            f(fv(PV)),
            v2s(PW)
        ),
    ).min_text(12);
    let fr = Frame::square(24.0, 30.0, 40.0, (-2.2, 5.2), (-2.2, 4.8));
    grid(&mut s, &fr, false);
    let step = 5.0;
    for k in -2..=5 {
        let val = step * k as f64;
        let base = sc2(val / wl, u);
        dash_d(&mut s, &fr, add2(base, sc2(-12.0, perp)), add2(base, sc2(12.0, perp)), tok::AXIS, 1.0);
    }
    seg(&mut s, &fr, sc2(-12.0, u), sc2(12.0, u), tok::INK2, 1.5);
    for k in -2..=5 {
        let val = step * k as f64;
        let q = sc2(val / wl, u);
        let p = fr.pt(q);
        s.line(p.0 - 4.0 * u.0, p.1 + 4.0 * u.1, p.0 + 4.0 * u.0, p.1 - 4.0 * u.1, tok::INK2, 1.5);
        if (fr.x0..=fr.x1).contains(&q.0) && (fr.y0..=fr.y1).contains(&q.1) {
            tag(&mut s, p.0 + 8.0, p.1 + 12.0, &f(val), 12, tok::INK2, Anchor::Start);
        }
    }
    for (p, c) in [((1.0, 0.0), tok::S3), ((0.0, 1.0), tok::S2), (PV, tok::S1)] {
        let land = sc2(fv(p) / wl, u);
        dash_d(&mut s, &fr, p, land, c, 1.2);
        arrow_d(&mut s, &fr, (0.0, 0.0), p, c, 2.5);
        s.dot(fr.px(land.0), fr.py(land.1), 4.0, c, None);
    }
    tag(&mut s, fr.px(PV.0) + 6.0, fr.py(PV.1) + 16.0, &v2s(PV), 12, tok::S1, Anchor::Start);
    let x = 360.0;
    s.text_bold(x, 50.0, &format!("the rule [{} {}] turns each arrow into a number", f(PW.0), f(PW.1)), 13, tok::INK, Anchor::Start);
    lines(
        &mut s,
        x,
        80.0,
        &[
            (&format!("î → {}", f(fv((1.0, 0.0)))), tok::S3),
            (&format!("ĵ → {}", f(fv((0.0, 1.0)))), tok::S2),
            (&format!("{} → {}×{} + {}×{} = {}", v2s(PV), f(PW.0), f(PV.0), f(PW.1), f(PV.1), f(fv(PV))), tok::S1),
        ],
    );
    s.text(x, 150.0, &format!("the dot product {} · {} = {}", v2s(PW), v2s(PV), f(dot2(PW, PV))), 12, tok::INK, Anchor::Start);
    s.text(x, 180.0, "dashed lines: every point on one of them", 11, tok::INK2, Anchor::Start);
    s.text(x, 195.0, &format!("gives the same number (0, {}, {}, …)", f(step), f(2.0 * step)), 11, tok::INK2, Anchor::Start);
    s.text(x, 225.0, &format!("the number line runs along {}; each number", v2s(PW)), 11, tok::INK2, Anchor::Start);
    s.text(x, 240.0, &format!("is a shadow on it times the length {}", f(wl)), 11, tok::INK2, Anchor::Start);
    ("b04-2-dual".to_string(), s.finish())
}

fn fig_4_3() -> Figure {
    let [a, b] = PAIR;
    let d0 = dot2(a, b);
    let mats: [M2; 3] = [[[1.0, 0.0], [0.0, 1.0]], ROT, SHEAR];
    let dots: Vec<f64> = mats.iter().map(|m| dot2(mv(*m, a), mv(*m, b))).collect();
    let mut s = Svg::new(
        720,
        256,
        "A rotation keeps dot products; a shear does not",
        &format!(
            "Three panels with the vectors {} and {}. Their dot product is {} before, {} after a quarter turn, and {} after a shear, which also leaves ĵ tilted and longer.",
            v2s(a),
            v2s(b),
            f(dots[0]),
            f(dots[1]),
            f(dots[2])
        ),
    ).min_text(12);
    let heads = ["before", "after the quarter turn R", "after the shear H"];
    for k in 0..3 {
        let fr = Frame::square(14.0 + k as f64 * 238.0, 40.0, 33.0, (-2.6, 3.9), (-0.6, 3.2));
        let m = mats[k];
        let (pa, pb) = (mv(m, a), mv(m, b));
        let (ii, jj) = (col(m, 0), col(m, 1));
        grid(&mut s, &fr, false);
        arrow_d(&mut s, &fr, (0.0, 0.0), ii, tok::S3, 1.6);
        arrow_d(&mut s, &fr, (0.0, 0.0), jj, tok::S2, 1.6);
        arrow_d(&mut s, &fr, (0.0, 0.0), pa, tok::S1, 2.5);
        arrow_d(&mut s, &fr, (0.0, 0.0), pb, tok::INK, 2.5);
        tip_label(&mut s, &fr, pa, &v2s(pa), 12, tok::S1);
        tip_label(&mut s, &fr, pb, &v2s(pb), 12, tok::INK);
        s.text_bold(fr.cx(), 26.0, heads[k], 13, tok::INK, Anchor::Middle);
        let ty = fr.bottom() + 22.0;
        s.text(fr.left, ty, &format!("dot product: {}×{} + {}×{} = {}", fp(pa.0), fp(pb.0), fp(pa.1), fp(pb.1), f(dots[k])), 12, tok::INK, Anchor::Start);
        let ij = dot2(ii, jj);
        let jl = dot2(jj, jj).sqrt();
        s.text(fr.left, ty + 17.0, &format!("î · ĵ = {}, length of ĵ = {}", f(ij), f(jl)), 11, tok::INK2, Anchor::Start);
        let (verdict, vc) = if k == 0 {
            ("the value to keep".to_string(), tok::INK2)
        } else if (dots[k] - d0).abs() < 1e-9 {
            ("dot product kept".to_string(), tok::S3)
        } else {
            (format!("dot product changed from {}", f(d0)), tok::S2)
        };
        s.text(fr.left, ty + 33.0, &verdict, 11, vc, Anchor::Start);
    }
    ("b04-3-rotation-shear".to_string(), s.finish())
}

fn fig_4_4() -> Figure {
    let p = cross(XV, XW);
    let area = dot3(p, p).sqrt();
    let vol_dot = dot3(XX, p);
    let vol_det = det3(XX, XV, XW);
    let height = vol_dot / area;
    let n = sc3(1.0 / area, p);
    let shadow = sc3(height, n);
    let mut s = Svg::new(
        720,
        330,
        "The cross product and the volume of a box",
        &format!(
            "A box built on v = {}, w = {} and x = {}. Its base, the parallelogram of v and w, has area {}, the length of p = v × w = {}. The height is the shadow of x on p's direction, {} ÷ {}, and the volume is {} = det[x v w].",
            v3s(XV),
            v3s(XW),
            v3s(XX),
            f(area),
            v3s(p),
            f(vol_dot),
            f(area),
            f(vol_dot)
        ),
    ).min_text(12);
    // an orthographic view from 120 degrees round and 30 degrees up
    let (phi, theta) = (120.0_f64.to_radians(), 30.0_f64.to_radians());
    let (o, sc) = ((236.0, 250.0), 58.0);
    let cam = |q: V3| -> V2 {
        let xs = -q[0] * phi.sin() + q[1] * phi.cos();
        let ys = -(q[0] * phi.cos() + q[1] * phi.sin()) * theta.sin() + q[2] * theta.cos();
        (o.0 + sc * xs, o.1 - sc * ys)
    };
    let zero = [0.0; 3];
    let base = [zero, XV, add3(XV, XW), XW];
    let bp: Vec<V2> = base.iter().map(|b| cam(*b)).collect();
    let tp: Vec<V2> = base.iter().map(|b| cam(add3(*b, XX))).collect();
    hatch(&mut s, &[bp[0], bp[1], bp[2], bp[3]], tok::RAMP[2], tok::S1);
    for i in 0..4 {
        let j = (i + 1) % 4;
        s.line(tp[i].0, tp[i].1, tp[j].0, tp[j].1, tok::INK2, 1.2);
        s.line(bp[i].0, bp[i].1, tp[i].0, tp[i].1, tok::INK2, 1.0);
    }
    arrow_c(&mut s, cam(zero), cam(XV), tok::S3, 2.5);
    arrow_c(&mut s, cam(zero), cam(XW), tok::S2, 2.5);
    arrow_c(&mut s, cam(zero), cam(XX), tok::S1, 2.5);
    // p's direction (its length, 7, is not drawn), x's shadow on it, and the drop from x to that shadow
    let pend = sc3(2.7, n);
    dash(&mut s, cam(zero), cam(pend), tok::INK, 1.3);
    arrow_c(&mut s, cam(sc3(2.4, n)), cam(pend), tok::INK, 1.4);
    let (sa, sb) = (cam(zero), cam(shadow));
    s.line(sa.0, sa.1, sb.0, sb.1, tok::S1, 5.0);
    dash(&mut s, cam(XX), cam(shadow), tok::INK2, 1.2);
    let lab = |q: V3, dx: f64, dy: f64| -> V2 {
        let c = cam(q);
        (c.0 + dx, c.1 + dy)
    };
    let (lv, lw, lx, lp, lh) = (lab(XV, -8.0, 16.0), lab(XW, -4.0, 18.0), lab(XX, -10.0, -2.0), lab(pend, 8.0, 0.0), lab(shadow, 10.0, 10.0));
    s.text_bold(lv.0, lv.1, "v", 13, tok::S3, Anchor::End);
    s.text_bold(lw.0, lw.1, "w", 13, tok::S2, Anchor::Middle);
    s.text_bold(lx.0, lx.1, "x", 13, tok::S1, Anchor::End);
    s.text_bold(lp.0, lp.1, "p", 13, tok::INK, Anchor::Start);
    tag(&mut s, lh.0, lh.1, "height", 12, tok::S1, Anchor::Start);
    let x = 400.0;
    s.text_bold(x, 44.0, &format!("p = v × w = {}", v3s(p)), 13, tok::INK, Anchor::Start);
    lines(
        &mut s,
        x,
        70.0,
        &[
            (&format!("v = {}  w = {}", v3s(XV), v3s(XW)), tok::INK2),
            (&format!("x = {}", v3s(XX)), tok::S1),
        ],
    );
    lines(
        &mut s,
        x,
        118.0,
        &[
            (&format!("length of p = √{} = {}", f(dot3(p, p)), f(area)), tok::INK),
            ("= the area of the base, the v–w parallelogram", tok::INK2),
            (&format!("height = shadow of x on p = {} ÷ {} ≈ {}", f(vol_dot), f(area), num(height, 2)), tok::S1),
            (&format!("volume = area × height = {} × {} ÷ {} = {}", f(area), f(vol_dot), f(area), f(vol_dot)), tok::INK),
        ],
    );
    s.text(x, 204.0, &format!("check: det[x v w] = {}, and p · x = {}", f(vol_det), f(vol_dot)), 12, tok::INK, Anchor::Start);
    s.text(x, 234.0, "p is perpendicular to v and w:", 11, tok::INK2, Anchor::Start);
    s.text(x, 249.0, &format!("p · v = {}, p · w = {}", f(dot3(p, XV)), f(dot3(p, XW))), 11, tok::INK2, Anchor::Start);
    s.text(x, 274.0, "the thick blue bar on p's line is x's shadow:", 11, tok::INK2, Anchor::Start);
    s.text(x, 289.0, "the box's height", 11, tok::INK2, Anchor::Start);
    ("b04-4-parallelepiped".to_string(), s.finish())
}

fn fig_4_5() -> Figure {
    let p = cross(XV, XW);
    let sub = ["₁", "₂", "₃"];
    let mut s = Svg::new(
        720,
        290,
        "Where the three components of the cross product come from",
        &format!(
            "The determinant with x, y, z in the first column and v, w in the others. Expanding down the first column, each of x, y, z is multiplied by a 2 × 2 determinant of the other two rows; those three numbers, {}, {} and {}, are p = v × w.",
            f(p[0]),
            f(p[1]),
            f(p[2])
        ),
    ).min_text(12);
    let cw = 40.0;
    let rows: Vec<Vec<Cell>> = (0..3).map(|i| vec![cell(["x", "y", "z"][i]).ink(tok::S1), cell(f(XV[i])).ink(tok::S3), cell(f(XW[i])).ink(tok::S2)]).collect();
    s.text_bold(20.0, 30.0, "det of the columns (x, y, z), v, w", 13, tok::INK, Anchor::Start);
    s.cells_wh(40.0, 50.0, cw, 32.0, &rows);
    s.text(40.0 + cw / 2.0, 164.0, "x, y, z", 11, tok::S1, Anchor::Middle);
    s.text(40.0 + 1.5 * cw, 164.0, "v", 11, tok::S3, Anchor::Middle);
    s.text(40.0 + 2.5 * cw, 164.0, "w", 11, tok::S2, Anchor::Middle);
    s.text(20.0, 200.0, "for each letter, use the two", 11, tok::INK2, Anchor::Start);
    s.text(20.0, 216.0, "rows below it, wrapping round", 11, tok::INK2, Anchor::Start);
    s.text(20.0, 232.0, "to the top when needed", 11, tok::INK2, Anchor::Start);
    for (k, letter) in ["x", "y", "z"].iter().enumerate() {
        let (r1, r2) = ((k + 1) % 3, (k + 2) % 3);
        let y = 40.0 + k as f64 * 72.0;
        let minor: Vec<Vec<Cell>> = vec![
            vec![cell(f(XV[r1])).ink(tok::S3), cell(f(XW[r1])).ink(tok::S2)],
            vec![cell(f(XV[r2])).ink(tok::S3), cell(f(XW[r2])).ink(tok::S2)],
        ];
        s.text_bold(270.0, y + 30.0, letter, 13, tok::S1, Anchor::Middle);
        s.text(284.0, y + 30.0, "×", 12, tok::INK, Anchor::Start);
        s.cells_wh(300.0, y + 4.0, 34.0, 26.0, &minor);
        let val = XV[r1] * XW[r2] - XV[r2] * XW[r1];
        s.text(
            384.0,
            y + 22.0,
            &format!("p{} = v{}w{} − v{}w{} = {}×{} − {}×{} = {}", sub[k], sub[r1], sub[r2], sub[r2], sub[r1], fp(XV[r1]), fp(XW[r2]), fp(XV[r2]), fp(XW[r1]), f(val)),
            12,
            tok::INK,
            Anchor::Start,
        );
        s.text(384.0, y + 40.0, &format!("the number that multiplies {letter}"), 11, tok::INK2, Anchor::Start);
    }
    s.text_bold(270.0, 274.0, &format!("so p = {} and det = p · (x, y, z)", v3s(p)), 13, tok::INK, Anchor::Start);
    ("b04-5-components".to_string(), s.finish())
}

// ------------------------------------------------------------------------------------------------------------------
// Chapter 5
// ------------------------------------------------------------------------------------------------------------------
fn fig_5_1() -> Figure {
    let a = qm(HER);
    let ai = qinv(a);
    let q = (Q::int(QV.0), Q::int(QV.1));
    let ours = qmv(a, q);
    let hers = qmv(ai, q);
    let (b1, b2) = (col(HER, 0), col(HER, 1));
    let ours_f = (ours.0.val(), ours.1.val());
    let hers_f = (hers.0.val(), hers.1.val());
    let mut s = Svg::new(
        720,
        300,
        "One arrow, two sets of coordinates",
        &format!(
            "Left: her coordinates {} mean {} of her first basis vector {} plus {} of her second {}, which is {} in our grid. Right: our {} is {} of her first vector plus {} of her second.",
            v2s(QV),
            f(QV.0),
            v2s(b1),
            f(QV.1),
            v2s(b2),
            qvs(ours),
            v2s(QV),
            hers.0.s(),
            hers.1.s()
        ),
    ).min_text(12);
    let l = Frame::square(24.0, 44.0, 36.0, (-5.2, 2.6), (-1.6, 3.2));
    let r = Frame::square(440.0, 44.0, 36.0, (-2.6, 3.0), (-1.6, 3.2));
    for (fr, coeffs, tip) in [(&l, QV, ours_f), (&r, hers_f, QV)] {
        grid(&mut s, fr, false);
        tgrid(&mut s, fr, HER);
        let p1 = sc2(coeffs.0, b1);
        arrow_d(&mut s, fr, (0.0, 0.0), p1, tok::S3, 2.2);
        arrow_d(&mut s, fr, p1, add2(p1, sc2(coeffs.1, b2)), tok::S2, 2.2);
        arrow_d(&mut s, fr, (0.0, 0.0), tip, tok::S1, 2.5);
    }
    s.text_bold(l.cx(), 26.0, &format!("her {} in our grid", v2s(QV)), 13, tok::INK, Anchor::Middle);
    s.text_bold(r.cx(), 26.0, &format!("our {} in her grid", v2s(QV)), 13, tok::INK, Anchor::Middle);
    tag(&mut s, l.px(ours_f.0), l.py(ours_f.1) - 10.0, &format!("ours: {}", qvs(ours)), 12, tok::S1, Anchor::Middle);
    tag(&mut s, r.px(QV.0) - 6.0, r.py(QV.1) - 8.0, &format!("ours: {}", v2s(QV)), 12, tok::S1, Anchor::End);
    let ty = l.bottom() + 24.0;
    s.text(l.left, ty, &format!("her words: {}", v2s(QV)), 12, tok::INK, Anchor::Start);
    s.text(l.left, ty + 17.0, &format!("= {} × {} + {} × {} = {} in ours", f(QV.0), v2s(b1), f(QV.1), v2s(b2), qvs(ours)), 12, tok::INK, Anchor::Start);
    s.text(l.left, ty + 34.0, "multiply by A: her words into ours", 11, tok::INK2, Anchor::Start);
    s.text(r.left - 30.0, ty, &format!("her words: {}", qvs(hers)), 12, tok::INK, Anchor::Start);
    s.text(r.left - 30.0, ty + 17.0, &format!("= {} × {} + {} × {}", hers.0.s(), v2s(b1), hers.1.s(), v2s(b2)), 12, tok::INK, Anchor::Start);
    s.text(r.left - 30.0, ty + 34.0, "multiply by A⁻¹: our words into hers", 11, tok::INK2, Anchor::Start);
    ("b05-1-two-grids".to_string(), s.finish())
}

fn fig_5_2() -> Figure {
    let a = qm(HER);
    let ai = qinv(a);
    let r = qm(ROT);
    let v0 = (Q::int(QV.0), Q::int(QV.1));
    let v1 = qmv(a, v0);
    let v2 = qmv(r, v1);
    let v3 = qmv(ai, v2);
    let whole = qmm(ai, qmm(r, a));
    let check = qmv(whole, v0);
    let mut s = Svg::new(
        720,
        236,
        "A quarter turn described in her words",
        &format!(
            "A pipeline: her vector {} is translated into our words by A, giving {}; our quarter turn R gives {}; A⁻¹ translates back, giving {} in her words. The single matrix A⁻¹RA does the same job.",
            qvs(v0),
            qvs(v1),
            qvs(v2),
            qvs(v3)
        ),
    ).min_text(12);
    let boxes = [("her words", qvs(v0)), ("our words", qvs(v1)), ("our words, turned", qvs(v2)), ("her words, turned", qvs(v3))];
    let arrows = ["A", "R", "A⁻¹"];
    let (bw, gap, y) = (128.0, 64.0, 56.0);
    for (k, (l1, l2)) in boxes.iter().enumerate() {
        let x = 8.0 + k as f64 * (bw + gap);
        let fill = if k == 0 || k == 3 { tok::FILL1 } else { tok::NEUTRAL };
        s.labelled_box2(x, y, bw, 52.0, l2, l1, fill, tok::AXIS);
        if k < 3 {
            s.arrow(x + bw + 6.0, y + 26.0, x + bw + gap - 6.0, y + 26.0, None);
            s.text_bold(x + bw + gap / 2.0, y + 16.0, arrows[k], 13, tok::INK, Anchor::Middle);
        }
    }
    s.text(8.0, 30.0, "to turn her vector: translate, turn, translate back", 12, tok::INK2, Anchor::Start);
    s.text(8.0, 134.0, "A: her words into ours · R: our quarter turn · A⁻¹: our words into hers", 11, tok::INK2, Anchor::Start);
    s.text_bold(8.0, 180.0, "all at once: A⁻¹ R A =", 13, tok::INK, Anchor::Start);
    let rows: Vec<Vec<Cell>> = (0..2).map(|i| (0..2).map(|j| cell(whole[i][j].s()).fill(tok::FILL1)).collect()).collect();
    s.cells_wh(180.0, 162.0, 52.0, 26.0, &rows);
    s.text(300.0, 180.0, &format!("times her {} gives {}: the same answer", qvs(v0), qvs(check)), 12, tok::INK, Anchor::Start);
    s.text(300.0, 198.0, "read right to left: A first, then R, then A⁻¹", 11, tok::INK2, Anchor::Start);
    ("b05-2-pipeline".to_string(), s.finish())
}

// ------------------------------------------------------------------------------------------------------------------
// Chapter 6
// ------------------------------------------------------------------------------------------------------------------
/// M − λI.
fn shift(m: M2, lambda: f64) -> M2 {
    [[m[0][0] - lambda, m[0][1]], [m[1][0], m[1][1] - lambda]]
}

/// The real eigenvalues of a 2 × 2 matrix, smaller first, from λ² − (trace)λ + det = 0; None when there are none.
fn eig2(m: M2) -> Option<V2> {
    let (tr, det) = (m[0][0] + m[1][1], det2(m));
    let disc = tr * tr - 4.0 * det;
    if disc < 0.0 {
        return None;
    }
    let r = disc.sqrt();
    Some(((tr - r) / 2.0, (tr + r) / 2.0))
}

/// A non-zero vector that the singular matrix `m` sends to zero, turned to point up or, on the x-axis, right.
fn null2(m: M2) -> V2 {
    let row = if m[0][0] != 0.0 || m[0][1] != 0.0 { m[0] } else { m[1] };
    let v = (-row[1], row[0]);
    if v.1 < 0.0 || (v.1 == 0.0 && v.0 < 0.0) { sc2(-1.0, v) } else { v }
}

/// "λ² − 5λ + 6" for a matrix with the given trace and determinant.
fn char_label(m: M2) -> String {
    let (tr, det) = (m[0][0] + m[1][1], det2(m));
    let mut s = "λ²".to_string();
    if tr != 0.0 {
        s.push_str(&format!(" {} {}λ", if tr > 0.0 { "−" } else { "+" }, f(tr.abs())));
    }
    if det != 0.0 {
        s.push_str(&format!(" {} {}", if det < 0.0 { "−" } else { "+" }, f(det.abs())));
    }
    s
}

/// Superscript digits for an exponent.
fn sup(n: u32) -> String {
    n.to_string().chars().map(|c| ['⁰', '¹', '²', '³', '⁴', '⁵', '⁶', '⁷', '⁸', '⁹'][c.to_digit(10).unwrap_or(0) as usize]).collect()
}

/// An integer vector with thousands separators and a typographic minus.
fn ivs(v: (i64, i64)) -> String {
    let one = |x: i64| if x < 0 { format!("−{}", thousands(x.unsigned_abs())) } else { thousands(x as u64) };
    format!("({}, {})", one(v.0), one(v.1))
}

/// The eigenvalues of M and an eigenvector for each, larger eigenvalue first.
fn eigenpairs() -> [(f64, V2); 2] {
    let (lo, hi) = eig2(EIG_M).unwrap_or((0.0, 0.0));
    [(hi, null2(shift(EIG_M, hi))), (lo, null2(shift(EIG_M, lo)))]
}

fn fig_6_1() -> Figure {
    let [(l1, e1), (l2, e2)] = eigenpairs();
    let (m1, m2, mp) = (mv(EIG_M, e1), mv(EIG_M, e2), mv(EIG_M, EIG_PROBE));
    let mut s = Svg::new(
        720,
        272,
        "Most vectors are knocked off their line; eigenvectors are not",
        &format!(
            "Left: the plane before the matrix with columns {} and {}, with two dashed lines of eigenvectors and an ordinary vector {}. Right: after it, {} has become {}, {} times as long, {} has become {}, {} times as long, and {} has become {}, off its own line.",
            v2s(col(EIG_M, 0)),
            v2s(col(EIG_M, 1)),
            v2s(EIG_PROBE),
            v2s(e1),
            v2s(m1),
            f(l1),
            v2s(e2),
            v2s(m2),
            f(l2),
            v2s(EIG_PROBE),
            v2s(mp)
        ),
    ).min_text(12);
    let l = Frame::square(26.0, 44.0, 38.0, (-2.6, 4.6), (-1.0, 2.9));
    let r = Frame::square(392.0, 44.0, 38.0, (-2.6, 4.6), (-1.0, 2.9));
    for (k, fr) in [&l, &r].iter().enumerate() {
        grid(&mut s, fr, false);
        if k == 1 {
            tgrid(&mut s, fr, EIG_M);
        }
        dash_d(&mut s, fr, sc2(-9.0, e1), sc2(9.0, e1), tok::S3, 1.5);
        dash_d(&mut s, fr, sc2(-9.0, e2), sc2(9.0, e2), tok::S2, 1.5);
        dash_d(&mut s, fr, sc2(-9.0, EIG_PROBE), sc2(9.0, EIG_PROBE), tok::S1, 1.2);
        let (a, b, c) = if k == 0 { (e1, e2, EIG_PROBE) } else { (m1, m2, mp) };
        arrow_d(&mut s, fr, (0.0, 0.0), a, tok::S3, 2.5);
        arrow_d(&mut s, fr, (0.0, 0.0), b, tok::S2, 2.5);
        arrow_d(&mut s, fr, (0.0, 0.0), c, tok::S1, 2.5);
        tip_label(&mut s, fr, a, &v2s(a), 12, tok::S3);
        tip_label(&mut s, fr, b, &v2s(b), 12, tok::S2);
        tip_label(&mut s, fr, c, &v2s(c), 12, tok::S1);
    }
    s.text_bold(l.cx(), 26.0, "before", 13, tok::INK, Anchor::Middle);
    s.text_bold(r.cx(), 26.0, "after M: the dashed lines stay put", 13, tok::INK, Anchor::Middle);
    s.text(l.left, l.bottom() + 22.0, "dashed green and orange: lines of eigenvectors", 11, tok::INK2, Anchor::Start);
    s.text(l.left, l.bottom() + 37.0, &format!("dashed blue: the line through {}", v2s(EIG_PROBE)), 11, tok::INK2, Anchor::Start);
    s.text(r.left, r.bottom() + 22.0, &format!("{} → {} = {} × {}", v2s(e1), v2s(m1), f(l1), v2s(e1)), 12, tok::S3, Anchor::Start);
    s.text(r.left, r.bottom() + 39.0, &format!("{} → {} = {} × {}", v2s(e2), v2s(m2), f(l2), v2s(e2)), 12, tok::S2, Anchor::Start);
    s.text(r.left, r.bottom() + 56.0, &format!("{} → {}: off its dashed line", v2s(EIG_PROBE), v2s(mp)), 12, tok::S1, Anchor::Start);
    ("b06-1-eigenlines".to_string(), s.finish())
}

fn fig_6_2() -> Figure {
    let (lo, hi) = eig2(EIG_M).unwrap_or((0.0, 0.0));
    let mid = (lo + hi) / 2.0;
    let lams = [EIG_PANEL_ENDS.0, lo, mid, hi, EIG_PANEL_ENDS.1];
    let d = |lam: f64| det2(shift(EIG_M, lam));
    let areas: Vec<String> = lams.iter().map(|x| format!("λ = {}: {}", f(*x), f(d(*x)))).collect();
    let mut s = Svg::new(
        720,
        404,
        "The determinant of M − λI, and the parallelogram behind each value",
        &format!(
            "Top: det(M − λI) = {} plotted for λ from {} to {}; it is zero at λ = {} and λ = {}, the eigenvalues, and lowest, {}, at λ = {}. Bottom: the parallelogram of the columns of M − λI at five values, {}.",
            char_label(EIG_M),
            f(EIG_RANGE.0),
            f(EIG_RANGE.1),
            f(lo),
            f(hi),
            f(d(mid)),
            f(mid),
            areas.join(", ")
        ),
    ).min_text(12);
    let fr = Frame::fit(66.0, 34.0, 400.0, 180.0, EIG_RANGE, (-1.0, 6.5));
    for k in -1..=6 {
        let y = k as f64;
        s.line(fr.left, fr.py(y), fr.right(), fr.py(y), if k == 0 { tok::AXIS } else { tok::GRID }, if k == 0 { 1.4 } else { 1.0 });
        s.text(fr.left - 6.0, fr.py(y) + 4.0, &f(y), 11, tok::MUTED, Anchor::End);
    }
    let (xa, xb) = (EIG_RANGE.0.ceil() as i64, EIG_RANGE.1.floor() as i64);
    for k in xa..=xb {
        let x = k as f64;
        s.line(fr.px(x), fr.top, fr.px(x), fr.bottom(), tok::GRID, 1.0);
        s.text(fr.px(x), fr.bottom() + 14.0, &f(x), 11, tok::MUTED, Anchor::Middle);
    }
    s.text(fr.right(), fr.bottom() + 30.0, "λ", 12, tok::INK2, Anchor::End);
    s.text(fr.left - 40.0, fr.top - 12.0, "det(M − λI)", 11, tok::INK2, Anchor::Start);
    curve(&mut s, &fr, &d, EIG_RANGE, 240, tok::S1, 2.2);
    for x in lams.iter() {
        let zero = d(*x).abs() < 1e-12;
        s.dot(fr.px(*x), fr.py(d(*x)), if zero { 6.0 } else { 4.0 }, if zero { tok::S2 } else { tok::S1 }, None);
    }
    for x in [lo, hi] {
        s.text(fr.px(x), fr.py(0.0) - 12.0, &format!("λ = {}", f(x)), 12, tok::S2, Anchor::Middle);
    }
    tag(&mut s, fr.px(mid), fr.py(d(mid)) + 18.0, &format!("{} at {}", f(d(mid)), f(mid)), 12, tok::INK2, Anchor::Middle);
    let x = 496.0;
    s.text_bold(x, 56.0, &format!("det(M − λI) = {}", char_label(EIG_M)), 12, tok::INK, Anchor::Start);
    s.text(x, 76.0, &format!("= ({} − λ)({} − λ)", f(hi), f(lo)), 12, tok::INK, Anchor::Start);
    s.text(x, 104.0, "zero exactly when M − λI", 11, tok::INK2, Anchor::Start);
    s.text(x, 120.0, "flattens the plane, so that", 11, tok::INK2, Anchor::Start);
    s.text(x, 136.0, "some vector is sent to zero:", 11, tok::INK2, Anchor::Start);
    s.text(x, 152.0, "Mv = λv", 11, tok::INK2, Anchor::Start);
    s.text_bold(x, 182.0, &format!("eigenvalues: {} and {}", f(lo), f(hi)), 12, tok::S2, Anchor::Start);
    for (k, lam) in lams.iter().enumerate() {
        let a = shift(EIG_M, *lam);
        let (c1, c2) = (col(a, 0), col(a, 1));
        let area = det2(a);
        let p = Frame::square(18.0 + k as f64 * 140.0, 268.0, 24.0, (-1.4, 3.4), (-2.4, 1.4));
        s.rect(p.left, p.top, p.right() - p.left, p.bottom() - p.top, tok::SURFACE, Some(tok::GRID));
        seg(&mut s, &p, (p.x0, 0.0), (p.x1, 0.0), tok::AXIS, 1.0);
        seg(&mut s, &p, (0.0, p.y0), (0.0, p.y1), tok::AXIS, 1.0);
        if area.abs() > 1e-12 {
            hatch_d(&mut s, &p, &[(0.0, 0.0), c1, add2(c1, c2), c2], tok::RAMP[2], tok::S1);
        } else {
            let far = add2(c1, c2);
            seg(&mut s, &p, (0.0, 0.0), far, tok::S1, 4.0);
        }
        for (c, colour) in [(c1, tok::S3), (c2, tok::S2)] {
            if dot2(c, c) > 1e-12 {
                arrow_d(&mut s, &p, (0.0, 0.0), c, colour, 2.0);
            } else {
                s.dot(p.px(0.0), p.py(0.0), 4.0, colour, None);
            }
        }
        let zero = area.abs() < 1e-12;
        s.text_bold(p.cx(), p.top - 8.0, &format!("λ = {}", f(*lam)), 12, if zero { tok::S2 } else { tok::INK }, Anchor::Middle);
        let words = if zero { "flattened" } else if area < 0.0 { "flipped" } else { "" };
        s.text(p.cx(), p.bottom() + 16.0, &format!("area {}", f(area)), 12, tok::INK, Anchor::Middle);
        if !words.is_empty() {
            s.text(p.cx(), p.bottom() + 31.0, words, 11, tok::INK2, Anchor::Middle);
        }
    }
    ("b06-2-det-curve".to_string(), s.finish())
}

fn fig_6_3() -> Figure {
    let d = |lam: f64| det2(shift(ROT, lam));
    let (low_at, lowest) = (0..=400)
        .map(|i| -2.0 + 0.01 * i as f64)
        .map(|x| (x, d(x)))
        .fold((0.0, f64::INFINITY), |best, p| if p.1 < best.1 { p } else { best });
    let mut s = Svg::new(
        720,
        300,
        "A quarter turn has no real eigenvector",
        &format!(
            "Left: three vectors and their images under the quarter turn R; each image stands at a right angle to its vector, off the vector's dashed line. Right: det(R − λI) = {} is never smaller than {}, so it is never zero and R has no real eigenvalue.",
            char_label(ROT),
            f(lowest)
        ),
    ).min_text(12);
    let fr = Frame::square(26.0, 44.0, 42.0, (-2.6, 2.6), (-1.8, 2.6));
    grid(&mut s, &fr, false);
    let colours = [tok::S1, tok::S2, tok::S3];
    let x = 268.0;
    s.text_bold(x, 56.0, "each vector turns off its own line", 12, tok::INK, Anchor::Start);
    for (k, v) in ROT_PROBES.iter().enumerate() {
        let img = mv(ROT, *v);
        dash_d(&mut s, &fr, sc2(-9.0, *v), sc2(9.0, *v), colours[k], 1.0);
        arrow_d(&mut s, &fr, (0.0, 0.0), *v, colours[k], 1.4);
        arrow_d(&mut s, &fr, (0.0, 0.0), img, colours[k], 2.8);
        s.text(x, 84.0 + 20.0 * k as f64, &format!("{} → {}", v2s(*v), v2s(img)), 12, colours[k], Anchor::Start);
        s.text(x + 138.0, 84.0 + 20.0 * k as f64, &format!("v · Rv = {}", f(dot2(*v, img))), 11, tok::INK2, Anchor::Start);
    }
    s.text(x, 160.0, "thin arrow: the vector; thick: its image;", 11, tok::INK2, Anchor::Start);
    s.text(x, 175.0, "dashed: the vector's own line", 11, tok::INK2, Anchor::Start);
    s.text_bold(fr.cx(), 26.0, "the quarter turn R", 13, tok::INK, Anchor::Middle);
    let p = Frame::fit(518.0, 60.0, 176.0, 150.0, (-2.0, 2.0), (-0.5, 5.0));
    for k in 0..=5 {
        let y = k as f64;
        s.line(p.left, p.py(y), p.right(), p.py(y), if k == 0 { tok::AXIS } else { tok::GRID }, if k == 0 { 1.4 } else { 1.0 });
        s.text(p.left - 6.0, p.py(y) + 4.0, &f(y), 11, tok::MUTED, Anchor::End);
    }
    for k in -2..=2 {
        s.text(p.px(k as f64), p.bottom() + 14.0, &f(k as f64), 11, tok::MUTED, Anchor::Middle);
    }
    s.line(p.px(0.0), p.top, p.px(0.0), p.bottom(), tok::GRID, 1.0);
    curve(&mut s, &p, &d, (-2.0, 2.0), 160, tok::S1, 2.2);
    s.dot(p.px(low_at), p.py(lowest), 4.0, tok::S1, None);
    s.text_bold(p.cx(), 26.0, &format!("det(R − λI) = {}", char_label(ROT)), 12, tok::INK, Anchor::Middle);
    s.text(p.cx(), 44.0, &format!("lowest value {}, at λ = {}", f(lowest), f(low_at)), 11, tok::INK2, Anchor::Middle);
    s.text(p.cx(), p.bottom() + 32.0, "λ", 12, tok::INK2, Anchor::Middle);
    let verdict = if eig2(ROT).is_none() { "never zero: no real eigenvalue" } else { "it reaches zero: real eigenvalues" };
    s.text(p.cx(), p.bottom() + 52.0, verdict, 12, tok::S2, Anchor::Middle);
    s.text(fr.left, fr.bottom() + 24.0, &format!("R has columns {} and {}", v2s(col(ROT, 0)), v2s(col(ROT, 1))), 12, tok::INK, Anchor::Start);
    ("b06-3-rotation".to_string(), s.finish())
}

fn fig_6_4() -> Figure {
    let [(l1, e1), (l2, e2)] = eigenpairs();
    let p = cols(e1, e2);
    let (pq, mq) = (qm(p), qm(EIG_M));
    let pi = qinv(pq);
    let dq = qmm(pi, qmm(mq, pq));
    let jh = (Q::int(0.0), Q::int(1.0));
    let jc = qmv(pi, jh);
    let mjc = qmv(dq, jc);
    let mj = mv(EIG_M, (0.0, 1.0));
    // the same move repeated: in the eigenbasis each coordinate is multiplied by its eigenvalue each time
    let (a, b) = (jc.0.val().round() as i64, jc.1.val().round() as i64);
    let (k1, k2) = (dq[0][0].val().round() as i64, dq[1][1].val().round() as i64);
    let eig_after = (a * k1.pow(EIG_POWER), b * k2.pow(EIG_POWER));
    let pi_int = |e: V2| (e.0.round() as i64, e.1.round() as i64);
    let (e1i, e2i) = (pi_int(e1), pi_int(e2));
    let ours = (eig_after.0 * e1i.0 + eig_after.1 * e2i.0, eig_after.0 * e1i.1 + eig_after.1 * e2i.1);
    // and directly, multiplying by M again and again
    let mut direct = (0_i64, 1_i64);
    let mi = [[EIG_M[0][0] as i64, EIG_M[0][1] as i64], [EIG_M[1][0] as i64, EIG_M[1][1] as i64]];
    for _ in 0..EIG_POWER {
        direct = (mi[0][0] * direct.0 + mi[0][1] * direct.1, mi[1][0] * direct.0 + mi[1][1] * direct.1);
    }
    let mut s = Svg::new(
        720,
        268,
        "In the eigenbasis the matrix only stretches",
        &format!(
            "Left: the grid of the eigenvectors {} and {}. The vector ĵ is {} step of the first and {} of the second, and M sends it to {}, which is {} steps of the first and {} of the second. Right: P⁻¹MP has {} and {} on its diagonal and zeros elsewhere, and applying M {} times multiplies the eigen-coordinates by {}{} and {}{}.",
            v2s(e1),
            v2s(e2),
            jc.0.s(),
            jc.1.s(),
            v2s(mj),
            mjc.0.s(),
            mjc.1.s(),
            f(l1),
            f(l2),
            EIG_POWER,
            f(l1),
            sup(EIG_POWER),
            f(l2),
            sup(EIG_POWER)
        ),
    ).min_text(12);
    let fr = Frame::square(26.0, 44.0, 44.0, (-1.6, 3.6), (-0.6, 2.6));
    grid(&mut s, &fr, false);
    tgrid(&mut s, &fr, p);
    // M ĵ as steps along the eigenvectors
    let n1 = mjc.0.val().round() as i64;
    let n2 = mjc.1.val().round() as i64;
    for i in 0..n1 {
        let a0 = sc2(i as f64, e1);
        arrow_d(&mut s, &fr, a0, add2(a0, e1), tok::S3, 2.2);
    }
    let corner = sc2(n1 as f64, e1);
    for i in 0..n2 {
        let a0 = add2(corner, sc2(i as f64, e2));
        arrow_d(&mut s, &fr, a0, add2(a0, e2), tok::S2, 2.2);
    }
    arrow_d(&mut s, &fr, (0.0, 0.0), (0.0, 1.0), tok::INK, 1.6);
    arrow_d(&mut s, &fr, (0.0, 0.0), mj, tok::S1, 2.5);
    tip_label(&mut s, &fr, mj, &format!("M ĵ = {}", v2s(mj)), 12, tok::S1);
    s.text(fr.px(0.0) - 6.0, fr.py(1.0) + 4.0, "ĵ", 13, tok::INK, Anchor::End);
    s.text_bold(fr.cx(), 26.0, "the grid of the eigenvectors", 13, tok::INK, Anchor::Middle);
    let ty = fr.bottom() + 22.0;
    s.text(fr.left, ty, &format!("eigen-coordinates of ĵ: {}", qvs(jc)), 12, tok::INK, Anchor::Start);
    s.text(fr.left, ty + 17.0, &format!("of M ĵ: {} = ({} × {}, {} × {})", qvs(mjc), f(l1), jc.0.s(), f(l2), jc.1.s()), 12, tok::INK, Anchor::Start);
    s.text(fr.left, ty + 34.0, &format!("{} green steps of {}, {} orange of {}", n1, v2s(e1), n2, v2s(e2)), 11, tok::INK2, Anchor::Start);
    // the sandwich P⁻¹ M P = D
    let x = 330.0;
    s.text_bold(x, 56.0, "P has the eigenvectors as columns:", 12, tok::INK, Anchor::Start);
    let qcells = |m: QM, fill: &'static str| -> Vec<Vec<Cell>> { (0..2).map(|i| (0..2).map(|j| cell(m[i][j].s()).fill(fill)).collect()).collect() };
    let y0 = 70.0;
    s.text(x, y0 + 30.0, "P⁻¹", 12, tok::INK, Anchor::Start);
    s.cells_wh(x + 26.0, y0, 34.0, 26.0, &qcells(pi, tok::SURFACE));
    s.text(x + 104.0, y0 + 30.0, "M", 12, tok::INK, Anchor::Start);
    s.cells_wh(x + 120.0, y0, 34.0, 26.0, &qcells(mq, tok::SURFACE));
    s.text(x + 196.0, y0 + 30.0, "P", 12, tok::INK, Anchor::Start);
    s.cells_wh(x + 208.0, y0, 34.0, 26.0, &qcells(pq, tok::SURFACE));
    s.text(x + 286.0, y0 + 30.0, "=", 13, tok::INK, Anchor::Start);
    s.cells_wh(x + 302.0, y0, 34.0, 26.0, &qcells(dq, tok::FILL1));
    s.text(x, 150.0, "D: the eigenvalues on the diagonal, zeros elsewhere", 11, tok::INK2, Anchor::Start);
    s.text_bold(x, 186.0, &format!("M applied {} times to ĵ", EIG_POWER), 12, tok::INK, Anchor::Start);
    lines(
        &mut s,
        x,
        206.0,
        &[
            (&format!("eigen-coordinates: ({} × {}{}, {} × {}{}) = {}", a, f(l1), sup(EIG_POWER), b, f(l2), sup(EIG_POWER), ivs(eig_after)), tok::INK),
            (&format!("in our words: {}", ivs(ours)), tok::S1),
            (&format!("checked by multiplying by M {} times: {}", EIG_POWER, ivs(direct)), tok::INK2),
        ],
    );
    ("b06-4-eigenbasis".to_string(), s.finish())
}

// ------------------------------------------------------------------------------------------------------------------
// Chapter 7
// ------------------------------------------------------------------------------------------------------------------
fn fig_7_1() -> Figure {
    let zs: Vec<f64> = N_BOOKS.iter().map(|x| dot2(N_W, *x) + N_B).collect();
    let x = N_BOOKS[0];
    let mut s = Svg::new(
        720,
        330,
        "One neuron: weigh, add, bend",
        &format!(
            "A neuron with two inputs, a book's pages in hundreds and its age in decades, weights {} and {} and bias {}. For the first book the weighted sum is {} and ReLU passes it; for the second the sum is {} and ReLU outputs {}. The weight {} stays negative.",
            f(N_W.0),
            f(N_W.1),
            f(N_B),
            f(zs[0]),
            f(zs[1]),
            f(relu(zs[1])),
            f(N_W.1)
        ),
    ).min_text(12);
    let (ix, iy1, iy2) = (70.0, 70.0, 170.0);
    s.dot(ix, iy1, 26.0, tok::FILL1, Some(tok::S1));
    s.dot(ix, iy2, 26.0, tok::FILL1, Some(tok::S1));
    s.text(ix, iy1 + 4.0, &format!("x₁ = {}", f(x.0)), 12, tok::INK, Anchor::Middle);
    s.text(ix, iy2 + 4.0, &format!("x₂ = {}", f(x.1)), 12, tok::INK, Anchor::Middle);
    s.text(ix, iy1 - 34.0, "pages (hundreds)", 11, tok::INK2, Anchor::Middle);
    s.text(ix, iy2 + 42.0, "age (decades)", 11, tok::INK2, Anchor::Middle);
    let (nx, ny) = (300.0, 120.0);
    s.dot(nx, ny, 48.0, tok::NEUTRAL, Some(tok::INK2));
    s.text(nx, ny - 4.0, "weighted sum", 11, tok::INK2, Anchor::Middle);
    s.text(nx, ny + 12.0, &format!("z = {}", f(zs[0])), 12, tok::INK, Anchor::Middle);
    s.arrow(ix + 28.0, iy1 + 6.0, nx - 49.0, ny - 12.0, None);
    s.arrow(ix + 28.0, iy2 - 6.0, nx - 49.0, ny + 12.0, None);
    s.text(170.0, 78.0, &format!("w₁ = {}", f(N_W.0)), 12, tok::S3, Anchor::Middle);
    s.text(170.0, 176.0, &format!("w₂ = {}", f(N_W.1)), 12, tok::S2, Anchor::Middle);
    s.arrow(nx, 30.0, nx, ny - 50.0, None);
    s.text(nx + 8.0, 40.0, &format!("bias b = {}", f(N_B)), 12, tok::INK, Anchor::Start);
    s.labelled_box2(410.0, 96.0, 110.0, 48.0, "ReLU", "negative → 0", tok::FILL1, tok::S1);
    s.arrow(nx + 50.0, ny, 408.0, ny, None);
    s.dot(620.0, ny, 34.0, tok::FILL1, Some(tok::S1));
    s.text(620.0, ny - 2.0, "output", 11, tok::INK2, Anchor::Middle);
    s.text(620.0, ny + 14.0, &format!("a = {}", f(relu(zs[0]))), 12, tok::INK, Anchor::Middle);
    s.arrow(522.0, ny, 584.0, ny, None);
    let head: Vec<Cell> = ["book", "x₁", "x₂", "z = w₁x₁ + w₂x₂ + b", "ReLU(z)"].iter().map(|h| cell(*h).fill(tok::NEUTRAL)).collect();
    let mut rows = vec![head];
    for (k, b) in N_BOOKS.iter().enumerate() {
        let z = zs[k];
        rows.push(vec![
            cell(["first", "second"][k]),
            cell(f(b.0)),
            cell(f(b.1)),
            cell(format!("{}×{} + {}×{} + {} = {}", f(N_W.0), f(b.0), fp(N_W.1), f(b.1), f(N_B), f(z))),
            cell(f(relu(z))).fill(if z < 0.0 { tok::FILL2 } else { tok::SURFACE }),
        ]);
    }
    table(&mut s, 130.0, 236.0, &[70.0, 50.0, 50.0, 260.0, 90.0], 26.0, &rows);
    ("b07-1-neuron".to_string(), s.finish())
}

fn fig_7_2() -> Figure {
    let wp = (L2_W.0 * L1_W[0][0] + L2_W.1 * L1_W[1][0], L2_W.0 * L1_W[0][1] + L2_W.1 * L1_W[1][1]);
    let bp = dot2(L2_W, L1_B) + L2_B;
    let h = add2(mv(L1_W, L_X), L1_B);
    let out2 = dot2(L2_W, h) + L2_B;
    let out1 = dot2(wp, L_X) + bp;
    let hr = (relu(h.0), relu(h.1));
    let out_relu = dot2(L2_W, hr) + L2_B;
    let mut s = Svg::new(
        720,
        330,
        "Two straight layers are one straight layer",
        &format!(
            "Left: two layers with no bend between them. Right: the single layer they add up to, with weights {} and {} and bias {}. For the input {} both give {}; with a ReLU between the layers the answer becomes {}.",
            f(wp.0),
            f(wp.1),
            f(bp),
            v2s(L_X),
            f(out1),
            f(out_relu)
        ),
    ).min_text(12);
    let node = |s: &mut Svg, x: f64, y: f64, t: &str| {
        s.dot(x, y, 18.0, tok::FILL1, Some(tok::S1));
        s.text(x, y + 4.0, t, 12, tok::INK, Anchor::Middle);
    };
    let (xin, xh, xo) = (50.0, 170.0, 290.0);
    let (ya, yb, yo) = (80.0, 170.0, 125.0);
    for (yi, t) in [(ya, "x₁"), (yb, "x₂")] {
        for yh in [ya, yb] {
            s.line(xin + 18.0, yi, xh - 18.0, yh, tok::AXIS, 1.2);
        }
        node(&mut s, xin, yi, t);
    }
    for (yh, t) in [(ya, "h₁"), (yb, "h₂")] {
        s.line(xh + 18.0, yh, xo - 18.0, yo, tok::AXIS, 1.2);
        node(&mut s, xh, yh, t);
    }
    node(&mut s, xo, yo, "out");
    s.text_bold(170.0, 26.0, "two layers, no bend", 13, tok::INK, Anchor::Middle);
    let xs = [(0usize, "x₁"), (1usize, "x₂")];
    let h1 = lin(&xs.map(|(j, n)| (L1_W[0][j], n)), L1_B.0);
    let h2 = lin(&xs.map(|(j, n)| (L1_W[1][j], n)), L1_B.1);
    s.text(20.0, 214.0, &format!("layer 1: h₁ = {h1}"), 12, tok::INK, Anchor::Start);
    s.text(20.0, 231.0, &format!("layer 1: h₂ = {h2}"), 12, tok::INK, Anchor::Start);
    s.text(20.0, 248.0, &format!("layer 2: out = {}", lin(&[(L2_W.0, "h₁"), (L2_W.1, "h₂")], L2_B)), 12, tok::INK, Anchor::Start);
    s.text_bold(360.0, 130.0, "=", 20, tok::INK, Anchor::Middle);
    let (x2, y2) = (430.0, 125.0);
    for (yi, t) in [(ya, "x₁"), (yb, "x₂")] {
        s.line(x2 + 18.0, yi, 600.0 - 18.0, y2, tok::AXIS, 1.2);
        node(&mut s, x2, yi, t);
    }
    node(&mut s, 600.0, y2, "out");
    s.text(505.0, 90.0, &f(wp.0), 12, tok::S3, Anchor::Middle);
    s.text(505.0, 170.0, &f(wp.1), 12, tok::S2, Anchor::Middle);
    s.text_bold(520.0, 26.0, "one layer", 13, tok::INK, Anchor::Middle);
    s.text(400.0, 214.0, &format!("out = {}", lin(&[(wp.0, "x₁"), (wp.1, "x₂")], bp)), 12, tok::INK, Anchor::Start);
    s.text(400.0, 231.0, "weights: layer 2's weights times layer 1's;", 11, tok::INK2, Anchor::Start);
    s.text(400.0, 247.0, "bias: layer 2 applied to layer 1's bias,", 11, tok::INK2, Anchor::Start);
    s.text(400.0, 263.0, "plus its own", 11, tok::INK2, Anchor::Start);
    s.line(20.0, 274.0, 700.0, 274.0, tok::GRID, 1.0);
    s.text(20.0, 294.0, &format!("check with x = {}: two layers give h = {}, out = {}; one layer gives {}", v2s(L_X), v2s(h), f(out2), f(out1)), 12, tok::INK, Anchor::Start);
    s.text(20.0, 313.0, &format!("with a ReLU between: h = {}, out = {}, and the single straight layer no longer matches", v2s(hr), f(out_relu)), 12, tok::S2, Anchor::Start);
    ("b07-2-collapse".to_string(), s.finish())
}

fn fig_7_3() -> Figure {
    let names = ["ReLU", "sigmoid", "tanh", "linear"];
    let gs: [fn(f64) -> f64; 4] = [relu, sigmoid, f64::tanh, |z| z];
    let mut s = Svg::new(
        720,
        262,
        "Four activation functions",
        &format!(
            "Four small plots over inputs from {} to {}: ReLU is 0 for negatives and the input otherwise; sigmoid squeezes into 0 to 1; tanh squeezes into −1 to 1; the linear function returns its input. At z = {} they give {}, {}, {} and {}.",
            f(-Z_EDGE),
            f(Z_EDGE),
            f(Z_PROBE),
            numt(gs[0](Z_PROBE), 2),
            numt(gs[1](Z_PROBE), 2),
            numt(gs[2](Z_PROBE), 2),
            numt(gs[3](Z_PROBE), 2)
        ),
    ).min_text(12);
    let specs: [(&str, (f64, f64), &str); 4] = [
        ("max(0, z)", (-1.0, 4.0), "0 up to any size"),
        ("an S from 0 to 1", (-0.25, 1.25), "between 0 and 1"),
        ("an S from −1 to 1", (-1.25, 1.25), "between −1 and 1"),
        ("z itself", (-Z_EDGE, Z_EDGE), "any number: no bend"),
    ];
    for (k, (formula, yr, range)) in specs.iter().enumerate() {
        let fr = Frame::fit(26.0 + k as f64 * 176.0, 50.0, 150.0, 130.0, (-Z_EDGE, Z_EDGE), *yr);
        s.rect(fr.left, fr.top, 150.0, 130.0, tok::SURFACE, Some(tok::GRID));
        seg(&mut s, &fr, (-Z_EDGE, 0.0), (Z_EDGE, 0.0), tok::AXIS, 1.0);
        seg(&mut s, &fr, (0.0, yr.0), (0.0, yr.1), tok::AXIS, 1.0);
        let g = gs[k];
        curve(&mut s, &fr, &g, (-Z_EDGE, Z_EDGE), 160, tok::S1, 2.2);
        s.text_bold(fr.left, 30.0, names[k], 13, tok::INK, Anchor::Start);
        s.text(fr.left, 44.0, formula, 11, tok::INK2, Anchor::Start);
        s.text(fr.left, fr.bottom() + 13.0, &f(-Z_EDGE), 11, tok::MUTED, Anchor::Start);
        s.text(fr.right(), fr.bottom() + 13.0, &f(Z_EDGE), 11, tok::MUTED, Anchor::End);
        s.text(fr.left, 214.0, range, 12, tok::INK, Anchor::Start);
        s.text(fr.left, 232.0, &format!("at z = {}: {}", f(-Z_PROBE), numt(g(-Z_PROBE), 2)), 11, tok::INK2, Anchor::Start);
        s.text(fr.left, 247.0, &format!("at z = {}: {}", f(Z_PROBE), numt(g(Z_PROBE), 2)), 11, tok::INK2, Anchor::Start);
    }
    ("b07-3-activations".to_string(), s.finish())
}

fn fig_7_4() -> Figure {
    let ys: Vec<f64> = KNOTS.iter().map(|x| true_price(*x)).collect();
    let slopes: Vec<f64> = (0..3).map(|i| (ys[i + 1] - ys[i]) / (KNOTS[i + 1] - KNOTS[i])).collect();
    let bent = |x: f64| -> f64 {
        slopes[0] * relu(x - KNOTS[0]) + (slopes[1] - slopes[0]) * relu(x - KNOTS[1]) + (slopes[2] - slopes[1]) * relu(x - KNOTS[2])
    };
    let mut s = Svg::new(
        720,
        320,
        "Three bends are enough to follow a curve",
        &format!(
            "The price curve against pages (dashed) and a line built from three ReLU pieces that switch on at {}, {} and {} pages (orange). The bent line meets the curve at {}, {}, {} and {} pages.",
            f(KNOTS[0]),
            f(KNOTS[1]),
            f(KNOTS[2]),
            f(KNOTS[0]),
            f(KNOTS[1]),
            f(KNOTS[2]),
            f(KNOTS[3])
        ),
    ).min_text(12);
    let fr = Frame::fit(60.0, 30.0, 340.0, 230.0, (0.0, 420.0), (0.0, 12.0));
    for k in 0..=4 {
        let y = 3.0 * k as f64;
        s.line(fr.left, fr.py(y), fr.right(), fr.py(y), tok::GRID, 1.0);
        s.text(fr.left - 6.0, fr.py(y) + 4.0, &f(y), 11, tok::MUTED, Anchor::End);
    }
    for k in 0..=4 {
        let x = 100.0 * k as f64;
        s.text(fr.px(x), fr.bottom() + 16.0, &f(x), 11, tok::MUTED, Anchor::Middle);
    }
    s.line(fr.left, fr.bottom(), fr.right(), fr.bottom(), tok::AXIS, 1.0);
    s.line(fr.left, fr.top, fr.left, fr.bottom(), tok::AXIS, 1.0);
    dashed_curve(&mut s, &fr, &true_price, (0.0, 420.0), 84, tok::INK2, 1.5);
    curve(&mut s, &fr, &bent, (KNOTS[0], KNOTS[3]), 400, tok::S2, 2.5);
    for (k, x) in KNOTS.iter().enumerate() {
        s.dot(fr.px(*x), fr.py(ys[k]), 4.0, tok::S2, None);
        if k < 3 {
            s.line(fr.px(*x), fr.bottom(), fr.px(*x), fr.bottom() - 6.0, tok::S2, 2.0);
        }
    }
    s.text(fr.cx(), fr.bottom() + 34.0, "pages", 11, tok::INK2, Anchor::Middle);
    s.text(fr.left - 40.0, fr.top - 10.0, "price (hundreds of rupees)", 11, tok::INK2, Anchor::Start);
    let x = 430.0;
    s.text_bold(x, 50.0, "a sum of three ReLUs", 13, tok::INK, Anchor::Start);
    lines(
        &mut s,
        x,
        76.0,
        &[
            (&format!("from {} pages: slope {} per 100 pages", f(KNOTS[0]), num(100.0 * slopes[0], 2)), tok::INK),
            (&format!("bend at {}: slope becomes {}", f(KNOTS[1]), num(100.0 * slopes[1], 2)), tok::INK),
            (&format!("bend at {}: slope becomes {}", f(KNOTS[2]), num(100.0 * slopes[2], 2)), tok::INK),
        ],
    );
    s.text(x, 140.0, "each bend is one hidden ReLU unit that", 11, tok::INK2, Anchor::Start);
    s.text(x, 155.0, "switches on at its page count", 11, tok::INK2, Anchor::Start);
    s.text(x, 185.0, "dashed: the true price curve", 11, tok::INK2, Anchor::Start);
    s.text(x, 200.0, "orange: the bent line, equal to the curve", 11, tok::S2, Anchor::Start);
    s.text(x, 215.0, &format!("at {}, {}, {} and {} pages", f(KNOTS[0]), f(KNOTS[1]), f(KNOTS[2]), f(KNOTS[3])), 11, tok::S2, Anchor::Start);
    s.text(x, 245.0, "without the bends, any stack of layers", 11, tok::INK2, Anchor::Start);
    s.text(x, 260.0, "could only draw one straight line", 11, tok::INK2, Anchor::Start);
    ("b07-4-bends".to_string(), s.finish())
}

// ------------------------------------------------------------------------------------------------------------------
// Chapter 8
// ------------------------------------------------------------------------------------------------------------------
/// The graph's values and, running backwards, each node's gradient (how much J moves per unit nudge of it).
struct Graph {
    u: f64,
    v: f64,
    j: f64,
    dj_dv: f64,
    dv_du: f64,
    dv_da: f64,
    du_db: f64,
    du_dc: f64,
    ga: f64,
    gb: f64,
    gc: f64,
    gu: f64,
    gv: f64,
}

fn graph(a: f64, b: f64, c: f64) -> Graph {
    let u = b * c;
    let v = a + u;
    let j = G_K * v;
    // local slopes of each step
    let (dj_dv, dv_du, dv_da, du_db, du_dc) = (G_K, 1.0, 1.0, c, b);
    // the chain rule, multiplying backwards from J (whose gradient with respect to itself is 1)
    let gv = 1.0 * dj_dv;
    let gu = gv * dv_du;
    Graph { u, v, j, dj_dv, dv_du, dv_da, du_db, du_dc, ga: gv * dv_da, gb: gu * du_db, gc: gu * du_dc, gu, gv }
}

/// The node boxes of the computation graph at fixed places.
fn graph_boxes(s: &mut Svg, g: &Graph, grads: bool) {
    let bx = |s: &mut Svg, x: f64, y: f64, name: &str, val: f64, grad: f64| {
        let h = if grads { 56.0 } else { 40.0 };
        s.rect(x, y, 116.0, h, tok::NEUTRAL, Some(tok::AXIS));
        s.text_bold(x + 58.0, y + 17.0, name, 12, tok::INK, Anchor::Middle);
        s.text(x + 58.0, y + 33.0, &format!("value {}", f(val)), 12, tok::S1, Anchor::Middle);
        if grads {
            s.text(x + 58.0, y + 49.0, &format!("grad {}", f(grad)), 12, tok::S2, Anchor::Middle);
        }
    };
    bx(s, 16.0, 40.0, "a", G_A, g.ga);
    bx(s, 16.0, 120.0, "b", G_B, g.gb);
    bx(s, 16.0, 200.0, "c", G_C, g.gc);
    bx(s, 212.0, 160.0, "u = b × c", g.u, g.gu);
    bx(s, 400.0, 100.0, "v = a + u", g.v, g.gv);
    bx(s, 588.0, 100.0, &format!("J = {} × v", f(G_K)), g.j, 1.0);
}

fn fig_8_1() -> Figure {
    let g = graph(G_A, G_B, G_C);
    let mut s = Svg::new(
        720,
        280,
        "A computation graph, run forwards",
        &format!(
            "Boxes for a = {}, b = {} and c = {} feed u = b × c = {}, then v = a + u = {}, then J = {} × v = {}. Arrows point left to right, the direction the values flow.",
            f(G_A),
            f(G_B),
            f(G_C),
            f(g.u),
            f(g.v),
            f(G_K),
            f(g.j)
        ),
    ).min_text(12);
    graph_boxes(&mut s, &g, false);
    s.arrow(134.0, 140.0, 210.0, 172.0, None);
    s.arrow(134.0, 220.0, 210.0, 188.0, None);
    s.arrow(134.0, 60.0, 398.0, 112.0, None);
    s.arrow(330.0, 176.0, 398.0, 132.0, None);
    s.arrow(518.0, 120.0, 586.0, 120.0, None);
    s.text(360.0, 258.0, "forward: each box computes its value from the boxes that feed it", 12, tok::INK2, Anchor::Middle);
    ("b08-1-forward".to_string(), s.finish())
}

fn fig_8_2() -> Figure {
    let g = graph(G_A, G_B, G_C);
    let mut s = Svg::new(
        720,
        300,
        "The same graph, run backwards",
        &format!(
            "Each box now also holds its gradient, how much J moves per unit nudge of that box: J 1, v {}, u {}, a {}, b {}, c {}. Arrows point right to left and carry the local slopes that are multiplied along the way.",
            f(g.gv),
            f(g.gu),
            f(g.ga),
            f(g.gb),
            f(g.gc)
        ),
    ).min_text(12);
    graph_boxes(&mut s, &g, true);
    s.arrow(586.0, 128.0, 518.0, 128.0, Some(&format!("× {}", f(g.dj_dv))));
    s.arrow(398.0, 140.0, 330.0, 180.0, None);
    s.text(372.0, 176.0, &format!("× {}", f(g.dv_du)), 11, tok::INK2, Anchor::Start);
    s.arrow(398.0, 112.0, 134.0, 64.0, None);
    s.text(270.0, 80.0, &format!("× {}", f(g.dv_da)), 11, tok::INK2, Anchor::Middle);
    s.arrow(210.0, 176.0, 134.0, 148.0, None);
    s.text(168.0, 150.0, &format!("× c = {}", f(g.du_db)), 11, tok::INK2, Anchor::Middle);
    s.arrow(210.0, 200.0, 134.0, 228.0, None);
    s.text(172.0, 238.0, &format!("× b = {}", f(g.du_dc)), 11, tok::INK2, Anchor::Middle);
    s.text(
        360.0,
        282.0,
        &format!("grad of b = {} × {} × {} = {}: the chain rule multiplies the slopes on the path back from J", f(g.dj_dv), f(g.dv_du), f(g.du_db), f(g.gb)),
        12,
        tok::INK,
        Anchor::Middle,
    );
    ("b08-2-backward".to_string(), s.finish())
}

fn fig_8_3() -> Figure {
    let base = graph(G_A, G_B, G_C);
    let cases = [
        ("a", graph(G_A + NUDGE, G_B, G_C), base.ga, G_A),
        ("b", graph(G_A, G_B + NUDGE, G_C), base.gb, G_B),
        ("c", graph(G_A, G_B, G_C + NUDGE), base.gc, G_C),
    ];
    let mut s = Svg::new(
        720,
        230,
        "Nudge an input and watch J",
        &format!(
            "A table: nudging a, b or c by {} moves J from {} to {}, {} or {}; the change divided by the nudge is {}, {} and {}, the gradients the backward pass found.",
            num(NUDGE, 3),
            f(base.j),
            num(cases[0].1.j, 3),
            num(cases[1].1.j, 3),
            num(cases[2].1.j, 3),
            numt((cases[0].1.j - base.j) / NUDGE, 2),
            numt((cases[1].1.j - base.j) / NUDGE, 2),
            numt((cases[2].1.j - base.j) / NUDGE, 2)
        ),
    ).min_text(12);
    let head = [
        "nudge".to_string(),
        "u = b × c".to_string(),
        "v = a + u".to_string(),
        format!("J = {} × v", f(G_K)),
        "change in J".to_string(),
        format!("change ÷ {}", num(NUDGE, 3)),
        "backward pass".to_string(),
    ];
    let mut rows: Vec<Vec<Cell>> = vec![head.iter().map(|h| cell(h.as_str()).fill(tok::NEUTRAL)).collect()];
    for (name, g, grad, v0) in cases.iter() {
        let dj = g.j - base.j;
        rows.push(vec![
            cell(format!("{name}: {} → {}", f(*v0), num(v0 + NUDGE, 3))),
            cell(num(g.u, 3)),
            cell(num(g.v, 3)),
            cell(num(g.j, 3)),
            cell(num(dj, 3)),
            cell(num(dj / NUDGE, 2)).fill(tok::FILL1),
            cell(f(*grad)).ink(tok::S2),
        ]);
    }
    table(&mut s, 16.0, 30.0, &[110.0, 90.0, 90.0, 90.0, 100.0, 110.0, 100.0], 30.0, &rows);
    s.text(16.0, 180.0, &format!("before any nudge: u = {}, v = {}, J = {}", f(base.u), f(base.v), f(base.j)), 12, tok::INK, Anchor::Start);
    s.text(16.0, 200.0, "the nudge measures one gradient at a time; the backward pass gets all three in one sweep", 11, tok::INK2, Anchor::Start);
    ("b08-3-nudge".to_string(), s.finish())
}

// ------------------------------------------------------------------------------------------------------------------
// Chapter 9
// ------------------------------------------------------------------------------------------------------------------
/// The price plot's axes, grid and the dashed true relationship; returns the frame.
fn price_axes(s: &mut Svg, left: f64, top: f64, w: f64, h: f64) -> Frame {
    let fr = Frame::fit(left, top, w, h, (40.0, 420.0), (0.0, 14.0));
    for k in 0..=7 {
        let y = 2.0 * k as f64;
        s.line(fr.left, fr.py(y), fr.right(), fr.py(y), tok::GRID, 1.0);
        s.text(fr.left - 5.0, fr.py(y) + 4.0, &f(y), 11, tok::MUTED, Anchor::End);
    }
    for x in [100.0, 200.0, 300.0, 400.0] {
        s.text(fr.px(x), fr.bottom() + 14.0, &f(x), 11, tok::MUTED, Anchor::Middle);
    }
    s.line(fr.left, fr.bottom(), fr.right(), fr.bottom(), tok::AXIS, 1.0);
    s.line(fr.left, fr.top, fr.left, fr.bottom(), tok::AXIS, 1.0);
    dashed_curve(s, &fr, &true_price, (40.0, 420.0), 76, tok::INK2, 1.3);
    s.text(fr.right(), fr.bottom() + 28.0, "pages", 11, tok::INK2, Anchor::End);
    fr
}

/// Fits of chapter 9: the straight line and the squiggle on the training books.
fn ch9_fits() -> (Vec<f64>, Vec<f64>) {
    let tr = train_prices();
    (polyfit(&PAGES, &tr, 1), polyfit(&PAGES, &tr, SQUIGGLE))
}

fn fits_fig(test: bool) -> Figure {
    let ys = if test { test_prices() } else { train_prices() };
    let (line, squig) = ch9_fits();
    let (peak_x, peak) = (0..=3400)
        .map(|i| PAGES[0] + 0.1 * i as f64)
        .map(|x| (x, polyval(&squig, x)))
        .fold((0.0, f64::NEG_INFINITY), |best, p| if p.1 > best.1 { p } else { best });
    let after = PAGES.iter().position(|x| *x > peak_x).unwrap_or(PAGES.len() - 1).max(1);
    let (gap_lo, gap_hi) = (PAGES[after - 1], PAGES[after]);
    let dot_c = if test { tok::S3 } else { tok::S1 };
    let which = if test { "test" } else { "training" };
    let mut s = Svg::new(
        720,
        336,
        if test { "The same two fits on new books" } else { "A straight line and a squiggle on the training books" },
        &format!(
            "Two panels of price against pages with the true curve dashed and eight {which} books as dots. Left: the straight line fitted to the training books, squared errors {}. Right: the squiggle through every training book, squared errors {}; between {} and {} pages it leaves the chart and peaks at {}.",
            num(sse(&line, &PAGES, &ys), 2),
            num(sse(&squig, &PAGES, &ys), 2),
            f(gap_lo),
            f(gap_hi),
            num(peak, 1)
        ),
    ).min_text(12);
    for (k, c) in [&line, &squig].iter().enumerate() {
        let fr = price_axes(&mut s, 50.0 + k as f64 * 360.0, 40.0, 300.0, 200.0);
        let g = |x: f64| polyval(c, x);
        curve(&mut s, &fr, &g, (PAGES[0], PAGES[PAGES.len() - 1]), 680, tok::S2, 2.2);
        for (x, y) in PAGES.iter().zip(&ys) {
            dash_d(&mut s, &fr, (*x, *y), (*x, g(*x)), tok::INK, 1.2);
            s.dot(fr.px(*x), fr.py(*y), 4.5, dot_c, None);
        }
        let e = sse(c, &PAGES, &ys);
        let name = if k == 0 { "straight line".to_string() } else { format!("squiggle (degree {SQUIGGLE})") };
        s.text_bold(fr.cx(), 26.0, &name, 13, tok::INK, Anchor::Middle);
        s.text(fr.left, fr.bottom() + 42.0, &format!("squared errors, {which} books: {}", num(e, 2)), 12, tok::INK, Anchor::Start);
        if k == 1 {
            s.text(fr.left, fr.bottom() + 58.0, &format!("peaks at {} between {} and {} pages", num(peak, 1), f(gap_lo), f(gap_hi)), 11, tok::S2, Anchor::Start);
        }
    }
    s.text(50.0, 318.0, "price in hundreds of rupees · dashed curve: the true price · dotted lines: each book's error", 11, tok::INK2, Anchor::Start);
    ((if test { "b09-2-test-fits" } else { "b09-1-train-fits" }).to_string(), s.finish())
}

fn fig_9_3() -> Figure {
    let (tr, te) = (train_prices(), test_prices());
    let n = PAGES.len() as f64;
    let errs: Vec<V2> = (0..=SQUIGGLE)
        .map(|d| {
            let c = polyfit(&PAGES, &tr, d);
            (sse(&c, &PAGES, &tr) / n, sse(&c, &PAGES, &te) / n)
        })
        .collect();
    let best = (0..errs.len()).min_by(|a, b| errs[*a].1.partial_cmp(&errs[*b].1).unwrap_or(std::cmp::Ordering::Equal)).unwrap_or(0);
    let top = 1.5;
    // degrees whose training or test error is above the chart, with both values written out
    let off: Vec<(usize, V2)> = errs.iter().enumerate().filter(|(_, e)| e.0 > top || e.1 > top).map(|(d, e)| (d, *e)).collect();
    let off_words: Vec<String> = off.iter().map(|(d, e)| format!("degree {d}: training {}, test {}", num(e.0, 2), num(e.1, 2))).collect();
    let mut s = Svg::new(
        720,
        320,
        "Training error falls; test error falls, then rises",
        &format!(
            "Mean squared error against polynomial degree from 0 to {SQUIGGLE}. The training error (blue) falls to {} at degree {SQUIGGLE}. The test error (green) is lowest, {}, at degree {best} and climbs after it. Off the top of the chart: {}.",
            num(errs[SQUIGGLE].0, 2),
            num(errs[best].1, 2),
            off_words.join("; ")
        ),
    ).min_text(12);
    let fr = Frame::fit(70.0, 40.0, 420.0, 220.0, (-0.4, SQUIGGLE as f64 + 0.4), (0.0, top));
    for k in 0..=6 {
        let y = 0.25 * k as f64;
        s.line(fr.left, fr.py(y), fr.right(), fr.py(y), tok::GRID, 1.0);
        s.text(fr.left - 6.0, fr.py(y) + 4.0, &num(y, 2), 11, tok::MUTED, Anchor::End);
    }
    for d in 0..=SQUIGGLE {
        s.text(fr.px(d as f64), fr.bottom() + 16.0, &d.to_string(), 11, tok::MUTED, Anchor::Middle);
    }
    s.line(fr.left, fr.bottom(), fr.right(), fr.bottom(), tok::AXIS, 1.0);
    s.text(fr.cx(), fr.bottom() + 34.0, "polynomial degree (how bendy the curve may be)", 11, tok::INK2, Anchor::Middle);
    s.text(fr.left - 50.0, fr.top - 14.0, "mean squared error", 11, tok::INK2, Anchor::Start);
    for (series, colour, name) in [(0usize, tok::S1, "training"), (1usize, tok::S3, "test")] {
        let pts: Vec<V2> = errs.iter().enumerate().map(|(d, e)| (d as f64, if series == 0 { e.0 } else { e.1 })).collect();
        for w in pts.windows(2) {
            seg(&mut s, &fr, w[0], w[1], colour, 2.0);
        }
        for p in pts.iter().filter(|p| p.1 <= fr.y1) {
            s.dot(fr.px(p.0), fr.py(p.1), 4.0, colour, None);
        }
        let last = pts[pts.len() - 1];
        s.text(fr.px(last.0) - 10.0, fr.py(last.1) + if series == 0 { -8.0 } else { 4.0 }, name, 12, colour, Anchor::End);
    }
    for (d, _) in off.iter() {
        s.arrow(fr.px(*d as f64), fr.top + 26.0, fr.px(*d as f64), fr.top + 4.0, None);
    }
    if !off.is_empty() {
        s.text(fr.px(1.15), fr.top + 14.0, &format!("off the chart at {}", off_words.join("; ")), 11, tok::INK2, Anchor::Start);
    }
    let bp = fr.pt((best as f64, errs[best].1));
    s.text(bp.0, bp.1 - 34.0, "sweet spot", 12, tok::S3, Anchor::Middle);
    s.arrow(bp.0, bp.1 - 28.0, bp.0, bp.1 - 7.0, None);
    let x = 530.0;
    s.text_bold(x, 60.0, "mean squared error", 12, tok::INK, Anchor::Start);
    let mut rows: Vec<Vec<Cell>> = vec![vec![cell("degree").fill(tok::NEUTRAL), cell("training").fill(tok::NEUTRAL), cell("test").fill(tok::NEUTRAL)]];
    for (d, e) in errs.iter().enumerate() {
        let hi = if d == best { tok::FILL1 } else { tok::SURFACE };
        rows.push(vec![cell(d.to_string()), cell(num(e.0, 2)).ink(tok::S1), cell(num(e.1, 2)).ink(tok::S3).fill(hi)]);
    }
    s.cells_wh(x, 70.0, 58.0, 24.0, &rows);
    ("b09-3-complexity".to_string(), s.finish())
}

// ------------------------------------------------------------------------------------------------------------------
// Chapter 10
// ------------------------------------------------------------------------------------------------------------------
fn fig_10_1() -> Figure {
    let books = cv_books();
    let mut s = Svg::new(
        720,
        296,
        "Four blocks, each tested once",
        &format!(
            "{} priced books, sorted by pages and dealt into {FOLDS} blocks. In each of {FOLDS} rounds one block (green) is held back for testing and the others (blue) train the model; every book is tested exactly once.",
            books.len()
        ),
    ).min_text(12);
    let (bx, bw, bh) = (110.0, 140.0, 34.0);
    for k in 0..FOLDS {
        let pages: Vec<String> = books.iter().filter(|b| b.2 == k).map(|b| f(b.0)).collect();
        s.text(bx + k as f64 * bw + bw / 2.0, 36.0, &format!("block {}", k + 1), 12, tok::INK, Anchor::Middle);
        s.text(bx + k as f64 * bw + bw / 2.0, 52.0, &pages.join(", "), 11, tok::INK2, Anchor::Middle);
    }
    for r in 0..FOLDS {
        let y = 66.0 + r as f64 * (bh + 12.0);
        s.text(bx - 12.0, y + bh / 2.0 + 4.0, &format!("round {}", r + 1), 12, tok::INK, Anchor::End);
        for k in 0..FOLDS {
            let test = k == r;
            let x = bx + k as f64 * bw;
            s.rect(x + 3.0, y, bw - 6.0, bh, if test { tok::S3 } else { tok::FILL2 }, Some(if test { tok::S3 } else { tok::S1 }));
            if test {
                s.text_bold(x + bw / 2.0, y + bh / 2.0 + 4.0, "test", 12, tok::INK, Anchor::Middle);
            } else {
                s.text(x + bw / 2.0, y + bh / 2.0 + 4.0, "train", 12, tok::INK, Anchor::Middle);
            }
        }
    }
    s.text(bx, 264.0, "under each block: the page counts of the four books it holds", 11, tok::INK2, Anchor::Start);
    s.text(bx, 280.0, "a method's score is the average of its four test errors", 11, tok::INK2, Anchor::Start);
    ("b10-1-folds".to_string(), s.finish())
}

fn fig_10_2() -> Figure {
    let scores: Vec<Vec<f64>> = CV_DEGREES.iter().map(|d| cv_scores(*d)).collect();
    let means: Vec<f64> = scores.iter().map(|v| v.iter().sum::<f64>() / v.len() as f64).collect();
    let best = (0..means.len()).min_by(|a, b| means[*a].partial_cmp(&means[*b]).unwrap_or(std::cmp::Ordering::Equal)).unwrap_or(0);
    let names: Vec<String> = CV_DEGREES.iter().map(|d| format!("degree {d}")).collect();
    let mut s = Svg::new(
        720,
        270,
        "The cross-validation scoreboard",
        &format!(
            "A table of test errors: rows are the {FOLDS} rounds and their average, columns are curves of degree {}. The {} curve has the lowest average, {}.",
            CV_DEGREES.iter().map(|d| d.to_string()).collect::<Vec<_>>().join(", "),
            names[best],
            num(means[best], 2)
        ),
    ).min_text(12);
    let (x0, cw0, cw) = (30.0, 150.0, 104.0);
    let mut head = vec![cell("held-out block").fill(tok::NEUTRAL)];
    for n in names.iter() {
        head.push(cell(n.as_str()).fill(tok::NEUTRAL));
    }
    let mut rows: Vec<Vec<Cell>> = vec![head];
    for k in 0..FOLDS {
        let mut r = vec![cell(format!("round {} (block {})", k + 1, k + 1))];
        for sc in scores.iter() {
            r.push(cell(num(sc[k], 2)));
        }
        rows.push(r);
    }
    let mut last = vec![cell("average").fill(tok::NEUTRAL)];
    for (i, m) in means.iter().enumerate() {
        last.push(cell(num(*m, 2)).fill(if i == best { tok::FILL3 } else { tok::NEUTRAL }));
    }
    rows.push(last);
    let mut widths = vec![cw0];
    widths.extend(std::iter::repeat_n(cw, CV_DEGREES.len()));
    table(&mut s, x0, 40.0, &widths, 28.0, &rows);
    for (i, d) in CV_DEGREES.iter().enumerate() {
        let w = match d {
            1 => "straight line",
            2 => "gentle curve",
            _ if *d == SQUIGGLE => "squiggle",
            _ => "",
        };
        if !w.is_empty() {
            s.text(x0 + cw0 + cw * (i as f64 + 0.5), 32.0, w, 11, tok::INK2, Anchor::Middle);
        }
    }
    s.text(x0, 234.0, &format!("lowest average: {} ({}), so cross-validation picks it", names[best], num(means[best], 2)), 12, tok::INK, Anchor::Start);
    s.text(x0, 252.0, "each number is a mean squared error on four held-out books", 11, tok::INK2, Anchor::Start);
    ("b10-2-scoreboard".to_string(), s.finish())
}

// ------------------------------------------------------------------------------------------------------------------
// Chapter 11
// ------------------------------------------------------------------------------------------------------------------
fn ridge_axes(s: &mut Svg, left: f64, top: f64, w: f64, h: f64) -> Frame {
    let fr = Frame::fit(left, top, w, h, (0.0, 4.5), (0.0, 7.0));
    for k in 0..=7 {
        s.line(fr.left, fr.py(k as f64), fr.right(), fr.py(k as f64), tok::GRID, 1.0);
        s.text(fr.left - 5.0, fr.py(k as f64) + 4.0, &k.to_string(), 11, tok::MUTED, Anchor::End);
    }
    for k in 1..=4 {
        s.line(fr.px(k as f64), fr.top, fr.px(k as f64), fr.bottom(), tok::GRID, 1.0);
        s.text(fr.px(k as f64), fr.bottom() + 14.0, &k.to_string(), 11, tok::MUTED, Anchor::Middle);
    }
    s.line(fr.left, fr.bottom(), fr.right(), fr.bottom(), tok::AXIS, 1.0);
    s.line(fr.left, fr.top, fr.left, fr.bottom(), tok::AXIS, 1.0);
    fr
}

fn fig_11_1() -> Figure {
    let ls = ridge_line(LAMBDAS[0]);
    let rl = ridge_line(LAMBDAS[1]);
    let lam = LAMBDAS[1];
    let (ssr_ls, ssr_r) = (ridge_ssr(ls), ridge_ssr(rl));
    let miss: Vec<f64> = RIDGE.iter().map(|p| p.1 - (rl.0 + rl.1 * p.0)).collect();
    let mut s = Svg::new(
        720,
        320,
        "Least squares against ridge on two points",
        &format!(
            "Two books, {} and {} (pages and price in hundreds). The least-squares line {} passes through both. The ridge line for λ = {}, {}, misses each by {} but has the smaller total of squared misses plus λ times slope squared.",
            v2s(RIDGE[0]),
            v2s(RIDGE[1]),
            line_label(ls),
            f(lam),
            line_label(rl),
            f(miss[1].abs())
        ),
    ).min_text(12);
    let fr = ridge_axes(&mut s, 50.0, 30.0, 300.0, 230.0);
    let (lc, rc) = (tok::S2, tok::S3);
    curve(&mut s, &fr, &|x: f64| ls.0 + ls.1 * x, (0.0, 4.5), 2, lc, 2.2);
    curve(&mut s, &fr, &|x: f64| rl.0 + rl.1 * x, (0.0, 4.5), 2, rc, 2.2);
    for p in RIDGE.iter() {
        dash_d(&mut s, &fr, *p, (p.0, rl.0 + rl.1 * p.0), rc, 1.4);
        s.dot(fr.px(p.0), fr.py(p.1), 5.0, tok::S1, None);
    }
    // name each line where it leaves the plot: the steep one through the top, the flat one through the right side
    let top_exit = (fr.y1 - ls.0) / ls.1;
    s.text(fr.px(top_exit), fr.top - 6.0, "least squares", 11, lc, Anchor::Middle);
    s.text(fr.right() + 4.0, fr.py(rl.0 + rl.1 * fr.x1) + 4.0, "ridge", 11, rc, Anchor::Start);
    s.text(fr.cx(), fr.bottom() + 32.0, "pages (hundreds)", 11, tok::INK2, Anchor::Middle);
    s.text(fr.left - 30.0, fr.top - 12.0, "price (hundreds of rupees)", 11, tok::INK2, Anchor::Start);
    let x = 390.0;
    s.text_bold(x, 50.0, &format!("both lines scored with λ = {}", f(lam)), 13, tok::INK, Anchor::Start);
    s.text_bold(x, 80.0, &format!("least squares: {}", line_label(ls)), 12, lc, Anchor::Start);
    lines(
        &mut s,
        x,
        98.0,
        &[
            (&format!("squared misses: {}", f(ssr_ls)), tok::INK),
            (&format!("penalty: {} × {}² = {}", f(lam), f(ls.1), f(lam * ls.1 * ls.1)), tok::INK),
            (&format!("total: {}", f(ssr_ls + lam * ls.1 * ls.1)), tok::INK),
        ],
    );
    s.text_bold(x, 170.0, &format!("ridge: {}", line_label(rl)), 12, rc, Anchor::Start);
    lines(
        &mut s,
        x,
        188.0,
        &[
            (&format!("squared misses: {}² + {}² = {}", fp(miss[0]), fp(miss[1]), f(ssr_r)), tok::INK),
            (&format!("penalty: {} × {}² = {}", f(lam), f(rl.1), f(lam * rl.1 * rl.1)), tok::INK),
            (&format!("total: {}, the smallest possible", f(ssr_r + lam * rl.1 * rl.1)), tok::INK),
        ],
    );
    s.text(x, 262.0, "the penalty is charged on the slope only,", 11, tok::INK2, Anchor::Start);
    s.text(x, 277.0, "never on the intercept", 11, tok::INK2, Anchor::Start);
    ("b11-1-lines".to_string(), s.finish())
}

fn fig_11_2() -> Figure {
    let fits: Vec<(f64, V2, f64)> = LAMBDAS
        .iter()
        .map(|lam| {
            let l = ridge_line(*lam);
            (*lam, l, ridge_cost_at_slope(l.1, *lam))
        })
        .collect();
    let mut s = Svg::new(
        720,
        320,
        "The penalty moves the best slope towards zero",
        &format!(
            "Total cost against slope for λ = {}, {} and {}, with the intercept always set to its best value. The lowest points sit at slopes {}, {} and {}.",
            f(fits[0].0),
            f(fits[1].0),
            f(fits[2].0),
            f(fits[0].1.1),
            f(fits[1].1.1),
            f(fits[2].1.1)
        ),
    ).min_text(12);
    let fr = Frame::fit(60.0, 36.0, 400.0, 230.0, (-0.25, 2.25), (0.0, 6.0));
    for k in 0..=6 {
        s.line(fr.left, fr.py(k as f64), fr.right(), fr.py(k as f64), tok::GRID, 1.0);
        s.text(fr.left - 6.0, fr.py(k as f64) + 4.0, &k.to_string(), 11, tok::MUTED, Anchor::End);
    }
    for k in 0..=4 {
        let x = 0.5 * k as f64;
        s.line(fr.px(x), fr.top, fr.px(x), fr.bottom(), tok::GRID, 1.0);
        s.text(fr.px(x), fr.bottom() + 15.0, &num(x, 1), 11, tok::MUTED, Anchor::Middle);
    }
    s.line(fr.left, fr.bottom(), fr.right(), fr.bottom(), tok::AXIS, 1.0);
    s.text(fr.cx(), fr.bottom() + 33.0, "slope of the line", 11, tok::INK2, Anchor::Middle);
    s.text(fr.left - 40.0, fr.top - 14.0, "squared misses + λ × slope²", 11, tok::INK2, Anchor::Start);
    let colours = [tok::S2, tok::S3, tok::S1];
    let x = 490.0;
    s.text_bold(x, 56.0, "lowest point of each curve", 12, tok::INK, Anchor::Start);
    for (k, (lam, l, c)) in fits.iter().enumerate() {
        let g = |b: f64| ridge_cost_at_slope(b, *lam);
        curve(&mut s, &fr, &g, (-0.25, 2.25), 250, colours[k], 2.2);
        s.dot(fr.px(l.1), fr.py(*c), 5.0, colours[k], None);
        s.text_bold(x, 84.0 + 44.0 * k as f64, &format!("λ = {}", f(*lam)), 12, colours[k], Anchor::Start);
        s.text(x, 101.0 + 44.0 * k as f64, &format!("slope {}, cost {}", f(l.1), f(*c)), 12, tok::INK, Anchor::Start);
    }
    s.text(x, 230.0, "a bigger λ makes steepness", 11, tok::INK2, Anchor::Start);
    s.text(x, 246.0, "dearer, so the cheapest", 11, tok::INK2, Anchor::Start);
    s.text(x, 262.0, "slope is smaller", 11, tok::INK2, Anchor::Start);
    ("b11-2-cost".to_string(), s.finish())
}

fn fig_11_3() -> Figure {
    let test = ridge_test();
    let ls = ridge_line(LAMBDAS[0]);
    let rl = ridge_line(LAMBDAS[1]);
    let r3 = ridge_line(LAMBDAS[2]);
    let mut s = Svg::new(
        720,
        320,
        "The flatter line predicts new books better",
        &format!(
            "{} new books (green) around a gentler trend. The least-squares line misses them by a total squared error of {}; the ridge line for λ = {} by {}.",
            test.len(),
            f(ridge_test_sse(ls)),
            f(LAMBDAS[1]),
            f(ridge_test_sse(rl))
        ),
    ).min_text(12);
    for (k, (l, c, name)) in [(ls, tok::S2, format!("least squares (λ = {})", f(LAMBDAS[0]))), (rl, tok::S3, format!("ridge (λ = {})", f(LAMBDAS[1])))]
        .iter()
        .enumerate()
    {
        let fr = ridge_axes(&mut s, 50.0 + k as f64 * 360.0, 40.0, 290.0, 200.0);
        curve(&mut s, &fr, &|x: f64| l.0 + l.1 * x, (0.0, 4.5), 2, c, 2.2);
        for p in test.iter() {
            dash_d(&mut s, &fr, *p, (p.0, l.0 + l.1 * p.0), tok::INK, 1.2);
            s.dot(fr.px(p.0), fr.py(p.1), 4.5, tok::S3, None);
        }
        for p in RIDGE.iter() {
            s.dot(fr.px(p.0), fr.py(p.1), 4.0, tok::SURFACE, Some(tok::S1));
        }
        s.text_bold(fr.cx(), 26.0, name, 13, c, Anchor::Middle);
        s.text(fr.left, fr.bottom() + 34.0, &format!("squared misses on the {} new books: {}", test.len(), f(ridge_test_sse(*l))), 12, tok::INK, Anchor::Start);
    }
    s.text(
        50.0,
        292.0,
        &format!("open circles: the two training books · the line for λ = {} scores {}", f(LAMBDAS[2]), f(ridge_test_sse(r3))),
        11,
        tok::INK2,
        Anchor::Start,
    );
    s.text(50.0, 308.0, &format!("the new books follow {} plus small fixed noise", line_label(RIDGE_TRUE)), 11, tok::INK2, Anchor::Start);
    ("b11-3-test".to_string(), s.finish())
}

fn fig_11_4() -> Figure {
    let fits: Vec<(f64, V2)> = LAMBDAS.iter().map(|lam| (*lam, ridge_line(*lam))).collect();
    let mut s = Svg::new(
        720,
        300,
        "A smaller slope means calmer predictions",
        &format!(
            "The three lines for λ = {}, {} and {} with a step of one unit of pages drawn on each: the predicted price rises by {}, {} and {}.",
            f(fits[0].0),
            f(fits[1].0),
            f(fits[2].0),
            f(fits[0].1.1),
            f(fits[1].1.1),
            f(fits[2].1.1)
        ),
    ).min_text(12);
    let fr = ridge_axes(&mut s, 50.0, 30.0, 330.0, 220.0);
    let colours = [tok::S2, tok::S3, tok::S1];
    let x = 420.0;
    s.text_bold(x, 50.0, "one more unit of x adds this much:", 12, tok::INK, Anchor::Start);
    for (k, (lam, l)) in fits.iter().enumerate() {
        curve(&mut s, &fr, &|t: f64| l.0 + l.1 * t, (0.0, 4.5), 2, colours[k], 2.2);
        let x0 = 3.0;
        let y0 = l.0 + l.1 * x0;
        dash_d(&mut s, &fr, (x0, y0), (x0 + 1.0, y0), colours[k], 1.3);
        arrow_d(&mut s, &fr, (x0 + 1.0, y0), (x0 + 1.0, y0 + l.1), colours[k], 2.0);
        s.text_bold(x, 80.0 + 34.0 * k as f64, &format!("λ = {}: {}", f(*lam), f(l.1)), 12, colours[k], Anchor::Start);
        s.text(x + 110.0, 80.0 + 34.0 * k as f64, &line_label(*l), 12, tok::INK2, Anchor::Start);
    }
    s.text(fr.cx(), fr.bottom() + 32.0, "x: pages (hundreds)", 11, tok::INK2, Anchor::Middle);
    let (xm, ym) = ridge_means();
    s.text(x, 196.0, "a flat line shrugs off a change in", 11, tok::INK2, Anchor::Start);
    s.text(x, 212.0, "the input; a steep one passes", 11, tok::INK2, Anchor::Start);
    s.text(x, 228.0, "every wobble straight on", 11, tok::INK2, Anchor::Start);
    s.text(x, 256.0, &format!("all pass through {},", v2s((xm, ym))), 11, tok::INK2, Anchor::Start);
    s.text(x, 272.0, "the average book", 11, tok::INK2, Anchor::Start);
    ("b11-4-sensitivity".to_string(), s.finish())
}
