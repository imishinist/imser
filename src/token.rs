use lindera::tokenizer::{Tokenizer, TokenizerConfig};
use lindera_core::viterbi::{Mode, Penalty};

#[derive(Debug, PartialEq)]
pub enum TokenKind<'a> {
    Term(&'a str),
    Punct(&'a str),
}

#[derive(Debug, PartialEq)]
pub struct Token<'a> {
    pub kind: TokenKind<'a>,

    // offset to the beginning of the word
    pub offset: usize,

    // length of token
    pub length: usize,

    // position of the word in the sentence
    pub position: usize,
}

impl<'a> Token<'a> {
    pub fn new_term(term: &'a str, offset: usize, position: usize) -> Self {
        Self {
            kind: TokenKind::Term(term),
            offset,
            length: term.len(),
            position,
        }
    }

    pub fn new_punct(punct: &'a str, offset: usize, position: usize) -> Self {
        Self {
            kind: TokenKind::Punct(punct),
            offset,
            length: punct.len(),
            position,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum TokenizeType {
    Whitespace,
    Japanese,
}

impl Default for TokenizeType {
    fn default() -> Self {
        TokenizeType::Whitespace
    }
}

pub fn japanese_tokenize(sentence: &str) -> Vec<Token> {
    let mut tokenizer = Tokenizer::with_config(TokenizerConfig {
        dict_path: None,
        user_dict_path: None,
        user_dict_bin_path: None,
        mode: Mode::Decompose(Penalty::default()),
    })
    .unwrap();

    let tokens = tokenizer.tokenize(sentence).unwrap();
    let mut base_offset = 0;
    let mut word_count = 0;

    let mut ret = Vec::with_capacity(tokens.len());
    for token in tokens {
        let term = match token.detail[0].as_str() {
            "名詞" | "動詞" | "形容詞" | "形容動詞" | "助詞" | "助動詞" | "副詞" | "連体詞"
            | "接続詞" | "感動詞" | "UNK" => {
                Token::new_term(token.text, base_offset, word_count)
            }
            "記号" => Token::new_punct(token.text, base_offset, word_count),
            _ => {
                eprintln!("unsupported {:?}", token.detail[0]);
                base_offset += token.text.len();
                word_count += 1;
                continue;
            }
        };
        word_count += 1;
        base_offset += token.text.len();
        ret.push(term);
    }
    ret
}

pub fn whitespace_tokenize(sentence: &str) -> Vec<Token> {
    let mut tokens = Vec::new();

    let mut length = 0;
    let mut base_offset = 0;
    let mut word_count = 0;
    for c in sentence.chars() {
        if c.is_whitespace() {
            if length > 0 {
                tokens.push(Token::new_term(
                    &sentence[base_offset - length..base_offset],
                    base_offset - length,
                    word_count,
                ));
                length = 0;
                word_count += 1;
            }

            base_offset += c.len_utf8();
            continue;
        }
        if c.is_ascii_punctuation() {
            tokens.push(Token::new_term(
                &sentence[base_offset - length..base_offset],
                base_offset - length,
                word_count,
            ));
            word_count += 1;

            tokens.push(Token::new_punct(
                &sentence[base_offset..base_offset + c.len_utf8()],
                base_offset,
                word_count,
            ));
            length = 0;
            word_count += 1;

            base_offset += c.len_utf8();
            continue;
        }

        base_offset += c.len_utf8();
        length += c.len_utf8();
    }
    if length > 0 {
        tokens.push(Token::new_term(
            &sentence[base_offset - length..base_offset],
            base_offset - length,
            word_count,
        ));
    }

    tokens
}

#[cfg(test)]
mod tests {
    use crate::{japanese_tokenize, whitespace_tokenize, Token};

    #[test]
    fn japanese_tokenize_test() {
        assert_eq!(japanese_tokenize(""), vec![]);

        assert_eq!(
            japanese_tokenize("関西国際空港限定トートバッグ"),
            vec![
                Token::new_term("関西", 0, 0),
                Token::new_term("国際", 6, 1),
                Token::new_term("空港", 12, 2),
                Token::new_term("限定", 18, 3),
                Token::new_term("トートバッグ", 24, 4),
            ]
        );

        assert_eq!(
            japanese_tokenize("すもももももももものうち"),
            vec![
                Token::new_term("すもも", 0, 0),
                Token::new_term("も", 9, 1),
                Token::new_term("もも", 12, 2),
                Token::new_term("も", 18, 3),
                Token::new_term("もも", 21, 4),
                Token::new_term("の", 27, 5),
                Token::new_term("うち", 30, 6),
            ]
        );

        // 動詞
        assert_eq!(
            japanese_tokenize("好き"),
            vec![Token::new_term("好き", 0, 0)]
        );
        // 形容詞
        assert_eq!(
            japanese_tokenize("赤い"),
            vec![Token::new_term("赤い", 0, 0)]
        );
        // 形容動詞
        assert_eq!(
            japanese_tokenize("静かだ"),
            vec![Token::new_term("静か", 0, 0), Token::new_term("だ", 6, 1)]
        );
        // 助詞
        assert_eq!(
            japanese_tokenize("見て"),
            vec![Token::new_term("見", 0, 0), Token::new_term("て", 3, 1)]
        );
        // 助動詞
        assert_eq!(
            japanese_tokenize("見えない"),
            vec![Token::new_term("見え", 0, 0), Token::new_term("ない", 6, 1)]
        );
        // 副詞
        assert_eq!(
            japanese_tokenize("ゆっくり"),
            vec![Token::new_term("ゆっくり", 0, 0)]
        );
        // 連体詞
        assert_eq!(
            japanese_tokenize("大きな"),
            vec![Token::new_term("大きな", 0, 0)]
        );
        // 接続詞
        assert_eq!(
            japanese_tokenize("そして"),
            vec![Token::new_term("そして", 0, 0)]
        );
        // 感動詞
        assert_eq!(
            japanese_tokenize("あら"),
            vec![Token::new_term("あら", 0, 0)]
        );
    }

    #[test]
    fn whitespace_tokenize_test() {
        let sentence = "".to_string();
        assert_eq!(whitespace_tokenize(&sentence), vec![]);

        let sentence = "I am  Taisuke".to_string();

        assert_eq!(
            whitespace_tokenize(&sentence),
            vec![
                Token::new_term("I", 0, 0),
                Token::new_term("am", 2, 1),
                Token::new_term("Taisuke", 6, 2),
            ]
        );

        let sentence = "I am Taisuke.".to_string();
        assert_eq!(
            whitespace_tokenize(&sentence),
            vec![
                Token::new_term("I", 0, 0),
                Token::new_term("am", 2, 1),
                Token::new_term("Taisuke", 5, 2),
                Token::new_punct(".", 12, 3),
            ]
        );

        let sentence = "What is that?".to_string();
        assert_eq!(
            whitespace_tokenize(&sentence),
            vec![
                Token::new_term("What", 0, 0),
                Token::new_term("is", 5, 1),
                Token::new_term("that", 8, 2),
                Token::new_punct("?", 12, 3),
            ]
        );

        let sentence = "What's that?".to_string();
        assert_eq!(
            whitespace_tokenize(&sentence),
            vec![
                Token::new_term("What", 0, 0),
                Token::new_punct("'", 4, 1),
                Token::new_term("s", 5, 2),
                Token::new_term("that", 7, 3),
                Token::new_punct("?", 11, 4),
            ]
        );

        let sentence = "すもも も もも も もも の うち";
        assert_eq!(
            whitespace_tokenize(sentence),
            vec![
                Token::new_term("すもも", 0, 0),
                Token::new_term("も", 10, 1),
                Token::new_term("もも", 14, 2),
                Token::new_term("も", 21, 3),
                Token::new_term("もも", 25, 4),
                Token::new_term("の", 32, 5),
                Token::new_term("うち", 36, 6),
            ]
        );
    }
}
