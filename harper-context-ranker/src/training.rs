use std::fs::File;
use std::io::{self, BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::time::Instant;

use burn::{
    backend::Autodiff,
    module::{AutodiffModule, Module, Param},
    nn::{Embedding, EmbeddingConfig, Initializer},
    optim::{AdamConfig, GradientsParams, Optimizer},
    tensor::{
        Int, Tensor, TensorData,
        activation::log_softmax,
        backend::{AutodiffBackend, Backend},
        cast::ToElement,
    },
};
use burn_ndarray::{NdArray, NdArrayDevice};
use hashbrown::HashSet;
use rand::{SeedableRng, rngs::StdRng, seq::SliceRandom};

use crate::corpus::GROUPS;
use crate::ranker::{
    CONTEXT_RADIUS, CONTEXT_SLOTS, ContextRanker, ContextRankerConfig, PAD_IDX, UNK_IDX,
    combined_score, is_known_word,
};

const REJECTED_LOGIT: f32 = -1e4;

/// One typo in one sentence, with every spelling candidate that Harper proposes for it.
#[derive(Clone, Debug, PartialEq)]
pub struct RankerExample {
    pub group: u8,
    pub sentence_hash: u64,
    pub left: [u16; CONTEXT_RADIUS],
    pub right: [u16; CONTEXT_RADIUS],
    /// The position of the right word in `candidates`.
    pub target: u8,
    /// Word id and spelling score, in Harper's current order.
    pub candidates: Vec<(u16, i16)>,
}

impl RankerExample {
    pub fn write_to(&self, writer: &mut impl Write) -> io::Result<()> {
        writer.write_all(&[self.group, self.target, self.candidates.len() as u8])?;
        writer.write_all(&self.sentence_hash.to_le_bytes())?;
        for id in self.left.iter().chain(&self.right) {
            writer.write_all(&id.to_le_bytes())?;
        }
        for (id, score) in &self.candidates {
            writer.write_all(&id.to_le_bytes())?;
            writer.write_all(&score.to_le_bytes())?;
        }
        Ok(())
    }

    pub fn read_from(reader: &mut impl Read) -> io::Result<Option<Self>> {
        let mut head = [0u8; 3];
        match reader.read_exact(&mut head) {
            Ok(()) => {}
            Err(err) if err.kind() == io::ErrorKind::UnexpectedEof => return Ok(None),
            Err(err) => return Err(err),
        }
        let [group, target, count] = head;

        let mut hash = [0u8; 8];
        reader.read_exact(&mut hash)?;

        let mut context = [0u8; CONTEXT_RADIUS * 4];
        reader.read_exact(&mut context)?;
        let ids: Vec<u16> = context
            .as_chunks::<2>()
            .0
            .iter()
            .map(|b| u16::from_le_bytes(*b))
            .collect();

        let mut items = vec![0u8; count as usize * 4];
        reader.read_exact(&mut items)?;
        let candidates = items
            .as_chunks::<4>()
            .0
            .iter()
            .map(|b| {
                (
                    u16::from_le_bytes([b[0], b[1]]),
                    i16::from_le_bytes([b[2], b[3]]),
                )
            })
            .collect();

        Ok(Some(Self {
            group,
            sentence_hash: u64::from_le_bytes(hash),
            left: ids[..CONTEXT_RADIUS].try_into().unwrap(),
            right: ids[CONTEXT_RADIUS..].try_into().unwrap(),
            target,
            candidates,
        }))
    }

    /// The word ids by context slot.
    fn context(&self) -> [u16; CONTEXT_SLOTS] {
        std::array::from_fn(|slot| {
            if slot < CONTEXT_RADIUS {
                self.left[slot]
            } else {
                self.right[slot - CONTEXT_RADIUS]
            }
        })
    }

    fn has_known_context(&self) -> bool {
        self.context().iter().any(|id| is_known_word(*id))
    }
}

pub fn read_vocabulary(path: impl AsRef<Path>) -> Vec<String> {
    serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap()
}

/// Read every `.bin` shard in a directory, in file name order.
pub fn read_examples(dir: impl AsRef<Path>) -> io::Result<Vec<RankerExample>> {
    let mut shards: Vec<PathBuf> = std::fs::read_dir(dir)?
        .map(|entry| entry.map(|e| e.path()))
        .collect::<Result<_, _>>()?;
    shards.retain(|path| path.extension().is_some_and(|ext| ext == "bin"));
    shards.sort();

    let mut examples = Vec::new();
    for shard in shards {
        let mut reader = BufReader::with_capacity(1 << 20, File::open(shard)?);
        while let Some(example) = RankerExample::read_from(&mut reader)? {
            examples.push(example);
        }
    }

    Ok(examples)
}

#[derive(Clone, Debug)]
pub struct TrainingOptions {
    pub epochs: usize,
    pub learning_rate: f64,
    pub batch_size: usize,
    /// The right word and the hardest wrong candidates, per example.
    pub candidates_per_example: usize,
    pub seed: u64,
}

/// Fixed-width training rows: the context, then the right word, then wrong candidates.
struct TrainingRows {
    rows: Vec<u16>,
    width: usize,
}

impl TrainingRows {
    fn new(train: Vec<RankerExample>, held_out: &HashSet<u64>, candidates: usize) -> Self {
        let width = CONTEXT_SLOTS + candidates;
        let mut seen = HashSet::with_capacity(train.len());
        let mut rows = Vec::with_capacity(train.len() * width);

        for example in train {
            if held_out.contains(&example.sentence_hash)
                || !seen.insert(example.sentence_hash)
                || !example.has_known_context()
            {
                continue;
            }

            let target = example.target as usize;
            let start = rows.len();
            rows.extend(example.context());
            rows.push(example.candidates[target].0);
            rows.extend(
                example
                    .candidates
                    .iter()
                    .enumerate()
                    .filter(|(i, _)| *i != target)
                    .map(|(_, (id, _))| *id)
                    .take(candidates - 1),
            );
            rows.resize(start + width, PAD_IDX as u16);
        }

        Self { rows, width }
    }

    fn len(&self) -> usize {
        self.rows.len() / self.width
    }

    fn batch<B: Backend>(
        &self,
        indices: &[u32],
        device: &B::Device,
    ) -> (Tensor<B, 2, Int>, Tensor<B, 2, Int>) {
        let mut context = Vec::with_capacity(indices.len() * CONTEXT_SLOTS);
        let mut candidates = Vec::with_capacity(indices.len() * (self.width - CONTEXT_SLOTS));

        for index in indices {
            let row = &self.rows[*index as usize * self.width..][..self.width];
            context.extend(row[..CONTEXT_SLOTS].iter().map(|id| *id as i32));
            candidates.extend(row[CONTEXT_SLOTS..].iter().map(|id| *id as i32));
        }

        (
            Tensor::from_data(
                TensorData::new(context, [indices.len(), CONTEXT_SLOTS]),
                device,
            ),
            Tensor::from_data(
                TensorData::new(candidates, [indices.len(), self.width - CONTEXT_SLOTS]),
                device,
            ),
        )
    }
}

#[derive(Module, Debug)]
struct TableModel<B: Backend> {
    table: Embedding<B>,
    word_weights: Embedding<B>,
    slot_weights: Param<Tensor<B, 1>>,
}

impl<B: Backend> TableModel<B> {
    fn new(vocab_len: usize, dimensions: usize, device: &B::Device) -> Self {
        Self {
            table: EmbeddingConfig::new(vocab_len, dimensions)
                .with_initializer(Initializer::Normal {
                    mean: 0.0,
                    std: 0.1,
                })
                .init(device),
            word_weights: EmbeddingConfig::new(vocab_len, 1)
                .with_initializer(Initializer::Ones)
                .init(device),
            slot_weights: Param::from_tensor(Tensor::ones([CONTEXT_SLOTS], device)),
        }
    }

    fn logits(&self, context: Tensor<B, 2, Int>, candidates: Tensor<B, 2, Int>) -> Tensor<B, 2> {
        let [batch, slots] = context.dims();
        let present = context
            .clone()
            .not_equal_elem(PAD_IDX as i32)
            .float()
            .reshape([batch, slots, 1]);
        let slot_weights = self.slot_weights.val().reshape([1, slots, 1]);

        let context_vector = (self.table.forward(context.clone())
            * self.word_weights.forward(context)
            * slot_weights
            * present)
            .sum_dim(1)
            .swap_dims(1, 2);
        let rejected = candidates.clone().equal_elem(PAD_IDX as i32);

        self.table
            .forward(candidates)
            .matmul(context_vector)
            .squeeze_dim::<2>(2)
            .mask_fill(rejected, REJECTED_LOGIT)
    }

    /// The bytes of `table.bin`, with each value of the table reduced to one `i8`.
    fn encode(&self) -> Vec<u8> {
        let [_, dimensions] = self.table.weight.dims();
        let rows: Vec<f32> = self.table.weight.val().into_data().iter().collect();
        let word_weights: Vec<f32> = self.word_weights.weight.val().into_data().iter().collect();
        let slot_weights: Vec<f32> = self.slot_weights.val().into_data().iter().collect();
        let limit = rows[dimensions * (PAD_IDX + 1)..]
            .iter()
            .fold(0f32, |limit, value| limit.max(value.abs()));

        limit
            .to_le_bytes()
            .into_iter()
            .chain(slot_weights.iter().flat_map(|weight| weight.to_le_bytes()))
            .chain(word_weights.iter().flat_map(|weight| weight.to_le_bytes()))
            .chain(rows.iter().enumerate().map(|(position, value)| {
                if position / dimensions == PAD_IDX {
                    0
                } else {
                    (value / limit * i8::MAX as f32).round() as i8 as u8
                }
            }))
            .collect()
    }
}

pub fn train_context_ranker_cpu(
    train: Vec<RankerExample>,
    tuning: &[RankerExample],
    test: &[RankerExample],
    words: &[String],
    dimensions: usize,
    options: &TrainingOptions,
    output: &Path,
) -> ContextRanker {
    train_context_ranker::<Autodiff<NdArray>>(
        train,
        tuning,
        test,
        words,
        dimensions,
        options,
        output,
        NdArrayDevice::Cpu,
    )
}

#[allow(clippy::too_many_arguments)]
pub fn train_context_ranker<B: AutodiffBackend>(
    train: Vec<RankerExample>,
    tuning: &[RankerExample],
    test: &[RankerExample],
    words: &[String],
    dimensions: usize,
    options: &TrainingOptions,
    output: &Path,
    device: B::Device,
) -> ContextRanker {
    let held_out: HashSet<u64> = tuning
        .iter()
        .chain(test)
        .map(|example| example.sentence_hash)
        .collect();

    let total = train.len();
    let rows = TrainingRows::new(train, &held_out, options.candidates_per_example);
    println!(
        "{} of {total} train examples remain after filters",
        rows.len()
    );

    B::seed(&device, options.seed);

    let mut model = TableModel::<B>::new(words.len(), dimensions, &device);
    let mut optimizer = AdamConfig::new().with_epsilon(1e-8).init();
    let mut rng = StdRng::seed_from_u64(options.seed);
    let mut order: Vec<u32> = (0..rows.len() as u32).collect();

    let mut ranker = None;

    for epoch in 1..=options.epochs {
        order.shuffle(&mut rng);

        let started = Instant::now();
        let mut window_loss = 0.0;
        let mut window_batches = 0;

        for (batch_no, indices) in order.chunks(options.batch_size).enumerate() {
            let (context, candidates) = rows.batch::<B>(indices, &device);
            let logits = model.logits(context, candidates);
            let [batch, _] = logits.dims();

            let loss = log_softmax(logits, 1).slice([0..batch, 0..1]).mean().neg();

            window_loss += loss.clone().into_scalar().to_f64();
            window_batches += 1;

            let grads = GradientsParams::from_grads(loss.backward(), &model);
            model = optimizer.step(options.learning_rate, model, grads);

            if (batch_no + 1) % 200 == 0 {
                let done = (batch_no + 1) * options.batch_size;
                println!(
                    "epoch {epoch} examples {done}/{} loss {:.4} rate {:.0}/s",
                    rows.len(),
                    window_loss / window_batches as f64,
                    done as f64 / started.elapsed().as_secs_f64(),
                );
                window_loss = 0.0;
                window_batches = 0;
            }
        }

        let mut epoch_ranker = ContextRanker::from_parts(
            words,
            model.valid().encode().into(),
            ContextRankerConfig {
                dimensions,
                spelling_weight: 0.0,
            },
        );

        let report = evaluate_context_ranker(&epoch_ranker, tuning, None);
        epoch_ranker.set_spelling_weight(report.spelling_weight);
        epoch_ranker.save_to(output);

        println!(
            "epoch {epoch} finished in {:.0}s",
            started.elapsed().as_secs_f64()
        );
        println!("{report}");

        ranker = Some(epoch_ranker);
    }

    ranker.expect("training needs at least one epoch")
}

/// A held-out example with the fit of each candidate.
struct ScoredExample<'a> {
    example: &'a RankerExample,
    fits: Option<Vec<f32>>,
}

