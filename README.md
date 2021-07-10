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
- [Maturin](https://github.com/PyO3/maturin) 
- -  For Windows, use "pip install --pre -U maturin" as of 2021-07-10. See (this Maturin github issue for details)[https://github.com/PyO3/maturin/issues/579] 

### Build steps

#### Linux
```bash
maturin build --release -i python --many-linux off -m python_binding/Cargo.toml
```
or
```bash
maturin build --release -i python -m python_binding/Cargo.toml
```

#### Windows (PowerShell)
```shell
path\\to\\maturin.exe build --release -i python -m python_binding/Cargo.toml
```

#### macOS
```zsh
maturin build --release -i python3 -m python_binding/Cargo.toml
```

This should generate a wheel file, in `target/wheels/` directory, which can be installed by pip.

Note: Omitting "-i python" will let Maturin build for all Python versions detected.

## Support

Please report issues at https://github.com/PyThaiNLP/oxidized-thainlp

# TODO

Add NodeJS binding