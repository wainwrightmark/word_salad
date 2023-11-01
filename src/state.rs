use lazy_static::lazy_static;
use std::collections::BTreeMap;

use crate::prelude::*;

pub struct StatePlugin;

impl Plugin for StatePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ChosenState>();
        app.init_resource::<CurrentLevel>();
        app.init_resource::<FoundWordsState>();

        app.add_systems(Update, track_found_words);
        app.add_systems(Update, track_level_change);
    }
}

#[derive(Debug, Clone, Resource, PartialEq, Eq, Default)]
pub struct ChosenState(pub Solution);

#[derive(Debug, Clone, Resource, PartialEq, Eq)]
pub struct CurrentLevel {
    pub grid: Grid,
    pub words: Vec<Word>,
    pub level_index: usize,
}

#[derive(Debug, Clone, Resource, Default)]
pub struct FoundWordsState {
    pub found: BTreeMap<CharsArray, bool>,
}

impl Default for CurrentLevel {
    fn default() -> Self {
        Self::get_level(0)
    }
}

const LEVEL_LINES: &'static str = include_str!("levels.tsv");
lazy_static! {
    static ref LEVELS: Vec<CurrentLevel> = {
        let lines = LEVEL_LINES.lines();

        let r: Vec<CurrentLevel> = lines
            .enumerate()
            .map(|(index, line)| CurrentLevel::from_tsv_line(line, index))
            .collect();
        r
    };
}

impl CurrentLevel {
    pub fn get_level(level: usize) -> Self {
        let index = level % LEVELS.len();
        let level: CurrentLevel = LEVELS[index].clone();
        level
    }

    fn from_tsv_line(line: &'static str, level_index: usize) -> Self {
        let mut iter = line.split('\t');

        let _name: &'static str = iter.next().expect("Level should have name");
        let chars: &'static str = iter.next().expect("Level should have a grid");

        let grid = try_make_grid(chars).expect("Should be able to make grid");

        let words = iter.next().expect("Level should have words");

        let mut words: Vec<Word> = words
            .split_ascii_whitespace()
            .map(|s| Word::from_static_str(s).expect("Could not convert string to word"))
            .collect();

        words.sort_by(|a,b|a.characters.cmp(&b.characters));

        Self {
            grid,
            words,
            level_index,
        }
    }
}

impl ChosenState {
    pub fn on_click(&mut self, location: Tile, level: &CurrentLevel) {
        //todo better interactions

        if let Some(last) = self.0.last() {
            if let Some(index) = self.0.iter().position(|x| *x == location) {
                // element is already present
                if index + 1 == self.0.len() {
                    self.0.clear();
                } else {
                    self.0.truncate(index + 1);
                }
            } else if last.is_adjacent_to(&location) {
                //element is not already present
                self.0.push(location);
            }
        } else {
            //array is empty
            self.0.push(location);
        }
    }
}

fn track_level_change(
    level: Res<CurrentLevel>,
    mut chosen: ResMut<ChosenState>,
    mut found_words: ResMut<FoundWordsState>,
) {
    if level.is_changed() {
        chosen.0.clear();
        found_words.found = level
            .words
            .iter()
            .map(|w| (w.characters.clone(), false))
            .collect();
    }
}

fn track_found_words(
    chosen: Res<ChosenState>,
    level: Res<CurrentLevel>,
    mut found_words: ResMut<FoundWordsState>,
) {
    if chosen.is_changed() {
        let chars: CharsArray = chosen.0.iter().map(|t| level.grid[*t]).collect();

        if found_words.found.get(&chars) == Some(&false) {
            found_words.found.insert(chars, true);
        }
    }
}
