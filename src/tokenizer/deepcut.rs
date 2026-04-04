// SPDX-FileCopyrightText: 2024-2026 PyThaiNLP Project
// SPDX-License-Identifier: Apache-2.0

//! Deepcut Thai word tokenizer using ONNX inference via the pure-Rust
//! `tract-onnx` engine.
//!
//! Based on LEKCut <https://github.com/PyThaiNLP/LEKCut>
//! and Deepcut <https://github.com/rkcosmos/deepcut>.
//!
//! Original deepcut authors:
//! Rakpong Kittinaradorn, Titipat Achakulvisut, Korakot Chaovavanich,
//! Kittinan Srithaworn, Pattarawat Chormai, Chanwit Kaewkasi,
//! Tulakan Ruangrong, Krichkorn Oparad.
//!
//! Original deepcut license: MIT License.

use std::collections::HashMap;
use std::io::Cursor;
use std::sync::Arc;

use anyhow::Result as AnyResult;
use lazy_static::lazy_static;
use ndarray::Array2;
use tract_onnx::prelude::*;
// `Factoid` is a trait that provides the `concretize()` method used in
// `load_model_from_bytes()` to extract the batch `TDim` from the inference
// model's input fact.
use tract_onnx::tract_hir::infer::Factoid;

use super::parallel_helper;
use super::parallel_options::ParallelOptions;
use super::tcc::tcc_tokenizer;
use super::tokenizer_trait::Tokenizer;

/// Bundled deepcut ONNX model weights.
const MODEL_BYTES: &[u8] = include_bytes!("../../model/deepcut.onnx");

/// Context window width (forward context + current + backward context = 21).
const N_PAD: usize = 21;
/// Half-width of the context window: (21 - 1) / 2 = 10.
const N_PAD_2: usize = (N_PAD - 1) / 2;
/// Character-vocabulary index used for characters not in the vocabulary.
const CHAR_INDEX_OTHER: u32 = 80;
/// Character-type index used for types not in the type map.
const CHAR_TYPE_INDEX_OTHER: u32 = 4;
/// Probability threshold for classifying a position as a word boundary.
const WORD_BOUNDARY_THRESHOLD: f32 = 0.5;

type DeepcutModel = RunnableModel<TypedFact, Box<dyn TypedOp>, Graph<TypedFact, Box<dyn TypedOp>>>;

// ---------------------------------------------------------------------------
// Vocabulary tables (constant data; shared across all instances)
// ---------------------------------------------------------------------------

