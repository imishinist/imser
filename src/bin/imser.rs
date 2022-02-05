extern crate imser;

use std::env;
use std::process;

fn main() {
    let argv: Vec<String> = env::args().collect();
    if argv.len() != 3 {
        eprintln!("invalid arguments");
        process::exit(1);
    }

    let sentence = argv[1].clone();
    let term = argv[2].clone();

    let pos = match imser::search_main(&sentence, &term) {
        None => {
            eprintln!("term not found: {}", &term);
            process::exit(1);
        }
        Some(pos) => pos,
    };

    println!("{}", pos);
}
