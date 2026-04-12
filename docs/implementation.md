---
SPDX-FileCopyrightText: 2026 PyThaiNLP Project
SPDX-License-Identifier: CC0-1.0
---

# Implementation notes

This document stores implementation details and design decisions that are not
intended for end-user release notes.

## API redesign summary (v2 line)

- The tokenizer API was normalized around explicit tokenizer objects.
- Core trait methods were simplified to text-only calls for default behavior.
- Advanced controls were moved to explicit option-bearing methods.
- Node.js and Python bindings migrated from global registry helpers to
  class-based APIs.

## Tokenizer families

- NewMM (Trie backend): default dictionary tokenizer, optimized for speed.
- NewMM (FST backend): same algorithm with reduced dictionary memory.
- Deepcut: neural tokenizer backed by ONNX inference.

## Parallel segmentation design

- Parallel controls use chunk-based processing.
- User-facing explicit control is exposed through `parallel_chunk_size`.
- Ergonomic `segment_parallel(...)` methods compute chunk size automatically
  and call the same underlying options path.

### Auto chunk-size heuristic

Current heuristic inputs:

- Input byte length.
- Runtime available parallelism from `std::thread::available_parallelism()`.

Current safeguards:

- Bound target chunk size to a min/max range.
- Limit target chunk fan-out.
- Disable parallel mode for small input.

## One-chunk rule

- If total input length is below `MIN_CHUNK_SIZE * 2`, processing remains
  single-chunk.
- This rule applies in auto mode and in parallelization gating logic.

Rationale:

- Splitting small text usually adds overhead without throughput benefit.
- It reduces extra allocations and scheduling overhead.

## UTF-8 chunk safety

- Chunk search windows are aligned to UTF-8 character boundaries before slicing.
- Boundary correction uses `is_char_boundary()` checks.
- This prevents slicing in the middle of a multi-byte character.

## Split-point strategy

Chunk boundaries prefer:

1. Sentence-ending punctuation.
2. Whitespace.
3. Valid Thai Character Cluster boundaries.

Fallback:

- Use nearest safe boundary to preserve forward progress.

## Chunk boundary behavior

When `parallel_chunk_size` is set, text is split into chunks before
tokenization. Token sequences near chunk boundaries can differ from
full-text tokenization.

Reasons for the divergence:

- **NewMM:** dictionary matching at the boundary may prefer different token
  paths when the surrounding context changes between full-text and chunked
  processing.
- **Deepcut:** the CNN model uses a fixed-width context window (21 characters).
  Characters near chunk boundaries have fewer adjacent context characters from
  the neighboring chunk, which shifts model predictions.

The chunked output is acceptable for tasks that treat text holistically, such
as text classification and word embedding. It may not be suitable for tasks
that require precise linguistic unit identification.

### Future direction: fine-grained chunk merging

Smooth boundary stitching is a potential future improvement. For
dictionary-based tokenizers (NewMM), a partial implementation exists in the
safe-mode boundary-scanning logic (`TEXT_SCAN_LEFT`/`TEXT_SCAN_RIGHT` window),
but it trades performance for accuracy, and enabling it on the chunk boundary
alone may degrade overall throughput. The boundary window size and the chunk
size ratio that yield negligible accuracy loss have not been established. A
future implementation would need to measure this trade-off to find practical
default settings.

## NewMM safety and ambiguity behavior

- Safe mode remains available for highly ambiguous text.
- Previous BFS path explosion risk was mitigated with visited-set controls.

## Concurrency model

- Tokenizer instances are designed for read-heavy concurrent usage.
- Dictionary structures are shared where possible.
- Mutation methods use copy-on-write behavior when shared ownership exists.

## Dictionary backends

Three backends implement `DictBackend` and are interchangeable in
`NewmmTokenizer<D>`.

### TrieChar (default)

- Structure: `HashMap<char, TrieNode>` tree rooted at a single root node.
- Word storage: exclusively in the trie paths.  No separate word list is kept.
- `word_count: usize` tracks the number of distinct entries.
- `contain()` walks the trie O(k); no separate membership data structure.
- `iterate()` does a depth-first traversal collecting all end-flagged paths.
- Memory: ~43 MB for 62 018 words (~699 bytes/word, dominated by `HashMap`
  node overhead of ~80 bytes per character edge).

### TrieCharLegacy

- Same trie structure as `TrieChar`.
- Also keeps a `HashSet<String>` parallel word store.
- `contain()` is O(1) via the `HashSet` (vs O(k) trie walk in `TrieChar`).
- `iterate()` returns `HashSet::iter()` (unspecified order).
- Memory overhead: ~92 bytes/word extra for the `HashSet` entries
  (~49 MB total for 62 018 words, +12% vs `TrieChar`).
- Construction is ~50% slower than `TrieChar` (adds a heap alloc + hash
  insert per word on top of the trie insert).
- `contain()` speed advantage has no effect on tokenization throughput because
  `contain()` is not on the hot path — `prefix_ref()` is.
- Kept as `TrieCharLegacy` to enable direct comparison with the optimized
  `TrieChar` and as a fallback for workloads that call `contain()` heavily.

### FstDict

- Structure: `fst::Set<Vec<u8>>` — a minimized finite-state automaton.
- Immutable base set; dynamic add/remove use small delta `HashSet`s.
- Memory: ~0.85 MB for 62 018 words (~14 bytes/word, ~49× smaller than trie).
- `prefix_lengths()` streams the FST byte by byte: O(k·B) where B = 3 for Thai.
- Add/remove are O(1) delta-set operations (no FST rebuild).
- Clone clones a ~0.85 MB byte vector, much cheaper than cloning a full trie.

### Choosing a backend

| Scenario | Recommended backend |
|----------|---------------------|
| General tokenization | `TrieChar` (default) |
| Memory-constrained deployment | `FstDict` |
| Frequent `contain()` calls outside tokenization | `TrieCharLegacy` |
| Frequent dynamic add/remove | `FstDict` |
