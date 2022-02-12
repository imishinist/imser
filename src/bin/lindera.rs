use lindera::tokenizer::{Tokenizer, TokenizerConfig};
use lindera_core::viterbi::{Mode, Penalty};

fn main() {
    let mut tokenizer = Tokenizer::with_config(TokenizerConfig {
        dict_path: None,
        user_dict_path: None,
        user_dict_bin_path: None,

        // for search
        mode: Mode::Decompose(Penalty::default()),
    })
    .unwrap();

    let sentences = ["関西国際空港限定トートバッグ", "すもももももももものうち"];

    for sentence in sentences {
        let tokens = tokenizer.tokenize(sentence).unwrap();
        println!("sentence={:?}", sentence);
        for token in tokens {
            println!("token={:?}, len={}", token.text, token.text.len());
        }
        println!();
    }
}
