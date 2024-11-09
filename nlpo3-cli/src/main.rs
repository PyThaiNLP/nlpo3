// SPDX-FileCopyrightText: 2024 PyThaiNLP Project
// SPDX-License-Identifier: Apache-2.0

use clap::Clap;
use nlpo3::tokenizer::newmm_custom::Newmm;
use nlpo3::tokenizer::tokenizer_trait::Tokenizer;
use std::io;
use std::io::BufRead;

#[derive(Clap, Debug)]
#[clap(name = "nlpo3")]
struct App {
    #[clap(subcommand)]
    subcommand: SubCommand,
}

#[derive(Clap, Debug)]
enum SubCommand {
    /// Tokenize a string into words.
    #[clap()]
    Segment(SegmentOpts),
}

#[derive(Clap, Debug)]
struct SegmentOpts {
    #[clap(short = 'd', long, default_value = "default")]
    dict_path: String,

    #[clap(short = 's', long, default_value = "|")]
    word_delimiter: String,

    /// Run in safe mode to avoid long running edge cases
    #[clap(short = 'z', long)]
    safe: bool,

    /// Run in multithread mode
    #[clap(short = 'p', long)]
    parallel: bool,
}

fn main() {
    let opt = App::parse();

    let SubCommand::Segment(segment_opts) = opt.subcommand;
    let dict_path = match segment_opts.dict_path.as_str() {
        "default" => None,
        dict_name => Some(dict_name),
    };

    let newmm = Newmm::new(dict_path);
    for line_opt in io::stdin().lock().lines() {
        let cleaned_line = match line_opt {
            Ok(line) => line.trim_end_matches('\n').to_string(),
            Err(e) => panic!("Cannot read line {}", e),
        };
        let toks = newmm.segment(
            &cleaned_line,
            Some(segment_opts.safe),
            Some(segment_opts.parallel),
        );
        println!("{}", toks.join(segment_opts.word_delimiter.as_str()));
    }
}
