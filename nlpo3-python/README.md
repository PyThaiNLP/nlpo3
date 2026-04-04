---
SPDX-FileCopyrightText: 2024-2026 PyThaiNLP Project
SPDX-License-Identifier: Apache-2.0
---

# nlpO3 Python binding

[![PyPI](https://img.shields.io/pypi/v/nlpo3.svg "PyPI")](https://pypi.python.org/pypi/nlpo3)
[![Python 3.9](https://img.shields.io/badge/python-3.9-blue.svg "Python 3.9")](https://www.python.org/downloads/)
[![Apache-2.0](https://img.shields.io/badge/License-Apache%202.0-blue.svg "Apache-2.0")](https://opensource.org/license/apache-2-0)
[![DOI](https://zenodo.org/badge/DOI/10.5281/zenodo.14082448.svg)](https://doi.org/10.5281/zenodo.14082448)

Python binding for nlpO3 Thai word tokenization.

## Overview

- Class-based tokenizer API.
- Three tokenizer classes:
  - `NewmmTokenizer`
  - `NewmmFstTokenizer`
  - `DeepcutTokenizer`
- Improved behavior on ambiguous and long input.
- Improved parallel processing controls for large input.

For implementation details and design choices, see
[../docs/implementation.md](../docs/implementation.md).

## Install

```bash
pip install nlpo3
```

## Requirements

- Python 3.9 or newer
- Rust Edition 2024 (required when building from source)

## Quick start

```python
from nlpo3 import NewmmTokenizer, NewmmFstTokenizer, DeepcutTokenizer

tok = NewmmTokenizer("/path/to/dict.txt")
print(tok.segment("สวัสดีครับ"))

tok_fst = NewmmFstTokenizer("/path/to/dict.txt")
print(tok_fst.segment("สวัสดีครับ"))

tok_deepcut = DeepcutTokenizer()
print(tok_deepcut.segment("สวัสดีครับ"))
```

## Error handling

- `segment(...)` raises `RuntimeError` if tokenization or inference fails.
- Constructor methods (for example, loading dictionaries or models) also raise
  `RuntimeError` on failure.

Handle these with standard Python `try/except` blocks.

## Segment options

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `safe` | `bool` | `False` | For `NewmmTokenizer` and `NewmmFstTokenizer`: avoid long run times on inputs with many ambiguous word boundaries |
| `parallel_chunk_size` | `Optional[int]` | `None` | Enable chunked parallel processing for larger text; `None`, `0`, or too-small values disable parallel mode |

`DeepcutTokenizer` supports `parallel_chunk_size` but not `safe`.

> **Note on parallel mode accuracy:** when `parallel_chunk_size` is set, text
> is split into chunks before tokenization. Token sequences near chunk
> boundaries may differ from full-text results. This is acceptable for tasks
> such as text classification and word embedding, but may not be suitable for
> tasks that require precise linguistic unit identification.

[benchmark]: ./notebooks/nlpo3_segment_benchmarks.ipynb
[deepcut]: https://github.com/rkcosmos/deepcut
[lekcut]: https://github.com/PyThaiNLP/LEKCut

## Dictionary

To keep the library small, nlpO3 does not include a dictionary.
For dictionary-based tokenizers you must provide one.

Recommended dictionaries:

- [words_th.txt][dict-pythainlp] from [PyThaiNLP][pythainlp]
  (~62,000 words, CC0-1.0)
- [word break dictionary][dict-libthai] from [libthai][libthai]
  (categories, LGPL-2.1)

[pythainlp]: https://github.com/PyThaiNLP/pythainlp
[libthai]: https://github.com/tlwg/libthai/
[dict-pythainlp]: https://github.com/PyThaiNLP/pythainlp/blob/dev/pythainlp/corpus/words_th.txt
[dict-libthai]: https://github.com/tlwg/libthai/tree/master/data

## Support

- Issues: <https://github.com/PyThaiNLP/nlpo3/issues>

## License

nlpO3 Python binding is copyrighted by its authors
and licensed under terms of the Apache Software License 2.0 (Apache-2.0).
See file [LICENSE](./LICENSE) for details.

## Binary wheels

Pre-built binary packages are available on [PyPI][pypi].

[pypi]: https://pypi.org/project/nlpo3/

| Python       | OS        | Architecture | Binary wheel  |
| ------------ | --------- | ------------ | ------------- |
| 3.14         | Windows   | x86          | ✓             |
|              |           | AMD64        | ✓             |
|              | macOS     | x86_64       | ✓             |
|              |           | arm64        | ✓             |
|              | manylinux | x86_64       | ✓             |
|              |           | i686         | ✓             |
|              | musllinux | x86_64       | ✓             |
| 3.14t        | Windows   | x86          | ✓             |
|              |           | AMD64        | ✓             |
|              | macOS     | x86_64       | ✓             |
|              |           | arm64        | ✓             |
|              | manylinux | x86_64       | ✓             |
|              |           | i686         | ✓             |
|              | musllinux | x86_64       | ✓             |
| 3.13         | Windows   | x86          | ✓             |
|              |           | AMD64        | ✓             |
|              | macOS     | x86_64       | ✓             |
|              |           | arm64        | ✓             |
|              | manylinux | x86_64       | ✓             |
|              |           | i686         | ✓             |
|              | musllinux | x86_64       | ✓             |
| 3.12         | Windows   | x86          | ✓             |
|              |           | AMD64        | ✓             |
|              | macOS     | x86_64       | ✓             |
|              |           | arm64        | ✓             |
|              | manylinux | x86_64       | ✓             |
|              |           | i686         | ✓             |
|              | musllinux | x86_64       | ✓             |
| 3.11         | Windows   | x86          | ✓             |
|              |           | AMD64        | ✓             |
|              | macOS     | x86_64       | ✓             |
|              |           | arm64        | ✓             |
|              | manylinux | x86_64       | ✓             |
|              |           | i686         | ✓             |
|              | musllinux | x86_64       | ✓             |
| 3.10         | Windows   | x86          | ✓             |
|              |           | AMD64        | ✓             |
|              | macOS     | x86_64       | ✓             |
|              |           | arm64        | ✓             |
|              | manylinux | x86_64       | ✓             |
|              |           | i686         | ✓             |
|              | musllinux | x86_64       | ✓             |
| 3.9          | Windows   | x86          | ✓             |
|              |           | AMD64        | ✓             |
|              | macOS     | x86_64       | ✓             |
|              |           | arm64        | ✓             |
|              | manylinux | x86_64       | ✓             |
|              |           | i686         | ✓             |
|              | musllinux | x86_64       | ✓             |
