---
SPDX-FileCopyrightText: 2024-2026 PyThaiNLP Project
SPDX-License-Identifier: Apache-2.0
---

# Benchmark results: dictionary backends and tokenizers compared

Updated on: 2026-04-12

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

| File | Size | Chars | Space ratio | OOV profile |
| ---- | ---: | ----: | ----------: | ----------- |
| `text-wikipedia-s.txt` | ~2.4 KB | 847 | 3.5% | Wikipedia Thai/Latin mix |
| `text-wikipedia-m.txt` | ~41 KB | 14 470 | 4.4% | Wikipedia Thai/Latin mix |
| `text-wikipedia-l.txt` | ~615 KB | 226 287 | 4.7% | Wikipedia Thai/Latin mix |
| `text-only-dict-10k-words.txt` | ~1 MB | 338 156 | **0.9%** | Only words from `dict-10k` — low OOV when using `dict-10k` |
| `text-only-dict-10k-1k-words.txt` | ~1 MB | 336 169 | **0.7%** | Mix of `dict-10k` + `dict-1k-*` words — moderate OOV for `dict-10k` |
| `text-ws-social.txt` | ~6.3 MB | 2 455 505 | 6.2% | Social-media posts — high OOV |

> **Space ratio** = spaces ÷ non-space characters.  Thai text uses spaces only
> between phrases, not between every word, so even natural text has a low ratio.
>
> `text-only-dict-10k-words.txt` and `text-only-dict-10k-1k-words.txt` have an
> unusually low space ratio (0.7–0.9%) because they are synthetically generated
> by concatenating dictionary words with minimal spacing.  This produces very
> long runs of Thai characters between spaces — effectively long "sentences"
> with no embedded word boundaries.  The Newmm algorithm builds a graph of all
> possible word-boundary positions before choosing the best path; a long run of
> Thai characters with many overlapping prefix matches creates a large graph,
> increasing tokenizer work per byte compared to the Wikipedia texts.
>
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

*Run `cargo bench -- dict_construction` to reproduce these results.*

**Findings:** `TrieChar` is the fastest to build. `TrieCharLegacy` adds ~9% overhead across all sizes (hash table maintenance during insertion). `FstDict` is 1.9–2.5× slower than `TrieChar` due to the sort pass and FST automaton construction, but the 62k build (32.6 ms) is still fast enough to be practical at startup.

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

*Run `cargo bench -- prefix_lookup` to reproduce these results.*

**Findings:** `TrieChar` and `TrieCharLegacy` are within 1–2 ns of each other at every query length — the `TrieCharLegacy` hash table is not on the prefix-lookup path. `FstDict` is 22–42× slower per query (3 bytes/Thai char × byte-level FST traversal). This per-query penalty compounds across every character position during tokenization, explaining FstDict's collapse on large texts.

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

*Run `cargo bench -- dict_operations` to reproduce these results.*

**Findings — contain:** `TrieCharLegacy` O(1) hash is ~24% faster than `TrieChar`'s O(k) trie walk (~79 ns vs ~104 ns), consistent across all dict sizes. `FstDict` is fastest at 1k words (47–59 ns) but approaches trie speeds at 62k (89 ns).

**Findings — add/remove:** Trie `add`/`remove` times are dominated by the **clone cost** (O(n·k) node copies). `FstDict::add`/`remove` clone only a byte vector plus a small delta `HashSet`, making them 280–650× faster at 62k words (12.7 µs vs 8.2–10.5 ms).

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

*Run `cargo bench -- dict_backend_tokenization` to reproduce these results.*

**Findings:** `TrieChar` and `TrieCharLegacy` produce identical end-to-end times — the hot path (`prefix_ref`) is shared. `FstDict` is 12× slower on `wikipedia-s` (852 µs vs 70 µs) and 83× slower on `wikipedia-m` (129 ms vs 1.56 ms). The gap widens with text size because OOV characters trigger per-byte FST fallback at every position. Larger dictionaries add a modest latency increase for trie backends (47 µs → 70 µs from 1k to 62k words) with no change to FstDict's relative penalty.

---

## 6. Tokenizer performance (Set 2)

`NewmmTokenizer` (TrieChar) and `NewmmLegacyTokenizer` (TrieCharLegacy)
on larger texts.  `DeepcutTokenizer` included with `--features deepcut`.
`FstDict`/`NewmmFstTokenizer` excluded (impractically slow on large text).

