---
SPDX-FileCopyrightText: 2024-2026 PyThaiNLP Project
SPDX-License-Identifier: Apache-2.0
---

# nlpo3-cli

[![crates.io](https://img.shields.io/crates/v/nlpo3-cli.svg "crates.io")](https://crates.io/crates/nlpo3-cli/)
[![Apache-2.0](https://img.shields.io/badge/License-Apache%202.0-blue.svg "Apache-2.0")](https://opensource.org/licenses/Apache-2.0)

Command-line interface for nlpO3 Thai word tokenization.

## Overview

- Added tokenizer selection in CLI:
  - `newmm` (default)
  - `nf` (FST dictionary backend)
  - `deepcut` (neural tokenizer)
- Improved error handling and user-facing messages.
- Better support for long-input processing.

For implementation details, see [../docs/implementation.md](../docs/implementation.md).

## Install

```bash
cargo install nlpo3-cli
```

## Requirements

- Rust Edition 2024
- rustc 1.88.0 or newer

## Quick start

```bash
nlpo3 --help
nlpo3 segment --help

echo "ฉันกินข้าว" | nlpo3 segment
```

## Segment options

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `--safe` | flag | off | Avoid long run times on inputs with many ambiguous word boundaries |
| `--parallel [CHUNK_SIZE]` | optional integer | `65536` (when flag is set without value) | Enable chunked parallel processing; if no value is provided, uses 65536 bytes |

Other useful segment flags:

- `--tokenizer` (`newmm`, `nf`, `deepcut`)
- `--dict-path` (for dictionary-based tokenizers)
- `--word-delimiter`

## Examples

Tokenize from standard input using the built-in dictionary:

```bash
echo "ฉันกินข้าว" | nlpo3 segment
```

Select a tokenizer:

```bash
# FST backend (less memory)
echo "ฉันกินข้าว" | nlpo3 segment --tokenizer nf

# Short form
echo "ฉันกินข้าว" | nlpo3 segment -t nf

# Neural CNN tokenizer (no dictionary needed)
echo "ฉันกินข้าว" | nlpo3 segment --tokenizer deepcut
```

Use a custom dictionary file:

```bash
echo "ฉันกินข้าว" | nlpo3 segment --dict-path /path/to/dict.txt
echo "ฉันกินข้าว" | nlpo3 segment -t nf --dict-path /path/to/dict.txt
```

Use a custom word delimiter:

```bash
echo "ฉันกินข้าว" | nlpo3 segment --word-delimiter " "
```

Enable safe mode (avoids long run times on ambiguous input):

```bash
echo "ฉันกินข้าว" | nlpo3 segment --safe
```

Enable parallel processing for larger text:

```bash
echo "ฉันกินข้าว" | nlpo3 segment -p
echo "ฉันกินข้าว" | nlpo3 segment -p 16384
```

## Support

- Issues: <https://github.com/PyThaiNLP/nlpo3/issues>

## License

nlpo3-cli is copyrighted by its authors
and licensed under terms of the Apache Software License 2.0 (Apache-2.0).
See file [LICENSE](./LICENSE) for details.
