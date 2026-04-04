// SPDX-FileCopyrightText: 2024-2026 PyThaiNLP Project
// SPDX-License-Identifier: Apache-2.0

//! Performance and memory benchmarks for Thai dictionary backends and tokenizers.
//!
//! # Dictionary backends compared
//!
//! | Backend | Prefix-lookup | Memory | `contain()` |
//! |---------|--------------|--------|-------------|
//! | [`TrieChar`] | O(k) trie walk | lower (trie only) | O(k) trie walk |
//! | [`TrieCharLegacy`] | O(k) trie walk | slightly higher (+HashSet) | O(1) hash lookup |
//! | [`FstDict`] | O(k·B) FST | minimal (~14 B/word) | O(k·B) FST |
//!
//! # Benchmark groups
//!
//! | Group | What is measured |
//! |-------|-----------------|
//! | `dict_construction` | Build time: all 3 backends × 4 dictionary sizes |
//! | `prefix_lookup` | Per-query prefix scan: all 3 backends × 4 dict sizes |
//! | `dict_operations` | add / remove / contain: all 3 backends × 4 dict sizes |
//! | `full_tokenization` | End-to-end segment(): all 3 NewMM backends |
//! | `memory_footprint` | Heap-size estimates printed to stderr |
//! | `clone_cost` | O(1) Arc clone verification |
//!
//! # Dictionaries
//!
//! | File | Words | Notes |
//! |------|------:|-------|
//! | `500-short.txt` | 500 | 3–10 chars, diverse prefixes |
//! | `500-long.txt` | 500 | 15–36 chars, stress test for long tokens |
//! | `10k.txt` | 10 000 | mid-size realistic vocabulary |
//! | `words_th.txt` | 62 018 | full Thai dictionary |
//!
//! Run with:
//! ```sh
//! cargo bench
//! # with Deepcut:
//! cargo bench --features deepcut
//! # specific group only:
//! cargo bench -- dict_construction
//! ```
//! HTML reports land in `target/criterion/`.

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

const SHORT_TEXT: &str = "พิสูจน์ได้ค่ะสวัสดีประเทศไทย";

const MEDIUM_TEXT: &str = "\
ไต้หวัน (แป่ะเอ๋ยี้: Tâi-oân; ไต่อวัน) หรือ ไถวาน \
(อักษรโรมัน: Taiwan; จีนตัวย่อ: 台湾; จีนตัวเต็ม: 臺灣/台灣; พินอิน: \
Táiwān; ไถวาน) หรือชื่อทางการว่า สาธารณรัฐจีน (จีนตัวย่อ: 中华民国; \
จีนตัวเต็ม: 中華民國; พินอิน: Zhōnghuá Mínguó)";

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

/// The four benchmark dictionaries. Ordered from smallest to largest.
const BENCH_DICTS: &[(&str, &str)] = &[
    ("500-short", "tests/data/500-short.txt"),
    ("500-long", "tests/data/500-long.txt"),
    ("10k", "tests/data/10k.txt"),
    ("words_th", "tests/data/words_th.txt"),
];

fn dict_path(filename: &str) -> String {
    format!("{}/{}", BASE_PATH, filename)
}

fn load_dict_words(filename: &str) -> Vec<String> {
    let path = dict_path(filename);
    std::fs::read_to_string(&path)
        .unwrap_or_else(|_| panic!("dict file not found: {}", path))
        .lines()
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty())
        .collect()
}

/// Convenience: load the full words_th dict (used by tokenizer benchmarks).
fn load_word_list() -> Vec<String> {
    load_dict_words("tests/data/words_th.txt")
}

// ===========================================================================
// 1. Dictionary construction — TrieChar vs TrieCharLegacy vs FstDict
//    Parameterised over all 4 dictionary sizes.
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
// 2. Dictionary prefix lookup — TrieChar vs TrieCharLegacy vs FstDict
//    Parameterised over all 4 dictionary sizes and 3 query strings.
// ===========================================================================

fn bench_prefix_lookup(c: &mut Criterion) {
    let queries: &[(&str, &str)] = &[
        ("short_thai", "สวัสดีครับ"),
        ("mixed", "ไต้หวัน1984"),
        ("medium_thai", "อาชญากรรมทางการแพทย์"),
    ];

    let mut group = c.benchmark_group("prefix_lookup");

    for (dict_name, dict_file) in BENCH_DICTS {
        let words = load_dict_words(dict_file);
        let char_words: Vec<CharString> = words.iter().map(|w| CharString::new(w)).collect();
        let trie = TrieChar::new(&char_words);
        let legacy = TrieCharLegacy::new(&char_words);
        let fst_dict = FstDict::from_words(words.iter().map(|s| s.as_str())).unwrap();

        for (query_label, text) in queries {
            let cs = CharString::new(text);
            let id_suffix = format!("{}/{}", dict_name, query_label);

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
                |b, t| b.iter(|| black_box(fst_dict.prefix_lengths(black_box(t)))),
            );
        }
    }

    group.finish();
}

