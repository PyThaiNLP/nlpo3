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

### Requirements

- [Rust 2018 Edition](https://www.rust-lang.org/tools/install)
- Python 3.6 or newer
- Python Development Headers
  - Ubuntu: `sudo apt-get install python3-dev`
  - macOS: No action needed
- [PyO3](https://github.com/PyO3/pyo3) - already included in Cargo.toml
- [Maturin](https://github.com/PyO3/maturin)

### Build steps

At `rust_modules/` directory, run:

#### Linux
```bash
maturin build --release -i python --manylinux off
```
or
```bash
maturin build --release -i python
```

#### Windows (PowerShell)
```shell
path\\to\\maturin.exe build --release -i python
```

#### macOS
```zsh
maturin build --release -i python3
```

This should generate a wheel file, in `rust_modules/target/wheels/` directory, which can be installed by pip.

Note: Omitting "-i python" will let Maturin build for all Python versions detected.

## Support

Please report issues at https://github.com/PyThaiNLP/oxidized-thainlp
