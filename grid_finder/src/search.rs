use std::str::FromStr;

use indicatif::{ProgressBar, ProgressStyle};
use log::info;
use ws_core::{Word, DesignedLevel};

pub fn do_search(search: String){
    let folder = std::fs::read_dir("grids").unwrap();

    let words: Vec<_> = search.split_ascii_whitespace().map(|s| Word::from_str(s).unwrap())
    .map(|w| {let lc = w.letter_counts().unwrap(); (w,lc)})
    .collect();




    let paths: Vec<_> = folder.collect();

    let pb: ProgressBar = ProgressBar::new(paths.len() as u64)
        .with_style(ProgressStyle::with_template("{msg} {wide_bar} {pos:2}/{len:2}").unwrap())
        .with_message("Grid files");

    let mut total_solutions = 0usize;
    let mut total_possibles = 0usize;

    for path in paths.iter() {
        let grids_path = path.as_ref().unwrap().path();
        let grid_file_text = std::fs::read_to_string(grids_path.clone()).unwrap();

        for line in grid_file_text.lines(){
            let level = DesignedLevel::from_tsv_line(line);

            let level_letter_counts = level.letter_counts().expect("Could not get grid letter counts");

            for (word, _) in words.iter().filter(|(_, lc)|level_letter_counts.is_superset(lc)){
                total_possibles += 1;
                if let Some(..) = word.find_solution(&level.grid){
                    total_solutions += 1;
                    info!("Found {word} in {file} {level} ", file= grids_path.file_name().map(|x|x.to_string_lossy()).unwrap_or_default(), word = word.text)
                }
            }


        }

        pb.inc(1);
    }

    info!("{total_solutions} solutions found ({total_possibles} possibles)")
}