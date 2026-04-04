# SPDX-FileCopyrightText: 2026 PyThaiNLP Project
# SPDX-License-Identifier: Apache-2.0

"""Type stubs for _nlpo3_deepcut_backend Rust extension module."""

from __future__ import annotations

from typing import List, Optional

class DeepcutTokenizer:
    """Deepcut CNN-based Thai word tokenizer.

    Each instance compiles and owns the ONNX model.  Internally the compiled
    model is reference-counted (Arc), so cloning is O(1).  The same instance
    is safe to call from multiple threads simultaneously.

    For distributed or parallel workloads, create one instance per worker
    process to avoid sharing state across process boundaries.

    Example::

        from nlpo3_deepcut import DeepcutTokenizer

        # Bundled model
        tok = DeepcutTokenizer()
        tokens = tok.segment("สวัสดีครับ")

        # Custom ONNX model
        tok = DeepcutTokenizer(model_path="/path/to/custom.onnx")

    You can also import from the base package when ``nlpo3-deepcut`` is
    installed::

        from nlpo3 import DeepcutTokenizer
        tok = DeepcutTokenizer()
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
