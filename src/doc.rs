use std::cmp::Ordering;

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

#[derive(Debug, PartialEq)]
pub struct DocAndScore {
    pub score: f32,
    pub doc_id: usize,
}

impl DocAndScore {
    pub fn new_with_score(doc_id: usize, score: f32) -> Self {
        Self { doc_id, score }
    }

    fn score_cmp(&self, other: &Self) -> Ordering {
        let sub = self.score - other.score;
        if sub.abs() <= f32::EPSILON {
            return Ordering::Equal;
        }
        // self > other
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
        Some(self.cmp(other))
    }
}

impl Ord for DocAndScore {
    fn cmp(&self, other: &Self) -> Ordering {
        self.score_cmp(other)
            .reverse()
            .then_with(|| self.doc_id.cmp(&other.doc_id))
    }
}

#[cfg(test)]
mod tests {
    use crate::DocAndScore;
    use std::cmp::Ordering;

    #[test]
    fn doc_and_score_compare_test() {
        let doc1 = DocAndScore::new_with_score(1, 1.0);
        let doc2 = DocAndScore::new_with_score(2, 1.0);
        let doc3 = DocAndScore::new_with_score(3, 0.0);
        let doc4 = DocAndScore::new_with_score(4, 1.0);

        assert_eq!(doc1.cmp(&doc2), Ordering::Less);
        assert_eq!(doc2.cmp(&doc1), Ordering::Greater);
        assert_eq!(doc1.cmp(&doc1), Ordering::Equal);
        assert_eq!(doc1.cmp(&doc3), Ordering::Less);
        assert_eq!(doc3.cmp(&doc1), Ordering::Greater);
        assert_eq!(doc3.cmp(&doc4), Ordering::Greater);
        assert_eq!(doc4.cmp(&doc3), Ordering::Less);

        assert_eq!(doc1.partial_cmp(&doc2), Some(Ordering::Less));
        assert_eq!(doc2.partial_cmp(&doc1), Some(Ordering::Greater));
        assert_eq!(doc1.partial_cmp(&doc1), Some(Ordering::Equal));
        assert_eq!(doc1.partial_cmp(&doc3), Some(Ordering::Less));
        assert_eq!(doc3.partial_cmp(&doc1), Some(Ordering::Greater));
        assert_eq!(doc3.partial_cmp(&doc4), Some(Ordering::Greater));
        assert_eq!(doc4.partial_cmp(&doc3), Some(Ordering::Less));
    }

    #[test]
    fn doc_and_score_sort_test() {
        let mut docs = vec![
            DocAndScore::new_with_score(1, 0.0),
            DocAndScore::new_with_score(0, 0.0),
            DocAndScore::new_with_score(2, 0.0),
        ];
        docs.sort();
        assert_eq!(
            docs.iter().map(|d| d.doc_id).collect::<Vec<_>>(),
            vec![0, 1, 2]
        );

        let mut docs = vec![
            DocAndScore::new_with_score(1, 0.0),
            DocAndScore::new_with_score(0, 0.5),
            DocAndScore::new_with_score(2, 0.5),
        ];
        docs.sort();
        assert_eq!(
            docs.iter().map(|d| d.doc_id).collect::<Vec<_>>(),
            vec![0, 2, 1]
        );
    }
}
