# nlpO3 (formerly oxidized-thainlp)

Thai Natural Language Processing library in Rust,
with Python and Node bindings.

## Features

- Thai word tokenizer
  - use maximal-matching dictionary-based tokenization algorithm and honor Thai Character Cluster boundaries
    - [2x faster](https://github.com/PyThaiNLP/nlpo3/blob/main/nlpo3-python/notebooks/nlpo3_segment_benchmarks.ipynb) than similar pure Python implementation (PyThaiNLP's newmm)
  - support custom dictionary
  - default dictionary included (62,000 words, a copy [from PyThaiNLP](https://github.com/PyThaiNLP/pythainlp))


## Usage

### Bindings
- [Node.js](nlpo3-nodejs/README.md)
- [Python](nlpo3-python/README.md)

### As Rust library
In `Cargo.toml`:

```toml
[dependencies]
# ...
nlpo3 = "1.1.2"
```

## Build

### Requirements

- [Rust 2018 Edition](https://www.rust-lang.org/tools/install)

### Steps

Generic test:
```bash
cargo test
```

Build (remove `--release` to keep debug information):
```bash
cargo build --release
```

Build and open API document:
```bash
cargo doc --open
```

Check `target/` for build artifacts.


## Issues

Please report issues at https://github.com/PyThaiNLP/nlpo3/issues

## TODO

- API document.
