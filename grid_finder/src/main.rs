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
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use std::{
    collections::{BTreeMap, HashSet},
    fs::{DirEntry, File},
    io::{self, BufWriter, Write},
    path::{Path, PathBuf},
    str::FromStr,
};
use ws_core::{
    finder::{
        counter::FakeCounter,
        helpers::*,
        node::GridResult,
        orientation::{self, *},
    },
    prelude::*,
};

use hashbrown::{hash_set::HashSet as MySet, HashMap};

use crate::{
    clustering::cluster_words,
    combinations::{get_combinations, WordCombination},
};

//const BIT_SET_WORDS: usize = 2;

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

    /// If set, will reorient existing grids rather than finding new grids
    #[arg(short, long, default_value = "false")]
    pub reorient: bool,

    #[arg(long, default_value = "false")]
    pub remove_duplicates: bool,

    #[arg(long, default_value = "50")]
    pub max_clusters: u32,

    /// Whether to resume execution
    #[arg(long, default_value = "false")]
    pub resume: bool,
}

fn main() {
    let logger =
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).build();
    let multi = MultiProgress::new();

    LogWrapper::new(multi.clone(), logger).try_init().unwrap();

    let options = Options::parse();

    let mut did_something = false;

    if let Some(search) = &options.search {
        did_something = true;
        search::do_search(search);
    }

    if options.reorient {
        did_something = true;
        reorient_grids(&options);
    }

    if options.remove_duplicates {
        did_something = true;
        remove_duplicate_grids(&options);
    }

    if options.check_layout {
        did_something = true;
        word_layout::do_word_layout();
    }
    if options.cluster {
        did_something = true;
        cluster_files(&options);
    }

    if !did_something {
        do_finder(options);
    }

    info!("Finished... Press enter");
    io::stdin().read_line(&mut String::new()).unwrap();
}

fn remove_duplicate_grids(_options: &Options) {
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

        let grids = grid_file_text
            .lines()
            .map(|l| GridResult::from_str(l).unwrap())
            .collect_vec();

        let all_words = grids
            .iter()
            .flat_map(|grid| &grid.words)
            .cloned()
            .sorted()
            .dedup()
            .collect_vec();

        let sets: Vec<BitSet<4>> = grids
            .iter()
            .map(|grid_result| {
                let set = BitSet::<4>::from_iter(
                    grid_result
                        .words
                        .iter()
                        .map(|w| all_words.binary_search(w).unwrap()),
                );
                set
            })
            .sorted()
            .dedup()
            .collect();

        let new_grids: Vec<GridResult> = grids
            .iter()
            .filter(|grid_result| {
                let set = BitSet::<4>::from_iter(
                    grid_result
                        .words
                        .iter()
                        .map(|w| all_words.binary_search(w).unwrap()),
                );

                !sets.iter().any(|s| s.is_superset(&set) && *s != set)
            })
            .cloned()
            .collect();

        if new_grids.len() < grids.len() {
            info!(
                "Only {:5} of {:6} are optimal and unique",
                new_grids.len(),
                grids.len()
            );
            let new_contents = new_grids.into_iter().join("\n");

            match std::fs::write(grid_path, new_contents) {
                Ok(_) => {}
                Err(e) => log::error!("{e}"),
            }
        }
        pb.inc(1);
    }

    pb.finish();
}

fn reorient_grids(_options: &Options) {
    let folder = std::fs::read_dir("grids").unwrap();

    let paths: Vec<_> = folder.collect();

    let pb: ProgressBar = ProgressBar::new(paths.len() as u64)
        .with_style(ProgressStyle::with_template("{msg:50} {wide_bar} {pos:2}/{len:2}").unwrap())
        .with_message("Data files");

    for path in paths.iter() {
        let path = path.as_ref().unwrap().path();
        let file_name = path.file_name().unwrap().to_string_lossy();

        pb.set_message(file_name.to_string());

        let grid_file_text = std::fs::read_to_string(path.clone()).unwrap();

        let mut changed = 0;
        let mut errors: Vec<String> = Default::default();

        let grids = grid_file_text
            .lines()
            .map(|l| GridResult::from_str(l).unwrap())
            .map(|mut x| {
                let r = try_optimize_orientation(&mut x);

                match r {
                    Ok(true) => {
                        changed += 1;
                    }
                    Ok(false) => {}
                    Err(message) => errors.push(message),
                }
                x
            })
            .collect_vec();

        if errors.len() > 0 {
            warn!(
                "{:5} of {:6} Grids are impossible to orient safely",
                errors.len(),
                grids.len()
            );

            for line in errors {
                warn!("{line}")
            }
        }

        if changed > 0 {
            info!(
                "{changed:5} of {:6} Grids have suboptimal orientation",
                grids.len()
            );
            let new_contents = grids.into_iter().join("\n");

            match std::fs::write(path, new_contents) {
                Ok(_) => {}
                Err(e) => log::error!("{e}"),
            }
        }

        pb.inc(1);
    }

    pb.finish();
}

