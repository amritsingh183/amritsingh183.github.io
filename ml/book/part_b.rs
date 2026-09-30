//! Figures of Parts III and IV (chapters 12 to 24) of "Machine Learning, Drawn Out". The values these figures draw are
//! computed here from the shared inputs of the book's specification (the page-count sample, the book ages, the reviewer
//! grid, the cover image and its kernels, the edge image and the Sobel kernel, the pooling input) or from small inputs
//! defined below. Two things are typed as text instead: numbers in labels, such as sizes ("stride 2") and simple counts
//! ("64 different weights"), and the pair of confidences that Figure 23.3 quotes from Goodfellow, Shlens and Szegedy.
use crate::Figure;
use crate::svg::{Anchor, Cell, Svg, cell, heat, num, numt, thousands, tok};

/// Inputs for the figures of Parts III and IV, a seeded pseudo-random generator for the page-count sample, and
/// functions that compute values the figures draw. Numbers typed as text in labels, and the two confidences that
/// Figure 23.3 quotes from Goodfellow, Shlens and Szegedy, do not come from here.
pub mod calc {
    use std::f64::consts::PI;

    // ------------------------------------------------------------------------------------------------
    // The page-count sample: LCG x <- 6364136223846793005·x + 1442695040888963407 (mod 2^64), seed 20260929.
    // Each uniform uses the top 53 bits of the new state plus one half, so it lies strictly inside (0, 1).
    // Box–Muller turns each pair of uniforms into two standard normals z (cosine first, then sine); a page
    // count is 240·exp(0.45·z) rounded to a whole page (a lognormal with median 240 pages and σ = 0.45).
    // ------------------------------------------------------------------------------------------------
    pub struct Lcg {
        state: u64,
    }

    impl Lcg {
        pub fn new(seed: u64) -> Lcg {
            Lcg { state: seed }
        }
        fn next_u64(&mut self) -> u64 {
            self.state = self.state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            self.state
        }
        pub fn uniform(&mut self) -> f64 {
            ((self.next_u64() >> 11) as f64 + 0.5) / 9007199254740992.0
        }
    }

    pub const SAMPLE_N: usize = 60;
    pub const MEDIAN_PAGES: f64 = 240.0;
    pub const SIGMA: f64 = 0.45;

    pub fn page_sample() -> Vec<f64> {
        let mut g = Lcg::new(20260929);
        let mut out = Vec::with_capacity(SAMPLE_N);
        while out.len() < SAMPLE_N {
            let (u1, u2) = (g.uniform(), g.uniform());
            let r = (-2.0 * u1.ln()).sqrt();
            let t = 2.0 * PI * u2;
            for z in [r * t.cos(), r * t.sin()] {
                if out.len() < SAMPLE_N {
                    out.push((MEDIAN_PAGES * (SIGMA * z).exp()).round());
                }
            }
        }
        out
    }

    pub fn sorted(x: &[f64]) -> Vec<f64> {
        let mut v = x.to_vec();
        v.sort_by(|a, b| a.total_cmp(b));
        v
    }

    pub fn mean(x: &[f64]) -> f64 {
        x.iter().sum::<f64>() / x.len() as f64
    }

    /// Sample standard deviation (divides by n − 1).
    pub fn sd_sample(x: &[f64]) -> f64 {
        let m = mean(x);
        (x.iter().map(|v| (v - m).powi(2)).sum::<f64>() / (x.len() as f64 - 1.0)).sqrt()
    }

    /// Population standard deviation (divides by n), as scikit-learn's StandardScaler uses.
    pub fn sd_pop(x: &[f64]) -> f64 {
        let m = mean(x);
        (x.iter().map(|v| (v - m).powi(2)).sum::<f64>() / x.len() as f64).sqrt()
    }

    /// Quantile with linear interpolation between order statistics (NumPy's default, used by RobustScaler).
    pub fn quantile(x: &[f64], q: f64) -> f64 {
        let s = sorted(x);
        let pos = q * (s.len() as f64 - 1.0);
        let lo = pos.floor() as usize;
        let hi = pos.ceil() as usize;
        s[lo] + (pos - lo as f64) * (s[hi] - s[lo])
    }

    // ------------------------------------------------------------------------------------------------
    // Special functions.
    // ------------------------------------------------------------------------------------------------
    /// Complementary error function (Chebyshev fit, fractional error below 1.2e-7 everywhere).
    pub fn erfc(x: f64) -> f64 {
        let z = x.abs();
        let t = 1.0 / (1.0 + 0.5 * z);
        let r = t * (-z * z - 1.26551223
            + t * (1.00002368
                + t * (0.37409196
                    + t * (0.09678418
                        + t * (-0.18628806
                            + t * (0.27886807 + t * (-1.13520398 + t * (1.48851587 + t * (-0.82215223 + t * 0.17087277)))))))))
            .exp();
        if x >= 0.0 { r } else { 2.0 - r }
    }

    pub fn norm_cdf(z: f64) -> f64 {
        0.5 * erfc(-z / std::f64::consts::SQRT_2)
    }

    pub fn norm_pdf(z: f64) -> f64 {
        (-0.5 * z * z).exp() / (2.0 * PI).sqrt()
    }

    /// Inverse of the standard normal CDF (Acklam's rational approximation, relative error below 1.2e-9).
    pub fn norm_ppf(p: f64) -> f64 {
        let a = [-3.969683028665376e1, 2.209460984245205e2, -2.759285104469687e2, 1.383577518672690e2, -3.066479806614716e1, 2.506628277459239];
        let b = [-5.447609879822406e1, 1.615858368580409e2, -1.556989798598866e2, 6.680131188771972e1, -1.328068155288572e1];
        let c = [-7.784894002430293e-3, -3.223964580411365e-1, -2.400758277161838, -2.549732539343734, 4.374664141464968, 2.938163982698783];
        let d = [7.784695709041462e-3, 3.224671290700398e-1, 2.445134137142996, 3.754408661907416];
        let lo = 0.02425;
        if p < lo {
            let q = (-2.0 * p.ln()).sqrt();
            (((((c[0] * q + c[1]) * q + c[2]) * q + c[3]) * q + c[4]) * q + c[5]) / ((((d[0] * q + d[1]) * q + d[2]) * q + d[3]) * q + 1.0)
        } else if p <= 1.0 - lo {
            let q = p - 0.5;
            let r = q * q;
            (((((a[0] * r + a[1]) * r + a[2]) * r + a[3]) * r + a[4]) * r + a[5]) * q
                / (((((b[0] * r + b[1]) * r + b[2]) * r + b[3]) * r + b[4]) * r + 1.0)
        } else {
            let q = (-2.0 * (1.0 - p).ln()).sqrt();
            -(((((c[0] * q + c[1]) * q + c[2]) * q + c[3]) * q + c[4]) * q + c[5]) / ((((d[0] * q + d[1]) * q + d[2]) * q + d[3]) * q + 1.0)
        }
    }

    /// ln Γ(x) for x > 0 (Lanczos, g = 7, nine coefficients).
    pub fn ln_gamma(x: f64) -> f64 {
        let g = [
            0.999_999_999_999_809_9,
            676.5203681218851,
            -1259.1392167224028,
            771.323_428_777_653_1,
            -176.615_029_162_140_6,
            12.507343278686905,
            -0.13857109526572012,
            9.984_369_578_019_572e-6,
            1.5056327351493116e-7,
        ];
        let x = x - 1.0;
        let mut a = g[0];
        let t = x + 7.5;
        for (i, gi) in g.iter().enumerate().skip(1) {
            a += gi / (x + i as f64);
        }
        0.5 * (2.0 * PI).ln() + (x + 0.5) * t.ln() - t + a.ln()
    }

    pub fn digamma(mut x: f64) -> f64 {
        let mut r = 0.0;
        while x < 6.0 {
            r -= 1.0 / x;
            x += 1.0;
        }
        let f = 1.0 / (x * x);
        r + x.ln() - 0.5 / x - f * (1.0 / 12.0 - f * (1.0 / 120.0 - f * (1.0 / 252.0 - f * (1.0 / 240.0 - f / 132.0))))
    }

    pub fn trigamma(mut x: f64) -> f64 {
        let mut r = 0.0;
        while x < 6.0 {
            r += 1.0 / (x * x);
            x += 1.0;
        }
        let f = 1.0 / (x * x);
        r + 1.0 / x + f / 2.0 + f / x * (1.0 / 6.0 - f * (1.0 / 30.0 - f * (1.0 / 42.0 - f / 30.0)))
    }

    /// Regularised lower incomplete gamma P(a, x): series below a + 1, continued fraction above.
    pub fn gamma_p(a: f64, x: f64) -> f64 {
        if x <= 0.0 {
            return 0.0;
        }
        let lead = (-x + a * x.ln() - ln_gamma(a)).exp();
        if x < a + 1.0 {
            let (mut ap, mut sum, mut del) = (a, 1.0 / a, 1.0 / a);
            for _ in 0..500 {
                ap += 1.0;
                del *= x / ap;
                sum += del;
                if del.abs() < sum.abs() * 1e-15 {
                    break;
                }
            }
            sum * lead
        } else {
            let tiny = 1e-300;
            let mut b = x + 1.0 - a;
            let mut c = 1.0 / tiny;
            let mut d = 1.0 / b;
            let mut h = d;
            for i in 1..500 {
                let an = -(i as f64) * (i as f64 - a);
                b += 2.0;
                d = an * d + b;
                if d.abs() < tiny {
                    d = tiny;
                }
                c = b + an / c;
                if c.abs() < tiny {
                    c = tiny;
                }
                d = 1.0 / d;
                let del = d * c;
                h *= del;
                if (del - 1.0).abs() < 1e-15 {
                    break;
                }
            }
            1.0 - lead * h
        }
    }

    /// Solves F(x) = p for an increasing CDF by bisection on [lo, hi].
    fn invert(f: impl Fn(f64) -> f64, p: f64, mut lo: f64, mut hi: f64) -> f64 {
        for _ in 0..200 {
            let mid = 0.5 * (lo + hi);
            if f(mid) < p {
                lo = mid;
            } else {
                hi = mid;
            }
        }
        0.5 * (lo + hi)
    }

    // ------------------------------------------------------------------------------------------------
    // Candidate distributions for the page counts, each fitted to the sample.
    // ------------------------------------------------------------------------------------------------
    #[derive(Clone, Copy)]
    pub enum Dist {
        /// mean, standard deviation
        Normal(f64, f64),
        /// mean and standard deviation of ln x
        Lognormal(f64, f64),
        /// shape k, scale θ
        Gamma(f64, f64),
        /// shape β, scale λ
        Weibull(f64, f64),
        /// threshold θ, then mean and standard deviation of ln(x − θ)
        Lognormal3(f64, f64, f64),
    }

    impl Dist {
        /// The two parameters every family here shares a slot for: location and spread (mean and sd, or the mean and sd
        /// of the logarithms), or shape and scale for the gamma and Weibull.
        pub fn params(&self) -> (f64, f64) {
            match *self {
                Dist::Normal(a, b) | Dist::Lognormal(a, b) | Dist::Gamma(a, b) | Dist::Weibull(a, b) | Dist::Lognormal3(_, a, b) => (a, b),
            }
        }
        pub fn cdf(&self, x: f64) -> f64 {
            match *self {
                Dist::Normal(m, s) => norm_cdf((x - m) / s),
                Dist::Lognormal(m, s) => {
                    if x <= 0.0 { 0.0 } else { norm_cdf((x.ln() - m) / s) }
                }
                Dist::Gamma(k, t) => gamma_p(k, x / t),
                Dist::Weibull(b, l) => {
                    if x <= 0.0 { 0.0 } else { 1.0 - (-(x / l).powf(b)).exp() }
                }
                Dist::Lognormal3(th, m, s) => {
                    if x <= th { 0.0 } else { norm_cdf(((x - th).ln() - m) / s) }
                }
            }
        }
        pub fn pdf(&self, x: f64) -> f64 {
            match *self {
                Dist::Normal(m, s) => norm_pdf((x - m) / s) / s,
                Dist::Lognormal(m, s) => {
                    if x <= 0.0 { 0.0 } else { norm_pdf((x.ln() - m) / s) / (s * x) }
                }
                Dist::Gamma(k, t) => {
                    if x <= 0.0 { 0.0 } else { ((k - 1.0) * (x / t).ln() - x / t - ln_gamma(k)).exp() / t }
                }
                Dist::Weibull(b, l) => {
                    if x <= 0.0 { 0.0 } else { b / l * (x / l).powf(b - 1.0) * (-(x / l).powf(b)).exp() }
                }
                Dist::Lognormal3(th, m, s) => {
                    if x <= th { 0.0 } else { norm_pdf(((x - th).ln() - m) / s) / (s * (x - th)) }
                }
            }
        }
        pub fn ppf(&self, p: f64) -> f64 {
            match *self {
                Dist::Normal(m, s) => m + s * norm_ppf(p),
                Dist::Lognormal(m, s) => (m + s * norm_ppf(p)).exp(),
                Dist::Gamma(k, t) => invert(|x| gamma_p(k, x / t), p, 0.0, 100.0 * k * t + 100.0 * t),
                Dist::Weibull(b, l) => l * (-(1.0 - p).ln()).powf(1.0 / b),
                Dist::Lognormal3(th, m, s) => th + (m + s * norm_ppf(p)).exp(),
            }
        }
    }

    pub fn fit_normal(x: &[f64]) -> Dist {
        Dist::Normal(mean(x), sd_sample(x))
    }

    pub fn fit_lognormal(x: &[f64]) -> Dist {
        let l: Vec<f64> = x.iter().map(|v| v.ln()).collect();
        Dist::Lognormal(mean(&l), sd_sample(&l))
    }

    /// A three-parameter lognormal whose threshold is fixed by subject knowledge; the other two are fitted to ln(x − θ).
    pub fn fit_lognormal3(x: &[f64], threshold: f64) -> Dist {
        let l: Vec<f64> = x.iter().map(|v| (v - threshold).ln()).collect();
        Dist::Lognormal3(threshold, mean(&l), sd_sample(&l))
    }

    /// Maximum-likelihood gamma: Newton's method on the shape, from the usual closed-form starting value.
    pub fn fit_gamma(x: &[f64]) -> Dist {
        let m = mean(x);
        let s = m.ln() - mean(&x.iter().map(|v| v.ln()).collect::<Vec<f64>>());
        let mut k = (3.0 - s + ((s - 3.0).powi(2) + 24.0 * s).sqrt()) / (12.0 * s);
        for _ in 0..100 {
            let step = (k.ln() - digamma(k) - s) / (1.0 / k - trigamma(k));
            k -= step;
            if step.abs() < 1e-12 * k {
                break;
            }
        }
        Dist::Gamma(k, m / k)
    }

    /// Maximum-likelihood Weibull: bisection on the shape equation, then the scale in closed form.
    pub fn fit_weibull(x: &[f64]) -> Dist {
        let top = x.iter().copied().fold(f64::MIN, f64::max);
        let u: Vec<f64> = x.iter().map(|v| v / top).collect();
        let mean_ln = mean(&u.iter().map(|v| v.ln()).collect::<Vec<f64>>());
        let g = |b: f64| {
            let (mut num, mut den) = (0.0, 0.0);
            for v in &u {
                let p = v.powf(b);
                num += p * v.ln();
                den += p;
            }
            num / den - 1.0 / b - mean_ln
        };
        let (mut lo, mut hi) = (0.05, 60.0);
        for _ in 0..200 {
            let mid = 0.5 * (lo + hi);
            if g(mid) < 0.0 {
                lo = mid;
            } else {
                hi = mid;
            }
        }
        let b = 0.5 * (lo + hi);
        let scale = (u.iter().map(|v| v.powf(b)).sum::<f64>() / u.len() as f64).powf(1.0 / b) * top;
        Dist::Weibull(b, scale)
    }

    /// Anderson–Darling A² of sorted data against a fitted CDF (NIST e-Handbook 1.3.5.14).
    pub fn anderson_darling(sorted_x: &[f64], d: &Dist) -> f64 {
        let n = sorted_x.len();
        let mut s = 0.0;
        for i in 0..n {
            let f_lo = d.cdf(sorted_x[i]).clamp(1e-300, 1.0 - 1e-16);
            let f_hi = d.cdf(sorted_x[n - 1 - i]).clamp(1e-300, 1.0 - 1e-16);
            s += (2.0 * (i as f64 + 1.0) - 1.0) / n as f64 * (f_lo.ln() + (1.0 - f_hi).ln());
        }
        -(n as f64) - s
    }

    /// The small-sample adjustment A*² = A²(1 + 0.75/n + 2.25/n²) (R nortest, after Stephens 1986).
    pub fn ad_adjusted(a2: f64, n: usize) -> f64 {
        let n = n as f64;
        a2 * (1.0 + 0.75 / n + 2.25 / (n * n))
    }

    /// The p-value of the adjusted statistic for a normal fit with estimated mean and sd (Stephens 1986, Table 4.9,
    /// as implemented in R's nortest::ad.test).
    pub fn ad_pvalue_normal(aa: f64) -> f64 {
        if aa < 0.2 {
            1.0 - (-13.436 + 101.14 * aa - 223.73 * aa * aa).exp()
        } else if aa < 0.34 {
            1.0 - (-8.318 + 42.796 * aa - 59.938 * aa * aa).exp()
        } else if aa < 0.6 {
            (0.9177 - 4.279 * aa - 1.38 * aa * aa).exp()
        } else if aa < 10.0 {
            (1.2937 - 5.709 * aa + 0.0186 * aa * aa).exp()
        } else {
            3.7e-24
        }
    }

    // ------------------------------------------------------------------------------------------------
    // Small inputs of Part III.
    // ------------------------------------------------------------------------------------------------
    /// How many books each of 40 customers bought on one visit: 14 bought one, 11 bought two, …, 2 bought six.
    pub const BASKET: [u32; 6] = [14, 11, 7, 4, 2, 2];

    pub fn basket_pmf() -> Vec<f64> {
        let total: u32 = BASKET.iter().sum();
        BASKET.iter().map(|c| f64::from(*c) / f64::from(total)).collect()
    }

    /// The book ages column of the specification, in years (40 is the outlier).
    pub const AGES: [f64; 8] = [1.0, 2.0, 2.0, 3.0, 4.0, 5.0, 6.0, 40.0];

    /// Print runs (copies printed) of six titles on the shelf, spanning four orders of magnitude.
    pub const PRINT_RUNS: [f64; 6] = [800.0, 2500.0, 6000.0, 40000.0, 300000.0, 2000000.0];

    /// UNESCO's 1964 statistical definition of a book: a publication of at least 49 pages.
    pub const BOOK_FLOOR: f64 = 49.0;

    /// Gradient descent on L(w) = ½(λ₁w₁² + λ₂w₂²) with the best fixed rate η = 2/(λ₁ + λ₂): the path and the number of
    /// steps until the distance to the minimum falls below 1% of the start.
    pub fn descent(l1: f64, l2: f64, start: (f64, f64), keep: usize) -> (Vec<(f64, f64)>, usize) {
        let eta = 2.0 / (l1 + l2);
        let (mut w1, mut w2) = start;
        let d0 = (w1 * w1 + w2 * w2).sqrt();
        let mut path = vec![(w1, w2)];
        let mut steps = 0;
        while (w1 * w1 + w2 * w2).sqrt() > 0.01 * d0 && steps < 100_000 {
            w1 -= eta * l1 * w1;
            w2 -= eta * l2 * w2;
            steps += 1;
            if path.len() <= keep {
                path.push((w1, w2));
            }
        }
        (path, steps)
    }

    /// The reviewers' grid of the specification: three reviewers (columns), five books (rows).
    pub fn reviewers() -> Vec<Vec<f64>> {
        vec![vec![5.0, 2.0, 3.0, 4.0, 1.0], vec![4.0, 1.0, 4.0, 2.0, 3.0], vec![3.0, 4.0, 6.0, 8.0, 2.0]]
    }

    /// Two critics and two fans score five books; fans genuinely score books 1–4 three points higher and book 5 the same;
    /// the second reviewer of each group marks one point lower (a technical difference) and swaps books 2 and 3.
    pub fn critics_and_fans() -> Vec<Vec<f64>> {
        vec![
            vec![2.0, 3.0, 4.0, 5.0, 6.0],
            vec![1.0, 3.0, 2.0, 4.0, 5.0],
            vec![5.0, 6.0, 7.0, 8.0, 6.0],
            vec![4.0, 6.0, 5.0, 7.0, 5.0],
        ]
    }

    pub struct Qn {
        pub sorted_cols: Vec<Vec<f64>>,
        pub rank_means: Vec<f64>,
        pub out: Vec<Vec<f64>>,
    }

    /// Quantile normalisation of columns (samples): sort each column, average across rows, substitute back in each
    /// column's original order; tied values receive the mean of the rank means they span.
    pub fn quantile_normalise(cols: &[Vec<f64>]) -> Qn {
        let n = cols[0].len();
        let sorted_cols: Vec<Vec<f64>> = cols.iter().map(|c| sorted(c)).collect();
        let rank_means: Vec<f64> =
            (0..n).map(|r| sorted_cols.iter().map(|c| c[r]).sum::<f64>() / cols.len() as f64).collect();
        let out = cols
            .iter()
            .zip(&sorted_cols)
            .map(|(c, sc)| {
                c.iter()
                    .map(|v| {
                        let first = sc.iter().position(|x| x == v).unwrap_or(0);
                        let last = sc.iter().rposition(|x| x == v).unwrap_or(first);
                        rank_means[first..=last].iter().sum::<f64>() / (last - first + 1) as f64
                    })
                    .collect()
            })
            .collect();
        Qn { sorted_cols, rank_means, out }
    }

    /// Per book (row): mean of the fans' columns minus mean of the critics' columns (columns 0–1 critics, 2–3 fans).
    pub fn fan_gap(cols: &[Vec<f64>]) -> Vec<f64> {
        (0..cols[0].len()).map(|r| (cols[2][r] + cols[3][r]) / 2.0 - (cols[0][r] + cols[1][r]) / 2.0).collect()
    }

    /// One hidden unit's weighted sums for a mini-batch of six books, and the learned scale and shift used in the figure.
    pub const BATCH: [f64; 6] = [14.0, 9.5, 21.0, 12.5, 17.0, 10.0];
    pub const GAMMA: f64 = 2.0;
    pub const BETA: f64 = 1.0;
    pub const EPS: f64 = 1e-5;

    pub struct Bn {
        pub mean: f64,
        pub var: f64,
        pub xhat: Vec<f64>,
        pub y: Vec<f64>,
    }

