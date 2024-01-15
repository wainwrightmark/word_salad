pub mod cluster_ordering;
pub mod clustering;
pub mod combinations;
pub mod search;
pub mod word_layout;

use clap::Parser;
use const_sized_bit_set::BitSet;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use indicatif_log_bridge::LogWrapper;
use itertools::Itertools;
use log::{info, warn};
use rayon::iter::{ParallelDrainFull, ParallelIterator, IntoParallelRefIterator, IntoParallelIterator};
use std::{
    collections::{HashSet, BTreeSet},
    fs::{DirEntry, File},
    io::{self, BufWriter, Write},
    path::{Path, PathBuf},
    str::FromStr, borrow::BorrowMut,
};
use ws_core::{
    finder::{counter::FakeCounter, helpers::*, node::GridResult},
    prelude::*,
};

use hashbrown::hash_set::HashSet as MySet;

use crate::{
    clustering::cluster_words,
    combinations::{get_combinations, WordCombination},
};

const BIT_SET_WORDS: usize = 2;

#[derive(Parser, Debug)]
#[command()]
struct Options {
    /// Folder to look in for data
    #[arg(short, long, default_value = "data")]
    pub folder: String,

    /// Minimum number of words in a grid
    #[arg(short, long, default_value = "5")]
    pub minimum: u32,

    /// Maximum number of grids to return
    #[arg(short, long, default_value = "10000")]
    pub grids: u32,
    /// search all found grids for a particular word or list of words
    #[arg(long)]
    pub search: Option<String>,

    /// Check word layouts
    #[arg(long)]
    pub check_layout: bool,

    /// If set, will find clusters for all existing grids rather than finding new grids
    #[arg(short, long, default_value = "false")]
    pub cluster: bool,

    #[arg(long, default_value = "10")]
    pub max_clusters: u32,
}

fn main() {
    let logger =
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).build();
    let multi = MultiProgress::new();

    LogWrapper::new(multi.clone(), logger).try_init().unwrap();

    let options = Options::parse();

    if let Some(search) = options.search {
        search::do_search(search);
    } else if options.check_layout {
        word_layout::do_word_layout();
    } else if options.cluster {
        do_cluster(options);
    } else {
        do_finder(options);
    }

    info!("Finished... Press enter");
    io::stdin().read_line(&mut String::new()).unwrap();
}

fn do_cluster(options: Options) {
    let folder = std::fs::read_dir("grids").unwrap();

    let paths: Vec<_> = folder.collect();

    let pb: ProgressBar = ProgressBar::new(paths.len() as u64)
        .with_style(ProgressStyle::with_template("{msg:50} {wide_bar} {pos:2}/{len:2}").unwrap())
        .with_message("Data files");

    let _ = std::fs::create_dir("clusters");

    for path in paths {
        let grid_path = path.as_ref().unwrap().path();
        let file_name = grid_path.file_name().unwrap().to_string_lossy();

        pb.set_message(file_name.to_string());

        let grid_file_text = std::fs::read_to_string(grid_path.clone()).unwrap();

        fn filter_enough_grids(
            grid: &GridResult,
            count: &mut usize,
            enough: usize,
            filter_below: &mut usize,
        ) -> bool {
            if grid.words.len() < *filter_below {
                return false;
            }
            *count += 1;
            if *count >= enough {
                *filter_below = (*filter_below).max(grid.words.len());
            }
            true
        }
        let mut count = 0;
        let mut filter_below = 0;

        let grids = grid_file_text
            .lines()
            .map(|l| GridResult::from_str(l).unwrap())
            .filter(|x| x.words.len() >= options.minimum as usize)
            .filter(|x| {
                filter_enough_grids(x, &mut count, options.grids as usize, &mut filter_below)
            })
            .collect_vec();

        let all_words = grids
            .iter()
            .flat_map(|grid| &grid.words)
            .cloned()
            .sorted()
            .dedup()
            .collect_vec();

        info!(
            "{file_name} found {:6} grids with {:3} different words",
            grids.len(),
            all_words.len()
        );

        let clusters = cluster_words::<BIT_SET_WORDS>(grids, &all_words, options.max_clusters as usize);

        let clusters_write_path = format!("clusters/{file_name}",);
        let clusters_write_path = Path::new(clusters_write_path.as_str());
        let clusters_text = clusters.into_iter().map(|x| x.to_string()).join("\n\n");
        std::fs::write(clusters_write_path, clusters_text).unwrap();

        pb.inc(1);
    }

    pb.finish();
}

