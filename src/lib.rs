
pub fn search_term(sentence: &String, term: &String) -> Vec<usize> {
    let sentence = sentence.as_str();

    let mut positions = Vec::new();
    let mut base = 0;
    loop {
        match &sentence[base..].find(term) {
            None => break,
            Some(relative_pos) => {
                let abs_pos = relative_pos + base;
                positions.push(abs_pos);
                base = abs_pos + term.len();
            }
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
    use crate::{search_main, search_term};

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
        let term ="Taisuke".to_string();
        assert_eq!(search_main(&sentences, &term), vec![vec![5], vec![]]);

        let term ="that".to_string();
        assert_eq!(search_main(&sentences, &term), vec![vec![], vec![0, 5, 16, 21, 43]]);

        let term ="foo".to_string();
        assert_eq!(search_main(&sentences, &term), vec![vec![], vec![]]);
    }
}
