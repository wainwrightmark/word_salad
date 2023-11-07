use bit_set::BitSet;
use indicatif::{ProgressBar, ProgressStyle};
use itertools::Itertools;
use rayon::prelude::*;
use ws_core::finder::helpers::*;

pub fn get_combinations(possible_words: &[LetterCounts], max_size: u8) -> Vec<WordCombination> {
    let pb = ProgressBar::new(possible_words.len() as u64)
        .with_style(ProgressStyle::with_template("{msg} {wide_bar} {pos}/{len}").unwrap())
        .with_message("Getting word combinations");

    let upper_bounds = 1..(possible_words.len());
    let result: Vec<WordCombination> = upper_bounds
        .into_iter()
        //.map(|upper| &possible_words[0..=upper])
        .par_bridge()
        .map(|upper| {
            let words = &possible_words[0..=upper];
            let mut found_combinations: Vec<WordCombination> = vec![];
            get_combinations_inner(
                &mut found_combinations,
                WordCombination::default(),
                words,
                possible_words,
                max_size,
            );
            pb.inc(1);
            found_combinations
        })
        .reduce(
            || vec![],
            |a, b| {
                let (mut big, small) = if a.len() >= b.len() { (a, b) } else { (b, a) };

                big.extend_from_slice(&small);
                big
            },
        );

    pb.finish();
    result
}

fn get_combinations_inner(
    found_combinations: &mut Vec<WordCombination>,
    current_combination: WordCombination,
    mut possible_words: &[LetterCounts],
    all_possible_words: &[LetterCounts],
    max_size: u8,
) {
    loop {
        let Some((word, npw)) = possible_words.split_last() else {
            break;
        };
        possible_words = npw;

        let Some(new_combination) = current_combination.try_add_word(&word, possible_words.len())
        else {
            panic!("Could not add word to multiplicities");
        };

        if new_combination.total_letters <= max_size {
            let current_total = found_combinations.len();

            get_combinations_inner(
                found_combinations,
                new_combination.clone(),
                possible_words,
                all_possible_words,
                max_size,
            );

            let new_total = found_combinations.len();

            if new_total == current_total {
                //no children were added so this might be a maximal set

                let subset = found_combinations
                    .iter()
                    .any(|fc| fc.word_indexes.is_superset(&new_combination.word_indexes));

                if !subset {
                    let any_word_fits = all_possible_words
                        .iter()
                        .skip(possible_words.len() + new_combination.total_letters as usize)
                        .rev()
                        .any(|w| w.is_subset(&new_combination.letter_counts));

                    if !any_word_fits {
                        if new_combination.total_letters == max_size
                            || !all_possible_words
                                .iter()
                                .enumerate()
                                .skip(possible_words.len())
                                .rev()
                                .any(|(index, other_word)| {
                                    new_combination.can_add(other_word, index, max_size)
                                })
                        {
                            found_combinations.push(new_combination);
                        }
                    }
                }
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default, Ord, PartialOrd, Eq)]
pub struct WordCombination {
    pub word_indexes: bit_set::BitSet,
    pub letter_counts: LetterCounts,
    pub total_letters: u8,
}

pub fn shrink_bit_sets<'a>(set: &'a bit_set::BitSet) -> impl Iterator<Item = bit_set::BitSet> + 'a {
    set.iter().map(|bit| {
        let mut s = set.clone();
        s.remove(bit);
        s
    })
}

impl WordCombination {
    pub fn from_bit_set(word_indexes: BitSet, words: &[LetterCounts]) -> Option<Self> {
        let mut letter_counts = LetterCounts::default();
        for word in word_indexes.iter().map(|i| words[i]) {
            letter_counts = letter_counts.try_union(&word)?
        }

        Some(Self {
            word_indexes,
            letter_counts,
            total_letters: letter_counts.into_iter().count() as u8,
        })
    }
    pub fn get_words(&self, words: &[FinderWord]) -> Vec<FinderWord> {
        self.word_indexes
            .iter()
            .map(|index| words[index].clone())
            .collect_vec()
    }