// ===========================================================================
// 3. Dictionary operations — add / remove / contain
//    Parameterised over all 4 dictionary sizes.
//
//    The query word "กาแฟ" (coffee, 4 chars) is present in words_th.txt and
//    most sub-dicts.  Operations are benchmarked on cloned dicts to avoid
//    cumulative mutation side-effects.
// ===========================================================================

fn bench_dict_operations(c: &mut Criterion) {
    let test_word_cs = CharString::new("กาแฟ");
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

        // --- add (clone dict first so each iteration starts from baseline) ---
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

        // --- remove (clone dict first so each iteration starts from baseline) ---
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
// 4. End-to-end tokenization — NewmmTokenizer with all three dict backends
//
// NewmmTokenizer<TrieChar>       = optimized trie (default, lowest memory, fast)
// NewmmTokenizer<TrieCharLegacy> = legacy trie with HashSet
// NewmmFstTokenizer              = FST backend (memory-efficient)
// ===========================================================================

fn bench_full_tokenization(c: &mut Criterion) {
    let path = dict_path("tests/data/words_th.txt");
    let word_list = load_word_list();

    let tok_trie = NewmmTokenizer::new(&path).unwrap();
    let tok_legacy = NewmmTokenizer::<TrieCharLegacy>::from_word_list(word_list);
    let tok_fst = NewmmFstTokenizer::new(&path).unwrap();

    #[cfg(feature = "deepcut")]
    let tok_deepcut = nlpo3::tokenizer::deepcut::DeepcutTokenizer::new()
        .expect("deepcut: ONNX model failed to load");

    let mut group = c.benchmark_group("full_tokenization");

    for (label, text) in &[
        ("short", SHORT_TEXT),
        ("medium", MEDIUM_TEXT),
        ("long", LONG_TEXT),
    ] {
        group.throughput(Throughput::Bytes(text.len() as u64));

        // NewmmTokenizer<TrieChar> — optimized trie (default)
        group.bench_with_input(
            BenchmarkId::new("NewmmTokenizer/safe=false", label),
            text,
            |b, t| b.iter(|| black_box(tok_trie.segment(black_box(t)).unwrap())),
        );
        group.bench_with_input(
            BenchmarkId::new("NewmmTokenizer/safe=true", label),
            text,
            |b, t| {
                b.iter(|| {
                    black_box(
                        tok_trie
                            .segment_with_options(black_box(t), true, None)
                            .unwrap(),
                    )
                })
            },
        );

        // NewmmTokenizer<TrieCharLegacy> — legacy trie with HashSet
        group.bench_with_input(
            BenchmarkId::new("NewmmLegacyTokenizer/safe=false", label),
            text,
            |b, t| b.iter(|| black_box(tok_legacy.segment(black_box(t)).unwrap())),
        );
        group.bench_with_input(
            BenchmarkId::new("NewmmLegacyTokenizer/safe=true", label),
            text,
            |b, t| {
                b.iter(|| {
                    black_box(
                        tok_legacy
                            .segment_with_options(black_box(t), true, None)
                            .unwrap(),
                    )
                })
            },
        );

        // NewmmFstTokenizer — FST backend (memory-efficient)
        group.bench_with_input(
            BenchmarkId::new("NewmmFstTokenizer/safe=false", label),
            text,
            |b, t| b.iter(|| black_box(tok_fst.segment(black_box(t)).unwrap())),
        );
        group.bench_with_input(
            BenchmarkId::new("NewmmFstTokenizer/safe=true", label),
            text,
            |b, t| {
                b.iter(|| {
                    black_box(
                        tok_fst
                            .segment_with_options(black_box(t), true, None)
                            .unwrap(),
                    )
                })
            },
        );

        // DeepcutTokenizer — CNN/ONNX (only with --features deepcut)
        #[cfg(feature = "deepcut")]
        group.bench_with_input(BenchmarkId::new("DeepcutTokenizer", label), text, |b, t| {
            b.iter(|| black_box(tok_deepcut.segment(black_box(t)).unwrap()))
        });
    }
    group.finish();
}

// ===========================================================================
// 5. Memory footprint — printed to stderr during benchmark run
// ===========================================================================

fn bench_memory_footprint(c: &mut Criterion) {
    use std::mem;

    let word_list = load_word_list();
    let n_words = word_list.len();
    let fst_dict = FstDict::from_words(word_list.iter().map(|s| s.as_str())).unwrap();

    eprintln!("\n╔══════════════════════════════════════════════════════╗");
    eprintln!("║          Memory footprint analysis                   ║");
    eprintln!("╚══════════════════════════════════════════════════════╝");

    // --- struct stack sizes ---
    eprintln!("Stack sizes:");
    eprintln!("  CharString: {} bytes", mem::size_of::<CharString>());

    // --- per-character heap usage ---
    let text = MEDIUM_TEXT;
    let n_chars = text.chars().count();
    let utf8_bytes = text.len();

    // UTF-8 bytes + u32 positions table
    let heap_per_char =
        (utf8_bytes + (n_chars + 1) * mem::size_of::<u32>()) as f64 / n_chars as f64;

    eprintln!(
        "\nPer-character heap ({} chars, mixed Thai/Latin/digits):",
        n_chars
    );
    eprintln!(
        "  CharString (UTF-8 source + u32 pos table): {:.1} bytes/char",
        heap_per_char
    );

    // --- dictionary memory ---
    let fst_bytes = fst_dict.fst_size_bytes();
    let total_chars: usize = word_list.iter().map(|w| w.chars().count()).sum();
    // 48 bytes per String: 24 bytes stack (ptr+len+cap) + ~24 bytes heap overhead.
    let words_set_bytes: usize = word_list.iter().map(|w| w.len() + 48).sum();
    // ~80 bytes per trie edge: 24-byte TrieNode stack + HashMap bucket (~56 bytes).
    let trie_base_estimate = total_chars * 80;
    let trie_new_estimate = trie_base_estimate; // no HashSet in new TrieChar
    let trie_legacy_estimate = trie_base_estimate + words_set_bytes; // + HashSet overhead

    eprintln!("\nDictionary ({} words):", n_words);
    eprintln!(
        "  FstDict base FST:          {:>8} bytes  ({:.1} bytes/word)",
        fst_bytes,
        fst_bytes as f64 / n_words as f64
    );
    eprintln!(
        "  TrieChar (new, no HashSet): ~{:>7} MB  (~{:.0} bytes/word)",
        trie_new_estimate / 1_000_000,
        trie_new_estimate as f64 / n_words as f64
    );
    eprintln!(
        "  TrieCharLegacy (+HashSet):  ~{:>7} MB  (~{:.0} bytes/word)  (+{:.0} MB)",
        trie_legacy_estimate / 1_000_000,
        trie_legacy_estimate as f64 / n_words as f64,
        words_set_bytes as f64 / 1_000_000.0
    );
    eprintln!(
        "  → FstDict is ~{:.0}× smaller than TrieChar",
        trie_new_estimate as f64 / fst_bytes as f64
    );
    eprintln!(
        "  → FstDict is ~{:.0}× smaller than TrieCharLegacy",
        trie_legacy_estimate as f64 / fst_bytes as f64
    );

    #[cfg(feature = "deepcut")]
    {
        let model_path = format!("{}/model/deepcut.onnx", BASE_PATH);
        let model_bytes = std::fs::metadata(&model_path).map(|m| m.len()).unwrap_or(0);
        eprintln!("\nDeepcutTokenizer:");
        eprintln!("  ONNX model (bundled): {} bytes", model_bytes);
        eprintln!("  No dictionary — model weights are fixed-size.");
    }
    eprintln!();

    let mut group = c.benchmark_group("memory_footprint");
    group.bench_function("CharString::new/overhead", |b| {
        b.iter(|| {
            let cs = CharString::new(black_box(MEDIUM_TEXT));
            black_box(mem::size_of::<CharString>() + cs.as_str().len() + (cs.chars_len() + 1) * 4)
        })
    });
    group.finish();
}

// ===========================================================================
// 6. Clone cost — Arc-backed dicts make clone O(1)
// ===========================================================================

fn bench_clone_cost(c: &mut Criterion) {
    let path = dict_path("tests/data/words_th.txt");
    let word_list = load_word_list();
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
// 7. Deepcut chunking overhead
// ===========================================================================

#[cfg(feature = "deepcut")]
fn bench_deepcut_chunking_overhead(c: &mut Criterion) {
    use std::time::Duration;

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

// ---------------------------------------------------------------------------
// Register all benchmark groups
// ---------------------------------------------------------------------------

#[cfg(feature = "deepcut")]
criterion_group!(
    benches,
    bench_dict_construction,
    bench_prefix_lookup,
    bench_dict_operations,
    bench_full_tokenization,
    bench_memory_footprint,
    bench_clone_cost,
    bench_deepcut_chunking_overhead,
);

#[cfg(not(feature = "deepcut"))]
criterion_group!(
    benches,
    bench_dict_construction,
    bench_prefix_lookup,
    bench_dict_operations,
    bench_full_tokenization,
    bench_memory_footprint,
    bench_clone_cost,
);
criterion_main!(benches);
