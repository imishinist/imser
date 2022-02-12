use std::cmp::Ordering;
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

    // term.len() returns size of bytes
    let mut term = String::new();
    let mut base_offset = 0;
    let mut word_count = 0;
    for c in sentence.chars() {
        if c.is_whitespace() {
            if !term.is_empty() {
                tokens.push(Token::new_term(
                    term.as_str(),
                    base_offset - term.len(),
                    word_count,
                ));
                term.clear();
                word_count += 1;
            }

            base_offset += c.len_utf8();
            continue;
        }
        if c.is_ascii_punctuation() {
            tokens.push(Token::new_term(
                term.as_str(),
                base_offset - term.len(),
                word_count,
            ));
            term.clear();
            word_count += 1;

            term.push(c);
            tokens.push(Token::new_punct(term.as_str(), base_offset, word_count));
            term.clear();
            word_count += 1;

            base_offset += c.len_utf8();
            continue;
        }

        base_offset += c.len_utf8();
        term.push(c);
    }
    if !term.is_empty() {
        tokens.push(Token::new_term(
            term.as_str(),
            base_offset - term.len(),
            word_count,
        ));
    }

    tokens
}

#[derive(Debug, Clone, PartialEq)]
pub struct Document {
    pub body: String,
}

impl Document {
    pub fn new(body: String) -> Self {
        Self { body }
    }
}

#[macro_export]
macro_rules! doc {
    ($x:expr) => {
        crate::Document::new($x.to_string())
    };
}

type Term = String;

#[derive(Debug, PartialEq)]
struct TermFreq {
    term_count: usize,
    terms: HashMap<Term, usize>,
}

impl TermFreq {
    fn new() -> Self {
        Self {
            term_count: 0,
            terms: HashMap::new(),
        }
    }

    fn put_term(&mut self, term: Term, term_count: usize) {
        self.term_count += term_count;
        self.terms.insert(term, term_count);
    }

    fn tf(&self, term: &Term) -> f32 {
        let term_freq = match self.terms.get(term) {
            None => 0,
            Some(freq) => *freq,
        };

        (term_freq as f32) / (self.term_count as f32)
    }
}

#[derive(Debug, PartialEq)]
struct PositionalIndex {
    doc_count: usize,

    // Term to PostingList mapping
    postings: HashMap<Term, PostingList>,

    // doc_id => Document mapping
    stored: HashMap<usize, Document>,

    // doc_id => TermFreq mapping
    term_freq: HashMap<usize, TermFreq>,
}

impl PositionalIndex {
    fn new(doc_count: usize) -> Self {
        PositionalIndex {
            doc_count,
            postings: HashMap::new(),
            stored: HashMap::new(),
            term_freq: HashMap::new(),
        }
    }

    fn push_posting(&mut self, term: Term, posting: PostingData) {
        let posting_list = self.postings.entry(term).or_insert_with(PostingList::new);

        posting_list.push(posting);
    }

    fn push_term_freq(&mut self, id: usize, term: Term, term_count: usize) {
        self.term_freq
            .entry(id)
            .or_insert_with(TermFreq::new)
            .put_term(term, term_count);
    }

    fn store_document(&mut self, id: usize, doc: Document) {
        self.stored.insert(id, doc);
    }

    fn doc(&self, id: usize) -> Option<&Document> {
        self.stored.get(&id)
    }

    fn idf(&self, term: &Term) -> f32 {
        let term_doc_count = match self.postings.get(term) {
            None => 0,
            Some(pl) => pl.postings.len(),
        } + 1;
        ((self.doc_count as f32) / (term_doc_count as f32))
            .log2()
            .max(0f32)
    }

    fn tf(&self, doc_id: usize, term: &Term) -> f32 {
        match self.term_freq.get(&doc_id) {
            None => 0f32,
            Some(term_freq) => term_freq.tf(term),
        }
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
    seq: usize,

    term_dict: TermDict,

    // (doc_id, dict_index, positions)
    term_positions: Vec<(usize, usize, Vec<usize>)>,

    // (doc_id, Document)
    stored: Vec<(usize, Document)>,
}

impl IndexWriter {
    fn new() -> Self {
        Self {
            seq: 0,
            term_dict: TermDict::new(),
            term_positions: Vec::new(),
            stored: Vec::new(),
        }
    }

