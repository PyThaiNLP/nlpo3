// SPDX-FileCopyrightText: 2024-2026 PyThaiNLP Project
// SPDX-License-Identifier: Apache-2.0

//! Performance and memory benchmarks for Thai dictionary backends and tokenizers.
//!
//! # Benchmark sets
//!
//! ## Set 1 — Dictionary backend comparison
//! Compares TrieChar, TrieCharLegacy, and FstDict across all four dictionary
//! sizes using small text inputs (wikipedia-s, wikipedia-m) where FstDict is
//! still practical:
//!
//! | Group | What is measured |
//! |-------|-----------------|
//! | `dict_construction` | Build time per backend × 4 dict sizes |
//! | `prefix_lookup` | Per-query prefix scan per backend × 4 dict sizes |
//! | `dict_operations` | `add` / `remove` / `contain` per backend × 4 dict sizes |
//! | `memory_footprint` | Heap estimates for all 4 dict sizes (printed to stderr) |
//! | `clone_cost` | O(1) Arc-clone verification |
//! | `dict_backend_tokenization` | End-to-end speed: all 3 backends × 4 dicts × 2 small texts |
//!
//! ## Set 2 — Tokenizer comparison
//! Compares NewmmTokenizer (TrieChar), NewmmLegacyTokenizer (TrieCharLegacy),
//! and DeepcutTokenizer on large texts.  FstDict is excluded here: its
//! character-level FST fallback on out-of-vocabulary input makes it
//! impractically slow on texts longer than ~100 KB.
//!
//! | Group | What is measured |
//! |-------|-----------------|
//! | `tokenizer_performance` | End-to-end speed: 2 tokenizers (+ Deepcut) × 2 dicts × up to 4 texts |
//!
//! # Dictionaries (`tests/data/`)
//!
//! | File | Words | Word-length profile |
//! |------|------:|---------------------|
//! | `dict-1k-long.txt`  | 1 000 | 15–36 chars — stress-tests long-token paths |
//! | `dict-1k-short.txt` | 1 000 | 3–10 chars — dense prefix overlap |
//! | `dict-10k.txt`      | 10 000 | 1–34 chars — mid-size realistic vocabulary |
//! | `dict-words-th.txt` | 62 018 | 1–36 chars — full Thai dictionary |
//!
//! # Text inputs (`tests/data/`)
//!
//! | File | Size | Notes |
//! |------|-----:|-------|
//! | `text-wikipedia-s.txt`          | ~2.4 KB  | Small Wikipedia excerpt (Thai/Latin mix) |
//! | `text-wikipedia-m.txt`          | ~41 KB   | Medium Wikipedia excerpt |
//! | `text-wikipedia-l.txt`          | ~615 KB  | Large Wikipedia excerpt |
//! | `text-only-dict-10k-words.txt`  | ~1 MB    | Only words from dict-10k — low OOV |
//! | `text-only-dict-10k-1k-words.txt` | ~1 MB  | Mix of dict-10k + dict-1k-* — moderate OOV |
//! | `text-ws-social.txt`            | ~6.3 MB  | Social-media posts — high OOV |
//!
//! Run with:
//! ```sh
//! cargo bench
//! # include Deepcut:
//! cargo bench --features deepcut
//! # single group:
//! cargo bench -- dict_construction
//! ```
//! HTML reports: `target/criterion/`.

use std::time::Duration;

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
#[cfg(feature = "deepcut")]
use nlpo3::tokenizer::{
    deepcut::DeepcutTokenizer, parallel_helper, parallel_options::ParallelOptions,
    tcc::tcc_tokenizer,
};
use nlpo3::{
    char_string::CharString,
    tokenizer::{
        fst_dict::FstDict,
        newmm::{NewmmFstTokenizer, NewmmTokenizer},
        trie_char::{TrieChar, TrieCharLegacy},
    },
};
use std::hint::black_box;

// ---------------------------------------------------------------------------
// Shared test fixtures
// ---------------------------------------------------------------------------