### Run A — dict-10k

| Tokenizer | text-only-10k (low OOV) | text-only-10k-1k (mod. OOV) | wikipedia-l |
| --------- | ----------------------: | --------------------------: | ----------: |
| `NewmmTokenizer` (TrieChar) | 134.49 ms | 127.89 ms | 48.001 ms |
| `NewmmLegacyTokenizer` (TrieCharLegacy) | 146.10 ms | 125.78 ms | 47.010 ms |
| `DeepcutTokenizer` | 305.69 s | 305.15 s | 189.79 s |

### Run B — dict-words-th

| Tokenizer | text-only-10k (low OOV) | text-only-10k-1k (low OOV) | wikipedia-l | ws-social (high OOV) |
| --------- | ----------------------: | -------------------------: | ----------: | -------------------: |
| `NewmmTokenizer` (TrieChar) | 151.36 ms | 146.48 ms | 68.517 ms | 9.8367 s |
| `NewmmLegacyTokenizer` (TrieCharLegacy) | 150.75 ms | 141.02 ms | 65.811 ms | 10.367 s |
| `DeepcutTokenizer` | 308.69 s | 301.65 s | 295.55 s | n/a (OOM) |

*Run `cargo bench -- tokenizer_performance` to reproduce these results.*

**Findings — Newmm throughput:**

| Text | Size | Dict | Throughput |
| ---- | ---: | ---- | ---------: |
| `text-only-dict-10k-words.txt` | ~1 MB | 10k | ~7.4 MB/s |
| `text-only-dict-10k-1k-words.txt` | ~1 MB | 10k | ~7.8 MB/s |
| `text-wikipedia-l.txt` | ~615 KB | 10k | ~12.8 MB/s |
| `text-only-dict-10k-words.txt` | ~1 MB | 62k | ~6.6 MB/s |
| `text-only-dict-10k-1k-words.txt` | ~1 MB | 62k | ~6.8 MB/s |
| `text-wikipedia-l.txt` | ~615 KB | 62k | ~9.0 MB/s |
| `text-ws-social.txt` | ~6.3 MB | 62k | ~0.64 MB/s |

`NewmmTokenizer` and `NewmmLegacyTokenizer` are within 1–9% of each other across all inputs — the trie traversal path is shared.

`wikipedia-l` achieves higher throughput than the synthetic 1 MB texts because its Thai/Latin mix allows fast ASCII passthrough on Latin characters.

Switching from `dict-10k` to `dict-words-th` costs ~11% on low-OOV Thai text (7.4 → 6.6 MB/s) due to deeper trie traversal in the larger dictionary.

High OOV (`ws-social`) drops throughput by ~90% (6.6 → 0.64 MB/s): the character-level fallback in Newmm fires on every unknown token, turning linear trie walks into per-character scans.

**Findings — DeepcutTokenizer:** ~3.2 KB/s on 1 MB Thai text — approximately **2 300× slower** than `NewmmTokenizer`. The `ws-social` benchmark (6.3 MB) was terminated by the OS (SIGKILL) during Criterion warmup; the ONNX model's memory pressure at that scale exceeded available RAM. `DeepcutTokenizer` is intended for accuracy-critical, latency-tolerant workloads on short to medium texts.

---

## 7. TrieChar vs TrieCharLegacy: summary

| Property | `TrieChar` (default) | `TrieCharLegacy` |
| -------- | -------------------- | ---------------- |
| `prefix_ref()` speed | 45.6 ns (62k, mixed) | 45.6 ns — identical |
| End-to-end tokenization | 1.564 ms (62k, wiki-m) | 1.563 ms — identical |
| `contain()` | ~104 ns (62k) | **~79 ns — 24% faster** |
| Memory (62k words) | **~43 MB** | ~49 MB (+14%) |
| Construction (62k) | **26.0 ms** | 28.3 ms (+9%) |
| `add()` / `remove()` (clone+mutate, 62k) | ~8–9 ms | ~10–11 ms |
| `add()` / `remove()` (clone+mutate, 1k) | ~141–342 µs | ~165–368 µs |
| Clone cost | O(n·k) deep copy | O(n·k) + O(n) strings |

**Use `TrieChar`** for all production tokenization. It is the default.

