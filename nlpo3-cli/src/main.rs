#[macro_use]
extern crate clap;

use clap::App;
use nlpo3::tokenizer::newmm_custom::Newmm;
use nlpo3::tokenizer::tokenizer_trait::Tokenizer;
use std::io;
use std::io::BufRead;

fn main() {
    let yaml = load_yaml!("cli.yaml");
    let matches = App::from_yaml(yaml).get_matches();

    if let Some(ref matches) = matches.subcommand_matches("segment") {
        let dict_path = match matches.value_of("dict_path") {
            Some("default") => None,
            Some(dict_name) => Some(dict_name),
            None => None,
        };
        let word_delim = match matches.value_of("word_delimiter") {
            Some(word_delim) => word_delim,
            None => "|",
        };
        let is_parallel = Some(matches.occurrences_of("p") > 0);
        let is_safe = Some(matches.occurrences_of("z") > 0);

        let newmm = Newmm::new(dict_path);

        for line_opt in io::stdin().lock().lines() {
            let cleaned_line = match line_opt {
                Ok(line) => line.trim_end_matches('\n').to_string(),
                Err(e) => panic!("Cannot read line {}", e),
            };
            let toks = newmm.segment(&cleaned_line, is_safe, is_parallel);
            println!("{}", toks.join(word_delim));
        }
    }
}