    fn seq_incr(&mut self) -> usize {
        let curr = self.seq;
        self.seq += 1;
        curr
    }

    fn write(&mut self, doc: Document) {
        let id = self.seq_incr();
        let tokens = tokenize(doc.body.as_str());

        let mut data: HashMap<usize, Vec<usize>> = HashMap::new();
        for token in tokens {
            match token.kind {
                TokenKind::Term(t) => {
                    let index = self.term_dict.add_term(t);
                    data.entry(index)
                        .or_insert_with(Vec::new)
                        .push(token.position);
                }
                _ => continue,
            }
        }
        for (index, positions) in data.into_iter() {
            self.term_positions.push((id, index, positions));
        }

        self.stored.push((id, doc));
    }

    fn build(self) -> PositionalIndex {
        let mut index = PositionalIndex::new(self.seq);

        for (doc_id, idx, positions) in self.term_positions {
            let term = self.term_dict.term(idx).unwrap();
            index.push_term_freq(doc_id, term.clone(), positions.len());
            index.push_posting(term.clone(), PostingData { doc_id, positions });
        }

        for (id, doc) in self.stored {
            index.store_document(id, doc);
        }

        index
    }
}

#[derive(Debug, PartialEq)]
struct DocAndScore {
    score: f32,
    doc_id: usize,
}

impl DocAndScore {
    fn new_with_score(doc_id: usize, score: f32) -> Self {
        Self { doc_id, score }
    }

    fn score_cmp(&self, other: &Self) -> Ordering {
        let sub = self.score - other.score;
        if sub.abs() <= f32::EPSILON {
            return Ordering::Equal;
        }
        // self > score
        if sub > 0f32 {
            Ordering::Greater
        } else {
            Ordering::Less
        }
    }
}

impl Eq for DocAndScore {}

impl PartialOrd<Self> for DocAndScore {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.score
            .partial_cmp(&other.score)
            .map(|f| f.then_with(|| self.doc_id.cmp(&other.doc_id)))
    }
}

impl Ord for DocAndScore {
    fn cmp(&self, other: &Self) -> Ordering {
        self.score_cmp(other)
            .then_with(|| self.doc_id.cmp(&other.doc_id).reverse())
    }
}

fn search_term(index: &PositionalIndex, term: &Term) -> Vec<usize> {
    let posting_list = match index.postings.get(term.as_str()) {
        None => return Vec::new(),
        Some(posting_list) => posting_list,
    };

    let idf = index.idf(term);

    let mut docs_scores: Vec<DocAndScore> = posting_list
        .postings
        .iter()
        .map(|pl| DocAndScore::new_with_score(pl.doc_id, index.tf(pl.doc_id, term) * idf))
        .collect();
    docs_scores.sort();

    docs_scores.into_iter().map(|ds| ds.doc_id).collect()
}

pub fn search_main(docs: Vec<Document>, term: &Term) -> Vec<Document> {
    let mut index_writer = IndexWriter::new();
    for doc in docs {
        index_writer.write(doc);
    }
    let index = index_writer.build();

    search_term(&index, term)
        .iter()
        .map(|id| index.doc(*id).unwrap().clone())
        .collect()
}

#[cfg(test)]
mod tests {
    use crate::{search_main, search_term, tokenize, IndexWriter, TermDict, Token};