fn cluster_files(options: &Options) {
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
            .filter(|grid| {
                if let Some(taboo) = orientation::find_taboo_word(&grid.grid) {
                    warn!(
                        "Grid \n{} contains taboo word {} and will not be clustered",
                        grid.grid,
                        taboo.iter().join("")
                    );
                    false
                } else {
                    true
                }
            })
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

        let clusters = match all_words.len() {
            0..=64 => cluster_words::<1>(grids, &all_words, options.max_clusters as usize),
            65..=128 => cluster_words::<2>(grids, &all_words, options.max_clusters as usize),
            129..=192 => cluster_words::<3>(grids, &all_words, options.max_clusters as usize),
            193..=256 => cluster_words::<4>(grids, &all_words, options.max_clusters as usize),
            _ => panic!("Too many words to do clustering"),
        };

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
    stem: String
}

impl FinderCase {
    fn new_from_path(dir_entry: DirEntry) -> Self {
        let data_path = dir_entry.path();
        let stem = data_path.file_stem().unwrap().to_string_lossy().to_string();
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
            stem
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
            stem
        } = finder_case;

        info!("Found {} Words", word_map.len());
        for word in word_map.iter().map(|x| x.text).sorted() {
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
        if !master_words.is_empty() {
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
        }

        let resume_grids: Vec<GridResult> = if options.resume {
            let resume_file_text = std::fs::read_to_string(write_path).unwrap_or_default();
            let resume_grids: Vec<GridResult> = resume_file_text
                .lines()
                .map(|line| GridResult::from_str(line).unwrap())
                .collect();
            info!("Resuming with {} grids", resume_grids.len());
            resume_grids
        } else {
            vec![]
        };

        let grids_write_path = Path::new(write_path.as_str());
        let grids_file =
            std::fs::File::create(grids_write_path).expect("Could not find output folder");
        let grids_writer = BufWriter::new(grids_file);

        let max_grids = (options.grids > 0).then_some(options.grids as usize);

        let mut grids: Vec<GridResult> = match word_map.len() {
            0..=64 => create_grids::<1>(
                &stem,
                word_map,
                &master_words,
                grids_writer,
                options.minimum,
                max_grids,
                resume_grids,
            ),
            65..=128 => create_grids::<2>(
                &stem,
                word_map,
                &master_words,
                grids_writer,
                options.minimum,
                max_grids,
                resume_grids,
            ),
            129..=192 => create_grids::<3>(
                &stem,
                word_map,
                &master_words,
                grids_writer,
                options.minimum,
                max_grids,
                resume_grids,
            ),
            193..=256 => create_grids::<4>(
                &stem,
                word_map,
                &master_words,
                grids_writer,
                options.minimum,
                max_grids,
                resume_grids,
            ),
            _ => panic!("Too many words to do grid creation"),
        };

        grids.retain(|grid| {
            if let Some(taboo) = orientation::find_taboo_word(&grid.grid) {
                warn!(
                    "Grid \n{} contains taboo word {} and will not be clustered",
                    grid.grid,
                    taboo.iter().join("")
                );
                false
            } else {
                true
            }
        });

        let all_words = grids
            .iter()
            .flat_map(|grid| &grid.words)
            .cloned()
            .sorted()
            .dedup()
            .collect_vec();

        let clusters: Vec<clustering::Cluster> = match all_words.len() {
            0..=64 => cluster_words::<1>(grids, &all_words, options.max_clusters as usize),
            65..=128 => cluster_words::<2>(grids, &all_words, options.max_clusters as usize),
            129..=192 => cluster_words::<3>(grids, &all_words, options.max_clusters as usize),
            193..=256 => cluster_words::<4>(grids, &all_words, options.max_clusters as usize),
            _ => panic!("Too many words to do clustering"),
        };

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

#[derive(Debug, Default)]
struct SolutionGroup<const W: usize> {
    sets: Vec<BitSet<W>>,
    extras: MySet<BitSet<W>>,
}

fn create_grids<const W: usize>(category: &str,
    all_words: &Vec<FinderGroup>,
    exclude_words: &Vec<FinderSingleWord>,
    mut file: BufWriter<File>,
    min_size: u32,
    max_grids: Option<usize>,
    resume_grids: Vec<GridResult>,
) -> Vec<GridResult> {
    let word_letters: Vec<LetterCounts> = all_words.iter().map(|x| x.counts).collect();
    let mut possible_combinations: Vec<BitSet<W>> = get_combinations(Some(category.to_string()), word_letters.as_slice(), 16);

    info!(
        "{c} possible combinations founds",
        c = possible_combinations.len()
    );

    possible_combinations.sort_unstable_by_key(|x| std::cmp::Reverse(x.count()));

    let mut all_solutions: Vec<GridResult> = vec![];
    let mut all_solved_combinations: Vec<BitSet<W>> = vec![];

    let mut grouped_combinations: BTreeMap<u32, SolutionGroup<W>> = Default::default();

    possible_combinations
        .into_iter()
        .group_by(|x| x.count())
        .into_iter()
        .for_each(|(count, group)| {
            let sg = grouped_combinations.entry(count).or_default();
            sg.sets.extend(group);
        });

    let resume_grids: HashMap<BitSet<W>, GridResult> = resume_grids
        .into_iter()
        .map(|g| (g.get_word_bitset(&all_words), g))
        .collect();
    let min_resume_grid = resume_grids.keys().map(|x| x.count()).min();
    if min_resume_grid.is_some() {
        info!(
            "Resuming with {} grids of size {} or above",
            resume_grids.len(),
            min_resume_grid.unwrap_or_default()
        );

        // for (set, gr) in resume_grids.iter() {
        //     info!("{:?} - {}", set.into_iter().join(", "), gr.grid);
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
                    "{prefix} {msg} {elapsed:6} {human_pos:11}/{human_len:11} {wide_bar}",
                )
                .unwrap(),
            )
            .with_prefix(format!("{category}: {size:2} words"));

        let results: Vec<(WordCombination<W>, Option<GridResult>)> =
            if min_resume_grid.is_some_and(|mrg| mrg <= size) {
                pb.set_message(format!("resuming"));
                group
                    .sets
                    .into_par_iter()
                    .chain(group.extras.into_par_iter())
                    .filter(|x| !all_solved_combinations.iter().any(|sol| sol.is_superset(x)))
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
                pb.set_message(format!("finding"));
                group
                    .sets
                    .into_par_iter()
                    .chain(group.extras.into_par_iter())
                    .filter(|x| !all_solved_combinations.iter().any(|sol| sol.is_superset(x)))
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

        pb.set_message(format!("finishing"));

        let results_count = results.len();

        let mut solutions: Vec<GridResult> = vec![];
        let next_group = grouped_combinations
            .entry(size.saturating_sub(1))
            .or_default();

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
        }

        if !solutions.is_empty() {
            let lines = solutions
                .iter()
                .sorted_by_cached_key(|x| x.words.iter().sorted().join(""))
                .join("\n");
            file.write_all((lines + "\n").as_bytes()).unwrap();
            file.flush().expect("Could not flush to file");
        }

        let solution_count = solutions.len();
        
        pb.finish_with_message(format!(
            "{solution_count:6} Solutions"
        ));

        all_solutions.extend(solutions);

        if max_grids.is_some_and(|mg| mg < all_solutions.len()) {
            return all_solutions;
        }
    }

    panic!();
    all_solutions
}
