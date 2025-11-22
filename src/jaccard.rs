use std::collections::HashSet;

use foldhash::fast::RandomState;

pub enum Strategy {
    Word,
    QGrams(usize),
}

fn jaccard_index_word(str_1: &str, str_2: &str) -> f32 {
    let set_1: HashSet<&str, RandomState> = HashSet::from_iter(str_1.split_whitespace());
    let set_2: HashSet<&str, RandomState> = HashSet::from_iter(str_2.split_whitespace());

    set_1.intersection(&set_2).count() as f32 / set_1.union(&set_2).count() as f32
}
fn jaccard_index_q_gram(str_1: &str, str_2: &str, q_grams: usize) -> f32 {
    todo!()
}

pub fn jaccard_index(str_1: &str, str_2: &str, strategy: Strategy) -> f32 {
    match strategy {
        Strategy::Word => jaccard_index_word(str_1, str_2),
        Strategy::QGrams(q_grams) => jaccard_index_q_gram(str_1, str_2, q_grams),
    }
}

#[cfg(test)]
mod tests {
    use crate::jaccard::{Strategy, jaccard_index, jaccard_index_word};

    #[test]
    fn test_jaccard_index_word_valid() {
        assert_eq!(jaccard_index("test", "test", Strategy::Word), 1.);
    }
    #[test]
    fn test_jaccard_index_word_full_match() {
        assert_eq!(jaccard_index_word("test", "test"), 1.);
    }
    #[test]
    fn test_jaccard_index_word_half_match() {
        assert_eq!(jaccard_index_word("test", "test test2"), 0.5);
    }
    #[test]
    fn test_jaccard_index_word_no_match() {
        assert_eq!(jaccard_index_word("test", "no_match"), 0.);
    }
}
