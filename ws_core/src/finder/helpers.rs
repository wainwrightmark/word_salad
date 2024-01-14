use std::{collections::BTreeSet, str::FromStr};

use crate::prelude::*;
use itertools::Itertools;
use prime_bag::PrimeBag128;

pub type LetterCounts = PrimeBag128<Character>;

pub fn make_finder_group_vec_from_file(text: &str) -> Vec<FinderGroup> {
    text.lines()
        //.flat_map(|x| x.split(','))
        .flat_map(|s| match FinderGroup::from_str(s) {
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
pub struct FinderGroup {
    pub text: String,
    pub words: Vec<FinderSingleWord>,
    pub counts: PrimeBag128<Character>,
}

impl FromStr for FinderGroup {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let words: Vec<FinderSingleWord> = s
            .split("+")
            .map(|s| FinderSingleWord::from_str(s))
            .try_collect()?;

        let counts = words
            .iter()
            .map(|x| x.counts)
            .fold(PrimeBag128::default(), |a, b| {
                a.try_union(&b).expect("Could not combine")
            });
        Ok(FinderGroup { text:s.to_string(), words, counts })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FinderSingleWord {
    pub text: String,
    pub array: CharsArray,
    pub counts: PrimeBag128<Character>,
}

impl FinderSingleWord {
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

impl std::fmt::Display for FinderSingleWord {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.text)
    }
}

impl FromStr for FinderSingleWord {
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
pub fn count_adjacent_indexes(word: &FinderSingleWord, char: Character) -> usize {
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

    use super::FinderSingleWord;

    #[test_case("abcd", "bcde", false)]
    #[test_case("abcd", "abcde", true)]
    #[test_case("bcde", "abcde", true)]
    #[test_case("abcd", "abcd", false)]
    fn test_substring(sub: &str, super_string: &str, expected: bool) {
        let sub = FinderSingleWord::from_str(sub).unwrap();
        let ss = FinderSingleWord::from_str(super_string).unwrap();

        let actual = sub.is_strict_substring(&ss);

        assert_eq!(actual, expected)
    }
}