/// Inline text used only by the Deepcut chunking-overhead benchmark.
#[allow(dead_code)]
const LONG_TEXT: &str = "\
ไต้หวัน (แป่ะเอ๋ยี้: Tâi-oân; ไต่อวัน) หรือ ไถวาน \
(อักษรโรมัน: Taiwan; จีนตัวย่อ: 台湾; จีนตัวเต็ม: 臺灣/台灣; พินอิน: \
Táiwān; ไถวาน) หรือชื่อทางการว่า สาธารณรัฐจีน (จีนตัวย่อ: 中华民国; \
จีนตัวเต็ม: 中華民國; พินอิน: Zhōnghuá \
Mínguó) เป็นรัฐในทวีปเอเชียตะวันออก[7][8][9] ปัจจุบันประกอบด้วย\
เกาะใหญ่ 5 แห่ง คือ จินเหมิน (金門), ไต้หวัน, เผิงหู (澎湖), หมาจู่ \
(馬祖), และอูชิว (烏坵) กับทั้งเกาะเล็กเกาะน้อยอีกจำนวนหนึ่ง \
ท้องที่ดังกล่าวเรียกรวมกันว่า \"พื้นที่ไต้หวัน\" (臺灣地區)\n\
ไต้หวันด้านตะวันตกติดกับจีนแผ่นดินใหญ่ ด้านตะวันออกและตะวันออก\
เฉียงเหนือติดกับญี่ปุ่น และด้านใต้ติดกับฟิลิปปินส์ กรุงไทเปเป็น\
เมืองหลวง ส่วนไทเปใหม่เป็นเขตปกครองที่จัดตั้งขึ้นใหม่ กินพื้นที่\
กรุงไทเปและเป็นเขตซึ่งประชากรหนาแน่นที่สุดในเวลานี้\n\
เกาะไต้หวันเดิมเป็นที่อยู่ของชนพื้นเมือง และมีชาวจีนจากแผ่นดิน\
ใหญ่เข้ามาอาศัยร่วมด้วย จนกระทั่งชาววิลันดาและสเปนเดินทางเข้า\
มาในยุคสำรวจเมื่อศตวรรษที่ 17 และมาตั้งบ้านเรือนกลายเป็นนิคม\
ใหญ่โต ต่อมาปี 1662 ราชวงศ์หมิงในแผ่นดินใหญ่ถูกราชวงศ์ชิงแทนที่";

const BASE_PATH: &str = env!("CARGO_MANIFEST_DIR");

/// Four benchmark dictionaries used throughout Set 1.  Ordered small → large.
const BENCH_DICTS: &[(&str, &str)] = &[
    ("1k-long", "tests/data/dict-1k-long.txt"),
    ("1k-short", "tests/data/dict-1k-short.txt"),
    ("10k", "tests/data/dict-10k.txt"),
    ("words-th", "tests/data/dict-words-th.txt"),
];

/// Six text files available for end-to-end benchmarks.  Listed for reference;
/// individual benchmark functions load the files they need directly.
#[allow(dead_code)]
const BENCH_TEXTS: &[(&str, &str)] = &[
    (
        "text-only-10k-1k",
        "tests/data/text-only-dict-10k-1k-words.txt",
    ),
    ("text-only-10k", "tests/data/text-only-dict-10k-words.txt"),
    ("wikipedia-s", "tests/data/text-wikipedia-s.txt"),
    ("wikipedia-m", "tests/data/text-wikipedia-m.txt"),
    ("wikipedia-l", "tests/data/text-wikipedia-l.txt"),
    ("ws-social", "tests/data/text-ws-social.txt"),
];

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn dict_path(filename: &str) -> String {
    format!("{}/{}", BASE_PATH, filename)
}

fn load_dict_words(filename: &str) -> Vec<String> {
    let path = dict_path(filename);
    std::fs::read_to_string(&path)
        .unwrap_or_else(|_| panic!("dict file not found: {path}"))
        .lines()
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty())
        .collect()
}

fn load_text_file(filename: &str) -> String {
    let path = dict_path(filename);
    std::fs::read_to_string(&path).unwrap_or_else(|_| panic!("text file not found: {path}"))
}

// ===========================================================================
// Set 1-A  Dictionary construction
// Build time for each backend × all 4 dict sizes.
// ===========================================================================

