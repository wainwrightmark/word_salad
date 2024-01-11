use std::{collections::BTreeSet, str::FromStr};

use crate::prelude::*;
use itertools::Itertools;
use prime_bag::PrimeBag128;

pub type LetterCounts = PrimeBag128<Character>;

pub fn make_words_vec_from_file(text: &str) -> Vec<FinderWord> {
    text.lines()
        //.flat_map(|x| x.split(','))
        .flat_map(|s| match FinderWord::from_str(s) {
            Ok(word) => Some(word),
            Err(err) => {
                log::warn!("Word '{s}' is invalid: {err}");
                None
            }
        })
        .sorted_by_key(|x| x.counts)
        .dedup()
        .collect_vec()
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FinderWord {
    pub text: String,
    pub array: CharsArray,
    pub counts: PrimeBag128<Character>,
}

impl FinderWord {
    pub fn is_strict_substring(&self, super_string: &Self) -> bool {
        if self.array.len() >= super_string.array.len() {
            return false;
        }
        if !self.counts.is_subset(&super_string.counts) {
            return false;
        }

        for start in 0..=(super_string.array.len() - self.array.len()) {
            if super_string.array.as_slice()[start..].starts_with(self.array.as_slice()) {
                return true;
            }
        }
        return false;
    }
}

impl std::fmt::Display for FinderWord {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.text)
    }
}

impl FromStr for FinderWord {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let word = Word::from_str(s)?;

        let counts: PrimeBag128<Character> =
            PrimeBag128::try_from_iter(word.characters.iter().cloned())
                .ok_or("Could not create prime bag")?;
        Ok(Self {
            array: word.characters,
            counts,
            text: word.text,
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
    use std::str::FromStr;

    use test_case::test_case;

    use super::FinderWord;

    #[test_case("abcd", "bcde", false)]
    #[test_case("abcd", "abcde", true)]
    #[test_case("bcde", "abcde", true)]
    #[test_case("abcd", "abcd", false)]
    fn test_substring(sub: &str, super_string: &str, expected: bool) {
        let sub = FinderWord::from_str(sub).unwrap();
        let ss = FinderWord::from_str(super_string).unwrap();

        let actual = sub.is_strict_substring(&ss);

        assert_eq!(actual, expected)
    }
}
