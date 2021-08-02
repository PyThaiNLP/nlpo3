<a href="https://pypi.python.org/pypi/nlpo3"><img alt="pypi" src="https://img.shields.io/pypi/v/nlpo3.svg"/></a>
<a href="https://www.python.org/downloads/release/python-360/"><img alt="Python 3.6" src="https://img.shields.io/badge/python-3.6-blue.svg"/></a>
<a href="https://opensource.org/licenses/Apache-2.0"><img alt="License" src="https://img.shields.io/badge/License-Apache%202.0-blue.svg"/></a>
<a href="https://pepy.tech/project/nlpo3"><img alt="Downloads" src="https://pepy.tech/badge/nlpo3/month"/></a>

# nlpO3 Python binding

Python binding for nlpO3, a Thai natural language processing library in Rust.

## Features

- Thai word tokenizer
  - `segment()` - use maximal-matching dictionary-based tokenization algorithm and honor Thai Character Cluster boundaries
    - with default built-in dictionary (62,000 words, a copy [from PyThaiNLP](https://github.com/PyThaiNLP/pythainlp))
    - [2.5x faster](notebooks/nlpo3_segment_benchmarks.ipynb) than similar pure Python implementation (PyThaiNLP's newmm)
  - support custom dictionary via `load_dict()`

## Install

```bash
pip install nlpo3
```

## Usage

Tokenization using default dictionary:
```python
from nlpo3 import segment

segment("สวัสดีครับ")
```

will return a list of strings:
```python
['สวัสดี', 'ครับ']
```

Load file `path/to/dict.file` to memory and assigned it with name `custom_dict`. Then tokenize a text with `custom_dict` dictionary:
```python
from nlpo3 import load_dict, segment

load_dict("path/to/dict.file", "custom_dict")
segment("สวัสดีครับ", "custom_dict")
```

Use multithread mode, also use the `custom_dict` dictionary:
```python
segment("สวัสดีครับ", parallel=True, dict_name="custom_dict")
```

Use safe mode to avoid long waiting time in some edge cases for text with lots of ambiguous word boundaries:
```python
segment("สวัสดีครับ", safe=True)
```

## Build

### Requirements

- [Rust 2018 Edition](https://www.rust-lang.org/tools/install)
- Python 3.6 or newer
- Python Development Headers
  - Ubuntu: `sudo apt-get install python3-dev`
  - macOS: No action needed
- [PyO3](https://github.com/PyO3/pyo3) - already included in Cargo.toml
- [setuptools-rust](https://github.com/PyO3/setuptools-rust)

### Steps

```bash
python -m pip install --upgrade build
python -m build
```

This should generate a wheel file, in `dist/` directory, which can be installed by pip.

## Issues

Please report issues at https://github.com/PyThaiNLP/nlpo3/issues