fn bench_dict_construction(c: &mut Criterion) {
    let mut group = c.benchmark_group("dict_construction");
    group.sample_size(10);

    for (dict_name, dict_file) in BENCH_DICTS {
        let words = load_dict_words(dict_file);
        let char_words: Vec<CharString> = words.iter().map(|w| CharString::new(w)).collect();
        let word_strs: Vec<&str> = words.iter().map(|s| s.as_str()).collect();

        group.bench_with_input(
            BenchmarkId::new("TrieChar::new", dict_name),
            &char_words,
            |b, cw| b.iter(|| black_box(TrieChar::new(black_box(cw)))),
        );
        group.bench_with_input(
            BenchmarkId::new("TrieCharLegacy::new", dict_name),
            &char_words,
            |b, cw| b.iter(|| black_box(TrieCharLegacy::new(black_box(cw)))),
        );
        group.bench_with_input(
            BenchmarkId::new("FstDict::from_words", dict_name),
            &word_strs,
            |b, ws| {
                b.iter(|| black_box(FstDict::from_words(black_box(ws.iter().copied())).unwrap()))
            },
        );
    }

    group.finish();
}

// ===========================================================================
// Set 1-B  Dictionary prefix lookup
// Finding all entries that are prefixes of a query — the hot tokenization path.
// ===========================================================================

fn bench_prefix_lookup(c: &mut Criterion) {
    let queries: &[(&str, &str)] = &[
        ("short-thai", "สวัสดีครับ"),
        ("mixed", "ไต้หวัน1984"),
        ("medium-thai", "อาชญากรรมทางการแพทย์"),
    ];

    let mut group = c.benchmark_group("prefix_lookup");

    for (dict_name, dict_file) in BENCH_DICTS {
        let words = load_dict_words(dict_file);
        let char_words: Vec<CharString> = words.iter().map(|w| CharString::new(w)).collect();
        let trie = TrieChar::new(&char_words);
        let legacy = TrieCharLegacy::new(&char_words);
        let fst = FstDict::from_words(words.iter().map(|s| s.as_str())).unwrap();

        for (query_label, text) in queries {
            let cs = CharString::new(text);
            let id_suffix = format!("{dict_name}/{query_label}");

            group.bench_with_input(
                BenchmarkId::new("TrieChar::prefix_ref", &id_suffix),
                &cs,
                |b, input| b.iter(|| black_box(TrieChar::prefix_ref(black_box(input), &trie))),
            );
            group.bench_with_input(
                BenchmarkId::new("TrieCharLegacy::prefix_ref", &id_suffix),
                &cs,
                |b, input| {
                    b.iter(|| black_box(TrieCharLegacy::prefix_ref(black_box(input), &legacy)))
                },
            );
            group.bench_with_input(
                BenchmarkId::new("FstDict::prefix_lengths", &id_suffix),
                text,
                |b, t| b.iter(|| black_box(fst.prefix_lengths(black_box(t)))),
            );
        }
    }

    group.finish();
}

// ===========================================================================
// Set 1-C  Dictionary operations — contain / add / remove
// Each add/remove bench clones the pre-built dict per iteration to avoid
// cumulative mutation.
// ===========================================================================

