---
SPDX-FileCopyrightText: 2026 PyThaiNLP Project
SPDX-License-Identifier: CC0-1.0
---

# Design notes

## Proposed split packaging for bindings (design only)

Status: proposed architecture. This section documents a packaging direction and
release strategy. It is not implemented yet.

### Goals

- Keep the base package small for common dictionary-based tokenization.
- Avoid shipping heavy model/runtime dependencies by default.
- Keep user-facing API stable where possible.
- Let users install optional assets/features explicitly.

### Python packaging proposal

Proposed packages:

- `nlpo3` (base):
  - Includes NewMM tokenizers and core Rust extension.
  - Does not bundle default dictionary.
  - Does not include Deepcut model or ONNX-heavy dependency path.
  - Users pass their own dictionary path.
- `nlpo3-dict` (optional data package):
  - Provides default `words_th.txt` and dictionary access helpers.
  - No ONNX/model dependency.
- `nlpo3-deepcut` (optional model package):
  - Provides Deepcut model assets and Deepcut-enabled extension/runtime.
  - Pulls required model/runtime dependencies.

Expected install flows:

- `pip install nlpo3` for lightweight dictionary-based tokenization with
  user-supplied dictionary.
- `pip install nlpo3-dict` to add default dictionary data.
- `pip install nlpo3-deepcut` to add `DeepcutTokenizer` support.

API behavior proposal:

- Base package exports NewMM classes always.
- Deepcut import is conditional:
  - If deepcut package is present, `DeepcutTokenizer` is available.
  - If absent, import or construction returns a clear guidance error message.

### npm packaging proposal

Proposed packages:

- `nlpo3` (base):
  - Includes dictionary-based tokenizers only.
  - No bundled dictionary by default.
  - No Deepcut model/runtime dependency.
- `nlpo3-dict` (optional data package):
  - Ships `words_th.txt` and helper utilities for locating/loading it.
- `nlpo3-deepcut` (optional model package):
  - Ships Deepcut model assets and Deepcut-enabled native addon/runtime.

Expected install flows:

- `npm install nlpo3` for lightweight default usage.
- `npm install nlpo3-dict` to add default dictionary assets.
- `npm install nlpo3-deepcut` to add deepcut tokenizer support.

API behavior proposal:

- Base package exports NewMM tokenizers always.
- Deepcut symbol is optional and loaded only when deepcut package is installed.
- Missing deepcut package should produce deterministic runtime guidance.

### Build and release model

- Maintain separate release pipelines per package.
- Keep core tokenizer code in one source tree; package-specific wrappers select
  enabled features and bundled assets.
- Use compatibility constraints so `-dict` and `-deepcut` packages track the
  same major/minor line as base.

### CI and test strategy

- Base package CI validates:
  - NewMM tokenizers, dictionary-from-user-path flows, no deepcut dependency.
- Dict package CI validates:
  - Data packaging, default dictionary loading behavior, version sync.
- Deepcut package CI validates:
  - Model availability, deepcut inference path, platform wheel/addon coverage.
- Add compatibility tests for mixed installs (base + dict, base + deepcut,
  base + dict + deepcut).

### Migration and compatibility notes

- Keep current monolithic behavior for one transition window.
- Provide deprecation notes and migration examples in README/CHANGELOG.
- Preserve class names and method signatures where possible.
- Document package-selection matrix for common use cases (small install,
  dictionary convenience, neural tokenizer support).

## Benchmark dictionary files

Three sub-dictionaries are derived from `words_th.txt` (PyThaiNLP, see LICENSE)
and committed to `tests/data/` for reproducible benchmarking. All three are
generated with `random.seed(42)`.

| File | Words | Char length | Distinct first chars | Selection rule |
|------|------:|-------------|---------------------:|----------------|
| `500-short.txt` | 500 | 3–10 | 40 | 3–10 chars; round-robin across shuffled 3-char prefix groups |
| `500-long.txt` | 500 | 15–36 | 39 | 15+ chars; round-robin across shuffled 3-char prefix groups |
| `10k.txt` | 10,000 | 1–34 | 51 | All lengths; round-robin across shuffled 3-char prefix groups |

The 3-char prefix is measured in Unicode codepoints. For Thai, each codepoint is
one character (consonant, vowel mark, or tone mark), so a 3-codepoint prefix
captures roughly the first syllable onset and provides a meaningful grouping.

`500-short.txt` and `500-long.txt` have at most one word per prefix group.
`10k.txt` also starts with one word per distinct prefix group, but it requires a
second pass for ~318 slots after exhausting all ~9,700 distinct prefixes in the
source dictionary, so a small minority of prefixes contribute 2 words.

### Purpose

These files let benchmarks compare `TrieChar` and `FstDict` backends across
vocabulary sizes and word-length profiles without loading the full
62,000-word dictionary every time. Prefix diversity ensures no backend gets
an artificial advantage from shared trie paths. Use them as:

- `500-short.txt` — small-vocabulary, short-token baseline.
- `500-long.txt` — stress test for long-token matching.
- `10k.txt` — realistic mid-size vocabulary for throughput tests.

### Regenerating

Run `python3 tests/data/gen-dicts.py` from the repository root after any
change to `words_th.txt`. The seed, length filters, and prefix-shuffle logic
are documented above so results are fully reproducible.
