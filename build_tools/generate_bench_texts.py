#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2024-2026 PyThaiNLP Project
# SPDX-License-Identifier: Apache-2.0
"""
Generate synthetic benchmark texts with controlled space and OOV ratios.

Produces 12 files in tests/data/ (all exactly 512 KB), organised in 4 sets:

All sets include typical Thai punctuation and numbers (~7 % of tokens) to
mimic natural text.

  Dict Set 1 — dict-10k.txt + punct/numbers
    Avg word length ~6 chars.  All words in dict-words-th → vocab OOV ≈ 0 %.
    text-only-dict-10k-space-01.txt   ~1 % spaces
    text-only-dict-10k-space-05.txt   ~5 % spaces
    text-only-dict-10k-space-10.txt  ~10 % spaces

  Dict Set 2 — 25 % each from dict-10k, dict-1k-long, dict-1k-short,
                dict-200-common (mixed lengths ≈ 7 chars avg) + punct/numbers
    Vocab OOV ≈ 11 %.
    text-only-dict-10k-1k-space-01.txt   ~1 % spaces
    text-only-dict-10k-1k-space-05.txt   ~5 % spaces
    text-only-dict-10k-1k-space-10.txt  ~10 % spaces

  Dict Set 3 — dict-1k-long.txt + punct/numbers
    Uniform long words ≈ 12 chars.  Vocab OOV ≈ 26 %.
    text-only-dict-1k-long-space-01.txt   ~1 % spaces
    text-only-dict-1k-long-space-05.txt   ~5 % spaces
    text-only-dict-1k-long-space-10.txt  ~10 % spaces

  Dict Set 4 — dict-1k-short.txt + punct/numbers
    Uniform short words ≈ 6 chars.  Vocab OOV ≈ 16 %.
    text-only-dict-1k-short-space-01.txt   ~1 % spaces
    text-only-dict-1k-short-space-05.txt   ~5 % spaces
    text-only-dict-1k-short-space-10.txt  ~10 % spaces

Sets 3 and 4 are homogeneous-length vocabularies, which gives a fairer
distribution of words per space-delimited chunk at each space ratio.
This lets you isolate word-length effects (Set 3 vs 4) from OOV effects
(Set 1 vs 4, or Set 1 vs 3), independently of the mixed-length noise in
Set 2.

Spacing strategy:
  Spaces are inserted every k_chars Unicode code points (character boundary),
  where k_chars is derived from the target byte-level ratio assuming Thai
  characters average 3 UTF-8 bytes:

      k_chars = round((1 − r) / (r × 3))

  Targets and resulting k_chars:
      1 %  →  k_chars = 33   ≈ 5 long words / 5 short words between spaces
      5 %  →  k_chars =  6   ≈ 0.5 long words / 1 short word  between spaces
     10 %  →  k_chars =  3   ≈ 0.25 long / 0.5 short words    between spaces

  Because spacing is at character (not word) boundaries, short spaces slice
  words into partial sub-sequences, which are OOV even in Set 1 (zero-OOV
  vocabulary).  This is intentional: it tests tokenizer resilience against
  dense, unknown fragments at high space ratios.

Usage:
  python3 build_tools/generate_bench_texts.py [--data-dir tests/data]
"""

import argparse
import random
import statistics
import sys
from pathlib import Path

# ---------------------------------------------------------------------------
# Configuration
# ---------------------------------------------------------------------------

TARGET_SIZE = 512 * 1024        # 512 KB in bytes
SEED = 42
SPACE_RATIOS = [0.01, 0.05, 0.10]
APPROX_BPC = 3.0                # approx UTF-8 bytes per Thai code point

# Punctuations and digits for Dict Set 1
PUNCT_NUMBERS = [
    "1", "2", "3", "4", "5", "6", "7", "8", "9", "0",
    "๑", "๒", "๓", "๔", "๕", "๖", "๗", "๘", "๙", "๐",
    ",", ".", "!", "?", "(", ")", ":", ";",
    "ๆ", "ฯ", "ฯลฯ",
]
PUNCT_WEIGHT = 0.07  # fraction of pool occupied by punctuation slots


# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

def load_words(path: Path) -> list[str]:
    return [
        ln.strip()
        for ln in path.read_text("utf-8").splitlines()
        if ln.strip()
    ]


def repeat_to(lst: list, n: int) -> list:
    out: list = []
    while len(out) < n:
        out.extend(lst)
    return out[:n]


def k_chars_for_ratio(ratio: float) -> int:
    """Unicode chars of content between spaces to achieve target byte ratio."""
    return max(1, round((1.0 - ratio) / (ratio * APPROX_BPC)))


def build_sequence(
    pool: list[str], target_bytes: int, rng: random.Random
) -> list[str]:
    """Sample words until concatenated UTF-8 length ≥ target_bytes."""
    seq: list[str] = []
    total = 0
    while total < target_bytes:
        w = rng.choice(pool)
        seq.append(w)
        total += len(w.encode("utf-8"))
    return seq


