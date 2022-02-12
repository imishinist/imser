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

    let sentences = [
        "関西国際空港限定トートバッグ",
        "すもももももももものうち",
        "大変でした。　そうですね。",
        "赤い",
        "静かだ",
        "見て",
        "見えない",
        "ゆっくり",
        "大きな",
        "そして",
        "あら",
    ];

    for sentence in sentences {
        let tokens = tokenizer.tokenize(sentence).unwrap();
        println!("sentence={:?}", sentence);
        for token in tokens {
            println!(
                "token={:?}, detail={:?},len={}, len={}",
                token.text,
                token.detail,
                token.detail.len(),
                token.text.len()
            );
        }
        println!();
    }
}