fn build_chars_map() -> HashMap<char, u32> {
    // Maps each character to its vocabulary index.
    // Index 80 is the "other" sentinel (represented by the string "other",
    // not a single character) so it is absent from this map.
    let entries: &[(&str, u32)] = &[
        ("\n", 0),
        (" ", 1),
        ("!", 2),
        ("\"", 3),
        ("#", 4),
        ("$", 5),
        ("%", 6),
        ("&", 7),
        ("'", 8),
        ("(", 9),
        (")", 10),
        ("*", 11),
        ("+", 12),
        (",", 13),
        ("-", 14),
        (".", 15),
        ("/", 16),
        ("0", 17),
        ("1", 18),
        ("2", 19),
        ("3", 20),
        ("4", 21),
        ("5", 22),
        ("6", 23),
        ("7", 24),
        ("8", 25),
        ("9", 26),
        (":", 27),
        (";", 28),
        ("<", 29),
        ("=", 30),
        (">", 31),
        ("?", 32),
        ("@", 33),
        ("A", 34),
        ("B", 35),
        ("C", 36),
        ("D", 37),
        ("E", 38),
        ("F", 39),
        ("G", 40),
        ("H", 41),
        ("I", 42),
        ("J", 43),
        ("K", 44),
        ("L", 45),
        ("M", 46),
        ("N", 47),
        ("O", 48),
        ("P", 49),
        ("Q", 50),
        ("R", 51),
        ("S", 52),
        ("T", 53),
        ("U", 54),
        ("V", 55),
        ("W", 56),
        ("X", 57),
        ("Y", 58),
        ("Z", 59),
        ("[", 60),
        ("\\", 61),
        ("]", 62),
        ("^", 63),
        ("_", 64),
        ("a", 65),
        ("b", 66),
        ("c", 67),
        ("d", 68),
        ("e", 69),
        ("f", 70),
        ("g", 71),
        ("h", 72),
        ("i", 73),
        ("j", 74),
        ("k", 75),
        ("l", 76),
        ("m", 77),
        ("n", 78),
        ("o", 79),
        // 80 = "other" sentinel — not a single char, absent from this map
        ("p", 81),
        ("q", 82),
        ("r", 83),
        ("s", 84),
        ("t", 85),
        ("u", 86),
        ("v", 87),
        ("w", 88),
        ("x", 89),
        ("y", 90),
        ("z", 91),
        ("}", 92),
        ("~", 93),
        ("ก", 94),
        ("ข", 95),
        ("ฃ", 96),
        ("ค", 97),
        ("ฅ", 98),
        ("ฆ", 99),
        ("ง", 100),
        ("จ", 101),
        ("ฉ", 102),
        ("ช", 103),
        ("ซ", 104),
        ("ฌ", 105),
        ("ญ", 106),
        ("ฎ", 107),
        ("ฏ", 108),
        ("ฐ", 109),
        ("ฑ", 110),
        ("ฒ", 111),
        ("ณ", 112),
        ("ด", 113),
        ("ต", 114),
        ("ถ", 115),
        ("ท", 116),
        ("ธ", 117),
        ("น", 118),
        ("บ", 119),
        ("ป", 120),
        ("ผ", 121),
        ("ฝ", 122),
        ("พ", 123),
        ("ฟ", 124),
        ("ภ", 125),
        ("ม", 126),
        ("ย", 127),
        ("ร", 128),
        ("ฤ", 129),
        ("ล", 130),
        ("ว", 131),
        ("ศ", 132),
        ("ษ", 133),
        ("ส", 134),
        ("ห", 135),
        ("ฬ", 136),
        ("อ", 137),
        ("ฮ", 138),
        ("ฯ", 139),
        ("ะ", 140),
        ("\u{0E31}", 141), // ั  sara a above (U+0E31)
        ("า", 142),
        ("ำ", 143),
        ("ิ", 144),
        ("ี", 145),
        ("ึ", 146),
        ("ื", 147),
        ("ุ", 148),
        ("ู", 149),
        ("ฺ", 150),
        ("เ", 151),
        ("แ", 152),
        ("โ", 153),
        ("ใ", 154),
        ("ไ", 155),
        ("ๅ", 156),
        ("ๆ", 157),
        ("็", 158),
        ("่", 159),
        ("้", 160),
        ("๊", 161),
        ("๋", 162),
        ("์", 163),
        ("ํ", 164),
        ("๐", 165),
        ("๑", 166),
        ("๒", 167),
        ("๓", 168),
        ("๔", 169),
        ("๕", 170),
        ("๖", 171),
        ("๗", 172),
        ("๘", 173),
        ("๙", 174),
        ("\u{2018}", 175), // ' LEFT SINGLE QUOTATION MARK
        ("\u{2019}", 176), // ' RIGHT SINGLE QUOTATION MARK
        ("\u{FEFF}", 177), // BOM
    ];
    entries
        .iter()
        .filter_map(|&(s, idx)| {
            let mut chars = s.chars();
            let ch = chars.next()?;
            // Keep only single-character keys.
            if chars.next().is_none() {
                Some((ch, idx))
            } else {
                None
            }
        })
        .collect()
}