def trim_to_utf8_boundary(data: bytearray, target: int) -> bytes:
    """Return data[:target] trimmed back to the last valid UTF-8 char start.

    Truncating a bytearray at an arbitrary byte offset can split a multi-byte
    UTF-8 sequence.  This function walks back from `target` until it finds a
    byte that is NOT a UTF-8 continuation byte (0x80–0xBF), then checks
    whether the remaining sequence is complete.  If not, it trims one more
    character.
    """
    out = data[:target]
    i = len(out) - 1
    # Walk back over continuation bytes (10xxxxxx)
    while i >= 0 and (out[i] & 0xC0) == 0x80:
        i -= 1
    if i < 0:
        return b""
    lead = out[i]
    if lead < 0x80:
        seq = 1       # ASCII
    elif lead < 0xE0:
        seq = 2       # 2-byte sequence
    elif lead < 0xF0:
        seq = 3       # 3-byte sequence (Thai falls here)
    else:
        seq = 4       # 4-byte sequence
    if i + seq <= len(out):
        return bytes(out)     # character is complete — no trim needed
    return bytes(out[:i])     # trim the incomplete character


def apply_spacing(
    word_seq: list[str], ratio: float, target_size: int
) -> bytes:
    """Emit characters from word_seq, inserting a space every k_chars chars.

    Returns exactly target_size bytes (trimmed to a valid UTF-8 boundary if
    needed, so the actual size may be up to 2 bytes less).
    """
    k = k_chars_for_ratio(ratio)
    result = bytearray()
    char_count = 0

    for word in word_seq:
        for ch in word:
            result.extend(ch.encode("utf-8"))
            char_count += 1
            if char_count >= k:
                result.append(ord(" "))
                char_count = 0
            if len(result) >= target_size:
                return trim_to_utf8_boundary(result, target_size)

    if len(result) < target_size:
        sys.stderr.write(
            f"  [warn] sequence exhausted at {len(result)} B"
            f" (target {target_size})\n"
        )
    return trim_to_utf8_boundary(result, target_size)


def word_length_stats(words: list[str]) -> dict:
    lengths = [len(w) for w in words]  # Unicode char count
    return {
        "mean": statistics.mean(lengths),
        "median": statistics.median(lengths),
        "min": min(lengths),
        "max": max(lengths),
    }


def compute_stats(text_bytes: bytes, words_th: set[str]) -> dict:
    total = len(text_bytes)
    n_spaces = text_bytes.count(b" ")
    text_str = text_bytes.decode("utf-8", errors="replace")
    tokens = [t for t in text_str.split(" ") if t]
    tok_b = [len(t.encode("utf-8")) for t in tokens]
    oov = sum(1 for t in tokens if t not in words_th)
    return {
        "size_b": total,
        "n_spaces": n_spaces,
        "space_pct": n_spaces / total * 100,
        "n_tokens": len(tokens),
        "oov_pct": oov / len(tokens) * 100 if tokens else 0.0,
        "tok_mean": statistics.mean(tok_b) if tok_b else 0,
        "tok_med": statistics.median(tok_b) if tok_b else 0,
        "tok_max": max(tok_b) if tok_b else 0,
    }


def print_header():
    print(
        f"\n{'File':<48}  {'Size':>7}  {'Space%':>7}  "
        f"{'Tokens':>7}  {'OOV%':>6}  "
        f"{'AvgTokB':>8}  {'MedTokB':>8}  {'MaxTokB':>8}"
    )
    print("-" * 118)


def print_row(name: str, s: dict):
    print(
        f"{name:<48}  {s['size_b']:>7,}  "
        f"{s['space_pct']:>6.2f}%  {s['n_tokens']:>7,}  "
        f"{s['oov_pct']:>5.1f}%  "
        f"{s['tok_mean']:>8.1f}  {s['tok_med']:>8.1f}  {s['tok_max']:>8}"
    )


def generate_set(
    pool: list[str],
    file_prefix: str,
    rng: random.Random,
    words_th: set[str],
    data_dir: Path,
):
    """Generate three files (space-01/05/10) for one dictionary set."""
    seq = build_sequence(pool, int(TARGET_SIZE * 1.05), rng)
    print(f"  Pool: {len(pool):,} entries  |  Sequence: {len(seq):,} words")
    print_header()
    for ratio in SPACE_RATIOS:
        suffix = f"{int(ratio * 100):02d}"
        outname = f"{file_prefix}-space-{suffix}.txt"
        text_b = apply_spacing(seq, ratio, TARGET_SIZE)
        (data_dir / outname).write_bytes(text_b)
        print_row(outname, compute_stats(text_b, words_th))


# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------