**Use `TrieCharLegacy`** only if your application calls `contain()` heavily
and the 12% extra memory is acceptable.

---

## 8. Parallel tokenizer performance (Set 3)

Parallel chunked segmentation × 3 chunk sizes × 2 large texts.
Dictionary: `dict-words-th` (62k words) for all rows.
Chunk sizes: 32 KB (`DEFAULT/2`), 64 KB (`DEFAULT`), 128 KB (`DEFAULT×2`).

All three backends split the text at TCC (Thai Character Cluster) boundaries so
chunks are always valid Unicode.  The chunk-size knob trades off scheduling
overhead against cache locality and thread-fan-out.

*Run individually:*

```sh
cargo bench -- tokenizer_parallel_newmm
cargo bench -- tokenizer_parallel_legacy
cargo bench --features deepcut -- tokenizer_parallel_deepcut
```

### 8-A NewmmTokenizer (TrieChar) — parallel

| Text | Size | Chunk 32 KB | Chunk 64 KB (default) | Chunk 128 KB |
| ---- | ---: | ----------: | --------------------: | -----------: |
| `text-wikipedia-l.txt` | ~615 KB | 21.380 ms (27.5 MiB/s) | 22.338 ms (26.3 MiB/s) | **20.088 ms (29.2 MiB/s)** |
| `text-ws-social.txt` | ~6.3 MB | 244.85 ms (24.6 MiB/s) | **235.75 ms (25.6 MiB/s)** | 238.00 ms (25.3 MiB/s) |

### 8-B NewmmLegacyTokenizer (TrieCharLegacy) — parallel

| Text | Size | Chunk 32 KB | Chunk 64 KB (default) | Chunk 128 KB |
| ---- | ---: | ----------: | --------------------: | -----------: |
| `text-wikipedia-l.txt` | ~615 KB | 21.788 ms (27.0 MiB/s) | 22.564 ms (26.0 MiB/s) | **18.941 ms (31.0 MiB/s)** |
| `text-ws-social.txt` | ~6.3 MB | 237.34 ms (25.4 MiB/s) | **231.95 ms (26.0 MiB/s)** | 235.25 ms (25.6 MiB/s) |

### 8-C DeepcutTokenizer — parallel (requires `--features deepcut`)

| Text | Size | Chunk 32 KB | Chunk 64 KB (default) | Chunk 128 KB |
| ---- | ---: | ----------: | --------------------: | -----------: |
| `text-wikipedia-l.txt` | ~615 KB | **41.687 s (14.4 KiB/s)** | 99.484 s (6.0 KiB/s) | 126.73 s (4.7 KiB/s) |
| `text-ws-social.txt` | ~6.3 MB | **615.41 s (10.0 KiB/s)** | 1207.7 s (5.1 KiB/s) | OOM / SIGKILL |

> **Note — OOM kill on ws-social/chunk-128k**: The 6.3 MB text split into 128 KB chunks
> yields ~50 concurrent ONNX inference sessions.  Each session holds intermediate tensors
> in memory; at this concurrency level the process was killed by the OS (SIGKILL / signal 9).
> Smaller chunk sizes are preferred for very large texts with DeepcutTokenizer.
>
> **Key trend**: smaller chunks → more parallelism → faster for `wikipedia-l` (where
> CPU is the bottleneck), but for `ws-social` smaller chunks degrade throughput because
> more context boundaries interrupt the neural model's TCC-level decisions.  The 32 KB
> chunk is best for `ws-social` despite its extreme variance (p50: 615 s, range 503–750 s).

---

## 9. Space ratio and OOV effects (Set 4)

This set isolates two independent text properties that affect tokenizer throughput:

1. **Space ratio** — how often whitespace appears, which controls the size of each
   segment the tokenizer must process.  Larger segments → larger lattice → more work.
2. **Vocabulary OOV** — what fraction of source words are absent from the tokenizer's
   dictionary, forcing the algorithm to fall back to character-level or sub-word paths.

All texts are exactly **512 KB**, generated by `build_tools/generate_bench_texts.py`.
Tokenizer dictionary: `dict-words-th` (62k words).
Parallel chunk size: `DEFAULT` (64 KB).

### Text input characteristics

