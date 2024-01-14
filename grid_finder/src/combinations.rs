use indicatif::{ProgressBar, ProgressStyle};
use itertools::Itertools;
use rayon::prelude::*;
use ws_core::finder::helpers::*;

pub type WordSet = geometrid::tile_set::TileSet128<128, 1, 128>;
pub type WordId = geometrid::tile::Tile<128, 1>;

pub fn get_combinations(possible_words: &[LetterCounts], max_size: u8) -> Vec<WordCombination> {
    if possible_words.len() > 128 {
        panic!("Maximum of 128 words")
    }
    let pb = ProgressBar::new(possible_words.len() as u64)
        .with_style(ProgressStyle::with_template("{msg} {wide_bar} {pos:3}/{len:3}").unwrap())
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

        let Some(new_combination) = current_combination
            .try_add_word(&word, WordId::try_from_usize(possible_words.len()).unwrap())
        else {
            panic!("Could not add word to multiplicities");
        };

        if new_combination.total_letters <= max_size {
            let previous_total = found_combinations.len();

            get_combinations_inner(
                found_combinations,
                new_combination.clone(),
                possible_words,
                all_possible_words,
                max_size,
            );

            let new_total = found_combinations.len();

            if new_total != previous_total {
                continue;
            }

            // if new_combination
            //     .letter_counts
            //     .contains_at_least(ws_core::Character::Blank, BLANK_COUNT)
            // {
            //     continue;
            // }

            // if found_combinations.iter().any(|fc| {
            //     fc.word_indexes.intersect(&new_combination.word_indexes)
            //         == new_combination.word_indexes
            // }) {
            //     continue;
            // }

            // if all_possible_words
            //     .iter()
            //     .skip(possible_words.len() + new_combination.total_letters as usize)
            //     .rev()
            //     .any(|w| w.is_subset(&new_combination.letter_counts))
            // {
            //     continue;
            // }

            // if new_combination.total_letters != max_size
            //     || all_possible_words
            //         .iter()
            //         .enumerate()
            //         .skip(possible_words.len())
            //         .rev()
            //         .any(|(index, other_word)| {
            //             new_combination.can_add(
            //                 other_word,
            //                 WordId::try_from_usize(index).unwrap(),
            //                 max_size,
            //             )
            //         })
            // {
            //     continue;
            // }

            found_combinations.push(new_combination);
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default, Ord, PartialOrd, Eq)]
pub struct WordCombination {
    pub word_indexes: WordSet,
    pub letter_counts: LetterCounts,
    pub total_letters: u8,
}

pub fn shrink_bit_sets<'a>(set: &'a WordSet) -> impl Iterator<Item = WordSet> + 'a {
    set.iter_true_tiles().map(|tile| {
        let mut s = set.clone();
        s.set_bit(&tile, false);
        s
    })
}

impl WordCombination {
    pub fn from_bit_set(word_indexes: WordSet, words: &[LetterCounts]) -> Option<Self> {
        let mut letter_counts = LetterCounts::default();
        for word in word_indexes
            .iter_true_tiles()
            .map(|i| words[i.inner() as usize])
        {
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
            .iter_true_tiles()
            .flat_map(|index| words[index.inner() as usize].words.iter())
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
    fn try_add_word(&self, word: &LetterCounts, word_index: WordId) -> Option<Self> {
        let letter_counts = self.letter_counts.try_union(&word)?;
        let mut word_indexes = self.word_indexes.clone();
        word_indexes.set_bit(&word_index, true);

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

    // fn can_add(&self, other_word: &LetterCounts, word_index: WordId, max_size: u8) -> bool {
    //     if self.word_indexes.get_bit(&word_index) {
    //         return false;
    //     }
    //     let Some(letter_counts) = self.letter_counts.try_union(&other_word) else {
    //         return false;
    //     };
    //     if letter_counts == self.letter_counts {
    //         return true;
    //     } else {
    //         if self.total_letters == max_size {
    //             return false;
    //         }

    //         let diff = letter_counts
    //             .try_difference(&self.letter_counts)
    //             .unwrap_or_default();
    //         let new_elements = diff.into_iter().count() as u8;

    //         let new_count = self.total_letters + new_elements;

    //         return new_count <= max_size;
    //     }
    // }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use itertools::Itertools;
    use std::{collections::HashSet, time::Instant};
    use test_case::test_case;

    #[test]
    pub fn test_possible_combinations() {
        let input = "hydrogen\nhelium\nlithium\nboron\ncarbon";

        let _now = Instant::now();

        let words = make_finder_group_vec_from_file(input);
        let word_letters: Vec<LetterCounts> = words.iter().map(|x| x.counts).collect_vec();

        let possible_combinations: Vec<WordCombination> =
            get_combinations(word_letters.as_slice(), 16);

        //println!("{:?}", _now.elapsed());

        let expected = "boron\nboron, carbon\nboron, carbon, helium\nboron, carbon, helium, hydrogen\nboron, carbon, helium, lithium\nboron, carbon, hydrogen\nboron, carbon, lithium\nboron, helium\nboron, helium, hydrogen\nboron, helium, hydrogen, lithium\nboron, helium, lithium\nboron, hydrogen\nboron, hydrogen, lithium\nboron, lithium";

        //println!("{}", possible_combinations.iter().map(|x| x.display_string(words.as_slice())).sorted().dedup().join("\\n"));

        let actual_set: HashSet<_> = possible_combinations
            .iter()
            .map(|x| x.display_string(words.as_slice()))
            .collect();

        for combo in expected.split("\n") {
            assert!(actual_set.contains(combo));
        }

        assert_eq!(
            actual_set.len(),
            expected.split("\n").count(),
            "Number of combinations"
        );
    }

    #[test_case("monkey\ncow\nant\nantelope", "monkey\ncow\nant\nantelope")]
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
    #[test_case("White\nYellow\nBlue\nRed\nGreen\nBlack\nBrown\nAzure\nIvory\nTeal\nSilver\nPurple\nGray\nOrange\nMaroon\nCharcoal\nAquamarine\nCoral\nFuchsia\nWheat\nLime\nCrimson\nKhaki\npink\nMagenta\nGold\nPlum\nOlive\nCyan","Black\nCoral\nCyan\nGray\nGreen\nIvory\nOlive\nOrange\nRed\nTeal") ]
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
                .sorted()
                .map(|x| x.display_string(words.as_slice()))
                .join("\n");

            println!("actual {actual}");
        }

        assert!(contains_expected);
    }
}