fn build_char_type_map() -> HashMap<char, u32> {
    // Maps each character to its type index.
    // Type indices: b_e=0, c=1, d=2, n=3, o=4(fallback), p=5, q=6,
    //               s=7, s_e=8, t=9, v=10, w=11
    let type_groups: &[(&str, u32)] = &[
        // c (1) — common Thai consonants
        // กขฃคฆงจชซญฎฏฐฑฒณดตถทธนบปพฟภมยรลวศษสฬอ
        (
            "\u{0E01}\u{0E02}\u{0E03}\u{0E04}\u{0E06}\u{0E07}\u{0E08}\
             \u{0E0A}\u{0E0B}\u{0E0D}\u{0E0E}\u{0E0F}\u{0E10}\u{0E11}\
             \u{0E12}\u{0E13}\u{0E14}\u{0E15}\u{0E16}\u{0E17}\u{0E18}\
             \u{0E19}\u{0E1A}\u{0E1B}\u{0E1E}\u{0E1F}\u{0E20}\u{0E21}\
             \u{0E22}\u{0E23}\u{0E25}\u{0E27}\u{0E28}\u{0E29}\u{0E2A}\
             \u{0E2C}\u{0E2D}",
            1,
        ),
        // n (3) — rare Thai consonants: ฅฉผฟฌหฮ
        (
            "\u{0E05}\u{0E09}\u{0E1C}\u{0E1F}\u{0E0C}\u{0E2B}\u{0E2E}",
            3,
        ),
        // v (10) — Thai vowels: ะาำิีืึุู
        (
            "\u{0E30}\u{0E32}\u{0E33}\u{0E34}\u{0E35}\u{0E37}\u{0E36}\
             \u{0E38}\u{0E39}",
            10,
        ),
        // w (11) — leading Thai vowels: เแโใไ
        ("\u{0E40}\u{0E41}\u{0E42}\u{0E43}\u{0E44}", 11),
        // t (9) — Thai tone marks: ่้๊๋
        ("\u{0E48}\u{0E49}\u{0E4A}\u{0E4B}", 9),
        // s (7) — Thai symbols and period: ์ๆฯ.
        ("\u{0E4C}\u{0E46}\u{0E2F}.", 7),
        // d (2) — digits: 0-9 and Thai digits ๑๒๓๔๕๖๗๘๙
        (
            "0123456789\
             \u{0E51}\u{0E52}\u{0E53}\u{0E54}\u{0E55}\u{0E56}\u{0E57}\
             \u{0E58}\u{0E59}",
            2,
        ),
        // q (6) — quotation marks: " ' \u{2018} \u{2019}
        ("\"\u{2018}\u{2019}'", 6),
        // p (5) — space
        (" ", 5),
        // s_e (8) — lowercase Latin
        ("abcdefghijklmnopqrstuvwxyz", 8),
        // b_e (0) — uppercase Latin
        ("ABCDEFGHIJKLMNOPQRSTUVWXYZ", 0),
    ];
    let mut map = HashMap::new();
    for &(group, type_idx) in type_groups {
        for ch in group.chars() {
            map.insert(ch, type_idx);
        }
    }
    map
}

// Vocabulary maps are constant and shared across all tokenizer instances.
lazy_static! {
    static ref CHARS_MAP: HashMap<char, u32> = build_chars_map();
    static ref CHAR_TYPE_MAP: HashMap<char, u32> = build_char_type_map();
}

// ---------------------------------------------------------------------------
// Model loading
// ---------------------------------------------------------------------------

/// Load and compile a deepcut ONNX model from raw bytes.
///
/// The two ONNX model inputs carry different symbolic batch dimensions.
/// We unify them with the same `TDim` so that the internal concatenation
/// constraint in the model is satisfied.
fn load_model_from_bytes(bytes: &[u8]) -> AnyResult<DeepcutModel> {
    let inf_model = tract_onnx::onnx()
        .model_for_read(&mut Cursor::new(bytes))
        .map_err(|e| anyhow::anyhow!("deepcut: failed to read ONNX model: {}", e))?;

    let dims0: Vec<_> = inf_model
        .input_fact(0)
        .map_err(|e| anyhow::anyhow!("deepcut: failed to read input fact 0: {}", e))?
        .shape
        .dims()
        .collect();
    // The two inputs carry different symbolic batch dimensions.  Unify them by
    // assigning the same TDim to both so tract can satisfy the internal shape
    // constraint.  If the factoid is `Any` (shape fully unknown), use a
    // dtype-only fact and let tract infer both inputs freely.
    let shared_fact = if let Some(batch_tdim) = dims0.first().and_then(|d| d.concretize()) {
        InferenceFact::dt_shape(f32::datum_type(), tvec![batch_tdim, N_PAD.into()])
    } else {
        InferenceFact::dt(f32::datum_type())
    };

    inf_model
        .with_input_fact(0, shared_fact.clone())
        .map_err(|e| anyhow::anyhow!("deepcut: failed to set input fact 0: {}", e))?
        .with_input_fact(1, shared_fact)
        .map_err(|e| anyhow::anyhow!("deepcut: failed to set input fact 1: {}", e))?
        .into_optimized()
        .map_err(|e| anyhow::anyhow!("deepcut: model optimization failed: {}", e))?
        .into_runnable()
        .map_err(|e| anyhow::anyhow!("deepcut: model compilation failed: {}", e))
}