    /// The batch-normalising transform of Ioffe and Szegedy (2015), Algorithm 1.
    pub fn batch_norm(x: &[f64], gamma: f64, beta: f64) -> Bn {
        let m = mean(x);
        let var = x.iter().map(|v| (v - m).powi(2)).sum::<f64>() / x.len() as f64;
        let xhat: Vec<f64> = x.iter().map(|v| (v - m) / (var + EPS).sqrt()).collect();
        let y = xhat.iter().map(|v| gamma * v + beta).collect();
        Bn { mean: m, var, xhat, y }
    }

    // ------------------------------------------------------------------------------------------------
    // Inputs of Part IV (images, kernels, feature maps).
    // ------------------------------------------------------------------------------------------------
    pub type Grid = Vec<Vec<f64>>;

    /// The 6 × 6 cover of the specification: 0 everywhere except a hollow square of 9s (rows and columns 1–4, inner 2 × 2 empty).
    pub fn cover() -> Grid {
        (0..6)
            .map(|r| {
                (0..6)
                    .map(|c| {
                        let ring = (1..=4).contains(&r) && (1..=4).contains(&c);
                        let inner = (2..=3).contains(&r) && (2..=3).contains(&c);
                        if ring && !inner { 9.0 } else { 0.0 }
                    })
                    .collect()
            })
            .collect()
    }

    /// The plain kernel of the specification.
    pub fn plain_kernel() -> Grid {
        vec![vec![1.0, 0.0, 1.0], vec![0.0, 1.0, 0.0], vec![1.0, 0.0, 1.0]]
    }

    /// The 8 × 8 edge image of the specification: 1 everywhere except a 4 × 4 block of 9 at rows and columns 2–5.
    pub fn edge_image() -> Grid {
        (0..8).map(|r| (0..8).map(|c| if (2..=5).contains(&r) && (2..=5).contains(&c) { 9.0 } else { 1.0 }).collect()).collect()
    }

    /// The vertical Sobel kernel of the specification (it measures change from left to right).
    pub fn sobel() -> Grid {
        vec![vec![1.0, 0.0, -1.0], vec![2.0, 0.0, -2.0], vec![1.0, 0.0, -1.0]]
    }

    /// The pooling input of the specification.
    pub fn pooling_input() -> Grid {
        vec![vec![3.0, 7.0, 1.0, 0.0], vec![2.0, 9.0, 4.0, 5.0], vec![6.0, 1.0, 8.0, 2.0], vec![0.0, 5.0, 3.0, 9.0]]
    }

    /// A small three-channel crop (red, green, blue planes of 4 × 4) and one filter's three kernels, for chapters 18 and 22.
    pub fn rgb_crop() -> [Grid; 3] {
        [
            vec![vec![2.0, 0.0, 1.0, 3.0], vec![1.0, 3.0, 0.0, 2.0], vec![0.0, 2.0, 2.0, 1.0], vec![3.0, 1.0, 0.0, 0.0]],
            vec![vec![1.0, 1.0, 0.0, 2.0], vec![0.0, 2.0, 1.0, 1.0], vec![2.0, 0.0, 1.0, 3.0], vec![1.0, 0.0, 2.0, 1.0]],
            vec![vec![0.0, 2.0, 1.0, 0.0], vec![3.0, 0.0, 1.0, 2.0], vec![1.0, 1.0, 0.0, 2.0], vec![0.0, 3.0, 1.0, 1.0]],
        ]
    }

    pub fn filter_kernels() -> [Grid; 3] {
        [
            vec![vec![1.0, 0.0, -1.0], vec![1.0, 0.0, -1.0], vec![1.0, 0.0, -1.0]],
            vec![vec![0.0, 1.0, 0.0], vec![1.0, -2.0, 1.0], vec![0.0, 1.0, 0.0]],
            vec![vec![1.0, 1.0, 1.0], vec![0.0, 0.0, 0.0], vec![-1.0, -1.0, -1.0]],
        ]
    }

    /// The filter's single bias.
    pub const FILTER_BIAS: f64 = -2.0;

    /// Zero padding of `p` cells on every side.
    pub fn pad(x: &Grid, p: usize) -> Grid {
        let (h, w) = (x.len(), x[0].len());
        (0..h + 2 * p)
            .map(|r| (0..w + 2 * p).map(|c| if r >= p && r < h + p && c >= p && c < w + p { x[r - p][c - p] } else { 0.0 }).collect())
            .collect()
    }

    /// Output size of a convolution or pooling along one axis: ⌊(i + 2p − k)/s⌋ + 1 (Dumoulin and Visin, Relationship 6).
    pub fn out_size(i: usize, k: usize, p: usize, s: usize) -> usize {
        (i + 2 * p - k) / s + 1
    }

    /// Cross-correlation without padding (the "convolution" of deep-learning libraries): the kernel is not flipped.
    pub fn conv2d(x: &Grid, k: &Grid, stride: usize) -> Grid {
        let (kh, kw) = (k.len(), k[0].len());
        let oh = out_size(x.len(), kh, 0, stride);
        let ow = out_size(x[0].len(), kw, 0, stride);
        (0..oh)
            .map(|i| {
                (0..ow)
                    .map(|j| {
                        let mut s = 0.0;
                        for a in 0..kh {
                            for b in 0..kw {
                                s += k[a][b] * x[i * stride + a][j * stride + b];
                            }
                        }
                        s
                    })
                    .collect()
            })
            .collect()
    }

    /// Element-wise sum of grids of equal size.
    pub fn add(a: &Grid, b: &Grid) -> Grid {
        a.iter().zip(b).map(|(ra, rb)| ra.iter().zip(rb).map(|(x, y)| x + y).collect()).collect()
    }

    /// Pooling with a k × k window and stride s, taking the maximum or the average.
    pub fn pool(x: &Grid, k: usize, s: usize, max: bool) -> Grid {
        let oh = out_size(x.len(), k, 0, s);
        let ow = out_size(x[0].len(), k, 0, s);
        (0..oh)
            .map(|i| {
                (0..ow)
                    .map(|j| {
                        let vals: Vec<f64> = (0..k).flat_map(|a| (0..k).map(move |b| (a, b))).map(|(a, b)| x[i * s + a][j * s + b]).collect();
                        if max { vals.iter().copied().fold(f64::NEG_INFINITY, f64::max) } else { vals.iter().sum::<f64>() / vals.len() as f64 }
                    })
                    .collect()
            })
            .collect()
    }

    /// One-dimensional cross-correlation without padding.
    pub fn conv1d(x: &[f64], k: &[f64]) -> Vec<f64> {
        (0..x.len() + 1 - k.len()).map(|i| k.iter().enumerate().map(|(t, w)| w * x[i + t]).sum()).collect()
    }

    /// Receptive-field size and jump after each layer (kernel, stride), from r = 1 and jump = 1:
    /// r ← r + (k − 1)·jump, then jump ← jump·s (Araujo, Norris and Sim, 2019).
    pub fn receptive(layers: &[(usize, usize)]) -> Vec<(usize, usize)> {
        let (mut r, mut jump) = (1usize, 1usize);
        layers
            .iter()
            .map(|&(k, s)| {
                r += (k - 1) * jump;
                jump *= s;
                (r, jump)
            })
            .collect()
    }

    /// The input span [first, last] of every unit of every layer of a one-dimensional stack without padding.
    pub fn spans(input: usize, layers: &[(usize, usize)]) -> Vec<Vec<(usize, usize)>> {
        let mut out: Vec<Vec<(usize, usize)>> = vec![(0..input).map(|j| (j, j)).collect()];
        for &(k, s) in layers {
            let prev = out.last().cloned().unwrap_or_default();
            if prev.len() < k {
                break;
            }
            let n = (prev.len() - k) / s + 1;
            out.push((0..n).map(|j| (prev[j * s].0, prev[j * s + k - 1].1)).collect());
        }
        out
    }

    /// Number of paths from each input position to one output after `layers` layers of width-k all-ones kernels.
    pub fn path_counts(layers: usize, k: usize) -> Vec<f64> {
        let mut v = vec![1.0];
        for _ in 0..layers {
            let mut next = vec![0.0; v.len() + k - 1];
            for (i, a) in v.iter().enumerate() {
                for t in 0..k {
                    next[i + t] += a;
                }
            }
            v = next;
        }
        v
    }

    /// The probability mass function of the sum of two independent variables whose values start at 1.
    pub fn convolve_pmf(p: &[f64], q: &[f64]) -> Vec<f64> {
        let mut out = vec![0.0; p.len() + q.len() - 1];
        for (i, a) in p.iter().enumerate() {
            for (j, b) in q.iter().enumerate() {
                out[i + j] += a * b;
            }
        }
        out
    }

    /// Red, green and blue components of a "#rrggbb" colour token.
    pub fn rgb(hex: &str) -> [f64; 3] {
        let h = hex.trim_start_matches('#');
        let comp = |i: usize| u8::from_str_radix(h.get(i..i + 2).unwrap_or("00"), 16).map(f64::from).unwrap_or(0.0);
        [comp(0), comp(2), comp(4)]
    }

    pub fn sigmoid(x: f64) -> f64 {
        1.0 / (1.0 + (-x).exp())
    }

    /// Filliben's uniform order statistic medians (NIST e-Handbook 1.3.3.22).
    pub fn filliben(n: usize) -> Vec<f64> {
        let nf = n as f64;
        let last = 0.5f64.powf(1.0 / nf);
        (1..=n)
            .map(|i| {
                if i == 1 {
                    1.0 - last
                } else if i == n {
                    last
                } else {
                    (i as f64 - 0.3175) / (nf + 0.365)
                }
            })
            .collect()
    }
}


// ====================================================================================================
// Drawing helpers shared by the figures of Parts III and IV.
// ====================================================================================================

/// A coordinate with at most one decimal (for path data).
fn f1(v: f64) -> String {
    if (v - v.round()).abs() < 1e-9 { format!("{}", v.round() as i64) } else { format!("{v:.1}") }
}

/// A rectangle of data space mapped onto the canvas.
struct Plot {
    x0: f64,
    y0: f64,
    w: f64,
    h: f64,
    xmin: f64,
    xmax: f64,
    ymin: f64,
    ymax: f64,
}

impl Plot {
    fn px(&self, x: f64) -> f64 {
        self.x0 + (x - self.xmin) / (self.xmax - self.xmin) * self.w
    }
    fn py(&self, y: f64) -> f64 {
        self.y0 + self.h - (y - self.ymin) / (self.ymax - self.ymin) * self.h
    }
    fn bottom(&self) -> f64 {
        self.y0 + self.h
    }
    /// Horizontal grid lines labelled on the left, the two axis lines, x ticks with labels, and the axis titles.
    fn frame(&self, s: &mut Svg, yticks: &[f64], ydec: usize, xticks: &[f64], xdec: usize, xlabel: &str, ylabel: &str) {
        for v in yticks {
            s.line(self.x0, self.py(*v), self.x0 + self.w, self.py(*v), tok::GRID, 1.0);
            s.text(self.x0 - 6.0, self.py(*v) + 4.0, &numt(*v, ydec), 11, tok::MUTED, Anchor::End);
        }
        s.line(self.x0, self.bottom(), self.x0 + self.w, self.bottom(), tok::AXIS, 1.0);
        s.line(self.x0, self.y0, self.x0, self.bottom(), tok::AXIS, 1.0);
        for v in xticks {
            s.line(self.px(*v), self.bottom(), self.px(*v), self.bottom() + 4.0, tok::AXIS, 1.0);
            s.text(self.px(*v), self.bottom() + 16.0, &numt(*v, xdec), 11, tok::MUTED, Anchor::Middle);
        }
        if !xlabel.is_empty() {
            s.text(self.x0 + self.w / 2.0, self.bottom() + 32.0, xlabel, 11, tok::INK2, Anchor::Middle);
        }
        if !ylabel.is_empty() {
            s.text(self.x0 - 6.0, self.y0 - 12.0, ylabel, 11, tok::INK2, Anchor::Start);
        }
    }
    /// A function drawn as a polyline over [from, to], clipped to the plot's y range.
    fn curve(&self, s: &mut Svg, f: &dyn Fn(f64) -> f64, from: f64, to: f64, steps: usize, colour: &str, width: f64) {
        let mut d = String::new();
        for i in 0..=steps {
            let x = from + (to - from) * i as f64 / steps as f64;
            let y = f(x).clamp(self.ymin, self.ymax);
            d.push_str(&format!("{}{} {}", if i == 0 { "M" } else { " L" }, f1(self.px(x)), f1(self.py(y))));
        }
        s.path(&d, colour, width);
    }
    /// The area under f between a and b, filled with closely spaced vertical strokes.
    fn shade(&self, s: &mut Svg, f: &dyn Fn(f64) -> f64, a: f64, b: f64, colour: &str) {
        let (pa, pb) = (self.px(a), self.px(b));
        let n = ((pb - pa) / 2.0).ceil() as usize;
        for i in 0..n {
            let px = pa + 1.0 + i as f64 * (pb - pa - 2.0) / (n as f64 - 1.0).max(1.0);
            let x = self.xmin + (px - self.x0) / self.w * (self.xmax - self.xmin);
            s.line(px, self.bottom(), px, self.py(f(x).min(self.ymax)), colour, 2.2);
        }
    }
}

/// A vertical dashed line drawn as short strokes.
fn dashed_v(s: &mut Svg, x: f64, y1: f64, y2: f64, colour: &str) {
    let mut y = y1;
    while y < y2 {
        s.line(x, y, x, (y + 5.0).min(y2), colour, 1.5);
        y += 9.0;
    }
}

/// A horizontal dashed line drawn as short strokes.
fn dashed_h(s: &mut Svg, x1: f64, x2: f64, y: f64, colour: &str) {
    let mut x = x1;
    while x < x2 {
        s.line(x, y, (x + 5.0).min(x2), y, colour, 1.5);
        x += 9.0;
    }
}

/// Digits as Unicode superscripts (for powers of ten in labels).
fn superscript(n: u32) -> String {
    n.to_string()
        .chars()
        .map(|d| match d {
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
        })
        .collect()
}

/// Label levels for points on a line: in left-to-right order, a label moves up one level when it would sit closer than
/// `gap` pixels to the previous label on the same level.
fn stagger(px: &[f64], gap: f64) -> Vec<usize> {
    let mut order: Vec<usize> = (0..px.len()).collect();
    order.sort_by(|a, b| px[*a].total_cmp(&px[*b]));
    let mut level = vec![0usize; px.len()];
    let mut last: Vec<f64> = Vec::new();
    for i in order {
        let mut l = 0;
        while l < last.len() && px[i] - last[l] < gap {
            l += 1;
        }
        if l == last.len() {
            last.push(px[i]);
        } else {
            last[l] = px[i];
        }
        level[i] = l;
    }
    level
}

/// A grid of numbers as cells, formatted with `numt` at the given decimals.
fn grid(vals: &[Vec<f64>], dec: usize) -> Vec<Vec<Cell>> {
    vals.iter().map(|r| r.iter().map(|v| cell(numt(*v, dec))).collect()).collect()
}

/// Rows of a column-major table (columns are samples) turned into rows for drawing.
fn rows_of(cols: &[Vec<f64>]) -> Vec<Vec<f64>> {
    (0..cols[0].len()).map(|r| cols.iter().map(|c| c[r]).collect()).collect()
}

// ====================================================================================================
// Chapter 12. Distributions: PMF, PDF, CDF
// ====================================================================================================

fn b12_pmf() -> Figure {
    let p = calc::basket_pmf();
    let total: u32 = calc::BASKET.iter().sum();
    let mut s = Svg::new(
        700,
        300,
        "The probability mass function of how many books a customer buys",
        "Six blue bars over 1 to 6 books with heights equal to the count divided by 40; the heights are printed and add up to one.",
    ).min_text(12);
    let pl = Plot { x0: 70.0, y0: 40.0, w: 400.0, h: 200.0, xmin: 0.5, xmax: 6.5, ymin: 0.0, ymax: 0.45 };
    pl.frame(&mut s, &[0.0, 0.1, 0.2, 0.3, 0.4], 1, &[1.0, 2.0, 3.0, 4.0, 5.0, 6.0], 0, "books bought in one visit", "probability");
    for (i, pk) in p.iter().enumerate() {
        let k = i as f64 + 1.0;
        let (xl, xr) = (pl.px(k - 0.28), pl.px(k + 0.28));
        s.rect(xl, pl.py(*pk), xr - xl, pl.bottom() - pl.py(*pk), tok::S1, None);
        s.text((xl + xr) / 2.0, pl.py(*pk) - 20.0, &numt(*pk, 3), 12, tok::INK, Anchor::Middle);
        s.text((xl + xr) / 2.0, pl.py(*pk) - 7.0, &format!("{} of {}", calc::BASKET[i], total), 11, tok::MUTED, Anchor::Middle);
    }
    let sum: f64 = p.iter().sum();
    let tx = 500.0;
    s.text(tx, 90.0, &format!("{total} customers were counted."), 12, tok::INK, Anchor::Start);
    s.text(tx, 112.0, "Each bar is a probability:", 12, tok::INK, Anchor::Start);
    s.text(tx, 130.0, &format!("a count divided by {total}."), 12, tok::INK, Anchor::Start);
    s.text(tx, 156.0, "Only whole numbers of books", 12, tok::INK, Anchor::Start);
    s.text(tx, 174.0, "are possible, so the chance", 12, tok::INK, Anchor::Start);
    s.text(tx, 192.0, "sits on six separate values.", 12, tok::INK, Anchor::Start);
    s.text(tx, 218.0, &format!("The six heights add up to {}.", numt(sum, 3)), 12, tok::INK, Anchor::Start);
    ("b12-1-pmf".to_string(), s.finish())
}

fn b12_pdf() -> Figure {
    let x = calc::page_sample();
    let truth = calc::Dist::Lognormal(calc::MEDIAN_PAGES.ln(), calc::SIGMA);
    let per100 = |v: f64| truth.pdf(v) * 100.0;
    let (a, b) = (200.0, 300.0);
    let area = truth.cdf(b) - truth.cdf(a);
    let inside = x.iter().filter(|v| **v >= a && **v <= b).count();
    let mut s = Svg::new(
        700,
        320,
        "A probability density: the chance of an interval is the area under the curve",
        "Outlined histogram bars of the 60 sampled page counts drawn as a density, the lognormal density curve in blue, and the area between 200 and 300 pages shaded orange with its value.",
    ).min_text(12);
    let pl = Plot { x0: 70.0, y0: 50.0, w: 440.0, h: 200.0, xmin: 0.0, xmax: 700.0, ymin: 0.0, ymax: 0.5 };
    pl.frame(&mut s, &[0.0, 0.1, 0.2, 0.3, 0.4, 0.5], 1, &[0.0, 100.0, 200.0, 300.0, 400.0, 500.0, 600.0, 700.0], 0, "pages", "density (probability per 100 pages)");
    pl.shade(&mut s, &per100, a, b, tok::S2);
    for bin in 0..14 {
        let lo = bin as f64 * 50.0;
        let count = x.iter().filter(|v| **v >= lo && **v < lo + 50.0).count();
        if count > 0 {
            let hgt = count as f64 / x.len() as f64 / 50.0 * 100.0;
            s.rect(pl.px(lo), pl.py(hgt), pl.px(lo + 50.0) - pl.px(lo), pl.bottom() - pl.py(hgt), "none", Some(tok::INK2));
        }
    }
    pl.curve(&mut s, &per100, 1.0, 700.0, 280, tok::S1, 2.0);
    let (ax, ay) = (pl.px(250.0), pl.py(0.12));
    s.arrow(pl.px(390.0), pl.py(0.3) + 4.0, ax + 6.0, ay, None);
    s.text(pl.px(360.0), pl.py(0.3) - 24.0, &format!("shaded area = {}", num(area, 3)), 12, tok::INK, Anchor::Start);
    s.text(pl.px(360.0), pl.py(0.3) - 8.0, "chance of 200 to 300 pages", 11, tok::INK2, Anchor::Start);
    let tx = 522.0;
    s.text(tx, 70.0, "Bars: the 60 page counts,", 12, tok::INK, Anchor::Start);
    s.text(tx, 88.0, "rescaled so their total", 12, tok::INK, Anchor::Start);
    s.text(tx, 106.0, "area is 1.", 12, tok::INK, Anchor::Start);
    s.text(tx, 132.0, "Curve: lognormal density,", 12, tok::S1, Anchor::Start);
    s.text(tx, 150.0, &format!("median {}, σ = {}.", numt(calc::MEDIAN_PAGES, 0), numt(calc::SIGMA, 2)), 12, tok::S1, Anchor::Start);
    s.text(tx, 176.0, &format!("In the sample, {inside} of 60"), 12, tok::INK, Anchor::Start);
    s.text(tx, 194.0, &format!("books ({}) fall in", num(inside as f64 / 60.0, 3)), 12, tok::INK, Anchor::Start);
    s.text(tx, 212.0, "the shaded range.", 12, tok::INK, Anchor::Start);
    ("b12-2-pdf".to_string(), s.finish())
}