fn score_examples<'a>(
    ranker: &ContextRanker,
    examples: &'a [RankerExample],
) -> Vec<ScoredExample<'a>> {
    examples
        .iter()
        .map(|example| ScoredExample {
            example,
            fits: example.has_known_context().then(|| {
                ranker.fits(
                    &example.context(),
                    example.candidates.iter().map(|(id, _)| *id),
                )
            }),
        })
        .collect()
}

#[derive(Clone, Copy, Debug)]
enum Method {
    Current,
    Frequency,
    Table { spelling: f32 },
}

fn frequency_rank(id: u16) -> u32 {
    if id as usize == UNK_IDX {
        u32::MAX
    } else {
        id as u32
    }
}

fn order_for(scored: &ScoredExample, method: Method) -> Vec<usize> {
    let candidates = &scored.example.candidates;
    let mut order: Vec<usize> = (0..candidates.len()).collect();

    match (method, &scored.fits) {
        (Method::Frequency, _) => {
            order.sort_by_key(|i| (candidates[*i].1, frequency_rank(candidates[*i].0)));
        }
        (Method::Table { spelling }, Some(fits)) => {
            let scores: Vec<f32> = fits
                .iter()
                .zip(candidates)
                .map(|(fit, (_, score))| combined_score(*fit, *score as i32, spelling))
                .collect();
            order.sort_by(|a, b| scores[*b].total_cmp(&scores[*a]));
        }
        _ => {}
    }

    order
}