Spaces are inserted every `k` Unicode code points (character boundary), so at high
ratios many space-delimited chunks are partial words and are OOV even in zero-OOV
vocabularies.  The table below shows word-level OOV (the vocabulary fraction) and
the resulting chunk-level OOV observed in the generated files.

| File prefix | Vocab source | Avg word | Vocab OOV | Space ratio |
| ----------- | ------------ | -------: | --------: | ----------: |
| `text-only-dict-10k-space-*` | dict-10k + punct | ~6 chars | ~0 % | ~1 / ~5 / ~10 % |
| `text-only-dict-10k-1k-space-*` | 25 % × 4 dicts + punct | ~7 chars (variable) | ~11 % | ~1 / ~5 / ~10 % |
| `text-only-dict-1k-long-space-*` | dict-1k-long + punct | ~12 chars | ~26 % | ~1 / ~5 / ~10 % |
| `text-only-dict-1k-short-space-*`| dict-1k-short + punct | ~6 chars | ~16 % | ~1 / ~5 / ~10 % |

Planned comparisons:
- **Set 1 vs Set 4** (same ~6-char word length, OOV 0 % vs 16 %) → isolates OOV effect
- **Set 3 vs Set 4** (similar OOV range, ~12 vs ~6 chars) → isolates word-length effect
- **Space-01 vs Space-05 vs Space-10** within each set → isolates space-ratio effect

*Run individually:*

```sh
cargo bench -- tokenizer_space_ratio_newmm
cargo bench -- tokenizer_space_ratio_legacy
cargo bench --features deepcut -- tokenizer_space_ratio_deepcut
```

### 9-A NewmmTokenizer (TrieChar) — space ratio & OOV

All texts are 512 KB; parallel chunk size = DEFAULT (64 KB); dict = `dict-words-th`.
Bold = fastest within each vocabulary set.

| Text set | Vocab OOV | Space % | Median time | Throughput |
| -------- | --------: | ------: | ----------: | ---------: |
| `10k/space-01` | ~0 % | ~1 % | **27.6 ms** | **18.1 MiB/s** |
| `10k/space-05` | ~0 % | ~5 % | 34.3 ms | 14.6 MiB/s |
| `10k/space-10` | ~0 % | ~10 % | 43.1 ms | 11.6 MiB/s |
| `10k-1k/space-01` | ~11 % | ~1 % | **27.7 ms** | **18.1 MiB/s** |
| `10k-1k/space-05` | ~11 % | ~5 % | 43.6 ms | 11.5 MiB/s |
| `10k-1k/space-10` | ~11 % | ~10 % | 42.4 ms | 11.8 MiB/s |
| `1k-long/space-01` | ~26 % | ~1 % | **40.0 ms** | **12.5 MiB/s** |
| `1k-long/space-05` | ~26 % | ~5 % | 57.4 ms | 8.71 MiB/s |
| `1k-long/space-10` | ~26 % | ~10 % | 73.6 ms | 6.80 MiB/s |
| `1k-short/space-01` | ~16 % | ~1 % | **63.6 ms** | **7.86 MiB/s** |
| `1k-short/space-05` | ~16 % | ~5 % | 75.3 ms | 6.64 MiB/s |
| `1k-short/space-10` | ~16 % | ~10 % | 70.0 ms | 7.15 MiB/s |

### 9-B NewmmLegacyTokenizer (TrieCharLegacy) — space ratio & OOV

| Text set | Vocab OOV | Space % | Median time | Throughput |
| -------- | --------: | ------: | ----------: | ---------: |
| `10k/space-01` | ~0 % | ~1 % | **49.8 ms** | **10.0 MiB/s** |
| `10k/space-05` | ~0 % | ~5 % | 71.7 ms | 6.97 MiB/s |
| `10k/space-10` | ~0 % | ~10 % | 78.3 ms | 6.38 MiB/s |
| `10k-1k/space-01` | ~11 % | ~1 % | **67.3 ms** | **7.43 MiB/s** |
| `10k-1k/space-05` | ~11 % | ~5 % | 75.7 ms | 6.61 MiB/s |
| `10k-1k/space-10` | ~11 % | ~10 % | 163.2 ms | 3.06 MiB/s |
| `1k-long/space-01` | ~26 % | ~1 % | **131.9 ms** | **3.79 MiB/s** |
| `1k-long/space-05` | ~26 % | ~5 % | 164.9 ms | 3.03 MiB/s |
| `1k-long/space-10` | ~26 % | ~10 % | 198.3 ms | 2.52 MiB/s |
| `1k-short/space-01` | ~16 % | ~1 % | **160.0 ms** | **3.13 MiB/s** |
| `1k-short/space-05` | ~16 % | ~5 % | 195.7 ms | 2.56 MiB/s |
| `1k-short/space-10` | ~16 % | ~10 % | 241.8 ms | 2.07 MiB/s |

