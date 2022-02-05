pub fn search_main(sentence: &String, term: &String) -> Vec<usize> {
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

#[cfg(test)]
mod tests {
    use crate::search_main;

    #[test]
    fn search_main_test() {
        let sentence = "I am Taisuke".to_string();
        let term = "Taisuke".to_string();
        assert_eq!(search_main(&sentence, &term), vec![5]);

        let sentence = "that that is is that that is not is not is that it it is".to_string();
        let term = "that".to_string();
        assert_eq!(search_main(&sentence, &term), vec![0, 5, 16, 21, 43]);

        let sentence = "I am Taisuke".to_string();
        let term = "foo".to_string();
        assert_eq!(search_main(&sentence, &term), vec![]);
    }
}
