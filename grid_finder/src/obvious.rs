use std::{ops::Neg, str::FromStr};

use base64::Engine;
use clap::Args;
use hashbrown::hash_set;
use indicatif::{ProgressBar, ProgressStyle};
use itertools::Itertools;
use log::info;
use ws_core::{ArrayVec, Character, DesignedLevel, DynamicTile, Vector, Word};

#[derive(Args, Debug)]
pub struct ObviousArgs {
    #[arg(short, long)]
    words: String,
    #[arg(short, long, default_value = "false")]
    corner: bool,

    /// minimum number of zeros required
    #[arg(short, long, default_value = "1")]
    zeros: usize,

    /// Max number of direction changes
    #[arg(short, long, default_value = "3")]
    direction_changes: usize,
}

pub fn do_obvious(args: ObviousArgs) {
    let folder = std::fs::read_dir("grids").unwrap();

    let words: Vec<_> = args
        .words
        .split_ascii_whitespace()
        .map(|s| Word::from_str(s).unwrap())
        .collect();

    let words_characters: hash_set::HashSet<ArrayVec<Character, 16>> =
        words.iter().map(|x| &x.characters).cloned().collect();

    let paths: Vec<_> = folder.collect();

    let pb: ProgressBar = ProgressBar::new(paths.len() as u64)
        .with_style(ProgressStyle::with_template("{msg} {bar} {pos:2}/{len:2}").unwrap())
        .with_message("Grid files");

    let mut total_solutions = 0usize;
    let mut possible_solutions = 0usize;

    for path in paths.iter() {
        let grids_path = path.as_ref().unwrap().path();
        let grid_file_text = std::fs::read_to_string(grids_path.clone()).unwrap();

        for line in grid_file_text.lines() {
            let level = DesignedLevel::from_tsv_line(line).unwrap();

            if !words_characters
                .iter()
                .all(|wc| level.words.iter().any(|w| wc.eq(&w.characters)))
            {
                continue;
            }

            // if !level
            //     .words
            //     .iter()
            //     .all(|w| words_characters.contains(&w.characters))
            // {
            //     continue;
            // }
            //What we maybe want is grids where two words are very obvious...

            for word in level
                .words
                .iter()
                .filter(|w| words_characters.contains(&w.characters))
            {
                for solution in word.find_solutions(&level.grid) {
                    let first = solution.first().unwrap();
                    if args.corner && !first.is_corner() {
                        continue;
                    }

                    let mut left = 0usize;
                    let mut right = 0usize;
                    let mut up = 0usize;
                    let mut down = 0usize;
                    let mut direction_changes = 0usize;
                    let mut last_direction = None;

                    for (a, b) in solution.iter().tuple_windows() {
                        let av = Vector::new(a.x() as i8, a.y() as i8);
                        let bv = Vector::new(b.x() as i8, b.y() as i8);

                        let direction = bv.const_add(&av.neg());
                        match last_direction {
                            None => {}
                            Some(d) => {
                                if d != direction {
                                    direction_changes += 1;
                                }
                            }
                        }

                        last_direction = Some(direction);

                        match a.x().cmp(&b.x()) {
                            std::cmp::Ordering::Less => left += 1,
                            std::cmp::Ordering::Equal => {}
                            std::cmp::Ordering::Greater => right += 1,
                        }
                        match a.y().cmp(&b.y()) {
                            std::cmp::Ordering::Less => down += 1,
                            std::cmp::Ordering::Equal => {}
                            std::cmp::Ordering::Greater => up += 1,
                        }
                    }

                    let zeros = [left, right, up, down]
                        .into_iter()
                        .filter(|x| *x == 0)
                        .count();

                    if zeros >= args.zeros && direction_changes <= args.direction_changes {
                        total_solutions += 1;
                        let base64_level =
                            base64::engine::general_purpose::URL_SAFE.encode(level.to_string());
                        info!(
                                "Found {word} in {file}: {first} {left} {right} {up} {down} {grid} {base64_level} ",
                                file = grids_path
                                    .file_name()
                                    .map(|x| x.to_string_lossy())
                                    .unwrap_or_default(),
                                word = word.text,
                                grid= level.grid.iter().join("|")
                            );
                        continue;
                    }
                }
            }
        }

        pb.inc(1);
    }

    info!("{total_solutions} solutions found")
}