    macro_rules! map (
        () => {
            std::collections::HashMap::new()
        };
        { $($key:expr => $val:expr),* $(,)? } => {
            {
                let mut h = std::collections::HashMap::new();
                $(
                    h.insert($key.into(), $val.into());
                )+
                h
            }
        };
    );
    macro_rules! posting (
        () => {
            $crate::PostingList::new()
        };
        { $($key:expr => $value:expr),* $(,)? } => {
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
    fn tfidf_test() {
        let mut index_writer = IndexWriter::new();
        index_writer.write(doc!("dog dog dog monkey bird"));
        index_writer.write(doc!("dog cat cat fox"));
        index_writer.write(doc!("dog raccoon fox"));
        let index = index_writer.build();

        let term = "dog".to_string();
        assert_eq!(index.idf(&term), 0f32);
        assert_eq!(index.tf(0, &term), 0.6);
        assert_eq!(index.tf(1, &term), 0.25);
        assert_eq!(index.tf(2, &term), 1f32 / 3f32);

        let term = "bird".to_string();
        assert_eq!(index.idf(&term), 0.5849625007f32);
        assert_eq!(index.tf(0, &term), 0.2);
        assert_eq!(index.tf(1, &term), 0f32);
        assert_eq!(index.tf(2, &term), 0f32);

        let term = "fox".to_string();
        assert_eq!(index.idf(&term), 0f32);
        assert_eq!(index.tf(0, &term), 0f32);
        assert_eq!(index.tf(1, &term), 0.25);
        assert_eq!(index.tf(2, &term), 1f32 / 3f32);
    }

    #[test]
    fn indexing_test() {
        let mut index_writer = IndexWriter::new();
        index_writer.write(doc!("What is this"));
        index_writer.write(doc!("I am Taisuke"));
        index_writer.write(doc!(
            "that that is is that that is not is not is that it it is"
        ));

        let postings = map! {
            "I" => posting! { 1 => vec![0] },
            "am" => posting! { 1 => vec![1] },
            "Taisuke" => posting! { 1 => vec![2] },
            "this" => posting! { 0 => vec![2] },
            "that" => posting! { 2 => vec![0, 1, 4, 5, 11] },
            "is" => posting! { 0 => vec![1], 2 => vec![2, 3, 6, 8, 10, 14] },
            "not" => posting! { 2 => vec![7, 9] },
            "it" => posting! { 2 => vec![12, 13] },
            "What" => posting! { 0 => vec![0] },
        };

        let stored = map! {
            0usize => doc!("What is this"),
            1usize => doc!("I am Taisuke"),
            2usize => doc!("that that is is that that is not is not is that it it is"),
        };

        let index = index_writer.build();
        assert_eq!(index.postings, postings);
        assert_eq!(index.stored, stored);

        assert_eq!(index.doc(100), None);
        assert_eq!(index.doc(0), Some(&doc!("What is this")));
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

        let sentence = "すもも も もも も もも の うち";
        assert_eq!(
            tokenize(sentence),
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

    #[test]
    fn tfidf_term_search_test() {
        let mut index_writer = IndexWriter::new();
        index_writer.write(doc!("dog dog dog monkey bird"));
        index_writer.write(doc!("dog cat cat fox"));
        index_writer.write(doc!("dog raccoon fox"));
        let index = index_writer.build();

        let term = "dog".to_string();
        assert_eq!(search_term(&index, &term), vec![0, 1, 2]);

        let term = "fox".to_string();
        assert_eq!(search_term(&index, &term), vec![1, 2]);
    }

    #[test]
    fn search_term_test() {
        let mut iw = IndexWriter::new();
        iw.write(doc!("I am Taisuke"));
        iw.write(doc!(
            "that that is is that that is not is not is that it it is"
        ));
        let index = iw.build();

        let term = "Taisuke".to_string();
        assert_eq!(search_term(&index, &term), vec![0]);

        let term = "that".to_string();
        assert_eq!(search_term(&index, &term), vec![1]);

        let term = "foo".to_string();
        assert_eq!(search_term(&index, &term), vec![]);
    }

    #[test]
    fn search_main_test() {
        let sentences = vec![
            doc!("I am Taisuke"),
            doc!("that that is is that that is not is not is that it it is"),
        ];
        let term = "Taisuke".to_string();
        assert_eq!(
            search_main(sentences.clone(), &term),
            vec![doc!("I am Taisuke"),]
        );

        let term = "that".to_string();
        assert_eq!(
            search_main(sentences.clone(), &term),
            vec![doc!(
                "that that is is that that is not is not is that it it is"
            ),]
        );

        let term = "foo".to_string();
        assert_eq!(search_main(sentences.clone(), &term), vec![]);
    }
}
