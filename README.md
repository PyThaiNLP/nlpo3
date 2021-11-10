# nlpO3

Thai Natural Language Processing library in Rust,
with Python and Node bindings. Formerly oxidized-thainlp.

## Features

- Thai word tokenizer
  - use maximal-matching dictionary-based tokenization algorithm and honor Thai Character Cluster boundaries
    - [2.5x faster](https://github.com/PyThaiNLP/nlpo3/blob/main/nlpo3-python/notebooks/nlpo3_segment_benchmarks.ipynb) than similar pure Python implementation (PyThaiNLP's newmm)
  - load a dictionary from a plain text file (one word per line) or from `Vec<String>`


## Dictionary file

- For the interest of library size, nlpO3 does not assume what dictionary the developer would like to use.
  It does not come with a dictionary. A dictionary is needed for the dictionary-based word tokenizer.
- For tokenization dictionary, try
  - [words_th.tx](https://github.com/PyThaiNLP/pythainlp/blob/dev/pythainlp/corpus/words_th.txt) from [PyThaiNLP](https://github.com/PyThaiNLP/pythainlp/) - around 62,000 words (CC0)
  - [word break dictionary](https://github.com/tlwg/libthai/tree/master/data) from [libthai](https://github.com/tlwg/libthai/) - consists of dictionaries in different categories, with make script (LGPL-2.1)


## Usage

### Command-line interface

- [nlpo3-cli](nlpo3-cli/) <a href="https://crates.io/crates/nlpo3-cli/"><img alt="crates.io" src="https://img.shields.io/crates/v/nlpo3-cli.svg"/></a>

```bash
echo "ฉันกินข้าว" | nlpo3 segment
```

### Bindings
- [Node.js](nlpo3-nodejs/)
- [Python](nlpo3-python/) <a href="https://pypi.python.org/pypi/nlpo3"><img alt="pypi" src="https://img.shields.io/pypi/v/nlpo3.svg"/></a>

```python
from nlpo3 import load_dict, segment

load_dict("path/to/dict.file", "dict_name")
segment("สวัสดีครับ", "dict_name")
```

### As Rust library
<a href="https://crates.io/crates/nlpo3/"><img alt="crates.io" src="https://img.shields.io/crates/v/nlpo3.svg"/></a>

In `Cargo.toml`:

```toml
[dependencies]
# ...
nlpo3 = "1.3.1"
```

```rust
use nlpo3::tokenizer::newmm::NewmmTokenizer;
use nlpo3::tokenizer::tokenizer_trait::Tokenizer;

let tokenizer = NewmmTokenizer::new("path/to/dict.file");
let tokens = tokenizer.segment("สวัสดีครับ", false, true).unwrap();
```

## Build

### Requirements

- [Rust 2018 Edition](https://www.rust-lang.org/tools/install)

### Steps

Generic test:
```bash
cargo test
```

Build API document and open it to check:
```bash
cargo doc --open
```

Build (remove `--release` to keep debug information):
```bash
cargo build --release
```

Check `target/` for build artifacts.


## Issues

Please report issues at https://github.com/PyThaiNLP/nlpo3/issues
