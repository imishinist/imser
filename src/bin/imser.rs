extern crate imser;

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

    let positions_per_sentence = imser::search_main(sentences, &term);
    for positions in positions_per_sentence {
        if positions.len() == 0 {
            eprintln!("term not found: {}", &term);
            continue;
        }
        println!("{:?}", positions);
    }
}
