---
SPDX-FileCopyrightText: 2024 PyThaiNLP Project
SPDX-License-Identifier: Apache-2.0
---

# nlpO3

[![crates.io](https://img.shields.io/crates/v/nlpo3.svg "crates.io")](https://crates.io/crates/nlpo3/)
[![crates.io](https://img.shields.io/crates/v/nlpo3-cli.svg "crates.io")](https://crates.io/crates/nlpo3-cli/)
[![PyPI](https://img.shields.io/pypi/v/nlpo3.svg "PyPI")](https://pypi.python.org/pypi/nlpo3)

Thai natural language processing library in Rust,
with Python and Node bindings. Formerly oxidized-thainlp.

```shell
cargo install nlpo3
```

```shell
pip install nlpo3
```

## Table of contents

- [Features](#features)
- [Dictionary file](#dictionary-file)
- [Usage](#usage)
  - [Python binding](#python-binding)
  - [Node.js binding](#nodejs-binding)
  - [Rust library](#rust-library)
  - [Command-line interface](#command-line-interface)
- [Build](#build)
- [Development](#development)
- [License](#license)

## Features

- Thai word tokenizer
  - Use maximal-matching dictionary-based tokenization algorithm
    and honor [Thai Character Cluster][tcc] boundaries
    - [2.5x faster][benchmark]
      than similar pure Python implementation (PyThaiNLP's newmm)
  - Load a dictionary from a plain text file (one word per line)
    or from `Vec<String>`

[tcc]: https://dl.acm.org/doi/10.1145/355214.355225
[benchmark]: https://github.com/PyThaiNLP/nlpo3/blob/main/nlpo3-python/notebooks/nlpo3_segment_benchmarks.ipynb

## Dictionary file

- For the interest of library size, nlpO3 does not assume what dictionary the
  user would like to use, and it does not come with a dictionary.
- A dictionary is needed for the dictionary-based word tokenizer.
- For tokenization dictionary, try
  - [words_th.tx][dict-pythainlp] from [PyThaiNLP][pythainlp]
    - ~62,000 words
    - CC0-1.0
  - [word break dictionary][dict-libthai] from [libthai][libthai]
    - consists of dictionaries in different categories, with a make script
    - LGPL-2.1

[pythainlp]: https://github.com/PyThaiNLP/pythainlp
[libthai]: https://github.com/tlwg/libthai/
[dict-pythainlp]: https://github.com/PyThaiNLP/pythainlp/blob/dev/pythainlp/corpus/words_th.txt
[dict-libthai]: https://github.com/tlwg/libthai/tree/master/data

## Usage

### Node.js binding

#### Source code

See [nlpo3-nodejs](./nlpo3-nodejs/) directory.

### Python binding

[![PyPI](https://img.shields.io/pypi/v/nlpo3.svg "PyPI")](https://pypi.python.org/pypi/nlpo3)

#### Install

```shell
pip install nlpo3
```

#### Example

```python
from nlpo3 import load_dict, segment

load_dict("path/to/dict.file", "dict_name")
segment("สวัสดีครับ", "dict_name")
```

#### Source code

See [nlpo3-python](./nlpo3-python/) directory.

### Rust library

[![crates.io](https://img.shields.io/crates/v/nlpo3.svg "crates.io")](https://crates.io/crates/nlpo3/)

#### Install

```shell
cargo install nlpo3
```

In `Cargo.toml`:

```toml
[dependencies]
# ...
nlpo3 = "1.3.2"
```

#### Example

Create a tokenizer using a dictionary from file,
then use it to tokenize a string (safe mode = true, and parallel mode = false):

```rust
use nlpo3::tokenizer::newmm::NewmmTokenizer;
use nlpo3::tokenizer::tokenizer_trait::Tokenizer;

let tokenizer = NewmmTokenizer::new("path/to/dict.file");
let tokens = tokenizer.segment("ห้องสมุดประชาชน", true, false).unwrap();
```

Create a tokenizer using a dictionary from a vector of Strings:

```rust
let words = vec!["ปาลิเมนต์".to_string(), "คอนสติติวชั่น".to_string()];
let tokenizer = NewmmTokenizer::from_word_list(words);
```

Add words to an existing tokenizer:

```rust
tokenizer.add_word(&["มิวเซียม"]);
```

Remove words from an existing tokenizer:

```rust
tokenizer.remove_word(&["กระเพรา", "ชานชลา"]);
```

#### Source code

See the [root](/) directory.

### Command-line interface

[![crates.io](https://img.shields.io/crates/v/nlpo3-cli.svg "crates.io")](https://crates.io/crates/nlpo3-cli/)

#### Install

```shell
cargo install nlpo3-cli
```

#### Usage

```shell
nlpo3 help
```

#### Example

```bash
echo "ฉันกินข้าว" | nlpo3 segment
```

#### Source code

See [nlpo3-cli](./nlpo3-cli/) directory.

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

## Development

Development document:

- [Notes on custom string](src/NOTE_ON_STRING.md)

Issues:

- Please report issues at <https://github.com/PyThaiNLP/nlpo3/issues>

## License

nlpO3 is copyrighted by its authors and licensed under terms of the Apache
Software License 2.0 (Apache-2.0) - see file [LICENSE](./LICENSE) for details.
