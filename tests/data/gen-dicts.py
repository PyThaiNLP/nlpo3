# SPDX-FileCopyrightText: 2026 PyThaiNLP Project
# SPDX-License-Identifier: Apache-2.0

"""Regenerate all three benchmark dictionaries with proper prefix-diverse sampling.

Fix: shuffle the list of prefix groups before round-robin so we don't
just take the first N sorted prefixes (which are all ก/ข for Thai).
"""

import random
from collections import defaultdict, Counter
from pathlib import Path

random.seed(42)

base = Path(__file__).parent
src = base / "words_th.txt"
PREFIX_LEN = 3

with open(src, encoding="utf-8") as f:
    all_words = [w.strip() for w in f if w.strip()]


def prefix_diverse_sample(pool, target, prefix_len=PREFIX_LEN):
    """Round-robin across prefix groups in shuffled order to spread across alphabet."""
    groups: dict[str, list[str]] = defaultdict(list)
    for w in pool:
        groups[w[:prefix_len]].append(w)

    # Shuffle the word list within each group (for variety within a prefix)
    for g in groups.values():
        random.shuffle(g)

    # Shuffle the prefix order so we don't bias toward early alphabet letters
    shuffled_prefixes = list(groups.keys())
    random.shuffle(shuffled_prefixes)

    iters = {p: iter(groups[p]) for p in shuffled_prefixes}
    picked = []
    exhausted = set()

    while len(picked) < target:
        progress = False
        for p in shuffled_prefixes:
            if p in exhausted:
                continue
            try:
                picked.append(next(iters[p]))
                progress = True
                if len(picked) == target:
                    break
            except StopIteration:
                exhausted.add(p)
        if not progress:
            break

    return picked


def stats(label, picked):
    prefix_counts = defaultdict(int)
    for w in picked:
        prefix_counts[w[:PREFIX_LEN]] += 1
    lc = Counter(len(w) for w in picked)
    # count distinct first chars
    first_chars = Counter(w[0] for w in picked)
    print(f"{label}: {len(picked)} words")
    print(
        f"  distinct 3-char prefixes: {len(prefix_counts)}, max per prefix: {max(prefix_counts.values())}"
    )
    print(f"  distinct first chars: {len(first_chars)}")
    print(f"  len range: {min(len(w) for w in picked)}-{max(len(w) for w in picked)}")
    print(f"  len dist: { {k: lc[k] for k in sorted(lc)} }")


# --- Short: 3–10 chars, 500 words ---
short_pool = [w for w in all_words if 3 <= len(w) <= 10]
print(f"Short pool (3-10 chars): {len(short_pool)}")
picked_short = prefix_diverse_sample(short_pool, 500)
stats("Short", picked_short)

# --- Long: 15+ chars, 500 words ---
long_pool = [w for w in all_words if len(w) >= 15]
print(f"\nLong pool (>=15 chars): {len(long_pool)}")
picked_long = prefix_diverse_sample(long_pool, 500)
stats("Long", picked_long)

# --- Medium: all words, 10,000 words ---
print(f"\nMedium pool (all): {len(all_words)}")
picked_medium = prefix_diverse_sample(all_words, 10000)
stats("Medium", picked_medium)

# Write files
for name, picked in [
    ("500-short", picked_short),
    ("500-long", picked_long),
    ("10k", picked_medium),
]:
    out = base / f"{name}.txt"
    with open(out, "w", encoding="utf-8") as f:
        f.write("\n".join(sorted(picked)) + "\n")
    print(f"Wrote {name}.txt")

print("\nDone.")
