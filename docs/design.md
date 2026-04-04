---
SPDX-FileCopyrightText: 2026 PyThaiNLP Project
SPDX-License-Identifier: CC0-1.0
---

# Design notes

## Split packaging for Python bindings

Status: implemented in v2.0.0.

### Goals

- Keep the base package small for common dictionary-based tokenization.
- Avoid shipping heavy model/runtime dependencies by default.
- Keep user-facing API stable where possible.
- Let users install optional assets/features explicitly.

### Python packages

Two packages are published to PyPI:

- `nlpo3` (base):
  - Includes `NewmmTokenizer`, `NewmmFstTokenizer`, and the core Rust extension.
  - Compiled without the `deepcut` Cargo feature: no `tract-onnx`, no `ndarray`,
    no embedded ONNX model.
  - `DeepcutTokenizer` is available as a Python-level shim that delegates to
    `nlpo3_deepcut.DeepcutTokenizer` when the optional package is present.
  - If `nlpo3-deepcut` is absent, constructing `DeepcutTokenizer` raises
    `ImportError` with a clear installation hint.
- `nlpo3-deepcut` (optional model package, `nlpo3-deepcut-python/`):
  - Provides `DeepcutTokenizer` backed by the deepcut ONNX model.
  - Compiled with the `deepcut` Cargo feature: embeds the ONNX model and
    links `tract-onnx` + `ndarray`.
  - Declares `nlpo3~=2.0` as a runtime dependency.

Install flows:

- `pip install nlpo3` — lightweight dictionary-based tokenization with
  user-supplied dictionary.
- `pip install nlpo3 nlpo3-deepcut` — adds `DeepcutTokenizer` support.

API behavior:

- Base package always exports `NewmmTokenizer` and `NewmmFstTokenizer`.
- `DeepcutTokenizer` in the base package is a forwarding shim:
  - If `nlpo3-deepcut` is installed: `DeepcutTokenizer(...)` constructs and
    returns an `nlpo3_deepcut.DeepcutTokenizer` instance.
  - If absent: raises `ImportError` with the hint
    `pip install nlpo3-deepcut`.

### Rust package layout

| Directory | Cargo crate | Compiled feature set |
|-----------|-------------|----------------------|
| `nlpo3-python/` | `nlpo3-python` | no `deepcut` feature |
| `nlpo3-deepcut-python/` | `nlpo3-deepcut-python` | `deepcut` feature (default) |

### npm packaging proposal

Proposed packages (not yet implemented):

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
- Use compatibility constraints so `-deepcut` package tracks the same
  major/minor line as base (`nlpo3~=2.0`).

### CI and test strategy

- Base package CI validates:
  - NewMM tokenizers, dictionary-from-user-path flows.
  - `DeepcutTokenizer` raises `ImportError` with guidance when `nlpo3-deepcut`
    is absent.
- Deepcut package CI validates:
  - Model availability, deepcut inference path, platform wheel coverage.
  - `nlpo3.DeepcutTokenizer` shim delegates correctly when `nlpo3-deepcut` is
    installed.
- Both packages are built and published in the same CI release workflow.

### Migration and compatibility notes

- Class names and method signatures are preserved.
- Existing code that imports `DeepcutTokenizer` from `nlpo3` continues to work
  when `nlpo3-deepcut` is installed.
- Code that imports from `nlpo3_deepcut` directly also works.
- Users upgrading from `nlpo3<2.0` should run `pip install nlpo3-deepcut` to
  restore `DeepcutTokenizer` functionality.


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