fn bench_dict_operations(c: &mut Criterion) {
    let test_word_cs = CharString::new("กาแฟ"); // present in words-th and most sub-dicts
    let add_word_cs = CharString::new("เบนช์มาร์ก");
    let add_word_str = "เบนช์มาร์ก";

    let mut group = c.benchmark_group("dict_operations");

    for (dict_name, dict_file) in BENCH_DICTS {
        let words = load_dict_words(dict_file);
        let char_words: Vec<CharString> = words.iter().map(|w| CharString::new(w)).collect();

        let trie_base = TrieChar::new(&char_words);
        let legacy_base = TrieCharLegacy::new(&char_words);
        let fst_base = FstDict::from_words(words.iter().map(|s| s.as_str())).unwrap();

        // --- contain ---
        group.bench_with_input(
            BenchmarkId::new("TrieChar::contain", dict_name),
            &test_word_cs,
            |b, w| b.iter(|| black_box(trie_base.contain(black_box(w)))),
        );
        group.bench_with_input(
            BenchmarkId::new("TrieCharLegacy::contain", dict_name),
            &test_word_cs,
            |b, w| b.iter(|| black_box(legacy_base.contain(black_box(w)))),
        );
        group.bench_with_input(
            BenchmarkId::new("FstDict::contains", dict_name),
            &"กาแฟ",
            |b, w| b.iter(|| black_box(fst_base.contains(black_box(w)))),
        );

        // --- add (clone per iteration) ---
        let trie_for_add = trie_base.clone();
        let legacy_for_add = legacy_base.clone();
        let fst_for_add = fst_base.clone();

        group.bench_with_input(
            BenchmarkId::new("TrieChar::add", dict_name),
            &add_word_cs,
            |b, w| {
                b.iter(|| {
                    let mut d = trie_for_add.clone();
                    black_box(d.add(black_box(w)));
                })
            },
        );
        group.bench_with_input(
            BenchmarkId::new("TrieCharLegacy::add", dict_name),
            &add_word_cs,
            |b, w| {
                b.iter(|| {
                    let mut d = legacy_for_add.clone();
                    black_box(d.add(black_box(w)));
                })
            },
        );
        group.bench_with_input(
            BenchmarkId::new("FstDict::add", dict_name),
            &add_word_str,
            |b, w| {
                b.iter(|| {
                    let mut d = fst_for_add.clone();
                    black_box(d.add(black_box(w)));
                })
            },
        );

        // --- remove (clone per iteration) ---
        let trie_for_rm = trie_base.clone();
        let legacy_for_rm = legacy_base.clone();
        let fst_for_rm = fst_base.clone();

        group.bench_with_input(
            BenchmarkId::new("TrieChar::remove", dict_name),
            &test_word_cs,
            |b, w| {
                b.iter(|| {
                    let mut d = trie_for_rm.clone();
                    black_box(d.remove(black_box(w)));
                })
            },
        );
        group.bench_with_input(
            BenchmarkId::new("TrieCharLegacy::remove", dict_name),
            &test_word_cs,
            |b, w| {
                b.iter(|| {
                    let mut d = legacy_for_rm.clone();
                    black_box(d.remove(black_box(w)));
                })
            },
        );
        group.bench_with_input(
            BenchmarkId::new("FstDict::remove", dict_name),
            &"กาแฟ",
            |b, w| {
                b.iter(|| {
                    let mut d = fst_for_rm.clone();
                    black_box(d.remove(black_box(w)));
                })
            },
        );
    }

    group.finish();
}

// ===========================================================================
// Set 1-D  Memory footprint
// Heap-size estimates are printed to stderr so they appear in `cargo bench`
// output even when Criterion's statistical loop is not the right tool for
// measuring allocation sizes.
// ===========================================================================

fn bench_memory_footprint(c: &mut Criterion) {
    use std::mem;

    eprintln!();
    eprintln!("╔══════════════════════════════════════════════════════════════╗");
    eprintln!("║              Memory footprint analysis                       ║");
    eprintln!("╠══════════════════════════════════════════════════════════════╣");

    // Per-character heap usage of CharString
    let sample_text = "ไต้หวัน (แป่ะเอ๋ยี้: Tâi-oân; ไต่อวัน) หรือ ไถวาน";
    let n_chars = sample_text.chars().count();
    let utf8_bytes = sample_text.len();
    let heap_per_char =
        (utf8_bytes + (n_chars + 1) * mem::size_of::<u32>()) as f64 / n_chars as f64;
    eprintln!(
        "║  CharString stack:  {} bytes                                  ║",
        mem::size_of::<CharString>()
    );
    eprintln!(
        "║  CharString heap:   {:.1} bytes/char (UTF-8 + u32 pos table)  ║",
        heap_per_char
    );
    eprintln!("╠══════════════════════════════════════════════════════════════╣");
    eprintln!("║  Dictionary storage estimates                                ║");
    eprintln!("║                                                              ║");
    eprintln!(
        "║  {:<12}  {:>8}  {:>12}  {:>12}  {:>9}  ║",
        "Dict", "Words", "FstDict", "TrieChar", "TrieCharL"
    );
    eprintln!(
        "║  {:<12}  {:>8}  {:>12}  {:>12}  {:>9}  ║",
        "----", "-----", "-------", "--------", "---------"
    );

    for (dict_name, dict_file) in BENCH_DICTS {
        let words = load_dict_words(dict_file);
        let n_words = words.len();
        let fst = FstDict::from_words(words.iter().map(|s| s.as_str())).unwrap();
        let fst_bytes = fst.fst_size_bytes();

        let total_chars: usize = words.iter().map(|w| w.chars().count()).sum();
        // ~80 bytes per trie edge: 24-byte TrieNode + HashMap bucket (~56 bytes)
        let trie_est = total_chars * 80;
        // extra HashSet overhead for TrieCharLegacy: ~24+~12+~56 bytes per word
        let words_set_bytes: usize = words.iter().map(|w| w.len() + 48).sum();
        let legacy_est = trie_est + words_set_bytes;

        eprintln!(
            "║  {:<12}  {:>8}  {:>9} KB  {:>9} MB  {:>6} MB  ║",
            dict_name,
            n_words,
            fst_bytes / 1_024,
            trie_est / 1_000_000,
            legacy_est / 1_000_000,
        );
    }

    eprintln!("╚══════════════════════════════════════════════════════════════╝");

    #[cfg(feature = "deepcut")]
    {
        let model_path = format!("{}/model/deepcut.onnx", BASE_PATH);
        let model_bytes = std::fs::metadata(&model_path).map(|m| m.len()).unwrap_or(0);
        eprintln!(
            "  DeepcutTokenizer ONNX model (bundled): {} bytes  ({:.1} MB)",
            model_bytes,
            model_bytes as f64 / 1_000_000.0
        );
    }
    eprintln!();

    let mut group = c.benchmark_group("memory_footprint");
    group.bench_function("CharString::new/overhead", |b| {
        b.iter(|| {
            let cs = CharString::new(black_box(sample_text));
            black_box(mem::size_of::<CharString>() + cs.as_str().len() + (cs.chars_len() + 1) * 4)
        })
    });
    group.finish();
}

