# nlpO3 Python binding

Python binding for nlpO3, a Thai natural language processing library in Rust.

## Features

- Word tokenizer
  - maximal-matching dictionary-based tokenization
  - 2x faster than similar pure Python implementation (PyThaiNLP's newmm)
  - support custom dictionary

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

Load file `path/to/dict.file` to memory and assigned it with name `dict_name`. Then tokenize a text with `dict_name` dictionary:
```python
from nlpo3 import load_dict, segment

load_dict("path/to/dict.file", "dict_name")
segment("สวัสดีครับ", "dict_name")
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

Please report issues at https://github.com/PyThaiNLP/oxidized-thainlp/issues

# TODO

- Find a way to build and publish on PyPI.
