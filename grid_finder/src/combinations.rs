use crate::word_set::WordSet;

use indicatif::{ProgressBar, ProgressStyle};
use itertools::Itertools;
use rayon::prelude::*;
use ws_core::finder::helpers::*;

pub fn get_combinations<const W: usize>(
    category: Option<String>,
    possible_words: &[LetterCounts],
    max_size: u8,
) -> Vec<WordSet<W>> {
    let pb = category.map(|category| {
        ProgressBar::new(possible_words.len() as u64)
            .with_style(
                ProgressStyle::with_template("{prefix} {msg} {pos:3}/{len:3} {elapsed} {wide_bar}")
                    .unwrap(),
            )
            .with_prefix(category.clone())
            .with_message("Getting word combinations")
    });

    let upper_bounds = 0..(possible_words.len());
    let result: Vec<WordSet<W>> = upper_bounds
        .into_iter()
        .par_bridge()
        .map(|upper| {
            let words = &possible_words[0..upper];
            let first_word = possible_words[upper];

            let mut found_combinations: Vec<WordSet<W>> = vec![];

            if let Some(first_combination) =
                WordCombination::<W>::default().try_add_word(&first_word, upper)
            {
                get_combinations_inner(
                    &mut found_combinations,
                    first_combination,
                    words,
                    possible_words,
                    max_size,
                );
            }

            pb.iter().for_each(|x| x.inc(1));
            //println!("Found {} for upper {upper}", found_combinations.len());
            found_combinations
        })
        .reduce(std::vec::Vec::new, |a, b| {
            let (mut big, small) = if a.len() >= b.len() { (a, b) } else { (b, a) };

            big.extend_from_slice(&small);
            big
        });

    //println!("Found a total of {} results", result.len());

    pb.iter().for_each(|x| x.finish());
    result
}

#[derive(Debug, Default, PartialEq)]
enum GCIResult {
    #[default]
    NothingFound,
    PossibleCombinationFound,
}

fn get_combinations_inner<const W: usize>(
    found_combinations: &mut Vec<WordSet<W>>,
    current_combination: WordCombination<W>,
    mut possible_words: &[LetterCounts],
    all_possible_words: &[LetterCounts],
    max_size: u8,
) -> GCIResult {
    let mut result = GCIResult::NothingFound;
    loop {
        let Some((word, npw)) = possible_words.split_last() else {
            break;
        };
        possible_words = npw;

        let Some(new_combination) = current_combination.try_add_word(word, possible_words.len())
        else {
            panic!("Could not add word to multiplicities");
        };

        if new_combination.total_letters <= max_size {
            let sub_result = get_combinations_inner(
                found_combinations,
                new_combination.clone(),
                possible_words,
                all_possible_words,
                max_size,
            );

            if sub_result == GCIResult::PossibleCombinationFound {
                continue;
            }
            result = GCIResult::PossibleCombinationFound;

            if all_possible_words
                .iter()
                .enumerate()
                .any(|(word_index, letter_indices)| {
                    !new_combination.word_indexes.get_bit(word_index)
                        && letter_indices.is_subset(&new_combination.letter_counts)
                })
            {
                continue;
            }

            if all_possible_words
                .iter()
                .enumerate()
                .rev()
                .any(|(word_index, letter_indices)| {
                    new_combination.can_add(word_index, letter_indices, max_size)
                })
            {
                // if new_combination.display_string(words) == "Claret, Coral, Cream, Cyan, Gray, Green, Ivory, Lime, Olive, Orange, Teal"{
                //     panic!("Hello world")
                // }
                continue;
            }

            found_combinations.push(new_combination.word_indexes);
        }
    }
    result
}

#[derive(Debug, Clone, PartialEq, Default, Ord, PartialOrd, Eq)]
pub struct WordCombination<const W: usize> {
    pub word_indexes: WordSet<W>,
    pub letter_counts: LetterCounts,
    pub total_letters: u8,
}

/// Iterate all subsets of this set whose element count is exactly one less
pub fn shrink_bit_sets<const W: usize>(set: &WordSet<W>) -> impl Iterator<Item = WordSet<W>> + '_ {
    set.into_iter().map(|index| {
        let mut s = *set;
        s.set_bit(index, false);
        s
    })
}

impl<const W: usize> WordCombination<W> {
    pub fn from_bit_set(word_indexes: WordSet<W>, words: &[LetterCounts]) -> Option<Self> {
        let mut letter_counts = LetterCounts::default();
        for word in word_indexes.into_iter().map(|i| words[i]) {
            letter_counts = letter_counts.try_union(&word)?
        }

        Some(Self {
            word_indexes,
            letter_counts,
            total_letters: letter_counts.into_iter().count() as u8,
        })
    }
    pub fn get_single_words(&self, words: &[FinderGroup]) -> Vec<FinderSingleWord> {
        self.word_indexes
            .into_iter()
            .flat_map(|i| words[i].words.iter())
            .cloned()
            .collect_vec()
    }

