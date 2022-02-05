pub fn search_main(sentence: &String, term: &String) -> Option<i64> {
    sentence.find(term).map(|pos| pos as i64)
}

#[cfg(test)]
mod tests {
    use crate::search_main;

    #[test]
    fn search_main_test() {
        let sentence = "I am Taisuke".to_string();
        let term = "Taisuke".to_string();
        assert_eq!(search_main(&sentence, &term), Some(5i64));

        let sentence = "that that is is that that is not is not is that it it is".to_string();
        let term = "that".to_string();
        assert_eq!(search_main(&sentence, &term), Some(0i64));

        let sentence = "I am Taisuke".to_string();
        let term = "foo".to_string();
        assert_eq!(search_main(&sentence, &term), None);
    }
}
