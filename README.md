# oxidized-thainlp

Thai Natural Language Processing in Rust, with Python-binding.

## Features

- newmm dictionary-based word tokenization, at ultra fast speed
- support custom dictionary

## Usage

Install:
```bash
pip install pythainlp-rust-modules
```

Use in Python:
```python
from oxidized_thainlp import load_dict, segment

load_dict("path/to/dict.file", "dict_name")
segment("สวัสดีครับ", "dict_name")
```

Just that!

## Build It Yourself

### Build requirements

- [Rust 2018 Edition](https://www.rust-lang.org/tools/install)
- Python 3.6 or newer
- Python Development Headers
  - Ubuntu: `sudo apt-get install python3-dev`
  - macOS: No action needed
- [PyO3](https://github.com/PyO3/pyo3) - already included in Cargo.toml
- [setuptools-rust](https://github.com/PyO3/setuptools-rust)

### Build steps

#### Linux / macOS
```bash
python3 -m pip install --upgrade build
python3 -m build
```

#### Windows
```shell
py -m pip install --upgrade build
py -m build
```

This should generate a wheel file, in `dist/` directory, which can be installed by pip.

## Support

Please report issues at https://github.com/PyThaiNLP/oxidized-thainlp