fn b12_cdf() -> Figure {
    let p = calc::basket_pmf();
    let cum: Vec<f64> = p.iter().scan(0.0, |acc, v| {
        *acc += v;
        Some(*acc)
    }).collect();
    let x = calc::page_sample();
    let sx = calc::sorted(&x);
    let truth = calc::Dist::Lognormal(calc::MEDIAN_PAGES.ln(), calc::SIGMA);
    let below300 = x.iter().filter(|v| **v <= 300.0).count();
    let mut s = Svg::new(
        700,
        320,
        "Cumulative distribution functions: a staircase for books bought and a smooth rise for page counts",
        "Left, the cumulative probabilities of books bought per visit as a blue staircase with P(X ≤ 4) marked; right, the sample's cumulative fraction of page counts as a blue step line and the lognormal CDF as an orange curve with the value at 300 pages marked.",
    ).min_text(12);
    let left = Plot { x0: 60.0, y0: 56.0, w: 250.0, h: 180.0, xmin: 0.0, xmax: 7.0, ymin: 0.0, ymax: 1.0 };
    left.frame(&mut s, &[0.0, 0.25, 0.5, 0.75, 1.0], 2, &[1.0, 2.0, 3.0, 4.0, 5.0, 6.0], 0, "books bought", "P(X ≤ x)");
    s.text(left.x0, 24.0, "Books per visit: a staircase", 12, tok::INK, Anchor::Start);
    s.line(left.px(0.0), left.py(0.0), left.px(1.0), left.py(0.0), tok::S1, 2.0);
    for (i, c) in cum.iter().enumerate() {
        let k = i as f64 + 1.0;
        s.line(left.px(k), left.py(*c), left.px(k + 1.0), left.py(*c), tok::S1, 2.0);
        s.dot(left.px(k), left.py(*c), 3.5, tok::S1, None);
    }
    let f4 = cum[3];
    dashed_h(&mut s, left.x0, left.px(4.0), left.py(f4), tok::S2);
    s.text(left.px(4.0) + 8.0, left.py(f4) + 28.0, &format!("P(X ≤ 4) = {}", numt(f4, 3)), 12, tok::INK, Anchor::Start);
    s.text(left.px(4.0) + 8.0, left.py(f4) + 44.0, "= the first four bars", 11, tok::INK2, Anchor::Start);
    s.text(left.px(4.0) + 8.0, left.py(f4) + 58.0, "of Figure 12.1 added", 11, tok::INK2, Anchor::Start);
    let right = Plot { x0: 410.0, y0: 56.0, w: 260.0, h: 180.0, xmin: 0.0, xmax: 700.0, ymin: 0.0, ymax: 1.0 };
    right.frame(&mut s, &[0.0, 0.25, 0.5, 0.75, 1.0], 2, &[0.0, 200.0, 400.0, 600.0], 0, "pages", "P(pages ≤ x)");
    s.text(right.x0, 24.0, "Page counts: a smooth rise", 12, tok::INK, Anchor::Start);
    let mut d = format!("M{} {}", f1(right.px(0.0)), f1(right.py(0.0)));
    for (i, v) in sx.iter().enumerate() {
        let before = i as f64 / sx.len() as f64;
        let after = (i as f64 + 1.0) / sx.len() as f64;
        d.push_str(&format!(" L{} {} L{} {}", f1(right.px(*v)), f1(right.py(before)), f1(right.px(*v)), f1(right.py(after))));
    }
    d.push_str(&format!(" L{} {}", f1(right.px(700.0)), f1(right.py(1.0))));
    s.path(&d, tok::S1, 1.5);
    right.curve(&mut s, &|v| truth.cdf(v), 1.0, 700.0, 200, tok::S2, 2.0);
    let f300 = truth.cdf(300.0);
    dashed_v(&mut s, right.px(300.0), right.py(f300), right.bottom(), tok::INK2);
    s.dot(right.px(300.0), right.py(f300), 3.5, tok::S2, None);
    s.text(right.px(300.0) + 8.0, right.py(f300) + 34.0, &format!("curve at 300: {}", num(f300, 3)), 11, tok::S2, Anchor::Start);
    s.text(right.px(300.0) + 8.0, right.py(f300) + 50.0, &format!("sample: {below300}/60 = {}", num(below300 as f64 / 60.0, 3)), 11, tok::S1, Anchor::Start);
    ("b12-3-cdf".to_string(), s.finish())
}

// ====================================================================================================
// Chapter 13. Which distribution does my data follow?
// ====================================================================================================

/// The four candidates of chapter 13, fitted to the sample, with their names.
fn candidates(x: &[f64]) -> Vec<(&'static str, calc::Dist)> {
    vec![
        ("Normal", calc::fit_normal(x)),
        ("Lognormal", calc::fit_lognormal(x)),
        ("Gamma", calc::fit_gamma(x)),
        ("Weibull", calc::fit_weibull(x)),
    ]
}

fn b13_histogram() -> Figure {
    let x = calc::page_sample();
    let normal = calc::fit_normal(&x);
    let logn = calc::fit_lognormal(&x);
    let (m, sd) = normal.params();
    let (lm, ls) = logn.params();
    let below_zero = normal.cdf(0.0);
    let scale = x.len() as f64 * 50.0;
    let mut s = Svg::new(
        720,
        320,
        "The page counts are skewed to the right; a lognormal curve follows them, a normal curve does not",
        "Pale blue histogram of the 60 page counts in bins of 50 pages, an orange normal curve that spills below zero pages and misses the peak, and a blue lognormal curve that follows the bars and the long right tail.",
    ).min_text(12);
    let pl = Plot { x0: 70.0, y0: 50.0, w: 440.0, h: 200.0, xmin: -100.0, xmax: 700.0, ymin: 0.0, ymax: 14.0 };
    pl.frame(&mut s, &[0.0, 2.0, 4.0, 6.0, 8.0, 10.0, 12.0, 14.0], 0, &[-100.0, 0.0, 100.0, 200.0, 300.0, 400.0, 500.0, 600.0, 700.0], 0, "pages (bins of 50)", "books per bin");
    for bin in 0..14 {
        let lo = bin as f64 * 50.0;
        let count = x.iter().filter(|v| **v >= lo && **v < lo + 50.0).count() as f64;
        if count > 0.0 {
            s.rect(pl.px(lo), pl.py(count), pl.px(lo + 50.0) - pl.px(lo), pl.bottom() - pl.py(count), tok::FILL1, Some(tok::GRID));
        }
    }
    pl.shade(&mut s, &|v| normal.pdf(v) * scale, -100.0, 0.0, tok::S2);
    dashed_v(&mut s, pl.px(0.0), pl.y0, pl.bottom(), tok::INK2);
    pl.curve(&mut s, &|v| normal.pdf(v) * scale, -100.0, 700.0, 300, tok::S2, 2.0);
    pl.curve(&mut s, &|v| logn.pdf(v) * scale, 1.0, 700.0, 300, tok::S1, 2.0);
    s.text(pl.px(0.0) + 6.0, pl.py(13.0), "no book has", 11, tok::INK2, Anchor::Start);
    s.text(pl.px(0.0) + 6.0, pl.py(13.0) + 14.0, "fewer than", 11, tok::INK2, Anchor::Start);
    s.text(pl.px(0.0) + 6.0, pl.py(13.0) + 28.0, "0 pages", 11, tok::INK2, Anchor::Start);
    s.arrow(pl.px(-40.0), pl.py(4.5), pl.px(-40.0), pl.py(0.9), None);
    s.text(pl.px(-40.0), pl.py(4.5) - 6.0, &format!("{}%", num(below_zero * 100.0, 1)), 11, tok::S2, Anchor::Middle);
    s.arrow(pl.px(620.0), pl.py(5.0), pl.px(640.0), pl.py(1.6), None);
    s.text(pl.px(620.0), pl.py(5.0) - 8.0, "long right tail", 11, tok::INK2, Anchor::Middle);
    let tx = 530.0;
    s.line(tx, 76.0, tx + 22.0, 76.0, tok::S2, 2.0);
    s.text(tx + 28.0, 80.0, "normal fit", 12, tok::INK, Anchor::Start);
    s.text(tx + 28.0, 96.0, &format!("mean {}, sd {}", num(m, 0), num(sd, 0)), 11, tok::INK2, Anchor::Start);
    s.line(tx, 124.0, tx + 22.0, 124.0, tok::S1, 2.0);
    s.text(tx + 28.0, 128.0, "lognormal fit", 12, tok::INK, Anchor::Start);
    s.text(tx + 28.0, 144.0, &format!("median {}, σ = {}", num(lm.exp(), 0), num(ls, 2)), 11, tok::INK2, Anchor::Start);
    s.text(tx, 178.0, &format!("The normal curve puts {}%", num(below_zero * 100.0, 1)), 11, tok::INK2, Anchor::Start);
    s.text(tx, 192.0, "of books below 0 pages,", 11, tok::INK2, Anchor::Start);
    s.text(tx, 206.0, "a limit the data respect.", 11, tok::INK2, Anchor::Start);
    ("b13-1-histogram-fits".to_string(), s.finish())
}

fn b13_qq() -> Figure {
    let x = calc::page_sample();
    let sx = calc::sorted(&x);
    let med = calc::filliben(sx.len());
    let mut s = Svg::new(
        700,
        300,
        "Probability plots of the page counts against four fitted distributions",
        "Four square panels; in each, the sorted page counts are plotted against the fitted distribution's quantiles over a pale fat-pencil band; the normal panel bends, the lognormal and gamma panels stay straight, the Weibull panel bends slightly at the top.",
    ).min_text(12);
    for (i, (name, d)) in candidates(&x).into_iter().enumerate() {
        let pl = Plot { x0: 62.0 + i as f64 * 164.0, y0: 66.0, w: 128.0, h: 128.0, xmin: -100.0, xmax: 800.0, ymin: -100.0, ymax: 800.0 };
        let a2 = calc::anderson_darling(&sx, &d);
        s.text(pl.x0 + pl.w / 2.0, 28.0, name, 12, tok::INK, Anchor::Middle);
        let stat = match d {
            calc::Dist::Normal(..) => {
                let p = calc::ad_pvalue_normal(calc::ad_adjusted(a2, sx.len()));
                format!("A² = {}, p = {}", num(a2, 2), num(p, 3))
            }
            calc::Dist::Lognormal(m, sd) => {
                let logs: Vec<f64> = sx.iter().map(|v| v.ln()).collect();
                let a2l = calc::anderson_darling(&logs, &calc::Dist::Normal(m, sd));
                let p = calc::ad_pvalue_normal(calc::ad_adjusted(a2l, sx.len()));
                format!("A² = {}, p = {}", num(a2, 2), num(p, 2))
            }
            _ => format!("A² = {}", num(a2, 2)),
        };
        s.text(pl.x0 + pl.w / 2.0, 46.0, &stat, 11, tok::INK2, Anchor::Middle);
        pl.frame(&mut s, &[0.0, 300.0, 600.0], 0, &[0.0, 300.0, 600.0], 0, "model quantile", "");
        s.line(pl.px(-100.0), pl.py(-100.0), pl.px(800.0), pl.py(800.0), tok::FILL1, 14.0);
        s.line(pl.px(-100.0), pl.py(-100.0), pl.px(800.0), pl.py(800.0), tok::AXIS, 1.0);
        for (obs, m) in sx.iter().zip(&med) {
            let q = d.ppf(*m);
            s.dot(pl.px(q), pl.py(*obs), 2.4, tok::S1, None);
        }
    }
    s.text(350.0, 268.0, "Each dot is one book: its page count (up) against where the fitted model puts the same rank (across).", 11, tok::INK2, Anchor::Middle);
    s.text(350.0, 283.0, "The pale band is the fat pencil: dots that stay inside it fit.", 11, tok::INK2, Anchor::Middle);
    ("b13-2-qq-plots".to_string(), s.finish())
}

fn b13_threshold() -> Figure {
    let x = calc::page_sample();
    let sx = calc::sorted(&x);
    let two = calc::fit_lognormal(&x);
    let three = calc::fit_lognormal3(&x, calc::BOOK_FLOOR);
    let (a2_two, a2_three) = (calc::anderson_darling(&sx, &two), calc::anderson_darling(&sx, &three));
    let mut s = Svg::new(
        720,
        310,
        "A threshold parameter makes the density exactly zero below a chosen smallest value",
        "Two fitted density curves over page counts: a blue two-parameter lognormal that continues towards zero pages and an orange three-parameter lognormal that is zero up to a dashed threshold at 49 pages; tick marks along the axis show the 60 books.",
    ).min_text(12);
    let pl = Plot { x0: 70.0, y0: 50.0, w: 440.0, h: 200.0, xmin: 0.0, xmax: 700.0, ymin: 0.0, ymax: 0.5 };
    pl.frame(&mut s, &[0.0, 0.1, 0.2, 0.3, 0.4, 0.5], 1, &[0.0, 100.0, 200.0, 300.0, 400.0, 500.0, 600.0, 700.0], 0, "pages", "density (probability per 100 pages)");
    for v in &sx {
        s.line(pl.px(*v), pl.bottom() - 9.0, pl.px(*v), pl.bottom(), tok::INK2, 1.0);
    }
    dashed_v(&mut s, pl.px(calc::BOOK_FLOOR), pl.y0 + 20.0, pl.bottom(), tok::S2);
    pl.curve(&mut s, &|v| two.pdf(v) * 100.0, 1.0, 700.0, 300, tok::S1, 2.0);
    pl.curve(&mut s, &|v| three.pdf(v) * 100.0, 0.0, 700.0, 350, tok::S2, 2.0);
    s.text(pl.px(calc::BOOK_FLOOR) - 4.0, pl.y0 + 12.0, &format!("threshold θ = {} pages", numt(calc::BOOK_FLOOR, 0)), 11, tok::S2, Anchor::Start);
    s.arrow(pl.px(150.0), pl.py(0.12), pl.px(sx[0]) + 2.0, pl.bottom() - 12.0, None);
    s.text(pl.px(150.0) + 4.0, pl.py(0.12) - 4.0, "shortest book:", 11, tok::INK2, Anchor::Start);
    s.text(pl.px(150.0) + 4.0, pl.py(0.12) + 10.0, &format!("{} pages", numt(sx[0], 0)), 11, tok::INK2, Anchor::Start);
    let tx = 520.0;
    s.line(tx, 76.0, tx + 22.0, 76.0, tok::S1, 2.0);
    s.text(tx + 28.0, 80.0, "two parameters", 12, tok::INK, Anchor::Start);
    s.text(tx + 28.0, 96.0, &format!("A² = {}", num(a2_two, 2)), 11, tok::INK2, Anchor::Start);
    s.line(tx, 124.0, tx + 22.0, 124.0, tok::S2, 2.0);
    s.text(tx + 28.0, 128.0, "three parameters,", 12, tok::INK, Anchor::Start);
    s.text(tx + 28.0, 144.0, &format!("θ = {} fixed: A² = {}", numt(calc::BOOK_FLOOR, 0), num(a2_three, 2)), 11, tok::INK2, Anchor::Start);
    s.text(tx, 178.0, "The orange curve is zero", 11, tok::INK2, Anchor::Start);
    s.text(tx, 192.0, "below its threshold; the", 11, tok::INK2, Anchor::Start);
    s.text(tx, 206.0, "blue one allows any length", 11, tok::INK2, Anchor::Start);
    s.text(tx, 220.0, "above 0. Here the extra", 11, tok::INK2, Anchor::Start);
    s.text(tx, 234.0, "parameter fits worse.", 11, tok::INK2, Anchor::Start);
    ("b13-3-threshold".to_string(), s.finish())
}

fn b13_decision() -> Figure {
    let x = calc::page_sample();
    let sx = calc::sorted(&x);
    let fits = candidates(&x);
    let normal = fits[0].1;
    let (logn_m, logn_s) = fits[1].1.params();
    let p_normal = calc::ad_pvalue_normal(calc::ad_adjusted(calc::anderson_darling(&sx, &normal), sx.len()));
    let logs: Vec<f64> = sx.iter().map(|v| v.ln()).collect();
    let p_logn = calc::ad_pvalue_normal(calc::ad_adjusted(calc::anderson_darling(&logs, &calc::Dist::Normal(logn_m, logn_s)), sx.len()));
    let steps: [(&str, &str, String); 6] = [
        ("1. Draw the histogram", "shape, skew, natural limits", format!("skewed right; {} to {} pages, none below 0", numt(sx[0], 0), numt(sx[sx.len() - 1], 0))),
        ("2. Shortlist candidates", "shapes that can skew right", "normal (as a check), lognormal, gamma, Weibull".to_string()),
        ("3. Test each fit", "Anderson–Darling; a high p keeps it", format!("normal p = {}, out; lognormal p = {}, kept", num(p_normal, 3), num(p_logn, 2))),
        ("4. Read the probability plots", "do the dots stay in the fat pencil?", "lognormal and gamma: yes; normal: no".to_string()),
        ("5. A third parameter?", "only for a real smallest value", format!("no floor near the data; θ = {} fits worse", numt(calc::BOOK_FLOOR, 0))),
        ("6. Keep the simplest that fits", "a parsimonious description", format!("lognormal: median {} pages, σ = {}", num(logn_m.exp(), 0), num(logn_s, 2))),
    ];
    let mut s = Svg::new(
        700,
        400,
        "A decision path for choosing a distribution, with the page-count sample's result at each step",
        "Six boxes joined by arrows down the left, from drawing the histogram to keeping the simplest distribution that fits, and beside each box the outcome for the 60 page counts.",
    ).min_text(12);
    s.text(40.0, 26.0, "Step", 12, tok::INK2, Anchor::Start);
    s.text(366.0, 26.0, "What it gave for the 60 page counts", 12, tok::INK2, Anchor::Start);
    for (i, (title, detail, verdict)) in steps.iter().enumerate() {
        let y = 40.0 + i as f64 * 58.0;
        s.labelled_box2(40.0, y, 300.0, 44.0, title, detail, tok::NEUTRAL, tok::GRID);
        s.text(366.0, y + 26.0, verdict, 12, tok::INK, Anchor::Start);
        s.line(344.0, y + 22.0, 358.0, y + 22.0, tok::AXIS, 1.0);
        if i + 1 < steps.len() {
            s.arrow(190.0, y + 45.0, 190.0, y + 57.0, None);
        }
    }
    ("b13-4-decision-path".to_string(), s.finish())
}

// ====================================================================================================
// Chapter 14. Features: telling categories from numbers, and scaling
// ====================================================================================================

fn b14_add_test() -> Figure {
    let pages = [312.0, 148.0];
    let copies = [3.0, 2.0];
    let price = [250.0, 180.0];
    let genre_sum = 1 + 2;
    let rows: [(&str, &str, String, &str, bool); 6] = [
        ("pages", "312, 148, 96, …", format!("{} + {} = {} pages", numt(pages[0], 0), numt(pages[1], 0), numt(pages[0] + pages[1], 0)), "a number", true),
        ("copies in stock", "0, 1, 2, 3", format!("{} + {} = {} copies", numt(copies[0], 0), numt(copies[1], 0), numt(copies[0] + copies[1], 0)), "a number (few values)", true),
        ("price (rupees)", "250, 180, 90, …", format!("{} + {} = {} rupees", numt(price[0], 0), numt(price[1], 0), numt(price[0] + price[1], 0)), "a number", true),
        ("condition", "fine, good, fair, poor", "fine + fair = ?".to_string(), "a category (ordered)", false),
        ("genre code", "1 fiction, 2 history, 3 travel", format!("1 + 2 = {genre_sum}: travel?"), "a category", false),
        ("ISBN", "13-digit codes: 978…", "the sum names no book".to_string(), "a category (a label)", false),
    ];
    let mut s = Svg::new(
        700,
        330,
        "The addition test: a column is a number only if adding two of its values means something",
        "A six-row table of bookshop columns with example values, the result of adding two values, and the verdict; pages, copies and price add up meaningfully, while condition, genre code and ISBN do not.",
    ).min_text(12);
    let cols = [(24.0, "column"), (150.0, "values you see"), (350.0, "add two of them"), (522.0, "so it is")];
    for (x, h) in cols {
        s.text_bold(x, 34.0, h, 12, tok::INK, Anchor::Start);
    }
    s.line(20.0, 44.0, 680.0, 44.0, tok::AXIS, 1.0);
    for (i, (name, vals, sum, verdict, numeric)) in rows.iter().enumerate() {
        let y = 50.0 + i as f64 * 42.0;
        let fill = if *numeric { tok::FILL1 } else { tok::NEUTRAL };
        s.rect(20.0, y, 660.0, 36.0, fill, None);
        s.text(24.0, y + 23.0, name, 12, tok::INK, Anchor::Start);
        s.text(150.0, y + 23.0, vals, 12, tok::INK2, Anchor::Start);
        s.text(350.0, y + 23.0, sum, 12, tok::INK, Anchor::Start);
        s.text_bold(522.0, y + 23.0, verdict, 12, if *numeric { tok::S1 } else { tok::S2 }, Anchor::Start);
    }
    s.text(20.0, 316.0, "Blue rows pass the test; grey rows fail it, whatever the column's storage type says.", 11, tok::INK2, Anchor::Start);
    ("b14-1-add-test".to_string(), s.finish())
}

fn b14_scalers() -> Figure {
    let a = calc::AGES.to_vec();
    let (mn, mx) = (a.iter().copied().fold(f64::INFINITY, f64::min), a.iter().copied().fold(f64::NEG_INFINITY, f64::max));
    let m = calc::mean(&a);
    let sd = calc::sd_pop(&a);
    let med = calc::quantile(&a, 0.5);
    let (q1, q3) = (calc::quantile(&a, 0.25), calc::quantile(&a, 0.75));
    let iqr = q3 - q1;
    let fence = q3 + 1.5 * iqr;
    let rows: Vec<(&str, String, Vec<f64>)> = vec![
        ("min–max", "(x − min) / (max − min)".to_string(), a.iter().map(|v| (v - mn) / (mx - mn)).collect()),
        ("z-score", format!("(x − {}) / {}", num(m, 2), num(sd, 2)), a.iter().map(|v| (v - m) / sd).collect()),
        ("robust", format!("(x − {}) / {}", numt(med, 2), numt(iqr, 2)), a.iter().map(|v| (v - med) / iqr).collect()),
    ];
    let mut s = Svg::new(
        720,
        360,
        "One outlier under three scalers: min–max and z-scores crowd the other seven ages, the robust scaler keeps them spread",
        "Top, the raw ages on a line from 0 to 40 years with the interquartile fence marked; below, the scaled values of min–max, z-score and robust scaling on one shared axis from −1 to 3, the seven ordinary books in blue and the 40-year-old book in orange.",
    ).min_text(12);
    // raw row
    let raw = Plot { x0: 180.0, y0: 40.0, w: 440.0, h: 30.0, xmin: 0.0, xmax: 40.0, ymin: 0.0, ymax: 1.0 };
    s.text_bold(20.0, 62.0, "raw ages", 12, tok::INK, Anchor::Start);
    s.text(20.0, 78.0, "years", 11, tok::INK2, Anchor::Start);
    s.line(raw.x0, 60.0, raw.x0 + raw.w, 60.0, tok::AXIS, 1.0);
    for t in [0.0, 10.0, 20.0, 30.0, 40.0] {
        s.line(raw.px(t), 60.0, raw.px(t), 64.0, tok::AXIS, 1.0);
        s.text(raw.px(t), 78.0, &numt(t, 0), 11, tok::MUTED, Anchor::Middle);
    }
    dashed_v(&mut s, raw.px(fence), 36.0, 66.0, tok::INK2);
    s.text(raw.px(fence) + 6.0, 38.0, &format!("IQR fence {} = {} + 1.5 × {}", numt(fence, 3), numt(q3, 2), numt(iqr, 2)), 11, tok::INK2, Anchor::Start);
    let mut seen: Vec<f64> = Vec::new();
    for v in &a {
        let stack = seen.iter().filter(|w| (**w - *v).abs() < 1e-9).count() as f64;
        seen.push(*v);
        let colour = if *v > fence { tok::S2 } else { tok::S1 };
        s.dot(raw.px(*v), 60.0 - 8.0 * stack, 4.0, colour, Some(tok::SURFACE));
    }
    // scaled rows on one axis
    let sc = Plot { x0: 180.0, y0: 110.0, w: 440.0, h: 200.0, xmin: -1.0, xmax: 3.0, ymin: 0.0, ymax: 1.0 };
    for t in [-1.0, 0.0, 1.0, 2.0, 3.0] {
        s.line(sc.px(t), 104.0, sc.px(t), 300.0, tok::GRID, 1.0);
        s.text(sc.px(t), 318.0, &numt(t, 0), 11, tok::MUTED, Anchor::Middle);
    }
    s.text(sc.px(1.0), 336.0, "scaled value (one shared axis)", 11, tok::INK2, Anchor::Middle);
    for (i, (name, formula, vals)) in rows.iter().enumerate() {
        let y = 140.0 + i as f64 * 62.0;
        s.text_bold(20.0, y + 4.0, name, 12, tok::INK, Anchor::Start);
        s.text(20.0, y + 20.0, formula, 11, tok::INK2, Anchor::Start);
        s.line(sc.x0, y, sc.x0 + sc.w, y, tok::AXIS, 1.0);
        let inl: Vec<f64> = vals.iter().zip(&a).filter(|(_, v)| **v <= fence).map(|(z, _)| *z).collect();
        let (lo, hi) = (inl.iter().copied().fold(f64::MAX, f64::min), inl.iter().copied().fold(f64::MIN, f64::max));
        let mut seen: Vec<f64> = Vec::new();
        for (z, v) in vals.iter().zip(&a) {
            let stack = seen.iter().filter(|w| (**w - *z).abs() < 1e-9).count() as f64;
            seen.push(*z);
            if *z <= 3.0 {
                let colour = if *v > fence { tok::S2 } else { tok::S1 };
                s.dot(sc.px(*z), y - 8.0 * stack, 3.5, colour, Some(tok::SURFACE));
            } else {
                s.arrow(sc.px(2.7), y, sc.px(3.0) + 8.0, y, None);
                s.dot(sc.px(3.0) + 14.0, y, 4.0, tok::S2, Some(tok::SURFACE));
            }
        }
        let out = vals[vals.len() - 1];
        s.text(628.0, y - 10.0, &format!("40 → {}", num(out, 2)), 11, tok::S2, Anchor::Start);
        s.text(sc.px(lo), y + 20.0, &format!("seven books: {} to {}", num(lo, 2), num(hi, 2)), 11, tok::S1, Anchor::Start);
    }
    ("b14-2-scalers".to_string(), s.finish())
}

