use std::collections::HashMap;

#[derive(Debug, PartialEq)]
enum TokenKind {
    Term(String),
    Punct(String),
}

#[derive(Debug, PartialEq)]
pub struct Token {
    kind: TokenKind,

    // offset to the beginning of the word
    offset: usize,

    // length of token
    length: usize,

    // position of the word in the sentence
    position: usize,
}

impl Token {
    fn new_term(term: &str, offset: usize, position: usize) -> Self {
        Self {
            kind: TokenKind::Term(term.to_string()),
            offset,
            length: term.len(),
            position,
        }
    }

    fn new_punct(punct: &str, offset: usize, position: usize) -> Self {
        Self {
            kind: TokenKind::Punct(punct.to_string()),
            offset,
            length: punct.len(),
            position,
        }
    }
}

fn tokenize(sentence: &str) -> Vec<Token> {
    let mut tokens = Vec::new();

    let mut term = String::new();
    let mut base = 0;
    let mut word_count = 0;
    for c in sentence.chars() {
        if c.is_whitespace() {
            if !term.is_empty() {
                tokens.push(Token::new_term(
                    term.as_str(),
                    base - term.len(),
                    word_count,
                ));
                term.clear();
                word_count += 1;
            }

            base += 1;
            continue;
        }
        if c.is_ascii_punctuation() {
            tokens.push(Token::new_term(
                term.as_str(),
                base - term.len(),
                word_count,
            ));
            term.clear();
            word_count += 1;

            term.push(c);
            tokens.push(Token::new_punct(term.as_str(), base, word_count));
            term.clear();
            word_count += 1;

            base += 1;
            continue;
        }

        base += 1;
        term.push(c);
    }
    if !term.is_empty() {
        tokens.push(Token::new_term(
            term.as_str(),
            base - term.len(),
            word_count,
        ));
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

#[derive(Debug)]
struct TermDict {
    term2idx: HashMap<Term, usize>,
    idx2term: HashMap<usize, Term>,
    len: usize,
}

impl TermDict {
    fn new() -> Self {
        Self {
            term2idx: HashMap::new(),
            idx2term: HashMap::new(),
            len: 0,
        }
    }

    fn add_term<T: Into<String>>(&mut self, term: T) -> usize {
        let term = term.into();
        let index = self.term2idx.entry(term.clone()).or_insert_with(|| {
            let len = self.len;
            self.len += 1;
            len
        });
        self.idx2term.entry(*index).or_insert_with(|| term.clone());
        *index
    }

    fn term(&self, idx: usize) -> Option<&Term> {
        self.idx2term.get(&idx)
    }

    #[allow(dead_code)]
    fn index(&self, term: &Term) -> Option<usize> {
        self.term2idx.get(term).copied()
    }
}

#[derive(Debug)]
struct IndexWriter {
    term_dict: TermDict,

    // (doc_id, dict_index, positions)
    term_positions: Vec<(usize, usize, Vec<usize>)>,
}

impl IndexWriter {
    fn new() -> Self {
        Self {
            term_dict: TermDict::new(),
            term_positions: Vec::new(),
        }
    }

    fn write(&mut self, doc: &Document) {
        let tokens = tokenize(doc.body.as_str());

        let mut data: HashMap<usize, Vec<usize>> = HashMap::new();
        for token in tokens {
            match token.kind {
                TokenKind::Term(t) => {
                    let index = self.term_dict.add_term(t);
                    data.entry(index)
                        .or_insert_with(Vec::new)
                        .push(token.offset);
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
            let term = self.term_dict.term(idx).unwrap();
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
    use crate::{search_main, search_term, tokenize, IndexWriter, TermDict, Token};

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
    fn term_dict_test() {
        let mut term_dict = TermDict::new();

        assert_eq!(term_dict.add_term("This"), 0);
        assert_eq!(term_dict.add_term("is"), 1);
        assert_eq!(term_dict.add_term("a"), 2);
        assert_eq!(term_dict.add_term("pen"), 3);

        let term = "This".to_string();
        assert_eq!(term_dict.index(&term), Some(0));
        assert_eq!(term_dict.term(0), Some(&term));
        let term = "is".to_string();
        assert_eq!(term_dict.index(&term), Some(1));
        assert_eq!(term_dict.term(1), Some(&term));
        let term = "pen".to_string();
        assert_eq!(term_dict.index(&term), Some(3));
        assert_eq!(term_dict.term(3), Some(&term));
    }

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
                Token::new_term("I", 0, 0),
                Token::new_term("am", 2, 1),
                Token::new_term("Taisuke", 6, 2),
            ]
        );

        let sentence = "I am Taisuke.".to_string();
        assert_eq!(
            tokenize(&sentence),
            vec![
                Token::new_term("I", 0, 0),
                Token::new_term("am", 2, 1),
                Token::new_term("Taisuke", 5, 2),
                Token::new_punct(".", 12, 3),
            ]
        );

        let sentence = "What is that?".to_string();
        assert_eq!(
            tokenize(&sentence),
            vec![
                Token::new_term("What", 0, 0),
                Token::new_term("is", 5, 1),
                Token::new_term("that", 8, 2),
                Token::new_punct("?", 12, 3),
            ]
        );

        let sentence = "What's that?".to_string();
        assert_eq!(
            tokenize(&sentence),
            vec![
                Token::new_term("What", 0, 0),
                Token::new_punct("'", 4, 1),
                Token::new_term("s", 5, 2),
                Token::new_term("that", 7, 3),
                Token::new_punct("?", 11, 4),
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
