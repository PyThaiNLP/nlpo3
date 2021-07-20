# import from .so (Rust)
from ._nlpo3_python_backend import (
    load_dict as rust_load_dict,
    segment as rust_segment,
)
from typing import List


def load_dict(file_path: str, dict_name: str):
    """Load dictionary from a file.

    :param file_path: Absolute path to a dictionary file
    :type file_path: str
    :param dict_name: A unique dictionary name, use for reference.
        Can be any valid utf-8 string, except "default" which is reserved.
    :type dict_name: str
    """
    load_result = rust_load_dict(file_path, dict_name)
    print(load_result)


def segment(
    text: str,
    dict_name: str = "default",
    safe: bool = False,
    parallel: bool = False,
) -> List[str]:
    """Break text into tokens.

    This method is an implementation of newmm segmentaion.
    Support multithread mode - set by parallel flag.

    :param text: Input text
    :type text: str
    :param dict_name: Path to dictionary, defaults to "default"
    :type dict_name: str, optional
    :param safe: Use safe mode to avoid long waiting time in
        a text with lots of ambiguous word boundaries,
        defaults to False
    :type safe: bool, optional
    :param parallel: Use multithread mode, defaults to False
    :type parallel: bool, optional
    :return: List of tokens
    :rtype: List[str]
    """
    result = rust_segment(text, dict_name, safe, parallel)
    return result