/// Load and compile the bundled deepcut ONNX model.
fn load_model() -> AnyResult<DeepcutModel> {
    load_model_from_bytes(MODEL_BYTES)
}

// ---------------------------------------------------------------------------
// Feature extraction
// ---------------------------------------------------------------------------

/// Build the character-index and character-type feature arrays for `text`.
///
/// For each position `i` the 21-element feature vector is:
///  - positions i+1 … i+10  (forward context, 10 chars)
///  - positions i-10 … i-1  (backward context, reversed, 10 chars)
///  - position i             (current character, 1 char)
fn build_features(text_chars: &[char]) -> (Vec<f32>, Vec<f32>) {
    let n = text_chars.len();

    // Pad with spaces on both sides.
    let mut padded: Vec<char> = Vec::with_capacity(n + 2 * N_PAD_2);
    padded.extend(std::iter::repeat_n(' ', N_PAD_2));
    padded.extend_from_slice(text_chars);
    padded.extend(std::iter::repeat_n(' ', N_PAD_2));

    let mut x_char = Vec::with_capacity(n * N_PAD);
    let mut x_type = Vec::with_capacity(n * N_PAD);

    for i in N_PAD_2..(N_PAD_2 + n) {
        // Forward context: positions i+1 … i+N_PAD_2
        for &ch in &padded[i + 1..i + N_PAD_2 + 1] {
            x_char.push(*CHARS_MAP.get(&ch).unwrap_or(&CHAR_INDEX_OTHER) as f32);
            x_type.push(*CHAR_TYPE_MAP.get(&ch).unwrap_or(&CHAR_TYPE_INDEX_OTHER) as f32);
        }
        // Backward context (reversed): positions i-1 … i-N_PAD_2
        for &ch in padded[i - N_PAD_2..i].iter().rev() {
            x_char.push(*CHARS_MAP.get(&ch).unwrap_or(&CHAR_INDEX_OTHER) as f32);
            x_type.push(*CHAR_TYPE_MAP.get(&ch).unwrap_or(&CHAR_TYPE_INDEX_OTHER) as f32);
        }
        // Current character
        x_char.push(*CHARS_MAP.get(&padded[i]).unwrap_or(&CHAR_INDEX_OTHER) as f32);
        x_type.push(
            *CHAR_TYPE_MAP
                .get(&padded[i])
                .unwrap_or(&CHAR_TYPE_INDEX_OTHER) as f32,
        );
    }

    (x_char, x_type)
}

// ---------------------------------------------------------------------------
// Public tokenizer
// ---------------------------------------------------------------------------

