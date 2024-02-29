pub mod clustering;
pub mod combinations;
pub mod grid_creator;
pub mod obvious;
pub mod search;
pub mod word_layout;
pub mod word_set;

use clap::{Args, Parser, Subcommand};
use const_sized_bit_set::BitSet;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use indicatif_log_bridge::LogWrapper;
use itertools::Itertools;
use log::{info, warn};
use obvious::ObviousArgs;
use std::{
    collections::HashSet,
    fs::DirEntry,
    io::{self, BufWriter},
    path::{Path, PathBuf},
    str::FromStr,
};
use ws_core::finder::{
    cluster::Cluster, falling_probability, helpers::*, node::GridResult, orientation::{self, *}
};

use crate::clustering::cluster_words;

#[derive(Parser, Debug, Default)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}
#[derive(Subcommand, Debug)]

pub enum Commands {
    FindGrids(FindGridsArgs),

    /// search all found grids for ones where ones in this list are written in an obvious way
    Obvious(ObviousArgs),
    /// find clusters for all existing grids rather than finding new grids
    Cluster(ClusterArgs),
    /// search all found grids for a particular word or list of words
    Search {
        words: String,
    },
    /// Check word layouts
    CheckLayout {},
    /// reorient existing grids rather than finding new grids
    Reorient {},

    /// Remove duplicate grids
    RemoveDuplicates {},
}

#[derive(Args, Debug)]
pub struct ClusterArgs {
    /// Folder to look for data in
    #[arg(short, long, default_value = "data")]
    pub folder: String,

    #[arg(long, default_value = "50")]
    pub max_clusters: u32,

    /// Minimum number of words in a grid
    #[arg(short, long, default_value = "5")]
    pub minimum_words: u32,

    /// Maximum number of grids to return
    #[arg(short, long, default_value = "0")]
    pub grids: u32,

    /// All grids should have at least this cumulative falling probability
    #[arg( long, default_value = "0.0")]
    pub min_falling: f32
}

#[derive(Args, Debug)]
pub struct FindGridsArgs {
    /// Folder to look for data in
    #[arg(short, long, default_value = "data")]
    pub folder: String,

    /// Maximum number of grids to return
    #[arg(short, long, default_value = "0")]
    grids: u32,

    /// Minimum number of words in a grid
    #[arg(short, long, default_value = "5")]
    pub minimum: u32,

    #[arg(long, default_value = "50")]
    pub max_clusters: u32,

    /// Whether to resume execution
    #[arg(long, default_value = "false")]
    pub resume: bool,
}

impl Default for FindGridsArgs {
    fn default() -> Self {
        Self {
            folder: "data".to_string(),
            grids: 0,
            minimum: 5,
            max_clusters: 50,
            resume: false,
        }
    }
}

fn main() {
    let logger =
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).build();
    let multi = MultiProgress::new();

    LogWrapper::new(multi.clone(), logger).try_init().unwrap();

    let options = Cli::parse();

    match options.command {
        Some(Commands::Obvious(args)) => {
            obvious::do_obvious(args);
        }
        Some(Commands::Cluster(cluster_args)) => {
            cluster_files(&cluster_args);
        }
        Some(Commands::Search { words }) => search::do_search(&words),
        Some(Commands::CheckLayout {}) => word_layout::do_word_layout(),
        Some(Commands::Reorient {}) => reorient_grids(),
        Some(Commands::FindGrids(find_args)) => {
            do_finder(find_args);
        }
        Some(Commands::RemoveDuplicates {}) => {
            remove_duplicate_grids(&options);
        }
        None => do_finder(FindGridsArgs::default()),
    }

    info!("Finished... Press enter");
    io::stdin().read_line(&mut String::new()).unwrap();
}