fn b14_log() -> Figure {
    let runs = calc::PRINT_RUNS;
    let mut s = Svg::new(
        700,
        260,
        "A log scale spreads values that span orders of magnitude",
        "Two number lines for six print runs: on the ordinary line four dots crowd near zero, one sits at 300,000 and one at two million; on the log line the six dots spread out, one step per factor of ten.",
    ).min_text(12);
    let lin = Plot { x0: 60.0, y0: 40.0, w: 580.0, h: 10.0, xmin: 0.0, xmax: 2_000_000.0, ymin: 0.0, ymax: 1.0 };
    s.text_bold(60.0, 30.0, "copies printed, ordinary scale", 12, tok::INK, Anchor::Start);
    s.line(lin.x0, 70.0, lin.x0 + lin.w, 70.0, tok::AXIS, 1.0);
    for t in [0.0, 500_000.0, 1_000_000.0, 1_500_000.0, 2_000_000.0] {
        s.line(lin.px(t), 70.0, lin.px(t), 74.0, tok::AXIS, 1.0);
        s.text(lin.px(t), 88.0, &thousands(t as u64), 11, tok::MUTED, Anchor::Middle);
    }
    let crowd = runs.iter().filter(|v| lin.px(**v) - lin.x0 < 60.0).count();
    for v in &runs {
        s.dot(lin.px(*v), 70.0, 4.5, tok::S1, Some(tok::SURFACE));
    }
    s.text(lin.x0 + 8.0, 56.0, &format!("{crowd} titles squeezed into this corner (their dots overlap)"), 11, tok::INK2, Anchor::Start);
    let lg = Plot { x0: 60.0, y0: 150.0, w: 580.0, h: 10.0, xmin: 2.0, xmax: 7.0, ymin: 0.0, ymax: 1.0 };
    s.text_bold(60.0, 122.0, "the same copies, log scale (log₁₀ of copies)", 12, tok::INK, Anchor::Start);
    s.line(lg.x0, 170.0, lg.x0 + lg.w, 170.0, tok::AXIS, 1.0);
    for t in 2..=7 {
        let t = f64::from(t);
        s.line(lg.px(t), 170.0, lg.px(t), 174.0, tok::AXIS, 1.0);
        s.text(lg.px(t), 188.0, &format!("{} (10{})", numt(t, 0), superscript(t as u32)), 11, tok::MUTED, Anchor::Middle);
    }
    for (i, v) in runs.iter().enumerate() {
        let l = v.log10();
        s.dot(lg.px(l), 170.0, 4.5, tok::S1, Some(tok::SURFACE));
        let dy = if i % 2 == 0 { -12.0 } else { -26.0 };
        s.text(lg.px(l), 170.0 + dy, &thousands(*v as u64), 11, tok::INK, Anchor::Middle);
    }
    let d43 = (10f64.powi(4) - 10f64.powi(3), 10f64.powi(4).log10() - 10f64.powi(3).log10());
    s.text(60.0, 228.0, &format!("On the ordinary scale 10,000 − 1,000 = {}; on the log scale the pair differs by {} − {} = {}.", thousands(d43.0 as u64), numt(4.0, 0), numt(3.0, 0), numt(d43.1, 0)), 11, tok::INK2, Anchor::Start);
    s.text(60.0, 244.0, "Each factor of ten is one equal step, so small and huge print runs share one readable line.", 11, tok::INK2, Anchor::Start);
    ("b14-3-log-squeeze".to_string(), s.finish())
}

fn b14_bowls() -> Figure {
    let x = calc::page_sample();
    let var_pages = calc::sd_pop(&x).powi(2);
    let var_age = calc::sd_pop(&calc::AGES).powi(2);
    let ratio = var_pages / var_age;
    let start_raw = ((var_age / var_pages).sqrt() * 0.7071, 0.7071);
    let (raw_path, raw_steps) = calc::descent(var_pages, var_age, start_raw, 40);
    let (std_path, std_steps) = calc::descent(1.0, 1.0, (0.7071, 0.7071), 40);
    let mut s = Svg::new(
        700,
        350,
        "Gradient descent in an elongated bowl zigzags; in a round bowl it goes straight to the bottom",
        "Left, contour ellipses, drawn to scale, of a stylised two-weight error bowl whose curvatures are the variances of the raw page counts and raw ages, very long and thin, with an orange descent path bouncing across the narrow valley; right, circular contours after standardising both columns and a single orange step to the centre.",
    ).min_text(12);
    // left panel: w2 (age weight) horizontal, w1 (pages weight) vertical, equal scales
    let (lx, ly, unit) = (190.0, 150.0, 150.0);
    s.text_bold(40.0, 30.0, "raw columns: pages and age", 12, tok::INK, Anchor::Start);
    s.text(40.0, 48.0, &format!("curvature ratio {} : 1", num(ratio, 0)), 11, tok::INK2, Anchor::Start);
    for level in [1.0f64, 0.6, 0.3, 0.1] {
        let (ax, ay) = (level.sqrt(), level.sqrt() * (var_age / var_pages).sqrt());
        let mut d = String::new();
        for k in 0..=72 {
            let t = k as f64 / 72.0 * std::f64::consts::TAU;
            d.push_str(&format!("{}{} {}", if k == 0 { "M" } else { " L" }, f1(lx + unit * ax * t.cos()), f1(ly - unit * ay * t.sin())));
        }
        s.path(&d, tok::AXIS, 1.0);
    }
    let mut d = String::new();
    for (k, (w1, w2)) in raw_path.iter().enumerate() {
        d.push_str(&format!("{}{} {}", if k == 0 { "M" } else { " L" }, f1(lx + unit * w2), f1(ly - unit * w1)));
    }
    s.path(&d, tok::S2, 1.5);
    s.dot(lx + unit * raw_path[0].1, ly - unit * raw_path[0].0, 3.5, tok::S2, None);
    s.dot(lx, ly, 3.0, tok::INK, None);
    s.text(lx, ly + 44.0, "age weight →", 11, tok::MUTED, Anchor::Middle);
    s.text(lx - unit, ly - 30.0, "↕ pages weight", 11, tok::MUTED, Anchor::Start);
    s.text(lx, 250.0, &format!("first {} steps shown", raw_path.len() - 1), 11, tok::INK2, Anchor::Middle);
    s.text_bold(lx, 276.0, &format!("{raw_steps} steps to get within 1% of the bottom"), 12, tok::S2, Anchor::Middle);
    // right panel
    let (rx, ry, runit) = (540.0, 150.0, 100.0);
    s.text_bold(420.0, 30.0, "both columns standardised", 12, tok::INK, Anchor::Start);
    s.text(420.0, 48.0, "curvature ratio 1 : 1", 11, tok::INK2, Anchor::Start);
    for level in [1.0f64, 0.6, 0.3, 0.1] {
        let r = level.sqrt() * runit;
        let mut d = String::new();
        for k in 0..=72 {
            let t = k as f64 / 72.0 * std::f64::consts::TAU;
            d.push_str(&format!("{}{} {}", if k == 0 { "M" } else { " L" }, f1(rx + r * t.cos()), f1(ry - r * t.sin())));
        }
        s.path(&d, tok::AXIS, 1.0);
    }
    let (sw1, sw2) = std_path[0];
    s.dot(rx + runit * sw2, ry - runit * sw1, 3.5, tok::S2, None);
    s.arrow(rx + runit * sw2, ry - runit * sw1, rx + 4.0, ry - 4.0, None);
    s.dot(rx, ry, 3.0, tok::INK, None);
    s.text_bold(rx, 276.0, &format!("{std_steps} step to get within 1% of the bottom"), 12, tok::S2, Anchor::Middle);
    s.text(350.0, 316.0, "A stylised two-weight bowl: its curvatures are the two columns' variances, drawn to scale.", 11, tok::INK2, Anchor::Middle);
    s.text(350.0, 332.0, "Each run uses the best single learning rate for its bowl; the centre dot is the lowest error.", 11, tok::INK2, Anchor::Middle);
    ("b14-4-bowls".to_string(), s.finish())
}

// ====================================================================================================
// Chapter 15. Quantile normalisation
// ====================================================================================================

/// Draws a books × reviewers grid with reviewer names above and, optionally, book labels on the left.
fn score_grid(s: &mut Svg, x: f64, y: f64, cw: f64, rows: &[Vec<Cell>], names: &[&str], book_labels: bool) {
    for (j, n) in names.iter().enumerate() {
        s.text(x + j as f64 * cw + cw / 2.0, y - 8.0, n, 11, tok::INK2, Anchor::Middle);
    }
    if book_labels {
        for r in 0..rows.len() {
            s.text(x - 8.0, y + r as f64 * cw + cw / 2.0 + 4.0, &format!("book {}", r + 1), 11, tok::INK2, Anchor::End);
        }
    }
    s.cells(x, y, cw, rows);
}

fn b15_steps() -> Figure {
    let cols = calc::reviewers();
    let qn = calc::quantile_normalise(&cols);
    let names = ["R1", "R2", "R3"];
    let cw = 34.0;
    let mut s = Svg::new(
        720,
        320,
        "Quantile normalisation in four steps on three reviewers' scores",
        "Four grids of five books by three reviewers: the raw scores, each reviewer's scores sorted, each sorted row replaced by its mean, and the means put back in each reviewer's original order, with the tied value of reviewer 2 outlined.",
    ).min_text(12);
    let xs = [70.0, 262.0, 418.0, 574.0];
    let titles = ["1. raw scores", "2. sort each reviewer", "3. average each row", "4. restore each order"];
    for (i, t) in titles.iter().enumerate() {
        s.text_bold(xs[i] + 1.5 * cw, 30.0, t, 12, tok::INK, Anchor::Middle);
    }
    let raw = rows_of(&cols);
    let raw_cells: Vec<Vec<Cell>> = raw.iter().map(|r| r.iter().map(|v| cell(numt(*v, 0)).fill(heat(*v, 8.0))).collect()).collect();
    score_grid(&mut s, xs[0], 64.0, cw, &raw_cells, &names, true);
    let sorted_rows = rows_of(&qn.sorted_cols);
    let sorted_cells: Vec<Vec<Cell>> = sorted_rows.iter().map(|r| r.iter().map(|v| cell(numt(*v, 0)).fill(heat(*v, 8.0))).collect()).collect();
    score_grid(&mut s, xs[1], 64.0, cw, &sorted_cells, &names, false);
    for r in 0..5 {
        s.text(xs[1] - 6.0, 64.0 + r as f64 * cw + cw / 2.0 + 4.0, &format!("rank {}", r + 1), 11, tok::MUTED, Anchor::End);
    }
    let mean_cells: Vec<Vec<Cell>> = qn.rank_means.iter().map(|m| (0..3).map(|_| cell(numt(*m, 2)).fill(heat(*m, 8.0))).collect()).collect();
    score_grid(&mut s, xs[2], 64.0, cw, &mean_cells, &names, false);
    let out_rows = rows_of(&qn.out);
    let out_cells: Vec<Vec<Cell>> = out_rows.iter().map(|r| r.iter().map(|v| cell(numt(*v, 2)).fill(heat(*v, 8.0))).collect()).collect();
    score_grid(&mut s, xs[3], 64.0, cw, &out_cells, &names, false);
    for (r, row) in out_rows.iter().enumerate() {
        for (j, v) in row.iter().enumerate() {
            if !qn.rank_means.iter().any(|m| (m - v).abs() < 1e-9) {
                s.rect_bold(xs[3] + j as f64 * cw, 64.0 + r as f64 * cw, cw, cw, "none", tok::S2);
            }
        }
    }
    for i in 0..3 {
        s.arrow(xs[i] + 3.0 * cw + 6.0, 64.0 + 2.5 * cw, xs[i + 1] - (if i == 0 { 50.0 } else { 8.0 }), 64.0 + 2.5 * cw, None);
    }
    let tie = qn.out[1][0];
    s.text(390.0, 270.0, "R2 gave two books a 4 (ranks 4 and 5),", 11, tok::S2, Anchor::Start);
    s.text(390.0, 285.0, &format!("so both get ({} + {}) / 2 = {}.", numt(qn.rank_means[3], 2), numt(qn.rank_means[4], 2), numt(tie, 2)), 11, tok::S2, Anchor::Start);
    s.text(70.0, 270.0, &format!("Row means: {}.", qn.rank_means.iter().map(|m| numt(*m, 2)).collect::<Vec<_>>().join(", ")), 11, tok::INK2, Anchor::Start);
    s.text(70.0, 285.0, "Every reviewer now uses the same five values.", 11, tok::INK2, Anchor::Start);
    ("b15-1-four-steps".to_string(), s.finish())
}

/// The critics-and-fans grid with a gap column (fans minus critics) drawn at (x, y).
fn gap_block(s: &mut Svg, x: f64, y: f64, cols: &[Vec<f64>], dec: usize, title: &str, book_labels: bool, truth: &[f64]) {
    let cw = 34.0;
    let rows = rows_of(cols);
    let cells: Vec<Vec<Cell>> = rows.iter().map(|r| r.iter().map(|v| cell(numt(*v, dec)).fill(heat(*v, 8.0))).collect()).collect();
    s.text_bold(x + 2.0 * cw + 30.0, y - 34.0, title, 12, tok::INK, Anchor::Middle);
    score_grid(s, x, y, cw, &cells, &["C1", "C2", "F1", "F2"], book_labels);
    s.line(x + 2.0 * cw, y - 20.0, x + 2.0 * cw, y + 5.0 * cw + 4.0, tok::INK2, 1.5);
    let gap = calc::fan_gap(cols);
    let gx = x + 4.0 * cw + 14.0;
    s.text(gx + 30.0, y - 8.0, "fans − critics", 11, tok::INK2, Anchor::Middle);
    for (r, g) in gap.iter().enumerate() {
        let wrong = (g - truth[r]).abs() > 1e-9;
        let c = cell(numt(*g, 3)).fill(if wrong { tok::NEUTRAL } else { tok::FILL1 }).ink(if wrong { tok::S2 } else { tok::INK });
        s.cells_wh(gx, y + r as f64 * cw, 60.0, cw, &[vec![c]]);
        if wrong {
            s.rect_bold(gx, y + r as f64 * cw, 60.0, cw, "none", tok::S2);
        }
    }
}

fn b15_erased() -> Figure {
    let raw = calc::critics_and_fans();
    let truth = calc::fan_gap(&raw);
    let all = calc::quantile_normalise(&raw);
    let mut s = Svg::new(
        720,
        330,
        "Quantile normalisation over all reviewers erases a real difference and invents a false one",
        "Left, raw scores of two critics and two fans for five books with a fans-minus-critics column reading 3, 3, 3, 3 and 0; right, the same grid after quantile normalisation of all four columns, where the gaps shrink towards zero and book 5 gains a false negative gap, outlined in orange.",
    ).min_text(12);
    gap_block(&mut s, 80.0, 84.0, &raw, 0, "raw scores", true, &truth);
    gap_block(&mut s, 430.0, 84.0, &all.out, 3, "after quantile normalisation of all four", false, &truth);
    s.text(80.0, 290.0, "The fans really do score books 1 to 4 three points higher, and book 5 the same (left).", 11, tok::INK2, Anchor::Start);
    s.text(80.0, 306.0, "Normalising all four together removes most of that gap and creates one for book 5 (right).", 11, tok::INK2, Anchor::Start);
    ("b15-2-signal-erased".to_string(), s.finish())
}

fn b15_class_specific() -> Figure {
    let raw = calc::critics_and_fans();
    let truth = calc::fan_gap(&raw);
    let all = calc::quantile_normalise(&raw);
    let critics = calc::quantile_normalise(&raw[0..2]);
    let fans = calc::quantile_normalise(&raw[2..4]);
    let class_specific: Vec<Vec<f64>> = critics.out.iter().chain(fans.out.iter()).cloned().collect();
    let mut s = Svg::new(
        720,
        330,
        "Class-specific quantile normalisation keeps the real difference between critics and fans",
        "Left, the grid after normalising the critics together and the fans together, with the fans-minus-critics column back at 3, 3, 3, 3 and 0; right, a table comparing the gaps in the raw scores, after normalising all columns, and after class-specific normalisation.",
    ).min_text(12);
    gap_block(&mut s, 80.0, 84.0, &class_specific, 2, "normalised within each group", true, &truth);
    let gaps = [("raw", truth.clone()), ("all", calc::fan_gap(&all.out)), ("class-specific", calc::fan_gap(&class_specific))];
    let (tx, ty, cw, ch) = (470.0, 84.0, 74.0, 34.0);
    s.text_bold(tx + 1.5 * cw, 50.0, "fans − critics, three ways", 12, tok::INK, Anchor::Middle);
    for (j, (name, g)) in gaps.iter().enumerate() {
        s.text(tx + j as f64 * cw + cw / 2.0, ty - 8.0, name, 11, tok::INK2, Anchor::Middle);
        for (r, v) in g.iter().enumerate() {
            let wrong = (v - truth[r]).abs() > 1e-9;
            let c = cell(numt(*v, 3)).fill(if wrong { tok::NEUTRAL } else { tok::FILL1 }).ink(if wrong { tok::S2 } else { tok::INK });
            s.cells_wh(tx + j as f64 * cw, ty + r as f64 * ch, cw, ch, &[vec![c]]);
        }
    }
    for r in 0..5 {
        s.text(tx - 8.0, ty + r as f64 * ch + ch / 2.0 + 4.0, &format!("book {}", r + 1), 11, tok::INK2, Anchor::End);
    }
    s.text(80.0, 290.0, "Normalising within each group still evens out the harsher marker in each group (C2 and F2),", 11, tok::INK2, Anchor::Start);
    s.text(80.0, 306.0, "but it never pits critics against fans, so the real gap survives and book 5 stays level.", 11, tok::INK2, Anchor::Start);
    ("b15-3-class-specific".to_string(), s.finish())
}

// ====================================================================================================
// Chapter 16. Batch normalisation
// ====================================================================================================

fn b16_where() -> Figure {
    let mut s = Svg::new(
        720,
        250,
        "Where batch normalisation sits: between a layer's weighted sums and its activation",
        "Five boxes left to right: the inputs of one book, the weighted sum, the batch-normalisation step with its two stages, the ReLU activation, and the next layer; notes below say which statistics are used in training and in use.",
    ).min_text(12);
    let y = 70.0;
    let boxes: [(f64, f64, &str, &str); 5] = [
        (20.0, 110.0, "inputs", "pages, price, age"),
        (160.0, 110.0, "weighted sum", "z = w·x"),
        (300.0, 190.0, "batch norm", "(z − μ) / σ, then γ·ẑ + β"),
        (520.0, 80.0, "ReLU", "max(0, y)"),
        (630.0, 80.0, "next", "layer"),
    ];
    for (x, w, a, b) in boxes {
        let fill = if a == "batch norm" { tok::FILL1 } else { tok::NEUTRAL };
        let stroke = if a == "batch norm" { tok::S1 } else { tok::GRID };
        s.labelled_box2(x, y, w, 56.0, a, b, fill, stroke);
    }
    for (x1, x2) in [(130.0, 160.0), (270.0, 300.0), (490.0, 520.0), (600.0, 630.0)] {
        s.arrow(x1 + 2.0, y + 28.0, x2 - 2.0, y + 28.0, None);
    }
    s.text(395.0, y - 14.0, "one unit, one mini-batch", 11, tok::INK2, Anchor::Middle);
    s.text_bold(300.0, 160.0, "In training:", 12, tok::INK, Anchor::Start);
    s.text(300.0, 178.0, "μ and σ come from the current mini-batch.", 12, tok::INK, Anchor::Start);
    s.text_bold(300.0, 204.0, "In use:", 12, tok::INK, Anchor::Start);
    s.text(300.0, 222.0, "μ and σ are averages kept during training.", 12, tok::INK, Anchor::Start);
    s.text(20.0, 160.0, "γ and β are learned", 12, tok::INK, Anchor::Start);
    s.text(20.0, 178.0, "like weights, one pair", 12, tok::INK, Anchor::Start);
    s.text(20.0, 196.0, "per unit.", 12, tok::INK, Anchor::Start);
    ("b16-1-where-bn-sits".to_string(), s.finish())
}