#[derive(Clone, Copy, Debug, Default)]
struct Accuracy {
    total: usize,
    first: usize,
    in_three: usize,
}

impl Accuracy {
    fn add(&mut self, order: &[usize], target: usize) {
        self.total += 1;
        self.first += usize::from(order.first() == Some(&target));
        self.in_three += usize::from(order.iter().take(3).any(|i| *i == target));
    }

    fn first_pct(&self) -> f64 {
        100.0 * self.first as f64 / self.total.max(1) as f64
    }

    fn in_three_pct(&self) -> f64 {
        100.0 * self.in_three as f64 / self.total.max(1) as f64
    }
}

/// Accuracy over all examples, then one entry per group.
fn measure(scored: &[ScoredExample], method: Method) -> Vec<Accuracy> {
    let mut results = vec![Accuracy::default(); GROUPS.len() + 1];

    for item in scored {
        let order = order_for(item, method);
        let target = item.example.target as usize;
        results[0].add(&order, target);
        results[item.example.group as usize + 1].add(&order, target);
    }

    results
}

fn best_spelling_weight(scored: &[ScoredExample]) -> f32 {
    const SPELLING: [f32; 10] = [0.0, 0.05, 0.1, 0.15, 0.2, 0.3, 0.5, 0.75, 1.0, 1.5];

    SPELLING
        .into_iter()
        .map(|spelling| {
            (
                spelling,
                measure(scored, Method::Table { spelling })[0].first,
            )
        })
        .fold(
            (0.0, 0),
            |best, next| if next.1 > best.1 { next } else { best },
        )
        .0
}

