# SPDX-FileCopyrightText: 2026 PyThaiNLP Project
# SPDX-License-Identifier: Apache-2.0

"""Type stubs for _nlpo3_python_backend Rust extension module."""

from typing import List, Tuple

def load_dict(file_path: str, dict_name: str) -> Tuple[str, bool]:
    """Load a dictionary file to a tokenizer.

    Load a dictionary file into an in-memory dictionary collection,
    and assign dict_name to it. This function does not override an
    existing dict name.

    Args:
        file_path: Path to a dictionary file (one word per line)
        dict_name: A unique dictionary name, used for reference

    Returns:
        A tuple of (human_readable_result_str, success_bool)
    """
    ...

def segment(
    text: str,
    dict_name: str,
    safe: bool = False,
    parallel: bool = False,
) -> List[str]:
    """Break text into tokens using newmm algorithm.

    Args:
        text: Input text to segment
        dict_name: Dictionary name, as assigned in load_dict()
        safe: Use safe mode to avoid long waiting time in a text with
              lots of ambiguous word boundaries (default: False)
        parallel: Use multithread mode (default: False)

    Returns:
        List of tokens

    Raises:
        RuntimeError: If dictionary name does not exist
    """
    ...
