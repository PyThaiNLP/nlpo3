---
SPDX-FileCopyrightText: 2024-2026 PyThaiNLP Project
SPDX-License-Identifier: Apache-2.0
---

# Changelog

All notable changes to this project are documented in this file.

This file follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/)
conventions. Version numbers follow [Semantic Versioning](https://semver.org/).

## [2.0.0] - 2026-04-03

### Added

- `src/char_string.rs`: `CharString` — a native UTF-8 string view with a
  precomputed byte-position index that gives O(1) character access and O(1)
  zero-copy `as_str()` slicing.
- `src/tokenizer/dict_backend.rs`: `DictBackend` trait that decouples the
  string representation (`CharString`) from the dictionary backend. Implemented
  by both `TrieChar` and `FstDict`.
- `src/tokenizer/fst_dict.rs`: `FstDict` — a memory-efficient dictionary
  backed by `fst::Set` (minimized finite-state automaton). Stores the 62k-word
  Thai dictionary in ~0.85 MB (≈14 bytes/word) compared with ~43 MB
  (≈699 bytes/word) for `TrieChar`.
- `src/tokenizer/newmm.rs`: `NewmmFstTokenizer` — a concrete tokenizer struct
  (CharString + FstDict backend) with the same API as `NewmmTokenizer`.
- `src/tokenizer/deepcut.rs`: `DeepcutTokenizer` — CNN-based Thai word tokenizer
  using ONNX inference via `tract-onnx`. Based on
  [LEKCut](https://github.com/PyThaiNLP/LEKCut) and
  [deepcut](https://github.com/rkcosmos/deepcut).
  Enabled by the `deepcut` Cargo feature.
- `BENCHMARK_RESULTS.md`: benchmark measurements for all three tokenizers.

#### Core library (`nlpo3`)

- **`Tokenizer` trait now requires `Send + Sync`** — all implementations
  satisfy these bounds; `Box<dyn Tokenizer>` can be shared across threads
  without additional bounds at call sites.
- **`NewmmTokenizer<D>` and `NewmmFstTokenizer` store the dictionary behind
  `Arc<D>`** (previously `Box<D>`). Cloning a tokenizer is now O(1)
  (Arc ref-count increment); no dictionary data is copied.
  `add_word`/`remove_word` use copy-on-write semantics via `Arc::make_mut`.
- **`TrieChar` and `FstDict` now implement `Clone`** (required by the
  copy-on-write path).
- **`TrieChar` memory optimization** — removed the parallel `HashSet<String>`
  word store. Words are now encoded exclusively in the trie structure,
  eliminating the per-word duplicate string allocation (~12–20 bytes/word
  overhead for Thai text). `contain()` does a trie walk (O(k)); `iterate()`
  does a depth-first traversal; a single `word_count: usize` counter replaces
  `HashSet::len()`.
- **Fallible constructors** — `NewmmTokenizer::new()`,
  `NewmmFstTokenizer::new()`, and `NewmmFstTokenizer::from_word_list()` now
  return `anyhow::Result<Self>` instead of panicking on I/O or parse errors.
  `NewmmTokenizer::from_word_list()` remains infallible.
- **`dict_reader` functions** now return `anyhow::Result` instead of
  `Result<_, Box<dyn Error>>`, giving `Send + Sync` error values.

#### Python binding (`nlpo3-python`)

- **Breaking**: `NewmmTokenizer` class added. Replace
  `load_dict(path, name)` + `segment(text, name)` with
  `NewmmTokenizer(path).segment(text)`.
- **Breaking**: `NewmmFstTokenizer` class added.
- All three tokenizer classes (`NewmmTokenizer`, `NewmmFstTokenizer`,
  `DeepcutTokenizer`) are exposed at the top-level `nlpo3` namespace.
- All classes are `frozen` in PyO3 (immutable Python objects), enabling
  lock-free concurrent use from multiple threads, including free-threaded
  CPython (PEP 703).

#### Node.js binding (`nlpo3-nodejs`)

- **Breaking**: replaced the global dictionary registry with class-based
  tokenizer objects. Replace `loadDict(path, name)` + `segment(text, name)`
  with `new NewmmTokenizer(path); tok.segment(text)`.
- `NewmmTokenizer`, `NewmmFstTokenizer`, `DeepcutTokenizer` JavaScript classes
  added; each holds an opaque `JsBox<TokenizerWrapper>` handle.
- Targets JavaScript ES2025, TypeScript 6.0, and Node.js 24 (LTS).
- `SegmentOptions` interface added (`safe`, `parallel` optional flags).
- TypeScript declarations (`index.d.ts`) generated from source at build time.
- Removed `lazy_static` dependency.

#### CLI (`nlpo3-cli`)

- **New `--tokenizer`/`-t` flag** with three choices:
  - `newmm` (default) — dictionary-based maximal-matching, TrieChar backend
  - `nf` — same algorithm with FST backend (lower memory use)
  - `deepcut` — neural CNN tokenizer (no dictionary needed)
- `deepcut` feature enabled by default.
- Dictionary path errors now propagate as proper error messages instead of
  panics.

### Changed

- **Breaking**: all `Cargo.toml` editions updated from 2018 to 2024;
  `rust-version` set to `"1.88.0"` across all packages.
- **Breaking**: version bumped to 2.0.0 across all packages (nlpo3, nlpo3-cli,
  nlpo3-python, nlpo3-nodejs).

### Removed

- **Breaking (Python)**: `load_dict()`, `segment()`, and `segment_deepcut()`
  free functions removed. Use `NewmmTokenizer`, `NewmmFstTokenizer`, and
  `DeepcutTokenizer` classes instead.
- **Breaking (Node.js)**: `loadDict()` and `segment()` free functions removed.
  Use tokenizer class instances instead.
- Custom four-byte string implementation removed. Use `CharString` instead.

### Fixed

- `src/tokenizer/newmm.rs`: Resolved exponential BFS path explosion in
  `bfs_paths_graph` (#101). 
  - Mirrors the fix in PyThaiNLP/pythainlp#1319.

### Migration guide

#### Python

```python
# Before (v1.x)
from nlpo3 import load_dict, segment, DeepcutTokenizer, segment_deepcut
load_dict("path/to/dict.txt", "mydict")
tokens = segment("สวัสดีครับ", "mydict")
tokens = segment_deepcut("สวัสดีครับ")

# Now (v2.0)
from nlpo3 import NewmmTokenizer, NewmmFstTokenizer, DeepcutTokenizer
tok = NewmmTokenizer("path/to/dict.txt")
tokens = tok.segment("สวัสดีครับ")
tokens = DeepcutTokenizer().segment("สวัสดีครับ")
```

#### Node.js / TypeScript

```typescript
// Before (v1.x)
loadDict("path/to/dict.txt", "mydict");
const tokens = segment("สวัสดีครับ", "mydict", false, false);

// Now (v2.0)
import { NewmmTokenizer } from "nlpo3-nodejs";
const tok = new NewmmTokenizer("path/to/dict.txt");
const tokens = tok.segment("สวัสดีครับ");
```

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

[Unreleased]: https://github.com/PyThaiNLP/nlpo3/compare/v2.0.0...HEAD
[2.0.0]: https://github.com/PyThaiNLP/nlpo3/compare/v1.4.0...v2.0.0
[1.4.0]: https://github.com/PyThaiNLP/nlpo3/releases/tag/v1.4.0
[1.1.2]: https://github.com/PyThaiNLP/nlpo3/releases/tag/v1.1.2
[0.2.0-beta]: https://github.com/PyThaiNLP/nlpo3/releases/tag/v0.2.0-beta
