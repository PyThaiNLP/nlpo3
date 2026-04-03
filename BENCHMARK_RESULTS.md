---
SPDX-FileCopyrightText: 2024-2026 PyThaiNLP Project
SPDX-License-Identifier: Apache-2.0
---

# Benchmark results: three Thai tokenizers compared

This document records the results of running `cargo bench` (and
`cargo bench --features deepcut` for `DeepcutTokenizer`).

Measurements were collected with [Criterion.rs](https://github.com/bheisler/criterion.rs) 0.8
on a single-threaded workload. Each timing is the **mean** of 100 samples
(10 for slow groups such as dictionary construction).

## Tokenizers

| Tokenizer | Algorithm | Dictionary | Speed | Dict memory |
|-----------|-----------|------------|-------|-------------|
| `NewmmTokenizer` | Maximal matching + TCC | `TrieChar` | fastest | ~43 MB |
| `NewmmFstTokenizer` | Maximal matching + TCC | `FstDict` | moderate | ~0.85 MB |
| `DeepcutTokenizer` | CNN / ONNX | bundled model | slowest | fixed |

All three implement the same [`Tokenizer`] trait:

```rust
pub trait Tokenizer {
    fn segment(&self, text: &str, safe: bool, parallel: bool) -> AnyResult<Vec<String>>;
    fn segment_to_string(&self, text: &str, safe: bool, parallel: bool) -> Vec<String>;
}
```

Switching tokenizers is a single-line change:

```rust
let tok: Box<dyn Tokenizer> = Box::new(NewmmTokenizer::new("words_th.txt"));
// or:
let tok: Box<dyn Tokenizer> = Box::new(NewmmFstTokenizer::new("words_th.txt"));
// or (requires deepcut feature):
let tok: Box<dyn Tokenizer> = Box::new(DeepcutTokenizer::new()?);
```

## Benchmark environment

- Rust: stable (release profile, `lto = true`, `codegen-units = 1`)
- Criterion.rs: 0.8
- Dictionary: `words_th.txt`, 62 018 words
- Text sizes:
  - **short** – 28 Unicode characters, Thai-only
  - **medium** – 219 characters, mixed Thai / Latin / CJK / digits
  - **long** – 937 characters, mixed

---

## 1. Dictionary construction

| Implementation | Time for 62 018 words |
|---|---:|
| `TrieChar::new` | **33.7 ms** |
| `FstDict::from_words` | 64.4 ms |

`TrieChar::new` is now ~1.9× faster than `FstDict::from_words`. The speedup
compared with the previous version (65.3 ms) comes from removing the parallel
`HashSet<String>` insert — each word now requires only one trie traversal
instead of a trie insert plus a hash insert (with heap allocation and hashing).

---

## 2. Dictionary prefix lookup

Finding all dictionary entries that are prefixes of a query string.

| Implementation | short_thai (5 chars) | mixed (7 chars) | medium_thai (14 chars) |
|---|---:|---:|---:|
| `TrieChar::prefix_ref` | **71 ns** | **79 ns** | **97 ns** |
| `FstDict::prefix_lengths` | 4 112 ns | 2 233 ns | 5 055 ns |
| Ratio | **58× faster** | **28× faster** | **52× faster** |

`TrieChar` wins on lookup speed because it navigates a `HashMap` per character
(O(k) pointer chasing, cache-friendly for short words). `FstDict`
runs a streaming FST search with higher per-call overhead.

**Recommendation:** use `TrieChar` (`NewmmTokenizer`) for the hot tokenization
path; use `FstDict` (`NewmmFstTokenizer`) when memory is constrained.

---

## 3. Full end-to-end tokenization

| Tokenizer | short (28 ch) | medium (219 ch) | long (937 ch) |
|---|---:|---:|---:|
| `NewmmTokenizer` (safe=false) | **2.73 µs** | **25.6 µs** | **117 µs** |
| `NewmmTokenizer` (safe=true) | 2.74 µs | 28.6 µs | 166 µs |
| `NewmmFstTokenizer` (safe=false) | 28.9 µs | 277 µs | 2 337 µs |
| `NewmmFstTokenizer` (safe=true) | 29.1 µs | 245 µs | 1 530 µs |
| Speed ratio (Trie vs Fst, safe=false) | **11× faster** | **11× faster** | **20× faster** |

`DeepcutTokenizer` results require the `deepcut` Cargo feature (`--features deepcut`).
CNN/ONNX inference is significantly slower than dictionary-based methods and
scales with input length at a different rate.

**Conclusions:**

- `NewmmTokenizer` is the fastest dictionary-based tokenizer. The `TrieChar`
  prefix-lookup loop is ~28–58× faster per query than `FstDict`.
- `NewmmFstTokenizer` is the memory-efficient alternative: 49× smaller
  dictionary with 11–20× lower throughput.
- The `Tokenizer` trait makes switching between all three tokenizers trivial.

---

## 4. Memory footprint

### String representation

| Representation | Heap bytes per character |
|---|---:|
| `CharString` (UTF-8 source + `u32` position table) | **6.3 bytes/char** |

Thai characters are 3-byte UTF-8 sequences (3 bytes) plus one `u32` position
entry (4 bytes) = ~7 bytes/char for Thai, less for ASCII.

### Dictionary storage

| Structure | Total bytes | Per word |
|---|---:|---:|
| `FstDict` (FST automaton) | 891 464 bytes (~0.85 MB) | **14.4 bytes** |
| `TrieChar` (trie with HashMap nodes, estimated) | ~43 MB | **~699 bytes** |
| Reduction | | **~49× smaller** |

The FST stores the full 62 018-word Thai dictionary in under 1 MB. The
`TrieChar` trie stores roughly 80 bytes per character edge in `HashMap`
entries. For memory-constrained deployments, `FstDict` is the preferred
choice.

### DeepcutTokenizer model

The bundled deepcut ONNX model (`model/deepcut.onnx`) is approximately
**3.9 MB** compiled into the binary. There is no runtime dictionary.

---

## 5. TrieChar: before and after removing the duplicate word store

The `TrieChar` implementation previously kept a parallel `HashSet<String>` copy
of every word alongside the trie nodes. This was removed so that words are
stored exclusively in the trie structure. A single `word_count: usize` counter
replaces `HashSet::len()`.

### Memory savings (62 018 words)

| Component | Before | After | Saved |
|---|---:|---:|---:|
| `HashSet<String>` entries | ~5.7 MB | 0 bytes | **~5.7 MB** |
| Trie nodes | ~43 MB | ~43 MB | — |
| **Total `TrieChar`** | **~49 MB** | **~43 MB** | **~5.7 MB (≈12%)** |

Each removed entry was approximately 92 bytes: 24 bytes (`String` header on the
stack) + ~12 bytes (average UTF-8 heap content for a 4-char Thai word) +
~56 bytes (`FxHashSet` bucket overhead).

### Construction speed

| | Before | After | Change |
|---|---:|---:|---:|
| `TrieChar::new` (62 018 words) | 65.3 ms | **33.7 ms** | **−1.9× faster** |

Removing the `HashSet` insert cuts the per-word work roughly in half: each word
now requires only one trie traversal (O(k)) instead of a trie insert plus
a heap allocation, UTF-8 copy, and hash insert.

### Lookup-path impact

`contain()` and the existence check in `add()`/`remove()` now walk the trie
(O(k)) instead of doing an O(1) hash lookup. In practice this is not on the
hot tokenization path — `prefix_ref` is — so end-to-end throughput is
unchanged.

---

## Summary

| Tokenizer | Speed (short/long) | Dict memory | Use when |
|-----------|-------------------|-------------|----------|
| `NewmmTokenizer` | **2.7 µs / 117 µs** | ~43 MB | Maximum throughput |
| `NewmmFstTokenizer` | 28.9 µs / 2 337 µs | **~0.85 MB** | Memory-constrained |
| `DeepcutTokenizer` | slower (ONNX) | ~3.9 MB model | No dictionary available |

All three implement `Tokenizer` and are interchangeable. Use `Box<dyn Tokenizer>`
to select the tokenizer at runtime.
