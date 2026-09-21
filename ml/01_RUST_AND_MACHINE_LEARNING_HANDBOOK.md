# Rust and Machine Learning: From Tensors to Attention

**An indexed, consolidated learning handbook — revised 19 September 2026**

This handbook brings together the 35 notes listed in the [source index](#source-index). Its purpose is to help you reason about the mathematics and the Rust implementation at the same time. Repeated explanations have been combined; useful distinctions, worked examples, and implementation traps have been retained and corrected.

Our recurring image problem is inspecting fruit on a conveyor belt. Sometimes we classify a fruit from measurements, sometimes we inspect an image for small defects, and sometimes we locate several fruits at once. For attention, we also use short sequences and tiny numerical examples. Each analogy introduces an idea; the equations and shape contracts define what the code actually does.

There are three questions to ask at every layer:

1. **What information should this operation combine?** Features within one item, nearby image locations, or distant tokens?
2. **What mathematical operation produces that result?** Write the input and output shapes and calculate a tiny example.
3. **What does the Rust program own, borrow, validate, and update?** A compiling program can still use the wrong axes, checkpoint, loss, or training mode.

Read the foundations, Rust, and Candle chapters first. Then follow the convolution path for vision or the MLP/attention path for sequences, returning to the other path afterwards. Work the examples with a pencil before running them. When you can predict a changed shape, explain a failed calculation, and fix the implementation without copying a template, you are building the expertise these notes are intended to support.

<a id="versions"></a>
## Version and evidence boundary

| Item | Basis for this revision |
|---|---|
| Review date | 19 September 2026 |
| Locally observed compiler | `rustc 1.98.1 (48a229cea 2026-09-01)` |
| Local host | `aarch64-apple-darwin` |
| Project edition | Rust 2024, as declared in `Cargo.toml` |
| Project Candle dependency | Git revision `f5838914f788d3950d0a25042cffe199d9325a9e` in `Cargo.lock`; crate version field `0.9.1` |
| Project feature selection | Metal is enabled in the existing manifest |
| API references | Immutable Candle source links where implementation matters; official Rust/PyTorch documentation and original research for the associated contracts |

Rust's compiler version and a crate's version are separate. Installing Rust 1.98.1 does not update Candle. A `version = "0.9.1"` registry dependency is also not automatically the same source as a Git commit whose package reports that version. This handbook uses the project's exact lockfile revision for Candle-specific claims and does not claim that it is the newest available Candle release.

Rust checks ownership and ordinary type rules at compile time. In this Candle API, tensor dimensions, many dtype/device requirements, and parameter names remain runtime contracts. `Tensor` is not a different Rust type for every shape. Learn both boundaries: neither substitutes for the other.

Code blocks are identified in their surrounding text as standalone programs, fragments, or mathematical pseudocode. A mathematical derivation establishes an identity under its assumptions; it does not establish GPU performance or model accuracy. The [verification note](#verification) records what was actually executed and reviewed for this edition.

For a standalone standard-library Rust example, save that one block as `example.rs`, compile with `rustc --edition=2024 example.rs -o example`, and run the resulting executable. A block using Candle needs the dependencies described in chapter 3; it cannot be compiled as a dependency-free file. Keep separate examples in separate files unless the text explicitly connects them.

<a id="contents"></a>
## Contents

- [1. Foundations: know what each number and axis means](#foundations)
    - [1.1 A tensor is an array; its axes acquire meaning from your contract](#tensor-axes)
    - [1.2 Matrix multiplication is many small weighted sums](#matrix-products)
    - [1.3 A dot product is not automatically a normalized similarity or a projection](#dot-projection)
    - [1.4 A complete gradient step: learning is a numerical change](#gradient-step)
    - [1.5 Broadcasting, reshaping, and transposing solve different problems](#broadcast-layout)
    - [1.6 Stable softmax: turn competing scores into weights](#softmax)
    - [1.7 A small, fallible Rust implementation with an explicit input contract](#softmax-rust)
    - [1.8 Log-softmax and cross-entropy: keep tiny probabilities in log space](#cross-entropy)

- [2. Rust that makes model code understandable](#rust)
    - [2.1 Ownership, borrowing, and lifetimes](#rust-ownership)
    - [2.2 Copying a value, cloning a handle, and sharing storage](#section-2-2-copying-a-value-cloning-a-handle-and-sharing-storage)
    - [2.3 Option, Result, and the unit type](#rust-errors)
    - [2.4 What the question mark really does](#section-2-4-what-the-question-mark-really-does)
    - [2.5 Boxed errors and application context](#section-2-5-boxed-errors-and-application-context)
    - [2.6 Turbofish, inference, and collect](#rust-generics)
    - [2.7 Traits describe behavior, not tensor correctness](#section-2-7-traits-describe-behavior-not-tensor-correctness)

- [3. Candle: tensors, parameters, and checkpoints](#candle)
    - [3.1 A tensor carries more than values](#candle-tensors)
    - [3.2 Shared handles, copies, and gradient history](#section-3-2-shared-handles-copies-and-gradient-history)
    - [3.3 Module and ModuleT](#section-3-3-module-and-modulet)
    - [3.4 Var, VarMap, and VarBuilder have different jobs](#candle-parameters)
    - [3.5 Names are part of model behavior](#section-3-5-names-are-part-of-model-behavior)
    - [3.6 Initializers are explicit choices](#section-3-6-initializers-are-explicit-choices)
    - [3.7 A small, complete CPU learning program](#candle-training-example)
    - [3.8 Freezing is a choice about more than one kind of state](#section-3-8-freezing-is-a-choice-about-more-than-one-kind-of-state)
    - [3.9 Loading for inference and restoring trainable variables](#candle-checkpoints)
    - [3.10 A weight file is only part of a checkpoint](#section-3-10-a-weight-file-is-only-part-of-a-checkpoint)
    - [3.11 Translating parameter management from PyTorch](#section-3-11-translating-parameter-management-from-pytorch)

- [4. An MLP: combining measurements and learning from mistakes](#mlp)
    - [4.1 From one neuron to a batch of predictions](#mlp-shapes)
    - [4.2 Why another linear layer is not enough](#mlp-nonlinearity)
    - [4.3 Let the prediction task determine the output](#mlp-output-loss)
    - [4.4 One complete forward, backward and update calculation](#mlp-backprop)
    - [4.5 Turning the arithmetic into a useful training process](#mlp-training)
    - [4.6 Initialization, optimizers and regularization](#mlp-optimization)
    - [4.7 What the advanced options actually change](#mlp-compression)
    - [4.8 Practice, with worked answers](#mlp-practice)

- [5. Convolution: learning local patterns and mixing channels](#conv)
    - [5.1 One output value, worked from the beginning](#conv-mechanics)
    - [5.2 Why a 1×1 convolution does useful work](#conv-pointwise)
    - [5.3 Count weights, activations, and arithmetic separately](#conv-counts)
    - [5.4 Grouped convolution controls who can talk to whom](#conv-groups)
    - [5.5 Depthwise and depthwise separable convolution](#conv-depthwise)
    - [5.6 How the pieces form useful architectures](#conv-bottlenecks)
    - [5.7 Express the connectivity in Candle](#conv-candle)
    - [5.8 Check your understanding](#conv-practice)

- [6. Context, resolution, dilation, and transposed convolution](#spatial)
    - [6.1 Derive the output shape instead of guessing](#spatial-shapes)
    - [6.2 Track both the receptive field and the spacing](#spatial-receptive-fields)
    - [6.3 Gridding is about the connections of one output](#spatial-gridding)
    - [6.4 A transposed convolution reverses connections, not lost information](#spatial-transpose)
    - [6.5 Stride, output padding, and checkerboards](#spatial-upsampling)
    - [6.6 Several ways to combine detail and context](#spatial-architectures)
    - [6.7 Check your understanding](#spatial-practice)

- [7. Normalization and folding BatchNorm into convolution](#normalization)
    - [7.1 BatchNorm's population is defined by axes](#normalization-batch)
    - [7.2 Training statistics and inference statistics are different objects](#normalization-running)
    - [7.3 What BatchNorm helps with, and what the evidence does not prove](#normalization-why)
    - [7.4 Choose normalization by its reduction axes](#normalization-alternatives)
    - [7.5 Fold a fixed BatchNorm into the preceding convolution](#normalization-fusion)
    - [7.6 The pinned Candle API and its limits](#normalization-candle)
    - [7.7 Check your understanding](#normalization-practice)

- [8. Squeeze-and-Excitation: let the image influence its channel gates](#se)
    - [8.1 Squeeze, excite, scale](#se-mechanics)
    - [8.2 A complete two-channel example](#se-example)
    - [8.3 Count the cost and choose the hidden width explicitly](#se-cost)
    - [8.4 A source-aligned Candle implementation](#se-candle)
    - [8.5 Placement, variants, and diagnostics](#se-variants)
    - [8.6 Check your understanding](#se-practice)

- [9. Attention: deciding which other items contribute](#attention)
    - [9.1 Queries, keys and values are learned calculations](#attention-qkv)
    - [9.2 Read the equation through its shapes](#attention-shapes)
    - [9.3 A complete numerical attention example](#attention-numerical)
    - [9.4 Why divide by the square root of the key width?](#attention-scaling)
    - [9.5 Masks define which information is available](#attention-masks)
    - [9.6 The three branches learn together](#attention-gradients)
    - [9.7 Practice, with worked answers](#attention-practice)

- [10. Multiple heads and the Transformer block](#transformer)
    - [10.1 Several blends before combining the results](#transformer-heads)
    - [10.2 Follow an actual shape through the whole operation](#transformer-layout)
    - [10.3 Count parameters, bytes and operations separately](#transformer-cost)
    - [10.4 Where order enters](#transformer-position)
    - [10.5 Attention is one part of the block](#transformer-block)
    - [10.6 Parallel training and sequential generation coexist](#transformer-decoding)
    - [10.7 Practice, with worked answers](#transformer-practice)

- [11. From image features to YOLOv10's partial attention](#yolo)
    - [11.1 An image location can be a token](#yolo-tokens)
    - [11.2 Detection needs categories and locations](#yolo-detector)
    - [11.3 What “partial” means in PSA](#yolo-psa)
    - [11.4 Why the attention is placed at low resolution](#yolo-resolution)
    - [11.5 Why removing NMS is a training-design question](#yolo-assignments)
    - [11.6 Practice, with worked answers](#yolo-practice)

- [12. Turn the chapters into skills you can use](#practice)
    - [12.1 Rust: own the data, expose the errors, make the contract visible](#practice-rust)
    - [12.2 MLP: predict a gradient before trusting automatic differentiation](#practice-learning)
    - [12.3 Vision: choose which information to mix](#practice-vision)
    - [12.4 Attention: make every axis earn its place](#practice-attention)
    - [12.5 A diagnostic sequence for Rust ML programs](#diagnosis)
    - [12.6 Questions you should be able to answer without the notes](#mastery-questions)

- [Appendix A. A working glossary](#glossary)

- [Appendix B. What changed from the original notes](#corrections)

- [Appendix C. Index of all 35 original notes](#source-index)

- [Appendix D. Primary-source reading routes](#reading-index)

- [Appendix E. Verification and review for this edition](#verification)

<a id="foundations"></a>
## 1. Foundations: know what each number and axis means

Imagine a camera above a conveyor belt carrying apples, bananas, and oranges. You may want to classify a cropped fruit, locate every fruit in a photograph, or label every pixel belonging to a bruise. These are different jobs. Classification needs a decision for an item; detection needs locations and classes; segmentation needs a decision at each pixel. The same mathematical building blocks serve all three, but their shapes and outputs differ.

The most useful habit is to describe an operation twice: first in ordinary language, then as a shape equation. If the two descriptions disagree, stop before writing more code.

<a id="tensor-axes"></a>
### 1.1 A tensor is an array; its axes acquire meaning from your contract

A scalar is one number. A vector is a one-dimensional array. A matrix is a two-dimensional array. In this handbook, *tensor* means the numerical array used by the ML library; its rank is the number of axes. This use of “rank” is different from matrix rank in linear algebra.

| Data | Example shape | What one element means |
|---|---|---|
| Three measured features of one fruit | `[3]` | One feature value |
| A batch of 8 fruits, each with 3 features | `[8, 3]` | `x[item, feature]` |
| Four RGB images, 32 pixels high and wide | `[4, 3, 32, 32]` | `x[image, channel, row, column]` |
| Two sentences, each padded to 5 tokens of width 16 | `[2, 5, 16]` | `x[sentence, token, feature]` |

We use `B` for batch size, `C` for channels, `H` and `W` for image height and width, `T` for sequence length, and `D` for feature width. `[B,C,H,W]` is often called NCHW; N means the batch axis. `[B,H,W,C]` is a different arrangement. The letters are documentation, not properties that a raw tensor automatically understands.

The original matrix note said that Candle uses rows to count items or neurons. That is a useful observation for certain matrices, not a universal rule. In a data matrix `[B,D]`, rows are examples. In a linear weight matrix `[O,D]`, rows are output units. In attention scores `[T,T]`, rows are queries and columns are keys. A square shape does not tell you which meaning is intended.

Write the meaning beside each shape when debugging. A tensor of `[32,32]` could be 32 images with 32 features, one grayscale image, or an attention matrix. Shape equality alone cannot establish semantic correctness.

<a id="matrix-products"></a>
### 1.2 Matrix multiplication is many small weighted sums

Suppose three measured fruit features are redness, roundness, and elongation. Our numbers are toy measurements, not a trained fruit classifier. Two items form the rows of `X`. Each row of `W` defines one scoring rule, and `b` shifts its score:

```text
X [2,3] = [2  1  0]     W [2,3] = [1   2  -1]     b [2] = [0.5  -0.5]
          [0  1  3]               [0  -1   2]
```

The output is `Y = X Wᵀ + b`. `ᵀ` means transpose: exchange the row and column axes. We need `Wᵀ [3,2]` because each input has three features, and each scoring rule expects three features.

```text
Y[0,0] = 2×1 + 1×2 + 0×(-1) + 0.5  =  4.5
Y[0,1] = 2×0 + 1×(-1) + 0×2 - 0.5 = -1.5
Y[1,0] = 0×1 + 1×2 + 3×(-1) + 0.5 = -0.5
Y[1,1] = 0×0 + 1×(-1) + 3×2 - 0.5 =  4.5

Y [2,2] = [ 4.5  -1.5]
          [-0.5   4.5]
```

The general rule is `[B,I] @ [I,O] -> [B,O]`. The inner dimensions must agree; the outer dimensions become the output shape. Here `I` means input features and `O` means output features. This is different from elementwise multiplication, which pairs corresponding elements after any permitted broadcasting. Basic array and matrix operations are developed in the authors' [linear algebra chapter of Dive into Deep Learning](https://d2l.ai/chapter_preliminaries/linear-algebra.html).

Library weight layout matters. PyTorch's `Linear` documents weights as `[out_features,in_features]` and applies an affine map to the last input dimension. Candle's pinned implementation is examined in [chapter 3](#candle). “Linear layer” usually includes a bias; in strict mathematics a nonzero bias makes the map *affine*. [PyTorch Linear contract](https://docs.pytorch.org/docs/2.9/generated/torch.nn.Linear.html).

For a sentence tensor `[B,T,D]`, applying the same linear layer to every token gives `[B,T,O]`. This changes each token's feature vector; it does not mix information between tokens. That distinction becomes central when we compare an MLP with attention.

<a id="dot-projection"></a>
### 1.3 A dot product is not automatically a normalized similarity or a projection

For two real vectors of the same length,

$$x\cdot w = \sum_i x_iw_i = \|x\|\|w\|\cos\theta.$$

`||x||` denotes Euclidean length, and `θ` is the angle between nonzero vectors. Multiplying matching coordinates and adding is the dot product. A useful intuition is a weighted score: how much does this input activate this scoring rule? Learning chooses the weights, and an individual rule need not acquire a simple human-readable meaning.

Three related operations answer different questions:

| Operation | Formula for nonzero `w` (and nonzero `x` where required) | Question |
|---|---|---|
| Dot product | `x·w` | What is the magnitude-sensitive weighted score? |
| Scalar projection onto the direction of `w` | `(x·w)/‖w‖` | How far does `x` extend along that unit direction? |
| Vector projection onto the line through `w` | `((x·w)/(w·w)) w` | What vector lies along that line? |
| Cosine similarity | `(x·w)/(‖x‖ ‖w‖)` | How aligned are the directions, ignoring length? |

For `x=[2,0]` and `w=[3,0]`, the dot product is 6, scalar projection is 2, vector projection is `[2,0]`, and cosine similarity is 1. Doubling `w` doubles the dot product while leaving the direction-based answers unchanged. This small example shows why calling every neural weighted sum “the projection” can mislead.

A zero dot product means orthogonality in the chosen coordinates. It does not prove that two words, images, or concepts are unrelated. A negative dot product does not establish a human semantic opposite. Those interpretations depend on representation and training.

The dot product is symmetric: `x·w = w·x`. Their *roles in a particular computation* can still differ. A layer may receive `x` as current data and retain `w` as a trainable parameter. During backpropagation, derivatives can flow into both: upstream layers produced `x`, and the optimizer may update `w`. An input to a hidden layer is not automatically outside the learning process.

<a id="gradient-step"></a>
### 1.4 A complete gradient step: learning is a numerical change

A loss is a number describing a chosen discrepancy. For one scalar prediction, use half the squared error:

$$\hat y=w\cdot x+b,\qquad L=\frac12(\hat y-y)^2.$$

`ŷ` is the prediction and `y` the target. Let `x=[2,1]`, `w=[0.1,-0.2]`, `b=0`, and `y=1`.

```text
prediction = 0.1×2 + (-0.2)×1 + 0 = 0
error      = prediction - target = -1
loss       = 0.5×(-1)² = 0.5
```

The derivative of the loss with respect to the prediction is the error. A unit change in `w[i]` changes the prediction by `x[i]`. Combining those effects gives:

$$\frac{\partial L}{\partial w_i}=(\hat y-y)x_i,\qquad
\frac{\partial L}{\partial b}=\hat y-y.$$

Our weight gradients are `[-2,-1]` and the bias gradient is `-1`. With learning rate `η=0.1`, ordinary gradient descent gives:

```text
w_new = [0.1,-0.2] - 0.1×[-2,-1] = [0.3,-0.1]
b_new = 0 - 0.1×(-1)             = 0.1
new prediction = 0.3×2 - 0.1×1 + 0.1 = 0.6
new loss = 0.5×(0.6-1)² = 0.08
```

We have actually changed parameters and recomputed the outcome. Merely evaluating `loss` would not be training. The gradient gives local sensitivity, and the chain rule connects sensitivities through compositions. The authors' [calculus chapter](https://d2l.ai/chapter_preliminaries/calculus.html) provides the mathematical background.

Under the ordinary Euclidean notion of distance, the gradient points toward the steepest first-order increase. Stepping against it is a local strategy; an excessively large step can increase loss. An update changes a vector's magnitude as well as its direction, so “rotating the feature detector” is only an analogy. Lower training loss also does not establish better performance on new images.

For a batch, explicitly define whether the loss sums or averages examples and output elements. That choice changes gradient scale. In a multi-layer network, compute gradients using the parameter values from the forward pass, then update the parameters. Chapter 4 works through this chain in detail.

<a id="broadcast-layout"></a>
### 1.5 Broadcasting, reshaping, and transposing solve different problems

Broadcasting lets an operation reuse values across compatible axes. In the usual trailing-axis convention, compare shapes from the right: dimensions must match or one must be 1. Candle exposes explicitly named broadcasting operations; do not infer that every operator automatically broadcasts.

To add one bias per channel to `[B,C,H,W]`, use a bias view shaped `[1,C,1,1]`. A plain `[C]` aligns with the last axis, `W`. It may error, or worse, run with the wrong meaning when `C == W`.

Reshaping changes the grouping of elements; it is not an arbitrary permutation. Transposing exchanges axes. Start with:

```text
A [2,3] = [1 2 3]
          [4 5 6]

reshape to [3,2], preserving row-major logical sequence:
          [1 2]
          [3 4]
          [5 6]

transpose to [3,2]:
          [1 4]
          [2 5]
          [3 6]
```

The two results have the same shape and different values at the same coordinates. This is why an attention head reshape cannot replace its required transpose.

Logical axes and physical storage layout are separate. A transpose may create a view with different strides; a later operation may require contiguous storage and make a copy. Do not promise that every reshape is free. Chapter 3 ties these operations to the pinned Candle source and Rust's ownership of tensor handles.

<a id="softmax"></a>
### 1.6 Stable softmax: turn competing scores into weights

Suppose a model produces three scores, one for each fruit class. These scores are called *logits*. For a nonempty vector of finite real logits `z`, define:

$$p_i=\frac{e^{z_i}}{\sum_j e^{z_j}}.$$

In exact arithmetic these numbers are positive and sum to 1. They are normalized model outputs; softmax alone does not establish that an advertised 90% probability will be correct 90% of the time. Calibration is an empirical property.

Softmax makes alternatives compete. For two classes, their probability ratio is `p_i/p_j = exp(z_i-z_j)`. Only differences matter. Adding the same constant to every score leaves the probabilities unchanged in exact arithmetic. This property lets us rewrite the calculation before floating-point overflow occurs:

$$m=\max_j z_j,\qquad
p_i=\frac{e^{z_i-m}}{\sum_j e^{z_j-m}}.$$

The common factor cancels algebraically. Each shifted exponent is at most 1, and at least one equals 1. For finite inputs, this prevents overflow in exponentiation and prevents every denominator term from underflowing to zero. Smaller terms can still round to zero; the output sum is subject to rounding. The numerical analysis is in [Blanchard, Higham, and Higham's softmax paper](https://eprints.maths.manchester.ac.uk/2765/).

Here is the full calculation for `[1,5,2]`:

```text
maximum:             5
shifted logits:      [-4, 0, -3]
exponentials:        [0.01831564, 1, 0.04978707]
sum:                 1.06810271
probabilities:       [0.01714783, 0.93623955, 0.04661262]
```

These probabilities correct the numerical values in the original note. `[1001,1005,1002]` has exactly the same differences and gives the same mathematical distribution. Computing `exp(1005)` first would already have lost the result to overflow; subtract the maximum *before* exponentiating.

Softmax is not argmax. Argmax returns a maximizing index, with a tie policy where necessary. A one-hot vector is a separate encoding of that selected index. In floating-point arithmetic a softmax output can contain exact zeroes through underflow, so “all alternatives always retain a nonzero probability” is not an implementation guarantee.

<a id="softmax-rust"></a>
### 1.7 A small, fallible Rust implementation with an explicit input contract

This standalone educational program accepts only nonempty, finite `f64` logits. If you are new to Rust, read the explanation now and return to the code after [chapter 2](#rust). `Result` makes invalid input visible to the caller. It does not replace NaN or infinity with an invented uniform prediction.

```rust
use std::{error::Error, fmt};

#[derive(Debug)]
enum SoftmaxError {
    Empty,
    NonFinite { index: usize },
}

impl fmt::Display for SoftmaxError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => write!(f, "softmax needs at least one logit"),
            Self::NonFinite { index } => {
                write!(f, "logit at index {index} is not finite")
            }
        }
    }
}

impl Error for SoftmaxError {}

fn softmax(logits: &[f64]) -> Result<Vec<f64>, SoftmaxError> {
    if logits.is_empty() {
        return Err(SoftmaxError::Empty);
    }
    for (index, value) in logits.iter().enumerate() {
        if !value.is_finite() {
            return Err(SoftmaxError::NonFinite { index });
        }
    }

    let maximum = logits.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let mut probabilities: Vec<f64> = logits
        .iter()
        .map(|&value| (value - maximum).exp())
        .collect();
    let denominator: f64 = probabilities.iter().sum();
    for value in &mut probabilities {
        *value /= denominator;
    }
    Ok(probabilities)
}

fn main() {
    let cases: &[&[f64]] = &[
        &[1.0, 5.0, 2.0],
        &[1001.0, 1005.0, 1002.0],
        &[5.0, 5.0, 5.0, 5.0],
        &[-f64::MAX, f64::MAX],
        &[],
        &[0.0, f64::NAN],
        &[f64::NEG_INFINITY, f64::NEG_INFINITY],
    ];
    for logits in cases {
        match softmax(logits) {
            Ok(probabilities) => println!("{logits:?} -> {probabilities:?}"),
            Err(error) => println!("{logits:?} -> error: {error}"),
        }
    }
}
```

The function borrows its input slice and owns its output vector. It validates before calculating the maximum. `copied()` copies small `f64` values out of references. `collect()` builds the output vector, and the final loop normalizes that vector in place. Rust's [`f64` documentation](https://doc.rust-lang.org/std/primitive.f64.html) defines the finite-value checks and floating-point operations used here; transcendental rounding can vary, so compare approximate numerical results rather than every printed digit across platforms.

For the first two cases, expect approximately `[0.01714783,0.93623955,0.04661262]`; equal logits give four values of `0.25`. With the extreme finite pair, subtraction can produce negative infinity for the smaller shifted value, whose exponential is zero; the result is `[0,1]`. The final three examples return explicit errors. A finite input contract does not imply every intermediate subtraction remains finite, but the negative overflow here has the correct negligible-contribution behavior.

The denominator cannot be zero for this accepted domain: the maximum's exponential contributes 1. A real in-memory slice on the checked 64-bit host cannot contain remotely enough terms, each at most 1, to overflow an `f64` sum. That argument is specific to this bounded program; it is not a general claim that arbitrary floating-point summation cannot overflow.

This implementation deliberately rejects negative infinity. Attention masks sometimes use negative infinity to mean “this key is forbidden.” A *masked* softmax needs a different documented contract: exclude forbidden keys, require at least one permitted finite score per query, and decide explicitly what an entirely masked row means. Applying ordinary shifted softmax to all `-∞` computes `-∞ - (-∞)`, which is NaN. Framework kernels may define additional behavior; do not assume that all libraries do the same thing.

<a id="cross-entropy"></a>
### 1.8 Log-softmax and cross-entropy: keep tiny probabilities in log space

Suppose the correct class has such a low probability that its floating-point softmax value rounds to zero. Computing `-log(0)` afterwards gives infinity. The original logits may still allow a finite loss if you avoid creating that tiny probability first.

For one correct class index `y`, with no class weighting or label smoothing:

$$L=-\log p_y
=\log\left(\sum_j e^{z_j}\right)-z_y
=\log\left(\sum_j e^{z_j-m}\right)-(z_y-m).$$

The final expression is a shifted form of log-sum-exp. For logits `[0,-1000]` and target class 1, it gives a loss approximately 1000 even if the second softmax probability underflows to zero. It does not make arbitrarily extreme losses representable; it avoids an unnecessary underflow route.

Likewise,

$$\log p_i=(z_i-m)-\log\left(\sum_j e^{z_j-m}\right).$$

Use a library's dedicated log-softmax or logits-based cross-entropy when its contract matches your task. PyTorch documents that its [log-softmax](https://docs.pytorch.org/docs/2.9/generated/torch.nn.functional.log_softmax.html) uses an alternative numerical formulation and that [CrossEntropyLoss](https://docs.pytorch.org/docs/2.9/generated/torch.nn.CrossEntropyLoss.html) takes unnormalized logits. Do not softmax first merely because the output “should look like probabilities.” Other loss APIs, such as negative log-likelihood, can require log probabilities instead; read the exact function contract.

For this single-target loss, differentiation gives `∂L/∂z_i = p_i - 1[i=y]`. The indicator is 1 for the target class and 0 otherwise. If the model assigns the correct class probability 0.2, its logit gradient is -0.8: gradient descent tends to increase that logit, with the parameter update still depending on the rest of the chain. The authors' [softmax regression chapter](https://d2l.ai/chapter_linear-classification/softmax-regression.html) connects these normalized outputs to classification learning.

**Checkpoint.** Before continuing, calculate the output of `[1,2] @ [[3],[4]]`, explain why a `[C]` bias is unsafe for NCHW channel addition, and state when a dot product equals the scalar projection. The answers are `[11]`, because trailing-axis alignment targets width, and when the direction vector has unit length. If these are clear, later tensor equations become bookkeeping you can reason through.

<a id="rust"></a>
## 2. Rust that makes model code understandable

Imagine the checkout camera has produced an image. A loader owns the pixel values, preprocessing reads or changes them, a model uses them, and a reporting step turns predictions into ordinary Rust values. Rust asks a practical question at each boundary: who owns this value, and what access does the next function need?

The examples in this chapter use Rust 1.98.1 and edition 2024. Ownership, borrowing, and trait semantics come from Rust; tensor dimensions, valid device operations, and checkpoint names remain Candle runtime concerns. Successfully compiling an attention layer does not prove that its dimensions describe the attention you intended.

<a id="rust-ownership"></a>
### 2.1 Ownership, borrowing, and lifetimes

An owned `Vec<f32>` manages a growable buffer of floating-point values. A borrowed `&[f32]` lets a function read a consecutive sequence without taking ownership. A `&mut [f32]` grants exclusive access through which that sequence can be changed. Prefer these slice arguments when an operation needs elements rather than the vector's ability to resize. [Vec](https://doc.rust-lang.org/std/vec/struct.Vec.html), [slices](https://doc.rust-lang.org/std/primitive.slice.html).

Passing a non-`Copy` value by value generally moves it. After `let saved = pixels;`, the original `pixels` binding cannot be used as though it still owns that vector. A move does not mean every pixel has been copied into a second allocation. It transfers responsibility for the value. Borrowing lets the original owner keep that responsibility while another part of the program temporarily uses the value. [Ownership](https://doc.rust-lang.org/book/ch04-01-what-is-ownership.html).

This standalone program makes those roles visible:

```rust
fn first_feature<'a>(values: &'a [f32]) -> Option<&'a f32> {
    values.first()
}

fn scale(values: &mut [f32], factor: f32) {
    for value in values {
        *value *= factor;
    }
}

fn main() {
    let mut features = vec![10.0_f32, 20.0, 30.0];
    scale(&mut features, 0.1);

    let first = first_feature(&features);
    println!("First normalized feature: {first:?}");
    // The shared borrow is no longer used after the line above.
    features.push(4.0);

    let saved_features = features;
    println!("Saved features: {saved_features:?}");
}
```

`Option<&f32>` means the function may return a reference to a value, or no value if the slice is empty. The apostrophe in `'a` names a lifetime relationship: a returned reference is valid only while the borrowed input can remain valid. It does not keep the vector alive, reserve memory, or ask the computer to extend an object's lifetime. Here Rust could infer the relationship, so `fn first_feature(values: &[f32]) -> Option<&f32>` is also sufficient. Explicit notation is shown to make the contract visible. [Lifetimes](https://doc.rust-lang.org/book/ch10-03-lifetime-syntax.html).

While a shared reference remains in use, Rust prevents conflicting mutation through ordinary references. With a mutable reference, access is exclusive for the duration of the borrow. The compiler can recognize that a borrow ends after its last use, before the closing brace. That is why this program can push another element after printing `first`. If `first` were used again after the push, the vector mutation would conflict with that live borrow. [Borrowing](https://doc.rust-lang.org/book/ch04-02-references-and-borrowing.html).

Read model signatures with the same questions. `forward(&self, input: &Tensor)` borrows the model and input. It does not transfer ownership of either. Returning an owned `Tensor` gives the caller a result handle. Whether its numerical storage is shared or newly allocated depends on the operation; the ampersand alone does not answer that question.

<a id="section-2-2-copying-a-value-cloning-a-handle-and-sharing-storage"></a>
### 2.2 Copying a value, cloning a handle, and sharing storage

`Copy` permits implicit duplication of suitable values, such as `usize` and `f32`. `Clone` is an explicit operation whose behavior is defined by the type. Cloning a vector clones its elements into a separate vector; cloning an `Arc<T>` creates another shared owner of the same allocation. Consequently, “add `.clone()`” is not a universal explanation of either correctness or cost. [Clone](https://doc.rust-lang.org/std/clone/trait.Clone.html).

An `Arc` uses reference counting to keep shared storage alive while owners remain. Shared ownership does not by itself permit arbitrary mutation: data that must be changed through shared handles needs an appropriate mechanism, such as a mutex or another form of interior mutability. `Arc` also does not make an otherwise unsafe-to-share inner type thread-safe. [Arc](https://doc.rust-lang.org/std/sync/struct.Arc.html).

This prepares us for Candle. The model, parameter map, and optimizer can each own handles that reach the same parameter storage. Rust's ownership rules apply to those handles. They do not require three independent numerical copies of every weight.

<a id="rust-errors"></a>
### 2.3 Option, Result, and the unit type

`Option<T>` answers “is there a value?” Its variants are `Some(value)` and `None`. An optional bias naturally fits this shape: the absence of a bias can be a deliberate model choice. `Result<T, E>` answers “did the operation succeed?” Its variants are `Ok(value)` and `Err(error)`. An unreadable checkpoint or incompatible weight shape is an error, not merely an absent optional value.

The unit type `()` has one value, also written `()`. A function returning `Result<(), E>` can report failure but has no additional success payload. Saving parameters might return `Ok(())`; loading a tensor returns `Ok(tensor)`. [Result](https://doc.rust-lang.org/std/result/enum.Result.html).

When an absent value should count as an error, choose that boundary explicitly. For example, `values.first().ok_or_else(|| "an image must contain pixels")` converts absence into an error. `as_ref()` is useful when inspecting an `Option<T>` without consuming its inner value. `map` transforms an available value, while `and_then` chains an operation that already returns an option. These methods express different contracts; they are not interchangeable abbreviations. [Option](https://doc.rust-lang.org/std/option/enum.Option.html).

<a id="section-2-4-what-the-question-mark-really-does"></a>
### 2.4 What the question mark really does

In an ordinary `Result`-returning function, `operation()?` produces the value from `Ok`, or returns early with an error converted into the enclosing function's error type. The conversion must exist. A useful local reading is:

```rust
// Explanatory fragment inside a Result-returning function:
let value = match operation() {
    Ok(value) => value,
    Err(error) => return Err(From::from(error)),
};
```

`?` does not catch a panic, retry, log, or roll back earlier effects. If five parameters were updated before the sixth update fails, propagating that error does not restore the first five. `Option` also supports `?` in a compatible return context, but it does not automatically invent an error when a function returns `Result`. [Try propagation](https://doc.rust-lang.org/reference/expressions/operator-expr.html#the-try-propagation-expression).

The following standalone parser demonstrates a useful error contract. Pixel features must be present, parseable, and finite; a successfully parsed `NaN` must not silently enter the numerical pipeline.

```rust
use std::{error::Error, fmt, num::ParseFloatError};

#[derive(Debug)]
enum FeatureError {
    Empty,
    Parse(ParseFloatError),
    NonFinite { index: usize },
}

impl fmt::Display for FeatureError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => write!(f, "at least one feature is required"),
            Self::Parse(error) => write!(f, "invalid feature number: {error}"),
            Self::NonFinite { index } => write!(f, "feature {index} is not finite"),
        }
    }
}

impl Error for FeatureError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Parse(error) => Some(error),
            _ => None,
        }
    }
}

impl From<ParseFloatError> for FeatureError {
    fn from(error: ParseFloatError) -> Self {
        Self::Parse(error)
    }
}

fn parse_features(text: &str) -> Result<Vec<f32>, FeatureError> {
    let values = text
        .split_whitespace()
        .map(str::parse::<f32>)
        .collect::<Result<Vec<_>, _>>()?;
    if values.is_empty() {
        return Err(FeatureError::Empty);
    }
    for (index, value) in values.iter().enumerate() {
        if !value.is_finite() {
            return Err(FeatureError::NonFinite { index });
        }
    }
    Ok(values)
}

fn main() {
    for input in ["0.2 0.8", "", "apple", "NaN"] {
        match parse_features(input) {
            Ok(values) => println!("Features: {values:?}"),
            Err(error) => println!("Cannot use {input:?}: {error}"),
        }
    }
}
```

The `From` implementation tells `?` how a parsing error becomes a `FeatureError`. It preserves the original error rather than reducing everything to a string. A caller can distinguish `Empty` from `NonFinite` by matching variants, without relying on the exact wording of a message. [From](https://doc.rust-lang.org/std/convert/trait.From.html), [Error](https://doc.rust-lang.org/std/error/trait.Error.html).

<a id="section-2-5-boxed-errors-and-application-context"></a>
### 2.5 Boxed errors and application context

`Result<(), Box<dyn std::error::Error>>` means “success without another value, or an owned error behind a common interface.” `dyn Error` is a trait object: the concrete error type can vary. The `Box` owns its value; trait-object metadata permits calls to the concrete implementation. This is not an error of literally any Rust type, nor a guarantee of a one-word pointer. [Trait objects](https://doc.rust-lang.org/reference/types/trait-object.html), [Box](https://doc.rust-lang.org/std/boxed/struct.Box.html).

Owned `Box<dyn Error>` in a return signature normally has the default `'static` object lifetime bound. For errors that need to move across threads, `Box<dyn Error + Send + Sync>` is a common stronger boundary. Suitable concrete error types have standard conversions into these boxes, allowing `?` to combine operations such as file reading and number parsing. Downcasting can recover a known concrete error; a typed enum is usually clearer when callers must make routine decisions by error kind.

Strings also have supported conversions into boxed error messages, so `return Err("image is empty".into());` can be appropriate in a function returning a compatible boxed-error result. That convenience does not make `String` an implementation of `Error`; it uses a conversion provided for the boxed target. For a structured error family, `thiserror` can generate repetitive `Display`, `Error`, and conversion implementations, but it is an optional dependency rather than a requirement for the enum pattern above. [Box conversions](https://doc.rust-lang.org/std/boxed/struct.Box.html), [thiserror](https://docs.rs/thiserror/latest/thiserror/).

For a public error enum expected to gain variants, `#[non_exhaustive]` requires downstream matches to allow future cases, normally with a wildcard arm. This is a deliberate API-evolution choice; adding the attribute is not a mandatory final stage for every application's private error type. [Non-exhaustive types](https://doc.rust-lang.org/reference/attributes/type_system.html#the-non_exhaustive-attribute).

For top-level applications, the existing project's `anyhow` dependency can add the operation's meaning while preserving its cause. This fragment requires `anyhow = "=1.0.100"`:

```rust
use anyhow::{Context, Result, ensure};

fn batch_size(text: &str) -> Result<usize> {
    let size = text
        .parse::<usize>()
        .with_context(|| format!("invalid batch size {text:?}"))?;
    ensure!(size > 0, "batch size must be greater than zero");
    Ok(size)
}
```

`with_context` builds context only when needed on failure. The resulting error keeps its underlying cause and supports downcasting. Avoid replacing an error with `anyhow!("failed: {error}")` solely to add a prefix: that formats the original into a message instead of retaining its structured cause in the same way. Backtrace capture and display depend on configuration; do not promise a backtrace on every run. [Context](https://docs.rs/anyhow/1.0.100/anyhow/trait.Context.html), [anyhow Error](https://docs.rs/anyhow/1.0.100/anyhow/struct.Error.html).

Choose the error boundary by what its consumer needs. Typed errors support programmatic recovery; erased errors with context are convenient for application reporting. Neither “every library must return an enum” nor “every error should become anyhow” is a language rule. Likewise, a smaller handle is not evidence that one error strategy is faster for an actual workload.

<a id="rust-generics"></a>
### 2.6 Turbofish, inference, and collect

In `text.parse::<f32>()`, `::<f32>` supplies the generic type argument explicitly. The informal name is *turbofish*. A type annotation can often give the same information: `let value: f32 = text.parse()?;`. This is compile-time type selection. It is not a numerical cast and not a runtime test of an object's concrete type. The extra `::` is required for this generic-argument syntax in expression paths. [Paths](https://doc.rust-lang.org/reference/paths.html#paths-in-expressions).

The parser above contains:

```rust
// Explanatory fragment from parse_features:
.map(str::parse::<f32>)
.collect::<Result<Vec<_>, _>>()?
```

The first line turns each text item into `Result<f32, ParseFloatError>`. The second asks the iterator to produce a single result containing a vector. `_` asks Rust to infer the element and error types where sufficient information exists. The `?` then converts a parsing error into the enclosing `FeatureError`.

`collect` is guided by its target type through `FromIterator`. A vector collects items in order. `Result<Vec<T>, E>` collects successful values until an error is encountered, then returns that error. `Vec<Result<T, E>>` instead keeps one result per item. The latter is useful for a report that must list every rejected record; the former is appropriate when one invalid feature invalidates the whole sample. [FromIterator](https://doc.rust-lang.org/std/iter/trait.FromIterator.html), [Iterator::collect](https://doc.rust-lang.org/std/iter/trait.Iterator.html#method.collect).

For an owned vector, `.iter()` borrows elements, `.iter_mut()` borrows them mutably, and `.into_iter()` consumes the vector to yield owned elements. Iterator adapters such as `map` are lazy; a consumer such as `collect` or a `for` loop drives them. When debugging, follow the item type at each stage. Most intimidating iterator expressions become straightforward once those types are written down.

<a id="section-2-7-traits-describe-behavior-not-tensor-correctness"></a>
### 2.7 Traits describe behavior, not tensor correctness

A trait defines behavior a type promises to provide. A generic function can require `T: SomeTrait`; a concrete struct implements the required methods. Importing the trait may be necessary to make its methods available through method-call syntax. Generics and trait objects offer different ways to call behavior: generic code can be specialized for concrete types, while a `dyn Trait` call uses runtime dispatch. Neither approach automatically checks a tensor's semantic axes. [Traits](https://doc.rust-lang.org/book/ch10-02-traits.html).

Candle's `Module`, `ModuleT`, and `Optimizer` are examples. The compiler checks that you call an available method with the declared Rust types. The called method still checks whether a weight matrix has the required shape, whether the dtype is supported, and whether the selected backend can execute the operation. Treat these as two cooperating layers of correctness.

<a id="candle"></a>
## 3. Candle: tensors, parameters, and checkpoints

This chapter refers to Candle commit `f5838914f788d3950d0a25042cffe199d9325a9e`, the revision in the project's lockfile. Its crate version fields say 0.9.1; that does not make an arbitrary 0.9.1 documentation page identical to this Git revision. The examples use CPU unless stated otherwise. Enabling the project's Metal feature does not prove that a particular machine has a usable Metal device, or that every CPU operation has the same accelerator support.

<a id="candle-tensors"></a>
### 3.1 A tensor carries more than values

At a model boundary, write down four things: shape, dtype, device, and axis meaning. For a batch of two checkout images, `[2, 3, 224, 224]` might mean batch, RGB channels, height, width. `[2, 224, 224, 3]` contains the same number of elements but assigns axes differently. A reshape cannot infer your intended meaning.

In this Candle API the Rust type is `Tensor`, not `Tensor<f32>`. Dtype is stored at runtime. `to_vec2::<f32>()` asks to extract a rank-two tensor whose dtype corresponds to `f32`. It does not first convert an arbitrary dtype into floats. Supported Rust scalar types are constrained by `WithDType`; this revision supports `i64`, for example, but not `i32`. [Pinned dtype implementation](https://github.com/huggingface/candle/blob/f5838914f788d3950d0a25042cffe199d9325a9e/candle-core/src/dtype.rs).

This complete CPU program distinguishes extraction from conversion:

```rust
use candle_core::{DType, Device, Result, Tensor};

fn main() -> Result<()> {
    let pixels = Tensor::new(&[[10u32, 20], [30, 40]], &Device::Cpu)?;
    let original = pixels.to_vec2::<u32>()?;
    let floats = pixels.to_dtype(DType::F32)?;
    let normalized = floats.affine(1.0 / 255.0, 0.0)?;
    println!("Pixels: {original:?}");
    println!("Normalized: {:?}", normalized.to_vec2::<f32>()?);
    Ok(())
}
```

`to_vec1`, `to_vec2`, and `to_vec3` correspond to ranks one, two, and three. The returned vectors own host-side values; accelerator extraction may require a device-to-host transfer. A `Vec<Vec<f32>>` produced from a rank-two tensor has equally long rows, but Rust's nested-vector type itself allows unequal row lengths. Extraction is useful at inspection or reporting boundaries; repeatedly extracting an entire activation inside a training loop creates unnecessary work. [Pinned tensor implementation](https://github.com/huggingface/candle/blob/f5838914f788d3950d0a25042cffe199d9325a9e/candle-core/src/tensor.rs).

<a id="section-3-2-shared-handles-copies-and-gradient-history"></a>
### 3.2 Shared handles, copies, and gradient history

These three operations answer different questions:

| Operation | Numerical storage | Gradient relationship |
|---|---|---|
| `tensor.clone()` | Shared handle | Same tensor and history |
| `tensor.copy()?` | Newly copied storage | Copy remains connected when history is tracked |
| `tensor.detach()` | Shared storage | Result cuts the backward connection |

Therefore, `clone()` is not a frozen numerical snapshot. A tensor that shares a variable's storage can observe changes made through that variable. `detach()` also does not promise an independent allocation. If both independence and a detached result are needed, those are two separate requirements to implement deliberately. [Pinned tensor copy and detach](https://github.com/huggingface/candle/blob/f5838914f788d3950d0a25042cffe199d9325a9e/candle-core/src/tensor.rs#L2158-L2193).

Candle records relevant operations as tensor computations execute. Backward differentiation follows that recorded history. This is a dynamic operation graph; Rust's compilation of your program does not turn it into a static, compile-time-validated neural network. [Pinned operation tracking](https://github.com/huggingface/candle/blob/f5838914f788d3950d0a25042cffe199d9325a9e/candle-core/src/op.rs), [backpropagation](https://github.com/huggingface/candle/blob/f5838914f788d3950d0a25042cffe199d9325a9e/candle-core/src/backprop.rs).

<a id="section-3-3-module-and-modulet"></a>
### 3.3 Module and ModuleT

`Module` provides `forward(&self, xs: &Tensor) -> Result<Tensor>`. `ModuleT` provides `forward_t(&self, xs: &Tensor, train: bool) -> Result<Tensor>`. The extra flag carries the training/evaluation choice to layers that need it. A Rust struct implementing either trait is still an ordinary struct; these traits do not inspect its fields and register parameters automatically. [Pinned module traits](https://github.com/huggingface/candle/blob/f5838914f788d3950d0a25042cffe199d9325a9e/candle-core/src/lib.rs).

Import `candle_core::Module` for ordinary trait-based forwarding and `ModuleT` for training-sensitive forwarding. Candle provides a blanket `ModuleT` implementation for types implementing `Module`; it ignores the training flag and calls `forward`. Consequently, adding `forward_t(..., false)` to a type that only implements `Module` does not invent evaluation behavior. A composite model containing dropout or BatchNorm must route the intended mode to those components.

A linear layer with two inputs and one output stores weights of shape `[1, 2]`. For input `X` of shape `[batch, 2]`, it computes `X Wᵀ + b`, yielding `[batch, 1]`. The optional bias has shape `[1]` and is broadcast across the batch. Candle's built-in linear layer stores `Tensor` and `Option<Tensor>` handles. [Pinned linear layer](https://github.com/huggingface/candle/blob/f5838914f788d3950d0a25042cffe199d9325a9e/candle-nn/src/linear.rs).

<a id="candle-parameters"></a>
### 3.4 Var, VarMap, and VarBuilder have different jobs

`candle_core::Var` wraps a tensor that can be updated. A `VarMap` associates names with variables. `VarBuilder` answers parameter requests using a chosen backend. In the familiar training setup, that backend is a VarMap, but it can instead be a checkpoint, a tensor map, or another backend. [Pinned variable](https://github.com/huggingface/candle/blob/f5838914f788d3950d0a25042cffe199d9325a9e/candle-core/src/variable.rs), [builder](https://github.com/huggingface/candle/blob/f5838914f788d3950d0a25042cffe199d9325a9e/candle-nn/src/var_builder.rs).

Think of a layer asking, “give me the tensor named `readout.weight`, with shape `[1, 2]`.” A training backend may create it. A checkpoint backend looks for stored data. The layer can use the returned tensor without knowing how the backend obtained it.

The main constructors express that choice directly: `from_varmap` retrieves or creates named variables, `from_tensors` retrieves entries from a supplied tensor map, `from_buffered_safetensors` uses owned checkpoint bytes, and `from_slice_safetensors` borrows checkpoint bytes. `zeros` provides zero tensors without becoming a trainable VarMap. This revision also offers `from_pth` and `from_npz` for its supported PyTorch and NumPy loading paths; a matching file extension alone does not establish architectural compatibility.

`get` and `get_with_hints` return `Result<Tensor>`, not `Result<Var>`. With a VarMap backend, the map retains a variable and the model receives a tensor handle to its storage. There is usually no reason to convert every returned tensor back into `Var` merely to store it in a layer. `Var::from_tensor` reuses an existing variable tensor, but converting an ordinary tensor creates a variable; that operation does not automatically register a name in an unrelated VarMap. [Pinned Var conversion](https://github.com/huggingface/candle/blob/f5838914f788d3950d0a25042cffe199d9325a9e/candle-core/src/variable.rs#L37-L45).

The key ownership pattern is:

```rust
// Construction fragment; build_model is your model constructor.
let parameters = candle_nn::VarMap::new();
let vb = candle_nn::VarBuilder::from_varmap(
    &parameters,
    candle_core::DType::F32,
    &candle_core::Device::Cpu,
);
let model = build_model(vb)?;
// Moving vb above did not move the separate parameters binding.
let variables = parameters.all_vars();
```

The original notes incorrectly identify this move as an obstacle to optimizer creation. The builder and VarMap are separate handles. Clone the builder when its handle must also remain available after a consuming call; do not clone it reflexively to “save the VarMap.” Builder cloning shares its backend and copies its path metadata. It avoids duplicating parameter arrays, but is not literally free.

<a id="section-3-5-names-are-part-of-model-behavior"></a>
### 3.5 Names are part of model behavior

`vb.pp("encoder").pp("layer0")` returns a builder with an extended prefix. A later `get(..., "weight")` uses the name `encoder.layer0.weight`. The original builder remains usable because `.pp` borrows it. Dynamic layers can use `vb.pp(format!("block_{index}"))`; these are ordinary runtime strings, not compiler-checked paths.

`contains_tensor("weight")` checks whether that exact prefixed tensor name is available from the backend. It does not mean “some descendant under this prefix exists,” validate a shape, or prove that a complete model can be loaded.

With one VarMap, requesting an existing name with the same shape reuses the stored parameter. Requesting that name with a different shape returns an error. Thus two same-shaped layers given the same prefix may accidentally share weights and still run. Separate prefixes express independence; deliberate repeated names express tying. The existing-entry path checks shape but does not recreate the variable according to a new initializer, dtype, or device request. Keep those expectations consistent when sharing a map. [Pinned VarMap retrieval](https://github.com/huggingface/candle/blob/f5838914f788d3950d0a25042cffe199d9325a9e/candle-nn/src/var_map.rs).

A useful inspection reports names, shapes, dtypes, and scalar counts. `all_vars().len()` counts variable tensors, while summing their element counts counts scalar entries. A matrix with shape `[100, 784]` is one tensor containing 78,400 numbers. Inspect the named map when checking prefixes, and sort names for readable output because hash-map iteration order is not a stable display order.

<a id="section-3-6-initializers-are-explicit-choices"></a>
### 3.6 Initializers are explicit choices

For this revision, these are valid options:

| Choice | Meaning when a fresh VarMap entry is created |
|---|---|
| `Init::Const(0.0)` or `init::ZERO` | Fill with zero |
| `Init::Randn { mean, stdev }` | Normal distribution |
| `Init::Uniform { lo, up }` | Uniform distribution |
| `init::DEFAULT_KAIMING_NORMAL` | Kaiming normal, fan-in, ReLU gain |
| `init::DEFAULT_KAIMING_UNIFORM` | Corresponding uniform choice |

There is no `init::DEFAULT` constant or `init::xavier_normal()` function in this revision. `Init::default()` is zero, so bare `vb.get(...)` is not a promise of random initialization. `get_with_hints` expresses the intended initializer, but a backend loading existing weights does not need to use that hint. [Pinned initialization](https://github.com/huggingface/candle/blob/f5838914f788d3950d0a25042cffe199d9325a9e/candle-nn/src/init.rs).

Kaiming initialization uses the number of incoming or outgoing connections and an activation-dependent gain to choose a scale. It is distinct from Xavier/Glorot initialization; neither name simply means “some random weights.” The built-in `linear` constructor here requests Kaiming normal weights and a bounded uniform bias, so an explanation claiming all default linear biases are zero would be inaccurate. The [MLP chapter](#mlp) explains why initialization matters to signal propagation and symmetry.

<a id="candle-training-example"></a>
### 3.7 A small, complete CPU learning program

Before training an image classifier, use a case whose target rule is obvious. Suppose normalized quantities of apples and oranges determine a price by `2 × apples + 3 × oranges + 1`. A single linear readout is sufficient. This example demonstrates parameter sharing and updates; it is not an image model or evidence of generalization.

In a separate Cargo package, the relevant dependency declarations are:

```toml
[package]
name = "candle-learning-example"
version = "0.1.0"
edition = "2024"

[dependencies]
candle-core = { git = "https://github.com/huggingface/candle", rev = "f5838914f788d3950d0a25042cffe199d9325a9e" }
candle-nn = { git = "https://github.com/huggingface/candle", rev = "f5838914f788d3950d0a25042cffe199d9325a9e" }
```

Put this complete program in that package's `src/main.rs`:

```rust
use candle_core::{DType, Device, Module, Result, Tensor};
use candle_nn::{Init, Linear, Optimizer, SGD, VarBuilder, VarMap, loss};

fn build_readout(vb: VarBuilder<'_>) -> Result<Linear> {
    let w = vb.get_with_hints((1, 2), "weight", Init::Const(0.0))?;
    let b = vb.get_with_hints(1, "bias", Init::Const(0.0))?;
    Ok(Linear::new(w, Some(b)))
}

fn main() -> Result<()> {
    let device = Device::Cpu;
    let parameters = VarMap::new();
    let vb = VarBuilder::from_varmap(&parameters, DType::F32, &device);
    let model = build_readout(vb.pp("readout"))?;

    let x = Tensor::new(&[[0f32, 0.], [1., 0.], [0., 1.], [1., 1.]], &device)?;
    let y = Tensor::new(&[[1f32], [3.], [4.], [6.]], &device)?;
    let initial = loss::mse(&model.forward(&x)?, &y)?.to_scalar::<f32>()?;

    let mut optimizer = SGD::new(parameters.all_vars(), 0.1)?;
    for _ in 0..400 {
        let prediction = model.forward(&x)?;
        optimizer.backward_step(&loss::mse(&prediction, &y)?)?;
    }

    let prediction = model.forward(&x)?;
    let final_loss = loss::mse(&prediction, &y)?.to_scalar::<f32>()?;
    println!("MSE: {initial:.6} -> {final_loss:.6}");
    println!("Predictions: {:?}", prediction.to_vec2::<f32>()?);
    println!("Weights: {:?}", model.weight().to_vec2::<f32>()?);
    if let Some(bias) = model.bias() {
        println!("Bias: {:?}", bias.to_vec1::<f32>()?);
    }
    Ok(())
}
```

All four examples stay fixed while the model learns. The initial mean squared error is `(1² + 3² + 4² + 6²)/4 = 15.5`. Successful optimization moves the weights toward `[2, 3]` and the bias toward `[1]`. Starting this single linear model at zero is appropriate; copying that initialization into every hidden unit of a multilayer network would create the symmetry problem discussed later.

Run the package with `cargo run`; retain its generated lockfile for subsequent reproducible dependency resolution. On the audited Rust 1.98.1 CPU run, this program printed weights `[[1.9999963, 2.9999967]]`, bias `[1.0000042]`, and predictions approximately `[1, 3, 4, 6]`. The loss printed as `0.000000` at six decimal places; that is rounding, not proof of exactly zero residual error. The run used the pinned Candle source and a separate scratch dependency lock, so it establishes this example's CPU behavior, not the original application's Metal behavior or identical results on every backend.

Build the model before collecting optimizer variables: constructors populate the map. `all_vars()` returns the variables present at that moment; it is not a subscription to future insertions. `backward_step` computes gradients from this loss and applies the optimizer to its stored variables. This pinned SGD implementation has no momentum. [Pinned optimizer](https://github.com/huggingface/candle/blob/f5838914f788d3950d0a25042cffe199d9325a9e/candle-nn/src/optim.rs).

<a id="section-3-8-freezing-is-a-choice-about-more-than-one-kind-of-state"></a>
### 3.8 Freezing is a choice about more than one kind of state

To update only a classifier head, pass only its variables to that optimizer. Separate prefixes help select them; separate maps can make that separation explicit. A map called `frozen_parameters` does not itself freeze anything or load a pretrained backbone. Those are operations your program must actually perform.

Distinguish three questions: which variables the optimizer updates, where gradients may flow, and whether layers run in training or evaluation mode. Excluding backbone variables from an optimizer answers the first question. Detaching backbone outputs can cut backward history when that is mathematically intended. BatchNorm statistics and dropout require the appropriate forward mode as well. The [normalization chapter](#normalization) develops this distinction.

If a layer has no bias, represent that as `None` and omit its creation. Zero-valued bias parameters are still parameters; “absent” and “present with value zero” are different states. Likewise, checkpoint bookkeeping may include non-optimized running statistics. Do not assume every named tensor in a model's saved state should receive gradient updates.

<a id="candle-checkpoints"></a>
### 3.9 Loading for inference and restoring trainable variables

An inference loader can read a safetensors file into owned bytes, then give a buffered builder to the same layer constructor. This complete function assumes the `build_readout` function above and adds `anyhow = "=1.0.100"`:

```rust
use anyhow::{Context, Result as AppResult};

fn load_readout(path: &std::path::Path) -> AppResult<candle_nn::Linear> {
    let device = candle_core::Device::Cpu;
    let bytes = std::fs::read(path)
        .with_context(|| format!("cannot read weights from {}", path.display()))?;
    let vb = candle_nn::VarBuilder::from_buffered_safetensors(
        bytes,
        candle_core::DType::F32,
        &device,
    )?;
    build_readout(vb.pp("readout")).context("readout weights do not match this model")
}
```

The constructor still requests `readout.weight` and `readout.bias`. Its initialization hints do not replace stored checkpoint values. Missing names or incorrect shapes must be treated as incompatibility, not as permission to silently initialize part of a supposedly restored model. Buffer loading uses owned bytes; slice loading instead ties the builder to the supplied slice's lifetime. [Pinned safetensors builder backends](https://github.com/huggingface/candle/blob/f5838914f788d3950d0a25042cffe199d9325a9e/candle-nn/src/var_builder.rs).

For trainable restoration, first create the named variables your model expects. `VarMap::load(&mut self, path)` updates variables already in the map. It does not discover and insert every file entry. Loading into an empty map therefore does not prepare a model for later construction. Extra checkpoint names are ignored, while a required missing variable produces an error. A later failure can occur after some earlier variables were updated, so loading is not transactional. [Pinned VarMap loading](https://github.com/huggingface/candle/blob/f5838914f788d3950d0a25042cffe199d9325a9e/candle-nn/src/var_map.rs#L42-L61).

For a restoration path using owned bytes throughout, build a fresh candidate model and map, obtain checkpoint tensors through a buffered builder, check all expected names and shapes, then apply those tensors to the candidate's existing variables with `set_one` or `set`. Adopt that candidate only after restoration succeeds. This separates an incomplete load from the model currently serving requests; it also gives you a place to validate label mappings and configuration.

Memory mapping is a separate loading option, not a prerequisite for learning Candle. `from_mmaped_safetensors` is unsafe because of the underlying file mapping contract. The underlying mapped files must remain unmodified and untruncated for the mappings' entire lifetime, including by another process; merely checking that a path exists or reading it through a read-only file handle is insufficient. Rust cannot enforce that external-file promise. This revision's public `VarMap::load` internally uses mmap as well, which is one reason the example above teaches buffered loading. [memmap2 safety](https://docs.rs/memmap2/latest/memmap2/struct.MmapOptions.html#method.map), [pinned safetensors implementation](https://github.com/huggingface/candle/blob/f5838914f788d3950d0a25042cffe199d9325a9e/candle-core/src/safetensors.rs).

<a id="section-3-10-a-weight-file-is-only-part-of-a-checkpoint"></a>
### 3.10 A weight file is only part of a checkpoint

`VarMap::save` serializes the map's named tensor values. Recreating the same computation also requires the right architecture, dimensions, axis conventions, preprocessing, vocabulary or class order, and any required non-parameter state. Resuming a training trajectory can additionally require optimizer moments, step counters, random-number state, scheduler state, and data-order position. An AdamW optimizer contains state beyond the model weights, so reconstructing a fresh optimizer from the same weight file does not resume its previous update history. [Pinned AdamW state](https://github.com/huggingface/candle/blob/f5838914f788d3950d0a25042cffe199d9325a9e/candle-nn/src/optim.rs).

For the checkout classifier, a file with correctly shaped outputs can still be wrong if class index zero used to mean “apple” and now means “orange.” The tensor library cannot infer that mismatch. Names and shapes are necessary checks; semantic metadata completes the contract.

<a id="section-3-11-translating-parameter-management-from-pytorch"></a>
### 3.11 Translating parameter management from PyTorch

PyTorch's `nn.Parameter` is a tensor subclass that is registered when assigned as a module attribute. Its `requires_grad` argument defaults to true but can be false. A plain tensor assigned to an attribute does not receive that same parameter-registration behavior. [PyTorch Parameter](https://docs.pytorch.org/docs/2.14/generated/torch.nn.parameter.Parameter.html).

`register_parameter(name, value)` is another explicit route; registering `None` records absence and excludes it from parameter iteration and `state_dict`. Nested PyTorch modules also form hierarchical names. Candle's prefixes therefore are not a unique ability to organize a hierarchy: the difference is how registration and discovery are expressed. [PyTorch Module](https://docs.pytorch.org/docs/2.14/generated/torch.nn.Module.html#torch.nn.Module.register_parameter).

| Requirement | PyTorch pattern | Candle pattern in this chapter |
|---|---|---|
| Declare updateable weights | `nn.Parameter` | `Var`, often created through a VarMap backend |
| Obtain weights in a layer | Module attributes | Tensor handles returned by a builder |
| Omit bias | `None` parameter | `Option<Tensor>::None` |
| Discover parameters | Module parameter traversal | Explicit VarMap or variable collection |
| Organize names | Nested module names | Explicit runtime prefixes |
| Restrict updates | Optimizer parameter selection | Optimizer variable selection |
| Restore weights | State-loading contract | Builder or existing-map restoration contract |

When translating a model, verify dimensions and state behavior operation by operation. Do not replace `nn.Parameter` mechanically with `vb.get` while leaving fields typed as `Var`. Do not assume Rust static typing validates names, guarantees numerical correctness, or establishes a performance advantage. A faithful translation preserves the mathematical operation, parameter identity, initialization, training mode, and saved-state meaning together.

<a id="mlp"></a>
## 4. An MLP: combining measurements and learning from mistakes

Imagine a supermarket conveyor with a scale and a camera. For each item, a preprocessing step produces measurements such as mass, average colour and roundness. An MLP takes a fixed-length list of these measurements and predicts something: perhaps a produce category or estimated ripeness. The network does not know what a kilogram or a colour means unless the data and training task make those measurements useful.

A *multi-layer perceptron* is a sequence of learned affine transformations with nonlinear functions between them. The intermediate numbers are hidden features. Their names are not usually assigned by a programmer: a hidden feature is simply a particular learned calculation. The same pattern later appears inside Transformer feed-forward networks.

<a id="mlp-shapes"></a>
### 4.1 From one neuron to a batch of predictions

One output feature computes a weighted sum plus a bias:

$$z_j=\sum_{i=1}^{I}x_iW_{ji}+b_j.$$

A positive weight makes that input increase this output, holding everything else fixed. A negative weight makes it decrease the output. The bias supplies an offset even when all inputs are zero. These are local arithmetic effects, not an explanation of the whole trained model.

Use the row-batch convention throughout implementation:

| Quantity | Shape | Meaning |
|---|---|---|
| `X` | `[B,I]` | B items, each with I input measurements |
| Stored `W` | `[O,I]` | One row of weights per output feature |
| `b` | `[O]` | One bias per output feature |
| `Z = X Wᵀ + b` | `[B,O]` | O calculated features per item |

This is the storage convention used by PyTorch `Linear` and the pinned Candle `Linear`. The batch size does not change the number of trainable parameters: a layer with bias has `I*O+O` parameters. Each row of a batch uses the same parameters. [PyTorch Linear](https://docs.pytorch.org/docs/2.9/generated/torch.nn.Linear.html), [pinned Candle Linear](https://github.com/huggingface/candle/blob/f5838914f788d3950d0a25042cffe199d9325a9e/candle-nn/src/linear.rs).

For example, an architecture `3 -> 8 -> 4` maps `[32,3]` to `[32,8]` and then `[32,4]`. Its parameter count is `(3*8+8)+(8*4+4)=68`. The hidden activation changes numbers but does not change this shape.

In textbook column notation the same operation is `z=Wx+b`, where `x[I,1]` and `W[O,I]`. Do not copy that formula into row-batch code without the transpose. A matrix dimension is part of the meaning of an equation.

<a id="mlp-nonlinearity"></a>
### 4.2 Why another linear layer is not enough

Suppose two layers have no activation between them. In column notation,

$$W_2(W_1x+b_1)+b_2=(W_2W_1)x+(W_2b_1+b_2).$$

They collapse into one affine transformation. Adding more such layers changes the parameterization but does not create a curved or piecewise decision boundary.

Consider two sensors, each reporting either 0 or 1. We want an alarm when exactly one sensor fires. The four desired outputs are `00 -> 0`, `01 -> 1`, `10 -> 1`, `11 -> 0`. One affine score followed by a single threshold cannot separate the two positive corners from the two negative corners.

A tiny ReLU network can calculate this rule exactly on those four inputs. Let `s=x₁+x₂`, `h₁=ReLU(s)`, `h₂=ReLU(s-1)`, and output `h₁-2h₂`:

| Input | s | h₁ | h₂ | Output |
|---|---:|---:|---:|---:|
| 0,0 | 0 | 0 | 0 | 0 |
| 0,1 | 1 | 1 | 0 | 1 |
| 1,0 | 1 | 1 | 0 | 1 |
| 1,1 | 2 | 2 | 1 | 0 |

The bend at `s=1` creates the useful change in behaviour. These are deliberately chosen weights; the calculation does not claim that a particular training run discovers them. Nor does the formula become a probability model for arbitrary real inputs.

ReLU is `max(0,z)`. Its derivative is 1 for positive z and 0 for negative z; at zero it needs a chosen convention because the ordinary derivative is undefined. A unit that remains negative for every training example receives no gradient through its ReLU output on those examples. [ReLU](https://docs.pytorch.org/docs/2.9/generated/torch.nn.ReLU.html), [PyTorch differentiation conventions](https://docs.pytorch.org/docs/2.9/notes/autograd.html).

Other common choices have different behaviour:

| Activation | Formula | Useful distinction |
|---|---|---|
| Sigmoid | `1/(1+exp(-z))` | Output in (0,1); derivatives become small at large magnitudes |
| Tanh | `(exp(z)-exp(-z))/(exp(z)+exp(-z))` | Output in (-1,1), centred around zero; also saturates |
| GELU | `z Φ(z)` | Smooth weighting by the standard-normal cumulative distribution; implementations may use an approximation |
| SiLU | `z sigmoid(z)` | Smooth activation; negative inputs are not all forced to zero |

The formulas describe ideal real arithmetic; use framework implementations rather than naive exponentials for numerical work. GELU and SiLU are different functions, so exchanging them changes a checkpoint's computation. [GELU](https://docs.pytorch.org/docs/2.9/generated/torch.nn.GELU.html), [SiLU](https://docs.pytorch.org/docs/2.9/generated/torch.nn.SiLU.html).

There is no promise that a larger network will learn a useful function from insufficient data. Representational capacity, successful optimization and generalization to new items are separate questions.

<a id="mlp-output-loss"></a>
### 4.3 Let the prediction task determine the output

For a real-valued target, a final affine layer can return an unconstrained prediction. Mean squared error penalizes squared differences. If you average over all `B*C` prediction elements, its gradient is `2*(prediction-target)/(B*C)`. If instead the definition uses half the sum, the factor changes. Write the reduction into the mathematics before implementing it.

For mutually exclusive produce categories, return one logit per category and use the [cross-entropy contract](#cross-entropy) from the foundations chapter. Apply softmax when probabilities are needed; do not apply it before a loss function that already accepts logits. For several independent yes/no labels, such as “damaged” and “unripe,” independent sigmoid outputs and binary cross-entropy express a different task. Those labels need not sum to one.

The last layer therefore does not automatically get the same activation as hidden layers. A ReLU on the output of a regression model forbids negative predictions; that might be intentional, but it is a modelling constraint. An `argmax` chooses a class and discards information needed to train with a differentiable classification loss.

<a id="mlp-backprop"></a>
### 4.4 One complete forward, backward and update calculation

Use two artificial normalized measurements `x=[1,2]`, two hidden units, ReLU and one output. The target is `y=0`. Stored hidden weights have one row per hidden unit:

$$W_1=\begin{bmatrix}1&0\\0&1\end{bmatrix},\quad b_1=[0,0],\quad w_2=[1,-1],\quad b_2=0.$$

For this example only, define the loss as

$$L=\tfrac12(\hat y-y)^2.$$

The forward pass is:

$$z_1=xW_1^T+b_1=[1,2],\quad h=\operatorname{ReLU}(z_1)=[1,2],$$
$$\hat y=h\cdot w_2+b_2=1-2=-1,\quad L=\tfrac12(-1)^2=0.5.$$

Backpropagation asks how a small change in each intermediate number would change L. Start at the loss and work backwards:

$$\frac{\partial L}{\partial\hat y}=\hat y-y=-1.$$

Changing the output bias changes the prediction one-for-one, so `dL/db₂=-1`. Changing output weight j changes the prediction by hidden feature `h_j`, so

$$\frac{\partial L}{\partial w_2}=(-1)[1,2]=[-1,-2].$$

The two hidden units influence the prediction through output weights `[1,-1]`. Therefore

$$\frac{\partial L}{\partial h}=(-1)[1,-1]=[-1,+1].$$

Both hidden preactivations are positive, so both ReLU derivatives are 1 and the same two numbers reach `z₁`. Each hidden unit's gradient multiplies the original input:

$$\frac{\partial L}{\partial W_1}=\begin{bmatrix}-1\\+1\end{bmatrix}[1,2]=\begin{bmatrix}-1&-2\\1&2\end{bmatrix},\quad \frac{\partial L}{\partial b_1}=[-1,+1].$$

Every derivative above used the parameters from the forward pass. Now perform one stochastic gradient descent (SGD) update with learning rate `η=0.1`:

$$W_1'=\begin{bmatrix}1.1&0.2\\-0.1&0.8\end{bmatrix},\quad b_1'=[0.1,-0.1],\quad w_2'=[1.1,-0.8],\quad b_2'=0.1.$$

Recomputing with the new parameters gives hidden values `[1.6,1.4]`, prediction `1.1*1.6-0.8*1.4+0.1=0.74`, and loss `0.5*0.74²=0.2738`.

The loss fell, although the prediction crossed the target. The calculation shows how this one step behaves. It does not prove that all learning rates reduce loss or that fitting this item helps unseen items.

**Compute every gradient before changing any parameter.** Updating `w₂` first and then using its new value to calculate the hidden gradients differentiates a mixture of two networks. The original handwritten backward loop made exactly that mistake. Reverse traversal determines the order of derivative calculation, not permission to mutate weights along the way.

For a general batch layer `Z=XWᵀ+b`, let `G` be the loss gradient with respect to Z. The matrix rules are:

$$\nabla_W L=G^TX,\quad \nabla_b L=\sum_{\text{batch}}G,\quad \nabla_X L=GW.$$

Check them by dimensions: `[O,B]@[B,I]=[O,I]`, matching W. If the upstream loss already divided by the batch size, do not divide again here. Through an elementwise activation, multiply its local derivative elementwise. When a tensor contributes through several branches, add the returning gradients. Automatic differentiation applies these chain-rule operations to the computation actually performed; it is not symbolic guessing from variable names.

<a id="mlp-training"></a>
### 4.5 Turning the arithmetic into a useful training process

An epoch is one pass through the selected training examples. A minibatch is the subset used for one update. An optimizer step updates parameters from gradients; several minibatches can intentionally accumulate gradients before one step, but accidental accumulation changes training.

An illustrative training procedure is:

```text
split examples into training, validation and final evaluation sets
fit preprocessing statistics using training examples only
initialize model parameters and optimizer state
for each epoch:
    enable training behaviour
    for each training minibatch:
        clear old gradient accumulation if the framework requires it
        compute predictions and the precisely defined loss
        calculate gradients using the forward-pass parameters
        optionally clip the completed gradient set
        update parameters
    enable evaluation behaviour
    measure validation results without parameter updates
    retain the best checkpoint according to the chosen validation criterion
restore the selected checkpoint and evaluate the reserved final set
```

For conveyor data, split by the source of correlation where necessary: adjacent photographs of the same apple should not create an easy training/validation leak. Fit mass and colour normalization on the training set, and carry those exact transformations into inference. A mean of zero and variance of one can help scale numeric features, but it is not an instruction to standardize class IDs or every kind of input.

Choose synthetic examples with a known relationship between features and targets when demonstrating learnability. Independently random labels can exercise code paths, but cannot demonstrate a useful predictive relationship on unseen independent samples. For five equally likely random classes, an input-independent guess has expected accuracy 1/5. Memorizing the training labels does not change that held-out expectation.

When reporting mean loss across unequal batches, weight by the number of terms contributing to each mean. Suppose one batch has two examples and mean loss 1, and the last batch has one example with loss 4. The example-weighted result is `(2*1+1*4)/3=2`, not `(1+4)/2=2.5`. Masked losses may need the number of valid elements instead of the number of examples.

Validation loss need not improve every epoch. Early stopping normally allows a chosen patience period and restores the selected checkpoint. Repeatedly tuning on the final evaluation set turns it into another validation set. A plotted curve is useful only when you know which data, reduction and model mode produced each point.

<a id="mlp-optimization"></a>
### 4.6 Initialization, optimizers and regularization

If all hidden units start with identical weights and receive identical updates, they remain indistinguishable. Random initialization breaks that symmetry. Scale also matters: large repeated transformations can explode activations, while small ones and saturated activations can weaken gradients.

Two common initialization targets, under their respective assumptions, are Glorot variance `2/(fan_in+fan_out)` and He variance `2/fan_in` for ReLU. For a normal distribution, the standard deviation is the square root of that variance. State which quantity an API accepts. These are principled starting points, not guarantees for arbitrary architectures. [Glorot and Bengio](https://proceedings.mlr.press/v9/glorot10a.html), [He et al.](https://arxiv.org/abs/1502.01852).

Plain SGD takes `θ <- θ-ηg`. Momentum keeps a running direction. Adam keeps exponential moving averages of the gradient and its square:

$$m_t=\beta_1m_{t-1}+(1-\beta_1)g_t,\qquad v_t=\beta_2v_{t-1}+(1-\beta_2)g_t^2,$$
$$\hat m_t=m_t/(1-\beta_1^t),\quad \hat v_t=v_t/(1-\beta_2^t),\quad \theta_{t+1}=\theta_t-\eta\frac{\hat m_t}{\sqrt{\hat v_t}+\epsilon}.$$

Operations on parameter arrays here are elementwise. Bias correction compensates for initializing the moving averages to zero. The optimizer therefore has state beyond the model weights; restoring weights alone does not resume an identical training trajectory. [Adam paper](https://arxiv.org/abs/1412.6980).

An L2 penalty `λΣw²` contributes gradient `2λw`. AdamW instead separates weight decay from Adam's adaptive gradient transformation. Adding an L2 penalty to Adam and selecting AdamW are not interchangeable operations. A schedule changes the learning rate over time; it does not change the gradient's definition. [Decoupled weight decay](https://arxiv.org/abs/1711.05101).

Dropout randomly zeroes selected activation elements during training. With drop probability p, the retained elements are scaled by `1/(1-p)` in the common inverted-dropout convention. For input value 6 and `p=0.25`, the output is 0 with probability .25 and 8 with probability .75; its expectation is still 6. Evaluation dropout returns the input unchanged. That expectation statement does not make the whole nonlinear network's training output equal its evaluation output. [PyTorch Dropout](https://docs.pytorch.org/docs/2.9/generated/torch.nn.Dropout.html).

The pinned Candle `Dropout::forward` receives an explicit `train` boolean; its training implementation accepts `0 <= p < 1`. A pure `Linear -> ReLU -> Linear` MLP has no dropout mode switch to apply. PyTorch's `eval()` changes mode-sensitive modules but does not itself disable gradient recording. The [normalization chapter](#normalization) explains BatchNorm's separate training and evaluation behaviour. [Pinned Candle dropout](https://github.com/huggingface/candle/blob/f5838914f788d3950d0a25042cffe199d9325a9e/candle-nn/src/ops.rs), [autograd modes](https://docs.pytorch.org/docs/2.9/notes/autograd.html).

Gradient clipping bounds a chosen gradient norm or element range before an update. It can limit a large step; it cannot fix an incorrect loss, invalid targets or NaNs already in the gradient. Monitor loss, prediction distributions, hidden activations and gradient magnitudes together. A zero gradient can mean a blocked ReLU path, missing graph connection or a locally flat objective. It does not automatically mean that learning has finished.

<a id="mlp-compression"></a>
### 4.7 What the advanced options actually change

The original notes mention compression and ensembles. Keep their purposes distinct; pruning changes a parameter structure, while distillation trains a student to use information from another model. [PyTorch pruning tutorial](https://docs.pytorch.org/tutorials/intermediate/pruning_tutorial.html), [distillation paper](https://arxiv.org/abs/1503.02531).

| Technique | What changes | Question to check |
|---|---|---|
| Pruning | Removes selected weights, channels or structures | Does the execution backend exploit the resulting structure? |
| Quantization | Stores or computes selected values in a lower-precision representation | Which operations remain floating point, and what accuracy changes? |
| Distillation | Trains a student using information from a teacher | Does the student improve on held-out task data? |
| Low-rank factorization | Replaces a weight matrix by smaller factors | Is the useful transformation well approximated at that rank? |
| Ensemble | Combines several models' outputs | Does the quality improvement justify extra storage and computation? |

For example, a dense `100×100` matrix has 10,000 weights. Factors `100×10` and `10×100` have 2,000 in total and a product of rank at most 10. That is a concrete representational restriction. It becomes a speed improvement only if the resulting operations execute efficiently for the actual workload.

Averaging logits and averaging probabilities are different ensemble rules because softmax is nonlinear. Choose the output contract deliberately. Do not assume that more members improve an ensemble if they repeat the same errors.

<a id="mlp-practice"></a>
### 4.8 Practice, with worked answers

1. **How many parameters does `5 -> 7 -> 2` have with bias in both layers?** `(5*7+7)+(7*2+2)=58`. The number is unchanged for batch sizes 1 and 100.
2. **Why did a hand-written backward routine change its hidden gradients after the output weights were updated?** It used a new output matrix to differentiate an old forward pass. Save the gradients first and update afterward.
3. **Does lowering training loss on random labels prove the classifier learned the feature-target relationship?** No. Independent labels contain no such relationship; separate held-out evaluation is essential.
4. **Can inference still be nondeterministic after dropout is disabled?** Yes. Other random operations, decoding choices or backend algorithms can still vary. Evaluation mode removes particular training behaviours; it is not a universal determinism guarantee.

Continue to [convolution](#conv) to learn how images add spatial structure to these feature transformations.

<a id="conv"></a>
## 5. Convolution: learning local patterns and mixing channels

A camera above a conveyor belt photographs apples as they pass. A classifier might answer, “This image contains a damaged apple.” A segmentation model must answer a more precise question: “Which pixels belong to the apple, and which belong to the damaged area?” Both need useful image features. A convolution builds those features by applying the same small calculation at many locations.

Throughout these chapters, an image tensor has shape `[N, C, H, W]`: number of images, number of channels, height, and width. RGB supplies three input channels, but a hidden layer might have 32 or 64 channels. Those later channels are learned numerical features; they are not guaranteed to correspond to named concepts such as “edge” or “bruise.”

<a id="conv-mechanics"></a>
### 5.1 One output value, worked from the beginning

Start with one input channel, a 3×3 patch, and a 2×2 kernel:

```text
Input                 Kernel
1  2  3               1   2
4  5  6               0  -1
7  8  9
```

Place the kernel at the top left. Multiply aligned entries and add them:

```text
1×1 + 2×2 + 4×0 + 5×(-1) = 0
```

Slide one position right, then repeat on the lower row:

```text
Top right:    2×1 + 3×2 + 5×0 + 6×(-1) = 2
Bottom left:  4×1 + 5×2 + 7×0 + 8×(-1) = 6
Bottom right: 5×1 + 6×2 + 8×0 + 9×(-1) = 8

Output
0  2
6  8
```

The kernel moved by one input position each time: **stride 1**. We added no border values: **padding 0**. The smaller output follows from there being only four valid placements.

Deep-learning libraries usually call this operation convolution, although the calculation shown is **cross-correlation**: we did not flip the kernel. Mathematical convolution reverses its spatial indices. With this asymmetric kernel, a flipped calculation would give different numbers. Learned kernels can accommodate either convention during training, but importing fixed weights requires matching the convention. PyTorch's Conv2d API explicitly specifies cross-correlation. [Conv2d definition](https://docs.pytorch.org/docs/2.9/generated/torch.nn.Conv2d.html)

For several input channels, one output filter contains a separate spatial kernel for each input channel. Calculate each channel's contribution, add all contributions, then add one optional bias for that output channel. A standard 3×3 filter receiving RGB therefore has `3×3×3=27` weights. Ten such filters produce ten output channels and use 270 weights, plus ten biases if enabled.

The word “2D” counts the spatial axes along which the filter moves. It does not mean the stored weights have only two dimensions. Standard Conv2d weights have four axes; an individual output filter has three. Depthwise Conv2d, discussed below, still has two spatial axes even though each output filter receives only one input channel.

Without a bias, the operation is linear in the input when the weights are fixed. Adding a bias makes it **affine**. An activation such as ReLU is a separate nonlinear operation. Keeping these steps separate helps explain both bottlenecks and [BatchNorm folding](#normalization-fusion).

<a id="conv-pointwise"></a>
### 5.2 Why a 1×1 convolution does useful work

A 1×1 kernel sees one spatial location, but that location can contain many channel values. With ordinary ungrouped convolution it can mix all those values.

Suppose one location has three sensor values `[R,G,B]=[8,2,1]`. We choose two example output filters:

```text
Output 1: 0.5R + 0.25G + 0.25B
Output 2: R − G
```

The resulting pair is `[4.75,6]`. At a second location with values `[1,6,3]`, the same filters produce `[2.75,−5]`. These deliberately chosen weights illustrate the operation; they are not trained color or defect detectors.

The same matrix acts on the channel vector at every location. Thus a stride-1, unpadded 1×1 convolution is equivalent to applying one shared linear layer, or affine layer with bias, separately to each location. For a tensor already arranged as `[N,H,W,C]`, the channel vector is the last axis. For NCHW, applying a Linear layer directly to the last axis would operate on width instead; change the layout or use Conv2d.

This layer can reduce channels, expand them, or retain their number. It cannot directly compare a location with its neighbors. If earlier layers already encoded neighboring evidence into the channel vector, however, the pointwise layer can combine that evidence. A 1×1 layer after a spatial convolution is therefore different from one applied directly to raw RGB.

Spatial size is preserved by the usual `stride=1, padding=0` configuration. A 1×1 convolution with stride 2 samples locations farther apart and can reduce size. “1×1 never changes height or width” is not a valid general rule.

Two consecutive pointwise affine mappings, with matching spatial sampling and no intervening nonlinear operation, can be combined:

```text
First:   u = A x + a
Second:  y = B u + b
Combined y = (B A)x + (B a + b)
```

ReLU between them changes this conclusion. The network now chooses different linear behavior for different inputs. Network in Network used small neural networks applied to local features, including pointwise transformations, to strengthen local modeling. A 1×1 layer is a building block of that architecture; it is not another name for the entire architecture. [Network in Network](https://arxiv.org/html/1312.4400v3)

<a id="conv-counts"></a>
### 5.3 Count weights, activations, and arithmetic separately

Let `Cin` and `Cout` be input and output channels, and `Kh`, `Kw` the kernel height and width. Let `g` be the number of channel groups. For valid positive channel/group counts:

```text
Weights = Cout × (Cin/g) × Kh × Kw
Biases  = Cout, if enabled; otherwise 0
Output values = N × Cout × Hout × Wout
Direct MAC count = output values × (Cin/g) × Kh × Kw
```

A **MAC** is a multiply-accumulate operation. Some FLOP reports count its multiply and add as two floating-point operations; others use different conventions. State the convention when comparing reports. The expression above counts the usual dense arithmetic, including nominal padded positions, and excludes bias, activation, and other layers.

For a 32→64 convolution with 3×3 kernels and one group, weights total `64×32×9=18,432`. On one 16×16 output grid, the direct count is `18,432×256=4,718,592` MACs. Bias adds 64 parameters, not another weight for every pixel. Weight sharing is precisely why image size changes activation storage and work without changing the layer's weight count.

A 1×1 layer with the same channels uses 2,048 weights. The ratio of weight counts is exactly nine. Including biases gives 18,496 versus 2,112 total parameters, whose ratio is less than nine. Claims such as “nine times fewer parameters” need that distinction.

Fewer MACs do not establish a speedup. Memory movement, intermediate tensors, shapes, dtype, kernel selection, hardware utilization, and launch overhead all matter. NVIDIA documents several convolution implementation strategies and how dimensions affect their performance. Treat arithmetic as a design estimate and latency as a measurement on a defined workload. [NVIDIA convolution guide](https://docs.nvidia.com/deeplearning/performance/dl-performance-convolutional/index.html)

<a id="conv-groups"></a>
### 5.4 Grouped convolution controls who can talk to whom

In a standard convolution, every output filter receives every input channel. Grouped convolution partitions the input and output channels into matching independent groups.

For `Cin=8`, `Cout=12`, and `g=2`:

```text
Input channels 0..3 ── six filters ── output channels 0..5
Input channels 4..7 ── six filters ── output channels 6..11
```

The twelve outputs are collected along the channel axis. They are not added together. Each output filter receives four input channels, so 3×3 weights have shape `[12,4,3,3]`, totaling 432. Ungrouped weights would have shape `[12,8,3,3]`, totaling 864. The bias count remains twelve.

Both channel counts must be divisible by the positive group count. `Cin=8,Cout=10,g=4` does not fit: output channels cannot be evenly assigned to four groups. Validate these conditions before constructing a layer; an integer division in a helper is not a substitute for checking the architecture.

Repeated grouped layers with the same grouping keep the groups isolated. A later ordinary 1×1 convolution can mix them. A channel shuffle changes which channels meet in the next grouped layer, which was a central part of ShuffleNet. Shuffling alone only rearranges values; the following computation performs the new interactions. [ShuffleNet](https://arxiv.org/html/1707.01083v2)

Historically, AlexNet divided parts of its computation between two GPUs. That is useful context for grouped connectivity, but modern grouping should be explained through its actual connectivity and cost rather than treated as an automatic accuracy improvement. [Original AlexNet paper](https://papers.nips.cc/paper/2012/file/c399862d3b9d6b76c8436e924a68c45b-Paper.pdf)

<a id="conv-depthwise"></a>
### 5.5 Depthwise and depthwise separable convolution

Set `g=Cin`. Every output filter now receives one input channel. This is **depthwise convolution**. With depth multiplier `m`, each input channel has `m` filters and `Cout=m×Cin` output channels. The common multiplier-one case keeps the channel count, but the general definition does not require that.

For three input channels and multiplier two:

```text
R ── two independent spatial filters ── R-feature 1, R-feature 2
G ── two independent spatial filters ── G-feature 1, G-feature 2
B ── two independent spatial filters ── B-feature 1, B-feature 2
```

No individual output mixes R with G or B. The six outputs can nevertheless contain different spatial responses because they use different filters.

A **depthwise separable convolution** follows depthwise spatial filtering with an ordinary 1×1 pointwise layer. The pointwise stage mixes the channel responses. MobileNet uses this pattern, with normalization and nonlinearities as part of its blocks. [MobileNet](https://arxiv.org/html/1704.04861v1)

For the earlier 32→64, 3×3 example, multiplier one gives:

| Stage | Weight count |
|---|---:|
| Depthwise 3×3, 32→32 | 32×9 = 288 |
| Pointwise 1×1, 32→64 | 32×64 = 2,048 |
| Combined | 2,336 |
| Standard 3×3, 32→64 | 18,432 |

With matching output sizes and a stride-1 pointwise stage, the direct MAC ratio is also `18,432/2,336 ≈ 7.89`. For one 16×16 output, the separable stages use `2,336×256=598,016` MACs. This comparison excludes bias, normalization, and activation.

The reduction comes with a structural restriction. Without intervening nonlinearities, an ordinary dense convolution can choose an independent spatial kernel for every input-output channel pair. A multiplier-one separable mapping uses one spatial kernel per input channel, then scales that response differently for the outputs. It cannot represent every dense kernel. Adding depth, width, or nonlinearities changes the model, but does not turn the original factorization into a lossless replacement.

<a id="conv-bottlenecks"></a>
### 5.6 How the pieces form useful architectures

A **bottleneck** makes an expensive spatial layer operate on fewer channels. Consider transforming 192 channels into 32 with a 5×5 kernel:

```text
Direct: 192 → 32 using 5×5
Weights: 192×32×25 = 153,600

Bottleneck: 192 → 16 using 1×1, then 16 → 32 using 5×5
Weights: 192×16 + 16×32×25 = 15,872
```

This saves about 89.7% of the weights for this specific comparison, with biases excluded. It does not prove equivalent information or accuracy: compressing 192 values to 16 restricts what reaches the next layer. Inception used pointwise reductions before some larger spatial operations to control cost. [Going Deeper with Convolutions](https://arxiv.org/html/1409.4842v1)

ResNet's original bottleneck pattern reduces channels with 1×1, processes them with 3×3, then expands with 1×1. A skip path adds the input to the residual branch when shapes match; otherwise a suitable projection can align them. The original ResNet-50/101/152 use bottleneck blocks, while the original ResNet-18/34 use basic blocks. [Deep Residual Learning](https://arxiv.org/html/1512.03385v1)

MobileNetV2 reverses the narrow-wide arrangement: expand channels, apply depthwise spatial filtering, then project to a narrow output. Its expanded stages use ReLU6, `min(max(x,0),6)`, while the final projection omits that clipping activation. Calling plain ReLU “ReLU6” in a comment does not implement the same function. A residual addition is used where stride and channel shapes permit it. [MobileNetV2](https://arxiv.org/html/1801.04381v4)

**Global average pooling** averages each channel over all spatial locations. It turns `[N,C,H,W]` into `[N,C]`, or `[N,C,1,1]` if axes are retained. A classifier can then map channels to class logits. Alternatively, a 1×1 layer can produce class score maps before the averaging. For an affine pointwise layer with no intervening nonlinearity, these orders commute: averaging `Ax+b` gives `A average(x)+b`.

Neither arrangement is equivalent to every fully connected layer over a flattened image. A flattened dense layer can assign distinct weights to the same feature at different positions. GAP discards that position information. This can suit image classification while being unsuitable as the final step of pixel-level segmentation.

<a id="conv-candle"></a>
### 5.7 Express the connectivity in Candle

This construction fragment uses the pinned Candle API. It fixes stride and dilation at one and uses an odd kernel so symmetric padding preserves size. It demonstrates layer construction rather than a complete training program.

```rust
use candle_core::Result;
use candle_nn::{conv2d, Conv2d, Conv2dConfig, VarBuilder};

fn make_conv(
    cin: usize,
    cout: usize,
    kernel: usize,
    groups: usize,
    vb: VarBuilder<'_>,
) -> Result<Conv2d> {
    if cin == 0 || cout == 0 || groups == 0 {
        candle_core::bail!("channel counts and groups must be positive");
    }
    if cin % groups != 0 || cout % groups != 0 {
        candle_core::bail!("groups must divide both channel counts");
    }
    if kernel == 0 || kernel % 2 == 0 {
        candle_core::bail!("this example expects a positive odd kernel");
    }
    let config = Conv2dConfig {
        padding: kernel / 2,
        groups,
        ..Default::default()
    };
    conv2d(cin, cout, kernel, config, vb)
}

fn examples(vb: VarBuilder<'_>) -> Result<Vec<Conv2d>> {
    Ok(vec![
        make_conv(8, 12, 3, 1, vb.pp("ordinary"))?,
        make_conv(8, 12, 1, 1, vb.pp("pointwise"))?,
        make_conv(8, 12, 3, 2, vb.pp("grouped"))?,
        make_conv(8, 16, 3, 8, vb.pp("depthwise_multiplier_two"))?,
    ])
}
```

The returned layers are separate examples, not a sequential network. Each expects eight input channels. The builder supplies or initializes the named parameters according to its backend. `conv2d` includes bias; `conv2d_no_bias` is available when the architecture omits it. Defaults also supply the pin's `cudnn_fwd_algo` field, which older four-field configuration literals omit. [Pinned Candle constructors](https://github.com/huggingface/candle/blob/f5838914f788d3950d0a25042cffe199d9325a9e/candle-nn/src/conv.rs)

<a id="conv-practice"></a>
### 5.8 Check your understanding

1. **A 1×1 layer maps 64 channels to 16 on a 20×20 grid. How many weights, and what spatial information does it add?** It has `64×16=1,024` weights. With stride 1 and padding 0, each output depends only on the channel vector at the matching location, so it adds no new direct neighbor connections.
2. **Can a depthwise layer map 4 channels to 12?** Yes: groups 4 and multiplier 3. Its 3×3 weights have shape `[12,1,3,3]`, so there are 108 weights. It still does not mix input channels.
3. **Why is a grouped layer's lower arithmetic count insufficient to promise lower latency?** Arithmetic counts do not tell us how well the selected backend kernel uses the hardware or how much memory it moves.
4. **Why does a residual addition need more than matching element counts?** Corresponding axes and spatial locations must have matching meaning. Two tensors containing the same number of elements can represent different channel or spatial arrangements.

<a id="spatial"></a>
## 6. Context, resolution, dilation, and transposed convolution

A tiny dark patch might be a bruise, a shadow, or part of the conveyor. Local appearance alone may be ambiguous. A wider view helps, but the model must still mark the correct boundary. This is the central tension in **dense prediction**: produce an answer at many spatial locations while using enough context to make each answer meaningful.

Spatial resolution describes the feature grid and its spacing relative to the input. More positions create an opportunity to retain detail; they do not guarantee sharp features or accurate predictions. A same-sized tensor can still discard information through filtering or nonlinearities.

<a id="spatial-shapes"></a>
### 6.1 Derive the output shape instead of guessing

Work along one axis first. Let:

- `L` be the input length;
- `k` be the number of kernel weights along that axis;
- `d` be dilation, the spacing between those weights;
- `s` be stride, the spacing between window starts;
- `p_left` and `p_right` be padding on the two sides.

The first and last of `k` weights are `d(k−1)` positions apart. Including both endpoints gives the **effective kernel width**:

```text
k_eff = d(k−1) + 1
```

A three-weight kernel with dilation 2 spans five positions:

```text
weight 0   weight 1   weight 2
    X    .     X    .     X
```

It has three learned weights, not five. In two dimensions a 3×3, dilation-2 kernel samples nine locations within a 5×5 bounding square, not all 25 locations.

The padded length is `L+p_left+p_right`. The first window starts at zero, and its last legal start is `padded_length−k_eff`. Counting starts at `0,s,2s,...` gives:

```text
Lout = floor((L + p_left + p_right − k_eff)/s) + 1
```

This assumes a valid configuration with positive sizes and at least one legal window. Apply the calculation independently to height and width. For symmetric padding `p`, substitute `p_left+p_right=2p`. [Convolution arithmetic](https://arxiv.org/html/1603.07285v2)

For `L=8,k=3,d=2,s=1`, the effective width is five. No padding produces four outputs. Two positions of padding on each side produce eight outputs. Dilation itself did not preserve the size; the padding did.

At stride 1, preserving length requires total padding `k_eff−1`. When that number is odd, the two sides cannot receive equal integer padding. For a width-four effective kernel, total padding must be three: perhaps one on the left and two on the right. That choice affects alignment. In an encoder-decoder model, matching tensor dimensions alone does not establish that their locations line up.

Some libraries use “same” to mean output length `ceil(L/s)`. The minimum total padding for that target is:

```text
target = ceil(L/s)
total_padding = max(0, (target−1)s + k_eff − L)
```

For `L=32,k_eff=9,s=2`, the target is 16 and total padding is seven. Padding 3 and 4 achieves it. Symmetric padding 4 also gives 16 outputs, although it uses one extra border position. A helper that always returns `(k_eff−s)/2` rounded down would give padding 3 on each side and only 15 outputs here.

PyTorch's `Conv2d(padding="same")` supports stride 1; do not assume the same keyword contract across frameworks. At the pinned Candle revision, Conv2dConfig uses one integer padding, stride, and dilation for both axes. Asymmetric padding requires an explicit operation outside that configuration. [PyTorch padding contract](https://docs.pytorch.org/docs/2.9/generated/torch.nn.Conv2d.html), [pinned Candle convolution source](https://github.com/huggingface/candle/blob/f5838914f788d3950d0a25042cffe199d9325a9e/candle-nn/src/conv.rs)

Stride normally reduces output resolution under conventional padding. It does not mathematically force a smaller output under arbitrary padding: adding enough border values can increase the number of window placements. Conversely, stride is not automatically information destruction in every possible architecture; sufficient output channels can encode multiple input positions. Ordinary pooling or compressed downsampling often loses information, but a universal percentage-loss formula based only on stride is unjustified.

<a id="spatial-receptive-fields"></a>
### 6.2 Track both the receptive field and the spacing

A unit's **theoretical receptive field** is the input region that can affect it through the computation graph. Its bounding width describes the outer span, potentially including holes. Its actual influence for a trained network depends on weights, activations, and input. A large theoretical span does not imply every pixel contributes strongly, or contributes at all. Research on the **effective receptive field** studies how influence is distributed within that theoretical region. [Effective receptive fields](https://arxiv.org/abs/1701.04128)

To calculate a chain of convolutional or pooling layers, track two quantities per axis:

- `R`: the bounding width in original input positions;
- `J`: the distance, measured in original input positions, between neighboring features on the current grid.

Initially `R=1,J=1`. For each new layer:

```text
R_new = R_old + (k_eff−1) J_old
J_new = J_old × s
```

The second formula explains the first. For a three-wide, dilation-1 kernel, if neighboring features are two original-input pixels apart, the neighboring kernel samples lie two original-input pixels to either side of the center. Use the **old** spacing when expanding the receptive field; the new layer's stride determines spacing between its outputs. Padding changes positions and boundary effects, while this interior bounding-width calculation remains the same. [Receptive-field derivations](https://distill.pub/2019/computing-receptive-fields/)

Here is a complete example:

| Layer | Kernel | Dilation | Stride | Calculation of R | New R | New J |
|---|---:|---:|---:|---|---:|---:|
| Input | — | — | — | One input position | 1 | 1 |
| 1 | 3 | 1 | 1 | 1 + 2×1 | 3 | 1 |
| 2 | 3 | 2 | 2 | 3 + 4×1 | 7 | 2 |
| 3 | 3 | 4 | 1 | 7 + 8×2 | 23 | 2 |
| 4 | 3 | 8 | 1 | 23 + 16×2 | 55 | 2 |

With square kernels and identical settings on both axes, the final bounding square is 55×55. Do not apply an extra factor of two to the recurrence because the operation has two spatial axes; calculate each axis separately.

For stride-1 3×3 layers with dilation sequence `1,2,4,8,16`, widths are `3,7,15,31,63`. This growth comes from doubling the dilation at each layer. Repeating dilation 2 instead gives linear growth `5,9,13,...`. Repeated stride can also produce rapid growth because it increases `J`.

The five-layer doubling example uses 45 spatial weights only in the special case of one input channel and one output channel per layer, excluding bias. Multi-channel layers must include their channel factors. The 63×63 span also does not make that stack equivalent to one freely parameterized 63×63 kernel. [Dilated context aggregation](https://arxiv.org/abs/1511.07122)

Output stride is the effective spacing `J` for an aligned simple chain. An output stride of eight means neighboring feature locations are eight input pixels apart; it does not mean each feature sees only an 8×8 patch. Its receptive field may be much larger. Replacing a later downsampling step with stride 1 and suitable dilation can maintain a denser output grid while retaining substantial context, at the cost of processing more output positions.

<a id="spatial-gridding"></a>
### 6.3 Gridding is about the connections of one output

Take a centered three-weight kernel with dilation 4. One output uses offsets `−4,0,4`. Stack another identical layer. Its input dependencies combine by **adding offsets**, giving `−8,−4,0,4,8` relative to the final location. They stay on a multiple-of-four grid.

This does not mean the whole layer ignores three quarters of the image. A neighboring output starts one position later and uses a different set of input positions. The output map is dense at stride 1; the issue is that some nearby positions do not communicate along that path. Early undilated layers, other branches, and later mixing can change the dependencies.

Varying dilation can improve coverage. For three-wide kernels:

```text
d=1: offsets −1,0,1
then d=2: each previous offset plus −2,0,2
result: every integer from −3 through 3

then d=5:
shift that interval left by 5:  −8..−2
leave it unchanged:            −3..3
shift it right by 5:            2..8
```

These intervals overlap, so the final output can depend on every position from −8 through 8. This is a 17-position span with dense theoretical coverage along that axis, away from boundaries.

“Use coprime dilation rates” is not a complete rule. Rates 2 and 3 have no common factor, but their two-layer offset set is `−5,−3,−2,−1,0,1,2,3,5`, missing `−4` and `4`. Kernel size, layer count, earlier stride, and actual connectivity matter. Hybrid Dilated Convolution studies such coverage problems; verify a proposed schedule by tracing its dependencies. [Understanding Convolution for Semantic Segmentation](https://arxiv.org/html/1702.08502v2)

Dilation preserves the number of stored kernel weights. At the same output size it preserves the nominal MAC count too. Its access pattern can nevertheless change runtime. A high-resolution dilated replacement for a downsampled stage may require far more work because it has more output positions. There is no universal dilation threshold at which every GPU becomes slow or every small object becomes invisible.

<a id="spatial-transpose"></a>
### 6.4 A transposed convolution reverses connections, not lost information

For fixed weights and no bias, convolution is a linear map. Flattening the input and output lets us write `y=A x`, where matrix `A` represents all the repeated local connections.

Use a one-dimensional input `[1,2,3]`, kernel `[1,2]`, stride 1, and no padding:

```text
A = [1  2  0]       x = [1]       y = [5]
    [0  1  2]           [2]           [8]
                        [3]
```

The first row computes `1+2×2=5`; the second computes `2+2×3=8`. The zero entries mean there is no connection at those positions.

Transpose the matrix by swapping its rows and columns:

```text
Aᵀ = [1  0]       Aᵀy = [ 5]
     [2  1]              [18]
     [0  2]              [16]
```

The middle value is `2×5+8=18` because two outputs contribute to that input location. This is the **scatter and add** view: distribute each input to the positions touched by its kernel, then sum overlaps.

The result `[5,18,16]` is plainly not the original `[1,2,3]`. A transposed convolution is the adjoint of the corresponding bias-free real-valued convolution, not its inverse. For compatible vectors `x` and `z`, an adjoint satisfies `dot(Ax,z)=dot(x,Aᵀz)`. Choosing `z=[4,−1]` verifies this here:

```text
dot([5,8], [4,−1]) = 12
Aᵀz = [4,7,−2]
dot([1,2,3], [4,7,−2]) = 12
```

That identity explains backpropagation: the gradient with respect to an input uses the transpose of the forward map. A convolution's bias does not change this input Jacobian. A separately learned transposed-convolution decoder can have its own weights and bias; it need not be tied to an encoder. [ConvTranspose2d](https://docs.pytorch.org/docs/2.9/generated/torch.nn.ConvTranspose2d.html)

In simple one-dimensional cases, repeated shifts produce Toeplitz structure. Two-dimensional cases have a related block structure; calling every flattened matrix an ordinary Toeplitz matrix is too imprecise. This is a mathematical representation, not a requirement to allocate the full matrix.

For a single-channel 224×224 image and a valid 3×3 convolution, a dense connection matrix would have `222²×224²`, about 2.47 billion, entries. At four bytes each that is about 9.9 GB, mostly zeros. It is wasteful, rather than universally impossible to store.

**im2col** instead gathers each input patch into a column and multiplies that matrix by flattened filters. It duplicates overlapping input values but avoids representing absent connections as a huge zero-filled matrix. Implicit-GEMM approaches generate the needed access pattern without materializing that entire patch matrix. Direct convolution, transform-based methods such as FFT or Winograd, and other implementations are also possible. Which is used depends on the library, shapes, dtype, and backend; there is no guarantee that a framework always selects the fastest algorithm. [NVIDIA convolution implementations](https://docs.nvidia.com/deeplearning/performance/dl-performance-convolutional/index.html)

<a id="spatial-upsampling"></a>
### 6.5 Stride, output padding, and checkerboards

For a transposed convolution using symmetric padding, output length is:

```text
Lout = (Lin−1)s − 2p + d(k−1) + output_padding + 1
```

Its `padding` has a different intuitive effect from forward padding: increasing it reduces the output size in this formula. Increasing stride spaces the scattered kernel placements farther apart. Stride 2 does not always exactly double the length; the other terms matter.

For example, `Lin=4,k=4,s=2,p=1,d=1,output_padding=0` gives eight outputs. Changing the kernel to three gives seven. This single change disproves the shortcut “transposed stride 2 always doubles.”

Why is `output_padding` needed? A forward convolution with `k=3,s=2,p=0,d=1` maps both length five and length six to length two. The output shape alone cannot tell us which original length was used. A matching transposed convolution maps length two to five when output_padding is zero, or six when it is one. It resolves size ambiguity; it does not restore values that were lost. Use the library's admissible range rather than treating output_padding as arbitrary padding. In the usual dilation-one case it is smaller than stride. [Transposed-convolution shape contract](https://docs.pytorch.org/docs/2.9/generated/torch.nn.ConvTranspose2d.html)

The “fractionally strided” interpretation can also help. For stride 2, insert a zero **between** adjacent input values. `[a,b]` becomes `[a,0,b]`, which has length three, not four. More generally that interior expansion has length `(Lin−1)s+1`; filtering and border handling then determine the final output.

For a three-weight kernel `[u,v,w]`, the uncropped full scatter result is:

```text
[a u, a v, a w + b u, b v, b w]
```

There are five positions. Appropriate cropping or size adjustment changes this result according to the configured operation. `output_padding` does not mean “append pixels whose values are zero.”

Uneven overlap can introduce **checkerboard artifacts**. With a three-weight kernel placed every two positions, some interior outputs receive two contributions and others one. In two dimensions the horizontal and vertical patterns combine. Choosing a kernel size divisible by stride can avoid this particular interior overlap-count imbalance, but learned weights and other architectural effects can still produce artifacts. Resize followed by ordinary convolution is another design: first enlarge using a defined interpolation rule, then learn spatial filtering. It has its own costs and does not recreate missing evidence automatically. [Checkerboard analysis](https://distill.pub/2016/deconv-checkerboard/)

At this Candle pin, ConvTranspose2d weights are `[Cin,Cout,Kh,Kw]`, and its configuration has no grouped-transpose option. PyTorch's grouped form uses `[Cin,Cout/g,Kh,Kw]`. A mathematical feature existing in another library does not establish its availability in the pinned Candle API. [Pinned ConvTranspose2d source](https://github.com/huggingface/candle/blob/f5838914f788d3950d0a25042cffe199d9325a9e/candle-nn/src/conv.rs)

<a id="spatial-architectures"></a>
### 6.6 Several ways to combine detail and context

**Atrous Spatial Pyramid Pooling, or ASPP**, applies parallel branches with different dilation rates to the same feature map. Concatenating their outputs preserves the different scales as separate channels; a later projection can mix them. DeepLab variants differ: a four-branch illustration is not automatically the complete DeepLabv3 module, which also uses image-level features. The exact rates depend on feature resolution and the chosen architecture. [Rethinking Atrous Convolution](https://arxiv.org/abs/1706.05587)

**U-Net** joins an encoder that builds coarser representations with a decoder that builds denser predictions. Skip connections bring earlier spatial features to later stages. In the original architecture, matching and cropping of feature maps matter, and decoder connections concatenate features rather than simply adding every pair. The combination helps localization, but cannot guarantee perfect boundaries. [U-Net](https://arxiv.org/html/1505.04597v1)

**Vision Transformers** ordinarily operate on patch tokens rather than making every raw pixel attend directly to every other pixel. Global attention can connect all tokens in a layer. Swin limits attention to windows and shifts their boundaries between blocks so information can move between windows over depth. Attention costs depend on token counts and window sizes; fixed-size windows give a different scaling pattern from global attention. Continue with [attention](#attention) for the calculations. [ViT](https://arxiv.org/html/2010.11929v2), [Swin](https://arxiv.org/html/2103.14030v2)

**Causal temporal convolutions** apply the same ideas along time. A sensor predictor at time `t` must not read future measurements. Left-sided context and the appropriate input/target shift enforce causality; dilation alone does not. Ten stride-1 layers with two-weight kernels and rates `1,2,...,512` span `1+(1+2+...+512)=1,024` time positions. With three-weight kernels the same rates span 2,047. WaveNet uses dilated causal modeling; its training can process known sequence positions in parallel, while autoregressive generation depends on earlier generated samples. Stride does not inherently violate causality if its time alignment is designed correctly. [WaveNet](https://arxiv.org/html/1609.03499v2)

Dense labels can be costly to obtain, so transfer learning, partially labeled data, and promptable models are relevant extensions. The original Segment Anything work assembled roughly one billion masks from 11 million images; “one billion images” would confuse masks with photographs. A promptable pretrained model still needs evaluation for the actual conveyor, lighting, and defect definition. [Segment Anything](https://arxiv.org/html/2304.02643v1)

These designs answer different constraints. Medical volumes add a third spatial axis and may have unequal physical spacing between voxels. A camera stream adds latency and temporal consistency requirements. High-resolution aerial imagery changes memory and tiling needs. There is no universal rule that segmentation must avoid stride, classification cannot benefit from dilation, or every decoder must use transposed convolution.

<a id="spatial-practice"></a>
### 6.7 Check your understanding

1. **A length-10 input uses k=3,d=3,s=1,p=0. What is the output length?** Effective width is seven, so there are `10−7+1=4` positions. Symmetric padding 3 would preserve ten positions.
2. **Two ordinary 3×3 layers have strides 2 then 1. What are R and J?** After layer one, `R=3,J=2`; after layer two, `R=3+2×2=7,J=2`. Counting only “two more pixels” for the second layer would miss the earlier stride.
3. **Does a 17×17 bounding field imply 289 contributing pixels?** No. Check dilation paths, boundaries, nonlinearities, and weights. A bounding box includes its holes.
4. **Can a transpose recover a unique original input solely by choosing output_padding?** No. It chooses a shape; it does not supply missing information or solve an inverse problem.
5. **A causal stack and a BatchNorm layer share future frames while training. Is the whole model causal?** Not necessarily. The normalization can introduce a dependency absent from the convolution. Follow every path that computes the prediction.

<a id="normalization"></a>
## 7. Normalization and folding BatchNorm into convolution

Training changes the values flowing between layers. Normalization makes selected aspects of those values easier to control, but the word covers several different operations. Begin by asking three questions: **Which values share a statistic? What parameters are learned? What state changes between training and inference?**

<a id="normalization-batch"></a>
### 7.1 BatchNorm's population is defined by axes

For image features shaped `[N,C,H,W]`, standard BatchNorm calculates one mean and variance for each channel using all `N×H×W` values in that channel. It does not normalize one image independently, and it does not combine different channels into one statistic.

Call the number of values in one such population `m`. During training:

```text
mean = sum(x_i) / m
variance = sum((x_i − mean)²) / m
normalized_i = (x_i − mean) / sqrt(variance + epsilon)
output_i = gamma × normalized_i + beta
```

Here `epsilon` is a small positive stabilizer, `gamma` is a learned scale, and `beta` is a learned offset. Each channel has its own gamma and beta, broadcast over that channel's images and spatial locations. Epsilon is inside the square root. [Original BatchNorm formulation](https://arxiv.org/html/1502.03167v3)

Work through one channel with values `[1,3,5,7]`. These might be two spatial positions from each of two images:

```text
Mean = (1+3+5+7)/4 = 4
Deviations = [−3,−1,1,3]
Squared deviations = [9,1,1,9]
Variance = 20/4 = 5
```

Choose epsilon 4 solely to make the arithmetic exact; this is not a practical recommendation. The denominator is `sqrt(5+4)=3`, so normalized values are `[-1,−1/3,1/3,1]`. With gamma 3 and beta 2, outputs are `[-1,1,3,5]`.

This example exposes two common mistakes. First, gamma 1 and beta 0 would return the normalized values, not the original input. Second, normalized variance is `5/9`, not one. In general it is `variance/(variance+epsilon)` before learned scaling. With positive epsilon and finite nonzero variance this is less than one. If every input value is identical, the centered values are zero and the outputs equal beta; epsilon avoids dividing by zero in the normalization itself.

Gamma and beta allow a learned affine adjustment after standardization. For fixed statistics, choosing the matching scale and shift could undo that standardization. A single learned pair cannot generally undo every possible batch's different statistics simultaneously.

The choice of population also affects gradients. Changing one input changes its channel's mean and variance, which can change other normalized values in the same population. BatchNorm's backward pass must differentiate through those statistics. Treating the training mean and variance as unrelated constants would produce a different derivative.

<a id="normalization-running"></a>
### 7.2 Training statistics and inference statistics are different objects

During ordinary BatchNorm training, normalization uses the current population's statistics. Running estimates are updated separately so that inference can use fixed reference values.

Let `a` denote the amount of the new observation included in an exponential moving average:

```text
running_new = (1−a) × running_old + a × observed_statistic
```

For `a=0.1`, a previous running mean of zero, and the observed mean four, the new running mean is 0.4. An EMA is an estimate shaped by its history, initialization, and momentum; it is not necessarily the exact mean of every example ever seen.

There is a variance convention to preserve when moving checkpoints between implementations. PyTorch uses the division-by-`m` variance for the training normalization, while the contribution to its running variance uses the corrected estimate dividing by `m−1`. For `[1,3,5,7]`, the latter is `20/3`. Starting from running variance one with `a=0.1` gives `0.9×1+0.1×20/3 ≈ 1.5667`. The pinned Candle training implementation makes the same `m/(m−1)` correction. [PyTorch BatchNorm2d](https://docs.pytorch.org/docs/2.9/generated/torch.nn.BatchNorm2d.html), [pinned Candle BatchNorm](https://github.com/huggingface/candle/blob/f5838914f788d3950d0a25042cffe199d9325a9e/candle-nn/src/batch_norm.rs)

PyTorch and this Candle implementation call `a` momentum. Keras's momentum instead weights the **old** running value: its update uses `old×momentum + observed×(1−momentum)`. Equal numerical momentum settings therefore do not mean equal updates. [Keras BatchNormalization](https://keras.io/api/layers/normalization_layers/batch_normalization/)

At inference with fixed statistics, the output of this BatchNorm layer no longer depends on the other images in the current inference batch. This is conditional on actually using stored statistics. In PyTorch, `track_running_stats=False` uses current batch statistics during evaluation too. Calling evaluation mode also does not itself disable autograd; mode and gradient recording are separate controls.

BatchNorm normally stores two learned vectors, gamma and beta, and two non-gradient running-statistic vectors. Some implementations also store a batch counter. Calling all of them “four trainable parameters” confuses trainable values with state.

There is no universal image-count threshold such as “BatchNorm fails below eight.” The arithmetic population for a convolutional channel is `m=N×H×W`, but nearby spatial values may be highly correlated. A large count is not the same as many independent observations. At `m=1`, the corrected running-variance estimate is undefined. A large epsilon does not solve that issue.

Distributed training adds another distinction: a large total batch across devices does not imply that each BatchNorm layer sees that entire population. Ordinary per-device normalization uses local values; synchronized variants communicate statistics across participating processes. Gradient accumulation alone does not retroactively make separate forward passes share one BatchNorm population. [Synchronized BatchNorm contract](https://docs.pytorch.org/docs/2.9/generated/torch.nn.SyncBatchNorm.html)

<a id="normalization-why"></a>
### 7.3 What BatchNorm helps with, and what the evidence does not prove

The original paper motivated BatchNorm using **internal covariate shift**: distributions inside the network change as preceding weights change. Later work challenged the idea that stabilizing those distributions is the central explanation. Santurkar and colleagues found training benefits even when deliberately disturbing normalized activation distributions, and studied how normalization can make gradients more predictive over a useful range of step sizes. [How Does Batch Normalization Help Optimization?](https://arxiv.org/html/1805.11604v5)

“Smoother optimization” means that a local gradient can remain a useful guide over a larger nearby region. It does not mean BatchNorm turns every loss surface into a gentle bowl, guarantees convergence, or prevents all vanishing and exploding gradients. Theoretical bounds have assumptions, and experimental results belong to the measured networks. Neither the original explanation nor a later alternative should become an unconditional slogan.

Batch-dependent statistics also add variability during training and can have a regularizing effect. That does not make Dropout universally unnecessary. Normalization placement likewise belongs to the architecture: Conv→BN→activation is common, while preactivation residual designs use a different order. Reordering layers changes the function; it is not a harmless formatting preference.

A preceding convolutional bias often becomes redundant in the common Conv→BN training arrangement. If `z=u+b`, subtracting its batch mean yields `u+b−(mean(u)+b)=u−mean(u)`. BN's beta supplies a learned offset afterward. This cancellation assumes the same constant bias within the normalized population and the usual mean-subtracting BN. It does not justify deleting bias from an arbitrary already trained inference graph without adjusting its stored state or folded parameters.

<a id="normalization-alternatives"></a>
### 7.4 Choose normalization by its reduction axes

For NCHW image features, the following table describes common forms. “Per image” means images do not share the statistic.

| Method | Values sharing mean/variance or scale | Main distinction |
|---|---|---|
| BatchNorm | N, H, W for each channel | Usually uses current statistics in training and stored statistics in evaluation |
| LayerNorm | The specified feature axes within each example or token | Its `normalized_shape` defines the axes; it is not automatically every axis except batch |
| GroupNorm | H, W and a selected group of channels within one image | No cross-image statistics |
| InstanceNorm | H, W within one image and one channel | Channel-wise per-image statistics; running-state options depend on implementation |
| Weight Standardization | Coefficients within one output filter | Transforms weights rather than activation populations |

In a Transformer, LayerNorm commonly normalizes a token's hidden-feature vector while keeping different tokens separate. In a CNN, a LayerNorm configured over C,H,W has a different population. GroupNorm separates channels into groups, but those groups concern statistics; they are distinct from grouped convolution's connectivity restriction. [Layer Normalization](https://arxiv.org/abs/1607.06450), [Group Normalization](https://arxiv.org/html/1803.08494v3)

Weight Standardization subtracts each output filter's mean and divides by a stabilized measure of its spread. It has been studied with normalization alternatives for small per-device batches. A reported improvement on a particular benchmark does not make it a universal replacement for BatchNorm. [Weight Standardization research](https://arxiv.org/html/1903.10520v2)

Batch dependence deserves special attention when examples are related. If a supposedly online video predictor normalizes together frames from both before and after its prediction time, its training computation can use future information. The problem is the actual reduction population, not video as a category. A study of surgical workflow models documents this concern. [BatchNorm in end-to-end video learning](https://arxiv.org/abs/2203.07976)

In federated learning, clients with different data distributions may develop incompatible local reference statistics. Local, shared, or hybrid handling must follow the chosen algorithm; “BatchNorm never works in federated learning” is too broad. [Federated BatchNorm research](https://arxiv.org/html/2405.14670v1)

<a id="normalization-fusion"></a>
### 7.5 Fold a fixed BatchNorm into the preceding convolution

At inference, a standard BatchNorm with fixed running statistics is an affine mapping for each channel. We can combine it with the affine convolution immediately before it.

For output channel `o`, let `W_o` be its convolution weights and `b_o` its bias. Let `mu_o`, `v_o`, `gamma_o`, and `beta_o` be the fixed BN values. Define:

```text
a_o = gamma_o / sqrt(v_o + epsilon)
```

Substitute the convolution output into BN:

```text
BN(Conv(x))_o
  = a_o × (W_o applied to x + b_o − mu_o) + beta_o
  = (a_o W_o) applied to x + beta_o + a_o(b_o − mu_o)
```

The fused convolution therefore has:

```text
W'_o = a_o W_o
b'_o = beta_o + a_o(b_o − mu_o)
```

If the convolution has no bias, use `b_o=0`. If BN has no learned affine transform, the mathematics uses gamma one and beta zero. The scale multiplies every coefficient belonging to an **output** channel. For Conv2d weights `[Cout,Cin/g,Kh,Kw]`, reshape it to `[Cout,1,1,1]` before broadcasting.

Here is a complete scalar example. Let the convolution be `z=2x+1`, and fixed BN values be mean 3, variance 3, epsilon 1, gamma 4, and beta −2. Again, the large epsilon is chosen only for transparent arithmetic.

```text
a = 4/sqrt(3+1) = 2
Fused weight = 2×2 = 4
Fused bias = −2 + 2×(1−3) = −6

For x=5:
Separate: z=11; BN=4×(11−3)/2−2=14
Fused:    4×5−6=14
```

Keeping epsilon was necessary to obtain the correct scale. Replacing `sqrt(v+epsilon)` with `sqrt(v)` would define a different function.

The derivation requires fixed statistics. Current-batch normalization depends on the input population, so a single fixed pair of fused weights and bias cannot replace its general training behavior. PyTorch's evaluation-fusion helper requires evaluation mode and populated running buffers. [Official fusion contract](https://docs.pytorch.org/docs/2.9/generated/torch.nn.utils.fuse_conv_bn_eval.html)

Graph structure matters too. If another branch consumes the convolution output **before** BN, replacing that shared output with the normalized result changes the branch. Several consumers of the already-normalized BN output do not cause that problem by themselves. Registered child-module order is not proof that layers execute consecutively.

Folding is exact algebra over real numbers. Floating-point computation can differ because multiplication, rounding, and accumulation occur in a different order. Low precision and extreme scales make numerical comparison especially relevant. Folding also does not guarantee a particular speedup or kernel count: a backend may already combine operations, or convolution may dominate the total time. Removing two BN layers around a ReLU does not by itself make the entire block one kernel.

<a id="normalization-candle"></a>
### 7.6 The pinned Candle API and its limits

The executable code at the pinned revision supports both training and evaluation through `ModuleT`, despite an outdated file-header comment describing inference only. The following is an illustrative fragment for already constructed compatible layers and an NCHW input:

```rust
use candle_core::{Module, ModuleT, Result, Tensor};
use candle_nn::{BatchNorm, Conv2d};

fn separate_eval(conv: &Conv2d, bn: &BatchNorm, x: &Tensor) -> Result<Tensor> {
    let z = conv.forward(x)?;
    bn.forward_t(&z, false)
}

fn folded_eval(conv: &Conv2d, bn: &BatchNorm, x: &Tensor) -> Result<Tensor> {
    let fused = conv.absorb_bn(bn)?;
    fused.forward(x)
}
```

For deployment, construct the fused layer once after selecting the fixed checkpoint, rather than folding it on every prediction as this tiny demonstration does. At this pin, `absorb_bn` requires BN's affine weight and bias tensors; it rejects affine-disabled BN even though the algebra supports that case. It also does not certify that the running statistics are trained or frozen. Those are caller responsibilities. [Pinned fusion implementation](https://github.com/huggingface/candle/blob/f5838914f788d3950d0a25042cffe199d9325a9e/candle-nn/src/conv.rs)

The pinned defaults are `eps=1e-5`, `momentum=0.1`, `affine=true`, and `remove_mean=true`. The ordinary equations in this chapter assume `remove_mean=true`. The pin exposes a nonstandard false setting; do not silently extend standard-BN training claims to it. The training implementation updates running state and uses the sample-count correction described above. Keep learned parameters, running state, and optimizer membership distinct when constructing or saving a model. [Pinned BatchNorm implementation](https://github.com/huggingface/candle/blob/f5838914f788d3950d0a25042cffe199d9325a9e/candle-nn/src/batch_norm.rs)

<a id="normalization-practice"></a>
### 7.7 Check your understanding

1. **For `[2,8,3,4]`, how many values contribute to each standard BatchNorm channel statistic?** `2×3×4=24`. There are eight separate means and variances, not one mean over all 192 entries.
2. **Does gamma one, beta zero make training BN an identity?** No. It leaves the standardized values unchanged after normalization, rather than undoing normalization.
3. **If convolution bias is absent, is the fused bias also zero?** Generally no. It is `beta−a×running_mean`.
4. **Why should a fusion comparison use evaluation behavior?** The fused layer uses fixed statistics. Comparing it against current-batch training normalization compares different functions.
5. **Can an EMA variance update with one value be repaired merely by increasing epsilon?** Not if the update computes a division-by-`m−1` correction. That separate problem is undefined at `m=1`.

<a id="se"></a>
## 8. Squeeze-and-Excitation: let the image influence its channel gates

A pointwise convolution uses the same learned weights for every image. A Squeeze-and-Excitation block, usually called **SE**, adds a small input-dependent decision: compute a global summary of the current image's features, then use it to scale each channel.

For the conveyor model, one image might contain strong reflections while another has a matte surface. A learned gating network can respond differently to their feature summaries. This does not mean a channel is guaranteed to be a “reflection detector,” or that gate values are a faithful explanation of a prediction. The concrete operation is a learned channel-wise multiplication. [Squeeze-and-Excitation Networks](https://arxiv.org/html/1709.01507v4)

<a id="se-mechanics"></a>
### 8.1 Squeeze, excite, scale

The input has shape `[N,C,H,W]`. The block has three steps:

```text
Input features                     [N,C,H,W]
  → average each channel over H,W  [N,C]
  → small MLP ending in sigmoid    [N,C]
  → reshape gates                 [N,C,1,1]
  → multiply original features    [N,C,H,W]
```

**Squeeze** is global average pooling, independently for each image and channel. It creates one average per channel. It loses information about where features occurred: two maps with identical averages produce the same descriptor even if their spatial arrangements differ.

**Excite** passes that descriptor through a small neural network. If `q` is its hidden width, the first layer maps `C→q`, ReLU adds a nonlinearity, and the second maps `q→C`. Sigmoid converts its outputs to gates. In vector notation, for one image:

```text
hidden = ReLU(W1 × descriptor + b1)
gates  = sigmoid(W2 × hidden + b2)
```

**Scale** multiplies every location in a channel by that image's gate for the channel. The output preserves the input shape, but that does not make it an identity.

These gates are independent sigmoid outputs, not softmax probabilities. They need not sum to one. Several channels can simultaneously receive large gates. With finite real-valued logits, sigmoid gates lie strictly between zero and one; floating-point implementations may round saturated values to an endpoint.

Canonical sigmoid SE therefore cannot increase an entry's absolute magnitude relative to the immediate input. It can emphasize one channel relative to another by attenuating it less. Variants that use `2×sigmoid` or `1+gate` can exceed one, but they define different gating rules.

SE is not normalization: it does not enforce zero means, unit variances, or a probability sum. It is also not token-to-token self-attention: there is no matrix of pairwise spatial affinities. Its channel gates depend on a global summary, which gives it image-wide conditioning without directly moving a feature value from one location to another.

<a id="se-example"></a>
### 8.2 A complete two-channel example

Take one image with two 2×2 feature maps:

```text
Channel 1             Channel 2
1  3                  0  2
1  3                  0  2
```

The channel averages are `[2,1]`. Use one hidden unit with weights `[1,−1]` and bias zero:

```text
hidden = ReLU(2−1) = 1
```

For the second layer choose weights `ln(3)` and `−ln(3)`, with zero biases. Multiplying by hidden value one gives those same two logits. Because `exp(ln(3))=3`:

```text
gate 1 = 1/(1+exp(−ln(3))) = 3/4
gate 2 = 1/(1+exp( ln(3))) = 1/4
```

The scaled maps are:

```text
Channel 1 × 0.75      Channel 2 × 0.25
0.75  2.25            0  0.5
0.75  2.25            0  0.5
```

No pixel moved. All positions within a channel used the same gate. The two gates happen to sum to one because we deliberately chose opposite logits; SE does not enforce that property. Choosing two logits equal to `ln(3)` would give gates `[0.75,0.75]`.

If another image had descriptor `[1,2]`, the hidden unit would output zero, both logits would be zero, and both gates would be 0.5. That is input dependence with fixed learned MLP weights. The numbers are hand-selected to reveal the mechanics, not evidence of a trained model's behavior.

<a id="se-cost"></a>
### 8.3 Count the cost and choose the hidden width explicitly

A reduction ratio `r` is a convenient way to choose `q`, often near `C/r`. The implementation must still decide rounding and minimum width. Divisibility of C by r is a convention used by some constructors, not a mathematical requirement of SE. Choosing a positive integer `q` directly makes the intended dimensions explicit; Torchvision's interface similarly takes a separate squeeze-channel count. [Torchvision SE implementation](https://docs.pytorch.org/vision/main/_modules/torchvision/ops/misc.html)

Two biased linear layers contain:

```text
First layer: Cq weights + q biases
Second layer: qC weights + C biases
Total: 2Cq + q + C
```

For C=64 and q=4, the total is 580. Omitting biases gives 512. The often-quoted `2C²/r` expression counts weights only and assumes `q=C/r` exactly.

The MLP needs roughly `2NCq` MACs, but it is not the entire block's work. Pooling reads all `NCHW` features, and applying the gates multiplies all of them. Memory traffic and backend details matter. “Small MLP” is a useful structural description, not proof of negligible latency on every device.

<a id="se-candle"></a>
### 8.4 A source-aligned Candle implementation

This illustrative module uses the repository's pinned Candle API. It takes explicit channel and hidden widths and returns errors for invalid dimensions. It is a module definition, not a complete executable or a claimed training result.

```rust
use candle_core::{Module, Result, Tensor};
use candle_nn::{linear, ops, Linear, VarBuilder};

#[derive(Debug)]
struct SeBlock {
    reduce: Linear,
    expand: Linear,
    channels: usize,
}

impl SeBlock {
    fn new(channels: usize, hidden: usize, vb: VarBuilder<'_>) -> Result<Self> {
        if channels == 0 || hidden == 0 {
            candle_core::bail!("SE channel and hidden widths must be positive");
        }
        Ok(Self {
            reduce: linear(channels, hidden, vb.pp("reduce"))?,
            expand: linear(hidden, channels, vb.pp("expand"))?,
            channels,
        })
    }
}

impl Module for SeBlock {
    fn forward(&self, xs: &Tensor) -> Result<Tensor> {
        let (n, c, h, w) = xs.dims4()?;
        if c != self.channels || n == 0 || h == 0 || w == 0 {
            candle_core::bail!("SE expects a nonempty NCHW tensor with matching channels");
        }
        let descriptor = xs.mean_keepdim((2, 3))?.reshape((n, c))?;
        let hidden = self.reduce.forward(&descriptor)?.relu()?;
        let logits = self.expand.forward(&hidden)?;
        let gates = ops::sigmoid(&logits)?.reshape((n, c, 1, 1))?;
        xs.broadcast_mul(&gates)
    }
}
```

The explicit `broadcast_mul` expresses that one gate applies over H and W. Ordinary tensor multiplication is not a promise to perform this broadcast. At this pin, sigmoid is `candle_nn::ops::sigmoid(&tensor)`, rather than a Tensor method. The builder determines whether parameters are loaded or trainable; the input and parameters must use compatible devices and dtypes. [Pinned tensor operations](https://github.com/huggingface/candle/blob/f5838914f788d3950d0a25042cffe199d9325a9e/candle-core/src/tensor.rs), [pinned sigmoid implementation](https://github.com/huggingface/candle/blob/f5838914f788d3950d0a25042cffe199d9325a9e/candle-nn/src/ops.rs)

<a id="se-variants"></a>
### 8.5 Placement, variants, and diagnostics

In a common residual integration, apply SE to the residual branch after its convolution stack and before adding the skip path:

```text
input ── convolution branch ── SE ── add ── output
  └───────────────────────────────┘
```

An actual block may also contain BN and activations. Their order belongs to that architecture. A Conv→BN→ReLU pattern with activation after the addition is not a preactivation ResNet merely because it has a skip connection.

CBAM combines channel attention with a spatial attention stage, allowing different locations to receive different weights. ECA instead uses a small one-dimensional convolution over the pooled channel descriptor, avoiding SE's hidden dimensionality reduction. These are different computations with their own trade-offs; reported results do not establish that one universally replaces another. [CBAM](https://arxiv.org/html/1807.06521v2), [ECA-Net](https://arxiv.org/html/1910.03151v4)

Alternative pooling, activations, and gate functions change the architecture. Compare them under the same data split and resource constraints rather than assuming extra complexity improves results. Sigmoid saturation is worth inspecting because its derivative `gate×(1−gate)` becomes small near either endpoint. Large or small gates alone do not prove a failure: inspect losses, gradients, and the actual effect on predictions before changing the model.

<a id="se-practice"></a>
### 8.6 Check your understanding

1. **Can two spatially different images receive identical SE gates?** Yes. If their channel averages match, this block's descriptor and gates match. Their scaled feature maps can still differ.
2. **Does SE normalize each channel to average zero?** No. If its input channel average is two and its gate is 0.75, its output average is 1.5.
3. **Can canonical SE directly distinguish the top-left and bottom-right values of one channel?** It uses the same gate at both locations. The convolutional features may encode location-dependent evidence, but this scaling step does not assign separate spatial gates.
4. **For C=32,q=8, how many parameters do two biased linear layers use?** `2×32×8+8+32=552`. The pooled descriptor and gates are activations, not extra learned parameters per image.
5. **Why is a 1×1 convolution after GAP equivalent to a Linear layer for excitation?** After pooling, there is one spatial position. Its channels form the same vector transformed by the corresponding affine matrix and bias.

<a id="attention"></a>
## 9. Attention: deciding which other items contribute

An MLP transforms one item's feature vector. Attention adds a way for an item to gather information from other items. In a sentence, the representation of a pronoun may benefit from nearby nouns. In a conveyor image, one feature location may benefit from another part of the same fruit. The mechanism computes numerical compatibility and then blends information according to those scores.

Start with a recipe analogy. Three advisers suggest different quantities for two ingredients. You assign weights that sum to one and take a weighted average of their suggestions. The weighted average is easy; the extra idea in learned attention is that the weights depend on the current input and are learned indirectly through the task loss.

<a id="attention-qkv"></a>
### 9.1 Queries, keys and values are learned calculations

For each item, a **query** supplies numbers used to score other items. A **key** supplies numbers against which queries are scored. A **value** supplies the information that will be blended. Think “matching inputs” and “content inputs,” but remember that all three are numerical projections of model representations.

In self-attention, the same input sequence X supplies all three projections:

$$Q=XP_Q,\qquad K=XP_K,\qquad V=XP_V.$$

Here P denotes a mathematical projection matrix with input features in its first dimension; a framework `Linear` usually stores its transpose. Biases are omitted for clarity. **Self-attention does not require projected Q=K=V**. Cross-attention takes Q from one sequence and K,V from another, such as a decoder querying encoded source text. [MultiheadAttention input and projection contract](https://docs.pytorch.org/docs/2.9/generated/torch.nn.MultiheadAttention.html).

A query is not a literal sentence such as “find the subject,” and a key is not a dictionary field saying “noun.” Those are possible intuitions, not fixed meanings guaranteed by the architecture. Specific learned heads sometimes show linguistic patterns, but heads can also attend broadly or redundantly. [Analysis of BERT attention](https://arxiv.org/abs/1906.04341).

Different query and key projections can make compatibility directional. `q_i·k_j` need not equal `q_j·k_i`. This is useful when the relation “asks for information from” is not symmetric. Separate projections also mean that the original embedding's cosine similarity is not generally the attention score.

<a id="attention-shapes"></a>
### 9.2 Read the equation through its shapes

For one head, omitting batch dimensions:

| Tensor | Shape |
|---|---|
| Q | `[Tq,dk]` |
| K | `[Tk,dk]` |
| V | `[Tk,dv]` |
| Scores `QKᵀ` | `[Tq,Tk]` |
| Weights A | `[Tq,Tk]` |
| Output `AV` | `[Tq,dv]` |

Queries and keys need the same width for their dot product. Values need the same item count as keys, because every score must identify a corresponding value. Their feature width `dv` can differ from `dk`. The number of output rows follows the number of queries.

The calculation is

$$S=QK^T/\sqrt{d_k}+M,\qquad A=\operatorname{softmax}_{\text{keys}}(S),\qquad O=AV.$$

M is an optional additive mask or bias. For a valid finite row without dropout, exact arithmetic makes A nonnegative with sum one, and that row of O is a convex combination of the value vectors. Floating-point calculations approximate those properties. An output projection and residual addition can take the full block's output outside that convex combination. [PyTorch scaled dot-product attention reference](https://docs.pytorch.org/docs/2.9/generated/torch.nn.functional.scaled_dot_product_attention.html).

<a id="attention-numerical"></a>
### 9.3 A complete numerical attention example

Use three artificial tokens with two features each, no mask, no bias and no dropout. These numbers are chosen for transparent arithmetic, not measured from a language model:

$$X=\begin{bmatrix}1&0\\0&1\\1&1\end{bmatrix},\quad P_Q=\begin{bmatrix}1&0\\0&1\end{bmatrix},\quad P_K=\begin{bmatrix}0&1\\1&0\end{bmatrix},\quad P_V=\begin{bmatrix}2&0\\0&1\end{bmatrix}.$$

Multiply X by each projection:

$$Q=\begin{bmatrix}1&0\\0&1\\1&1\end{bmatrix},\quad K=\begin{bmatrix}0&1\\1&0\\1&1\end{bmatrix},\quad V=\begin{bmatrix}2&0\\0&1\\2&1\end{bmatrix}.$$

Every query scores every key. For example, query 1 `[1,0]` dotted with keys `[0,1]`, `[1,0]`, `[1,1]` gives `[0,1,1]`. All rows together give

$$QK^T=\begin{bmatrix}0&1&1\\1&0&1\\1&1&2\end{bmatrix}.$$

Since `dk=2`, let `a=1/√2≈0.70710678`. Divide the entire matrix once:

$$S=\begin{bmatrix}0&a&a\\a&0&a\\a&a&2a\end{bmatrix}.$$

For row 1, exponentiating gives `[1,exp(a),exp(a)]`. Its denominator is `1+2exp(a)`. Thus the first weight is about .197776 and the other two are each .401112. For row 3, subtracting a from every score gives `[0,0,a]`, making its denominator `2+exp(a)`. The stable softmax produces the same distribution.

Rounded to six decimals:

$$A\approx\begin{bmatrix}0.197776&0.401112&0.401112\\0.401112&0.197776&0.401112\\0.248255&0.248255&0.503490\end{bmatrix}.$$

Finally blend the values. Output row 1 is

$$0.197776[2,0]+0.401112[0,1]+0.401112[2,1]\approx[1.197776,0.802224].$$

The complete output is

$$O\approx\begin{bmatrix}1.197776&0.802224\\1.604448&0.598888\\1.503490&0.751745\end{bmatrix}.$$

Nothing was retrieved as an exact record. Every allowed value contributed according to a soft weight. Row 1 gave equal weight to tokens 2 and 3 because their keys had equal compatibility with query 1; their values remain different.

This standalone Rust program calculates the same example using arrays. It is an educational CPU calculation without Candle or automatic differentiation:

```rust
fn project(x: [f64; 2], p: [[f64; 2]; 2]) -> [f64; 2] {
    [x[0] * p[0][0] + x[1] * p[1][0],
     x[0] * p[0][1] + x[1] * p[1][1]]
}

fn main() {
    let x = [[1.0, 0.0], [0.0, 1.0], [1.0, 1.0]];
    let pq = [[1.0, 0.0], [0.0, 1.0]];
    let pk = [[0.0, 1.0], [1.0, 0.0]];
    let pv = [[2.0, 0.0], [0.0, 1.0]];
    let q = x.map(|row| project(row, pq));
    let k = x.map(|row| project(row, pk));
    let v = x.map(|row| project(row, pv));

    for query in q {
        let scores = k.map(|key| {
            (query[0] * key[0] + query[1] * key[1]) / 2.0_f64.sqrt()
        });
        let max = scores.into_iter().fold(f64::NEG_INFINITY, f64::max);
        let exp = scores.map(|s| (s - max).exp());
        let sum: f64 = exp.iter().sum();
        let weights = exp.map(|e| e / sum);
        let mut out = [0.0; 2];
        for j in 0..3 {
            for c in 0..2 {
                out[c] += weights[j] * v[j][c];
            }
        }
        println!("weights={weights:.6?}, output={out:.6?}");
    }
}
```

This exact program was compiled and run with Rust 1.98.1, edition 2024; its printed rows match the six-decimal tables above. The fixed input here is finite and nonempty. The general-purpose [softmax example](#softmax) explains why arbitrary inputs need an explicit validity contract.

<a id="attention-scaling"></a>
### 9.4 Why divide by the square root of the key width?

A dot product adds `dk` products. Suppose, as an initialization model, the query and key coordinates are independent, each has mean zero and variance one, and different coordinate products are independent. Each product then has variance one, so the sum has variance `dk` and standard deviation `sqrt(dk)`. Dividing by `sqrt(dk)` returns that modelled variance to one. These are assumptions for the scaling argument, not guaranteed statistics of trained activations. [Scaling footnote in the original paper](https://arxiv.org/html/1706.03762v7).

Softmax reacts to score differences. A row `[0,1]` is less concentrated than `[0,100]`; the latter is almost a hard choice. Its derivatives `a_i(δ_ij-a_j)` can become tiny. Scaling helps control one source of excessive concentration, but it cannot guarantee healthy gradients throughout a network.

Divide by the per-head **query/key width**, not the sequence length or, when widths differ, the value width. You may scale Q before the matrix multiplication or scale scores afterward; real arithmetic gives the same result, but finite-precision rounding and overflow can differ.

<a id="attention-masks"></a>
### 9.5 Masks define which information is available

In a score tensor `[B,H,Tq,Tk]`, normalize the last axis. One row describes a particular query's distribution over keys. Normalizing across queries answers a different question and creates the wrong mechanism, even if the code returns a tensor of the expected shape.

A padding mask blocks keys that represent padding. A causal mask blocks future keys. A custom mask can restrict other relationships. For three causal positions, an additive mask is

$$M=\begin{bmatrix}0&-\infty&-\infty\\0&0&-\infty\\0&0&0\end{bmatrix}.$$

With the earlier scores, row 1 becomes `[0,-∞,-∞]` and has weights `[1,0,0]`. Row 2 becomes `[a,0,-∞]`, with weights approximately `[.669762,.330238,0]`. Its output becomes `[1.339523,.330238]`. Row 3 is unchanged. The diagonal is allowed: the representation at a position may use its current input token while predicting the next token.

Do not say that `softmax(-∞)` by itself is zero. Softmax is defined over a whole row. A masked element gets zero weight when the row also contains at least one valid finite score. If every entry is `-∞`, the denominator is zero in the unshifted formula and subtracting the maximum produces `-∞-(-∞)`, which is NaN. Reject that situation or define a deliberate output policy before applying ordinary softmax. A very large finite negative constant is not the same contract: masking every position with that constant can yield a uniform distribution over forbidden values.

Mask polarity is an API decision. In this handbook's Candle fragment, U8 value 1 means **blocked**. PyTorch's functional scaled-dot-product attention boolean mask uses `True` for **allowed**. Read the specific API rather than transferring a convention from another function. [PyTorch SDPA mask semantics](https://docs.pytorch.org/docs/2.9/generated/torch.nn.functional.scaled_dot_product_attention.html).

A `[B,Tk]` mask normally needs explicit singleton axes to become `[B,1,1,Tk]` before broadcasting over heads and queries. Blocking padded keys does not remove padded query rows. Those outputs may need to be excluded from pooling or the loss separately.

<a id="attention-gradients"></a>
### 9.6 The three branches learn together

Let the output gradient be G and `O=AV`. Then

$$\nabla_V L=A^TG,\qquad\nabla_A L=GV^T.$$

For one softmax row with upstream gradient g, the score gradient is `a_j*(g_j-Σ_k a_k g_k)`. That subtraction shows how entries in a row influence one another. With `S=QKᵀ/√dk`, the score gradient then flows into both Q and K:

$$\nabla_Q L=(\nabla_S L)K/\sqrt{d_k},\qquad\nabla_K L=(\nabla_S L)^TQ/\sqrt{d_k}.$$

Consequently Q, K and V have distinct roles in the forward calculation but are optimized through a shared objective. Their gradients depend on one another. At X, the contributions through the three projection branches add. Separate projections do not mean independent learning without interference.

<a id="attention-practice"></a>
### 9.7 Practice, with worked answers

1. **Q is `[5,3]`, K is `[7,3]`, and V is `[7,4]`. What is the output?** Scores are `[5,7]`, softmax runs over 7 keys, and output is `[5,4]`. This is a valid cross-attention shape.
2. **All projected Q values are zero and every key is allowed. What happens?** All dot-product scores are equal, so each query averages the values uniformly. Learned biases or extra score terms could change that premise.
3. **Does a larger value vector automatically receive more attention weight?** No. In the basic formula the current weights depend on Q and K, not V; V determines what those weights blend.
4. **A fully padded item produces NaNs. Should they simply be zeroed at the end?** First fix the fully blocked softmax rows. Invalid operations may also corrupt gradients before the output replacement.

<a id="transformer"></a>
## 10. Multiple heads and the Transformer block

<a id="transformer-heads"></a>
### 10.1 Several blends before combining the results

One attention head forms one distribution over keys per query. Multiple heads permit several distributions, each from learned projections. Think of asking several advisers to combine evidence in different ways, then learning how to combine their reports. The architecture provides the opportunity for different patterns; it does not assign one grammatical or visual role to each head.

For head r,

$$O_r=\operatorname{Attention}(XP_{Q,r},XP_{K,r},XP_{V,r}),$$
$$Y=\operatorname{Concat}(O_1,\ldots,O_H)P_O.$$

The usual equal-width choice is `dk=dv=D/H`, where D is the model width. But that is a configuration, not a mathematical requirement of all attention mechanisms. The general output projection accepts `H*dv` features. For a simple equal-width implementation, require `D>0`, `H>0`, and `D%H==0` before dividing. A divisibility check alone can panic if H is zero.

Each head's projection can read every feature of the original X. Splitting the **projected** width into heads does not mean that head 1 sees only the first portion of the original embedding. The per-head projections can be packed into one larger matrix multiplication without changing that mathematical fact.

Research has found both specialization and redundancy in trained heads. More heads can therefore change what is learnable or useful, but “more heads means better generalization” is not a theorem. [Empirical BERT head analysis](https://arxiv.org/abs/1906.04341).

These historical configurations retain the original notes' comparison with corrected dimensions. They are examples, not recommended defaults:

| Model variant | D | Heads | Head width | Blocks |
|---|---:|---:|---:|---:|
| Original Transformer base | 512 | 8 | 64 | 6 encoder + 6 decoder |
| BERT-base | 768 | 12 | 64 | 12 |
| BERT-large | 1024 | 16 | 64 | 24 |
| GPT-2 small | 768 | 12 | 64 | 12 |
| GPT-2 large | 1280 | 20 | 64 | 36 |
| GPT-3 175B | 12288 | 96 | 128 | 96 |
| T5-base | 768 | 12 | 64 | 12 encoder + 12 decoder |

The original `GPT-2: D=1024, heads=12, head_width=85` row mixed incompatible settings. Sources: [Transformer](https://arxiv.org/html/1706.03762v7), [BERT](https://arxiv.org/html/1810.04805v2), [GPT-2 small configuration](https://huggingface.co/openai-community/gpt2/blob/main/config.json), [GPT-2 large configuration](https://huggingface.co/openai-community/gpt2-large/blob/main/config.json), [GPT-3 Table 2.1](https://arxiv.org/html/2005.14165v4), [T5-base configuration](https://huggingface.co/google-t5/t5-base/blob/main/config.json).

BERT uses bidirectional encoder representations; its original pretraining combined masked-language modelling with next-sentence prediction. GPT-style causal models predict the next token. T5 uses an encoder-decoder arrangement. These distinctions explain why a head-count table alone cannot identify a model's full computation. [BERT pretraining](https://arxiv.org/html/1810.04805v2), [GPT-2 report](https://cdn.openai.com/better-language-models/language_models_are_unsupervised_multitask_learners.pdf), [T5-base configuration](https://huggingface.co/google-t5/t5-base/blob/main/config.json).

<a id="transformer-layout"></a>
### 10.2 Follow an actual shape through the whole operation

Take `B=2`, `T=3`, `D=8`, `H=2`, and `d=4`:

| Step | Shape | What the axes mean |
|---|---|---|
| Input X | `[2,3,8]` | batch, token, feature |
| Each Q/K/V projection | `[2,3,8]` | both heads' projected features |
| Reshape | `[2,3,2,4]` | batch, token, head, feature within head |
| Transpose axes 1 and 2 | `[2,2,3,4]` | batch, head, token, feature |
| Transpose K's last two axes | `[2,2,4,3]` | batch, head, feature, key |
| `Q @ Kᵀ` | `[2,2,3,3]` | batch, head, query, key |
| Last-axis softmax | `[2,2,3,3]` | one three-key distribution per query/head |
| `A @ V` | `[2,2,3,4]` | each head's output per token |
| Transpose axes 1 and 2 | `[2,3,2,4]` | restore token before head |
| Merge final two axes | `[2,3,8]` | concatenate the head outputs |
| Output projection | `[2,3,8]` | mix their information |

Reshape and transpose solve different problems. Reshape separates a feature index into `(head,feature_in_head)`; transpose puts the head axis where batched matrix multiplication expects it. A missing transpose can make tokens and heads trade meanings while leaving several numerical dimensions plausible.

The following is a **Candle fragment, source-checked and executed inside an educational CPU wrapper**. It assumes initialized `Linear` fields `q_lin`, `k_lin`, `v_lin`, `out_lin`, and positive `n_heads`, `head_dim` satisfying the configuration above. The surrounding function returns `candle_core::Result<Tensor>` and imports `Tensor`, `DType`, `D`, and `Module` from `candle_core`.

```rust
let (bs, seq, dim) = hidden_states.dims3()?;
// Forward must establish dim == n_heads * head_dim.
let split = |x: Tensor| -> candle_core::Result<Tensor> {
    x.reshape((bs, seq, n_heads, head_dim))?.transpose(1, 2)
};
let q = split(q_lin.forward(hidden_states)?)?.to_dtype(DType::F32)?;
let k = split(k_lin.forward(hidden_states)?)?.to_dtype(DType::F32)?;
let v = split(v_lin.forward(hidden_states)?)?.to_dtype(DType::F32)?;
let q = (q / (head_dim as f64).sqrt())?;
let mut scores = q.contiguous()?.matmul(&k.transpose(2, 3)?.contiguous()?)?;

if let Some(blocked_u8) = attention_mask {
    // U8, 1 = blocked; no query may have every key blocked.
    let blocked = blocked_u8.broadcast_as(scores.shape())?;
    let minus_inf = Tensor::full(f32::NEG_INFINITY, scores.shape(), scores.device())?;
    scores = blocked.where_cond(&minus_inf, &scores)?;
}
let weights = candle_nn::ops::softmax(&scores, D::Minus1)?;
let context = weights.matmul(&v.contiguous()?)?;
let context = context.transpose(1, 2)?.reshape((bs, seq, dim))?;
let context = context.to_dtype(hidden_states.dtype())?;
let output = out_lin.forward(&context)?;
```

This fragment was also exercised inside a small wrapper on the pinned Candle CPU backend with two heads, three tokens and a causal U8 mask; that confirms this example's execution, not all inputs or backends. Mask validity and input finiteness are **preconditions**, not implemented protections here. Forward must verify its input width or propagate an appropriate shape error. Weights and inputs need compatible device and dtype. Scores, softmax and value mixing use F32; the preceding Q/K/V projections still use their configured dtype, and F32 can also overflow.

Pinned Candle `transpose` changes shape/strides while sharing storage. `reshape` shares contiguous input storage and otherwise copies. `contiguous()` returns a clone if already contiguous and copies otherwise. The calls above choose simple layouts for illustration, not a fastest schedule. [Pinned Tensor implementation](https://github.com/huggingface/candle/blob/f5838914f788d3950d0a25042cffe199d9325a9e/candle-core/src/tensor.rs).

Upcasting *after* a low-precision `QKᵀ` cannot recover values that already overflowed. Also, F32 weights from softmax cannot simply be multiplied by an incompatible low-precision V tensor. Choose the accumulation path deliberately. For training dropout, add mode-aware behaviour; do not silently leave inference dropout enabled.

A tracing span marks a unit of work with structured context. In synchronous code, its entry guard exits the span when dropped, including on an early return. An enabled subscriber is needed to collect it; the span alone does not prove GPU execution timing, which must account for asynchronous device work. [tracing Span](https://docs.rs/tracing/latest/tracing/span/struct.Span.html).

<a id="transformer-cost"></a>
### 10.3 Count parameters, bytes and operations separately

With `dk=dv=D/H`, ordinary separate Q,K,V and output matrices each have `D²` weights. The total is `4D²`, or `4D²+4D` if all four have bias. For `D=8`, that is 256 weights and 32 biases, totaling 288 parameters. Changing H from 2 to 4 while keeping D fixed does not change this parameter count; the per-head width changes from 4 to 2.

Naively materialized attention scores contain `B*H*T²` elements. For `B=2`, `H=8`, `T=1024`, there are 16,777,216 elements, or 64 MiB at four bytes per F32 value. A separate probability tensor of the same shape takes another 64 MiB. Gradients, QKV, activations, optimizer state and temporary workspaces are additional allocations. A count of score elements is not a complete training-memory estimate.

With fixed total width, projection work scales roughly as `O(B*T*D²)` and dense attention matrix products as `O(B*T²*D)`. Keeping all scores additionally costs `O(B*H*T²)` storage. Which operation dominates depends on T, D, batching, hardware and implementation.

FlashAttention computes exact dense attention using tiling and an online softmax organization that reduces memory traffic and avoids storing the full attention matrix in high-bandwidth memory. “Exact” distinguishes its attention operation from approximations; floating-point execution still has rounding. It does not make dense all-pairs attention's arithmetic linear in sequence length. Sparse/windowed attention changes which pairs are considered, while linear-attention formulations change the calculation under their own assumptions. [FlashAttention paper](https://arxiv.org/abs/2205.14135).

Heads are expressed as parallel work, but speed is not guaranteed to be independent of their number. Smaller matrix shapes, extra score rows, kernel launches and memory movement affect latency. Measure the exact backend and workload before presenting a speed comparison.

<a id="transformer-position"></a>
### 10.4 Where order enters

Without position information or a fixed order-dependent mask, self-attention treats a sequence as an ordered presentation of a set: rearranging input rows rearranges output rows correspondingly. The word is **permutation-equivariant**, not permutation-invariant.

Here is the algebra. Let P be a permutation matrix that rearranges rows, and write `F(X)=softmax(QKᵀ)V`, omitting the constant scale. Under `X'=PX`, the projected tensors become `PQ`, `PK`, `PV`. Scores become `P(QKᵀ)Pᵀ`; their rows and columns are rearranged. Row softmax follows the same rearrangement, so

$$F(PX)=P\operatorname{softmax}(QK^T)P^TPV=PF(X).$$

The output is reordered, not unchanged. An invariant pooling step such as taking the mean over token outputs could remove that ordering at the next stage.

This argument does **not** hold under an arbitrary token permutation while keeping a causal mask fixed. The mask says which positions are before which others and is already an order-dependent structure. Position information also enters through positional embeddings, score biases or spatial operations. Therefore “all Transformers need a separate added position vector because attention has no order” is too broad.

The original sinusoidal construction adds a position vector to each input embedding. For paired dimensions,

$$PE_{p,2i}=\sin(p/10000^{2i/D}),\quad PE_{p,2i+1}=\cos(p/10000^{2i/D}).$$

For `D=4`, the position-zero vector is `[0,1,0,1]`; position one is approximately `[.841471,.540302,.010000,.999950]`. Different frequencies change at different rates, providing distinguishable position-dependent signals. The formula is defined beyond the training lengths, but useful extrapolation is an empirical property, not a guarantee. [Original positional encoding](https://arxiv.org/html/1706.03762v7).

Learned absolute embeddings use a trainable table indexed by position and require a policy outside that table. Relative-bias methods add a function of query/key positions to scores. Rotary position embeddings instead rotate paired query/key coordinates so their dot products encode relative-position effects; they are not simply an extra vector added to X. [RoFormer / rotary embeddings](https://arxiv.org/abs/2104.09864).

<a id="transformer-block"></a>
### 10.5 Attention is one part of the block

After attention gathers information across tokens, a positionwise feed-forward network transforms each token's features. A simple version is `FFN(x)=W₂ activation(W₁x+b₁)+b₂` in column notation. The same weights apply to every token separately. It mixes feature coordinates, while attention mixes information across token positions. This distinction concerns the operations; an FFN's inputs may already contain information gathered from other tokens.

Residual additions preserve a direct path around a sublayer: `y=x+F(x)`. A Jacobian is a matrix describing how each output coordinate changes with each input coordinate. Here it is `I+J_F`, where I is the identity matrix and `J_F` is the Jacobian of F, so a gradient has a path through the identity term. This can ease optimization of deep networks, but the terms can still interact or cancel. A residual connection does not prove the absence of exploding or vanishing gradients. The two branches must have compatible shapes; a projection on the skip path changes the identity argument. [Residual learning](https://arxiv.org/abs/1512.03385).

LayerNorm with normalized shape D computes a mean and variance across each token's D features, then applies learned per-feature scale and bias. It uses the current input's statistics in both training and evaluation. BatchNorm groups statistics by channel across its specified batch/spatial axes and normally uses stored running statistics at evaluation. Those different axes and state rules make the operations noninterchangeable. [LayerNorm API](https://docs.pytorch.org/docs/2.9/generated/torch.nn.LayerNorm.html), [Layer Normalization paper](https://arxiv.org/abs/1607.06450).

For a concrete LayerNorm calculation, token `[1,3]` has mean 2 and population variance 1. With scale `[1,1]`, bias `[0,0]`, and negligible epsilon, its result is approximately `[-1,1]`. Another token `[10,14]` is normalized using its own mean 12 and variance 4, also producing approximately `[-1,1]`. This operation did not average the two tokens together.

Two common block organizations are, omitting dropout:

```text
Post-norm:
    u = LayerNorm(x + Attention(x))
    y = LayerNorm(u + FFN(u))

Pre-norm:
    u = x + Attention(LayerNorm(x))
    y = u + FFN(LayerNorm(u))
```

The placement changes the forward function and gradient paths. Pre-norm models may also apply a final norm after the stack. A checkpoint designed for one arrangement cannot be ported by casually moving its norms. [Analysis of pre- and post-norm Transformers](https://arxiv.org/abs/2002.04745).

<a id="transformer-decoding"></a>
### 10.6 Parallel training and sequential generation coexist

During causal language-model training, the target text is already known. A sequence such as `[start, the, apple]` can be supplied in one tensor, with shifted targets `[the, apple, fell]`. A causal mask prevents each representation from reading later input positions, while the model computes losses for all positions together. The model is not shown the correct future token at the position predicting it.

During ordinary autoregressive generation, the next input token does not exist until a prediction has been selected. Generate it, append it, and repeat. Attention within each step can still use parallel matrix operations, but future generation steps depend on earlier choices.

A KV cache stores earlier per-layer key and value tensors. In causal inference those earlier representations do not change when new tokens arrive, so each new query can attend to cached K,V plus the current token's K,V. The cache saves recomputation; it does not remove the dependency between generated tokens. [Hugging Face cache explanation](https://huggingface.co/docs/transformers/main/cache_explanation).

For ordinary MHA with both cached widths D, B examples, T cached tokens and L layers, the uncompressed cache contains about `2*B*T*D*L` scalar values. Multiply by bytes per value for the storage estimate. Grouped-query or multi-query attention changes the number of distinct cached K/V heads, so that formula must change with the architecture. Cache position and masks must also agree: a new query at absolute position 100 is not at position zero merely because its query tensor has length one.

An encoder's unrestricted self-attention and a decoder's causal self-attention are uses of the same broad mechanism with different information access. Encoder-decoder cross-attention uses decoder queries and encoder keys/values; a causal triangle over source positions is not automatically appropriate.

<a id="transformer-practice"></a>
### 10.7 Practice, with worked answers

1. **D=12 and H=3. What is the usual head width?** Four. Each projection can use all 12 input features; its 12 projected features are then grouped into three heads.
2. **A programmer changes H from 3 to 6 while keeping D=12. Do projection parameters double?** No for the equal-total-width design. The head width halves, although score storage and execution characteristics can change.
3. **Why must a mask `[B,T]` become `[B,1,1,T]`?** Broadcasting aligns trailing axes. The explicit singleton axes identify the same key validity across all heads and queries instead of accidentally aligning batch with a different axis.
4. **A network has no added position vectors. Is it necessarily unable to distinguish order?** No. Inspect its causal mask, relative biases, rotations, convolutions and other order-dependent operations before applying the position-free equivariance argument.

<a id="yolo"></a>
## 11. From image features to YOLOv10's partial attention

<a id="yolo-tokens"></a>
### 11.1 An image location can be a token

A convolutional feature map commonly has shape `[B,C,H,W]`. One spatial location contains C feature values. Reshape the spatial axes into one index and move channels last to obtain `[B,H*W,C]`:

```rust
// Illustrative Candle fragment: xs is a Tensor with NCHW semantics.
let (batch, channels, height, width) = xs.dims4()?;
let tokens = xs.reshape((batch, channels, height * width))?.transpose(1, 2)?;
// Equivalent axis operation:
let tokens_short = xs.flatten_from(2)?.transpose(1, 2)?;
// Inverse for these tokens, preserving the original image dimensions:
let restored = tokens.transpose(1, 2)?.reshape((batch, channels, height, width))?;
```

For `[2,64,3,4]`, there are 12 tokens per image, each with 64 features: output `[2,12,64]`. A particular spatial coordinate maps to sequence index `row*width+column` under this row-major flattening. Returning to the image grid requires the inverse axis operations with the original height and width.

`dims4()` proves rank four, not that the axes really mean NCHW. That meaning comes from the producer. Nor does the reshape extract patches: it only reorganizes existing values. A Vision Transformer normally forms patch embeddings, such as flattening each non-overlapping patch and applying a learned projection. For a `32×32` image and `4×4` patches, there are 64 patches before any extra tokens, rather than 1024 individual pixels. [Vision Transformer paper](https://arxiv.org/abs/2010.11929).

Applying attention to a convolutional feature map uses locations whose features already describe receptive fields. It is therefore different from applying attention directly to raw RGB triples, even if both can be represented as `[B,T,D]` tensors.

<a id="yolo-detector"></a>
### 11.2 Detection needs categories and locations

The conveyor example becomes detection when the system must identify several objects and locate each one. A classifier answers “what is in this input?”; a detector must also produce boxes and distinguish separate instances.

The official YOLOv10 implementation uses a convolutional backbone, feature fusion across resolutions and detection outputs at multiple scales. Its configuration combines familiar convolutions with C2f/C2fCIB blocks, SCDown downsampling, SPPF and PSA. Follow the actual model configuration to determine which variant uses which blocks; the existence of a class in the repository does not establish its use in a particular network. [Pinned model configuration](https://github.com/THU-MIG/yolov10/blob/453c6e38a51e9d1d5a2aa5fb7f1014a711913397/ultralytics/cfg/models/v10/yolov10b.yaml).

The paper's efficiency changes include a lighter classification head, separating channel expansion from spatial downsampling, and choosing compact inverted blocks using rank analysis. Its accuracy changes include selected large-kernel convolutions and partial self-attention. These describe the 2024 YOLOv10 design; this handbook makes no claim that it is the best detector available on the handbook date. [YOLOv10 paper](https://arxiv.org/pdf/2405.14458).

<a id="yolo-psa"></a>
### 11.3 What “partial” means in PSA

Partial self-attention (PSA) processes only part of the channel features through attention. In the pinned implementation, an initial `1×1` convolution creates two channel groups. One group bypasses attention; the other follows a residual attention step and a residual FFN step. They are concatenated and fused by another `1×1` convolution:

```text
x -> pointwise convolution -> split into a and b
                             a: keep for later
                             b: b + attention(b)
                             b: b + FFN(b)
concat(a, b) -> pointwise convolution -> output
```

The pinned `PSA` class has one attention/FFN pair. The paper's `N_PSA` counts such pairs on the selected branch, not heads or spatial tokens. That general repetition parameter is absent from this concrete class interface. [Pinned Attention and PSA code](https://github.com/THU-MIG/yolov10/blob/453c6e38a51e9d1d5a2aa5fb7f1014a711913397/ultralytics/nn/modules/block.py#L701).

The channel split preserves all spatial locations. For a toy incoming `[1,256,20,20]` tensor and half-channel split, the attention branch has `[1,128,20,20]`. The pinned configuration rule `num_heads=branch_channels/64` gives two heads, with value width 64 and query/key width 32. There are still `20*20=400` queries and 400 keys per head. The attention score tensor contains `1*2*400*400=320,000` entries.

This also illustrates why `dk` and `dv` are different concepts. Q and K use width 32 for their dot product, so the scale is `1/sqrt(32)`. The values carry 64 features per head, so weighted values return 64 features per query per head. Narrowing Q/K does not change the `400×400` attention matrix dimensions.

The official attention code stores Q,K as `[B,heads,dk,N]` and V as `[B,heads,dv,N]`, where `N=H*W`. It computes `QᵀK`, then `V Aᵀ`, the transpose-layout equivalent of our `AV`. It adds a depthwise `3×3` convolution of the value map before output projection, contributing local spatial structure. [Pinned attention calculation](https://github.com/THU-MIG/yolov10/blob/453c6e38a51e9d1d5a2aa5fb7f1014a711913397/ultralytics/nn/modules/block.py#L714).

The convolution wrapper uses BatchNorm and optional activation, so `act=False` does not mean “remove normalization.” A pointwise convolution mixes channels at each location, whereas the depthwise spatial convolution mixes nearby locations within channels. BatchNorm's evaluation statistics can be folded into the preceding convolution under the conditions in [chapter 7](#normalization). It is not equivalent to replacing a language Transformer's LayerNorm without retraining. [Pinned Conv wrapper](https://github.com/THU-MIG/yolov10/blob/453c6e38a51e9d1d5a2aa5fb7f1014a711913397/ultralytics/nn/modules/conv.py).

<a id="yolo-resolution"></a>
### 11.4 Why the attention is placed at low resolution

At `20×20`, N is 400 and one score matrix has 160,000 entries. At `80×80`, N is 6,400 and the matrix has 40,960,000 entries: **256 times as many**, even though height and width are each only four times larger. With two heads and F32 scores, the examples require about 1.22 MiB versus 312.5 MiB for scores alone.

This arithmetic explains the cost pressure independently of a benchmark. Lower-resolution features reduce the all-pairs term strongly; the channel split and narrower Q/K reduce other terms. None removes quadratic growth with the number of attended locations.

The paper places PSA after the lowest-resolution stage and, in implementation details, after SPPF. It uses FFN expansion factor 2 and sets the width scale of YOLOv10-M to 1.0 to obtain YOLOv10-B. Appendix A.1 reports 500-epoch SGD training from scratch; benchmark numbers belong to its stated T4/TensorRT FP16 setting, not to this Rust environment. [YOLOv10 methodology and Appendix A.1](https://arxiv.org/pdf/2405.14458).

<a id="yolo-assignments"></a>
### 11.5 Why removing NMS is a training-design question

Several predicted boxes may describe the same apple. Non-maximum suppression, or NMS, traditionally removes overlapping lower-scored candidates according to a chosen rule. Merely deleting NMS from a detector trained to emit several positives for each object does not teach it to emit a suitable final set.

YOLOv10 trains one-to-many and one-to-one detection branches with consistent matching criteria, then uses one-to-one predictions for its NMS-free deployment path. This objective is distinct from PSA: attention supplies features, while assignment determines how predictions receive supervision. [Official YOLOv10 project](https://github.com/THU-MIG/yolov10).

The pinned detector code detaches backbone/neck feature tensors on the one-to-one branch. That branch can train its own head parameters, while those particular gradients do not continue through the detached feature inputs. The one-to-many branch supplies feature-learning supervision. This is a concrete example of why “both losses train every parameter” would be an inaccurate description. [Pinned v10Detect](https://github.com/THU-MIG/yolov10/blob/453c6e38a51e9d1d5a2aa5fb7f1014a711913397/ultralytics/nn/modules/head.py#L445).

NMS-free does not mean there are no output-handling steps. The export path still decodes predictions and selects a bounded set of scored detections. The official repository specifically warns that non-exported execution can run unnecessary one-to-many-head operations during inference, biasing a speed measurement. Comparing export latency with an unoptimized eager path is not a clean comparison. [Repository benchmark note](https://github.com/THU-MIG/yolov10#notes).

<a id="yolo-practice"></a>
### 11.6 Practice, with worked answers

1. **Does flattening `[1,3,32,32]` to `[1,1024,3]` implement ViT patch embedding?** No. It creates one token per pixel. Patch extraction and learned projection are separate operations.
2. **Does halving PSA channels halve the sequence length?** No. N remains `H*W`. The processed feature width changes.
3. **If Q/K head width is 32 and V head width is 64, what is the attention scale?** `1/sqrt(32)`, because each compatibility score sums 32 coordinate products.
4. **A port reproduces PSA shapes. Does that establish YOLOv10 detection parity?** No. It must also reproduce the selected architecture, parameter naming/layout, preprocessing, normalization state, detection decoding and output selection. Shape agreement is one necessary check, not the whole model contract.

Use the [integrated practice chapter](#practice) to combine these checks with the Rust ownership, parameter and tensor lessons.

<a id="practice"></a>
## 12. Turn the chapters into skills you can use

Reading gives you vocabulary. Expertise appears when you can independently derive a result, choose the right tool, and diagnose a failure. The following projects are a suggested progression, not a fixed timetable or a promise of mastery. Move forward when you can explain the result and its limits.

<a id="practice-rust"></a>
### 12.1 Rust: own the data, expose the errors, make the contract visible

Build a small program that reads fruit measurements and returns either a well-formed example or a precise error. A record contains a name, a few floating-point measurements, and a class label. Start with a function over a borrowed string so parsing can be understood separately from file I/O.

Make these decisions explicitly:

| Decision | Explain it before implementation |
|---|---|
| Owned or borrowed name | Does the returned record need to outlive the input text? |
| `Option` or `Result` | Is a field legitimately absent, or is the record invalid? |
| Typed error or application context | Does the caller branch on the cause, or mainly present a useful report? |
| Iterator collection | Should the first invalid record stop the operation, or should all problems be collected? |
| Integer or floating-point label | Does the chosen classification loss require class indices? |
| `clone()` | What exactly is copied for this type, and why is that necessary? |

Explain the behavior for empty data, malformed numbers, NaN, infinity, an unknown label, and a record with the wrong number of features. These are product decisions; compiling the happy path does not make them disappear.

Then write two interfaces: one accepting `&[Record]`, and one consuming `Vec<Record>`. Explain which operations each allows and why passing ownership can avoid a clone. Finally, replace a concrete transformation with a generic function bounded by a small trait. Explain when you would instead accept `&dyn Trait`. This applies [chapter 2](#rust) to an actual program rather than memorizing syntax.

This handbook concentrates on the Rust needed for the listed ML topics. Broader Rust expertise also requires practice with API design, concurrency, async cancellation, unsafe boundaries, profiling, and packaging. Use the official [Rust Book](https://doc.rust-lang.org/book/) to extend the language path and the [Rust Reference](https://doc.rust-lang.org/reference/) to resolve precise rules. Add those subjects when your application needs them; do not hide an unfinished grasp of borrowing behind more framework code.

<a id="practice-learning"></a>
### 12.2 MLP: predict a gradient before trusting automatic differentiation

Use the tiny MLP in [chapter 4](#mlp). Record the inputs, parameter values, intermediate activations, prediction, and loss. Derive every gradient by hand. Only then compare the library's calculation with yours.

Change exactly one thing at a time. Move a hidden pre-activation from positive to negative and explain what ReLU does to that path. Change a sum loss to a mean and predict the factor in the gradients. Increase the learning rate and observe whether the next loss decreases. A rising loss is an observation to explain, not a reason to silently try random code changes.

For a small trainable fruit classifier, use features and labels with an actual relationship. Keep a held-out validation set that is not used to update weights. Fit any feature-normalization statistics using training data alone, then reuse those statistics for validation and inference. Random labels independent of inputs can exercise a training loop; they do not demonstrate meaningful generalization.

The completion criterion is stronger than “the loss went down”: you can explain the loss contract, the gradient scale, which parameters the optimizer updates, how validation differs from training, and how preprocessing and label order accompany saved weights.

<a id="practice-vision"></a>
### 12.3 Vision: choose which information to mix

Take one 32×32 RGB crop of a fruit. Begin with shape `[1,3,32,32]` and predict the output of every layer before executing it.

| Operation | Example choice | Shape prediction |
|---|---|---|
| Ordinary convolution | 3 input channels, 8 output channels, 3×3, stride 1, padding 1 | `[1,8,32,32]` |
| Pointwise convolution | 8 input channels, 16 output channels, 1×1, stride 1, padding 0 | `[1,16,32,32]` |
| Depthwise convolution | 16 input channels, multiplier 1, 3×3, stride 2, padding 1 | `[1,16,16,16]` |
| Pointwise channel mixing | 16 input channels, 32 output channels | `[1,32,16,16]` |
| Global spatial mean | Average the 16×16 positions for each channel | `[1,32]` |
| Classifier | 32 features to 3 logits | `[1,3]` |

Without bias, the first ordinary convolution has `8×3×3×3=216` weights. The depthwise layer has `16×1×3×3=144`, and the last pointwise layer has `32×16=512`. These counts are arithmetic facts about the stated configuration. Actual latency depends on shapes, backend, memory movement, and execution details.

For a tiny-bruise task, examine what stride 2 discards and why a later larger image cannot automatically restore those measurements. Compare dilation with downsampling: dilation expands the set of input coordinates within one kernel's span, while stride changes where outputs are produced. Predict a transposed-convolution shape with the full formula, then show why that shape equality does not make it the inverse of the first convolution.

For an inference optimization, derive one Conv–BatchNorm fusion by hand, including a nonzero convolution bias, nontrivial gamma/beta, and the layer's actual epsilon. Compare outputs in evaluation mode and inspect the maximum difference. A match on your chosen inputs is execution evidence; the derivation explains the general identity and its assumptions. It says nothing about training-mode equivalence.

<a id="practice-attention"></a>
### 12.4 Attention: make every axis earn its place

Start with three tokens and one head. Use the explicit matrices in [chapter 9](#attention), and compute one output as a weighted sum of value rows. Alter one query and predict which row of scores changes. Alter one key and predict which score column changes. Alter one value and predict how its contribution changes across outputs.

Then add a causal mask. For query position 1, explain which key positions remain permitted under the chosen indexing convention. Check that the softmax runs across permitted keys. Do not conflate masking a key with removing a padded query's output from the loss.

Expand to two heads. Write the full sequence of shapes, not just the initial and final ones. Use easily recognized values to distinguish a reshape from a transpose. Confirm that the per-head output concatenation restores token order before the output projection.

Finally, use a tiny image feature grid. Flatten its positions into tokens and invert that layout conversion after attention. Explain how this differs from creating ViT patch embeddings. Compare full attention with PSA: ask which channels are processed and how many spatial tokens remain. This joins [Candle layouts](#candle), [convolution](#conv), and [YOLOv10](#yolo) in one concrete exercise.

<a id="diagnosis"></a>
### 12.5 A diagnostic sequence for Rust ML programs

When an experiment fails, collect the facts in this order. The point is to narrow the cause, not to print every tensor.

1. **Input meaning.** Is the image RGB or BGR? Are axes NCHW or NHWC? Were features scaled using the training convention? Are class indices in the intended order?
2. **Shapes.** Record each layer's input/output rank and dimensions. Check channel/group divisibility, head divisibility, reshape element counts, and mask broadcasting.
3. **Dtypes and devices.** Trace both operands into each failing operation. A cast after overflow cannot recover the lost value. A Rust `f64` scalar does not make every tensor operation double precision.
4. **Numerical values.** Inspect small summaries: finite counts, min/max, mean where meaningful. Locate the first operation producing NaN or infinity; do not hide it with a replacement distribution.
5. **Parameters and state.** Confirm the names and shapes loaded, which variables the optimizer holds, whether the model was constructed before a VarMap load, and whether normalization state is included.
6. **Learning mode.** Separate gradient recording, optimizer updates, and the train/eval behavior of dropout and normalization. These are different decisions.
7. **Objective.** Check logits versus probabilities, target dtype/shape, masks, reduction, and sample weighting. A plausible loss curve cannot repair a wrong target contract.
8. **Execution evidence.** Reproduce on a small CPU example when feasible, then investigate the intended backend. Measure real workload latency with appropriate synchronization and include preprocessing/postprocessing in any end-to-end claim.

Use error messages as evidence. A shape error tells you that the operands disagree; it does not tell you which operand has the wrong meaning. A successful reshape establishes element-count compatibility; it does not establish a correct token permutation. A checkpoint file opening successfully establishes file access; it does not establish model compatibility.

<a id="mastery-questions"></a>
### 12.6 Questions you should be able to answer without the notes

| Question | Reasoned answer |
|---|---|
| Why can Rust accept a tensor computation that is mathematically wrong? | `Tensor` does not encode every axis meaning in its Rust type; even compatible shapes can represent the wrong semantics. |
| Why does moving a builder not necessarily move the variable map? | They are separate Rust values; the builder backend can hold a cloned shared handle to the map. |
| Why can a saved weight file be insufficient to resume training? | Architecture, preprocessing, label mapping, optimizer state, and other training state may be needed in addition to named tensors. |
| Why is a 1×1 layer useful if it looks at one spatial location? | It can combine all connected input channels there, with weights reused across locations. |
| Why does depthwise convolution usually need a pointwise partner? | Depthwise spatial filters do not mix separate input channels; the pointwise layer supplies cross-channel mixing. |
| Why does a large dilation not guarantee dense context? | A kernel spans a large region but samples a sparse set of coordinates; stacking pattern and earlier layers determine connectivity. |
| Why is a transposed convolution not an inverse? | It applies the transpose of a linear operator under matching conventions; a transpose generally does not undo lost or mixed information. |
| Why can BatchNorm be folded at inference? | Fixed mean/variance and affine parameters turn it into a per-output-channel scale and shift. |
| Why does self-attention not mean Q, K, and V are equal? | They can originate from the same input through different learned projections. |
| Why is position information a separate concern? | Unmasked attention without positional structure treats a consistent token permutation equivariantly; it does not automatically know sequence order. |
| Why does FlashAttention not make dense attention linear-time? | It changes how exact attention is evaluated and stored; the dense pairwise mathematical work remains quadratic in sequence length. |
| Why can fewer arithmetic operations still run slower? | Kernel efficiency, memory traffic, layouts, launch overhead, and hardware utilization also determine elapsed time. |

When answering, connect the equation to a program and a small example. That is the standard to aim for throughout this handbook.

<a id="glossary"></a>
## Appendix A. A working glossary

| Term | Meaning in this handbook |
|---|---|
| Activation | A layer's intermediate output; “activation function” means a function such as ReLU applied to a value. |
| Affine map | A linear transformation plus a bias, such as `xWᵀ+b`. |
| Autograd | Automatic differentiation through supported recorded operations; it computes derivatives, not parameter updates by itself. |
| Axis | One indexing direction of a tensor, whose intended meaning you must document. |
| Batch | A group of examples processed together; batch size is not the same as the number of values in every normalization population. |
| Bias | A trainable additive offset, often one value per output feature or channel. |
| Broadcast | Reuse values across compatible axes according to an operation's rules. |
| Channel | One feature plane at every spatial position; deeper channels need not correspond to colors. |
| Checkpoint | Saved state whose exact contents depend on the saving API; a tensor-only checkpoint is not a complete training session. |
| Contiguous | An arrangement of logical elements that follows the library's contiguous storage layout. |
| Cross-attention | Queries and keys/values come from different input streams or representations. |
| Cross-correlation | Slide a kernel without mathematically reversing its spatial coordinates; usual CNN “convolution” convention. |
| Dilation | Spacing between kernel samples along an input axis. |
| Dtype | The numerical element type stored in a tensor, such as F32; distinct from the Rust type `Tensor`. |
| Epoch | One pass over a specified training dataset, subject to the loader's sampling/drop policy. |
| Feature | A measured or learned coordinate in a representation. |
| Gate | An input-dependent multiplier that modulates a value or feature. |
| Gradient | Derivatives of a scalar quantity with respect to values or parameters. |
| Grouped convolution | Convolution with restricted, partitioned connectivity between input and output channels. |
| Hyperparameter | A design/training choice such as learning rate, width, or dilation, rather than an ordinary weight updated by backpropagation. |
| Inference | Producing outputs from a model; its normalization/dropout state and gradient-recording choices must still be explicit. |
| Kernel | Either convolution weights or a device program implementing an operation; context distinguishes the meanings. |
| Logit | An unnormalized score before a probability transformation such as softmax. |
| Loss | The scalar objective used to judge the selected predictions and targets. |
| MAC | Multiply–accumulate; reporting FLOPs requires stating the counting convention. |
| Mask | A rule excluding or altering selected interactions or loss contributions; polarity and axes depend on the API. |
| Parameter | A model value selected to be learned or retained; not every tensor is automatically in an optimizer. |
| Pointwise convolution | A spatial 1×1 convolution, usually used to mix channels at each selected position. |
| Receptive field | Input coordinates that can influence an output under the stated computational graph; distinguish the bounding span from actual sampled connectivity. |
| Residual connection | Add an input or projected shortcut to a learned branch with compatible shapes. |
| Self-attention | Queries, keys, and values are derived from the same source representation, usually by different learned projections. |
| Stride | Step between successive output-window starting positions in input coordinates. |
| Token | One sequence element; in vision it may represent a patch or feature-grid location rather than a single pixel. |
| Transposed convolution | The transpose/adjoint convolutional linear mapping under matching conventions, often used in learned upsampling. |
| View | A tensor interpretation that can share underlying storage with another tensor; exact behavior is operation-dependent. |

<a id="corrections"></a>
## Appendix B. What changed from the original notes

These are consequential corrections, not changes in terminology for their own sake. The linked chapters include the derivations or primary evidence.

| Earlier claim or pitfall | Corrected understanding |
|---|---|
| Candle rows always mean items; dot products are “the projection” | Axes have a declared meaning; distinguish weighted scores, scalar/vector projections, and cosine. [Foundations](#foundations) |
| “Safe” softmax can return a uniform distribution after invalid arithmetic | The educational function rejects invalid input; finite precision still permits underflow and rounding. Numerical examples have been recalculated. [Softmax](#softmax) |
| Turbofish converts a tensor's dtype | It selects generic arguments; extraction and conversion are different operations. [Rust](#rust), [Candle](#candle) |
| Moving a VarBuilder necessarily prevents later use of the VarMap | Separate bindings can retain shared handles. Parameter names are runtime strings, and backend behavior matters. [Candle](#candle) |
| Loading an empty VarMap discovers every checkpoint parameter | At the pinned revision, load updates already-created variables; construct the intended model first for that restoration path. [Candle](#candle) |
| Computing a loss is a complete training loop | Gradients and optimizer updates are separate required steps; manual backpropagation must use the forward-pass parameter snapshot. [MLP](#mlp) |
| 1×1 always preserves image dimensions; depthwise always keeps channel count | Shape depends on stride/padding; depthwise permits a positive channel multiplier. [Convolution](#conv) |
| Dilation creates guaranteed dense/global context; coprime rates guarantee no holes | Bounding width, sampling connectivity, and learned influence differ. The original mixed-stride receptive-field arithmetic has been corrected. [Spatial operations](#spatial) |
| Transposed convolution inverts convolution; output padding adds zero pixels | A transpose is generally not an inverse; output padding resolves output-size ambiguity under the API contract. [Spatial operations](#spatial) |
| BatchNorm outputs always have variance one, and its benefits have one settled explanation | Epsilon and affine parameters matter; historical hypotheses and later optimization evidence must be distinguished. [Normalization](#normalization) |
| Conv–BatchNorm fusion applies equally during training and inference | Fixed statistics permit an affine fold; epsilon, bias, channel broadcasting, and other graph consumers matter. [Fusion](#normalization-fusion) |
| Canonical sigmoid SE gates amplify their immediate inputs | Gates lie between 0 and 1 mathematically; they change relative emphasis and attenuate absolute magnitude. [SE](#se) |
| Self-attention means Q=K=V; head count is free parallelism | The source may be shared while projections differ; parameter counts, dimensions, and execution costs need explicit assumptions. [Attention](#attention), [Transformer](#transformer) |
| Position-free attention is invariant to token order | Under the specified unmasked assumptions it is permutation-equivariant; fixed masks change those assumptions. [Transformer](#transformer) |
| ViT attention normally runs between individual pixels; SAM used billions of images | ViT uses patch tokens; the cited SAM scale distinguishes masks from images. [Spatial architectures](#spatial-architectures) |
| YOLOv10 PSA is adequately explained by generic MultiheadAttention code | The actual PSA/Attention implementation has channel splitting, spatial attention, a positional branch, and a residual FFN. [YOLOv10](#yolo) |

Unsupported speed, accuracy, “production-ready,” and “tested” labels have not been carried forward. Where code was illustrative or incompatible with the pinned API, the integrated explanation uses corrected code or identifies the example as a fragment. The original files remain as historical inputs.

<a id="source-index"></a>
## Appendix C. Index of all 35 original notes

The source filenames below refer to the original `candle_practice/docs` directory. This is a concept-preserving consolidation: repeated versions point to a shared explanation, while distinct material is retained in the listed chapters. The handbook can be read without opening those originals.

| # | Original note | Where its material is covered |
|---|---|---|
| 1 | `00_1×1_Convolutions.md` | [Convolution](#conv): channel mixing, linear equivalence, bottlenecks, architecture examples, global pooling, and parameter/compute counts. |
| 2 | `00_Batch Normalization2.md` | [Normalization](#normalization): populations, affine parameters, running statistics, train/eval, variants, and difficult batch settings. |
| 3 | `00_conv2d-batchnorm-fusion-guide.md` | [Fusion](#normalization-fusion): full derivation, epsilon, absent bias/affine cases, graph safety, and Candle behavior. |
| 4 | `00_Convolutions_grouped.md` | [Convolution](#conv): group connectivity, weight layout, divisibility, and depthwise relationship. |
| 5 | `00_Convolutions_grouped1.md` | [Convolution](#conv): interaction limits, channel shuffle, and compute versus measured runtime. |
| 6 | `00_ConvTranspose.md` | [Spatial operations](#spatial): convolution matrices, adjoints, spatial indexing, and implementation strategies. |
| 7 | `00_ConvTranspose2.md` | [Spatial operations](#spatial): zero insertion, overlap, output size, output padding, and artifacts. |
| 8 | `00_ConvTranspose3.md` | [Spatial operations](#spatial): the repeated transposed-convolution explanation is consolidated into the same derivation. |
| 9 | `00_depthWiseConv.md` | [Convolution](#conv): per-channel spatial filtering followed by channel mixing. |
| 10 | `00_depthWiseConv1.md` | [Convolution](#conv): dimensional terminology, depth multiplier, and dense/separable arithmetic. |
| 11 | `00_dilation_enhanced.md` | [Spatial operations](#spatial): corrected stride/dilation recurrence, output stride, padding, alignment, and coverage. |
| 12 | `00_dilation.md` | [Spatial operations](#spatial): context/detail tradeoffs, effective kernels, gridding, hybrid dilation, ASPP, and temporal use. |
| 13 | `00_rust-result-error-guide.md` | [Rust](#rust): ownership through results, `?`, conversions, typed errors, trait objects, and context. |
| 14 | `00_Squeeze-and-Excitation(SE).md` | [SE](#se): global descriptor, bottleneck, sigmoid, spatial broadcast, residual placement, and cost. |
| 15 | `00_Squeeze-and-Excitation(SE)2.md` | [SE](#se): corrected code, gating interpretation, variants, and worked examples. |
| 16 | `00_turbo_fish.md` | [Rust](#rust), [Candle](#candle): generics/inference, collection types, tensor extraction versus conversion. |
| 17 | `00_VarBuilder.md` | [Candle](#candle): Var/VarMap/VarBuilder, backend selection, prefixes, initialization, and checkpoint construction. |
| 18 | `00_VarBuilder1.md` | [Candle](#candle): shared ownership, parameter reuse, optimizer membership, saved-state limits, and restoration. |
| 19 | `00_why_Batch Normalization_works.md` | [Normalization](#normalization): original motivation, optimization evidence, and limits of the explanation. |
| 20 | `01_dotProductAndGradientDescent.md` | [Foundations](#dot-projection), [MLP](#mlp): score/projection distinction, parameter roles, gradient direction, and actual updates. |
| 21 | `01_safe_softmax.md` | [Softmax](#softmax): shifted formula, corrected arithmetic, fallible Rust implementation, edge cases, and log-space losses. |
| 22 | `01_structureOfMatrices.md` | [Tensor axes](#tensor-axes), [matrix products](#matrix-products): matrix conventions and `XWᵀ` dimensions. |
| 23 | `2_attention_misc.md` | [Candle](#candle), [YOLOv10](#yolo): builder scoping, rank checks, image-to-token layouts, and reverse conversion. |
| 24 | `2_attention.md` | [Attention](#attention): general attention, self/cross distinction, and masking. |
| 25 | `2_self_attention_detailed.md` | [Attention](#attention), [Transformer](#transformer): projections, configuration guards, head reshaping, scaling, mask, softmax, and output. |
| 26 | `2_self_attention_detailed2.md` | [Attention](#attention), [Transformer](#transformer): numerical retrieval example, head dimensions, gradients, compute/memory, and optimizations. |
| 27 | `2_self_attention_detailed3.md` | [Attention](#attention), [Transformer](#transformer): different key/value widths, score asymmetry, parameter counts, and head interpretations. |
| 28 | `2_self_attention_detailed4.md` | [Transformer](#transformer): positions, sinusoidal/learned/relative schemes, equivariance, and causal qualifications. |
| 29 | `3_attention_misc_2.md` | [Transformer](#transformer): mask polarity, raw matmul versus modules, packed QKV, layout, dtype, dropout, and tracing limits. |
| 30 | `04_mlp_multi_layer_perceptron.md` | [Candle](#candle): parameter registration, optional bias, naming, initialization, freezing, and PyTorch comparison. |
| 31 | `04_mlp_multi_layer_perceptron2.md` | [MLP](#mlp), [practice](#practice): forward/backward computation, SGD/Adam, initialization, data preparation, regularization, monitoring, and model tradeoffs. |
| 32 | `04_mlp_multi_layer_perceptron3.md` | [Candle](#candle), [MLP](#mlp): corrected trainer/API contracts and the parameter/learning bridge. |
| 33 | `21_self_attention_detailsed.md` | [Transformer](#transformer): annotated multi-head layout, biases in parameter counts, dtype alignment, and memory costs. |
| 34 | `denseAndSptial.md` | [Spatial architectures](#spatial-architectures): dense prediction, context versus detail, encoder/decoder, skip connections, patch/window attention, and annotation. |
| 35 | `1_yolov10.md` | [YOLOv10](#yolo): original paper and actual code, PSA, head/FFN dimensions, repeated-block interpretation, and detector integration. |

<a id="reading-index"></a>
## Appendix D. Primary-source reading routes

Technical references appear beside the claims they support. Use this table to decide where to go deeper. Mutable documentation can evolve; use the immutable commit links in the chapters when reproducing this handbook's Candle examples.

| To investigate | Start with |
|---|---|
| Precise Rust syntax and semantics | [Rust Reference](https://doc.rust-lang.org/reference/), then the specific standard-library type/method linked in chapter 2. |
| Candle's actual implementation | [The project's pinned Candle tree](https://github.com/huggingface/candle/tree/f5838914f788d3950d0a25042cffe199d9325a9e), especially `candle-core/src` and `candle-nn/src`. |
| Convolution and normalization API details | The official operator documentation linked in chapters 5–7; distinguish PyTorch's API from Candle's. |
| Original architecture motivation | The original papers linked beside pointwise, depthwise, dilation, SE, residual, attention, and YOLO discussions. |
| Numerical softmax behavior | [Blanchard, Higham, and Higham](https://eprints.maths.manchester.ac.uk/2765/) and the official log-softmax contract linked in chapter 1. |
| YOLOv10 implementation choices | The [official repository](https://github.com/THU-MIG/yolov10) and chapter 11's pinned `Attention`, `PSA`, configuration, and detection-head sources. |

When a comment, blog, and implementation disagree, inspect the executable path and version before forming a conclusion. A source citation is a starting point for verification; an analogy is a starting point for understanding.

<a id="verification"></a>
## Appendix E. Verification and review for this edition

The 35 original notes were read in full. Their unique concepts were mapped into this handbook, and material claims were checked against official language/library documentation, the pinned Candle implementation, and original ML research. Three Astra collaborators divided the source audit across Rust/Candle, convolutions/normalization, and MLPs/attention/YOLO, then cross-reviewed the integrated candidate before this file was added. The source index records the consolidation; the original files were preserved.

| Evidence | What was checked |
|---|---|
| Rust compiler | Local Rust 1.98.1, edition 2024, `aarch64-apple-darwin`. |
| Standalone standard-library programs | The softmax, ownership/lifetime, typed feature-parser, and numerical attention programs were extracted from their Markdown blocks, compiled, and executed. |
| Candle CPU examples | The dtype-conversion and complete price-learning programs compiled and ran against the project's exact Candle source revision. The price model approached weights `[2,3]` and bias `[1]`; the displayed zero loss is rounded. |
| Function/module fragments | Educational CPU callers exercised the anyhow context function, buffered checkpoint loader, multi-head attention fragment, image/token conversion including its inverse, all four convolution constructors, Conv–BatchNorm evaluation/folding functions, and SE module. The fragments remain fragments in the book; their callers are not a delivered application. |
| Numerical reasoning | Worked matrix products, losses/gradients, attention weights, parameter/MAC counts, receptive-field recurrence, gridding offsets, normalization, and fusion were recalculated and cross-reviewed. |
| Navigation and preservation | Explicit section anchors, internal links, Markdown structure, and the 35-source coverage index were checked; the original source files were compared with their starting hashes. |

The Candle execution used an isolated CPU package pointing to an unmodified archive of commit `f5838914f788d3950d0a25042cffe199d9325a9e`. Its newly resolved transitive dependency lock was separate from the original application's `Cargo.lock`. This is evidence for the educational examples on that CPU environment, not a reconstruction of the original Metal-enabled project build.

The source project, dependencies, and preexisting changes were not edited. No Metal/CUDA execution, full detector training, learned-checkpoint parity, accuracy claim, latency benchmark, or complete ML application is established by this documentation revision. Those questions require the specific data, model, backend, and workload they concern. Exercises are proposed practice, not claims that those experiments were performed.

[Return to contents](#contents)
