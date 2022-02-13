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
