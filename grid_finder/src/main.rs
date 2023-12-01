pub mod clustering;
pub mod combinations;
pub mod orientation;
pub mod search;

use clap::Parser;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use indicatif_log_bridge::LogWrapper;
use itertools::Itertools;
use log::{info, warn};
use rayon::prelude::*;
use std::{
    collections::HashSet,
    fs::File,
    io::{self, BufWriter, Write},
    path::Path,
};
use ws_core::{
    finder::{counter::FakeCounter, helpers::*, node::GridResult},
    prelude::*,
};

use crate::{
    clustering::cluster_words,
    combinations::{get_combinations, WordCombination, WordSet},
};

#[derive(Parser, Debug)]
#[command()]
struct Options {
    /// Folder to look in for data
    #[arg(short, long, default_value = "data")]
    pub folder: String,

    /// Minimum number of words in a grid
    #[arg(short, long, default_value = "0")]
    pub minimum: u32,

    /// Maximum number of grids to return
    #[arg(short, long, default_value = "10000")]
    pub grids: u32,
    /// search all found grids for a particular word or list of words
    #[arg(long)]
    pub search: Option<String>,
}

fn main() {
    let logger =
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).build();
    let multi = MultiProgress::new();

    LogWrapper::new(multi.clone(), logger).try_init().unwrap();

    let options = Options::parse();

    if let Some(search) = options.search {
        search::do_search(search);
    } else {
        do_finder(options);
    }

    info!("Finished... Press enter");
    io::stdin().read_line(&mut String::new()).unwrap();
}

fn do_finder(options: Options) {
    info!("Starting up");

    let folder = std::fs::read_dir(options.folder).unwrap();

    let paths: Vec<_> = folder.collect();

    let pb: ProgressBar = ProgressBar::new(paths.len() as u64)
        .with_style(ProgressStyle::with_template("{msg} {wide_bar} {pos:2}/{len:2}").unwrap())
        .with_message("Data files");

    let _ = std::fs::create_dir("grids");
    let _ = std::fs::create_dir("clusters");
    let _ = std::fs::create_dir("done");


    for path in paths.iter() {
        let data_path = path.as_ref().unwrap().path();
        let file_name = data_path.file_name().unwrap().to_string_lossy();
        let write_path = format!("grids/{file_name}",);
        info!("{}", write_path);
        let data_file_text = std::fs::read_to_string(data_path.clone()).unwrap();

        let word_map = make_words_vec_from_file(data_file_text.as_str());

        info!("Found {} Words", word_map.len());
        for word in word_map.iter().map(|x| x.text.clone()).sorted() {
            info!("{word}",)
        }

        for (a, b) in word_map.iter().sorted_by_key(|x| &x.array).tuple_windows() {
            if b.array.starts_with(&a.array) {
                warn!("'{}' is a prefix of '{}'", a.text, b.text)
            }
        }

        let master_path = format!("master/{file_name}");
        let master_file_text = std::fs::read_to_string(master_path).unwrap_or_default();

        let mut master_words = make_words_vec_from_file(&master_file_text);
        if master_words.len() > 0 {
            let word_set: HashSet<_> = word_map.iter().map(|x| x.array.as_slice()).collect();
            let total_master_count = master_words.len();
            master_words.retain(|x| !word_set.contains(x.array.as_slice()));
            let new_master_count = master_words.len();

            info!("Found {total_master_count} words on the master list. {new_master_count} will be treated as exclusions")
        }

        let grids_write_path = Path::new(write_path.as_str());
        let grids_file =
            std::fs::File::create(grids_write_path).expect("Could not find output folder");
        let grids_writer = BufWriter::new(grids_file);
        let grids = create_grids(&word_map, &master_words, grids_writer, options.minimum, options.grids);

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

        let done_write_path = format!("done/{file_name}",);
        let done_write_path = Path::new(done_write_path.as_str());

        std::fs::write(done_write_path, data_file_text).unwrap();
        std::fs::remove_file(data_path).unwrap();

        pb.inc(1);
    }
}