/// Deepcut CNN-based Thai word tokenizer.
///
/// Each instance compiles and owns the ONNX model.  Cloning is cheap because
/// the compiled model is reference-counted with [`Arc`].  Both `Send` and
/// `Sync` are satisfied, so the same instance may be shared across threads.
///
/// For distributed or parallel workloads, create one `DeepcutTokenizer` per
/// worker process; do not rely on any process-level singleton.
///
/// # Example
///
/// ```rust
/// use nlpo3::tokenizer::deepcut::DeepcutTokenizer;
/// use nlpo3::tokenizer::tokenizer_trait::Tokenizer;
///
/// let tokenizer = DeepcutTokenizer::new().unwrap();
/// let tokens = tokenizer
///     .segment("ทดสอบการตัดคำ")
///     .expect("segmentation failed");
/// assert!(!tokens.is_empty());
/// assert_eq!(tokens.join(""), "ทดสอบการตัดคำ");
/// ```
#[derive(Clone)]
pub struct DeepcutTokenizer {
    /// Reference-counted compiled model.  Cloning the tokenizer is O(1).
    model: Arc<DeepcutModel>,
}

impl DeepcutTokenizer {
    /// Create a `DeepcutTokenizer` using the bundled ONNX model.
    ///
    /// Compiles the model on each call.  For repeated use within the same
    /// process, prefer to create a single instance and reuse it (or clone
    /// it cheaply with [`Clone`]).
    pub fn new() -> AnyResult<Self> {
        Ok(DeepcutTokenizer {
            model: Arc::new(load_model()?),
        })
    }

    /// Create a `DeepcutTokenizer` from a custom ONNX model file on disk.
    ///
    /// The model must be compatible with the deepcut input/output schema
    /// (two `float32` inputs of shape `[n, 21]`).
    pub fn from_path(model_path: &std::path::Path) -> AnyResult<Self> {
        let bytes = std::fs::read(model_path).map_err(|e| {
            anyhow::anyhow!("deepcut: failed to read model file {:?}: {}", model_path, e)
        })?;
        Ok(DeepcutTokenizer {
            model: Arc::new(load_model_from_bytes(&bytes)?),
        })
    }

    /// Tokenize `text`, returning word tokens.
    ///
    /// Inference is thread-safe: `tract` creates independent computation state
    /// per call, so multiple threads may call this concurrently on the same
    /// instance.
    pub fn tokenize(&self, text: &str) -> AnyResult<Vec<String>> {
        if text.is_empty() {
            return Ok(vec![]);
        }

        let text_chars: Vec<char> = text.chars().collect();
        let n = text_chars.len();

        let (x_char_flat, x_type_flat) = build_features(&text_chars);

        let x_char = Array2::<f32>::from_shape_vec((n, N_PAD), x_char_flat)
            .map_err(|e| anyhow::anyhow!("deepcut: failed to build x_char array: {}", e))?;
        let x_type = Array2::<f32>::from_shape_vec((n, N_PAD), x_type_flat)
            .map_err(|e| anyhow::anyhow!("deepcut: failed to build x_type array: {}", e))?;

        let outputs = self
            .model
            .run(tvec![
                x_char.into_tensor().into(),
                x_type.into_tensor().into(),
            ])
            .map_err(|e| anyhow::anyhow!("deepcut: ONNX inference failed: {}", e))?;

        let probs: Vec<f32> = outputs[0]
            .to_array_view::<f32>()
            .map_err(|e| anyhow::anyhow!("deepcut: failed to read model output: {}", e))?
            .iter()
            .copied()
            .collect();

        // Mirror the Python segmentation logic:
        //   y_predict = (probs > threshold).astype(int)
        //   word_end  = y_predict[1:].tolist() + [1]
        let word_end: Vec<bool> = probs[1..]
            .iter()
            .map(|&p| p > WORD_BOUNDARY_THRESHOLD)
            .chain(std::iter::once(true))
            .collect();

        let mut tokens: Vec<String> = Vec::new();
        let mut current = String::new();
        for (&ch, &is_end) in text_chars.iter().zip(word_end.iter()) {
            current.push(ch);
            if is_end {
                tokens.push(std::mem::take(&mut current));
            }
        }
        if !current.is_empty() {
            tokens.push(current);
        }

        Ok(tokens)
    }
}

impl Default for DeepcutTokenizer {
    /// Creates a tokenizer using the bundled model.
    ///
    /// # Panics
    /// Panics if the bundled ONNX model cannot be initialized.
    fn default() -> Self {
        DeepcutTokenizer::new().expect("deepcut: ONNX model initialization failed")
    }
}