fn remove_duplicate_grids(_options: &Cli) {
    let folder = std::fs::read_dir("grids").unwrap();
    let paths: Vec<_> = folder.collect();

    let pb: ProgressBar = ProgressBar::new(paths.len() as u64)
        .with_style(ProgressStyle::with_template("{msg:50} {bar} {pos:2}/{len:2}").unwrap())
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

fn reorient_grids() {
    let folder = std::fs::read_dir("grids").unwrap();

    let paths: Vec<_> = folder.collect();

    let pb: ProgressBar = ProgressBar::new(paths.len() as u64)
        .with_style(ProgressStyle::with_template("{msg:50} {bar} {pos:2}/{len:2}").unwrap())
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

fn cluster_files(options: &ClusterArgs) {
    let folder = std::fs::read_dir("grids").unwrap();

    let paths: Vec<_> = folder.collect();

    // let pb: ProgressBar = ProgressBar::new(paths.len() as u64)
    //     .with_style(ProgressStyle::with_template("{msg:50} {bar} {pos:2}/{len:2}").unwrap())
    //     .with_message("Data files");

    let _ = std::fs::create_dir("clusters");

    for path in paths {
        let grid_path = path.as_ref().unwrap().path();
        let file_name = grid_path.file_name().unwrap().to_string_lossy();

        let category = grid_path.file_stem().unwrap().to_string_lossy();
        // pb.set_message(file_name.to_string());

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
        let mut taboo_grids = 0;

        let grids = grid_file_text
            .lines()
            .map(|l| GridResult::from_str(l).unwrap())
            .filter(|x| x.words.len() >= options.minimum_words as usize)
            .filter(|x| options.min_falling == 0.0 || falling_probability::calculate_cumulative_falling_probability_2(x) >= options.min_falling)
            .filter(|grid| {
                if let Some(..) = orientation::find_taboo_word(&grid.grid) {
                    taboo_grids += 1;
                    false
                } else {
                    true
                }
            })
            .filter(|x| {
                if options.grids == 0 {
                    true
                } else {
                    filter_enough_grids(x, &mut count, options.grids as usize, &mut filter_below)
                }
            })
            .collect_vec();

        let all_words = grids
            .iter()
            .flat_map(|grid| &grid.words)
            .cloned()
            .sorted()
            .dedup()
            .collect_vec();

        // info!(
        //     "{file_name} found {:6} grids with {:3} different words",
        //     grids.len(),
        //     all_words.len()
        // );

        if taboo_grids > 0 {
            warn!("{taboo_grids} Grids contain taboo words and will not be clustered in category {category}",);
        }

        let clusters = match all_words.len() {
            0..=64 => cluster_words::<1>(
                grids,
                &all_words,
                options.max_clusters as usize,
                Some(category.to_string()),
            ),
            65..=128 => cluster_words::<2>(
                grids,
                &all_words,
                options.max_clusters as usize,
                Some(category.to_string()),
            ),
            129..=192 => cluster_words::<3>(
                grids,
                &all_words,
                options.max_clusters as usize,
                Some(category.to_string()),
            ),
            193..=256 => cluster_words::<4>(
                grids,
                &all_words,
                options.max_clusters as usize,
                Some(category.to_string()),
            ),
            _ => panic!("Too many words to do clustering"),
        };

        let clusters_write_path = format!("clusters/{file_name}",);
        let clusters_write_path = Path::new(clusters_write_path.as_str());
        let clusters_text = clusters.into_iter().map(|x| x.to_string()).join("\n\n");
        std::fs::write(clusters_write_path, clusters_text).unwrap();

        //pb.inc(1);
    }

    //pb.finish();
}

struct FinderCase {
    word_map: Vec<FinderGroup>,
    data_path: PathBuf,
    data_file_text: String,
    file_name: String,
    write_path: String,
    stem: String,
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
            stem,
        }
    }
}

fn do_finder(options: FindGridsArgs) {
    info!("Starting up");

    let folder = std::fs::read_dir(options.folder).unwrap();

    let paths: Vec<_> = folder
        .into_iter()
        .map(|path| FinderCase::new_from_path(path.unwrap()))
        .sorted_by_key(|x| x.word_map.len())
        .collect();

    let pb: ProgressBar = ProgressBar::new(paths.len() as u64)
        .with_style(ProgressStyle::with_template("{msg} {bar} {pos:2}/{len:2}").unwrap())
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
            stem,
        } = finder_case;

        info!("Found {} Words", word_map.len());
        info!("{}", word_map.iter().map(|x| x.text).sorted().join(", "));

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
            0..=64 => grid_creator::create_grids::<1>(
                &stem,
                word_map,
                &master_words,
                grids_writer,
                options.minimum,
                max_grids,
                resume_grids,
            ),
            65..=128 => grid_creator::create_grids::<2>(
                &stem,
                word_map,
                &master_words,
                grids_writer,
                options.minimum,
                max_grids,
                resume_grids,
            ),
            129..=192 => grid_creator::create_grids::<3>(
                &stem,
                word_map,
                &master_words,
                grids_writer,
                options.minimum,
                max_grids,
                resume_grids,
            ),
            193..=256 => grid_creator::create_grids::<4>(
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

        let mut taboo_grids = 0;

        grids.retain(|grid| {
            if let Some(..) = orientation::find_taboo_word(&grid.grid) {
                taboo_grids += 1;

                false
            } else {
                true
            }
        });

        if taboo_grids > 0 {
            warn!("{taboo_grids} Grids contain taboo words and will not be clustered in category {stem}",);
        }

        let all_words = grids
            .iter()
            .flat_map(|grid| &grid.words)
            .cloned()
            .sorted()
            .dedup()
            .collect_vec();

        let clusters: Vec<Cluster> = match all_words.len() {
            0..=64 => cluster_words::<1>(
                grids,
                &all_words,
                options.max_clusters as usize,
                Some(stem.clone()),
            ),
            65..=128 => cluster_words::<2>(
                grids,
                &all_words,
                options.max_clusters as usize,
                Some(stem.clone()),
            ),
            129..=192 => cluster_words::<3>(
                grids,
                &all_words,
                options.max_clusters as usize,
                Some(stem.clone()),
            ),
            193..=256 => cluster_words::<4>(
                grids,
                &all_words,
                options.max_clusters as usize,
                Some(stem.clone()),
            ),
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
