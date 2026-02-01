---
SPDX-FileCopyrightText: 2024-2026 PyThaiNLP Project
SPDX-License-Identifier: Apache-2.0
---

# nlpO3 Python binding

[![PyPI](https://img.shields.io/pypi/v/nlpo3.svg "PyPI")](https://pypi.python.org/pypi/nlpo3)
[![Python 3.9](https://img.shields.io/badge/python-3.9-blue.svg "Python 3.9")](https://www.python.org/downloads/)
[![Apache-2.0](https://img.shields.io/badge/License-Apache%202.0-blue.svg "Apache-2.0")](https://opensource.org/license/apache-2-0)
[![DOI](https://zenodo.org/badge/DOI/10.5281/zenodo.14082448.svg)](https://doi.org/10.5281/zenodo.14082448)

Python binding for nlpO3,
a Thai natural language processing library written in Rust.

To install:

```bash
pip install nlpo3
```

## Table of Contents

- [Features](#features)
- [Use](#use)
  - [Dictionary](#dictionary)
- [Build](#build)
- [Issues](#issues)
- [License](#license)
- [Binary wheels](#binary-wheels)

## Features

- Thai word tokenizer
  - `segment()` - use maximal-matching dictionary-based tokenization algorithm
    and honor [Thai Character Cluster][tcc] boundaries
    - [2.5x faster][benchmark]
      than similar pure Python implementation (PyThaiNLP's newmm)
  - `load_dict()` - load a dictionary from a plain text file
    (one word per line)

[tcc]: https://dl.acm.org/doi/10.1145/355214.355225
[benchmark]: ./notebooks/nlpo3_segment_benchmarks.ipynb

## Use

Load a dictionary file and assign it a name (for example, `dict_name`).

Then tokenize text using the named dictionary:

```python
from nlpo3 import load_dict, segment

load_dict("path/to/dict.file", "dict_name")
segment("สวัสดีครับ", "dict_name")
```

The function returns a list of strings, for example:

```python
['สวัสดี', 'ครับ']
```

The result depends on the words included in the dictionary.

Use multithread mode using the `dict_name` dictionary:

```python
segment("สวัสดีครับ", dict_name="dict_name", parallel=True)
```

Use safe mode to avoid long run times for inputs with many ambiguous
word boundaries:

```python
segment("สวัสดีครับ", dict_name="dict_name", safe=True)
```

### Dictionary

- To keep the library small, nlpO3 does not include a dictionary.
  Users must provide a dictionary when using the dictionary-based tokenizer.
- For tokenization dictionaries, try
  - [words_th.txt][dict-pythainlp] from [PyThaiNLP][pythainlp]
    - ~62,000 words
    - CC0-1.0
  - [word break dictionary][dict-libthai] from [libthai][libthai]
    - consists of dictionaries in different categories, with a make script
    - LGPL-2.1

[pythainlp]: https://github.com/PyThaiNLP/pythainlp
[libthai]: https://github.com/tlwg/libthai/
[dict-pythainlp]: https://github.com/PyThaiNLP/pythainlp/blob/dev/pythainlp/corpus/words_th.txt
[dict-libthai]: https://github.com/tlwg/libthai/tree/master/data

## Build

### Requirements

- [Rust 2018 Edition](https://www.rust-lang.org/tools/install)
- Python 3.7 or newer (PyO3's minimum supported version)
- Python Development Headers
  - Ubuntu: `sudo apt-get install python3-dev`
  - macOS: No action needed
- [PyO3](https://github.com/PyO3/pyo3) - already included in `Cargo.toml`
- [setuptools-rust](https://github.com/PyO3/setuptools-rust)

### Steps

```bash
python -m pip install --upgrade build
python -m build
```

This should generate a wheel file, in `dist/` directory,
which can be installed by pip.

To install a wheel from a local directory:

```bash
pip install dist/nlpo3-1.3.1-cp311-cp311-macosx_12_0_x86_64.whl 
```

### Test

To run a Python unit test:

```bash
cd tests
python -m unittest
```

## Issues

Please report issues at <https://github.com/PyThaiNLP/nlpo3/issues>

## License

nlpO3 Python binding is copyrighted by its authors
and licensed under terms of the Apache Software License 2.0 (Apache-2.0).
See file [LICENSE](./LICENSE) for details.

## Binary wheels

Pre-built binary packages for CPython, GraalPy, and PyPy are available
on [PyPI][pypi] for the platforms listed below.
Versions with a "t" suffix indicate CPython with free threading.

[pypi]: https://pypi.org/project/nlpo3/

| Python       | OS        | Architecture | Binary wheel  |
| ------------ | --------- | ------------ | ------------- |
| 3.14         | Windows   | x86          | ✓             |
|              |           | AMD64        | ✓             |
|              | macOS     | x86_64       | ✓             |
|              |           | arm64        | ✓             |
|              | manylinux | x86_64       | ✓             |
|              |           | i686         | ✓             |
|              | musllinux | x86_64       | ✓             |
| 3.14t        | Windows   | x86          | ✓             |
|              |           | AMD64        | ✓             |
|              | macOS     | x86_64       | ✓             |
|              |           | arm64        | ✓             |
|              | manylinux | x86_64       | ✓             |
|              |           | i686         | ✓             |
|              | musllinux | x86_64       | ✓             |
| 3.13         | Windows   | x86          | ✓             |
|              |           | AMD64        | ✓             |
|              | macOS     | x86_64       | ✓             |
|              |           | arm64        | ✓             |
|              | manylinux | x86_64       | ✓             |
|              |           | i686         | ✓             |
|              | musllinux | x86_64       | ✓             |
| 3.12         | Windows   | x86          | ✓             |
|              |           | AMD64        | ✓             |
|              | macOS     | x86_64       | ✓             |
|              |           | arm64        | ✓             |
|              | manylinux | x86_64       | ✓             |
|              |           | i686         | ✓             |
|              | musllinux | x86_64       | ✓             |
| 3.11         | Windows   | x86          | ✓             |
|              |           | AMD64        | ✓             |
|              | macOS     | x86_64       | ✓             |
|              |           | arm64        | ✓             |
|              | manylinux | x86_64       | ✓             |
|              |           | i686         | ✓             |
|              | musllinux | x86_64       | ✓             |
| 3.10         | Windows   | x86          | ✓             |
|              |           | AMD64        | ✓             |
|              | macOS     | x86_64       | ✓             |
|              |           | arm64        | ✓             |
|              | manylinux | x86_64       | ✓             |
|              |           | i686         | ✓             |
|              | musllinux | x86_64       | ✓             |
| 3.9          | Windows   | x86          | ✓             |
|              |           | AMD64        | ✓             |
|              | macOS     | x86_64       | ✓             |
|              |           | arm64        | ✓             |
|              | manylinux | x86_64       | ✓             |
|              |           | i686         | ✓             |
|              | musllinux | x86_64       | ✓             |
| 3.8          | Windows   | x86          | ✓ (v1.3.1)    |
|              |           | AMD64        | ✓ (v1.3.1)    |
|              | macOS     | x86_64       | ✓ (v1.3.1)    |
|              |           | arm64        | ✓ (v1.3.1)    |
|              | manylinux | x86_64       | ✓ (v1.3.1)    |
|              |           | i686         | ✓ (v1.3.1)    |
|              | musllinux | x86_64       | ✓ (v1.3.1)    |
| 3.7          | Windows   | x86          | ✓ (v1.3.1)    |
|              |           | AMD64        | ✓ (v1.3.1)    |
|              | macOS     | x86_64       | ✓ (v1.3.1)    |
|              |           | arm64        |               |
|              | manylinux | x86_64       | ✓ (v1.3.1)    |
|              |           | i686         | ✓ (v1.3.1)    |
|              | musllinux | x86_64       | ✓ (v1.3.1)    |
| GraalPy 3.12 | Windows   | x86          |               |
|              |           | AMD64        |               |
|              | macOS     | x86_64       | ✓             |
|              |           | arm64        | ✓             |
|              | manylinux | x86_64       | ✓             |
|              |           | i686         |               |
| GraalPy 3.11 | Windows   | x86          |               |
|              |           | AMD64        |               |
|              | macOS     | x86_64       | ✓             |
|              |           | arm64        | ✓             |
|              | manylinux | x86_64       | ✓             |
|              |           | i686         |               |
| PyPy 3.11    | Windows   | x86          |               |
|              |           | AMD64        | ✓             |
|              | macOS     | x86_64       | ✓             |
|              |           | arm64        | ✓             |
|              | manylinux | x86_64       | ✓             |
|              |           | i686         | ✓             |
| PyPy 3.10    | Windows   | x86          |               |
|              |           | AMD64        | ✓ (v1.3.1)    |
|              | macOS     | x86_64       | ✓ (v1.3.1)    |
|              |           | arm64        | ✓ (v1.3.1)    |
|              | manylinux | x86_64       | ✓ (v1.3.1)    |
|              |           | i686         | ✓ (v1.3.1)    |
| PyPy 3.9     | Windows   | x86          |               |
|              |           | AMD64        | ✓ (v1.3.1)    |
|              | macOS     | x86_64       | ✓ (v1.3.1)    |
|              |           | arm64        | ✓ (v1.3.1)    |
|              | manylinux | x86_64       | ✓ (v1.3.1)    |
|              |           | i686         | ✓ (v1.3.1)    |
| PyPy 3.8     | Windows   | x86          |               |
|              |           | AMD64        | ✓ (v1.3.1)    |
|              | macOS     | x86_64       | ✓ (v1.3.1)    |
|              |           | arm64        | ✓ (v1.3.1)    |
|              | manylinux | x86_64       | ✓ (v1.3.1)    |
|              |           | i686         | ✓ (v1.3.1)    |
| PyPy 3.7     | Windows   | x86          |               |
|              |           | AMD64        | ✓ (v1.3.1)    |
|              | macOS     | x86_64       | ✓ (v1.3.1)    |
|              |           | arm64        |               |
|              | manylinux | x86_64       | ✓ (v1.3.1)    |
|              |           | i686         | ✓ (v1.3.1)    |
