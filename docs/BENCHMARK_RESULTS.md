---
SPDX-FileCopyrightText: 2024-2026 PyThaiNLP Project
SPDX-License-Identifier: Apache-2.0
---

# Benchmark results: dictionary backends and tokenizers compared

Updated on: 2026-04-11

This document records the results of running `cargo bench`
(and `cargo bench --features deepcut` for `DeepcutTokenizer`).

Measurements were collected with [Criterion.rs](https://github.com/bheisler/criterion.rs) 0.8
on a single-threaded workload.

---

## Benchmark sets

### Set 1 — Dictionary backend comparison

Compares `TrieChar`, `TrieCharLegacy`, and `FstDict` on small text inputs
where all three backends are practical.  Covers:

| Group | What is measured |
| ----- | ---------------- |
| `dict_construction` | Build time per backend × 4 dict sizes |
| `prefix_lookup` | Per-query prefix scan per backend × 4 dict sizes |
| `dict_operations` | `add` / `remove` / `contain` per backend × 4 dict sizes |
| `memory_footprint` | Heap-size estimates for all 4 dict sizes (printed to stderr) |
| `clone_cost` | Arc-backed clone verification |
| `dict_backend_tokenization` | End-to-end speed: 3 backends × 4 dicts × 2 small texts |

### Set 2 — Tokenizer comparison

Compares `NewmmTokenizer` (TrieChar), `NewmmLegacyTokenizer` (TrieCharLegacy),
and `DeepcutTokenizer` on large, real-world texts.

**`FstDict` / `NewmmFstTokenizer` is excluded from Set 2.**  On texts larger
than ~100 KB its per-character FST fallback for OOV input makes individual
benchmark iterations take tens of seconds, which is impractical for regular
regression testing.

| Group | What is measured |
| ----- | ---------------- |
| `tokenizer_performance` | End-to-end speed: 2 tokenizers (+ Deepcut) × 2 dicts × up to 4 texts |

---

## Benchmark environment

- Rust: stable (release profile, `lto = true`, `codegen-units = 1`)
- Criterion.rs 0.8
- Dictionaries (all in `tests/data/`):

| File | Words | Word-length profile |
| ---- | ----: | ------------------- |
| `dict-1k-long.txt` | 1 000 | 15–36 chars |
| `dict-1k-short.txt` | 1 000 | 3–10 chars |
| `dict-10k.txt` | 10 000 | 1–34 chars |
| `dict-words-th.txt` | 62 018 | 1–36 chars |

- Text inputs (all in `tests/data/`):

| File | Size | OOV profile |
| ---- | ---: | ----------- |
| `text-wikipedia-s.txt` | ~2.4 KB | Wikipedia Thai/Latin mix |
| `text-wikipedia-m.txt` | ~41 KB | Wikipedia Thai/Latin mix |
| `text-wikipedia-l.txt` | ~615 KB | Wikipedia Thai/Latin mix |
| `text-only-dict-10k-words.txt` | ~1 MB | Only words from `dict-10k` — low OOV when using `dict-10k` |
| `text-only-dict-10k-1k-words.txt` | ~1 MB | Mix of `dict-10k` + `dict-1k-*` words — moderate OOV for `dict-10k` |
| `text-ws-social.txt` | ~6.3 MB | Social-media posts — high OOV |

> `text-only-dict-10k-words.txt` contains only words from `dict-10k.txt`.
> A tokenizer using `dict-10k` should see virtually no OOV when processing it.
>
> `text-only-dict-10k-1k-words.txt` mixes words from `dict-200-common.txt`,
> `dict-10k.txt`, `dict-1k-long.txt`, and `dict-1k-short.txt` (roughly 25% each).
> A tokenizer using only `dict-10k` will encounter moderate OOV from the 1k entries.

---

## 1. Dictionary construction

Build time for each backend × 4 dictionary sizes.
`FstDict` sorts the input before building the automaton; trie builds are
unsorted insert-per-word.

| Backend | 1k-long | 1k-short | 10k | 62k (words-th) |
| ------- | ------: | -------: | --: | -------------: |
| `TrieChar::new` | 669.60 µs | 360.42 µs | 3.4156 ms | 26.032 ms |
| `TrieCharLegacy::new` | 697.20 µs | 392.45 µs | 3.7421 ms | 28.309 ms |
| `FstDict::from_words` | 1.6467 ms | 670.53 µs | 5.1622 ms | 32.581 ms |

*Run `cargo bench -- dict_construction` to populate this table.*

### Complexity

| Backend | `new(n words, avg length k)` |
| ------- | ---------------------------- |
| `TrieChar` | O(n·k) |
| `TrieCharLegacy` | O(n·k) + O(n) hash inserts |
| `FstDict` | O(n·k·log n) (sort + automaton build) |

---

## 2. Dictionary prefix lookup (hot tokenization path)

Finding all dictionary entries that are prefixes of a query string.
Called at every character position during tokenization.

Three query types: `short-thai` (5 chars), `mixed` (7 chars), `medium-thai` (14 chars).

### Example: 62k-word dictionary

| Backend | short-thai | mixed | medium-thai |
| ------- | ---------: | ----: | ----------: |
| `TrieChar::prefix_ref` | 39.024 ns | 45.568 ns | 62.569 ns |
| `TrieCharLegacy::prefix_ref` | 38.914 ns | 45.638 ns | 60.160 ns |
| `FstDict::prefix_lengths` | 1.6373 µs | 1.0055 µs | 2.0908 µs |

*Run `cargo bench -- prefix_lookup` to populate this table.*

### Complexity

| Backend | `prefix_lengths(k chars)` |
| ------- | ------------------------- |
| `TrieChar` | O(k) pointer-chasing trie walk |
| `TrieCharLegacy` | O(k) — identical trie walk |
| `FstDict` | O(k·B) where B = bytes/char (3 for Thai) |

---

## 3. Dictionary operations: contain / add / remove

### contain — membership test

| Backend | 1k-long | 1k-short | 10k | 62k (words-th) |
| ------- | ------: | -------: | --: | -------------: |
| `TrieChar::contain` | 105.01 ns | 106.14 ns | 101.12 ns | 104.33 ns |
| `TrieCharLegacy::contain` | 79.264 ns | 79.450 ns | 77.382 ns | 78.803 ns |
| `FstDict::contains` | 46.727 ns | 58.818 ns | 77.720 ns | 89.090 ns |

### add / remove (includes dict-clone overhead)

Each iteration clones the pre-built dictionary, then mutates it.
This simulates copy-on-write mutation.

| Backend | 1k-long | 1k-short | 10k | 62k (words-th) |
| ------- | ------: | -------: | --: | -------------: |
| `TrieChar::add` | 341.67 µs | 141.87 µs | 1.3138 ms | 8.1854 ms |
| `TrieCharLegacy::add` | 368.00 µs | 164.73 µs | 1.6188 ms | 10.470 ms |
| `FstDict::add` | 1.2322 µs | 331.95 ns | 2.5838 µs | 12.724 µs |
| `TrieChar::remove` | 341.11 µs | 140.21 µs | 1.3308 ms | 8.7859 ms |
| `TrieCharLegacy::remove` | 365.81 µs | 164.96 µs | 1.6395 ms | 10.281 ms |
| `FstDict::remove` | 1.2079 µs | 311.95 ns | 2.5809 µs | 12.727 µs |

*Run `cargo bench -- dict_operations` to populate these tables.*

Trie `add`/`remove` times are dominated by the **clone cost** (O(n·k) node
copies).  `FstDict::add`/`remove` clone only a byte vector plus a small delta
`HashSet`, making them far cheaper.

### Complexity summary

| Operation | `TrieChar` | `TrieCharLegacy` | `FstDict` |
| --------- | ---------- | ---------------- | --------- |
| `contain(k)` | O(k) trie walk | O(1) hash | O(1) hash + O(k·B) FST |
| `add(k)` pure | O(k) | O(1) hash + O(k) | O(1) hash delta |
| `remove(k)` pure | O(k) | O(1) hash + O(k) | O(1) hash delta |
| clone | O(n·k) deep copy | O(n·k) + O(n) strings | O(FST bytes) ≈ O(n) |

---

## 4. Memory footprint

Run `cargo bench -- memory_footprint` to print live estimates to stderr.

### String representation

| Representation | Heap bytes per character |
| -------------- | -----------------------: |
| `CharString` (UTF-8 source + `u32` position table) | ~6.3 bytes/char |

Thai characters are 3-byte UTF-8 sequences plus one 4-byte `u32` position
entry ≈ 7 bytes/char for Thai, less for ASCII.

### Dictionary storage estimates

| Dict | Words | FstDict | TrieChar | TrieCharLegacy |
| ---- | ----: | ------: | -------: | -------------: |
| `dict-1k-long` | 1 000 | | | |
| `dict-1k-short` | 1 000 | | | |
| `dict-10k` | 10 000 | | | |
| `dict-words-th` | 62 018 | ~0.9 MB | ~43 MB | ~49 MB |

*Live values printed to stderr by `bench_memory_footprint`.*

The ~6 MB overhead of `TrieCharLegacy` over `TrieChar` comes from the
`HashSet<String>` entry: 24-byte `String` header + UTF-8 content + ~56 bytes
of `FxHashSet` bucket overhead per word.

### DeepcutTokenizer model

The bundled deepcut ONNX model (`model/deepcut.onnx`) is approximately
**3.9 MB** compiled into the binary. There is no runtime dictionary.

---

## 5. DictBackend end-to-end tokenization (Set 1-F)

All three backends × 4 dictionaries × 2 small text inputs.
FstDict is included here because these are small texts.

Text inputs: `text-wikipedia-s` (~2.4 KB), `text-wikipedia-m` (~41 KB).

| Backend / Dict | wikipedia-s | wikipedia-m |
| -------------- | ----------: | ----------: |
| `TrieChar / 1k-long` | 47.713 µs | 1.0240 ms |
| `TrieChar / 1k-short` | 47.266 µs | 1.0171 ms |
| `TrieChar / 10k` | 63.857 µs | 1.3502 ms |
| `TrieChar / words-th` | 70.458 µs | 1.5642 ms |
| `TrieCharLegacy / words-th` | 70.923 µs | 1.5629 ms |
| `FstDict / words-th` | 851.90 µs | 128.88 ms |

*Run `cargo bench -- dict_backend_tokenization` to populate this table.*

---

## 6. Tokenizer performance (Set 2)

`NewmmTokenizer` (TrieChar) and `NewmmLegacyTokenizer` (TrieCharLegacy)
on larger texts.  `DeepcutTokenizer` included with `--features deepcut`.
`FstDict`/`NewmmFstTokenizer` excluded (impractically slow on large text).

### Run A — dict-10k

| Tokenizer | text-only-10k (low OOV) | text-only-10k-1k (mod. OOV) | wikipedia-l |
| --------- | ----------------------: | --------------------------: | ----------: |
| `NewmmTokenizer` (TrieChar) | 134.49 ms | 127.89 ms | |
| `NewmmLegacyTokenizer` (TrieCharLegacy) | 146.10 ms | 125.78 ms | |
| `DeepcutTokenizer` | 305.69 s | | |

### Run B — dict-words-th

| Tokenizer | text-only-10k (low OOV) | text-only-10k-1k (low OOV) | wikipedia-l | ws-social (high OOV) |
| --------- | ----------------------: | -------------------------: | ----------: | -------------------: |
| `NewmmTokenizer` (TrieChar) | | | | |
| `NewmmLegacyTokenizer` (TrieCharLegacy) | | | | |
| `DeepcutTokenizer` | | | | |

*Run `cargo bench -- tokenizer_performance` to populate these tables.*

**OOV effect:** When a tokenizer uses `dict-10k` on `text-only-dict-10k-words.txt`
(no OOV), it should achieve maximum throughput.  The same tokenizer on
`text-only-dict-10k-1k-words.txt` (moderate OOV from the 1k entries) will be
measurably slower.  Switching to `dict-words-th` largely eliminates the OOV
penalty on both text files.

---

## 7. TrieChar vs TrieCharLegacy: summary

| Property | `TrieChar` (default) | `TrieCharLegacy` |
| -------- | -------------------- | ---------------- |
| `prefix_ref()` speed | identical | identical |
| End-to-end tokenization | identical | identical |
| `contain()` | O(k) trie walk | **O(1) hash** |
| Memory (62k words) | **~43 MB** | ~49 MB (+12%) |
| Construction (62k) | **fastest** | +~50% |
| `add()` / `remove()` pure | O(k) | O(k) + O(1) hash |
| Clone cost | O(n·k) | O(n·k) + O(n) |

**Use `TrieChar`** for all production tokenization.  It is the default.

**Use `TrieCharLegacy`** only if your application calls `contain()` heavily
and the 12% extra memory is acceptable.

---

## Summary: when to use each backend

| Scenario | Recommendation |
| -------- | -------------- |
| General-purpose tokenization | `NewmmTokenizer` (TrieChar) — default |
| Frequent `contain()` calls | `NewmmTokenizer<TrieCharLegacy>` — O(1) hash |
| Memory-constrained (< 1 MB dict budget) | `NewmmFstTokenizer` (FstDict) — ~14 B/word |
| Frequent dict mutations (add/remove) | `NewmmFstTokenizer` — O(1) delta, cheap clone |
| Highest accuracy, no dict needed | `DeepcutTokenizer` — CNN/ONNX, slower |

### Dict size × OOV rate decision guide

| Dict size | OOV rate | Backend choice |
| --------- | -------- | -------------- |
| 1k words | Any | Any backend; FstDict fine for small-text workloads |
| 10k words | Low | Any trie backend; avoid FstDict on texts > ~100 KB |
| 10k words | High | Trie only; FstDict FST-fallback amplifies OOV cost heavily |
| 62k words | Low | `TrieChar` preferred; FstDict only if memory budget < 1 MB |
| 62k words | High | `TrieChar` only; FstDict impractical on large texts |

### API example

```rust
use nlpo3::tokenizer::newmm::NewmmTokenizer;
use nlpo3::tokenizer::trie_char::{TrieChar, TrieCharLegacy};
use nlpo3::tokenizer::fst_dict::FstDict;

// Default (TrieChar)
let tok = NewmmTokenizer::new("dict.txt").unwrap();

// Legacy trie with O(1) contain()
let tok: NewmmTokenizer<TrieCharLegacy> =
    NewmmTokenizer::<TrieCharLegacy>::from_word_list(words);

// Memory-efficient FST (small texts only)
let tok = NewmmFstTokenizer::new("dict.txt").unwrap();
```
