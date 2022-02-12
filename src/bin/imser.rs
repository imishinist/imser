#[macro_use]
extern crate imser;
use imser::{Document, TokenizeType};

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

    let docs = sentences.iter().map(|s| doc!(s.as_str())).collect();
    let docs = imser::search_main(TokenizeType::Whitespace, docs, &term);
    if docs.is_empty() {
        eprintln!("term not found: {}", &term);
    }
    for doc in docs {
        println!("{}", doc.body);
    }
}