    pub fn display_string(&self, words: &[FinderWord]) -> String {
        self.get_words(words)
            .iter()
            .map(|x| x.text.as_str())
            .join(", ")
    }

    #[must_use]
    fn try_add_word(&self, word: &LetterCounts, word_index: usize) -> Option<Self> {
        let letter_counts = self.letter_counts.try_union(&word)?;
        let mut word_indexes = self.word_indexes.clone();
        word_indexes.insert(word_index);

        if letter_counts == self.letter_counts {
            Some(Self {
                word_indexes,
                letter_counts,
                total_letters: self.total_letters,
            })
        } else {
            let diff = letter_counts.try_difference(&self.letter_counts)?;
            let new_elements = diff.into_iter().count() as u8;
            Some(Self {
                letter_counts,
                word_indexes,
                total_letters: self.total_letters + new_elements,
            })
        }
    }

    fn can_add(&self, other_word: &LetterCounts, word_index: usize, max_size: u8) -> bool {
        if self.word_indexes.contains(word_index) {
            return false;
        }
        let Some(letter_counts) = self.letter_counts.try_union(&other_word) else {
            return false;
        };
        if letter_counts == self.letter_counts {
            return true;
        } else {
            if self.total_letters == max_size {
                return false;
            }

            let diff = letter_counts
                .try_difference(&self.letter_counts)
                .unwrap_or_default();
            let new_elements = diff.into_iter().count() as u8;

            let new_count = self.total_letters + new_elements;

            return new_count <= max_size;
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use itertools::Itertools;
    use std::time::Instant;
    use test_case::test_case;

    #[test]
    pub fn test_possible_combinations() {
        let input = "monkey\ncow\nant\nantelope";

        let now = Instant::now();

        let words = make_words_vec_from_file(input);
        let word_letters: Vec<LetterCounts> = words.iter().map(|x| x.counts).collect_vec();

        let possible_combinations: Vec<WordCombination> =
            get_combinations(word_letters.as_slice(), 16);

        println!("{:?}", now.elapsed());

        let expected = "ant, cow, antelope, monkey";

        let actual = possible_combinations
            .into_iter()
            .map(|x| x.display_string(words.as_slice()))
            .join("\n");

        assert_eq!(expected, actual)
    }

    #[test_case("monkey\ncow\nant\nantelope", "monkey\ncow\nant\nantelope")]
    #[test_case(
        "POLITICIAN, OPTICIAN, CASHIER, FLORIST, ARTIST, TAILOR, ACTOR",
        "POLITICIAN, OPTICIAN, CASHIER, FLORIST, ARTIST, TAILOR, ACTOR"
    )]
    #[test_case(
        "SILVER, ORANGE, GREEN, IVORY, CORAL, OLIVE, TEAL, GRAY, CYAN, RED",
        "SILVER, ORANGE, GREEN, IVORY, CORAL, OLIVE, TEAL, GRAY, CYAN, RED"
    )]
    pub fn test_membership(input: &'static str, expected_member: &'static str) {
        let now = Instant::now();

        let expected_words = make_words_vec_from_file(expected_member);
        let mut expected = LetterCounts::default();
        for fw in expected_words {
            expected = expected
                .try_union(&fw.counts)
                .expect("Should be able to union expected");
        }

        let words = make_words_vec_from_file(input);

        let word_letters: Vec<LetterCounts> = words.iter().map(|x| x.counts).collect_vec();

        let possible_combinations: Vec<WordCombination> =
            get_combinations(word_letters.as_slice(), 16);

        println!("{:?}", now.elapsed());

        let contains_expected = possible_combinations
            .iter()
            .map(|x| x.letter_counts)
            .contains(&expected);

        if !contains_expected {
            let actual = possible_combinations
                .into_iter()
                .map(|x| x.display_string(words.as_slice()))
                .join("\n");

            println!("{actual}");
        }

        assert!(contains_expected);
    }
}