fn b16_before_after() -> Figure {
    let z = calc::BATCH.to_vec();
    let bn = calc::batch_norm(&z, calc::GAMMA, calc::BETA);
    let rows: [(&str, String, Vec<f64>, f64, f64); 3] = [
        ("weighted sums z", format!("mean {}, sd {}", numt(bn.mean, 2), num(bn.var.sqrt(), 2)), z.clone(), 8.0, 22.0),
        ("normalised ẑ", format!("mean {}, sd {}", num(calc::mean(&bn.xhat), 2), num(calc::sd_pop(&bn.xhat), 2)), bn.xhat.clone(), -2.0, 5.0),
        ("output γẑ + β", format!("γ = {}, β = {}", numt(calc::GAMMA, 1), numt(calc::BETA, 1)), bn.y.clone(), -2.0, 5.0),
    ];
    let mut s = Svg::new(
        720,
        330,
        "Batch normalisation of six weighted sums: centre, rescale, then apply the learned scale and shift",
        "Three number lines: the raw weighted sums of six books around 14; the same values after subtracting the batch mean and dividing by the batch standard deviation, centred on 0; and after multiplying by gamma 2 and adding beta 1.",
    ).min_text(12);
    for (i, (name, stat, vals, lo, hi)) in rows.iter().enumerate() {
        let y = 70.0 + i as f64 * 92.0;
        let pl = Plot { x0: 200.0, y0: y, w: 480.0, h: 1.0, xmin: *lo, xmax: *hi, ymin: 0.0, ymax: 1.0 };
        s.text_bold(20.0, y + 4.0, name, 12, tok::INK, Anchor::Start);
        s.text(20.0, y + 22.0, stat, 11, tok::INK2, Anchor::Start);
        s.line(pl.x0, y, pl.x0 + pl.w, y, tok::AXIS, 1.0);
        let step = if hi - lo > 10.0 { 2.0 } else { 1.0 };
        let mut t = *lo;
        while t <= *hi + 1e-9 {
            s.line(pl.px(t), y, pl.px(t), y + 4.0, tok::AXIS, 1.0);
            s.text(pl.px(t), y + 18.0, &numt(t, 0), 11, tok::MUTED, Anchor::Middle);
            t += step;
        }
        let pxs: Vec<f64> = vals.iter().map(|v| pl.px(*v)).collect();
        let levels = stagger(&pxs, 40.0);
        for (k, v) in vals.iter().enumerate() {
            s.dot(pxs[k], y, 4.5, tok::S1, Some(tok::SURFACE));
            s.text(pxs[k], y - 10.0 - 14.0 * levels[k] as f64, &numt(*v, 2), 11, tok::INK, Anchor::Middle);
        }
    }
    s.text(20.0, 318.0, &format!("Batch mean {} and variance {} come from these six values alone; ε = {} guards the division.", numt(bn.mean, 2), num(bn.var, 3), num(calc::EPS, 5)), 11, tok::INK2, Anchor::Start);
    ("b16-2-before-after".to_string(), s.finish())
}

// ====================================================================================================
// Chapter 17. Images as numbers, and the convolution operation
// ====================================================================================================

/// Cells shaded by magnitude (pale for small, darker for large).
fn shaded(vals: &[Vec<f64>], max: f64, dec: usize) -> Vec<Vec<Cell>> {
    vals.iter().map(|r| r.iter().map(|v| cell(numt(*v, dec)).fill(heat(v.abs(), max))).collect()).collect()
}

/// Cells for a signed map: positive values on the blue ramp, negative values grey with orange ink, zeros plain.
fn signed(vals: &[Vec<f64>], max: f64) -> Vec<Vec<Cell>> {
    vals.iter()
        .map(|r| {
            r.iter()
                .map(|v| {
                    if *v > 0.0 {
                        cell(numt(*v, 2)).fill(heat(*v, max))
                    } else if *v < 0.0 {
                        cell(numt(*v, 2)).fill(tok::NEUTRAL).ink(tok::S2)
                    } else {
                        cell("0").ink(tok::MUTED)
                    }
                })
                .collect()
        })
        .collect()
}

/// A small picture of a grid drawn as pixels in two colours (for thumbnails).
fn thumbnail(s: &mut Svg, x: f64, y: f64, px: f64, g: &[Vec<f64>], on: &str, off: &str) {
    for (r, row) in g.iter().enumerate() {
        for (c, v) in row.iter().enumerate() {
            s.rect(x + c as f64 * px, y + r as f64 * px, px, px, if *v > 0.0 { on } else { off }, None);
        }
    }
    s.rect(x, y, px * g[0].len() as f64, px * g.len() as f64, "none", Some(tok::AXIS));
}

fn b17_cover() -> Figure {
    let cov = calc::cover();
    let (bg, sq) = (calc::rgb(tok::SURFACE), calc::rgb(tok::S2));
    let mut s = Svg::new(
        720,
        300,
        "An image is a grid of numbers: one plane for a grey picture, three planes for a colour one",
        "Left, the 6 by 6 cover as numbers, 9 for the bright hollow square and 0 elsewhere, with a small pixel picture of it; right, the colour version as red, green and blue planes of the top-left 4 by 4 corner.",
    ).min_text(12);
    s.text_bold(30.0, 30.0, "grey: one plane of numbers", 12, tok::INK, Anchor::Start);
    s.cells(30.0, 50.0, 30.0, &shaded(&cov, 9.0, 0));
    s.text(30.0, 250.0, "the 6 × 6 cover, 0 to 9", 11, tok::INK2, Anchor::Start);
    s.text(30.0, 264.0, "(darker cell = larger number)", 11, tok::MUTED, Anchor::Start);
    s.text(230.0, 46.0, "as pixels", 11, tok::INK2, Anchor::Start);
    thumbnail(&mut s, 230.0, 54.0, 10.0, &cov, tok::SURFACE, tok::INK);
    s.text_bold(356.0, 30.0, "colour: three planes (top-left 4 × 4 corner)", 12, tok::INK, Anchor::Start);
    let names = [("red", tok::S2), ("green", tok::S3), ("blue", tok::S1)];
    for (ch, (name, colour)) in names.iter().enumerate() {
        let plane: Vec<Vec<f64>> = cov.iter().take(4).map(|r| r.iter().take(4).map(|v| if *v > 0.0 { sq[ch] } else { bg[ch] }).collect()).collect();
        let x = 356.0 + ch as f64 * 116.0;
        s.text_bold(x + 54.0, 60.0, name, 12, colour, Anchor::Middle);
        let cells: Vec<Vec<Cell>> = plane.iter().map(|r| r.iter().map(|v| cell(numt(*v, 0))).collect()).collect();
        s.cells_wh(x, 68.0, 27.0, 26.0, &cells);
        s.rect_bold(x, 68.0, 108.0, 104.0, "none", colour);
    }
    s.text(356.0, 200.0, "as pixels", 11, tok::INK2, Anchor::Start);
    thumbnail(&mut s, 356.0, 208.0, 10.0, &cov, tok::S2, tok::SURFACE);
    s.text(426.0, 214.0, &format!("The orange square is red {}, green {},", numt(sq[0], 0), numt(sq[1], 0)), 11, tok::INK2, Anchor::Start);
    s.text(426.0, 228.0, &format!("blue {}; the cream background is {},", numt(sq[2], 0), numt(bg[0], 0)), 11, tok::INK2, Anchor::Start);
    s.text(426.0, 242.0, &format!("{}, {}. Each plane is a grid like the", numt(bg[1], 0), numt(bg[2], 0)), 11, tok::INK2, Anchor::Start);
    s.text(426.0, 256.0, "grey one, stored from 0 to 255.", 11, tok::INK2, Anchor::Start);
    ("b17-1-cover-numbers".to_string(), s.finish())
}

/// The five products that the plain kernel keeps for the window at (r, c), as "a + b + … = total".
fn plain_sum(img: &[Vec<f64>], r: usize, c: usize) -> String {
    let k = calc::plain_kernel();
    let mut terms: Vec<String> = Vec::new();
    let mut total = 0.0;
    for (a, row) in k.iter().enumerate() {
        for (b, w) in row.iter().enumerate() {
            if *w != 0.0 {
                let v = img[r + a][c + b] * w;
                total += v;
                terms.push(numt(v, 0));
            }
        }
    }
    format!("{} = {}", terms.join(" + "), numt(total, 0))
}

fn b17_filmstrip() -> Figure {
    let cov = calc::cover();
    let k = calc::plain_kernel();
    let out = calc::conv2d(&cov, &k, 1);
    let frames = [(0usize, 0usize), (0, 1), (0, 2), (0, 3), (1, 0), (1, 1)];
    let mut s = Svg::new(
        720,
        410,
        "The plain kernel slides over the cover; each position gives one output number",
        "Six frames in reading order; in each, the 6 by 6 cover with the current 3 by 3 window outlined in orange, the 4 by 4 output filled up to the current position, and the five kept products added up beneath.",
    ).min_text(12);
    s.text(30.0, 26.0, "Lay the kernel on a window, multiply cell by cell and add the nine products;", 11, tok::INK2, Anchor::Start);
    s.text(30.0, 42.0, "write the total, then move one cell.", 11, tok::INK2, Anchor::Start);
    s.text(620.0, 58.0, "kernel", 11, tok::INK2, Anchor::End);
    s.cells(628.0, 38.0, 16.0, &grid(&k, 0));
    let cw = 17.0;
    let ow = 19.0;
    for (n, (r, c)) in frames.iter().enumerate() {
        let (fx, fy) = (30.0 + (n % 3) as f64 * 232.0, 116.0 + (n / 3) as f64 * 150.0);
        s.text_bold(fx, fy - 12.0, &format!("{}. window at row {}, column {}", n + 1, r + 1, c + 1), 11, tok::INK, Anchor::Start);
        s.cells(fx, fy, cw, &shaded(&cov, 9.0, 0));
        s.rect_bold(fx + *c as f64 * cw, fy + *r as f64 * cw, 3.0 * cw, 3.0 * cw, "none", tok::S2);
        let ox = fx + 124.0;
        let cells: Vec<Vec<Cell>> = (0..4)
            .map(|i| {
                (0..4)
                    .map(|j| {
                        let done = i * 4 + j <= r * 4 + c;
                        if done { cell(numt(out[i][j], 0)).fill(heat(out[i][j], 27.0)) } else { cell("").fill(tok::NEUTRAL) }
                    })
                    .collect()
            })
            .collect();
        s.cells(ox, fy + cw, ow, &cells);
        s.rect_bold(ox + *c as f64 * ow, fy + cw + *r as f64 * ow, ow, ow, "none", tok::S2);
        s.arrow(fx + 6.0 * cw + 4.0, fy + 3.0 * cw, ox - 4.0, fy + 3.0 * cw, None);
        s.text(fx, fy + 6.0 * cw + 16.0, &plain_sum(&cov, *r, *c), 11, tok::INK2, Anchor::Start);
    }
    ("b17-2-filmstrip".to_string(), s.finish())
}

fn b17_padding() -> Figure {
    let cov = calc::cover();
    let k = calc::plain_kernel();
    let padded = calc::pad(&cov, 1);
    let out = calc::conv2d(&padded, &k, 1);
    let mut s = Svg::new(
        720,
        330,
        "Zero padding keeps the output the same size as the input",
        "Left, the cover with a ring of zeros added, the first window outlined at the corner; middle, the kernel; right, the 6 by 6 output, the same size as the cover.",
    ).min_text(12);
    let cw = 24.0;
    s.text_bold(30.0, 30.0, &format!("cover padded with zeros ({} × {})", padded.len(), padded[0].len()), 12, tok::INK, Anchor::Start);
    let cells: Vec<Vec<Cell>> = padded
        .iter()
        .enumerate()
        .map(|(r, row)| {
            row.iter()
                .enumerate()
                .map(|(c, v)| {
                    let edge = r == 0 || c == 0 || r == padded.len() - 1 || c == row.len() - 1;
                    if edge { cell("0").fill(tok::NEUTRAL).ink(tok::MUTED) } else { cell(numt(*v, 0)).fill(heat(*v, 9.0)) }
                })
                .collect()
        })
        .collect();
    s.cells(30.0, 46.0, cw, &cells);
    s.rect_bold(30.0, 46.0, 3.0 * cw, 3.0 * cw, "none", tok::S2);
    s.text(30.0, 46.0 + 8.0 * cw + 18.0, "grey ring: padding (p = 1)", 11, tok::INK2, Anchor::Start);
    s.text(30.0, 46.0 + 8.0 * cw + 32.0, "orange: the first window, mostly padding", 11, tok::S2, Anchor::Start);
    s.text_bold(262.0, 104.0, "kernel", 11, tok::INK2, Anchor::Start);
    s.cells(262.0, 112.0, 22.0, &grid(&k, 0));
    s.arrow(236.0, 142.0, 256.0, 142.0, None);
    s.arrow(334.0, 142.0, 396.0, 142.0, None);
    s.text_bold(400.0, 30.0, &format!("output ({} × {}), same size as the cover", out.len(), out[0].len()), 12, tok::INK, Anchor::Start);
    s.cells(400.0, 46.0, 34.0, &shaded(&out, 27.0, 0));
    s.rect_bold(400.0, 46.0, 34.0, 34.0, "none", tok::S2);
    s.text(400.0, 46.0 + 6.0 * 34.0 + 18.0, &format!("corner: {}", plain_sum(&padded, 0, 0)), 11, tok::S2, Anchor::Start);
    s.text(400.0, 46.0 + 6.0 * 34.0 + 32.0, "the edge pixels now reach", 11, tok::INK2, Anchor::Start);
    s.text(400.0, 46.0 + 6.0 * 34.0 + 46.0, "the centre of a window", 11, tok::INK2, Anchor::Start);
    ("b17-3-padding".to_string(), s.finish())
}

fn b17_stride() -> Figure {
    let cov = calc::cover();
    let k = calc::plain_kernel();
    let s2 = calc::conv2d(&cov, &k, 2);
    let padded = calc::pad(&cov, 1);
    let s2p = calc::conv2d(&padded, &k, 2);
    let mut s = Svg::new(
        720,
        350,
        "A stride of 2 moves the kernel two cells at a time and shrinks the output",
        "Left, the cover with the four windows of a stride-2 convolution outlined and the 2 by 2 output; right, the zero-padded cover with the first three windows outlined and the 3 by 3 output; grey cells are never read.",
    ).min_text(12);
    let colours = [tok::S1, tok::S2, tok::S3, tok::INK2];
    // left panel: stride 2, no padding
    let cw = 26.0;
    s.text_bold(30.0, 30.0, "stride 2, no padding", 12, tok::INK, Anchor::Start);
    let used_rows = (s2.len() - 1) * 2 + 3;
    let cells: Vec<Vec<Cell>> = cov
        .iter()
        .enumerate()
        .map(|(r, row)| {
            row.iter()
                .enumerate()
                .map(|(c, v)| if r >= used_rows || c >= used_rows { cell(numt(*v, 0)).fill(tok::NEUTRAL).ink(tok::MUTED) } else { cell(numt(*v, 0)).fill(heat(*v, 9.0)) })
                .collect()
        })
        .collect();
    s.cells(30.0, 50.0, cw, &cells);
    for (n, (r, c)) in [(0usize, 0usize), (0, 2), (2, 0), (2, 2)].iter().enumerate() {
        let inset = n as f64 * 1.5;
        s.rect_bold(30.0 + *c as f64 * cw + inset, 50.0 + *r as f64 * cw + inset, 3.0 * cw - 2.0 * inset, 3.0 * cw - 2.0 * inset, "none", colours[n]);
    }
    let out_cells: Vec<Vec<Cell>> = s2.iter().map(|r| r.iter().map(|v| cell(numt(*v, 0)).fill(heat(*v, 27.0))).collect()).collect();
    s.cells(226.0, 88.0, 34.0, &out_cells);
    for n in 0..4 {
        s.rect_bold(226.0 + (n % 2) as f64 * 34.0, 88.0 + (n / 2) as f64 * 34.0, 34.0, 34.0, "none", colours[n]);
    }
    s.arrow(192.0, 124.0, 220.0, 124.0, None);
    s.text(226.0, 80.0, &format!("{} × {}", s2.len(), s2[0].len()), 11, tok::INK2, Anchor::Start);
    s.text(30.0, 226.0, "Windows start at columns 1 and 3, then", 11, tok::INK2, Anchor::Start);
    s.text(30.0, 240.0, "the next would run off the edge: the grey", 11, tok::INK2, Anchor::Start);
    s.text(30.0, 254.0, "last row and column are never read.", 11, tok::INK2, Anchor::Start);
    // right panel: stride 2 with padding 1
    let pw = 22.0;
    let (px0, py0) = (344.0, 50.0);
    s.text_bold(px0, 30.0, "stride 2, padding 1", 12, tok::INK, Anchor::Start);
    let last_used = (s2p.len() - 1) * 2 + 2;
    let pcells: Vec<Vec<Cell>> = padded
        .iter()
        .enumerate()
        .map(|(r, row)| {
            row.iter()
                .enumerate()
                .map(|(c, v)| if r > last_used || c > last_used { cell(numt(*v, 0)).fill(tok::NEUTRAL).ink(tok::MUTED) } else { cell(numt(*v, 0)).fill(heat(*v, 9.0)) })
                .collect()
        })
        .collect();
    s.cells(px0, py0, pw, &pcells);
    for n in 0..3 {
        let inset = n as f64 * 1.5;
        s.rect_bold(px0 + (2 * n) as f64 * pw + inset, py0 + inset, 3.0 * pw - 2.0 * inset, 3.0 * pw - 2.0 * inset, "none", colours[n]);
    }
    let pout: Vec<Vec<Cell>> = s2p.iter().map(|r| r.iter().map(|v| cell(numt(*v, 0)).fill(heat(*v, 27.0))).collect()).collect();
    s.cells(566.0, 84.0, 34.0, &pout);
    for n in 0..3 {
        s.rect_bold(566.0 + n as f64 * 34.0, 84.0, 34.0, 34.0, "none", colours[n]);
    }
    s.arrow(px0 + 8.0 * pw + 4.0, 136.0, 560.0, 136.0, None);
    s.text(566.0, 76.0, &format!("{} × {}", s2p.len(), s2p[0].len()), 11, tok::INK2, Anchor::Start);
    s.text(px0, 250.0, "The first row of windows is outlined; the rows below", 11, tok::INK2, Anchor::Start);
    s.text(px0, 264.0, "repeat it two cells lower. The grey bottom row and", 11, tok::INK2, Anchor::Start);
    s.text(px0, 278.0, "right column of the padded cover are skipped, as in", 11, tok::INK2, Anchor::Start);
    s.text(px0, 292.0, "Dumoulin and Visin's Figure 2.7 for the same sizes.", 11, tok::INK2, Anchor::Start);
    s.text(30.0, 330.0, "Colours pair each window with the output cell it produces.", 11, tok::INK2, Anchor::Start);
    ("b17-4-stride".to_string(), s.finish())
}

fn b17_sizes() -> Figure {
    let i = 6usize;
    let rows: [(&str, usize, usize, usize); 8] = [
        ("no padding, stride 1 (Figure 17.2)", 3, 0, 1),
        ("same padding (Figure 17.3)", 3, 1, 1),
        ("full padding", 3, 2, 1),
        ("stride 2 (Figure 17.4, left)", 3, 0, 2),
        ("stride 2 and padding (Figure 17.4, right)", 3, 1, 2),
        ("a 5 × 5 kernel", 5, 0, 1),
        ("a 5 × 5 kernel with same padding", 5, 2, 1),
        ("a 1 × 1 kernel", 1, 0, 1),
    ];
    let mut s = Svg::new(
        720,
        360,
        "Output sizes for a 6 by 6 input under different kernels, padding and strides",
        "A table of eight settings with kernel size, padding, stride, the output-size arithmetic and the resulting output size for an input of 6.",
    ).min_text(12);
    s.text_bold(30.0, 30.0, &format!("output size o = ⌊(i + 2p − k) / s⌋ + 1, for an input of i = {i}"), 12, tok::INK, Anchor::Start);
    let heads = [(30.0, "setting"), (330.0, "k"), (372.0, "p"), (414.0, "s"), (460.0, "arithmetic"), (650.0, "output")];
    for (x, h) in heads {
        s.text_bold(x, 64.0, h, 12, tok::INK2, Anchor::Start);
    }
    s.line(30.0, 72.0, 700.0, 72.0, tok::AXIS, 1.0);
    for (n, (name, k, p, st)) in rows.iter().enumerate() {
        let y = 78.0 + n as f64 * 32.0;
        if n % 2 == 0 {
            s.rect(30.0, y, 670.0, 28.0, tok::NEUTRAL, None);
        }
        let o = calc::out_size(i, *k, *p, *st);
        s.text(36.0, y + 19.0, name, 12, tok::INK, Anchor::Start);
        s.text(334.0, y + 19.0, &k.to_string(), 12, tok::INK, Anchor::Start);
        s.text(376.0, y + 19.0, &p.to_string(), 12, tok::INK, Anchor::Start);
        s.text(418.0, y + 19.0, &st.to_string(), 12, tok::INK, Anchor::Start);
        s.text(460.0, y + 19.0, &format!("⌊({} + {} − {}) / {}⌋ + 1", i, 2 * p, k, st), 12, tok::INK2, Anchor::Start);
        s.text_bold(650.0, y + 19.0, &format!("{o} × {o}"), 12, tok::S1, Anchor::Start);
    }
    s.text(30.0, 344.0, "The floor ⌊ ⌋ drops a leftover when the last stride would run past the edge.", 11, tok::INK2, Anchor::Start);
    ("b17-5-output-sizes".to_string(), s.finish())
}

// ====================================================================================================
// Chapter 18. Kernels, filters, channels and bias
// ====================================================================================================