    pub fn display_string(&self, words: &[FinderGroup]) -> String {
        self.get_single_words(words)
            .iter()
            .map(|x| x.text.as_str())
            .sorted()
            .join(", ")
    }

    #[must_use]
    fn try_add_word(&self, word: &LetterCounts, word_index: usize) -> Option<Self> {
        let letter_counts = self.letter_counts.try_union(word)?;
        let mut word_indexes = self.word_indexes;
        word_indexes.set_bit(word_index, true);

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

    fn can_add(
        &self,
        other_word_index: usize,
        other_word_letters: &LetterCounts,
        max_size: u8,
    ) -> bool {
        if self.word_indexes.get_bit(other_word_index) {
            return false;
        }
        let Some(letter_counts) = self.letter_counts.try_union(other_word_letters) else {
            return false;
        };
        if letter_counts == self.letter_counts {
            true
        } else {
            if self.total_letters == max_size {
                return false;
            }

            let diff = letter_counts
                .try_difference(&self.letter_counts)
                .unwrap_or_default();
            let new_elements = diff.into_iter().count() as u8;

            let new_count = self.total_letters + new_elements;

            new_count <= max_size
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
        let input = "hydrogen\nhelium\nlithium\nboron\ncarbon";

        let _now = Instant::now();

        let words = make_finder_group_vec_from_file(input);
        let word_letters: Vec<LetterCounts> = words.iter().map(|x| x.counts).collect_vec();

        let possible_combinations: Vec<WordSet<1>> =
            get_combinations(None, word_letters.as_slice(), 16);

        let expected = "boron, carbon, helium, hydrogen\nboron, carbon, helium, lithium\nboron, helium, hydrogen, lithium";

        let actual = possible_combinations
            .iter()
            .map(|x| WordCombination::from_bit_set(*x, &word_letters).unwrap())
            .map(|x| x.display_string(words.as_slice()))
            .sorted()
            .join("\n");

        assert_eq!(expected, actual)
    }

    #[test_case("monkey\ncow\nant\nantelope", "monkey\ncow\nant\nantelope")]
    #[test_case(
        "Claret\nCoral\nCream\nCyan\nGray\nGreen\nIvory\nLime\nOlive\nOrange\nTeal",
        "Claret\nCoral\nCream\nCyan\nGray\nGreen\nIvory\nLime\nOlive\nOrange\nTeal"
    )]
    #[test_case(
        "POLITICIAN\nOPTICIAN\nCASHIER\nFLORIST\nARTIST\nTAILOR\nACTOR",
        "POLITICIAN\nOPTICIAN\nCASHIER\nFLORIST\nARTIST\nTAILOR\nACTOR"
    )]
    #[test_case(
        "SILVER\nORANGE\nGREEN\nIVORY\nCORAL\nOLIVE\nTEAL\nGRAY\nCYAN\nRED",
        "SILVER\nORANGE\nGREEN\nIVORY\nCORAL\nOLIVE\nTEAL\nGRAY\nCYAN\nRED"
    )]
    #[test_case(
        "Teal\nWheat\nWhite\nGreen\nCyan\nGray\nCoral\nOrange\nMagenta",
        "Teal\nWheat\nWhite\nGreen\nCyan\nGray\nCoral\nOrange\nMagenta"
    )]
    #[test_case(
        "Claret\nCoral\nCream\nCyan\nGray\nGreen\nIvory\nLime\nOlive\nOrange\nTeal\nWhite\nYellow",
        //"Claret\nCoral\nCream\nCyan\nGray\nGreen\nIvory\nLime\nOlive\nOrange\nTeal\nWhite\nYellow",
        "Claret\nCoral\nCream\nCyan\nGray\nGreen\nIvory\nLime\nOlive\nOrange\nTeal\nWhite"
    )]

    pub fn test_membership(input: &'static str, expected_member: &'static str) {
        let now = Instant::now();

        let expected_words = make_finder_group_vec_from_file(expected_member);
        let mut expected = LetterCounts::default();
        for fw in expected_words {
            expected = expected
                .try_union(&fw.counts)
                .expect("Should be able to union expected");
        }

        let words = make_finder_group_vec_from_file(input);

        let word_letters: Vec<LetterCounts> = words.iter().map(|x| x.counts).collect_vec();

        let possible_combinations: Vec<WordSet<1>> =
            get_combinations(None, word_letters.as_slice(), 16);

        println!("{:?}", now.elapsed());

        let contains_expected = possible_combinations
            .iter()
            .map(|x| WordCombination::from_bit_set(*x, &word_letters).unwrap())
            .map(|x| x.letter_counts)
            .contains(&expected);

        if !contains_expected {
            let actual = possible_combinations
                .into_iter()
                .map(|x| WordCombination::from_bit_set(x, &word_letters).unwrap())
                .map(|x| x.display_string(words.as_slice()))
                .sorted()
                .join("\n");

            println!("actual {actual}");
        }

        assert!(contains_expected);
    }
}
