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
    #[arg(short, long, default_value = "data")]
    pub folder: String,

    #[arg(short, long, default_value = "5")]
    pub minimum: u32,

    #[arg(long)]
    pub search: Option<String>
}

fn main() {
    let logger =
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).build();
    let multi = MultiProgress::new();

    LogWrapper::new(multi.clone(), logger).try_init().unwrap();

    let options = Options::parse();

    if let Some(search) = options.search{
        search::do_search(search);
    }
    else{
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

        info!("{} Words", word_map.len());
        for word in word_map.iter().map(|x|x.text.clone()).sorted(){
            info!("{word}", )
        }

        for (a,b) in word_map.iter().sorted_by_key(|x|&x.array).tuple_windows(){
            if b.array.starts_with(&a.array){
                warn!("'{}' is a prefix of '{}'", a.text, b.text)
            }
        }

        let grids_write_path = Path::new(write_path.as_str());
        let grids_file =
            std::fs::File::create(grids_write_path).expect("Could not find output folder");
        let grids_writer = BufWriter::new(grids_file);
        let grids = create_grids(&word_map, grids_writer, options.minimum);

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
    mut file: BufWriter<File>,
    min_size: u32,
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

                let result = ws_core::finder::node::try_make_grid_with_blank_filling(
                    set.letter_counts,
                    &finder_words,
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
        if !lines.is_empty() {
            file.write((lines + "\n").as_bytes()).unwrap();
        }

        let solution_count = solutions.len();
        let impossible_count = results_count - solution_count;

        pb.finish_with_message(format!(
            "{size:2} words: {solution_count:4} Solutions {impossible_count:5} Impossible"
        ));

        all_solutions.extend(solutions);
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
