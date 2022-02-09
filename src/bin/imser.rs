#[macro_use]
extern crate imser;
use imser::Document;

use std::env;
use std::process;

fn main() {
    let argv: Vec<String> = env::args().collect();
    if argv.len() < 3 {
        eprintln!("invalid arguments");
        process::exit(1);
    }

    let term = argv[1].clone();
    let sentences = &argv[2..];

    let mut docs = Vec::new();
    for (id, sentence) in sentences.iter().enumerate() {
        docs.push(doc!(sentence.as_str(), id));
    }

    let positions_per_sentence = imser::search_main(&docs, &term);
    if positions_per_sentence.is_empty() {
        eprintln!("term not found: {}", &term);
    }
    for (_, positions) in positions_per_sentence {
        println!("{:?}", positions);
    }
}