fn b18_sum() -> Figure {
    let planes = calc::rgb_crop();
    let kernels = calc::filter_kernels();
    let maps: Vec<calc::Grid> = planes.iter().zip(&kernels).map(|(p, k)| calc::conv2d(p, k, 1)).collect();
    let total = calc::add(&calc::add(&maps[0], &maps[1]), &maps[2]);
    let names = [("red", tok::S2), ("green", tok::S3), ("blue", tok::S1)];
    let mut s = Svg::new(
        720,
        420,
        "One filter is three kernels, one per input channel; their three maps are added into one",
        "Three rows, one per colour plane: the 4 by 4 plane, its own 3 by 3 kernel, and the resulting 2 by 2 map; arrows join the three maps into their 2 by 2 sum on the right.",
    ).min_text(12);
    s.text(30.0, 24.0, "input plane", 11, tok::INK2, Anchor::Start);
    s.text(196.0, 24.0, "this filter's kernel for it", 11, tok::INK2, Anchor::Start);
    s.text(372.0, 24.0, "per-plane map", 11, tok::INK2, Anchor::Start);
    for (ch, (name, colour)) in names.iter().enumerate() {
        let y = 40.0 + ch as f64 * 120.0;
        s.text_bold(30.0, y + 8.0, name, 12, colour, Anchor::Start);
        s.cells(30.0, y + 16.0, 22.0, &grid(&planes[ch], 0));
        s.rect_bold(30.0, y + 16.0, 88.0, 88.0, "none", colour);
        s.text(150.0, y + 64.0, "⊛", 16, tok::INK2, Anchor::Middle);
        s.cells(196.0, y + 27.0, 22.0, &grid(&kernels[ch], 0));
        s.rect_bold(196.0, y + 27.0, 66.0, 66.0, "none", colour);
        s.text(300.0, y + 64.0, "=", 16, tok::INK2, Anchor::Middle);
        s.cells(372.0, y + 30.0, 30.0, &signed(&maps[ch], 12.0));
        s.arrow(438.0, y + 60.0, 540.0, 190.0 + (ch as f64 - 1.0) * 12.0, None);
    }
    s.text_bold(550.0, 150.0, "sum of the three maps", 12, tok::INK, Anchor::Start);
    s.cells(560.0, 160.0, 44.0, &signed(&total, 12.0));
    s.text(560.0, 272.0, "one output channel", 11, tok::INK2, Anchor::Start);
    s.text(560.0, 286.0, "(the bias comes next,", 11, tok::INK2, Anchor::Start);
    s.text(560.0, 300.0, "in Figure 18.2)", 11, tok::INK2, Anchor::Start);
    s.text(30.0, 408.0, "⊛ means slide and multiply-add, as in Figure 17.2. Grey cells with orange numbers are negative.", 11, tok::INK2, Anchor::Start);
    ("b18-1-kernels-sum".to_string(), s.finish())
}

fn b18_bias() -> Figure {
    let planes = calc::rgb_crop();
    let kernels = calc::filter_kernels();
    let maps: Vec<calc::Grid> = planes.iter().zip(&kernels).map(|(p, k)| calc::conv2d(p, k, 1)).collect();
    let total = calc::add(&calc::add(&maps[0], &maps[1]), &maps[2]);
    let biased: calc::Grid = total.iter().map(|r| r.iter().map(|v| v + calc::FILTER_BIAS).collect()).collect();
    let relu: calc::Grid = biased.iter().map(|r| r.iter().map(|v| v.max(0.0)).collect()).collect();
    let mut s = Svg::new(
        720,
        240,
        "The filter adds its one bias to every position, then the activation follows",
        "Three 2 by 2 grids left to right: the summed map, the map after adding the filter's bias of minus 2 to every cell, and the map after ReLU sets the negative cells to zero.",
    ).min_text(12);
    let cw = 48.0;
    let xs = [40.0, 280.0, 520.0];
    let titles = ["sum of the kernels' maps".to_string(), format!("+ bias ({}) everywhere", numt(calc::FILTER_BIAS, 0)), "ReLU: negatives become 0".to_string()];
    for (n, (g, t)) in [&total, &biased, &relu].iter().zip(&titles).enumerate() {
        s.text_bold(xs[n], 40.0, t, 12, tok::INK, Anchor::Start);
        s.cells(xs[n], 56.0, cw, &signed(g, 12.0));
        if n < 2 {
            s.arrow(xs[n] + 2.0 * cw + 12.0, 56.0 + cw, xs[n + 1] - 12.0, 56.0 + cw, None);
        }
    }
    s.text(40.0, 190.0, &format!("A filter has exactly one bias, a single number ({}), however many kernels and positions it has.", numt(calc::FILTER_BIAS, 0)), 11, tok::INK2, Anchor::Start);
    s.text(40.0, 206.0, "The finished grid is one output channel, also called a feature map.", 11, tok::INK2, Anchor::Start);
    ("b18-2-bias".to_string(), s.finish())
}

/// A stack of `n` square planes drawn with an offset, front plane last.
fn stack(s: &mut Svg, x: f64, y: f64, size: f64, n: usize, offset: f64, strokes: &[&str], fill: &str) {
    for i in (0..n).rev() {
        let (px, py) = (x + i as f64 * offset, y - i as f64 * offset);
        s.rect(px, py, size, size, fill, Some(strokes[i % strokes.len()]));
    }
}

fn b18_many() -> Figure {
    let (k, c_in, filters, out) = (3usize, 3usize, 4usize, 2usize);
    let params = filters * (k * k * c_in + 1);
    let alex = 96 * (11 * 11 * 3 + 1);
    let mut s = Svg::new(
        720,
        330,
        "Four filters give four output channels; each filter spans every input channel",
        "Left, a 4 by 4 by 3 input block; middle, four filters drawn as small three-plane stacks in four colours; right, the four 2 by 2 output maps stacked into a 2 by 2 by 4 block; the parameter count is written beneath.",
    ).min_text(12);
    s.text_bold(30.0, 34.0, "input: 4 × 4 × 3", 12, tok::INK, Anchor::Start);
    stack(&mut s, 40.0, 110.0, 96.0, 3, 14.0, &[tok::S2, tok::S3, tok::S1], tok::SURFACE);
    s.text(40.0, 234.0, "red, green, blue planes", 11, tok::INK2, Anchor::Start);
    let colours = [tok::S1, tok::S2, tok::S3, tok::INK2];
    s.text_bold(270.0, 34.0, &format!("{filters} filters, each {k} × {k} × {c_in}"), 12, tok::INK, Anchor::Start);
    for f in 0..filters {
        let y = 70.0 + f as f64 * 56.0;
        stack(&mut s, 276.0, y, 30.0, 3, 6.0, &[colours[f]], tok::NEUTRAL);
        s.text(334.0, y + 22.0, &format!("filter {}: {} weights + 1 bias", f + 1, k * k * c_in), 11, tok::INK2, Anchor::Start);
        s.arrow(522.0, y + 16.0, 574.0, 130.0 + f as f64 * 6.0, None);
    }
    s.arrow(160.0, 150.0, 280.0, 150.0, None);
    s.text_bold(580.0, 34.0, &format!("output: {out} × {out} × {filters}"), 12, tok::INK, Anchor::Start);
    for f in (0..filters).rev() {
        let (x, y) = (580.0 + f as f64 * 14.0, 110.0 - f as f64 * 14.0);
        s.rect(x, y, 70.0, 70.0, tok::FILL1, Some(colours[f]));
        s.line(x + 35.0, y, x + 35.0, y + 70.0, colours[f], 1.0);
        s.line(x, y + 35.0, x + 70.0, y + 35.0, colours[f], 1.0);
    }
    s.text(580.0, 234.0, "one map per filter", 11, tok::INK2, Anchor::Start);
    s.text_bold(30.0, 280.0, &format!("parameters: {filters} × ({k} × {k} × {c_in} + 1) = {params}"), 12, tok::INK, Anchor::Start);
    s.text(30.0, 300.0, &format!("In CS231n's example, 96 filters of 11 × 11 × 3 give 96 × (11 × 11 × 3 + 1) = {} parameters.", thousands(alex as u64)), 11, tok::INK2, Anchor::Start);
    s.text(30.0, 316.0, "The number of output channels is always the number of filters.", 11, tok::INK2, Anchor::Start);
    ("b18-3-many-filters".to_string(), s.finish())
}

// ====================================================================================================
// Chapter 19. Why convolution works
// ====================================================================================================

fn b19_matrices() -> Figure {
    let letters = ["a", "b", "c", "d", "e", "f", "g", "h", "i"];
    let mut conv: Vec<Vec<String>> = vec![vec![String::new(); 16]; 4];
    for (o, row) in conv.iter_mut().enumerate() {
        let (oi, oj) = (o / 2, o % 2);
        for a in 0..3 {
            for b in 0..3 {
                row[(oi + a) * 4 + (oj + b)] = letters[a * 3 + b].to_string();
            }
        }
    }
    let used: usize = conv.iter().map(|r| r.iter().filter(|x| !x.is_empty()).count()).sum();
    let zeros = 64 - used;
    let mut s = Svg::new(
        720,
        346,
        "A dense layer needs 64 free weights to map a 4 by 4 input to 2 by 2; a convolution needs 9, shared",
        "Two 4 by 16 matrices. Top, a dense layer with a different weight in each of the 64 cells. Bottom, the convolution's matrix: each row holds the nine kernel weights a to i at the positions of its window and zeros elsewhere; the same letters repeat, shifted, in every row.",
    ).min_text(12);
    let (x0, cw, ch) = (60.0, 40.0, 22.0);
    s.text_bold(x0, 30.0, "dense layer: every output reads every pixel with its own weight", 12, tok::INK, Anchor::Start);
    for j in 0..16 {
        s.text(x0 + j as f64 * cw + cw / 2.0, 52.0, &format!("{}", j + 1), 11, tok::MUTED, Anchor::Middle);
    }
    let dense: Vec<Vec<Cell>> = (0..4).map(|r| (0..16).map(|j| cell(format!("w{}", r * 16 + j + 1)).fill(tok::FILL1)).collect()).collect();
    s.cells_wh(x0, 58.0, cw, ch, &dense);
    for r in 0..4 {
        s.text(x0 - 8.0, 58.0 + r as f64 * ch + 15.0, &format!("out {}", r + 1), 11, tok::INK2, Anchor::End);
    }
    s.text(x0, 164.0, "64 different weights", 11, tok::INK2, Anchor::Start);
    s.text_bold(x0, 196.0, "convolution with a 3 × 3 kernel: the same nine weights, shifted", 12, tok::INK, Anchor::Start);
    let convc: Vec<Vec<Cell>> = conv.iter().map(|r| r.iter().map(|v| if v.is_empty() { cell("0").ink(tok::MUTED) } else { cell(v.clone()).fill(tok::FILL1) }).collect()).collect();
    s.cells_wh(x0, 208.0, cw, ch, &convc);
    for r in 0..4 {
        s.text(x0 - 8.0, 208.0 + r as f64 * ch + 15.0, &format!("out {}", r + 1), 11, tok::INK2, Anchor::End);
    }
    s.text(x0, 316.0, &format!("9 weights (a to i), each used 4 times; the other {zeros} of the 64 entries are fixed at 0."), 11, tok::INK2, Anchor::Start);
    s.text(x0, 332.0, "Columns are the 16 pixels of the 4 × 4 input, read row by row.", 11, tok::MUTED, Anchor::Start);
    ("b19-1-dense-vs-conv".to_string(), s.finish())
}

fn b19_sobel() -> Figure {
    let img = calc::edge_image();
    let k = calc::sobel();
    let out = calc::conv2d(&img, &k, 1);
    let mut s = Svg::new(
        720,
        340,
        "The Sobel kernel turns an image into a map of its vertical edges",
        "Left, the 8 by 8 image with a bright 4 by 4 block; middle, the Sobel kernel; right, the 6 by 6 output: zero on flat areas and inside the block, negative along the block's left edge and positive along its right edge.",
    ).min_text(12);
    s.text_bold(30.0, 30.0, "image: a bright block (9) on 1", 12, tok::INK, Anchor::Start);
    s.cells(30.0, 46.0, 22.0, &shaded(&img, 9.0, 0));
    s.text_bold(242.0, 118.0, "Sobel kernel", 11, tok::INK2, Anchor::Start);
    s.cells(242.0, 126.0, 26.0, &signed(&k, 2.0));
    s.arrow(214.0, 164.0, 236.0, 164.0, None);
    s.arrow(324.0, 164.0, 362.0, 164.0, None);
    s.text_bold(370.0, 30.0, &format!("output ({} × {})", out.len(), out[0].len()), 12, tok::INK, Anchor::Start);
    s.cells(370.0, 46.0, 36.0, &signed(&out, 32.0));
    s.text(370.0, 46.0 + 6.0 * 36.0 + 18.0, "negative: the left edge (dark to bright)", 11, tok::S2, Anchor::Start);
    s.text(370.0, 46.0 + 6.0 * 36.0 + 32.0, "positive: the right edge (bright to dark)", 11, tok::S1, Anchor::Start);
    s.text(370.0, 46.0 + 6.0 * 36.0 + 46.0, "zero: flat areas, and the top and bottom edges", 11, tok::INK2, Anchor::Start);
    s.text(30.0, 46.0 + 8.0 * 22.0 + 18.0, "Each output compares the column on the", 11, tok::INK2, Anchor::Start);
    s.text(30.0, 46.0 + 8.0 * 22.0 + 32.0, "left of a window with the column on its right,", 11, tok::INK2, Anchor::Start);
    s.text(30.0, 46.0 + 8.0 * 22.0 + 46.0, "weighting the middle row twice.", 11, tok::INK2, Anchor::Start);
    ("b19-2-sobel".to_string(), s.finish())
}

fn b19_locality() -> Figure {
    let img = calc::edge_image();
    let out = calc::conv2d(&img, &calc::sobel(), 1);
    let pixels: u64 = 224 * 224 * 3;
    let dense: u64 = pixels * 1000;
    let conv_w: u64 = 3 * 3 * 3 * 64;
    let conv_p: u64 = conv_w + 64;
    let mut s = Svg::new(
        720,
        300,
        "Each output looks at a small patch; that is why a convolution needs so few weights",
        "Left, the 8 by 8 image with two 3 by 3 patches outlined in blue and orange and the output cells they produce; right, two bars on a logarithmic scale comparing the weights of a dense layer and of a convolution on a 224 by 224 colour photo.",
    ).min_text(12);
    let cw = 18.0;
    s.text_bold(30.0, 30.0, "one output, one patch", 12, tok::INK, Anchor::Start);
    s.cells(30.0, 46.0, cw, &shaded(&img, 9.0, 0));
    s.rect_bold(30.0 + 1.0 * cw, 46.0 + 2.0 * cw, 3.0 * cw, 3.0 * cw, "none", tok::S1);
    s.rect_bold(30.0 + 4.0 * cw, 46.0 + 4.0 * cw, 3.0 * cw, 3.0 * cw, "none", tok::S2);
    let ow = 28.0;
    s.arrow(30.0 + 8.0 * cw + 6.0, 46.0 + 4.0 * cw, 192.0, 46.0 + 4.0 * cw, None);
    s.cells_wh(198.0, 46.0 + cw, ow, cw, &signed(&out, 32.0));
    s.rect_bold(198.0 + 1.0 * ow, 46.0 + cw + 2.0 * cw, ow, cw, "none", tok::S1);
    s.rect_bold(198.0 + 4.0 * ow, 46.0 + cw + 4.0 * cw, ow, cw, "none", tok::S2);
    s.text(198.0, 46.0 + cw - 6.0, "Sobel output", 11, tok::INK2, Anchor::Start);
    s.text(30.0, 212.0, "Each outlined patch of 9 pixels produces the output", 11, tok::INK2, Anchor::Start);
    s.text(30.0, 226.0, "cell outlined in the same colour; the same nine", 11, tok::INK2, Anchor::Start);
    s.text(30.0, 240.0, "weights are used at every patch.", 11, tok::INK2, Anchor::Start);
    // bars on a log scale
    let (bx, by, bw) = (380.0, 70.0, 300.0);
    let lx = |n: u64| bx + (n as f64).log10() / 9.0 * bw;
    s.text_bold(bx, 30.0, "weights for a 224 × 224 colour photo", 12, tok::INK, Anchor::Start);
    s.text(bx, 46.0, &format!("({} input numbers; log scale, each tick ×10)", thousands(pixels)), 11, tok::INK2, Anchor::Start);
    for t in 0..=9 {
        let x = bx + f64::from(t) / 9.0 * bw;
        s.line(x, by, x, by + 150.0, tok::GRID, 1.0);
        s.text(x, by + 166.0, &format!("10{}", superscript(t)), 11, tok::MUTED, Anchor::Middle);
    }
    s.rect(bx, by + 20.0, lx(dense) - bx, 30.0, tok::S2, None);
    s.text(bx, by + 14.0, &format!("dense layer to 1,000 units: {} weights", thousands(dense)), 11, tok::INK, Anchor::Start);
    s.rect(bx, by + 94.0, lx(conv_p) - bx, 30.0, tok::S1, None);
    s.text(bx, by + 74.0, "3 × 3 convolution with 64 filters:", 11, tok::INK, Anchor::Start);
    s.text(bx, by + 88.0, &format!("{} weights + 64 biases = {}", thousands(conv_w), thousands(conv_p)), 11, tok::INK, Anchor::Start);
    ("b19-3-locality".to_string(), s.finish())
}

// ====================================================================================================
// Chapter 20. Pooling and strides
// ====================================================================================================

fn b20_pooling() -> Figure {
    let x = calc::pooling_input();
    let mx = calc::pool(&x, 2, 2, true);
    let av = calc::pool(&x, 2, 2, false);
    let colours = [tok::S1, tok::S2, tok::S3, tok::INK2];
    let mut s = Svg::new(
        720,
        300,
        "Max pooling keeps the largest number in each 2 by 2 window; average pooling keeps their mean",
        "A 4 by 4 grid split into four 2 by 2 windows in four colours, each window's largest cell outlined in black; arrows lead to a 2 by 2 max-pooled grid and a 2 by 2 average-pooled grid with matching colours.",
    ).min_text(12);
    let cw = 44.0;
    s.text_bold(40.0, 40.0, "input 4 × 4, windows 2 × 2, stride 2", 12, tok::INK, Anchor::Start);
    s.cells(40.0, 56.0, cw, &shaded(&x, 9.0, 0));
    for w in 0..4 {
        let (wr, wc) = (w / 2, w % 2);
        s.rect_bold(40.0 + (2 * wc) as f64 * cw + 1.5, 56.0 + (2 * wr) as f64 * cw + 1.5, 2.0 * cw - 3.0, 2.0 * cw - 3.0, "none", colours[w]);
        let (mut best, mut br, mut bc) = (f64::NEG_INFINITY, 0usize, 0usize);
        for a in 0..2 {
            for b in 0..2 {
                let v = x[2 * wr + a][2 * wc + b];
                if v > best {
                    best = v;
                    br = 2 * wr + a;
                    bc = 2 * wc + b;
                }
            }
        }
        s.rect_bold(40.0 + bc as f64 * cw + 7.0, 56.0 + br as f64 * cw + 7.0, cw - 14.0, cw - 14.0, "none", tok::INK);
    }
    let draw_out = |s: &mut Svg, y: f64, g: &calc::Grid, title: &str, dec: usize| {
        s.text_bold(300.0, y - 10.0, title, 12, tok::INK, Anchor::Start);
        s.cells(300.0, y, cw, &shaded(g, 9.0, dec));
        for w in 0..4 {
            s.rect_bold(300.0 + (w % 2) as f64 * cw + 1.5, y + (w / 2) as f64 * cw + 1.5, cw - 3.0, cw - 3.0, "none", colours[w]);
        }
    };
    draw_out(&mut s, 40.0, &mx, "max pooling", 0);
    draw_out(&mut s, 162.0, &av, "average pooling", 2);
    s.arrow(224.0, 120.0, 292.0, 84.0, None);
    s.arrow(224.0, 160.0, 292.0, 204.0, None);
    let tl: Vec<f64> = vec![x[0][0], x[0][1], x[1][0], x[1][1]];
    let tx = 412.0;
    s.text(tx, 70.0, "Blue window, top left:", 12, tok::INK, Anchor::Start);
    s.text(tx, 90.0, &format!("max({}) = {}", tl.iter().map(|v| numt(*v, 0)).collect::<Vec<_>>().join(", "), numt(mx[0][0], 0)), 12, tok::INK, Anchor::Start);
    s.text(tx, 110.0, &format!("({}) / 4 = {}", tl.iter().map(|v| numt(*v, 0)).collect::<Vec<_>>().join(" + "), numt(av[0][0], 2)), 12, tok::INK, Anchor::Start);
    s.text(tx, 150.0, "16 numbers become 4: each 2 × 2 window", 11, tok::INK2, Anchor::Start);
    s.text(tx, 164.0, "keeps one, so three quarters are dropped.", 11, tok::INK2, Anchor::Start);
    s.text(tx, 190.0, "Pooling has no weights to learn; it is the", 11, tok::INK2, Anchor::Start);
    s.text(tx, 204.0, "same fixed rule in every window and channel.", 11, tok::INK2, Anchor::Start);
    s.text(40.0, 284.0, "The black inner squares mark each window's largest value, the one max pooling keeps.", 11, tok::INK2, Anchor::Start);
    ("b20-1-pooling".to_string(), s.finish())
}

fn b20_shift() -> Figure {
    let a = vec![0.0, 0.0, 9.0, 9.0, 9.0, 0.0, 0.0, 0.0, 0.0, 0.0];
    let b: Vec<f64> = std::iter::once(0.0).chain(a.iter().take(a.len() - 1).copied()).collect();
    let k = vec![1.0, 1.0, 1.0];
    let (ca, cb) = (calc::conv1d(&a, &k), calc::conv1d(&b, &k));
    let pool = |v: &[f64]| -> Vec<f64> { v.chunks(4).map(|c| c.iter().copied().fold(f64::NEG_INFINITY, f64::max)).collect() };
    let (pa, pb) = (pool(&ca), pool(&cb));
    let mut s = Svg::new(
        720,
        330,
        "Convolution is equivariant to a shift; pooling is only approximately invariant",
        "Two columns, before and after shifting a bright bar one cell to the right: the input strips, the detector outputs, which shift by the same one cell, and the max over each half, where the peak value 27 stays in the same pooled cell.",
    ).min_text(12);
    let cw = 22.0;
    s.text(30.0, 24.0, "detector kernel [1, 1, 1] (adds three neighbours), then the max over each half of its output", 11, tok::INK2, Anchor::Start);
    for (col, (inp, conv, pooled, title)) in [(&a, &ca, &pa, "a bar at cells 3 to 5"), (&b, &cb, &pb, "the same bar, one cell to the right")].iter().enumerate() {
        let x0 = 40.0 + col as f64 * 344.0;
        s.text_bold(x0, 56.0, title, 12, tok::INK, Anchor::Start);
        s.text(x0, 80.0, "input", 11, tok::INK2, Anchor::Start);
        s.cells_wh(x0, 86.0, cw, cw, &[inp.iter().map(|v| cell(numt(*v, 0)).fill(heat(*v, 9.0))).collect()]);
        s.text(x0, 142.0, "detector output (moves with the bar)", 11, tok::INK2, Anchor::Start);
        s.cells_wh(x0 + cw, 148.0, cw, cw, &[conv.iter().map(|v| cell(numt(*v, 0)).fill(heat(*v, 27.0))).collect()]);
        let peak = conv.iter().position(|v| (*v - 27.0).abs() < 1e-9).unwrap_or(0);
        s.rect_bold(x0 + cw + peak as f64 * cw, 148.0, cw, cw, "none", tok::S2);
        s.text(x0, 208.0, "max over each half (4 cells)", 11, tok::INK2, Anchor::Start);
        s.cells_wh(x0 + cw, 214.0, 4.0 * cw, cw, &[pooled.iter().map(|v| cell(numt(*v, 0)).fill(heat(*v, 27.0))).collect()]);
        s.rect_bold(x0 + cw, 214.0, 4.0 * cw, cw, "none", tok::S2);
        for h in 0..3 {
            s.line(x0 + cw + h as f64 * 4.0 * cw, 172.0, x0 + cw + h as f64 * 4.0 * cw, 194.0, tok::AXIS, 1.0);
        }
    }
    s.text(40.0, 268.0, "Equivariant: the whole output moves one cell with the bar.", 11, tok::INK2, Anchor::Start);
    s.text(40.0, 284.0, &format!("Nearly invariant: the pooled peak stays {} in the left half, but the right half changes", numt(pa[0], 0)), 11, tok::INK2, Anchor::Start);
    s.text(40.0, 300.0, &format!("from {} to {}; a bigger shift would move the peak to another cell.", numt(pa[1], 0), numt(pb[1], 0)), 11, tok::INK2, Anchor::Start);
    ("b20-2-equivariance".to_string(), s.finish())
}

