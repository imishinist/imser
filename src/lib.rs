mod doc;
mod token;

pub use doc::Document;
pub use token::TokenizeType;

use doc::*;
use std::collections::HashMap;
use std::iter::Peekable;
use std::slice::Iter;
use token::*;

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

#[derive(Debug, Default)]
struct IndexWriterConfig {
    pub tokenize_type: TokenizeType,
}

#[derive(Debug)]
struct IndexWriter {
    seq: usize,

    term_dict: TermDict,

    // (doc_id, dict_index, positions)
    term_positions: Vec<(usize, usize, Vec<usize>)>,

    // (doc_id, Document)
    stored: Vec<(usize, Document)>,

    tokenize_type: TokenizeType,
}

impl IndexWriter {
    #[allow(dead_code)]
    pub fn new() -> Self {
        IndexWriter::with_config(IndexWriterConfig {
            ..Default::default()
        })
    }

    pub fn with_config(config: IndexWriterConfig) -> Self {
        Self {
            seq: 0,
            term_dict: TermDict::new(),
            term_positions: Vec::new(),
            stored: Vec::new(),
            tokenize_type: config.tokenize_type,
        }
    }

    fn seq_incr(&mut self) -> usize {
        let curr = self.seq;
        self.seq += 1;
        curr
    }

    fn write(&mut self, doc: Document) {
        let id = self.seq_incr();
        let tokens = tokenize(self.tokenize_type, doc.body.as_str());

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

struct MultiTermQuery {
    terms: Vec<Term>,
}

impl MultiTermQuery {
    fn new(terms: Vec<Term>) -> Self {
        Self { terms }
    }

    fn iter<'a>(&self, index: &'a PositionalIndex) -> DocIterator<'a> {
        DocIterator::new(self, index)
    }
}

struct DocIterator<'a> {
    posting_lists: Vec<Peekable<Iter<'a, PostingData>>>,

    next_doc: Option<usize>,
}

impl<'a> DocIterator<'a> {
    fn new(query: &MultiTermQuery, index: &'a PositionalIndex) -> Self {
        let mut posting_lists = Vec::with_capacity(query.terms.len());
        let mut next_doc = None;

        let mut have_none = false;
        for term in query.terms.iter() {
            match index.postings.get(term) {
                None => have_none = true,
                Some(pl) => {
                    let mut postings = pl.postings.iter().peekable();
                    if have_none {
                        next_doc = None;
                    } else {
                        next_doc = postings.peek().map(|pd| pd.doc_id);
                    }
                    posting_lists.push(postings);
                }
            }
        }

        Self {
            posting_lists,
            next_doc,
        }
    }
}

impl<'a> Iterator for DocIterator<'a> {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        'outer: loop {
            let target = self.next_doc?;
            for pl in self.posting_lists.iter_mut() {
                // skip until target > posting.doc_id
                while pl.next_if(|posting| target > posting.doc_id).is_some() {}
            }

            for pl in self.posting_lists.iter_mut() {
                let posting = pl.peek()?;
                if posting.doc_id != target {
                    self.next_doc.replace(posting.doc_id);
                    continue 'outer;
                }
            }
            self.next_doc.replace(target + 1);
            return Some(target);
        }
    }
}

#[allow(dead_code)]
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

fn search_multi_term(index: &PositionalIndex, query: MultiTermQuery) -> Vec<usize> {
    query.iter(index).collect()
}

pub fn search_main(
    tokenize_type: TokenizeType,
    docs: Vec<Document>,
    sentence: &str,
) -> Vec<Document> {
    let mut index_writer = IndexWriter::with_config(IndexWriterConfig { tokenize_type });
    for doc in docs {
        index_writer.write(doc);
    }
    let index = index_writer.build();

    let terms = tokenize(tokenize_type, sentence)
        .iter()
        .filter_map(|t| match t.kind {
            TokenKind::Term(term) => Some(term.to_string()),
            _ => None,
        })
        .collect::<Vec<_>>();
    let query = MultiTermQuery::new(terms);

    search_multi_term(&index, query)
        .iter()
        .map(|id| index.doc(*id).unwrap().clone())
        .collect()
}

#[cfg(test)]
mod tests {
    use crate::{
        doc, search_main, search_term, IndexWriter, MultiTermQuery, TermDict, TokenizeType,
    };

    #[test]
    fn doc_iter_test() {
        let mut index_writer = IndexWriter::new();
        index_writer.write(doc!("dog dog dog monkey bird"));
        index_writer.write(doc!("dog cat cat fox"));
        index_writer.write(doc!("dog raccoon fox"));
        let index = index_writer.build();

        // don't exist term
        let query = MultiTermQuery::new(vec!["mouse".to_string(), "fox".to_string()]);
        let mut iter = query.iter(&index);
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next(), None);

        let query = MultiTermQuery::new(vec!["dog".to_string()]);
        let mut iter = query.iter(&index);
        assert_eq!(iter.next(), Some(0));
        assert_eq!(iter.next(), Some(1));
        assert_eq!(iter.next(), Some(2));
        assert_eq!(iter.next(), None);

        let query = MultiTermQuery::new(vec!["dog".to_string(), "fox".to_string()]);
        let mut iter = query.iter(&index);
        assert_eq!(iter.next(), Some(1));
        assert_eq!(iter.next(), Some(2));
        assert_eq!(iter.next(), None);

        let query = MultiTermQuery::new(vec!["dog".to_string(), "dog".to_string()]);
        let mut iter = query.iter(&index);
        assert_eq!(iter.next(), Some(0));
        assert_eq!(iter.next(), Some(1));
        assert_eq!(iter.next(), Some(2));
        assert_eq!(iter.next(), None);

        let query = MultiTermQuery::new(vec!["dog".to_string(), "bird".to_string()]);
        let mut iter = query.iter(&index);
        assert_eq!(iter.next(), Some(0));
        assert_eq!(iter.next(), None);
    }

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
        assert_eq!(search_term(&index, &term), Vec::<usize>::new());
    }

    #[test]
    fn search_main_test() {
        let sentences = vec![
            doc!("I am Taisuke"),
            doc!("that that is is that that is not is not is that it it is"),
        ];
        let term = "Taisuke".to_string();
        assert_eq!(
            search_main(TokenizeType::Whitespace, sentences.clone(), &term),
            vec![doc!("I am Taisuke"),]
        );

        let term = "that".to_string();
        assert_eq!(
            search_main(TokenizeType::Whitespace, sentences.clone(), &term),
            vec![doc!(
                "that that is is that that is not is not is that it it is"
            ),]
        );

        let term = "foo".to_string();
        assert_eq!(
            search_main(TokenizeType::Whitespace, sentences.clone(), &term),
            vec![]
        );

        let sentences = vec![
            doc!("すもももももももものうち"),
            doc!("関西国際空港限定トートバッグ"),
            doc!("東京国際空港"),
        ];

        let term = "すもも".to_string();
        assert_eq!(
            search_main(TokenizeType::Japanese, sentences.clone(), &term),
            vec![doc!("すもももももももものうち"),]
        );
    }
}