def main():
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--data-dir", default="tests/data")
    args = ap.parse_args()

    data_dir = Path(args.data_dir)
    if not data_dir.is_dir():
        sys.exit(f"data-dir not found: {data_dir}")

    # ------------------------------------------------------------------
    # Load dictionaries
    # ------------------------------------------------------------------
    print("Loading dictionaries…")
    d10k = load_words(data_dir / "dict-10k.txt")
    d_long = load_words(data_dir / "dict-1k-long.txt")
    d_short = load_words(data_dir / "dict-1k-short.txt")
    d_comm = load_words(data_dir / "dict-200-common.txt")
    words_th = set(load_words(data_dir / "dict-words-th.txt"))

    def oov_pct(lst):
        return len(set(lst) - words_th) / len(set(lst)) * 100

    for name, lst in [
        ("dict-10k      (10 000 words)", d10k),
        ("dict-1k-long  ( 1 000 words)", d_long),
        ("dict-1k-short ( 1 000 words)", d_short),
        ("dict-200-comm (   200 words)", d_comm),
    ]:
        st = word_length_stats(lst)
        print(
            f"  {name}  "
            f"avg={st['mean']:.1f} chars  med={st['median']}  "
            f"[{st['min']}–{st['max']}]  "
            f"vocab OOV={oov_pct(lst):.1f}%"
        )
    print("  dict-words-th  (62 018 words) — tokenizer dictionary reference")

    print("\nSource dict overlaps:")
    for a, an, b, bn in [
        (d_long, "long", d_short, "short"),
        (d_long, "long", d_comm, "common"),
        (d_short, "short", d_comm, "common"),
        (d_long, "long", d10k, "10k"),
        (d_short, "short", d10k, "10k"),
        (d_comm, "common", d10k, "10k"),
    ]:
        print(f"  {an} ∩ {bn}: {len(set(a) & set(b))}")

    exp_oov_set2 = 0.25 * (
        oov_pct(d10k) + oov_pct(d_long)
        + oov_pct(d_short) + oov_pct(d_comm)
    )
    print(
        f"\nExpected vocabulary OOV (Set 2, 25 % each):"
        f" ≈ {exp_oov_set2:.1f}%"
    )

    print("\nChar-per-space thresholds:")
    for r in SPACE_RATIOS:
        k = k_chars_for_ratio(r)
        est = 1.0 / (k * APPROX_BPC + 1) * 100
        print(
            f"  target {r*100:4.1f}%"
            f" → k_chars={k:3d}  (≈ {est:.2f}% bytes)"
        )

    # ------------------------------------------------------------------
    # Set 1: dict-10k + punctuation/numbers   (0 % vocab OOV)
    # ------------------------------------------------------------------
    # Punctuation/number slots shared by all sets (mimic natural text)
    n_punct = int(len(d10k) * PUNCT_WEIGHT / (1 - PUNCT_WEIGHT))
    punct_slots = (
        PUNCT_NUMBERS * (n_punct // len(PUNCT_NUMBERS) + 1)
    )[:n_punct]

    # ------------------------------------------------------------------
    # Set 1: dict-10k + punct/numbers   (vocab OOV ≈ 0 %)
    # ------------------------------------------------------------------
    print("\n=== Dict Set 1: dict-10k + punct  (vocab OOV ≈ 0 %) ===")
    rng1 = random.Random(SEED)
    pool1 = d10k + punct_slots
    generate_set(pool1, "text-only-dict-10k", rng1, words_th, data_dir)

    # ------------------------------------------------------------------
    # Set 2: 25 % each from 4 dicts + punct  (~11 % vocab OOV, mixed lengths)
    # ------------------------------------------------------------------
    print(
        "\n=== Dict Set 2: 25% × 4 dicts + punct"
        "  (vocab OOV ≈ 11 %, mixed lengths) ==="
    )
    rng2 = random.Random(SEED)
    slot = max(len(d10k), len(d_long), len(d_short), len(d_comm))
    pool2 = (
        repeat_to(d10k, slot)
        + repeat_to(d_long, slot)
        + repeat_to(d_short, slot)
        + repeat_to(d_comm, slot)
        + punct_slots
    )
    generate_set(pool2, "text-only-dict-10k-1k", rng2, words_th, data_dir)

    # ------------------------------------------------------------------
    # Set 3: dict-1k-long + punct  (~26 % vocab OOV, uniform ~12-char words)
    # Homogeneous word lengths → fair distribution of words per chunk.
    # ------------------------------------------------------------------
    print(
        "\n=== Dict Set 3: dict-1k-long + punct"
        "  (vocab OOV ≈ 26 %, uniform ~12-char words) ==="
    )
    rng3 = random.Random(SEED)
    pool3 = d_long + punct_slots
    generate_set(pool3, "text-only-dict-1k-long", rng3, words_th, data_dir)

    # ------------------------------------------------------------------
    # Set 4: dict-1k-short + punct  (~16 % vocab OOV, uniform ~6-char words)
    # Similar avg word length as Set 1 but with OOV — isolates OOV effect.
    # ------------------------------------------------------------------
    print(
        "\n=== Dict Set 4: dict-1k-short + punct"
        "  (vocab OOV ≈ 16 %, uniform ~6-char words) ==="
    )
    rng4 = random.Random(SEED)
    pool4 = d_short + punct_slots
    generate_set(pool4, "text-only-dict-1k-short", rng4, words_th, data_dir)

    print(f"\nDone. 12 files written to {data_dir}/")


if __name__ == "__main__":
    main()
