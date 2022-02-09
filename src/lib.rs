use std::collections::HashMap;

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

fn tokenize(sentence: &str) -> Vec<Token> {
    let mut tokens = Vec::new();

    let mut term = String::new();
    let mut base = 0;
    for c in sentence.chars() {
        if c.is_whitespace() {
            if !term.is_empty() {
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
    if !term.is_empty() {
        tokens.push(Token::new_term(term.as_str(), base - term.len()));
    }

    tokens
}

pub struct Document {
    id: usize,
    body: String,
}

impl Document {
    pub fn new(body: String) -> Self {
        Self { id: 0, body }
    }

    pub fn set_id(&mut self, id: usize) {
        self.id = id;
    }
}

#[macro_export]
macro_rules! doc {
    ($x:expr) => {
        crate::Document::new($x.to_string())
    };
    ($x:expr, $y:expr) => {{
        let mut doc = crate::Document::new($x.to_string());
        doc.set_id($y);
        doc
    }};
}

type Term = String;

#[derive(Debug, PartialEq)]
struct PositionalIndex {
    postings: HashMap<Term, PostingList>,
}

impl PositionalIndex {
    fn new() -> Self {
        PositionalIndex {
            postings: HashMap::new(),
        }
    }

    fn push_posting(&mut self, term: Term, posting: PostingData) {
        let posting_list = self.postings.entry(term).or_insert_with(PostingList::new);

        posting_list.push(posting);
    }
}

#[derive(Debug, PartialEq)]
struct PostingList {
    postings: Vec<PostingData>,
}

impl PostingList {
    fn new() -> Self {
        Self {
            postings: Vec::new(),
        }
    }

    fn push(&mut self, posting: PostingData) {
        self.postings.push(posting);
    }
}

#[derive(Debug, PartialEq)]
struct PostingData {
    doc_id: usize,
    positions: Vec<usize>,
}

#[derive(Debug, PartialEq)]
struct IndexWriter {
    // term => index mapping
    dict: HashMap<Term, usize>,
    // index => term mapping
    reversed_dict: HashMap<usize, Term>,
    dict_len: usize,

    // (doc_id, dict_index, positions)
    term_positions: Vec<(usize, usize, Vec<usize>)>,
}

impl IndexWriter {
    fn new() -> Self {
        Self {
            dict: HashMap::new(),
            reversed_dict: HashMap::new(),
            dict_len: 0,
            term_positions: Vec::new(),
        }
    }

    fn write(&mut self, doc: &Document) {
        let tokens = tokenize(doc.body.as_str());

        let mut data: HashMap<usize, Vec<usize>> = HashMap::new();
        for token in tokens {
            match token.kind {
                TokenKind::Term(t) => {
                    let index = self.dict.entry(t.clone()).or_insert_with(|| {
                        let len = self.dict_len;
                        self.dict_len += 1;
                        len
                    });
                    self.reversed_dict
                        .entry(*index)
                        .or_insert_with(|| t.clone());
                    data.entry(*index).or_insert_with(Vec::new).push(token.loc);
                }
                _ => continue,
            }
        }
        for (index, positions) in data.into_iter() {
            self.term_positions.push((doc.id, index, positions));
        }
    }

    fn build(self) -> PositionalIndex {
        let mut index = PositionalIndex::new();
        for (doc_id, idx, positions) in self.term_positions {
            let term = self.reversed_dict.get(&idx).unwrap();
            index.push_posting(term.clone(), PostingData { doc_id, positions });
        }

        index
    }
}

fn search_term(index: &PositionalIndex, term: &Term) -> HashMap<usize, Vec<usize>> {
    let posting_list = match index.postings.get(term.as_str()) {
        None => return HashMap::new(),
        Some(posting_list) => posting_list,
    };

    let mut result = HashMap::new();
    for posting in posting_list.postings.iter() {
        result.insert(posting.doc_id, posting.positions.clone());
    }
    result
}

pub fn search_main(docs: &[Document], term: &Term) -> HashMap<usize, Vec<usize>> {
    let mut index_writer = IndexWriter::new();
    for doc in docs {
        index_writer.write(doc);
    }
    let index = index_writer.build();

    search_term(&index, term)
}

#[cfg(test)]
mod tests {
    use crate::{search_main, search_term, tokenize, IndexWriter, Token};

    macro_rules! map (
        () => {
            std::collections::HashMap::new()
        };
        { $($key:expr => $val:expr),+ } => {
            {
                let mut h = std::collections::HashMap::new();
                $(
                    h.insert($key.into(), $val.into());
                )+
                h
            }
        };
    );
    macro_rules! index (
        { $($key:expr => $val:expr),+ } => {
            {
                let mut postings = std::collections::HashMap::new();
                $(
                    postings.insert($key.into(), $val.into());
                )+
                $crate::PositionalIndex { postings }
            }
        };
    );
    macro_rules! posting (
        () => {
            $crate::PostingList::new()
        };
        { $($key:expr => $value:expr),+ } => {
            {
                let mut posting_list = $crate::PostingList::new();
                $(
                    posting_list.push($crate::PostingData { doc_id: $key, positions: $value });
                )+

                posting_list
            }
        };
    );

    #[test]
    fn indexing_test() {
        let mut index_writer = IndexWriter::new();
        index_writer.write(&doc!("I am Taisuke", 1));
        index_writer.write(&doc!(
            "that that is is that that is not is not is that it it is",
            2
        ));

        let index = index! {
            "I" => posting! { 1 => vec![0] },
            "am" => posting! { 1 => vec![2] },
            "Taisuke" => posting! { 1 => vec![5] },
            "that" => posting! { 2 => vec![0, 5, 16, 21, 43] },
            "is" => posting! { 2 => vec![10, 13, 26, 33, 40, 54] },
            "not" => posting! { 2 => vec![29, 36] },
            "it" => posting! { 2 => vec![48, 51] }
        };
        assert_eq!(index_writer.build(), index);
    }

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
                Token::new_term("Taisuke", 6),
            ]
        );

        let sentence = "I am Taisuke.".to_string();
        assert_eq!(
            tokenize(&sentence),
            vec![
                Token::new_term("I", 0),
                Token::new_term("am", 2),
                Token::new_term("Taisuke", 5),
                Token::new_punct(".", 12),
            ]
        );

        let sentence = "What is that?".to_string();
        assert_eq!(
            tokenize(&sentence),
            vec![
                Token::new_term("What", 0),
                Token::new_term("is", 5),
                Token::new_term("that", 8),
                Token::new_punct("?", 12),
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
                Token::new_punct("?", 11),
            ]
        );
    }

    #[test]
    fn search_term_test() {
        let mut iw = IndexWriter::new();
        iw.write(&doc!("I am Taisuke", 1));
        iw.write(&doc!(
            "that that is is that that is not is not is that it it is",
            2
        ));
        let index = iw.build();

        let term = "Taisuke".to_string();
        assert_eq!(search_term(&index, &term), map! { 1usize => vec![5] });

        let term = "that".to_string();
        assert_eq!(
            search_term(&index, &term),
            map! { 2usize => vec![0, 5, 16, 21, 43] },
        );

        let term = "foo".to_string();
        assert_eq!(search_term(&index, &term), map! {});
    }

    #[test]
    fn search_main_test() {
        let sentences = vec![
            doc!("I am Taisuke", 1),
            doc!(
                "that that is is that that is not is not is that it it is",
                2
            ),
        ];
        let term = "Taisuke".to_string();
        assert_eq!(search_main(&sentences, &term), map! { 1usize => vec![5] });

        let term = "that".to_string();
        assert_eq!(
            search_main(&sentences, &term),
            map! { 2usize => vec![0, 5, 16, 21, 43] }
        );

        let term = "foo".to_string();
        assert_eq!(search_main(&sentences, &term), map! {});
    }
}
