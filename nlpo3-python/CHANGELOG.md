---
SPDX-FileCopyrightText: 2024-2026 PyThaiNLP Project
SPDX-License-Identifier: Apache-2.0
---

# Changelog

All notable changes to the nlpO3 Python package are documented here.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

Major release with breaking changes.

### Added

- Added class-based tokenizer API with three classes:
  `NewmmTokenizer`, `NewmmFstTokenizer`, and `DeepcutTokenizer`.
- Added `parallel_chunk_size` options for large-input processing.
- Added top-level exports for all tokenizer classes in the `nlpo3` namespace.

### Changed

- **Breaking:** moved from global helper functions to explicit tokenizer objects.
- Improved behavior on ambiguous and long input.
- Updated build/toolchain baseline for the 2.0 line.

### Removed

- **Breaking:** removed `load_dict()`, `segment()`, and `segment_deepcut()`
  free functions.

### Migration

```python
# Before (v1.x)
from nlpo3 import load_dict, segment, segment_deepcut
load_dict("path/to/dict.txt", "mydict")
tokens = segment("สวัสดีครับ", "mydict")
tokens = segment_deepcut("สวัสดีครับ")

# After (v2.0)
from nlpo3 import NewmmTokenizer, NewmmFstTokenizer, DeepcutTokenizer
tok = NewmmTokenizer("path/to/dict.txt")
tokens = tok.segment("สวัสดีครับ")
tokens = DeepcutTokenizer().segment("สวัสดีครับ")
```

For implementation details and design choices, see
[docs/implementation.md](../docs/implementation.md) in the repository root.
