use bevy::utils::HashSet;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use crate::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DesignedLevel {
    pub name: String,
    pub grid: Grid,
    pub words: Vec<Word>,
    pub level_index: usize,
    pub words_set: HashSet<CharsArray>,
}

const LEVEL_LINES: &'static str = include_str!("levels.tsv");
lazy_static! {
    static ref LEVELS: Vec<DesignedLevel> = {
        let lines = LEVEL_LINES.lines();

        let r: Vec<DesignedLevel> = lines
            .enumerate()
            .map(|(index, line)| DesignedLevel::from_tsv_line(line, index))
            .collect();
        r
    };
}


impl DesignedLevel {
    fn from_tsv_line(line: &'static str, level_index: usize) -> Self {
        let mut iter = line.split('\t');

        let name: &'static str = iter.next().expect("Level should have name");
        let chars: &'static str = iter.next().expect("Level should have a grid");

        let grid = try_make_grid(chars).expect("Should be able to make grid");

        let words = iter.next().expect("Level should have words");

        let mut words: Vec<Word> = words
            .split_ascii_whitespace()
            .map(|s| Word::from_static_str(s).expect("Could not convert string to word"))
            .collect();

        words.sort_by(|a, b| a.characters.cmp(&b.characters));

        let words_set: bevy::utils::hashbrown::HashSet<CharsArray> =
            words.iter().map(|x| x.characters.clone()).collect();

        Self {
            name: name.to_string(),
            grid,
            words,
            level_index,
            words_set,
        }
    }
}

impl CurrentLevel {
    // pub fn get_level(level: usize) -> Self {
    //     let index = level % LEVELS.len();
    //     let level: CurrentLevel = LEVELS[index].clone();
    //     level
    // }

    pub fn level(&self) -> &DesignedLevel {
        let index = self.level_index % LEVELS.len();
        &LEVELS[index]
    }
}