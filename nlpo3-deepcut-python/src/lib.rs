// SPDX-FileCopyrightText: 2024-2026 PyThaiNLP Project
// SPDX-License-Identifier: Apache-2.0

/**
 * Python binding for the nlpO3 Deepcut tokenizer.
 *
 * This crate provides only the DeepcutTokenizer class.  It compiles with the
 * `deepcut` feature of the core `nlpo3` crate, embedding the ONNX model and
 * linking tract-onnx.  Users install this as the optional `nlpo3-deepcut`
 * package alongside the base `nlpo3` package.
 *
 * Original Rust implementation: Thanathip Suntorntip
 * Rewrite and extension: PyThaiNLP Project
 */
use std::path::Path;

use nlpo3::tokenizer::deepcut::DeepcutTokenizer;
use pyo3::exceptions;
use pyo3::prelude::*;

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
/// from nlpo3_deepcut import DeepcutTokenizer
///
/// # Use the bundled default model
/// tok = DeepcutTokenizer()
/// tokens = tok.segment("สวัสดีครับ")
///
/// # Use a custom ONNX model file
/// tok = DeepcutTokenizer(model_path="/path/to/custom.onnx")
/// tokens = tok.segment("สวัสดีครับ")
/// ```
///
/// You can also import from the base package when ``nlpo3-deepcut`` is
/// installed:
///
/// ```python
/// from nlpo3 import DeepcutTokenizer
/// tok = DeepcutTokenizer()
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
fn _nlpo3_deepcut_backend(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyDeepcutTokenizer>()?;
    Ok(())
}
