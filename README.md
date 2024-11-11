---
SPDX-FileCopyrightText: 2024 PyThaiNLP Project
SPDX-License-Identifier: Apache-2.0
---

# nlpO3

[![crates.io](https://img.shields.io/crates/v/nlpo3.svg "crates.io")](https://crates.io/crates/nlpo3/)

Thai natural language processing library in Rust,
with Python and Node bindings. Formerly oxidized-thainlp.

To use as a library in a Rust project:

```shell
cargo add nlpo3
```

To use as a library in a Python project:

```shell
pip install nlpo3
```

## Table of contents

- [Features](#features)
- [Use](#use)
  - [Node.js binding](#nodejs-binding)
  - [Python binding](#python-binding)
  - [Rust library](#rust-library)
  - [Command-line interface](#command-line-interface)
  - [Dictionary](#dictionary)
- [Build](#build)
- [Develop](#develop)
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
[benchmark]: ./nlpo3-python/notebooks/nlpo3_segment_benchmarks.ipynb

## Use

### Node.js binding

See [nlpo3-nodejs](./nlpo3-nodejs/).

### Python binding

[![PyPI](https://img.shields.io/pypi/v/nlpo3.svg "PyPI")](https://pypi.python.org/pypi/nlpo3)

Example:

```python
from nlpo3 import load_dict, segment

load_dict("path/to/dict.file", "dict_name")
segment("สวัสดีครับ", "dict_name")
```

See more at [nlpo3-python](./nlpo3-python/).

### Rust library

[![crates.io](https://img.shields.io/crates/v/nlpo3.svg "crates.io")](https://crates.io/crates/nlpo3/)

#### Add to dependency

To use as a library in a Rust project:

```shell
cargo add nlpo3
```

It will add "nlpo3" to `Cargo.toml`:

```toml
[dependencies]
# ...
nlpo3 = "1.4.0"
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

### Command-line interface

[![crates.io](https://img.shields.io/crates/v/nlpo3-cli.svg "crates.io")](https://crates.io/crates/nlpo3-cli/)

Example:

```bash
echo "ฉันกินข้าว" | nlpo3 segment
```

See more at [nlpo3-cli](./nlpo3-cli/).

### Dictionary

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

## Develop

### Development document

- [Notes on custom string](src/NOTE_ON_STRING.md)

### Issues

- Please report issues at <https://github.com/PyThaiNLP/nlpo3/issues>

## License

nlpO3 is copyrighted by its authors
and licensed under terms of the Apache Software License 2.0 (Apache-2.0).
See file [LICENSE](./LICENSE) for details.
