---
SPDX-FileCopyrightText: 2024-2026 PyThaiNLP Project
SPDX-License-Identifier: Apache-2.0
---

# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

Major release with breaking changes.

### Added

- Added three tokenizer choices with consistent usage across packages:
  `NewmmTokenizer`, `NewmmFstTokenizer`, and `DeepcutTokenizer`.
- Added better parallel processing controls for large input.
- Added CLI tokenizer selection (`newmm`, `nf`, `deepcut`).
- **`TrieChar` memory optimization**: removed the parallel `HashSet<String>`
  word store. Words are now encoded exclusively in the trie structure,
  eliminating per-word duplicate string allocations (~92 bytes per word for
  Thai text, ~12% total reduction for 62k-word dictionary). `contain()` does
  a trie walk; `iterate()` does a depth-first traversal; a `word_count: usize`
  counter replaces `HashSet::len()`. Construction is ~35% faster.
- **`TrieCharLegacy`**: the original `TrieChar` (with `HashSet<String>`) is
  preserved as `TrieCharLegacy` for comparison and for workloads that call
  `contain()` on a hot path. It implements `DictBackend` and can be used as
  `NewmmTokenizer<TrieCharLegacy>`.

### Changed

- Improved API consistency for Rust, Node.js, Python, and CLI.
- Improved behavior on difficult and highly ambiguous text.
- Updated project/toolchain to current major versions (2.0.0 line).
- **Breaking:** Node.js and Python moved from global/free-function style APIs to
  class-based tokenizer APIs.
- **Breaking (Rust):** removed legacy `Tokenizer::segment_to_string` and
  tokenizer-specific `segment_to_string*` methods. Use `segment(...)` and
  `segment_with_options(...)` instead; both remain fallible and return
  `AnyResult<Vec<String>>` (`anyhow::Result<Vec<String>>`).

### Removed

- **Breaking:** legacy helper entry points were removed in bindings in favor of explicit
  tokenizer objects.

### Fixed

- Fixed major slowdown risks on highly ambiguous tokenization paths.

### Migration

Python:

```python
# Before (v1.x)
from nlpo3 import load_dict, segment
load_dict("path/to/dict.txt", "mydict")
tokens = segment("สวัสดีครับ", "mydict")

# Since v2.0.0
from nlpo3 import NewmmTokenizer
tok = NewmmTokenizer("path/to/dict.txt")
tokens = tok.segment("สวัสดีครับ")
```

Node.js ESM:

```javascript
// Before (v1.x)
loadDict("path/to/dict.txt", "mydict");
const tokens = segment("สวัสดีครับ", "mydict", false, false);

// Since v2.0.0
import { NewmmTokenizer } from "nlpo3";
const tok = new NewmmTokenizer("path/to/dict.txt");
const tokens = tok.segment("สวัสดีครับ");
```

For implementation details and design choices, see
[docs/implementation.md](./docs/implementation.md).

## [1.4.0] - 2024-11-09

### Changed

- Improve karan handling (#61)

## [1.1.2] - 2021-07-21

### Changed

- Python binding published on PyPI under new name as `nlpo3`.

## [0.2.0-beta] - 2021-05-09

### Added

- Newmm word segmentation.
- Custom dictionary support.
- Python binding published on PyPI as `pythainlp-rust-modules`.

[1.4.0]: https://github.com/PyThaiNLP/nlpo3/releases/tag/v1.4.0
[1.1.2]: https://github.com/PyThaiNLP/nlpo3/releases/tag/v1.1.2
[0.2.0-beta]: https://github.com/PyThaiNLP/nlpo3/releases/tag/v0.2.0-beta
