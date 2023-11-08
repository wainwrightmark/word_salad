use std::collections::BTreeSet;

use crate::prelude::*;
use itertools::Itertools;
use prime_bag::PrimeBag128;

pub type LetterCounts = PrimeBag128<Character>;

pub fn make_words_vec_from_file(text: &str) -> Vec<FinderWord> {
    text.lines()
        .flat_map(|x| x.split(','))
        .flat_map(FinderWord::try_new)
        .sorted_by_key(|x|x.counts)
        .collect_vec()
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct FinderWord {
    pub text: String,
    pub array: CharsArray,
    pub counts: PrimeBag128<Character>,
}

impl std::fmt::Display for FinderWord {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.text)
    }
}

impl FinderWord {
    pub fn try_new(text: &str) -> Option<Self> {
        //println!("'{text}'");
        let array = Word::from_str(text).ok().map(|x| x.characters)?;

        let counts: PrimeBag128<Character> = PrimeBag128::try_from_iter(array.iter().cloned())?;
        Some(Self {
            array,
            counts,
            text: text.to_string(),
        })
    }
}

/// Counts the number of distinct index of of letters adjacent to a letter which is this character
pub fn count_adjacent_indexes(word: &FinderWord, char: Character) -> usize {
    let mut indexes: BTreeSet<usize> = Default::default();

    for (index, word_char) in word.array.iter().enumerate() {
        if *word_char == char {
            if let Some(checked_index) = index.checked_sub(1) {
                indexes.insert(checked_index);
            }
            if word.array.get(index + 1).is_some() {
                indexes.insert(index + 1);
            }
        }
    }

    indexes.len()
}

#[cfg(test)]
mod tests {



}