### 9-C DeepcutTokenizer — space ratio (requires `--features deepcut`)

DeepcutTokenizer does not use a dictionary, so vocabulary OOV has no effect on its
behaviour.  Only the space-ratio dimension is of interest.

From the parallel benchmark (Section 8-C), `DeepcutTokenizer` on `wikipedia-l` (~615 KB,
low space ratio) with a 32 KB chunk size processes at roughly **14 KiB/s** — around
2 000× slower than `NewmmTokenizer`.  Neural inference cost per token dominates
completely; space ratio and OOV are irrelevant to its throughput.

Full space-ratio sweeps for DeepcutTokenizer were not collected (each of the 12 texts
would require ~7 minutes per benchmark run and risks OOM at higher chunk sizes).
`DeepcutTokenizer` is recommended only for accuracy-critical tasks on short texts where
dictionary-based tokenizers are insufficient.

### 9 Findings

**Space ratio effect (Set 1, vocab OOV ≈ 0 %):**
Higher space ratio consistently degrades throughput for both tokenizers:
18.1 → 14.6 → 11.6 MiB/s (NewmmTokenizer, space-01 → 05 → 10).
This is counterintuitive at first: more spaces means shorter segments, so why is it
slower?  The cause is the generation strategy: spaces are placed at Unicode *character*
boundaries (not word boundaries), so at high ratios many space-delimited chunks are
partial words, which are OOV at the chunk level.  The tokenizer must then apply
sub-word / character-level fallback for each such chunk, multiplying the per-byte cost.

**OOV effect (comparing sets at the same ~1 % space ratio):**

| Set | Vocab OOV | NewmmTokenizer | NewmmLegacyTokenizer |
|-----|----------:|---------------:|---------------------:|
| 10k/space-01 | ~0 % | 18.1 MiB/s | 10.0 MiB/s |
| 10k-1k/space-01 | ~11 % | 18.1 MiB/s | 7.43 MiB/s |
| 1k-long/space-01 | ~26 % | 12.5 MiB/s | 3.79 MiB/s |
| 1k-short/space-01 | ~16 % | 7.86 MiB/s | 3.13 MiB/s |

`NewmmTokenizer` is resilient to moderate OOV: 0 % → 11 % OOV shows no measurable
change (18.1 MiB/s in both cases).  Only at higher OOV (26 %, 16 %) does throughput
drop, with the severity depending on word length (see below).

**Word-length effect (Set 3 long vs Set 4 short, controlled for similar OOV):**
`1k-long/space-01` (26 % OOV, ~12-char words) runs at **12.5 MiB/s**, while
`1k-short/space-01` (16 % OOV, ~6-char words) runs at only **7.86 MiB/s** — i.e., the
set with *shorter* words and *lower* OOV is significantly slower.  Why?
Long words, even when OOV, produce a single long unresolved span that the trie can
reject with one O(k) prefix walk.  Short OOV words create many small spans, each
requiring independent trie traversal, and their short length increases the probability
of false-positive prefix matches that lead to backtracking.

**NewmmLegacyTokenizer regression on synthetic texts:**
On natural texts (Section 8), `NewmmLegacyTokenizer` and `NewmmTokenizer` achieve
nearly identical throughput (~27 MiB/s for `wikipedia-l`).  On the synthetic texts
here, `NewmmLegacyTokenizer` is **1.8–3.9× slower** and the gap widens with OOV.
The likely cause: for OOV or partial-word chunks, `contain()` is called frequently
with words that are absent from the dictionary.  `TrieCharLegacy` uses a hash set
for `contain()`; failed lookups touch a different hash bucket each time, causing cache
misses.  `TrieChar`'s trie shares prefix paths, so failed lookups reuse warm cache
lines.  This effect is invisible on natural text (which has low OOV and few short
chunks) but dominates on the high-OOV, high-space-ratio synthetic data.

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
