// SPDX-FileCopyrightText: 2024-2026 PyThaiNLP Project
// SPDX-License-Identifier: Apache-2.0

/**
 * Python binding for nlpO3, a natural language processing library.
 *
 * Original Rust implementation: Thanathip Suntorntip
 * Rewrite and extension: PyThaiNLP Project
 */
use std::path::Path;

use nlpo3::tokenizer::deepcut::DeepcutTokenizer;
use nlpo3::tokenizer::newmm::{NewmmFstTokenizer, NewmmTokenizer};
use pyo3::exceptions;
use pyo3::prelude::*;

// ---------------------------------------------------------------------------
// NewmmTokenizer Python class
// ---------------------------------------------------------------------------

/// Dictionary-based maximal-matching Thai word tokenizer.
///
/// Uses the TrieChar backend (fastest lookup).  The instance is read-only
/// after construction and safe to call from multiple threads concurrently.
///
/// To share one loaded dictionary across many concurrent callers, create a
/// single instance and hold a reference to it from each caller — no copying
/// of the dictionary occurs.
///
/// Example:
///
/// ```python
/// from nlpo3 import NewmmTokenizer
///
/// tok = NewmmTokenizer("path/to/dict.txt")
/// tokens = tok.segment("สวัสดีครับ")
///
/// # safe=True avoids long run times on ambiguous input
/// tokens = tok.segment("สวัสดีครับ", safe=True)
///
/// # parallel_chunk_size enables chunk-based parallel processing
/// tokens = tok.segment("สวัสดีครับ", parallel_chunk_size=16_384)
/// ```
#[pyclass(name = "NewmmTokenizer", frozen)]
struct PyNewmmTokenizer {
    inner: NewmmTokenizer,
}

#[pymethods]
impl PyNewmmTokenizer {
    /// Create a tokenizer from a dictionary file.
    ///
    /// Args:
    ///     dict_path: Path to a plain-text dictionary file (one word per line).
    #[new]
    fn new(dict_path: &str) -> PyResult<Self> {
        NewmmTokenizer::new(dict_path)
            .map(|inner| PyNewmmTokenizer { inner })
            .map_err(|e| {
                exceptions::PyRuntimeError::new_err(format!("failed to load dictionary: {}", e))
            })
    }

    /// Tokenize text into words.
    ///
    /// The tokenizer is immutable — the same instance is safe to call from
    /// multiple threads concurrently without any locking.
    ///
    /// This method releases the Python Global Interpreter Lock (GIL) while
    /// running tokenization so other Python threads can continue executing.
    ///
    /// Args:
    ///     text:     Input text to tokenize.
    ///     safe:     Enable safe mode to avoid long run times on inputs with
    ///               many ambiguous word boundaries (default: False).
    ///     parallel_chunk_size: Target chunk size in bytes for parallel mode.
    ///               `None`, `0`, or low values disable parallel mode.
    ///
    /// Note:
    ///     When ``parallel_chunk_size`` is set, text is split into chunks.
    ///     Token sequences near chunk boundaries may differ from full-text
    ///     results. Suitable for classification and embedding tasks; not
    ///     recommended for tasks requiring precise token boundaries.
    ///
    /// Returns:
    ///     List of word tokens.
    ///
    /// Raises:
    ///     RuntimeError: If tokenization fails.
    #[pyo3(signature = (text, safe = false, parallel_chunk_size = None))]
    fn segment(
        &self,
        py: Python<'_>,
        text: &str,
        safe: bool,
        parallel_chunk_size: Option<usize>,
    ) -> PyResult<Vec<String>> {
        if text.is_empty() {
            return Ok(vec![]);
        }
        py.detach(|| {
            self.inner
                .segment_with_options(text, safe, parallel_chunk_size)
        })
        .map_err(|e| exceptions::PyRuntimeError::new_err(format!("segmentation failed: {}", e)))
    }
}

// ---------------------------------------------------------------------------
// NewmmFstTokenizer Python class
// ---------------------------------------------------------------------------

/// Dictionary-based maximal-matching Thai word tokenizer with FST backend.
///
/// Uses the same algorithm as :class:`NewmmTokenizer` but stores the
/// dictionary in a minimized finite-state automaton (FST), reducing memory
/// use by ~49× at the cost of slower per-lookup speed.
///
/// The instance is read-only after construction and safe to call from
/// multiple threads concurrently without any locking.
///
/// Example:
///
/// ```python
/// from nlpo3 import NewmmFstTokenizer
///
/// tok = NewmmFstTokenizer("path/to/dict.txt")
/// tokens = tok.segment("สวัสดีครับ")
/// ```
#[pyclass(name = "NewmmFstTokenizer", frozen)]
struct PyNewmmFstTokenizer {
    inner: NewmmFstTokenizer,
}

#[pymethods]
impl PyNewmmFstTokenizer {
    /// Create a tokenizer from a dictionary file.
    ///
    /// Args:
    ///     dict_path: Path to a plain-text dictionary file (one word per line).
    #[new]
    fn new(dict_path: &str) -> PyResult<Self> {
        NewmmFstTokenizer::new(dict_path)
            .map(|inner| PyNewmmFstTokenizer { inner })
            .map_err(|e| {
                exceptions::PyRuntimeError::new_err(format!("failed to load dictionary: {}", e))
            })
    }

