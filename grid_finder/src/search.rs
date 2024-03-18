use std::str::FromStr;

use indicatif::{ProgressBar, ProgressStyle};
use log::info;
use prime_bag::{PrimeBag128, PrimeBagElement};
use ws_core::{prelude::*, DesignedLevel, Word};

use crate::SearchArgs;

#[derive(Debug, PartialEq, Eq)]
struct WordLengthBagMember(usize);

impl PrimeBagElement for WordLengthBagMember {
    fn into_prime_index(&self) -> usize {
        match self.0.checked_sub(4) {
            Some(index) => index,
            None => panic!("Word length must be at least 4"),
        }
    }

    fn from_prime_index(value: usize) -> Self {
        Self(value + 4)
    }
}

fn level_to_prime_bag(level: &impl LevelTrait) -> PrimeBag128<WordLengthBagMember> {
    let bag = PrimeBag128::try_from_iter(
        level
            .words()
            .iter()
            .map(|x| x.characters().len())
            .map(|x| WordLengthBagMember(x)),
    )
    .expect("Cannot convert level to prime bag");
    bag
}

pub fn do_search(search_args: &SearchArgs) {
    let folder = std::fs::read_dir("grids").unwrap();

    let words: Vec<_> = search_args
        .words
        .split_ascii_whitespace()
        .map(|s| Word::from_str(s).unwrap())
        .map(|w| {
            let lc = w.letter_counts().unwrap();
            (w, lc)
        })
        .collect();

    let expected_lengths: PrimeBag128<WordLengthBagMember> = prime_bag::PrimeBag128::try_from_iter(
        search_args
            .lengths
            .split_ascii_whitespace()
            .map(|x| x.parse().expect("Could not parse value as integer"))
            .map(|x| WordLengthBagMember(x)),
    )
    .expect("Cannot create prime bag from lengths");

    let paths: Vec<_> = folder.collect();

    let pb: ProgressBar = ProgressBar::new(paths.len() as u64)
        .with_style(ProgressStyle::with_template("{msg} {bar} {pos:2}/{len:2}").unwrap())
        .with_message("Grid files");

    let mut total_solutions = 0usize;
    let mut total_possibles = 0usize;

    for path in paths.iter() {
        let grids_path = path.as_ref().unwrap().path();
        let grid_file_text = std::fs::read_to_string(grids_path.clone()).unwrap();

        for line in grid_file_text.lines() {
            let level = DesignedLevel::from_tsv_line(line).unwrap();

            let level_letter_counts = level
                .letter_counts()
                .expect("Could not get grid letter counts");

            let length_allowed: bool;

            if expected_lengths.is_empty() {
                length_allowed = true;
            } else {
                let level_lengths = level_to_prime_bag(&level);

                if search_args.exact_lengths {
                    length_allowed = level_lengths == expected_lengths;
                } else {
                    length_allowed = level_lengths.is_superset(&expected_lengths);
                }
            }

            if !length_allowed{
                continue;
            }

            if words.is_empty() {
                total_possibles += 1;
                total_solutions += 1;

                info!("Found  {level}",)
            } else {
                for (word, _) in words
                    .iter()
                    .filter(|(_, lc)| level_letter_counts.is_superset(lc))
                {
                    total_possibles += 1;
                    if let Some(..) = word.find_solution(&level.grid) {
                        total_solutions += 1;
                        info!(
                            "Found {word} in {file} {level} ",
                            file = grids_path
                                .file_name()
                                .map(|x| x.to_string_lossy())
                                .unwrap_or_default(),
                            word = word.text
                        )
                    }
                }
            }
        }

        pb.inc(1);
    }

    info!("{total_solutions} solutions found ({total_possibles} possibles)")
}
