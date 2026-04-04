// SPDX-FileCopyrightText: 2024-2026 PyThaiNLP Project
// SPDX-License-Identifier: Apache-2.0

use std::io;
use std::io::BufRead;

use clap::{Parser, Subcommand, ValueEnum};

#[cfg(feature = "deepcut")]
use nlpo3::tokenizer::deepcut::DeepcutTokenizer;
use nlpo3::tokenizer::newmm::{NewmmFstTokenizer, NewmmTokenizer};

enum TokenizerWrapper {
    Newmm(NewmmTokenizer),
    Nf(NewmmFstTokenizer),
    #[cfg(feature = "deepcut")]
    Deepcut(DeepcutTokenizer),
}

const DEFAULT_DICT: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/words_th.txt"));
const DEFAULT_PARALLEL_CHUNK_SIZE_STR: &str = "65536";

#[derive(Parser, Debug)]
#[command(name = "nlpo3", about = "Thai natural language processing CLI")]
struct App {
    #[command(subcommand)]
    subcommand: SubCommand,
}

#[derive(Subcommand, Debug)]
enum SubCommand {
    /// Tokenize text into words, reading from standard input line by line.
    Segment(SegmentOpts),
}

/// Tokenizer algorithm to use.
#[derive(ValueEnum, Clone, Debug, Default)]
enum TokenizerKind {
    /// Dictionary-based maximal-matching (default, fastest).
    #[default]
    Newmm,
    /// Dictionary-based maximal-matching with FST backend (lower memory use).
    Nf,
    /// Neural CNN-based tokenizer (deepcut).
    #[cfg(feature = "deepcut")]
    Deepcut,
}

#[derive(Parser, Debug)]
struct SegmentOpts {
    /// Tokenizer to use: newmm (default), nf, deepcut.
    #[arg(short = 't', long, default_value = "newmm")]
    tokenizer: TokenizerKind,

    /// Path to a dictionary file (one word per line).
    /// Use "default" or omit to use the built-in dictionary.
    /// Ignored when --tokenizer deepcut is selected.
    #[arg(short = 'd', long, default_value = "default")]
    dict_path: String,

    /// Token delimiter printed between words.
    #[arg(short = 's', long, default_value = "|")]
    word_delimiter: String,

    /// Enable safe mode to avoid long run times on inputs with many
    /// ambiguous word boundaries.
    #[arg(short = 'z', long)]
    safe: bool,

    /// Enable parallel chunk processing.
    ///
    /// Optionally pass chunk size in bytes. If the flag is provided without
    /// a value, the default chunk size is used.
    ///
    /// When parallel mode is active, text is split into chunks before
    /// tokenization. Token sequences near chunk boundaries may differ from
    /// full-text results. This is acceptable for bulk processing tasks such as
    /// classification and embedding, but may not be suitable for tasks that
    /// require precise token boundaries.
    #[arg(
        short = 'p',
        long = "parallel",
        num_args = 0..=1,
        default_missing_value = DEFAULT_PARALLEL_CHUNK_SIZE_STR
    )]
    parallel_chunk_size: Option<usize>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opt = App::parse();

    let SubCommand::Segment(segment_opts) = opt.subcommand;

    let tokenizer = match segment_opts.tokenizer {
        TokenizerKind::Newmm => {
            let dict_path = match segment_opts.dict_path.as_str() {
                "default" => None,
                dict_name => Some(dict_name),
            };
            match dict_path {
                None => TokenizerWrapper::Newmm(NewmmTokenizer::from_word_list(
                    DEFAULT_DICT.lines().map(|s| s.to_owned()).collect(),
                )),
                Some(path) => TokenizerWrapper::Newmm(NewmmTokenizer::new(path)?),
            }
        }
        TokenizerKind::Nf => {
            let dict_path = match segment_opts.dict_path.as_str() {
                "default" => None,
                dict_name => Some(dict_name),
            };
            match dict_path {
                None => TokenizerWrapper::Nf(NewmmFstTokenizer::from_word_list(
                    DEFAULT_DICT.lines().map(|s| s.to_owned()).collect(),
                )?),
                Some(path) => TokenizerWrapper::Nf(NewmmFstTokenizer::new(path)?),
            }
        }
        #[cfg(feature = "deepcut")]
        TokenizerKind::Deepcut => TokenizerWrapper::Deepcut(DeepcutTokenizer::new()?),
    };

    for line_opt in io::stdin().lock().lines() {
        let line = line_opt?;
        let toks = match &tokenizer {
            TokenizerWrapper::Newmm(tok) => tok.segment_with_options(
                &line,
                segment_opts.safe,
                segment_opts.parallel_chunk_size,
            )?,
            TokenizerWrapper::Nf(tok) => tok.segment_with_options(
                &line,
                segment_opts.safe,
                segment_opts.parallel_chunk_size,
            )?,
            #[cfg(feature = "deepcut")]
            TokenizerWrapper::Deepcut(tok) => {
                tok.segment_with_options(&line, segment_opts.parallel_chunk_size)?
            }
        };
        println!("{}", toks.join(segment_opts.word_delimiter.as_str()));
    }
    Ok(())
}
