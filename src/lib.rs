#[derive(Debug, PartialEq)]
enum TokenKind {
    Term(String),
    Punct(String),
}

#[derive(Debug, PartialEq)]
pub struct Token {
    kind: TokenKind,
    loc: usize,
}

impl Token {
    fn new_term(term: &str, loc: usize) -> Self {
        Self {
            kind: TokenKind::Term(term.to_string()),
            loc,
        }
    }

    fn new_punct(punct: &str, loc: usize) -> Self {
        Self {
            kind: TokenKind::Punct(punct.to_string()),
            loc,
        }
    }
}

fn tokenize(sentence: &String) -> Vec<Token> {
    let mut tokens = Vec::new();

    let mut term = String::new();
    let mut base = 0;
    for c in sentence.chars() {
        if c.is_whitespace() {
            if term.len() != 0 {
                tokens.push(Token::new_term(term.as_str(), base - term.len()));
                term.clear();
            }

            base += 1;
            continue;
        }
        if c.is_ascii_punctuation() {
            tokens.push(Token::new_term(term.as_str(), base - term.len()));
            term.clear();

            term.push(c);
            tokens.push(Token::new_punct(term.as_str(), base));

            term.clear();
            base += 1;
            continue;
        }

        base += 1;
        term.push(c);
    }
    if term.len() != 0 {
        tokens.push(Token::new_term(term.as_str(), base - term.len()));
    }

    tokens
}

pub fn search_term(sentence: &String, term: &String) -> Vec<usize> {
    let tokens = tokenize(sentence);
    let mut positions = Vec::new();

    for token in tokens {
        match token.kind {
            TokenKind::Term(t) if &t == term => {
                positions.push(token.loc);
            }
            _ => continue,
        }
    }
    positions
}

pub fn search_main(sentences: &[String], term: &String) -> Vec<Vec<usize>> {
    let mut positions_per_sentences = Vec::new();
    for sentence in sentences {
        positions_per_sentences.push(search_term(sentence, term));
    }
    positions_per_sentences
}

#[cfg(test)]
mod tests {
    use crate::{search_main, search_term, tokenize, Token};

    #[test]
    fn tokenize_test() {
        let sentence = "".to_string();
        assert_eq!(tokenize(&sentence), vec![]);

        let sentence = "I am  Taisuke".to_string();

        assert_eq!(
            tokenize(&sentence),
            vec![
                Token::new_term("I", 0),
                Token::new_term("am", 2),
                Token::new_term("Taisuke", 6)
            ]
        );

        let sentence = "I am Taisuke.".to_string();
        assert_eq!(
            tokenize(&sentence),
            vec![
                Token::new_term("I", 0),
                Token::new_term("am", 2),
                Token::new_term("Taisuke", 5),
                Token::new_punct(".", 12)
            ]
        );

        let sentence = "What is that?".to_string();
        assert_eq!(
            tokenize(&sentence),
            vec![
                Token::new_term("What", 0),
                Token::new_term("is", 5),
                Token::new_term("that", 8),
                Token::new_punct("?", 12)
            ]
        );

        let sentence = "What's that?".to_string();
        assert_eq!(
            tokenize(&sentence),
            vec![
                Token::new_term("What", 0),
                Token::new_punct("'", 4),
                Token::new_term("s", 5),
                Token::new_term("that", 7),
                Token::new_punct("?", 11)
            ]
        );
    }

    #[test]
    fn search_term_test() {
        let sentence = "I am Taisuke".to_string();
        let term = "Taisuke".to_string();
        assert_eq!(search_term(&sentence, &term), vec![5]);

        let sentence = "that that is is that that is not is not is that it it is".to_string();
        let term = "that".to_string();
        assert_eq!(search_term(&sentence, &term), vec![0, 5, 16, 21, 43]);

        let sentence = "I am Taisuke".to_string();
        let term = "foo".to_string();
        assert_eq!(search_term(&sentence, &term), vec![]);
    }

    #[test]
    fn search_main_test() {
        let sentences = vec![
            "I am Taisuke".to_string(),
            "that that is is that that is not is not is that it it is".to_string(),
        ];
        let term = "Taisuke".to_string();
        assert_eq!(search_main(&sentences, &term), vec![vec![5], vec![]]);

        let term = "that".to_string();
        assert_eq!(
            search_main(&sentences, &term),
            vec![vec![], vec![0, 5, 16, 21, 43]]
        );

        let term = "foo".to_string();
        assert_eq!(search_main(&sentences, &term), vec![vec![], vec![]]);
    }
}