    /// Tokenize text into words.
    ///
    /// The tokenizer is immutable — the same instance is safe to call from
    /// multiple threads concurrently without any locking.
    ///
    /// This method releases the Python Global Interpreter Lock (GIL) while
    /// running tokenization so other Python threads can continue executing.
    ///
    /// Args:
    ///     text:     Input text to tokenize.
    ///     safe:     Enable safe mode (default: False).
    ///     parallel_chunk_size: Target chunk size in bytes for parallel mode.
    ///               `None`, `0`, or low values disable parallel mode.
    ///
    /// Note:
    ///     When ``parallel_chunk_size`` is set, text is split into chunks.
    ///     Token sequences near chunk boundaries may differ from full-text
    ///     results. Suitable for classification and embedding tasks; not
    ///     recommended for tasks requiring precise token boundaries.
    ///
    /// Returns:
    ///     List of word tokens.
    ///
    /// Raises:
    ///     RuntimeError: If tokenization fails.
    #[pyo3(signature = (text, safe = false, parallel_chunk_size = None))]
    fn segment(
        &self,
        py: Python<'_>,
        text: &str,
        safe: bool,
        parallel_chunk_size: Option<usize>,
    ) -> PyResult<Vec<String>> {
        if text.is_empty() {
            return Ok(vec![]);
        }
        py.detach(|| {
            self.inner
                .segment_with_options(text, safe, parallel_chunk_size)
        })
        .map_err(|e| exceptions::PyRuntimeError::new_err(format!("tokenization failed: {}", e)))
    }
}

// ---------------------------------------------------------------------------
// DeepcutTokenizer Python class
// ---------------------------------------------------------------------------

/// Deepcut CNN-based Thai word tokenizer.
///
/// Each instance compiles and owns the ONNX model.  Internally the compiled
/// model is reference-counted (:class:`Arc`), so cloning is O(1).  The same
/// instance is safe to call from multiple threads simultaneously.
///
/// For distributed or parallel workloads, create one instance per worker
/// process to avoid sharing state across process boundaries.
///
/// Example:
///
/// ```python
/// from nlpo3 import DeepcutTokenizer
///
/// # Use the bundled default model
/// tok = DeepcutTokenizer()
/// tokens = tok.segment("สวัสดีครับ")
///
/// # Use a custom ONNX model file
/// tok = DeepcutTokenizer(model_path="/path/to/custom.onnx")
/// tokens = tok.segment("สวัสดีครับ")
/// ```
#[pyclass(name = "DeepcutTokenizer", frozen)]
struct PyDeepcutTokenizer {
    inner: DeepcutTokenizer,
}

#[pymethods]
impl PyDeepcutTokenizer {
    /// Create a new DeepcutTokenizer.
    ///
    /// If ``model_path`` is ``None`` (the default), the bundled ONNX model is
    /// used.  Pass a filesystem path to load a custom compatible model.
    ///
    /// Args:
    ///     model_path: Path to a custom deepcut ONNX model file, or ``None``
    ///                 to use the bundled default model.
    ///
    /// Raises:
    ///     RuntimeError: If the ONNX model cannot be loaded.
    #[new]
    #[pyo3(signature = (model_path = None))]
    fn new(model_path: Option<&str>) -> PyResult<Self> {
        let result = match model_path {
            None => DeepcutTokenizer::new(),
            Some(p) => DeepcutTokenizer::from_path(Path::new(p)),
        };
        result
            .map(|inner| PyDeepcutTokenizer { inner })
            .map_err(|e| {
                exceptions::PyRuntimeError::new_err(format!(
                    "deepcut: failed to load tokenizer: {}",
                    e
                ))
            })
    }

    /// Tokenize text using the deepcut CNN model.
    ///
    /// Inference is thread-safe: the same instance may be called concurrently
    /// from multiple threads.
    ///
    /// This method releases the Python Global Interpreter Lock (GIL) while
    /// running inference so other Python threads can continue executing.
    ///
    /// Args:
    ///     text:     Input text to tokenize.
    ///     parallel_chunk_size: Target chunk size in bytes for parallel mode.
    ///                 `None`, `0`, or low values disable parallel mode.
    ///
    /// Note:
    ///     When ``parallel_chunk_size`` is set, text is split into chunks.
    ///     Because deepcut uses a fixed-width context window, characters near
    ///     chunk boundaries have fewer adjacent context characters, so token
    ///     sequences near boundaries may differ from full-text results.
    ///     Suitable for classification and embedding tasks; not recommended
    ///     for tasks requiring precise token boundaries.
    ///
    /// Returns:
    ///     List of word tokens.
    ///
    /// Raises:
    ///     RuntimeError: If ONNX inference fails.
    #[pyo3(signature = (text, parallel_chunk_size = None))]
    fn segment(
        &self,
        py: Python<'_>,
        text: &str,
        parallel_chunk_size: Option<usize>,
    ) -> PyResult<Vec<String>> {
        if text.is_empty() {
            return Ok(vec![]);
        }

        py.detach(|| self.inner.segment_with_options(text, parallel_chunk_size))
            .map_err(|e| {
                exceptions::PyRuntimeError::new_err(format!("deepcut: inference failed: {}", e))
            })
    }
}

// ---------------------------------------------------------------------------
// Module
// ---------------------------------------------------------------------------

#[pymodule]
fn _nlpo3_python_backend(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyNewmmTokenizer>()?;
    m.add_class::<PyNewmmFstTokenizer>()?;
    m.add_class::<PyDeepcutTokenizer>()?;
    Ok(())
}