struct FinderCase {
    word_map: Vec<FinderGroup>,
    data_path: PathBuf,
    data_file_text: String,
    file_name: String,
    write_path: String,
}

impl FinderCase {
    fn new_from_path(dir_entry: DirEntry) -> Self {
        let data_path = dir_entry.path();
        let file_name = data_path.file_name().unwrap().to_string_lossy().to_string();
        let write_path = format!("grids/{file_name}",);
        info!("{}", write_path);
        let data_file_text = std::fs::read_to_string(data_path.clone()).unwrap();

        let word_map = make_finder_group_vec_from_file(data_file_text.as_str());

        Self {
            word_map,
            data_path,
            file_name,
            data_file_text,
            write_path,
        }
    }
}

fn do_finder(options: Options) {
    info!("Starting up");

    let folder = std::fs::read_dir(options.folder).unwrap();

    let paths: Vec<_> = folder
        .into_iter()
        .map(|path| FinderCase::new_from_path(path.unwrap()))
        .sorted_by_key(|x| x.word_map.len())
        .collect();

    let pb: ProgressBar = ProgressBar::new(paths.len() as u64)
        .with_style(ProgressStyle::with_template("{msg} {wide_bar} {pos:2}/{len:2}").unwrap())
        .with_message("Data files");

    let _ = std::fs::create_dir("grids");
    let _ = std::fs::create_dir("clusters");
    let _ = std::fs::create_dir("done");

    for finder_case in paths.iter() {
        let FinderCase {
            word_map,
            data_path,
            file_name,
            write_path,
            data_file_text,
        } = finder_case;

        info!("Found {} Words", word_map.len());
        for word in word_map.iter().map(|x| x.text.clone()).sorted() {
            info!("{word}",)
        }

        for (a, b) in word_map
            .iter()
            .flat_map(|x| x.words.iter())
            .cloned()
            .sorted_by_cached_key(|x| x.array.clone())
            .tuple_windows()
        {
            if b.array.starts_with(&a.array) {
                warn!("'{}' is a prefix of '{}'", a.text, b.text)
            }
        }

        let master_path = format!("master/{file_name}");
        let master_file_text = std::fs::read_to_string(master_path).unwrap_or_default();

        let mut master_words: Vec<FinderSingleWord> =
            make_finder_group_vec_from_file(&master_file_text)
                .into_iter()
                .flat_map(|x| x.words)
                .collect_vec();
        if master_words.len() > 0 {
            let word_set: HashSet<_> = word_map
                .iter()
                .flat_map(|x| x.words.iter())
                .cloned()
                .map(|x| x.array.clone())
                .collect();
            let total_master_count = master_words.len();
            master_words.retain(|x| !word_set.contains(x.array.as_slice()));
            let new_master_count = master_words.len();

            info!("Found {total_master_count} words on the master list. {new_master_count} will be treated as exclusions");

            if !master_words.is_empty() {
                let mut bad_master_words: HashSet<FinderSingleWord> = Default::default();
                for word in word_map.iter().flat_map(|x| x.words.iter()) {
                    for master_word in master_words.iter() {
                        if master_word.is_strict_substring(&word) {
                            warn!(
                                "{} is a superstring of {} so will be impossible",
                                word.text, master_word.text
                            );
                            bad_master_words.insert(master_word.clone());
                        }
                    }
                }
                for bad in bad_master_words.drain() {
                    if let Some((index, word)) = master_words.iter().find_position(|x| **x == bad) {
                        info!("Removed '{}' from master words", word.text);
                        master_words.remove(index);
                    }
                }
            }
        }

        let grids_write_path = Path::new(write_path.as_str());
        let grids_file =
            std::fs::File::create(grids_write_path).expect("Could not find output folder");
        let grids_writer = BufWriter::new(grids_file);
        let grids = create_grids(
            &word_map,
            &master_words,
            grids_writer,
            options.minimum,
            options.grids,
        );

        let all_words = grids
            .iter()
            .flat_map(|grid| &grid.words)
            .cloned()
            .sorted()
            .dedup()
            .collect_vec();

        let clusters: Vec<clustering::Cluster> =
            cluster_words::<BIT_SET_WORDS>(grids, &all_words, options.max_clusters as usize);

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
    all_words: &Vec<FinderGroup>,
    exclude_words: &Vec<FinderSingleWord>,
    mut file: BufWriter<File>,
    min_size: u32,
    max_grids: u32,
) -> Vec<GridResult> {
    let word_letters: Vec<LetterCounts> = all_words.iter().map(|x| x.counts).collect();
    let possible_combinations: Vec<combinations::WordCombination<BIT_SET_WORDS>> =
        get_combinations(word_letters.as_slice(), 16);

    info!(
        "{c} possible combinations found. They may take a while to sort",
        c = possible_combinations.len()
    );

    let mut ordered_combinations: [MySet<BitSet<2>>; 32] = std::array::from_fn(|_|MySet::<BitSet<BIT_SET_WORDS>>::default());

    for x in possible_combinations.into_iter(){
        let words = x.word_indexes.count();

        ordered_combinations[words as usize].insert(x.word_indexes);
    }

    let mut ordered_combinations: Vec<(usize, MySet<BitSet<2>>)> = ordered_combinations.into_iter().enumerate().filter(|x|!x.1.is_empty()) .collect();

    info!("{c} group sizes found", c = ordered_combinations.len());
    // for (s, set) in ordered_combinations.iter(){
    //     info!("{} of size {s}", set.len());
    // }

    let mut all_solutions: Vec<GridResult> = vec![];
    let mut solved_sets: Vec<BitSet<BIT_SET_WORDS>> = vec![];

    while let Some((size, mut sets)) = ordered_combinations.pop() {
        if (size as u32) < min_size {
            info!("Skipping {} of size {size}", sets.len());
            break;
        }
        let now = std::time::Instant::now();
        let pb = ProgressBar::new(sets.len() as u64)
            .with_style(ProgressStyle::with_template("{msg} {wide_bar} {pos:7}/{len:7}").unwrap())
            .with_message(format!("Groups of size {size}"));

        if !solved_sets.is_empty()
        {
            sets.retain(|x| !solved_sets.iter().any(|sol| x == &sol.intersect(x)));
            //NOTE this does actually do something
        }

        let results: Vec<(WordCombination<BIT_SET_WORDS>, Option<GridResult>)> = sets
        .into_par_iter()

            .flat_map(|set| {
                combinations::WordCombination::from_bit_set(set, word_letters.as_slice())
            })
            .map(|set| {
                let mut counter = FakeCounter;
                let finder_words = set.get_single_words(all_words);

                let exclude_words = exclude_words
                    .iter()
                    .filter(|w| set.letter_counts.is_superset(&w.counts))
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
            .collect();

        let results_count = results.len();

        let (_, mut next_set) = ordered_combinations.pop().unwrap_or_default();
        let mut solutions: Vec<GridResult> = vec![];

        for (combination, result) in results.into_iter() {
            if let Some(mut solution) = result {
                solved_sets.push(combination.word_indexes);
                ws_core::finder::orientation::optimize_orientation(&mut solution);
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
        let elapsed = now.elapsed().as_secs_f32();
        pb.finish_with_message(format!(
            "{size:2} words: {solution_count:4} Solutions {impossible_count:5} Impossible {elapsed:.3}s"
        ));

        all_solutions.extend(solutions);

        if all_solutions.len() > max_grids as usize {
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
