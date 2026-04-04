# SPDX-FileCopyrightText: 2024-2026 PyThaiNLP Project
# SPDX-License-Identifier: Apache-2.0

# Python binding for nlpO3, a natural language processing library.
#
# Authors:
# Thanathip Suntorntip
# Arthit Suriyawongkul

from __future__ import annotations

from typing import List, Optional

from ._nlpo3_python_backend import (
    NewmmFstTokenizer,
    NewmmTokenizer,
)


class DeepcutTokenizer:
    """Deepcut CNN-based Thai word tokenizer.

    Requires the ``nlpo3-deepcut`` package, which ships the ONNX model and
    the ONNX runtime.  Install it with::

        pip install nlpo3-deepcut

    Once installed, usage is identical to the bundled tokenizers:

    .. code-block:: python

        from nlpo3 import DeepcutTokenizer

        tok = DeepcutTokenizer()
        tokens = tok.segment("สวัสดีครับ")

        # Use a custom ONNX model file
        tok = DeepcutTokenizer(model_path="/path/to/custom.onnx")

    This class is a forwarding shim: construction delegates to
    ``nlpo3_deepcut.DeepcutTokenizer`` when that package is present, and
    raises :class:`ImportError` with an installation hint when it is absent.
    """

    def __new__(  # type: ignore[misc]
        cls,
        model_path: Optional[str] = None,
    ) -> "DeepcutTokenizer":
        try:
            from nlpo3_deepcut import (  # type: ignore[import-not-found]
                DeepcutTokenizer as _RealDeepcutTokenizer,
            )
        except ImportError:
            raise ImportError(
                "DeepcutTokenizer requires the nlpo3-deepcut package.\n"
                "Install it with:  pip install nlpo3-deepcut"
            ) from None
        return _RealDeepcutTokenizer(  # type: ignore[return-value]
            model_path=model_path
        )

    def segment(
        self,
        text: str,
        parallel_chunk_size: Optional[int] = None,
    ) -> List[str]:
        """Tokenize text using the deepcut CNN model.

        This stub is never called directly; ``__new__`` returns the real
        ``nlpo3_deepcut.DeepcutTokenizer`` instance.

        Args:
            text: Input text to tokenize.
            parallel_chunk_size: Target chunk size in bytes for parallel mode.
                ``None``, ``0``, or low values disable parallel mode.

        Returns:
            List of word tokens.

        Raises:
            RuntimeError: If ONNX inference fails.
        """
        ...  # pragma: no cover


__all__ = ["DeepcutTokenizer", "NewmmFstTokenizer", "NewmmTokenizer"]