impl DeepcutTokenizer {
    /// Segment text with parallel mode disabled.
    pub fn segment(&self, text: &str) -> AnyResult<Vec<String>> {
        self.segment_with_options(text, None)
    }

    /// Segment text with automatically tuned parallel chunking.
    ///
    /// This computes a chunk size from input length and runtime CPU
    /// parallelism, then forwards it to `segment_with_options()`.
    pub fn segment_parallel(&self, text: &str) -> AnyResult<Vec<String>> {
        let options = ParallelOptions::auto_for_text(text.len());
        self.segment_with_options(
            text,
            if options.enabled {
                Some(options.chunk_size)
            } else {
                None
            },
        )
    }

    /// Segment text with explicit parallel chunk configuration.
    ///
    /// - `None`, `0`, or values below `MIN_CHUNK_SIZE` disable parallel mode.
    /// - Valid values enable parallel chunk processing for long inputs.
    ///
    /// # Chunk boundary behavior
    ///
    /// When parallel mode is active, text is split into chunks before
    /// tokenization. Because deepcut uses a fixed-width context window, characters
    /// near chunk boundaries have fewer adjacent context characters from the
    /// neighboring chunk. This causes token sequences near boundaries to differ
    /// from full-text tokenization. The effect scales with chunk size: smaller
    /// chunks introduce more boundaries and thus more divergence.
    ///
    /// Chunked output is acceptable for throughput-oriented tasks such as text
    /// classification and word embedding. It may not be suitable for tasks that
    /// require precise linguistic unit identification.
    pub fn segment_with_options(
        &self,
        text: &str,
        parallel_chunk_size: Option<usize>,
    ) -> AnyResult<Vec<String>> {
        if text.is_empty() {
            return Ok(vec![]);
        }

        let options = ParallelOptions::from_chunk_size(parallel_chunk_size);
        let should_parallelize = options.should_parallelize(text.len());
        if !should_parallelize {
            return self.tokenize(text);
        }

        let tcc_positions = tcc_tokenizer::tcc_pos(text);
        let chunks =
            parallel_helper::split_text_into_chunks(text, options.chunk_size, &tcc_positions);
        let token_vecs = parallel_helper::tokenize_chunks(chunks, should_parallelize, |chunk| {
            self.tokenize(chunk)
        })?;

        Ok(parallel_helper::flatten_tokens(token_vecs))
    }
}

impl Tokenizer for DeepcutTokenizer {
    /// Tokenize `text` using the deepcut ONNX model.
    fn segment(&self, text: &str) -> AnyResult<Vec<String>> {
        self.segment_with_options(text, None)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn tok() -> DeepcutTokenizer {
        DeepcutTokenizer::new().expect("model should load")
    }

    #[test]
    fn test_empty_input() {
        assert_eq!(tok().tokenize("").unwrap(), Vec::<String>::new());
    }

    #[test]
    fn test_basic_thai_roundtrip() {
        let text = "ทดสอบการตัดคำ";
        let tokens = tok().tokenize(text).unwrap();
        assert!(!tokens.is_empty());
        assert_eq!(tokens.join(""), text);
    }

    #[test]
    fn test_mixed_thai_latin_roundtrip() {
        let text = "หมอนทองตากลมหูว์MBK39";
        let tokens = tok().tokenize(text).unwrap();
        assert_eq!(tokens.join(""), text);
    }

    #[test]
    fn test_tokenizer_trait() {
        let tok = DeepcutTokenizer::new().expect("model should load");
        let tokens = tok.segment("ทดสอบ").unwrap();
        assert!(!tokens.is_empty());
        assert_eq!(tokens.join(""), "ทดสอบ");
    }

    #[test]
    fn test_clone_shares_model() {
        let tok1 = tok();
        let tok2 = tok1.clone();
        // Both should tokenize identically.
        assert_eq!(
            tok1.tokenize("ทดสอบ").unwrap(),
            tok2.tokenize("ทดสอบ").unwrap()
        );
    }
}