// ===========================================================================
// Set 1-E  Clone cost — Arc-backed dicts make tokenizer clone O(1)
// ===========================================================================

fn bench_clone_cost(c: &mut Criterion) {
    let path = dict_path("tests/data/dict-words-th.txt");
    let word_list = load_dict_words("tests/data/dict-words-th.txt");
    let tok_trie = NewmmTokenizer::new(&path).unwrap();
    let tok_legacy = NewmmTokenizer::<TrieCharLegacy>::from_word_list(word_list);
    let tok_fst = NewmmFstTokenizer::new(&path).unwrap();

    let mut group = c.benchmark_group("clone_cost");
    group.sample_size(200);

    group.bench_function("NewmmTokenizer<TrieChar>::clone", |b| {
        b.iter(|| black_box(tok_trie.clone()))
    });
    group.bench_function("NewmmTokenizer<TrieCharLegacy>::clone", |b| {
        b.iter(|| black_box(tok_legacy.clone()))
    });
    group.bench_function("NewmmFstTokenizer::clone", |b| {
        b.iter(|| black_box(tok_fst.clone()))
    });

    group.finish();
}

// ===========================================================================
// Set 1-F  DictBackend end-to-end tokenization
//
// All three backends × all four dictionaries × two small text inputs.
// FstDict is included because the texts are small enough for it to finish in
// a practical time.  For large texts use Set 2 (bench_tokenizer_performance).
// ===========================================================================

fn bench_dict_backend_tokenization(c: &mut Criterion) {
    let text_wiki_s = load_text_file("tests/data/text-wikipedia-s.txt");
    let text_wiki_m = load_text_file("tests/data/text-wikipedia-m.txt");

    let small_texts: &[(&str, &str)] =
        &[("wikipedia-s", &text_wiki_s), ("wikipedia-m", &text_wiki_m)];

    let mut group = c.benchmark_group("dict_backend_tokenization");
    group.sample_size(50);

    for (dict_name, dict_file) in BENCH_DICTS {
        let words = load_dict_words(dict_file);
        let tok_trie = NewmmTokenizer::<TrieChar>::from_word_list(words.clone());
        let tok_legacy = NewmmTokenizer::<TrieCharLegacy>::from_word_list(words.clone());
        let tok_fst = NewmmFstTokenizer::from_word_list(words).unwrap();

        for (text_label, text) in small_texts {
            group.throughput(Throughput::Bytes(text.len() as u64));
            let id = format!("{dict_name}/{text_label}");

            group.bench_with_input(BenchmarkId::new("TrieChar", &id), *text, |b, t| {
                b.iter(|| black_box(tok_trie.segment(black_box(t)).unwrap()))
            });
            group.bench_with_input(BenchmarkId::new("TrieCharLegacy", &id), *text, |b, t| {
                b.iter(|| black_box(tok_legacy.segment(black_box(t)).unwrap()))
            });
            group.bench_with_input(BenchmarkId::new("FstDict", &id), *text, |b, t| {
                b.iter(|| black_box(tok_fst.segment(black_box(t)).unwrap()))
            });
        }
    }

    group.finish();
}

