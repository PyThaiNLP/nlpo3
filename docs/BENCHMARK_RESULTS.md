---
SPDX-FileCopyrightText: 2024-2026 PyThaiNLP Project
SPDX-License-Identifier: Apache-2.0
---

# Benchmark results: dictionary backends and tokenizers compared

Updated on: 2026-04-04 02:00

This document records the results of running `cargo bench`
(and `cargo bench --features deepcut` for `DeepcutTokenizer`).

Measurements were collected with [Criterion.rs](https://github.com/bheisler/criterion.rs) 0.8
on a single-threaded workload.  Each timing is the **mean** of 100 samples
(10 for slow groups such as dictionary construction).

## Dictionary backends

| Backend | Prefix-lookup | `contain()` | Memory (62 k words) |
|---------|--------------|-------------|---------------------|
| `TrieChar` | O(k) trie walk | O(k) trie walk | ~43 MB |
| `TrieCharLegacy` | O(k) trie walk | O(1) hash lookup | ~49 MB |
| `FstDict` | O(k·B) FST stream | O(1) hash + O(k·B) fallback | ~0.85 MB |

`k` = word length in characters; `B` = UTF-8 bytes per character (3 for Thai).

All three implement [`DictBackend`] and work as a drop-in replacement in
[`NewmmTokenizer<D>`]:

```rust
use nlpo3::tokenizer::newmm::NewmmTokenizer;
use nlpo3::tokenizer::trie_char::{TrieChar, TrieCharLegacy};
use nlpo3::tokenizer::fst_dict::FstDict;

// All three are one type-parameter away:
let _: NewmmTokenizer<TrieChar>       = NewmmTokenizer::new("dict.txt").unwrap();
let _: NewmmTokenizer<TrieCharLegacy> = NewmmTokenizer::<TrieCharLegacy>::from_word_list(words);
// NewmmFstTokenizer wraps NewmmTokenizer<FstDict>
let _  = NewmmFstTokenizer::new("dict.txt").unwrap();
```

## Benchmark environment

- Rust: stable (release profile, `lto = true`, `codegen-units = 1`)
- Criterion.rs 0.8
- Dictionaries (all in `tests/data/`):

| File | Words | Word-length profile |
|------|------:|---------------------|
| `500-short.txt` | 500 | 3–10 chars |
| `500-long.txt` | 500 | 15–36 chars |
| `10k.txt` | 10 000 | 1–34 chars |
| `words_th.txt` | 62 018 | 1–36 chars |

- Text sizes for end-to-end tokenization:
  - **short** – 28 Unicode characters, Thai-only
  - **medium** – 219 characters, mixed Thai / Latin / CJK / digits
  - **long** – 937 characters, mixed

---

## 1. Dictionary construction

Build time for each backend across four dictionary sizes.
`FstDict` sorts the input before building the automaton; trie builds are
unsorted insert-per-word.

| Backend | 500-short | 500-long | 10k | 62k (words_th) |
|---------|----------:|---------:|----:|---------------:|
| `TrieChar::new` | **220 µs** | **810 µs** | **5.3 ms** | **42.9 ms** |
| `TrieCharLegacy::new` | 317 µs | 909 µs | 6.8 ms | 64.4 ms |
| `FstDict::from_words` | 637 µs | 2.8 ms | 9.6 ms | 60.2 ms |

`TrieChar` is the fastest to build across all sizes.  The `TrieCharLegacy`
overhead over `TrieChar` is the `HashSet` insert per word: one heap
allocation + hash computation.  `FstDict` pays an O(n log n) sort cost
before constructing the minimised automaton.

### Complexity

| Backend | `new(n words, avg length k)` |
|---------|------------------------------|
| `TrieChar` | O(n·k) |
| `TrieCharLegacy` | O(n·k) + O(n) hash inserts |
| `FstDict` | O(n·k·log n) (sort + automaton build) |

---

## 2. Dictionary prefix lookup (hot tokenization path)

Finding all dictionary entries that are prefixes of a query string.
This is called on every character position during tokenization.

### 500-word dictionaries

| Backend | short_thai (5 ch) | mixed (7 ch) | medium_thai (14 ch) |
|---------|------------------:|-------------:|--------------------:|
| `TrieChar::prefix_ref` | **21 ns** | **17 ns** | **22 ns** |
| `TrieCharLegacy::prefix_ref` | 22 ns | 17 ns | 23 ns |
| `FstDict::prefix_lengths` | 1 727 ns | 731 ns | 1 672 ns |

### 10 000-word dictionary

| Backend | short_thai (5 ch) | mixed (7 ch) | medium_thai (14 ch) |
|---------|------------------:|-------------:|--------------------:|
| `TrieChar::prefix_ref` | **53 ns** | **48 ns** | **47 ns** |
| `TrieCharLegacy::prefix_ref` | 53 ns | 48 ns | 47 ns |
| `FstDict::prefix_lengths` | 3 438 ns | 1 657 ns | 3 694 ns |

### 62 018-word dictionary (words_th)

| Backend | short_thai (5 ch) | mixed (7 ch) | medium_thai (14 ch) |
|---------|------------------:|-------------:|--------------------:|
| `TrieChar::prefix_ref` | **68 ns** | **73 ns** | **88 ns** |
| `TrieCharLegacy::prefix_ref` | 69 ns | 72 ns | 91 ns |
| `FstDict::prefix_lengths` | 3 686 ns | 2 080 ns | 4 403 ns |
| Trie vs FST ratio | **54×** | **29×** | **50×** |

`TrieChar` and `TrieCharLegacy` have **identical prefix-lookup performance**;
both walk the same `TrieNode` structure.  `FstDict` is 29–54× slower because
it streams a byte-level FST automaton.

### Complexity

| Backend | `prefix_lengths(k chars)` |
|---------|---------------------------|
| `TrieChar` | O(k) pointer-chasing trie walk |
| `TrieCharLegacy` | O(k) — identical trie walk |
| `FstDict` | O(k·B) where B = bytes/char (3 for Thai) |

---

## 3. Dictionary operations: contain / add / remove

### contain — membership test

Each iteration calls `contain` on one word against the pre-built dictionary.

| Backend | 500-short | 500-long | 10k | 62k (words_th) |
|---------|----------:|---------:|----:|---------------:|
| `TrieChar::contain` | 109 ns | 101 ns | 113 ns | 118 ns |
| `TrieCharLegacy::contain` | **79 ns** | **77 ns** | **83 ns** | **91 ns** |
| `FstDict::contains` | 74 ns | 71 ns | 122 ns | 135 ns |

`TrieCharLegacy::contain` uses an O(1) `HashSet` lookup; `TrieChar::contain`
walks the trie (O(k)).  Both are fast in absolute terms.  `FstDict::contains`
uses hash lookup for the delta sets and the base FST for the main set; it grows
slightly with dictionary size due to FST traversal.

`contain` is **not on the hot tokenization path** — `prefix_ref` is.
The contain difference does not affect end-to-end throughput (see §5).

### add — insert one word (includes dict-clone overhead)

Each iteration clones the pre-built dictionary, then adds one word.
This simulates copy-on-write mutation.

| Backend | 500-short | 500-long | 10k | 62k (words_th) |
|---------|----------:|---------:|----:|---------------:|
| `TrieChar::add` | 133 µs | 591 µs | 2.7 ms | 21.8 ms |
| `TrieCharLegacy::add` | 159 µs | 621 µs | 3.3 ms | 26.2 ms |
| `FstDict::add` | **208 ns** | **1.1 µs** | **3.5 µs** | **52.7 µs** |

Trie add times are dominated by the **clone cost** (O(n·k) node copies).
`FstDict::add` clones only a byte vector (~0.85 MB for 62k words) plus
the delta `HashSet`, making it far cheaper.  The raw per-word insert
without clone is O(k) for trie variants and O(1) for `FstDict`.

### remove — delete one word (includes dict-clone overhead)

| Backend | 500-short | 500-long | 10k | 62k (words_th) |
|---------|----------:|---------:|----:|---------------:|
| `TrieChar::remove` | 133 µs | 595 µs | 2.7 ms | 21.8 ms |
| `TrieCharLegacy::remove` | 154 µs | 609 µs | 3.4 ms | 25.7 ms |
| `FstDict::remove` | **201 ns** | **1.1 µs** | **3.5 µs** | **52.7 µs** |

Pattern matches `add`: trie variants dominated by clone cost; `FstDict`
tracks removals in a small delta `HashSet`.

### Complexity summary

| Operation | `TrieChar` | `TrieCharLegacy` | `FstDict` |
|-----------|-----------|-----------------|-----------|
| `contain(k)` | O(k) trie walk | O(1) hash | O(1) hash + O(k·B) FST |
| `add(k)` pure | O(k) check + O(k) insert | O(1) hash + O(k) insert | O(1) hash delta |
| `remove(k)` pure | O(k) check + O(k) prune | O(1) hash + O(k) prune | O(1) hash delta |
| clone | O(n·k) deep copy | O(n·k) + O(n) strings | O(FST bytes) ≈ O(n) |

---

## 4. Full end-to-end tokenization

Using the full `words_th.txt` dictionary (62 018 words).

`NewmmTokenizer<TrieChar>` is the default; `NewmmTokenizer<TrieCharLegacy>`
wraps the legacy trie backend; `NewmmFstTokenizer` wraps `FstDict`.

| Tokenizer | short (28 ch) | medium (219 ch) | long (937 ch) |
|-----------|-------------:|----------------:|--------------:|
| `NewmmTokenizer` (TrieChar, safe=false) | **2.65 µs** | **27.7 µs** | **128 µs** |
| `NewmmTokenizer` (TrieChar, safe=true) | 2.65 µs | 27.6 µs | 172 µs |
| `NewmmLegacyTokenizer` (TrieCharLegacy, safe=false) | 2.66 µs | 24.8 µs | 129 µs |
| `NewmmLegacyTokenizer` (TrieCharLegacy, safe=true) | 2.64 µs | 27.6 µs | 173 µs |
| `NewmmFstTokenizer` (FstDict, safe=false) | 26.5 µs | 238.7 µs | 1 941 µs |
| `NewmmFstTokenizer` (FstDict, safe=true) | 26.7 µs | 221.6 µs | 1 417 µs |
| Trie vs FST ratio (safe=false) | **10×** | **9×** | **15×** |

**Key finding:** `NewmmTokenizer<TrieChar>` and
`NewmmTokenizer<TrieCharLegacy>` have **statistically identical tokenization
throughput** across all text sizes.  The faster `contain()` in `TrieCharLegacy`
provides no end-to-end benefit because `contain()` is not on the tokenization
hot path — only `prefix_ref()` is.  Both trie backends are 9–15× faster than
`FstDict` end-to-end.

`DeepcutTokenizer` results require `--features deepcut`. CNN/ONNX inference is
significantly slower than any dictionary-based method.

---

## 5. Memory footprint

### String representation

| Representation | Heap bytes per character |
|---|---:|
| `CharString` (UTF-8 source + `u32` position table) | **6.3 bytes/char** |

Thai characters are 3-byte UTF-8 sequences plus one 4-byte `u32` position
entry ≈ 7 bytes/char for Thai, less for ASCII.

### Dictionary storage (62 018 words)

| Structure | Total | Per word |
|---|---:|---:|
| `FstDict` (FST automaton) | ~0.85 MB | **14 bytes** |
| `TrieChar` (trie, no HashSet) | ~43 MB | **~699 bytes** |
| `TrieCharLegacy` (trie + HashSet) | ~49 MB | **~791 bytes** |
| `TrieChar` vs `TrieCharLegacy` savings | **~6 MB (≈12%)** | **~92 bytes/word** |
| `FstDict` vs `TrieChar` ratio | **~49× smaller** | |

The ~92 bytes/word overhead of `TrieCharLegacy` comes from the `HashSet<String>`
entry: 24-byte `String` header + average 12 bytes UTF-8 content for a 4-char
Thai word + ~56 bytes `FxHashSet` bucket overhead.

### DeepcutTokenizer model

The bundled deepcut ONNX model (`model/deepcut.onnx`) is approximately
**3.9 MB** compiled into the binary. There is no runtime dictionary.

---

## 6. TrieChar vs TrieCharLegacy: summary

The table below was the primary motivation for introducing the optimized
`TrieChar` (without the parallel `HashSet`).

| Property | `TrieChar` (new) | `TrieCharLegacy` |
|----------|-----------------|-----------------|
| `prefix_ref()` | identical | identical |
| End-to-end tokenization | identical | identical |
| `contain()` | O(k) trie walk | **O(1) hash** |
| Memory (62 k words) | **~43 MB** | ~49 MB (+12%) |
| Construction (62 k) | **42.9 ms** | 64.4 ms (+50%) |
| `add()` / `remove()` pure | O(k) | O(k) (+ O(1) hash) |
| Clone cost | O(n·k) | O(n·k) + O(n) |

**Use `TrieChar`** for all production tokenization.  It is the default.

**Use `TrieCharLegacy`** only if your application calls `contain()` on a
hot path and the 12 % extra memory is acceptable.

**Use `FstDict`** when dictionary memory is the primary constraint.

---

## Summary

| Backend | Construction | Prefix-lookup | Memory | `contain()` |
|---------|-------------|--------------|--------|-------------|
| `TrieChar` (default) | fastest | fastest | medium | O(k) |
| `TrieCharLegacy` | slower (+50%) | same as TrieChar | +12% | O(1) |
| `FstDict` | similar to legacy | **29–54× slower** | **49× smaller** | O(1) |

All three implement `DictBackend` and are interchangeable via
`NewmmTokenizer<D>`. Use `Box<dyn DictBackend>` or the generic type parameter
to select at runtime or compile time.
