#[cfg(test)]
mod tests {
    use crate::{Dictionary, MutableDictionary};

    #[test]
    fn curated_matches_capitalized_es() {
        let dict = MutableDictionary::curated("es");
        assert!(dict.contains_word_str("este"));
        assert!(dict.contains_word_str("Este"));
    }
}