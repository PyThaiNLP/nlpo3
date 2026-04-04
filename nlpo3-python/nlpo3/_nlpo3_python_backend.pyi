# SPDX-FileCopyrightText: 2026 PyThaiNLP Project
# SPDX-License-Identifier: Apache-2.0

"""Type stubs for _nlpo3_python_backend Rust extension module."""

from __future__ import annotations

from typing import List, Optional

class NewmmTokenizer:
    """Dictionary-based maximal-matching Thai word tokenizer.

    Uses the TrieChar backend (fastest lookup).  The instance is read-only
    after construction and safe to call from multiple threads concurrently.

    To share one loaded dictionary across many concurrent callers, create a
    single instance and hold a reference to it from each caller — no copying
    of the dictionary occurs.

    Example::

        from nlpo3 import NewmmTokenizer

        tok = NewmmTokenizer("path/to/dict.txt")
        tokens = tok.segment("สวัสดีครับ")

        # Reuse the same instance across all concurrent callers
        results = [tok.segment(t) for t in texts]
    """

    def __new__(cls, dict_path: str) -> "NewmmTokenizer":
        """Create a tokenizer from a dictionary file.

        Args:
            dict_path: Path to a plain-text dictionary file (one word per line).

        Raises:
            RuntimeError: If the dictionary file cannot be read or parsed.
        """
        ...

    def segment(
        self,
        text: str,
        safe: bool = False,
        parallel_chunk_size: Optional[int] = None,
    ) -> List[str]:
        """Tokenize text into words.

        The tokenizer is immutable — the same instance is safe to call from
        multiple threads concurrently without any locking.

        Args:
            text:     Input text to tokenize.
            safe:     Enable safe mode to avoid long run times on inputs with
                      many ambiguous word boundaries (default: False).
            parallel_chunk_size: Target chunk size in bytes for chunked parallel
                      processing. `None`, `0`, or low values disable parallel mode.

        Note:
            When ``parallel_chunk_size`` is set, text is split into chunks.
            Token sequences near chunk boundaries may differ from full-text
            results. Suitable for classification and embedding tasks; not
            recommended for tasks requiring precise token boundaries.

        Returns:
            List of word tokens.

        Raises:
            RuntimeError: If tokenization fails.
        """
        ...

class NewmmFstTokenizer:
    """Dictionary-based maximal-matching Thai word tokenizer (FST backend).

    Uses the same algorithm as :class:`NewmmTokenizer` but stores the
    dictionary in a minimized finite-state automaton, reducing memory use by
    ~49× at the cost of slower per-lookup speed.

    The instance is read-only after construction and safe to call from
    multiple threads concurrently without any locking.

    Example::

        from nlpo3 import NewmmFstTokenizer

        tok = NewmmFstTokenizer("path/to/dict.txt")
        tokens = tok.segment("สวัสดีครับ")
    """

    def __new__(cls, dict_path: str) -> "NewmmFstTokenizer":
        """Create a tokenizer from a dictionary file.

        Args:
            dict_path: Path to a plain-text dictionary file (one word per line).

        Raises:
            RuntimeError: If the dictionary file cannot be read or parsed.
        """
        ...

    def segment(
        self,
        text: str,
        safe: bool = False,
        parallel_chunk_size: Optional[int] = None,
    ) -> List[str]:
        """Tokenize text into words.

        The tokenizer is immutable — the same instance is safe to call from
        multiple threads concurrently without any locking.

        Args:
            text:     Input text to tokenize.
            safe:     Enable safe mode (default: False).
            parallel_chunk_size: Target chunk size in bytes for chunked parallel
                      processing. `None`, `0`, or low values disable parallel mode.

        Note:
            When ``parallel_chunk_size`` is set, text is split into chunks.
            Token sequences near chunk boundaries may differ from full-text
            results. Suitable for classification and embedding tasks; not
            recommended for tasks requiring precise token boundaries.

        Returns:
            List of word tokens.

        Raises:
            RuntimeError: If tokenization fails.
        """
        ...

class DeepcutTokenizer:
    """Deepcut CNN-based Thai word tokenizer.

    Each instance compiles and owns the ONNX model.  Internally the compiled
    model is reference-counted (Arc), so cloning is O(1).  The same instance
    is safe to call from multiple threads simultaneously.

    For distributed or parallel workloads, create one instance per worker
    process to avoid sharing state across process boundaries.

    Example::

        from nlpo3 import DeepcutTokenizer

        # Bundled model
        tok = DeepcutTokenizer()
        tokens = tok.segment("สวัสดีครับ")

        # Custom ONNX model
        tok = DeepcutTokenizer(model_path="/path/to/custom.onnx")
    """

    def __new__(cls, model_path: Optional[str] = None) -> "DeepcutTokenizer":
        """Create a DeepcutTokenizer.

        Args:
            model_path: Path to a custom deepcut ONNX model file, or ``None``
                        to use the bundled default model.

        Raises:
            RuntimeError: If the ONNX model cannot be loaded.
        """
        ...

    def segment(
        self, text: str, parallel_chunk_size: Optional[int] = None
    ) -> List[str]:
        """Tokenize text using the deepcut CNN model.

        Inference is thread-safe: the same instance may be called concurrently
        from multiple threads.

        Args:
            text: Input text to tokenize.
            parallel_chunk_size: Target chunk size in bytes for chunked parallel
                        processing. `None`, `0`, or low values disable parallel mode.

        Note:
            When ``parallel_chunk_size`` is set, text is split into chunks.
            Because deepcut uses a fixed-width context window, characters near
            chunk boundaries have fewer adjacent context characters, so token
            sequences near boundaries may differ from full-text results.
            Suitable for classification and embedding tasks; not recommended
            for tasks requiring precise token boundaries.

        Returns:
            List of word tokens.

        Raises:
            RuntimeError: If ONNX inference fails.
        """
        ...
