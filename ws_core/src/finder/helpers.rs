use std::collections::{BTreeSet, HashMap};

use crate::prelude::*;
use itertools::Itertools;
use prime_bag::PrimeBag128;

pub type LetterCounts = PrimeBag128<Character>;
pub type WordMultiMap = HashMap<LetterCounts, Vec<FinderWord>>;

pub fn make_words_from_file(text: &str) -> WordMultiMap {
    text.lines()
        .flat_map(|x| x.split(','))
        .flat_map(FinderWord::try_new)
        .into_group_map_by(|x| x.counts)
}

#[derive(Debug, Clone, PartialEq)]
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
    fn try_new(text: &str) -> Option<Self> {
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
pub fn count_adjacent_indexes(word: &CharsArray, char: Character) -> usize {
    let mut indexes: BTreeSet<usize> = Default::default();

    for (index, word_char) in word.iter().enumerate() {
        if *word_char == char {
            if let Some(checked_index) = index.checked_sub(1) {
                indexes.insert(checked_index);
            }
            if word.get(index + 1).is_some() {
                indexes.insert(index + 1);
            }
        }
    }

    indexes.len()
}

#[cfg(test)]
mod tests {



}