fn report_table(title: &str, scored: &[ScoredExample], methods: &[(&str, Method)]) -> String {
    let mut out = format!("{title}\n");
    let results: Vec<_> = methods
        .iter()
        .map(|(name, method)| (*name, measure(scored, *method)))
        .collect();

    for row in 0..=GROUPS.len() {
        let label = if row == 0 {
            "All"
        } else {
            GROUPS[row - 1].name
        };
        out += &format!("  {label} (n={})\n", results[0].1[row].total);
        for (name, accuracy) in &results {
            out += &format!(
                "    {name:<12} first {:6.2}%  in top 3 {:6.2}%\n",
                accuracy[row].first_pct(),
                accuracy[row].in_three_pct(),
            );
        }
    }

    out
}

/// Tune the spelling weight on `tuning`, report it, and also report `test` if it is present.
pub fn evaluate_context_ranker(
    ranker: &ContextRanker,
    tuning: &[RankerExample],
    test: Option<&[RankerExample]>,
) -> EvaluationReport {
    let tuning_scored = score_examples(ranker, tuning);
    let spelling_weight = best_spelling_weight(&tuning_scored);

    let methods = [
        ("current", Method::Current),
        ("frequency", Method::Frequency),
        (
            "table",
            Method::Table {
                spelling: spelling_weight,
            },
        ),
    ];

    let mut text = format!("best spelling_weight {spelling_weight}\n");
    text += &report_table("Tuning split", &tuning_scored, &methods);

    if let Some(test) = test {
        text += &report_table("Test split", &score_examples(ranker, test), &methods);
    }

    EvaluationReport {
        spelling_weight,
        text,
    }
}