fn create_grids(
    all_words: &Vec<FinderWord>,
    exclude_words: &Vec<FinderWord>,
    mut file: BufWriter<File>,
    min_size: u32,
    max_grids: u32
) -> Vec<GridResult> {
    let word_letters: Vec<LetterCounts> = all_words.iter().map(|x| x.counts).collect();
    let possible_combinations: Vec<combinations::WordCombination> =
        get_combinations(word_letters.as_slice(), 16);

    info!(
        "{c} possible combinations found",
        c = possible_combinations.len()
    );

    let mut ordered_combinations: Vec<(u32, HashSet<WordSet>)> = possible_combinations
        .into_iter()
        .map(|x| x.word_indexes)
        .sorted_by_key(|x| x.count())
        .group_by(|x| x.count())
        .into_iter()
        .sorted_unstable_by_key(|(k, _v)| *k)
        .map(|(word_count, v)| (word_count, HashSet::from_iter(v)))
        .collect_vec();

    let mut all_solutions: Vec<GridResult> = vec![];
    let mut solved_sets: Vec<WordSet> = vec![];

    while let Some((size, mut sets)) = ordered_combinations.pop() {
        if size < min_size {
            break;
        }
        let pb = ProgressBar::new(sets.len() as u64)
            .with_style(ProgressStyle::with_template("{msg} {wide_bar} {pos:7}/{len:7}").unwrap())
            .with_message(format!("Groups of size {size}"));

        sets.retain(|x| !solved_sets.iter().any(|sol| x == &sol.intersect(x)));

        let results: Vec<(WordCombination, Option<GridResult>)> = sets
            .par_drain()
            .flat_map(|set| {
                combinations::WordCombination::from_bit_set(set, word_letters.as_slice())
            })
            .map(|set| {
                let mut counter = FakeCounter;
                let finder_words = set.get_words(all_words);

                let exclude_words = exclude_words
                    .iter()
                    .filter(|w| set.letter_counts.is_superset(&w.counts))
                    .cloned()
                    .collect_vec();

                let result = ws_core::finder::node::try_make_grid_with_blank_filling(
                    set.letter_counts,
                    &finder_words,
                    &exclude_words,
                    Character::E,
                    &mut counter,
                );

                pb.inc(1);

                (set, result)
            })
            .collect();

        let results_count = results.len();

        let (_, mut next_set) = ordered_combinations.pop().unwrap_or_default();
        let mut solutions: Vec<GridResult> = vec![];

        for (combination, result) in results.into_iter() {
            if let Some(mut solution) = result {
                solved_sets.push(combination.word_indexes);
                orientation::optimize_orientation(&mut solution);
                solutions.push(solution);
            } else {
                for new_set in combinations::shrink_bit_sets(&combination.word_indexes) {
                    next_set.insert(new_set);
                }
            }
        }

        let lines = solutions
            .iter()
            .sorted_by_cached_key(|x| x.words.iter().sorted().join(""))
            .join("\n");
        if !lines.trim().is_empty() {
            file.write_all((lines + "\n").as_bytes()).unwrap();
        }

        let solution_count = solutions.len();
        let impossible_count = results_count - solution_count;

        pb.finish_with_message(format!(
            "{size:2} words: {solution_count:4} Solutions {impossible_count:5} Impossible"
        ));

        all_solutions.extend(solutions);

        if all_solutions.len() > max_grids as usize{
            return all_solutions;
        }

        if next_set.len() > 0 {
            ordered_combinations.push((size.saturating_sub(1), next_set));
        }
    }

    all_solutions
}

pub fn get_raw_text(counts: &LetterCounts) -> String {
    counts.into_iter().join("")
}

pub fn write_words(word: &Vec<CharsArray>) -> String {
    word.iter().map(|c| c.iter().join("")).join(", ")
}

#[derive(Debug, Clone, PartialEq, Default)]
struct CharacterCounter([u8; 26]);

#[cfg(test)]
pub mod tests {}