// ====================================================================================================
// Chapter 21. Receptive fields and the visual hierarchy
// ====================================================================================================

/// One panel of Figure 21.1: layers drawn from the input upwards, the chosen top unit's cone highlighted, labels on the right.
fn rf_panel(s: &mut Svg, y0: f64, input: usize, layers: &[(usize, usize)], top: usize, title: &str) {
    let spans = calc::spans(input, layers);
    let rfs = calc::receptive(layers);
    let (x0, cw, row_gap) = (40.0, 20.0, 40.0);
    let base = y0 + 24.0 + layers.len() as f64 * row_gap;
    s.text_bold(x0, y0 + 8.0, title, 12, tok::INK, Anchor::Start);
    let mut chosen: Vec<Vec<usize>> = vec![Vec::new(); spans.len()];
    chosen[spans.len() - 1] = vec![top];
    for l in (1..spans.len()).rev() {
        let (k, st) = layers[l - 1];
        let mut below: Vec<usize> = chosen[l].iter().flat_map(|j| (0..k).map(move |t| j * st + t)).collect();
        below.sort_unstable();
        below.dedup();
        chosen[l - 1] = below;
    }
    let centre = |l: usize, j: usize| x0 + (spans[l][j].0 + spans[l][j].1) as f64 / 2.0 * cw + cw / 2.0;
    for l in 1..spans.len() {
        let y = base - l as f64 * row_gap;
        let (k, st) = layers[l - 1];
        for &j in &chosen[l] {
            for t in 0..k {
                s.line(centre(l, j), y + 14.0, centre(l - 1, j * st + t), y + row_gap, tok::AXIS, 1.0);
            }
        }
    }
    for (l, units) in spans.iter().enumerate() {
        let y = base - l as f64 * row_gap;
        for (j, _) in units.iter().enumerate() {
            let on = chosen[l].contains(&j);
            let fill = if l == spans.len() - 1 && on { tok::S1 } else if on { tok::FILL3 } else { tok::NEUTRAL };
            s.rect(centre(l, j) - 8.0, y, 16.0, 14.0, fill, Some(tok::GRID));
        }
        let label = if l == 0 {
            format!("input: {input} pixels")
        } else {
            let (k, st) = layers[l - 1];
            format!("layer {l} (k = {k}, stride {st}): sees {}", rfs[l - 1].0)
        };
        s.text(330.0, y + 11.0, &label, 11, if l == 0 { tok::INK2 } else { tok::S1 }, Anchor::Start);
    }
    let first = chosen[0][0];
    let last = chosen[0][chosen[0].len() - 1];
    s.text(560.0, base - layers.len() as f64 * row_gap + 11.0, &format!("(pixels {} to {})", first + 1, last + 1), 11, tok::INK2, Anchor::Start);
}

fn b21_growth() -> Figure {
    let mut s = Svg::new(
        720,
        400,
        "Stacking layers grows the receptive field; a stride makes it grow faster",
        "Two panels of units in rows from the input up to layer 3, with lines from one top unit down to every unit it depends on: three 3-wide layers with stride 1 reach 3, 5 and 7 input pixels; with stride 2 in the first layer they reach 3, 7 and 11.",
    ).min_text(12);
    rf_panel(&mut s, 20.0, 9, &[(3, 1), (3, 1), (3, 1)], 1, "three 3-wide layers, stride 1");
    rf_panel(&mut s, 212.0, 13, &[(3, 2), (3, 1), (3, 1)], 0, "the same, with stride 2 in layer 1");
    ("b21-1-rf-growth".to_string(), s.finish())
}

fn b21_effective() -> Figure {
    let line = calc::path_counts(3, 3);
    let grid2: calc::Grid = line.iter().map(|a| line.iter().map(|b| a * b).collect()).collect();
    let max = grid2.iter().flatten().copied().fold(0.0, f64::max);
    let total: f64 = grid2.iter().flatten().sum();
    let centre9: f64 = (2..5).flat_map(|r| (2..5).map(move |c| (r, c))).map(|(r, c)| grid2[r][c]).sum();
    let mut s = Svg::new(
        720,
        330,
        "Pixels near the centre of a receptive field reach the output by many more paths",
        "A 7 by 7 heat grid of path counts from each input pixel to one output after three layers of 3 by 3 kernels, 1 in the corners rising to 49 in the centre, and a bar chart of the middle row.",
    ).min_text(12);
    let cw = 34.0;
    s.text_bold(30.0, 30.0, "paths to one output after three 3 × 3 layers", 12, tok::INK, Anchor::Start);
    s.cells(30.0, 46.0, cw, &shaded(&grid2, max, 0));
    s.text(30.0, 46.0 + 7.0 * cw + 18.0, &format!("corner pixel: {} path; centre pixel: {} paths", numt(grid2[0][0], 0), numt(max, 0)), 11, tok::INK2, Anchor::Start);
    s.text(30.0, 46.0 + 7.0 * cw + 32.0, &format!("the middle 3 × 3 holds {} of all {} paths ({}%)", numt(centre9, 0), numt(total, 0), num(centre9 / total * 100.0, 0)), 11, tok::INK2, Anchor::Start);
    let mid = &grid2[3];
    let pl = Plot { x0: 350.0, y0: 60.0, w: 330.0, h: 170.0, xmin: 0.5, xmax: 7.5, ymin: 0.0, ymax: max * 1.1 };
    pl.frame(&mut s, &[0.0, 10.0, 20.0, 30.0, 40.0, 50.0], 0, &[1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0], 0, "position along the middle row", "paths");
    for (i, v) in mid.iter().enumerate() {
        let xc = i as f64 + 1.0;
        s.rect(pl.px(xc - 0.3), pl.py(*v), pl.px(xc + 0.3) - pl.px(xc - 0.3), pl.bottom() - pl.py(*v), tok::S1, None);
        s.text(pl.px(xc), pl.py(*v) - 5.0, &numt(*v, 0), 11, tok::INK, Anchor::Middle);
    }
    s.text(350.0, 30.0, "With all weights equal, influence is the path count.", 11, tok::INK2, Anchor::Start);
    ("b21-2-effective-rf".to_string(), s.finish())
}

fn b21_hierarchy() -> Figure {
    let net = [(3usize, 1usize), (3, 1), (2, 2), (3, 1), (3, 1), (2, 2), (3, 1), (3, 1), (2, 2), (3, 1), (3, 1)];
    let rfs = calc::receptive(&net);
    let stages: [(usize, &str, &str); 4] = [
        (2, "edges", "strokes of letters, the border of a picture"),
        (5, "corners and contours", "where strokes meet and bend"),
        (8, "object parts", "whole letters, shapes in the picture"),
        (11, "objects", "the title block, the picture: a cookbook cover"),
    ];
    let cover_px = 64.0;
    let scale = 3.0;
    let mut s = Svg::new(
        720,
        360,
        "Deeper layers see larger regions and respond to larger, more complex patterns",
        "Left, a 64 by 64 pixel cover drawn to scale with nested squares for the regions seen after layers 2, 5, 8 and 11 of a small network; right, a ladder from edges to objects with each stage's receptive-field size.",
    ).min_text(12);
    let (cx, cy) = (40.0, 70.0);
    s.text_bold(cx, 40.0, "a 64 × 64 cover, drawn to scale", 12, tok::INK, Anchor::Start);
    s.rect(cx, cy, cover_px * scale, cover_px * scale, tok::NEUTRAL, Some(tok::AXIS));
    let colours = [tok::S1, tok::S3, tok::S2, tok::INK2];
    let mid = cx + cover_px * scale / 2.0;
    let midy = cy + cover_px * scale / 2.0;
    for (n, (layer, _, _)) in stages.iter().enumerate() {
        let r = rfs[layer - 1].0 as f64;
        let side = r * scale;
        s.rect_bold(mid - side / 2.0, midy - side / 2.0, side, side, "none", colours[n]);
    }
    s.text(cx, cy + cover_px * scale + 20.0, "the largest square spills past the cover:", 11, tok::INK2, Anchor::Start);
    s.text(cx, cy + cover_px * scale + 34.0, "its unit sees the whole cover and some padding", 11, tok::INK2, Anchor::Start);
    let lx = 330.0;
    for (n, (layer, name, detail)) in stages.iter().enumerate() {
        let y = 290.0 - n as f64 * 72.0;
        let r = rfs[layer - 1].0;
        s.rect_bold(lx, y - 30.0, 360.0, 50.0, tok::SURFACE, colours[n]);
        s.text_bold(lx + 12.0, y - 10.0, &format!("after layer {layer}: {name}"), 12, tok::INK, Anchor::Start);
        s.text(lx + 12.0, y + 8.0, detail, 11, tok::INK2, Anchor::Start);
        s.text_bold(lx + 348.0, y - 10.0, &format!("{r} × {r}"), 12, colours[n], Anchor::End);
        if n + 1 < stages.len() {
            s.arrow(lx + 180.0, y - 31.0, lx + 180.0, y - 51.0, None);
        }
    }
    s.text(lx, 336.0, "Network: pairs of 3 × 3 layers, 2 × 2 pooling between.", 11, tok::INK2, Anchor::Start);
    ("b21-3-hierarchy".to_string(), s.finish())
}

fn b21_tradeoff() -> Figure {
    let c: u64 = 64;
    let rows: [(&str, u64, u64, u64); 4] = [
        ("one 5 × 5 layer", 5, 1, 25),
        ("two 3 × 3 layers", 5, 2, 18),
        ("one 7 × 7 layer", 7, 1, 49),
        ("three 3 × 3 layers", 7, 3, 27),
    ];
    let mut s = Svg::new(
        720,
        310,
        "Stacks of small kernels see as far as one large kernel with fewer weights and more nonlinearities",
        "A four-row table comparing one 5 by 5 layer with two 3 by 3 layers and one 7 by 7 layer with three 3 by 3 layers: the receptive field, weights per channel pair, weights for 64 channels in and out drawn as bars, and the number of nonlinearities.",
    ).min_text(12);
    let heads = [(30.0, "stack"), (200.0, "sees"), (262.0, "weights per"), (380.0, "weights, 64 channels in and out"), (640.0, "ReLUs")];
    for (x, h) in heads {
        s.text_bold(x, 40.0, h, 12, tok::INK2, Anchor::Start);
    }
    s.text_bold(262.0, 54.0, "channel pair", 12, tok::INK2, Anchor::Start);
    s.line(30.0, 62.0, 700.0, 62.0, tok::AXIS, 1.0);
    let top = 49 * c * c;
    for (n, (name, rf, layers, w)) in rows.iter().enumerate() {
        let y = 70.0 + n as f64 * 44.0 + if n >= 2 { 14.0 } else { 0.0 };
        s.text(30.0, y + 22.0, name, 12, tok::INK, Anchor::Start);
        s.text(200.0, y + 22.0, &format!("{rf} × {rf}"), 12, tok::INK, Anchor::Start);
        s.text(262.0, y + 22.0, &w.to_string(), 12, tok::INK, Anchor::Start);
        let total = w * c * c;
        let bw = total as f64 / top as f64 * 150.0;
        s.rect(380.0, y + 8.0, bw, 20.0, if layers > &1 { tok::S1 } else { tok::S2 }, None);
        s.text(380.0 + bw + 8.0, y + 22.0, &thousands(total), 12, tok::INK, Anchor::Start);
        s.text(640.0, y + 22.0, &layers.to_string(), 12, tok::INK, Anchor::Start);
    }
    let save5 = 1.0 - 18.0 / 25.0;
    let more7 = 49.0 / 27.0 - 1.0;
    s.text(30.0, 266.0, &format!("Two 3 × 3 layers use {}% fewer weights than one 5 × 5; one 7 × 7 needs {}% more", num(save5 * 100.0, 0), num(more7 * 100.0, 0)), 11, tok::INK2, Anchor::Start);
    s.text(30.0, 280.0, "than three 3 × 3 (the VGG paper's figure). Each extra layer also adds a ReLU,", 11, tok::INK2, Anchor::Start);
    s.text(30.0, 294.0, "so the stack can bend its response more than one layer can.", 11, tok::INK2, Anchor::Start);
    ("b21-4-tradeoff".to_string(), s.finish())
}

// ====================================================================================================
// Chapter 22. 1×1 convolutions, dilation and separable convolutions
// ====================================================================================================

/// The 1 × 1 filters of Figure 22.1: two output channels, each a weighted sum of the red, green and blue values.
fn one_by_one_weights() -> calc::Grid {
    vec![vec![1.0, 1.0, 0.0], vec![0.0, -1.0, 2.0]]
}

fn b22_one_by_one() -> Figure {
    let planes = calc::rgb_crop();
    let w = one_by_one_weights();
    let outs: Vec<calc::Grid> = w
        .iter()
        .map(|row| (0..4).map(|r| (0..4).map(|c| (0..3).map(|ch| row[ch] * planes[ch][r][c]).sum()).collect()).collect())
        .collect();
    let (pr, pc) = (0usize, 0usize);
    let pix: Vec<f64> = (0..3).map(|ch| planes[ch][pr][pc]).collect();
    let pout: Vec<f64> = w.iter().map(|row| row.iter().zip(&pix).map(|(a, b)| a * b).sum()).collect();
    let mut s = Svg::new(
        720,
        356,
        "A 1 by 1 convolution mixes the channels at each pixel and leaves the pixels where they are",
        "Left, the three 4 by 4 colour planes with the top-left pixel outlined; middle, that pixel's three values multiplied by a 2 by 3 weight matrix to give two numbers; right, the two 4 by 4 output planes that result from doing the same at every pixel.",
    ).min_text(12);
    let names = [("red", tok::S2), ("green", tok::S3), ("blue", tok::S1)];
    let cw = 20.0;
    for (ch, (name, colour)) in names.iter().enumerate() {
        let y = 40.0 + ch as f64 * 100.0;
        s.text_bold(30.0, y - 6.0, name, 11, colour, Anchor::Start);
        s.cells(30.0, y, cw, &grid(&planes[ch], 0));
        s.rect_bold(30.0 + pc as f64 * cw, y + pr as f64 * cw, cw, cw, "none", tok::S2);
    }
    let mx = 190.0;
    s.text_bold(mx, 44.0, "one pixel, three channels", 12, tok::INK, Anchor::Start);
    s.cells_wh(mx, 76.0, 34.0, 26.0, &pix.iter().map(|v| vec![cell(numt(*v, 0)).fill(tok::FILL1)]).collect::<Vec<_>>());
    s.text(mx + 50.0, 118.0, "×", 16, tok::INK2, Anchor::Middle);
    s.text(mx + 64.0, 70.0, "weights: 2 outputs × 3 channels", 11, tok::INK2, Anchor::Start);
    s.cells_wh(mx + 64.0, 76.0, 34.0, 26.0, &grid(&w, 0));
    s.text(mx + 180.0, 104.0, "=", 16, tok::INK2, Anchor::Middle);
    s.cells_wh(mx + 196.0, 76.0, 34.0, 26.0, &pout.iter().map(|v| vec![cell(numt(*v, 0)).fill(tok::FILL1)]).collect::<Vec<_>>());
    s.text(mx, 170.0, &format!("first output: {}·{} + {}·{} + {}·{} = {}", numt(w[0][0], 0), numt(pix[0], 0), numt(w[0][1], 0), numt(pix[1], 0), numt(w[0][2], 0), numt(pix[2], 0), numt(pout[0], 0)), 11, tok::INK2, Anchor::Start);
    s.text(mx, 186.0, &format!("second output: {}·{} + ({})·{} + {}·{} = {}", numt(w[1][0], 0), numt(pix[0], 0), numt(w[1][1], 0), numt(pix[1], 0), numt(w[1][2], 0), numt(pix[2], 0), numt(pout[1], 0)), 11, tok::INK2, Anchor::Start);
    s.text(mx, 214.0, "The same 6 weights are applied at all 16 pixels:", 11, tok::INK2, Anchor::Start);
    s.text(mx, 228.0, "a small dense layer across channels, per pixel.", 11, tok::INK2, Anchor::Start);
    for (o, g) in outs.iter().enumerate() {
        let x = 540.0;
        let y = 40.0 + o as f64 * 140.0;
        s.text_bold(x, y - 6.0, &format!("output channel {}", o + 1), 11, tok::INK, Anchor::Start);
        s.cells(x, y, 26.0, &signed(g, 8.0));
        s.rect_bold(x, y, 26.0, 26.0, "none", tok::S2);
    }
    s.text(30.0, 340.0, "3 input channels become 2 output channels; the 4 × 4 layout is unchanged.", 11, tok::INK2, Anchor::Start);
    ("b22-1-one-by-one".to_string(), s.finish())
}

fn b22_inception() -> Figure {
    let (h, c_in, c_mid, c_out) = (28u64, 256u64, 64u64, 128u64);
    let direct = h * h * 3 * 3 * c_in * c_out;
    let reduce = h * h * c_in * c_mid;
    let after = h * h * 3 * 3 * c_mid * c_out;
    let total = reduce + after;
    let mut s = Svg::new(
        720,
        300,
        "A 1 by 1 reduction before a 3 by 3 convolution cuts the multiplications to less than a third",
        "Top, a 3 by 3 convolution straight from 256 channels to 128 with its multiplication count as a long orange bar; bottom, a 1 by 1 convolution down to 64 channels followed by the 3 by 3 convolution, with a two-part blue bar about a third as long.",
    ).min_text(12);
    let bar = |n: u64| n as f64 / direct as f64 * 380.0;
    s.text_bold(30.0, 34.0, &format!("input: {h} × {h} × {c_in}; output: {h} × {h} × {c_out}"), 12, tok::INK, Anchor::Start);
    s.labelled_box2(30.0, 60.0, 180.0, 44.0, "3 × 3 convolution", &format!("{c_in} → {c_out} channels"), tok::NEUTRAL, tok::GRID);
    s.rect(240.0, 70.0, bar(direct), 24.0, tok::S2, None);
    s.text(240.0 + bar(direct) + 8.0, 87.0, &thousands(direct), 12, tok::INK, Anchor::Start);
    s.labelled_box2(30.0, 150.0, 180.0, 44.0, "1 × 1 convolution", &format!("{c_in} → {c_mid} channels"), tok::FILL1, tok::S1);
    s.labelled_box2(30.0, 210.0, 180.0, 44.0, "3 × 3 convolution", &format!("{c_mid} → {c_out} channels"), tok::NEUTRAL, tok::GRID);
    s.arrow(120.0, 195.0, 120.0, 208.0, None);
    s.rect(240.0, 160.0, bar(reduce), 24.0, tok::FILL3, None);
    s.text(240.0 + bar(reduce) + 8.0, 177.0, &thousands(reduce), 12, tok::INK, Anchor::Start);
    s.rect(240.0, 220.0, bar(after), 24.0, tok::S1, None);
    s.text(240.0 + bar(after) + 8.0, 237.0, &thousands(after), 12, tok::INK, Anchor::Start);
    s.text(30.0, 282.0, &format!("Multiplications: {} with the reduction against {} without, {}% of the direct cost.", thousands(total), thousands(direct), num(total as f64 / direct as f64 * 100.0, 1)), 11, tok::INK2, Anchor::Start);
    s.text(240.0, 138.0, "multiplications per image", 11, tok::INK2, Anchor::Start);
    ("b22-2-inception-branch".to_string(), s.finish())
}

fn b22_dilation() -> Figure {
    let dilations = [1usize, 2, 4];
    let stacked: Vec<(usize, usize)> = dilations.iter().map(|d| (d * 2 + 1, 1)).collect();
    let rfs = calc::receptive(&stacked);
    let mut s = Svg::new(
        720,
        320,
        "Dilation spreads a 3 by 3 kernel's taps apart, widening what one layer sees without adding weights",
        "Three 9 by 9 grids with the nine taps of a 3 by 3 kernel filled in blue at dilation 1, 2 and 4, spanning 3, 5 and 9 pixels; below, the receptive field of the three layers stacked, 3, 7 and 15 pixels.",
    ).min_text(12);
    let cw = 16.0;
    for (n, d) in dilations.iter().enumerate() {
        let x0 = 40.0 + n as f64 * 224.0;
        let span = d * 2 + 1;
        let off = (9 - span) / 2;
        s.text_bold(x0, 34.0, &format!("dilation {d}: spans {span} × {span}"), 12, tok::INK, Anchor::Start);
        let cells: Vec<Vec<Cell>> = (0..9)
            .map(|r| {
                (0..9)
                    .map(|c| {
                        let tap = r >= off && c >= off && (r - off) % d == 0 && (c - off) % d == 0 && (r - off) / d < 3 && (c - off) / d < 3;
                        if tap { cell("").fill(tok::S1) } else { cell("").fill(tok::SURFACE) }
                    })
                    .collect()
            })
            .collect();
        s.cells(x0, 50.0, cw, &cells);
        s.rect_bold(x0 + off as f64 * cw, 50.0 + off as f64 * cw, span as f64 * cw, span as f64 * cw, "none", tok::S2);
        s.text(x0, 50.0 + 9.0 * cw + 18.0, &format!("9 weights; gaps of {} between taps", d - 1), 11, tok::INK2, Anchor::Start);
    }
    s.text_bold(40.0, 240.0, "stacked, with stride 1 throughout:", 12, tok::INK, Anchor::Start);
    let desc: Vec<String> = dilations.iter().zip(&rfs).map(|(d, (r, _))| format!("after dilation {d}: {r} × {r}")).collect();
    s.text(40.0, 262.0, &desc.join("; "), 12, tok::INK, Anchor::Start);
    s.text(40.0, 290.0, "Doubling the dilation at each layer doubles the reach each time, with the image never shrunk.", 11, tok::INK2, Anchor::Start);
    ("b22-3-dilation".to_string(), s.finish())
}

