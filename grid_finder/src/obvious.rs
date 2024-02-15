use std::str::FromStr;

use base64::Engine;
use hashbrown::hash_set;
use indicatif::{ProgressBar, ProgressStyle};
use itertools::Itertools;
use log::info;
use ws_core::{ArrayVec, Character, DesignedLevel, Word};

pub fn do_obvious(search: &str) {
    let folder = std::fs::read_dir("grids").unwrap();

    let words: Vec<_> = search
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

    for path in paths.iter() {
        let grids_path = path.as_ref().unwrap().path();
        let grid_file_text = std::fs::read_to_string(grids_path.clone()).unwrap();

        for line in grid_file_text.lines() {
            let level = DesignedLevel::from_tsv_line(line).unwrap();

            if !level
                .words
                .iter()
                .any(|w| words_characters.contains(&w.characters))
            {
                continue;
            }
            //What we maybe want is grids where two words are very obvious...

            for word in level
                .words
                .iter()
                .filter(|w| words_characters.contains(&w.characters))
            {
                for solution in word.find_solutions(&level.grid) {
                    let mut left = 0usize;
                    let mut right = 0usize;
                    let mut up = 0usize;
                    let mut down = 0usize;
                    for (a, b) in solution.iter().tuple_windows() {
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

                    let min = [left, right, up, down].into_iter().min();

                    if min == Some(0) {
                        if left.min(right) <= 1 && up.min(down) <= 1 {
                            total_solutions += 1;
                            let base64_level =
                                base64::engine::general_purpose::URL_SAFE.encode(level.to_string());
                            info!(
                                "Found {word} in {file} {left} {right} {up} {down} {base64_level} ",
                                file = grids_path
                                    .file_name()
                                    .map(|x| x.to_string_lossy())
                                    .unwrap_or_default(),
                                word = word.text
                            );
                            continue;
                        }
                    }
                }
            }
        }

        pb.inc(1);
    }

    info!("{total_solutions} solutions found")
}
