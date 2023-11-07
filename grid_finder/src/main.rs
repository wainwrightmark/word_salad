pub mod clustering;

use clap::Parser;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use indicatif_log_bridge::LogWrapper;
use itertools::Itertools;
use log::{debug, info};
use rayon::prelude::*;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::File,
    io::{self, BufWriter, Write},
    path::Path,
    sync::atomic::AtomicUsize,
};
use ws_core::{
    finder::{counter::FakeCounter, helpers::*, node::GridResult},
    prelude::*,
};

use crate::clustering::cluster_words;

#[derive(Parser, Debug)]
#[command()]
struct Options {
    #[arg(short, long, default_value = "data")]
    pub folder: String,
}

fn main() {
    let logger =
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).build();
    let multi = MultiProgress::new();

    LogWrapper::new(multi.clone(), logger).try_init().unwrap();

    let options = Options::parse();
    do_finder(options);

    info!("Finished... Press enter");
    io::stdin().read_line(&mut String::new()).unwrap();
}

fn do_finder(options: Options) {
    info!("Starting up");

    let folder = std::fs::read_dir(options.folder).unwrap();

    let paths: Vec<_> = folder.collect();

    let pb = ProgressBar::new(paths.len() as u64)
        .with_style(ProgressStyle::with_template("{msg} {wide_bar} {pos:4}/{len:4}").unwrap())
        .with_message("Data files");

    let _ = std::fs::create_dir("grids");
    let _ = std::fs::create_dir("clusters");

    for path in paths.iter() {
        let path = path.as_ref().unwrap().path();
        let file_name = path.file_name().unwrap().to_string_lossy();
        let write_path = format!("grids/{file_name}",);
        info!("{}", write_path);
        let data = std::fs::read_to_string(path.clone()).unwrap();

        let word_map = make_words_from_file(data.as_str());

        let grids_write_path = Path::new(write_path.as_str());
        let grids_file =
            std::fs::File::create(grids_write_path).expect("Could not find output folder");
        let grids_writer = BufWriter::new(grids_file);
        let grids = create_grids(&word_map, grids_writer);

        let all_words = grids
            .iter()
            .flat_map(|grid| &grid.words)
            .cloned()
            .sorted()
            .dedup()
            .collect_vec();

        let clusters = cluster_words(grids, &all_words, 10);

        let clusters_write_path = format!("clusters/{file_name}",);
        let clusters_write_path = Path::new(clusters_write_path.as_str());
        let clusters_text = clusters.into_iter().map(|x| x.to_string()).join("\n\n");
        std::fs::write(clusters_write_path, clusters_text).unwrap();

        pb.inc(1);
    }
}

fn create_grids(word_map: &WordMultiMap, mut file: BufWriter<File>) -> Vec<GridResult> {
    let word_letters: Vec<LetterCounts> = word_map.keys().cloned().sorted().collect_vec();
    let possible_combinations: BTreeMap<LetterCounts, usize> = get_combinations(
        Multiplicities::default(),
        word_letters.as_slice(),
        16,
        &word_map,
    );

    info!(
        "{c} possible combinations found",
        c = possible_combinations.len()
    );
    // info!("");

    let groups = possible_combinations.into_iter().into_group_map_by(|x| x.1);
    let ordered_groups = groups.into_iter().sorted_unstable().rev().collect_vec();

    let mut previous_solutions: BTreeSet<LetterCounts> = Default::default();

    let mut all_solutions: Vec<GridResult> = Default::default();

    //let (sender, receiver) = std::sync::mpsc::channel::<LetterCounts>();
    for (size, group) in ordered_groups {
        let solution_count = AtomicUsize::new(0);
        let impossible_count = AtomicUsize::new(0);
        let redundant_count = AtomicUsize::new(0);
        let pb = ProgressBar::new(group.len() as u64)
            .with_style(ProgressStyle::with_template("{msg} {wide_bar} {pos:4}/{len:4}").unwrap())
            .with_message(format!("Groups of size {size}"));
        //let latest_solution = ProgressBar::new_spinner();
        let solutions: Vec<GridResult> = group
            .par_iter()
            .map(|(letters, _)| {
                if previous_solutions
                    .range(letters..)
                    .any(|prev| prev.is_superset(letters))
                {
                    pb.inc(1);
                    redundant_count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                    return None;
                }

                let mut counter = FakeCounter;

                let finder_words: Vec<FinderWord> = word_map
                    .iter()
                    .flat_map(|s| s.1)
                    .filter(|word| letters.is_superset(&word.counts))
                    .map(|z| z)
                    .cloned()
                    .collect();
                //let raw_text = get_raw_text(&letters);

                let result = ws_core::finder::node::try_make_grid_with_blank_filling(
                    *letters,
                    &finder_words,
                    Character::E,
                    &mut counter,
                );
                pb.inc(1);
                match result {
                    Some(grid_result) => {
                        solution_count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                        return Some(grid_result);
                    }
                    None => {
                        impossible_count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                        return None;
                    }
                }
            })
            .flatten()
            .collect();

        let lines = solutions
            .iter()

            .join("\n");
        if !lines.is_empty() {
            file.write((lines + "\n").as_bytes()).unwrap();
        }

        let solution_count = solution_count.into_inner();
        let impossible_count = impossible_count.into_inner();
        let redundant_count = redundant_count.into_inner();

        pb.finish_with_message(format!("{size:2} words: {solution_count:4} Solutions {impossible_count:45} Impossible {redundant_count:4} Redundant"));
        //latest_solution.finish();

        previous_solutions.extend(solutions.iter().map(|x| x.letters).clone());
        all_solutions.extend(solutions.into_iter());
    }

    all_solutions
}

