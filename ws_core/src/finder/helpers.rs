use std::{cmp::Reverse, str::FromStr};

use crate::{prelude::*, word_trait::WordTrait};
use const_sized_bit_set::BitSet;
use itertools::Itertools;
use prime_bag::PrimeBag128;
use ustr::ustr;

pub type LetterCounts = PrimeBag128<Character>;

pub fn make_finder_group_vec_from_file(text: &str) -> Vec<FinderGroup> {
    let mut errors: Vec<(&str, &str)> = vec![];
    let result = text
        .lines()
        .filter(|x| !x.is_empty())
        .filter(|x| !x.starts_with("-"))
        //.flat_map(|x| x.split(','))
        .flat_map(|s| match FinderGroup::from_str(s) {
            Ok(word) => Some(word),
            Err(err) => {
                errors.push((err, s));
                //log::warn!("Word '{s}' is invalid: {err}");
                None
            }
        })
        .sorted_by_key(|x| x.counts)
        .dedup()
        .collect_vec();
    errors.sort();
    for (error, error_words) in errors
        .into_iter()
        .group_by(|(err, _)| err.to_string())
        .into_iter()
    {
        let words = error_words.map(|(_, word)| word).join(", ");
        log::warn!("{error}: {words}")
    }

    result
}

pub fn take_bad_words_from_file(text: &str) -> Vec<FinderSingleWord> {
    let mut errors: Vec<(&str, &str)> = vec![];
    let result = text
        .lines()
        .filter(|x| !x.is_empty())
        .filter(|x| x.starts_with("-"))
        //.flat_map(|x| x.split(','))
        .flat_map(|s| match FinderSingleWord::from_str(s) {
            Ok(word) => Some(word),
            Err(err) => {
                errors.push((err, s));
                //log::warn!("Word '{s}' is invalid: {err}");
                None
            }
        })
        .sorted_by_key(|x| x.counts)
        .dedup()
        .collect_vec();

    for (error, error_words) in errors
        .into_iter()
        .group_by(|(err, _)| err.to_string())
        .into_iter()
    {
        let words = error_words.map(|(_, word)| word).join(", ");
        log::warn!("{error}: {words}")
    }

    result
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FinderGroup {
    pub text: Ustr,
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
        Ok(FinderGroup {
            text: ustr(s),
            words,
            counts,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FinderSingleWord {
    pub text: Ustr,
    pub array: CharsArray,
    pub counts: PrimeBag128<Character>,
}

impl WordTrait for FinderSingleWord {
    fn characters(&self) -> &CharsArray {
        &self.array
    }

    fn letter_counts(&self) -> Option<LetterCounts> {
        Some(self.counts)
    }
}

impl PartialOrd for FinderSingleWord {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.array
            .iter()
            .map(|x| x.as_char())
            .partial_cmp(other.array.iter().map(|x| x.as_char()))
    }
}

impl Ord for FinderSingleWord {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.array
            .iter()
            .map(|x| x.as_char())
            .cmp(other.array.iter().map(|x| x.as_char()))
    }
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

impl From<&DisplayWord> for FinderSingleWord {
    fn from(value: &DisplayWord) -> Self {
        let DisplayWord {
            characters, text, ..
        } = value;
        let counts = PrimeBag128::try_from_iter(characters.iter().cloned()).unwrap();

        Self {
            text: text.clone(),
            array: characters.clone(),
            counts,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct AdjacencyStrength {
    appearances: u8,
    adjacencies: u8,
    distinct_adjacent_indices: Reverse<u8>,
}

impl AdjacencyStrength {
    /// Counts the number of distinct index of letters adjacent to a letter which is this character
    pub fn calculate(word: &FinderSingleWord, char: Character) -> Self {
        let mut appearances: u8 = 0;
        let mut adjacencies: u8 = 0;
        let mut adjacent_indices = BitSet::<1>::default();

        for (index, word_char) in word.array.iter().enumerate() {
            if *word_char == char {
                appearances += 1;
                if let Some(checked_index) = index.checked_sub(1) {
                    adjacencies += 1;
                    adjacent_indices.set_bit(checked_index, true);
                }
                if word.array.get(index + 1).is_some() {
                    adjacencies += 1;
                    adjacent_indices.set_bit(index + 1, true);
                }
            }
        }

        Self {
            appearances,
            adjacencies,
            distinct_adjacent_indices: Reverse(adjacent_indices.count() as u8),
        }
    }
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
