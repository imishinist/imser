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

    let docs = sentences.iter().map(|s| doc!(s.as_str())).collect();
    let doc_ids = imser::search_main(docs, &term);
    if doc_ids.is_empty() {
        eprintln!("term not found: {}", &term);
    }
    for doc_id in doc_ids {
        println!("{:?}", doc_id);
    }
}