fn get_combinations(
    multiplicities: Multiplicities,
    possible_words: &[LetterCounts],
    max_size: u8,
    multi_map: &WordMultiMap,
) -> BTreeMap<LetterCounts, usize> {
    let pb = ProgressBar::new(possible_words.len() as u64)
        .with_style(ProgressStyle::with_template("{msg} {wide_bar} {pos}/{len}").unwrap())
        .with_message("Getting word combinations");

    let upper_bounds = 1..(possible_words.len());
    let result = upper_bounds
        .into_iter()
        .map(|upper| &possible_words[0..=upper])
        .par_bridge()
        .map(|words| {
            let mut possible_combinations: BTreeMap<LetterCounts, usize> = BTreeMap::default();
            get_combinations_inner(
                &mut possible_combinations,
                0,
                multiplicities,
                words,
                max_size,
                multi_map,
            );
            possible_combinations
        })
        .reduce(
            || BTreeMap::<LetterCounts, usize>::default(),
            |a, b| {
                pb.inc(1);
                let (mut big, small) = if a.len() >= b.len() { (a, b) } else { (b, a) };
                if small.is_empty() {
                    return big;
                }

                for (key, value) in small.into_iter() {
                    match big.entry(key) {
                        std::collections::btree_map::Entry::Vacant(v) => {
                            v.insert(value);
                        }
                        std::collections::btree_map::Entry::Occupied(mut o) => {
                            if *o.get() < value {
                                *o.get_mut() = value;
                            }
                        }
                    }
                }

                big
            },
        );

    pb.finish();
    result
}

fn get_combinations_inner(
    possible_combinations: &mut BTreeMap<LetterCounts, usize>,
    word_count: usize,
    multiplicities: Multiplicities,
    mut possible_words: &[LetterCounts],
    max_size: u8,
    multi_map: &WordMultiMap,
) {
    loop {
        let Some((word, npw)) = possible_words.split_last() else {
            break;
        };
        possible_words = npw;

        let Some(new_multiplicities) = multiplicities.try_add_word(&word) else {
            panic!("Could not add word to multiplicities");
        };

        let new_word_count = word_count + 1;

        if new_multiplicities.count <= max_size {
            match possible_combinations.entry(new_multiplicities.set) {
                std::collections::btree_map::Entry::Vacant(v) => {
                    v.insert(new_word_count);
                }
                std::collections::btree_map::Entry::Occupied(mut o) => {
                    if *o.get() < new_word_count {
                        o.insert(new_word_count);
                    }
                }
            };

            get_combinations_inner(
                possible_combinations,
                new_word_count,
                new_multiplicities,
                possible_words,
                max_size,
                multi_map,
            );
        }
    }
}

pub fn get_raw_text(counts: &LetterCounts) -> String {
    counts.into_iter().join("")
}

pub fn write_words(word: &Vec<CharsArray>) -> String {
    word.iter().map(|c| c.iter().join("")).join(", ")
}

#[derive(Debug, Clone, PartialEq, Default)]
struct CharacterCounter([u8; 26]);

#[derive(Debug, Clone, PartialEq, Default, Copy, Ord, PartialOrd, Eq)]
struct Multiplicities {
    count: u8,
    set: LetterCounts,
}

impl Multiplicities {
    #[must_use]
    fn try_add_word(&self, word: &LetterCounts) -> Option<Self> {
        let union = self.set.try_union(&word)?;

        if union == self.set {
            Some(*self)
        } else {
            let diff = union.try_difference(&self.set)?;
            let new_elements = diff.into_iter().count() as u8;
            Some(Self {
                set: union,
                count: self.count + new_elements,
            })
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use std::time::Instant;
    use test_case::test_case;

    #[test]
    pub fn test_possible_combinations() {
        let input = "monkey\ncow\nant\nantelope";

        let now = Instant::now();

        let words = make_words_from_file(input);
        let word_letters: Vec<LetterCounts> = words.keys().cloned().collect_vec();

        let possible_combinations: BTreeMap<LetterCounts, usize> = get_combinations(
            Multiplicities::default(),
            word_letters.as_slice(),
            16,
            &words,
        );

        info!("{:?}", now.elapsed());

        let expected = "[ant]\n[cow]\n[ant, cow]\n[ant, antelope]\n[monkey]\n[ant, monkey]\n[ant, antelope, cow]\n[cow, monkey]\n[ant, cow, monkey]\n[ant, antelope, monkey]\n[ant, antelope, cow, monkey]";

        let actual = possible_combinations
            .into_iter()
            .map(|x| format!("[{}]", get_possible_words_text(&x.0, &words)))
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

        let expected_words = make_words_from_file(expected_member);
        let mut expected = LetterCounts::default();
        for (w, _) in expected_words {
            expected = expected
                .try_union(&w)
                .expect("Should be able to union expected");
        }

        let words = make_words_from_file(input);

        let word_letters: Vec<LetterCounts> = words.keys().cloned().collect_vec();

        let possible_combinations: BTreeMap<LetterCounts, usize> = get_combinations(
            Multiplicities::default(),
            word_letters.as_slice(),
            16,
            &words,
        );

        info!("{:?}", now.elapsed());

        let contains_expected = possible_combinations.contains_key(&expected);

        if !contains_expected {
            let actual = possible_combinations
                .into_iter()
                .map(|x| format!("[{}]", get_possible_words_text(&x.0, &words)))
                .join("\n");

            info!("{actual}");
        }

        assert!(contains_expected);
    }

    fn get_possible_words_text(counts: &LetterCounts, word_map: &WordMultiMap) -> String {
        let words = word_map.iter().filter(|(c, _w)| counts.is_superset(c));

        words
            .flat_map(|(_c, words)| words.iter().map(|w| w.text.as_str()))
            .sorted()
            .join(", ")
    }
}
