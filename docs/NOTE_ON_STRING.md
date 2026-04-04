---
SPDX-FileContributor: Thanathip Suntorntip Gorlph
SPDX-FileContributor: PyThaiNLP Project
SPDX-FileCopyrightText: 2024-2026 PyThaiNLP Project
SPDX-License-Identifier: Apache-2.0
---

# String Representation in the Tokenizer

## Previous design: custom four-byte encoding

Rust `String` (and `&str`) is a slice of valid UTF-8 bytes, which are
variable-length. Accessing the *n*-th character by index takes O(n) time,
making any algorithm that relies on character positions very slow.

The original implementation by Thanathip Suntorntip worked around this by
converting every character to a fixed 4-byte slot padded with leading zeros
(`fixed_bytes_str`):

- 1-byte ASCII: `[0, 0, 0, byte]`
- 2-byte UTF-8: `[0, 0, b1, b2]`
- 3-byte UTF-8 (Thai): `[0, b1, b2, b3]`
- 4-byte UTF-8: `[b1, b2, b3, b4]`

This gave O(1) character indexing (`bytes[i*4 .. (i+1)*4]`) and direct
`regex::bytes::Regex` matching, but required:

- A complex 4-byte encoding/decoding layer.
- A separate `custom_regex.rs` module that rewrote human-readable Unicode
  regex patterns into 4-byte padded patterns.
- Approximately 8 bytes per character (4-byte buffer + `Vec<char>` cache).

## Current design: native UTF-8 with byte-position index

The `CharString` type in `src/char_string.rs` stores:

- `Arc<String>` — the original UTF-8 source (shared across substrings).
- `Arc<Vec<u32>>` — a byte-position table: `byte_positions[i]` is the byte
  offset of character `i` in the source string.
- `start` and `end` — character indices defining the current view.

Key properties:

| Operation | Complexity |
|---|---|
| Character access by index | O(1) — one table lookup + ≤4 bytes decode |
| Character count | O(1) — `end - start` |
| Substring (view) | O(1) — copy Arc pointers + new start/end |
| `as_str()` for regex matching | O(1) — one table lookup, no allocation |

Memory footprint: ~3 bytes/char (UTF-8 Thai) + 4 bytes/char (u32 position)
= ~7 bytes/char, down from ~8 bytes/char in the old scheme. The real gain is
in code simplicity and the elimination of the encoding layer.

## Regex: native Unicode

The TCC rules (`tcc_rules.rs`) and non-Thai patterns (`newmm.rs`) are now
compiled with `regex::Regex` (Unicode mode) using plain UTF-8 patterns such
as `r"^[ก-ฮ]"`. The `custom_regex.rs` module and its `\x00` padding approach
have been removed.

> **Note on verbose mode (`(?x)`):** In Rust's `regex` crate, the `(?x)` flag
> ignores whitespace *including inside character classes*. For example,
> `(?x)^[ \t]+` would drop the literal space; use `(?x)^[\ \t]+` instead, or
> omit `(?x)` when the pattern contains meaningful whitespace.

## Dictionary: FST-backed (optional)

The `FstDict` type in `src/tokenizer/fst_dict.rs` provides a
memory-efficient alternative to `TrieChar` for storing the word list. It uses
the `fst` crate (finite state transducers) which stores sorted string sets in
a minimized DFA, achieving a few bytes per entry versus tens of bytes per
trie node.

`FstDict` supports:
- O(k) prefix lookup via `prefix_lengths(text)` (where k = text length).
- Dynamic add/remove operations via small delta `HashSet`s.

## Rust 2024 edition patterns

The codebase targets the **Rust 2024 edition** (`edition = "2024"`,
`rust-version = "1.88.0"`) and exploits several features that edition
stabilizes or refines.

### `let` chains

Complex nested `match` and `while match { true/false }` idioms are replaced
with `let` chains, which chain pattern conditions directly in `if` and `while`
guards:

```rust
// trie_char.rs — double-nested match → let chain
if let Some(node) = current_node
    && let Some(child) = node.find_child(ch)
{
    // …
}

// newmm.rs — while match { true / false } → while let && condition
while let Some(&begin_position) = position_list.peek()
    && begin_position < text_length
{
    // …
}
```

### Reference patterns

In the 2024 edition the ergonomics of reference patterns in `for` and
closure contexts are refined: you can write `&T` on the left-hand side of
a binding where the iterator yields `&T`, removing the need for an explicit
`*deref`:

```rust
// Before (explicit deref)
for position in idk {                 // position: &usize
    if *position != goal { … }
}

// After (reference pattern)
for &position in idk {                // position: usize — Copy, no deref needed
    if position != goal { … }
}

// Tuple reference pattern in a closure
entries.iter().filter_map(|&(s, idx)| { … })   // was |(s, idx)| { … *idx … }
```

### `Copy` types by value

Small `Copy` types (such as `char`) are now passed by value rather than by
reference, avoiding an indirection:

```rust
// trie_char.rs
fn find_child(&self, ch: char) -> Option<&Self>        // was &char
fn find_mut_child(&mut self, ch: char) -> Option<…>    // was &char
fn remove_child(&mut self, ch: char)                   // was &char
```

### `entry` API and iterator chains

Duplicated `match get_mut / insert` blocks are replaced with idiomatic
`HashMap::entry` calls; manual accumulator loops are replaced with
functional iterator chains:

```rust
// entry().or_default() replaces match + insert
graph.entry(begin_position).or_default().push(end_position_candidate);

// filter + last + map replaces a manual for loop
text.chars()
    .enumerate()
    .filter(|&(_, ch)| ch == ' ')
    .last()
    .map(|(i, _)| i)
```

### `unsafe` in hot paths — `SAFETY` comments

`fst_dict.rs` has one intentional `unsafe` block in the `prefix_lengths` hot
loop:

```rust
// SAFETY: All keys in the FST were inserted as valid UTF-8 strings in
// `from_words`. The FST stores raw bytes verbatim and never modifies them,
// so every key is guaranteed to be valid UTF-8. Using the unchecked variant
// avoids a redundant byte-scan on every iteration of this hot lookup loop.
let word = unsafe { std::str::from_utf8_unchecked(key) };
```

The invariant is established once at construction time in `from_words()`, which
accepts `&str` inputs (already valid UTF-8) and inserts their raw bytes into
the FST unchanged. Using `from_utf8_unchecked` in the hot lookup loop avoids
re-scanning the same bytes on every matching key, which would regress
`FstDict::prefix_lengths` performance.

## References

- [Rust String indexing](https://doc.rust-lang.org/book/ch08-02-strings.html#indexing-into-strings)
- [UTF-8 on Wikipedia](https://en.wikipedia.org/wiki/UTF-8)
- [`fst` crate documentation](https://docs.rs/fst/)
- [Rust 2024 edition guide](https://doc.rust-lang.org/edition-guide/rust-2024/)
- [RFC 3318 — `let` chains](https://github.com/rust-lang/rfcs/pull/3318)
- [RFC 3627 — match ergonomics 2024](https://github.com/rust-lang/rfcs/pull/3627)