// ===========================================================================
// Set 2  Tokenizer performance
//
// NewmmTokenizer (TrieChar) and NewmmLegacyTokenizer (TrieCharLegacy)
// measured on larger, real-world texts.  DeepcutTokenizer is included when
// compiled with `--features deepcut`.
//
// FstDict is deliberately excluded: on texts > ~100 KB its per-character FST
// fallback for OOV input makes individual iterations take tens of seconds,
// rendering the benchmark impractical for regular use.
//
// Run A — dict-10k (mid-size vocabulary):
//   texts: text-only-dict-10k-words (low OOV),
//          text-only-dict-10k-1k-words (moderate OOV),
//          text-wikipedia-l
//
// Run B — dict-words-th (full vocabulary):
//   texts: text-only-dict-10k-words (low OOV w.r.t. this dict),
//          text-only-dict-10k-1k-words (low OOV),
//          text-wikipedia-l,
//          text-ws-social (high OOV, social-media noise)
// ===========================================================================

fn bench_tokenizer_performance(c: &mut Criterion) {
    // Load texts once.
    let text_only_10k = load_text_file("tests/data/text-only-dict-10k-words.txt");
    let text_only_10k_1k = load_text_file("tests/data/text-only-dict-10k-1k-words.txt");
    let text_wiki_l = load_text_file("tests/data/text-wikipedia-l.txt");
    let text_ws_social = load_text_file("tests/data/text-ws-social.txt");

    let mut group = c.benchmark_group("tokenizer_performance");
    group.sample_size(10);
    group.measurement_time(Duration::from_secs(60));

    // -----------------------------------------------------------------------
    // Run A — dict-10k
    // -----------------------------------------------------------------------
    {
        let words_10k = load_dict_words("tests/data/dict-10k.txt");
        let tok_trie_10k = NewmmTokenizer::<TrieChar>::from_word_list(words_10k.clone());
        let tok_legacy_10k = NewmmTokenizer::<TrieCharLegacy>::from_word_list(words_10k);

        #[cfg(feature = "deepcut")]
        let tok_deepcut = DeepcutTokenizer::new().expect("deepcut: ONNX model failed to load");

        let texts_10k: &[(&str, &str)] = &[
            ("text-only-10k", &text_only_10k),
            ("text-only-10k-1k", &text_only_10k_1k),
            ("wikipedia-l", &text_wiki_l),
        ];

        for (text_label, text) in texts_10k {
            group.throughput(Throughput::Bytes(text.len() as u64));
            let id_trie = format!("10k/{text_label}/TrieChar");
            let id_legacy = format!("10k/{text_label}/TrieCharLegacy");

            group.bench_with_input(
                BenchmarkId::new("NewmmTokenizer", &id_trie),
                *text,
                |b, t| b.iter(|| black_box(tok_trie_10k.segment(black_box(t)).unwrap())),
            );
            group.bench_with_input(
                BenchmarkId::new("NewmmLegacyTokenizer", &id_legacy),
                *text,
                |b, t| b.iter(|| black_box(tok_legacy_10k.segment(black_box(t)).unwrap())),
            );

            #[cfg(feature = "deepcut")]
            {
                let id_dc = format!("10k/{text_label}/Deepcut");
                group.bench_with_input(
                    BenchmarkId::new("DeepcutTokenizer", &id_dc),
                    *text,
                    |b, t| b.iter(|| black_box(tok_deepcut.segment(black_box(t)).unwrap())),
                );
            }
        }
    }

    // -----------------------------------------------------------------------
    // Run B — dict-words-th (full vocabulary)
    // -----------------------------------------------------------------------
    {
        let words_th = load_dict_words("tests/data/dict-words-th.txt");
        let tok_trie_th = NewmmTokenizer::<TrieChar>::from_word_list(words_th.clone());
        let tok_legacy_th = NewmmTokenizer::<TrieCharLegacy>::from_word_list(words_th);

        #[cfg(feature = "deepcut")]
        let tok_deepcut = DeepcutTokenizer::new().expect("deepcut: ONNX model failed to load");

        let texts_th: &[(&str, &str)] = &[
            ("text-only-10k", &text_only_10k),
            ("text-only-10k-1k", &text_only_10k_1k),
            ("wikipedia-l", &text_wiki_l),
            ("ws-social", &text_ws_social),
        ];

        for (text_label, text) in texts_th {
            group.throughput(Throughput::Bytes(text.len() as u64));
            let id_trie = format!("words-th/{text_label}/TrieChar");
            let id_legacy = format!("words-th/{text_label}/TrieCharLegacy");

            group.bench_with_input(
                BenchmarkId::new("NewmmTokenizer", &id_trie),
                *text,
                |b, t| b.iter(|| black_box(tok_trie_th.segment(black_box(t)).unwrap())),
            );
            group.bench_with_input(
                BenchmarkId::new("NewmmLegacyTokenizer", &id_legacy),
                *text,
                |b, t| b.iter(|| black_box(tok_legacy_th.segment(black_box(t)).unwrap())),
            );

            #[cfg(feature = "deepcut")]
            {
                let id_dc = format!("words-th/{text_label}/Deepcut");
                group.bench_with_input(
                    BenchmarkId::new("DeepcutTokenizer", &id_dc),
                    *text,
                    |b, t| b.iter(|| black_box(tok_deepcut.segment(black_box(t)).unwrap())),
                );
            }
        }
    }

    group.finish();
}