pub struct EvaluationReport {
    pub spelling_weight: f32,
    pub text: String,
}

impl std::fmt::Display for EvaluationReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.text)
    }
}

#[cfg(test)]
mod tests {
    use super::{CONTEXT_RADIUS, RankerExample};

    #[test]
    fn example_round_trips() {
        let example = RankerExample {
            group: 3,
            sentence_hash: 0xDEAD_BEEF,
            left: [7; CONTEXT_RADIUS],
            right: [9; CONTEXT_RADIUS],
            target: 1,
            candidates: vec![(11, -4), (12, 6)],
        };

        let mut bytes = Vec::new();
        example.write_to(&mut bytes).unwrap();

        let mut reader = bytes.as_slice();
        assert_eq!(
            RankerExample::read_from(&mut reader).unwrap(),
            Some(example)
        );
        assert_eq!(RankerExample::read_from(&mut reader).unwrap(), None);
    }

    #[test]
    fn context_keeps_the_left_side_first() {
        let mut left = [0; CONTEXT_RADIUS];
        let mut right = [0; CONTEXT_RADIUS];
        left[CONTEXT_RADIUS - 1] = 5;
        right[0] = 6;

        let example = RankerExample {
            group: 0,
            sentence_hash: 0,
            left,
            right,
            target: 0,
            candidates: vec![],
        };

        let context = example.context();
        assert_eq!(context[CONTEXT_RADIUS - 1], 5);
        assert_eq!(context[CONTEXT_RADIUS], 6);
    }
}
