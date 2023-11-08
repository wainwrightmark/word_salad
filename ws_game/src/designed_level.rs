use std::collections::BTreeMap;

use bevy::utils::{HashMap, HashSet};
use bevy_utils::TrackableResource;
use itertools::Itertools;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};

use crate::prelude::*;

#[derive(Debug, Clone, Resource, PartialEq, Eq, Serialize, Deserialize)]
pub enum CurrentLevel {
    Fixed { level_index: usize },
    Custom(DesignedLevel), //pub level_index: usize, //todo more sophisticated pointer
}

impl TrackableResource for CurrentLevel {
    const KEY: &'static str = "CurrentLevel";
}

impl Default for CurrentLevel {
    fn default() -> Self {
        Self::Fixed { level_index: 0 }
    }
}

impl CurrentLevel {
    pub fn level(&self) -> &DesignedLevel {
        match self {
            CurrentLevel::Fixed { level_index } => {
                let index = level_index % LEVELS.len();
                &LEVELS[index]
            }
            CurrentLevel::Custom(level) => level,
        }
    }
}

pub fn update_lazy_level_data(level: Res<CurrentLevel>, mut data: ResMut<LazyLevelData>){
    if level.is_changed(){
        *data = LazyLevelData::new(level.level());
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Resource)]
pub struct LazyLevelData {
    pub words_map: BTreeMap<CharsArray, Word>,
    pub tiles_used: HashMap<CharsArray, GridSet>,
}

impl LazyLevelData {

    pub fn new_empty()->Self{
        Self { words_map: Default::default(), tiles_used: Default::default() }
    }

    pub fn new(level: &DesignedLevel) -> Self {
        let words_map: BTreeMap<CharsArray, Word> = level
            .words
            .iter()
            .map(|x| (x.characters.clone(), x.clone()))
            .collect();

        let mut tiles_used: HashMap<CharsArray, GridSet> = Default::default();

        for word in words_map.values() {
            let solutions = word.find_solutions(&level.grid);
            let mut set = GridSet::default();
            for solution in solutions.iter() {
                for tile in solution {
                    set.set_bit(tile, true);
                }
            }
            tiles_used.insert(word.characters.clone(), set);
        }

        Self {
            words_map,
            tiles_used,
        }
    }

    pub fn calculate_unneeded_tiles(&self, found_words: &HashSet<CharsArray>) -> GridSet {
        let mut result: GridSet = Default::default();

        for (word, used) in self.tiles_used.iter() {
            if found_words.contains(word) {
                continue;
            }; //this word has been found so its words aren't needed

            result = result.union(used);
        }

        result.negate()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DesignedLevel {
    pub name: String,
    pub grid: Grid,
    pub words: Vec<Word>,
}

const LEVEL_LINES: &'static str = include_str!("levels.tsv");
lazy_static! {
    static ref LEVELS: Vec<DesignedLevel> = {
        let lines = LEVEL_LINES.lines();

        let r: Vec<DesignedLevel> = lines
            .map(|line| DesignedLevel::from_tsv_line(line))
            .collect();
        r
    };
}

impl DesignedLevel {
    fn from_tsv_line(line: &str) -> Self {
        let mut iter = line.split('\t');

        let chars: &str = iter.next().expect("Level should have a grid");
        let name: &str = iter.next().expect("Level should have name");

        let grid = try_make_grid(chars).expect("Should be able to make grid");

        let words = iter
            .map(|x| x.trim().to_string())
            .flat_map(|x| Word::from_str(x.as_str()).ok())
            .sorted_by_cached_key(|x| x.text.to_ascii_lowercase())
            .collect();

        Self {
            name: name.to_string(),
            grid,
            words,
        }
    }
}

impl DesignedLevel {
    pub fn try_from_path(path: String) -> Option<Self> {
        info!("path: {path}");

        if path.is_empty() || path.eq_ignore_ascii_case("/") {
            return None;
        }

        if path.to_ascii_lowercase().starts_with("/game") {
            info!("Path starts with game");
            let data = path[6..].to_string();
            info!("{data}");

            use base64::Engine;

            let data = base64::engine::general_purpose::URL_SAFE
                .decode(data)
                .ok()?;

            let data = String::from_utf8(data).ok()?;
            info!("{data}");

            let level = DesignedLevel::from_tsv_line(&data);

            Some(level)
        } else {
            None
        }
    }
}

#[cfg(test)]
pub mod tests {
    use crate::prelude::*;
    use ws_core::Tile;

    #[test]
    pub fn test_calculate_needed_tiles() {
        let level = DesignedLevel::from_tsv_line(
            // spellchecker:disable-next-line
            "GNDTEUIOKILOASHP   Sports  POLO    SHOOTING    KENDO   SAILING LUGE    SKIING",
        );

        let lazy_data = LazyLevelData::new(&level);

        let mut found_words: bevy::utils::hashbrown::HashSet<CharsArray> = Default::default();

        let mut expected = GridSet::EMPTY;

        assert_eq!(lazy_data.calculate_unneeded_tiles(&found_words), expected);

        found_words.insert(Word::from_str("kendo").unwrap().characters);
        expected.set_bit(&Tile::new_const::<2, 0>(), true);
        let actual: geometrid::tile_set::TileSet16<4, 4, 16> =
            lazy_data.calculate_unneeded_tiles(&found_words);
        // for (tile, set) in actual.enumerate(){
        //     println!("{tile} {set}");
        // }
        assert_eq!(actual, expected);
    }
}