// ===========================================================================
// Supporting — Deepcut chunking overhead (requires --features deepcut)
// ===========================================================================

#[cfg(feature = "deepcut")]
fn bench_deepcut_chunking_overhead(c: &mut Criterion) {
    let tok = DeepcutTokenizer::new().expect("deepcut: ONNX model failed to load");

    let huge_text = LONG_TEXT.repeat(240);
    let len = huge_text.len();
    let seq_chunk_size = ParallelOptions::MIN_CHUNK_SIZE;

    let mut group = c.benchmark_group("deepcut_chunking_overhead");
    group.sample_size(10);
    group.measurement_time(Duration::from_secs(8));
    group.throughput(Throughput::Bytes(len as u64));

    group.bench_with_input(
        BenchmarkId::new("chunking", "disabled"),
        &huge_text,
        |b, t| b.iter(|| black_box(tok.segment_with_options(black_box(t), None).unwrap())),
    );
    group.bench_with_input(
        BenchmarkId::new("chunking", "enabled-sequential"),
        &huge_text,
        |b, t| {
            b.iter(|| {
                let tcc_positions = tcc_tokenizer::tcc_pos(black_box(t));
                let chunks = parallel_helper::split_text_into_chunks(
                    black_box(t),
                    seq_chunk_size,
                    &tcc_positions,
                );
                let token_vecs =
                    parallel_helper::tokenize_chunks(chunks, false, |chunk| tok.tokenize(chunk))
                        .unwrap();
                black_box(parallel_helper::flatten_tokens(token_vecs))
            })
        },
    );
    group.bench_with_input(
        BenchmarkId::new("chunking", "enabled-parallel"),
        &huge_text,
        |b, t| {
            b.iter(|| {
                black_box(
                    tok.segment_with_options(black_box(t), Some(ParallelOptions::MIN_CHUNK_SIZE))
                        .unwrap(),
                )
            })
        },
    );

    group.finish();
}

// ===========================================================================
// Summary — printed to stderr after all benchmarks have finished
//
// Helps users quickly decide which backend / tokenizer combination best fits
// their use-case (vocabulary size, OOV rate, memory budget, mutation needs).
// ===========================================================================

