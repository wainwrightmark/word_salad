use bevy::utils::HashSet;
use bevy_utils::{CanInitTrackedResource, TrackableResource};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};

use crate::prelude::*;

pub struct StatePlugin;

impl Plugin for StatePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ChosenState>();
        app.init_tracked_resource::<CurrentLevel>();
        app.init_tracked_resource::<FoundWordsState>();

        app.add_systems(Update, track_found_words);
        // app.add_systems(Update, track_level_change);
    }
}

#[derive(Debug, Clone, Resource, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct ChosenState(pub Solution);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DesignedLevel {
    pub name: String,
    pub grid: Grid,
    pub words: Vec<Word>,
    pub level_index: usize,
    pub words_set: HashSet<CharsArray>,
}

#[derive(Debug, Clone, Resource, PartialEq, Eq, Serialize, Deserialize)]
pub struct CurrentLevel {
    pub level_index: usize, //todo more sophisticated pointer
}

impl TrackableResource for CurrentLevel {
    const KEY: &'static str = "CurrentLevel";
}

#[derive(Debug, Clone, Resource, Default, Serialize, Deserialize)]
pub struct FoundWordsState {
    pub found: HashSet<CharsArray>,
}

impl TrackableResource for FoundWordsState {
    const KEY: &'static str = "FoundWOrds";
}

impl Default for CurrentLevel {
    fn default() -> Self {
        Self { level_index: 0 }
    }
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

#[derive(Debug, Default)]
pub struct InputState {
    last_tile: Option<Tile>,
    delete_on_end: bool,
}

impl InputState {
    pub fn handle_input_start(
        &mut self,
        chosen_state: &mut ResMut<ChosenState>,
        location: Tile,
        grid: &Grid,
    ) {
        if self.last_tile == Some(location) {
            self.delete_on_end = true;
            return;
        }
        self.delete_on_end = false;
        self.last_tile = Some(location);

        if let Some(last) = chosen_state.0.last() {
            if let Some(index) = chosen_state.0.iter().position(|x| *x == location) {
                // element is already present
                if index + 1 == chosen_state.0.len() {
                    self.delete_on_end = true;
                    //chosen_state.0.clear(); do nothing
                } else {
                    chosen_state.0.truncate(index + 1);
                }
            } else if last.is_adjacent_to(&location) {
                //element is not already present
                if !grid[location].is_blank() {
                    chosen_state.0.push(location);
                }
            }
        } else {
            //array is empty
            if !grid[location].is_blank() {
                chosen_state.0.push(location);
            }
        }
    }

    pub fn handle_input_move(&mut self, chosen_state: &mut ResMut<ChosenState>, location: Tile, grid: &Grid,) {
        if self.last_tile == Some(location) {
            return;
        }
        self.delete_on_end = false;
        self.last_tile = Some(location);

        if let Some(last) = chosen_state.0.last() {
            if let Some(index) = chosen_state.0.iter().position(|x| *x == location) {
                // element is already present
                if index + 1 == chosen_state.0.len() {
                    //chosen_state.0.clear(); do nothing
                } else {
                    chosen_state.0.truncate(index + 1);
                }
            } else if last.is_adjacent_to(&location) {
                //element is not already present
                if !grid[location].is_blank() {
                    chosen_state.0.push(location);
                }
            }
        }
    }

    pub fn handle_input_end(&mut self, chosen_state: &mut ResMut<ChosenState>, location: Tile) {
        if self.delete_on_end && self.last_tile == Some(location) {
            chosen_state.0.clear();
        }
        self.last_tile = None;
        self.delete_on_end = false;
    }

    pub fn handle_input_end_no_location(&mut self) {
        self.last_tile = None;
        self.delete_on_end = false;
    }
}

fn track_found_words(
    chosen: Res<ChosenState>,
    level: Res<CurrentLevel>,
    mut found_words: ResMut<FoundWordsState>,
) {
    if chosen.is_changed() {
        let grid = level.level().grid;
        let chars: CharsArray = chosen.0.iter().map(|t| grid[*t]).collect();

        if level.level().words_set.contains(&chars) {
            if !found_words.found.contains(&chars) {
                found_words.found.insert(chars);
            }
        }
    }
}
