use indicatif::{ProgressBar, ProgressStyle};

use itertools::Itertools;
use log::info;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use std::{
    collections::BTreeMap,
    fs::File,
    io::{BufWriter, Write},
};
use ws_core::{
    finder::{counter::FakeCounter, helpers::*, node::GridResult},
    prelude::*,
};

use crate::word_set::WordSet;
use hashbrown::HashMap;
use std::collections::btree_set::BTreeSet as MySet; //todo try other sets - roaring?

use crate::combinations::{self, get_combinations, WordCombination};

#[derive(Debug, Default)]
struct SolutionGroup<const W: usize> {
    sets: Vec<WordSet<W>>,
    extras: MySet<WordSet<W>>,
}

pub fn create_grids<const W: usize>(
    category: &str,
    all_words: &Vec<FinderGroup>,
    exclude_words: &Vec<FinderSingleWord>,
    mut file: BufWriter<File>,
    min_size: u32,
    max_grids: Option<usize>,
    resume_grids: Vec<GridResult>,
) -> Vec<GridResult> {
    let word_letters: Vec<LetterCounts> = all_words.iter().map(|x| x.counts).collect();
    let mut possible_combinations: Vec<WordSet<W>> =
        get_combinations(Some(category.to_string()), word_letters.as_slice(), 16);

    info!(
        "{c} possible combinations founds",
        c = possible_combinations.len()
    );

    possible_combinations.sort_unstable_by_key(|x| std::cmp::Reverse(x.count()));

    let mut all_solutions: Vec<GridResult> = vec![];
    let mut all_solved_combinations: Vec<WordSet<W>> = vec![];

    let mut grouped_combinations: BTreeMap<u32, SolutionGroup<W>> = Default::default();

    possible_combinations
        .into_iter()
        .group_by(|x| x.count())
        .into_iter()
        .for_each(|(count, group)| {
            let sg = grouped_combinations.entry(count).or_default();
            sg.sets.extend(group);
        });

    let resume_grids: HashMap<WordSet<W>, GridResult> = resume_grids
        .into_iter()
        .map(|g| (WordSet(g.get_word_bitset(&all_words)), g))
        .collect();
    let min_resume_grid = resume_grids.keys().map(|x| x.count()).min();
    if min_resume_grid.is_some() {
        info!(
            "Resuming with {} grids of size {} or above",
            resume_grids.len(),
            min_resume_grid.unwrap_or_default()
        );
        // for (size, count) in resume_grids.keys().map(|x| x.count()).counts(){
        //     info!("{count} grids of size {size}");
        // }
    }

    while let Some((size, group)) = grouped_combinations.pop_last() {
        if size < min_size {
            info!(
                "Skipping {} of size {size}",
                group.sets.len() + group.extras.len()
            );
            break;
        }

        let pb = ProgressBar::new((group.sets.len() + group.extras.len()) as u64)
            .with_style(
                ProgressStyle::with_template(
                    "{prefix} {msg} {elapsed:6} {human_pos:11}/{human_len:11} {bar}",
                )
                .unwrap(),
            )
            .with_prefix(format!("{category}: {size:2} words"));

        let results: Vec<(WordCombination<W>, Option<GridResult>)> =
            if min_resume_grid.is_some_and(|mrg| mrg <= size) {
                pb.set_message(format!("{:<16}", "resuming"));
                group
                    .sets
                    .into_par_iter()
                    .chain(group.extras.into_par_iter())
                    //.filter(|x| !all_solved_combinations.iter().any(|sol| sol.is_superset(x)))
                    .flat_map(|set| {
                        combinations::WordCombination::from_bit_set(set, word_letters.as_slice())
                    })
                    .map(|combination| {
                        let result = resume_grids.get(&combination.word_indexes).cloned();
                        pb.inc(1);
                        (combination, result)
                    })
                    .collect()
            } else {
                pb.set_message(format!("{:<16}", "finding"));
                group
                    .sets
                    .into_par_iter()
                    .chain(group.extras.into_par_iter())
                    //.filter(|x| !all_solved_combinations.iter().any(|sol| sol.is_superset(x)))
                    .flat_map(|set| {
                        combinations::WordCombination::from_bit_set(set, word_letters.as_slice())
                    })
                    .map(|set: WordCombination<W>| {
                        let mut counter = FakeCounter;
                        let finder_words = set.get_single_words(all_words);

                        let exclude_words = exclude_words
                            .iter()
                            .filter(|w| set.letter_counts.is_superset(&w.counts))
                            .filter(|excluded_word| {
                                !finder_words
                                    .iter()
                                    .any(|fw| excluded_word.is_strict_substring(fw))
                            })
                            .cloned()
                            .collect_vec();

                        let mut result = None;

                        ws_core::finder::node::try_make_grid_with_blank_filling(
                            set.letter_counts,
                            &finder_words,
                            &exclude_words,
                            Character::E,
                            &mut counter,
                            &mut result,
                        );

                        pb.inc(1);

                        (set, result)
                    })
                    .collect()
            };

        pb.set_message(format!("{:<16}", "finishing"));

        let _results_count = results.len();

        let mut solutions: Vec<GridResult> = vec![];
        let next_group = grouped_combinations
            .entry(size.saturating_sub(1))
            .or_default();

        pb.set_length(results.len() as u64);
        pb.set_position(0);

        for (combination, result) in results.into_iter() {
            if let Some(mut solution) = result {
                let _ = ws_core::finder::orientation::try_optimize_orientation(&mut solution);
                solutions.push(solution);
                all_solved_combinations.push(combination.word_indexes);
            } else {
                next_group
                    .extras
                    .extend(combinations::shrink_bit_sets(&combination.word_indexes));
            }
            pb.inc(1);
        }

        if !solutions.is_empty() {
            let lines = solutions
                .iter()
                .sorted_by_cached_key(|x| x.words.iter().sorted().join(""))
                .join("\n");
            file.write_all((lines + "\n").as_bytes()).unwrap();
            file.flush().expect("Could not flush to file");
        }

        if size > min_size {
            for c in all_solved_combinations.iter() {
                if c.count() == size {
                    //faster way to iter subsets
                    for element in c.into_iter() {
                        let mut subset = c.clone();
                        subset.set_bit(element, false);
                        next_group.extras.remove(&subset);
                    }
                } else {
                    for subset in c.iter_subsets(size - 1) {
                        next_group.extras.remove(&WordSet(subset));
                    }
                }
            }
        }

        let solution_count = solutions.len();

        pb.finish_with_message(format!("{solution_count:6} Solutions"));

        all_solutions.extend(solutions);

        if max_grids.is_some_and(|mg| mg < all_solutions.len()) {
            return all_solutions;
        }
    }
    all_solutions
}