fn bench_summary(c: &mut Criterion) {
    eprintln!();
    eprintln!("╔══════════════════════════════════════════════════════════════════════════════╗");
    eprintln!("║          TOKENIZER & DICTIONARY BACKEND — SELECTION GUIDE                   ║");
    eprintln!("╠══════════════════╦═════════════════╦═════════════════╦══════════════════════╣");
    eprintln!("║ Criterion        ║ TrieChar (def.) ║ TrieCharLegacy  ║ FstDict              ║");
    eprintln!("╠══════════════════╬═════════════════╬═════════════════╬══════════════════════╣");
    eprintln!("║ prefix_ref speed ║ fastest         ║ same as TrieChar║ 29–54× slower        ║");
    eprintln!("║ contain() speed  ║ O(k) trie walk  ║ O(1) hash       ║ O(1) hash + FST      ║");
    eprintln!("║ Memory (62k wds) ║ ~43 MB          ║ ~49 MB (+12%)   ║ ~0.9 MB (49× less)   ║");
    eprintln!("║ Build time (62k) ║ fastest         ║ +50%            ║ similar to legacy    ║");
    eprintln!("║ add/remove       ║ O(n·k) clone    ║ O(n·k) clone    ║ O(1) hash delta      ║");
    eprintln!("║ Large text e2e   ║ YES             ║ YES             ║ NO — too slow        ║");
    eprintln!("╚══════════════════╩═════════════════╩═════════════════╩══════════════════════╝");
    eprintln!();
    eprintln!("  Tokenizer wrappers:");
    eprintln!("    NewmmTokenizer<TrieChar>       — default, best all-round speed");
    eprintln!("    NewmmTokenizer<TrieCharLegacy> — identical throughput; O(1) contain()");
    eprintln!("    NewmmFstTokenizer              — small texts / memory-critical only");
    eprintln!("    DeepcutTokenizer               — neural, no dictionary, highest accuracy");
    eprintln!();
    eprintln!("  Dictionary size vs. tokenizer choice:");
    eprintln!("  ┌──────────────┬──────────────────────────────────────────────────────────┐");
    eprintln!("  │ Dict size    │ Recommendation                                           │");
    eprintln!("  ├──────────────┼──────────────────────────────────────────────────────────┤");
    eprintln!("  │ 1k words     │ Any backend works; FstDict fine for small-text workloads │");
    eprintln!("  │ 10k words    │ Trie preferred; avoid FstDict on texts > ~100 KB         │");
    eprintln!("  │ 62k words    │ Use TrieChar; FstDict only if memory budget < 1 MB       │");
    eprintln!("  └──────────────┴──────────────────────────────────────────────────────────┘");
    eprintln!();
    eprintln!("  OOV (out-of-vocabulary) characteristics:");
    eprintln!("    Low OOV  (vocab covers text well)  : all backends near-optimal");
    eprintln!("    High OOV (many unknown words)       : trie degrades gracefully;");
    eprintln!(
        "                                          FstDict degrades fast (byte FST fallback)"
    );
    eprintln!();
    eprintln!("  Typical tokenization throughput (TrieChar, dict-words-th, release build):");
    eprintln!("    text-wikipedia-s  (~2.4 KB) :  ~µs-range per call");
    eprintln!("    text-wikipedia-m  (~41 KB)  :  ~ms-range per call");
    eprintln!("    text-wikipedia-l  (~615 KB) :  ~tens of ms per call");
    eprintln!("    text-ws-social    (~6.3 MB) :  ~hundreds of ms per call");
    eprintln!();
    eprintln!("  See target/criterion/ for full HTML reports with per-run timing details.");
    eprintln!();

    // A trivial benchmark so Criterion registers this group properly.
    let mut group = c.benchmark_group("summary");
    group.bench_function("noop", |b| b.iter(|| black_box(())));
    group.finish();
}

// ---------------------------------------------------------------------------
// Register all benchmark groups
// ---------------------------------------------------------------------------

#[cfg(feature = "deepcut")]
criterion_group!(
    benches,
    bench_dict_construction,
    bench_prefix_lookup,
    bench_dict_operations,
    bench_memory_footprint,
    bench_clone_cost,
    bench_dict_backend_tokenization,
    bench_tokenizer_performance,
    bench_deepcut_chunking_overhead,
    bench_summary,
);

#[cfg(not(feature = "deepcut"))]
criterion_group!(
    benches,
    bench_dict_construction,
    bench_prefix_lookup,
    bench_dict_operations,
    bench_memory_footprint,
    bench_clone_cost,
    bench_dict_backend_tokenization,
    bench_tokenizer_performance,
    bench_summary,
);

criterion_main!(benches);