fn b22_separable() -> Figure {
    let (dk, m, n, df) = (3u64, 64u64, 128u64, 56u64);
    let standard = dk * dk * m * n * df * df;
    let depthwise = dk * dk * m * df * df;
    let pointwise = m * n * df * df;
    let ratio = (depthwise + pointwise) as f64 / standard as f64;
    let formula = 1.0 / n as f64 + 1.0 / (dk * dk) as f64;
    let (w_std, w_sep) = (dk * dk * m * n, dk * dk * m + m * n);
    let col = [1.0, 2.0, 1.0];
    let row = [1.0, 0.0, -1.0];
    let outer: calc::Grid = col.iter().map(|a| row.iter().map(|b| a * b).collect()).collect();
    let matches = outer == calc::sobel();
    let mut s = Svg::new(
        720,
        340,
        "Separable convolutions: a depthwise pass then a pointwise pass, and a kernel written as a column times a row",
        "Left, bars comparing the multiplications of a standard 3 by 3 convolution with those of its depthwise and pointwise parts; right, the Sobel kernel built as a column of 1, 2, 1 times a row of 1, 0, minus 1.",
    ).min_text(12);
    let bar = |v: u64| v as f64 / standard as f64 * 300.0;
    s.text_bold(30.0, 34.0, &format!("{df} × {df} × {m} in, {n} out, 3 × 3 kernels"), 12, tok::INK, Anchor::Start);
    s.text(30.0, 64.0, "standard convolution", 12, tok::INK, Anchor::Start);
    s.rect(30.0, 72.0, bar(standard), 22.0, tok::S2, None);
    s.text(30.0, 110.0, &thousands(standard), 12, tok::INK, Anchor::Start);
    s.text(30.0, 144.0, "depthwise (one 3 × 3 kernel per channel)", 12, tok::INK, Anchor::Start);
    s.rect(30.0, 152.0, bar(depthwise).max(1.0), 22.0, tok::FILL3, None);
    s.text(40.0 + bar(depthwise), 168.0, &thousands(depthwise), 12, tok::INK, Anchor::Start);
    s.text(30.0, 200.0, "pointwise (1 × 1, 64 → 128)", 12, tok::INK, Anchor::Start);
    s.rect(30.0, 208.0, bar(pointwise), 22.0, tok::S1, None);
    s.text(40.0 + bar(pointwise), 224.0, &thousands(pointwise), 12, tok::INK, Anchor::Start);
    s.text(30.0, 262.0, &format!("together {} of the standard cost: 1/{} + 1/{} = {}", num(ratio, 3), n, dk * dk, num(formula, 3)), 11, tok::INK2, Anchor::Start);
    s.text(30.0, 278.0, &format!("weights: {} against {}", thousands(w_sep), thousands(w_std)), 11, tok::INK2, Anchor::Start);
    let (rx, ry) = (440.0, 70.0);
    s.text_bold(rx, 34.0, "a spatially separable kernel", 12, tok::INK, Anchor::Start);
    s.cells(rx, ry + 30.0, 26.0, &col.iter().map(|v| vec![cell(numt(*v, 0)).fill(tok::FILL1)]).collect::<Vec<_>>());
    s.text(rx + 44.0, ry + 74.0, "×", 16, tok::INK2, Anchor::Middle);
    s.cells(rx + 60.0, ry, 26.0, &[row.iter().map(|v| cell(numt(*v, 0)).fill(tok::FILL1)).collect()]);
    s.text(rx + 158.0, ry + 74.0, "=", 16, tok::INK2, Anchor::Middle);
    s.cells(rx + 176.0, ry + 30.0, 26.0, &signed(&outer, 2.0));
    s.text(rx, ry + 136.0, if matches { "the Sobel kernel of Figure 19.2" } else { "a product kernel" }, 11, tok::INK2, Anchor::Start);
    s.text(rx, ry + 152.0, "Two passes of 3 taps (6 multiplications", 11, tok::INK2, Anchor::Start);
    s.text(rx, ry + 166.0, "per pixel) replace one pass of 9.", 11, tok::INK2, Anchor::Start);
    ("b22-4-separable".to_string(), s.finish())
}

// ====================================================================================================
// Chapter 23. Squeeze-and-Excitation, skip connections, and what CNNs get wrong
// ====================================================================================================

/// Three 2 × 2 feature maps and the two small weight layers of the squeeze-and-excitation example (reduction 3 → 1 → 3).
fn se_maps() -> [calc::Grid; 3] {
    [vec![vec![4.0, 2.0], vec![6.0, 4.0]], vec![vec![1.0, 3.0], vec![2.0, 2.0]], vec![vec![0.0, 2.0], vec![1.0, 1.0]]]
}
const SE_W1: [f64; 3] = [0.5, -0.25, 0.25];
const SE_W2: [f64; 3] = [1.2, -0.6, 0.2];

fn b23_se() -> Figure {
    let maps = se_maps();
    let z: Vec<f64> = maps.iter().map(|m| m.iter().flatten().sum::<f64>() / 4.0).collect();
    let hidden = z.iter().zip(SE_W1).map(|(a, w)| a * w).sum::<f64>().max(0.0);
    let logits: Vec<f64> = SE_W2.iter().map(|w| w * hidden).collect();
    let gates: Vec<f64> = logits.iter().map(|l| calc::sigmoid(*l)).collect();
    let scaled: Vec<calc::Grid> = maps.iter().zip(&gates).map(|(m, g)| m.iter().map(|r| r.iter().map(|v| v * g).collect()).collect()).collect();
    let mut s = Svg::new(
        720,
        330,
        "Squeeze-and-excitation: average each channel, pass the averages through two small layers, and rescale each channel by its gate",
        "Left, three 2 by 2 channel maps; then a column of their three averages, a single hidden number, three gate bars between 0 and 1, and on the right the three maps multiplied by their gates.",
    ).min_text(12);
    let colours = [tok::S1, tok::S2, tok::S3];
    let mid = 150.0;
    s.text_bold(30.0, 30.0, "channels", 12, tok::INK, Anchor::Start);
    for (c, m) in maps.iter().enumerate() {
        let y = 56.0 + c as f64 * 80.0;
        s.text(30.0, y - 6.0, &format!("channel {}", c + 1), 11, colours[c], Anchor::Start);
        s.cells(30.0, y, 28.0, &grid(m, 0));
        s.rect_bold(30.0, y, 56.0, 56.0, "none", colours[c]);
    }
    let sx = 130.0;
    s.text_bold(sx, 30.0, "squeeze", 12, tok::INK, Anchor::Start);
    s.cells_wh(sx, mid - 45.0, 40.0, 30.0, &z.iter().map(|v| vec![cell(numt(*v, 2)).fill(tok::FILL1)]).collect::<Vec<_>>());
    s.text(sx, mid + 66.0, "mean of", 11, tok::INK2, Anchor::Start);
    s.text(sx, mid + 80.0, "each map", 11, tok::INK2, Anchor::Start);
    s.arrow(96.0, mid, sx - 6.0, mid, None);
    let hx = 230.0;
    s.text_bold(hx, 30.0, "excite", 12, tok::INK, Anchor::Start);
    s.cells_wh(hx, mid - 15.0, 44.0, 30.0, &[vec![cell(numt(hidden, 2)).fill(tok::FILL1)]]);
    s.arrow(sx + 46.0, mid, hx - 6.0, mid, None);
    s.text(hx - 10.0, mid + 40.0, "ReLU(w₁·z)", 11, tok::INK2, Anchor::Start);
    s.text(hx - 10.0, mid + 54.0, &format!("w₁ = ({})", SE_W1.iter().map(|w| numt(*w, 2)).collect::<Vec<_>>().join(", ")), 11, tok::INK2, Anchor::Start);
    let gx = 360.0;
    s.text_bold(gx, 30.0, "gates in (0, 1)", 12, tok::INK, Anchor::Start);
    s.arrow(hx + 50.0, mid, gx - 6.0, mid, None);
    for (c, g) in gates.iter().enumerate() {
        let y = mid - 50.0 + c as f64 * 36.0;
        s.rect(gx, y, 110.0, 22.0, tok::SURFACE, Some(tok::GRID));
        s.rect(gx, y, 110.0 * g, 22.0, colours[c], None);
        s.text(gx + 116.0, y + 15.0, &numt(*g, 3), 11, tok::INK, Anchor::Start);
    }
    s.text(gx, mid + 68.0, "sigmoid(w₂ × hidden)", 11, tok::INK2, Anchor::Start);
    s.text(gx, mid + 82.0, &format!("w₂ = ({})", SE_W2.iter().map(|w| numt(*w, 2)).collect::<Vec<_>>().join(", ")), 11, tok::INK2, Anchor::Start);
    let ox = 560.0;
    s.text_bold(ox, 30.0, "scaled channels", 12, tok::INK, Anchor::Start);
    s.arrow(gx + 160.0, mid, ox - 6.0, mid, None);
    for (c, m) in scaled.iter().enumerate() {
        let y = 56.0 + c as f64 * 80.0;
        s.text(ox, y - 6.0, &format!("× {}", numt(gates[c], 3)), 11, colours[c], Anchor::Start);
        s.cells(ox, y, 30.0, &grid(m, 2));
        s.rect_bold(ox, y, 60.0, 60.0, "none", colours[c]);
    }
    s.text(30.0, 316.0, "All positions of a channel share one gate; the gates are independent and need not sum to 1.", 11, tok::INK2, Anchor::Start);
    ("b23-1-se-block".to_string(), s.finish())
}

fn b23_residual() -> Figure {
    let identity: calc::Grid = (0..3).map(|r| (0..3).map(|c| if r == 1 && c == 1 { 1.0 } else { 0.0 }).collect()).collect();
    let zero: calc::Grid = vec![vec![0.0; 3]; 3];
    let mut s = Svg::new(
        720,
        360,
        "A residual block adds its input back to its output, so doing nothing only needs zero weights",
        "Left, a plain block of two convolutions; right, the same block with a skip connection from its input to an addition after the second convolution; beside each, the 3 by 3 kernel it must learn to pass its input through unchanged.",
    ).min_text(12);
    let bw = 180.0;
    let draw_block = |s: &mut Svg, x: f64, residual: bool| {
        s.labelled_box(x, 60.0, bw, 30.0, "input x", tok::SURFACE, tok::GRID);
        s.arrow(x + bw / 2.0, 92.0, x + bw / 2.0, 108.0, None);
        s.labelled_box(x, 110.0, bw, 30.0, "3 × 3 convolution, ReLU", tok::NEUTRAL, tok::GRID);
        s.arrow(x + bw / 2.0, 142.0, x + bw / 2.0, 158.0, None);
        s.labelled_box(x, 160.0, bw, 30.0, "3 × 3 convolution", tok::NEUTRAL, tok::GRID);
        if residual {
            s.arrow(x + bw / 2.0, 192.0, x + bw / 2.0, 206.0, None);
            s.dot(x + bw / 2.0, 218.0, 11.0, tok::SURFACE, Some(tok::INK2));
            s.text(x + bw / 2.0, 223.0, "+", 14, tok::INK, Anchor::Middle);
            s.skip(x + bw, 75.0, x + bw / 2.0 + 12.0, 218.0, 40.0, Some("x"));
            s.arrow(x + bw / 2.0, 230.0, x + bw / 2.0, 244.0, None);
            s.labelled_box(x, 246.0, bw, 30.0, "ReLU → F(x) + x", tok::FILL1, tok::S1);
        } else {
            s.arrow(x + bw / 2.0, 192.0, x + bw / 2.0, 244.0, None);
            s.labelled_box(x, 246.0, bw, 30.0, "ReLU → H(x)", tok::NEUTRAL, tok::GRID);
        }
    };
    s.text_bold(40.0, 34.0, "plain block", 12, tok::INK, Anchor::Start);
    draw_block(&mut s, 40.0, false);
    s.text_bold(380.0, 34.0, "residual block", 12, tok::INK, Anchor::Start);
    draw_block(&mut s, 380.0, true);
    s.text(240.0, 110.0, "to copy x, each", 11, tok::INK2, Anchor::Start);
    s.text(240.0, 124.0, "convolution must", 11, tok::INK2, Anchor::Start);
    s.text(240.0, 138.0, "learn this kernel", 11, tok::INK2, Anchor::Start);
    s.text(240.0, 152.0, "for its own channel:", 11, tok::INK2, Anchor::Start);
    s.cells(250.0, 160.0, 18.0, &grid(&identity, 0));
    s.text(612.0, 110.0, "to copy x, the", 11, tok::INK2, Anchor::Start);
    s.text(612.0, 124.0, "layers need", 11, tok::INK2, Anchor::Start);
    s.text(612.0, 138.0, "only zeros:", 11, tok::INK2, Anchor::Start);
    s.cells(616.0, 148.0, 18.0, &grid(&zero, 0));
    s.text(40.0, 312.0, "A residual block learns F(x) = H(x) − x, the change to make, rather than the whole mapping H(x).", 11, tok::INK2, Anchor::Start);
    s.text(40.0, 328.0, "If a layer is best left out, pushing its weights to zero is easy; building an exact copy is harder.", 11, tok::INK2, Anchor::Start);
    ("b23-2-residual".to_string(), s.finish())
}

fn b23_adversarial() -> Figure {
    let eps = 0.007;
    let m = 0.01;
    let sizes: [(u64, &str); 4] = [(36, "the 6 × 6 cover"), (28 * 28, "a 28 × 28 grey image"), (224 * 224, "224 × 224, grey"), (224 * 224 * 3, "224 × 224, colour")];
    let mut s = Svg::new(
        720,
        340,
        "A change too small to see can move a linear score a long way when there are many pixels",
        "Bars on a logarithmic axis for four image sizes, showing the largest change in a linear score when every pixel moves by 0.007 in the direction of its weight, assuming an average weight of 0.01; the change grows from about 0.003 for 36 pixels to about 10.5 for 150,528.",
    ).min_text(12);
    s.text_bold(30.0, 30.0, &format!("largest score change = ε × m × n, with ε = {} per pixel and average weight m = {}", numt(eps, 3), numt(m, 2)), 12, tok::INK, Anchor::Start);
    let (bx, bw) = (220.0, 400.0);
    let lx = |v: f64| bx + (v.log10() + 3.0) / 5.0 * bw;
    for t in -3..=2 {
        let x = bx + f64::from(t + 3) / 5.0 * bw;
        s.line(x, 50.0, x, 250.0, tok::GRID, 1.0);
        s.text(x, 266.0, &numt(10f64.powi(t), 3), 11, tok::MUTED, Anchor::Middle);
    }
    s.text(bx + bw / 2.0, 282.0, "change in the score (log scale)", 11, tok::INK2, Anchor::Middle);
    for (i, (n, name)) in sizes.iter().enumerate() {
        let y = 66.0 + i as f64 * 46.0;
        let change = eps * m * *n as f64;
        s.text(30.0, y + 16.0, &format!("{} pixels", thousands(*n)), 12, tok::INK, Anchor::Start);
        s.text(30.0, y + 30.0, name, 11, tok::INK2, Anchor::Start);
        s.rect(bx, y, lx(change) - bx, 24.0, if change >= 1.0 { tok::S2 } else { tok::S1 }, None);
        s.text(lx(change) + 8.0, y + 16.0, &num(change, 3), 12, tok::INK, Anchor::Start);
    }
    s.text(30.0, 306.0, "Goodfellow, Shlens and Szegedy used ε = 0.007 on GoogLeNet: \"panda\" (57.7%) became \"gibbon\" (99.3%).", 11, tok::INK2, Anchor::Start);
    s.text(30.0, 322.0, "The toy score here is linear, as their explanation assumes; orange bars mark changes of 1 or more.", 11, tok::INK2, Anchor::Start);
    ("b23-3-adversarial".to_string(), s.finish())
}

// ====================================================================================================
// Chapter 24. Convolution beyond images
// ====================================================================================================

/// Bars of a probability mass function starting at value `first`, drawn in a plot.
fn pmf_bars(s: &mut Svg, pl: &Plot, p: &[f64], first: usize, colour: &str, highlight: Option<usize>, label_dec: usize) {
    for (i, v) in p.iter().enumerate() {
        let x = (first + i) as f64;
        let fill = if Some(first + i) == highlight { tok::S2 } else { colour };
        s.rect(pl.px(x - 0.32), pl.py(*v), pl.px(x + 0.32) - pl.px(x - 0.32), pl.bottom() - pl.py(*v), fill, None);
        if label_dec > 0 {
            s.text(pl.px(x), pl.py(*v) - 5.0, &numt(*v, label_dec), 11, tok::INK, Anchor::Middle);
        }
    }
}

fn b24_dice() -> Figure {
    let p = calc::basket_pmf();
    let total = calc::convolve_pmf(&p, &p);
    let p5 = total[5 - 2];
    let sum_total: f64 = total.iter().sum();
    let mut s = Svg::new(
        720,
        340,
        "The number of books two customers buy together has the convolution of their two distributions",
        "Left, the distribution of books bought by one customer, drawn twice for customers A and B; right, the distribution of their total from 2 to 12, with the bar for a total of 5 in orange.",
    ).min_text(12);
    let a = Plot { x0: 50.0, y0: 50.0, w: 170.0, h: 90.0, xmin: 0.5, xmax: 6.5, ymin: 0.0, ymax: 0.4 };
    a.frame(&mut s, &[0.0, 0.2, 0.4], 1, &[1.0, 2.0, 3.0, 4.0, 5.0, 6.0], 0, "", "customer A");
    pmf_bars(&mut s, &a, &p, 1, tok::S1, None, 0);
    let b = Plot { x0: 50.0, y0: 200.0, w: 170.0, h: 90.0, xmin: 0.5, xmax: 6.5, ymin: 0.0, ymax: 0.4 };
    b.frame(&mut s, &[0.0, 0.2, 0.4], 1, &[1.0, 2.0, 3.0, 4.0, 5.0, 6.0], 0, "books bought", "customer B");
    pmf_bars(&mut s, &b, &p, 1, tok::S3, None, 0);
    s.text(250.0, 170.0, "∗", 20, tok::INK2, Anchor::Middle);
    let t = Plot { x0: 300.0, y0: 50.0, w: 404.0, h: 220.0, xmin: 1.5, xmax: 12.5, ymin: 0.0, ymax: 0.2 };
    t.frame(&mut s, &[0.0, 0.05, 0.1, 0.15, 0.2], 2, &(2..=12).map(f64::from).collect::<Vec<_>>(), 0, "books bought by the two together", "P(total)");
    pmf_bars(&mut s, &t, &total, 2, tok::FILL3, Some(5), 3);
    s.text(t.px(8.4), t.py(0.17), &format!("P(total = 5) = {}", numt(p5, 5)), 12, tok::S2, Anchor::Start);
    s.text(t.px(8.4), t.py(0.17) + 16.0, &format!("the 11 bars add up to {}", numt(sum_total, 3)), 11, tok::INK2, Anchor::Start);
    ("b24-1-basket-convolution".to_string(), s.finish())
}

fn b24_flip() -> Figure {
    let p = calc::basket_pmf();
    let totals = [3usize, 5, 7];
    let mut s = Svg::new(
        720,
        400,
        "Flip one distribution, slide it along the other, multiply the pairs that line up, and add",
        "Three frames for totals of 3, 5 and 7: customer A's probabilities for 1 to 6 books in a row; beneath, customer B's probabilities in reverse order, shifted right by two cells from one frame to the next; the products of the pairs that line up, and their sum.",
    ).min_text(12);
    let cw = 46.0;
    let col0 = 326.0;
    let colx = |c: i64| col0 + c as f64 * cw;
    for (f, st) in totals.iter().enumerate() {
        let y = 34.0 + f as f64 * 112.0;
        s.text_bold(30.0, y + 8.0, &format!("total {st}"), 12, tok::INK, Anchor::Start);
        s.text(30.0, y + 34.0, "A buys x", 11, tok::INK2, Anchor::Start);
        s.text(30.0, y + 64.0, "B buys total − x", 11, tok::INK2, Anchor::Start);
        s.text(30.0, y + 96.0, "products", 11, tok::INK2, Anchor::Start);
        for x in 1..=6usize {
            s.text(colx(x as i64 - 1) + cw / 2.0, y + 8.0, &format!("x = {x}"), 11, tok::MUTED, Anchor::Middle);
        }
        let arow: Vec<Cell> = p.iter().map(|v| cell(numt(*v, 3)).fill(tok::FILL1)).collect();
        s.cells_wh(colx(0), y + 16.0, cw, 26.0, &[arow]);
        let mut sum = 0.0;
        for yb in 1..=6usize {
            let c = *st as i64 - yb as i64 - 1;
            let inside = (0..6).contains(&c);
            let fill = if inside { tok::NEUTRAL } else { tok::SURFACE };
            let ink = if inside { tok::INK } else { tok::MUTED };
            s.cells_wh(colx(c), y + 46.0, cw, 26.0, &[vec![cell(numt(p[yb - 1], 3)).fill(fill).ink(ink)]]);
            s.text(colx(c) + cw / 2.0, y + 84.0, &format!("{yb}"), 11, tok::MUTED, Anchor::Middle);
            if inside {
                let prod = p[c as usize] * p[yb - 1];
                sum += prod;
                s.text(colx(c) + cw / 2.0, y + 100.0, &numt(prod, 4), 11, tok::INK, Anchor::Middle);
            }
        }
        s.text(colx(6) + 12.0, y + 64.0, &format!("sum = {}", numt(sum, 5)), 12, tok::S2, Anchor::Start);
    }
    s.text(30.0, 380.0, "B's row is reversed (the flip) and slides two cells right per frame; pale cells face nothing.", 11, tok::INK2, Anchor::Start);
    ("b24-2-flip-and-slide".to_string(), s.finish())
}

/// Every figure of these chapters, in chapter order.
pub fn figures() -> Vec<Figure> {
    vec![
        b12_pmf(),
        b12_pdf(),
        b12_cdf(),
        b13_histogram(),
        b13_qq(),
        b13_threshold(),
        b13_decision(),
        b14_add_test(),
        b14_scalers(),
        b14_log(),
        b14_bowls(),
        b15_steps(),
        b15_erased(),
        b15_class_specific(),
        b16_where(),
        b16_before_after(),
        b17_cover(),
        b17_filmstrip(),
        b17_padding(),
        b17_stride(),
        b17_sizes(),
        b18_sum(),
        b18_bias(),
        b18_many(),
        b19_matrices(),
        b19_sobel(),
        b19_locality(),
        b20_pooling(),
        b20_shift(),
        b21_growth(),
        b21_effective(),
        b21_hierarchy(),
        b21_tradeoff(),
        b22_one_by_one(),
        b22_inception(),
        b22_dilation(),
        b22_separable(),
        b23_se(),
        b23_residual(),
        b23_adversarial(),
        b24_dice(),
        b24_flip(),
    ]
}
