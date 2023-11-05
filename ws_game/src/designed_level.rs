use std::collections::BTreeMap;

use bevy::utils::{HashMap, HashSet};
use lazy_static::lazy_static;

use crate::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesignedLevel {
    pub name: String,
    pub grid: Grid,

    pub level_index: usize,

    pub words_map: BTreeMap<CharsArray, Word>,
    //pub words_set: HashSet<CharsArray>,
    pub tiles_used: HashMap<CharsArray, GridSet>,
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

        let words_map: BTreeMap<CharsArray, Word> = words
            .split_ascii_whitespace()
            .map(|s| Word::from_str(s).expect("Could not convert string to word"))
            .map(|w| (w.characters.clone(), w))
            .collect();

        let mut tiles_used: HashMap<CharsArray, GridSet> = Default::default();

        for word in words_map.values() {
            let solutions = word.find_solutions(&grid);
            let mut set = GridSet::default();
            for solution in solutions.iter() {
                for tile in solution {
                    set.set_bit(tile, true);
                }
            }
            tiles_used.insert(word.characters.clone(), set);
        }

        Self {
            name: name.to_string(),
            grid,
            level_index,
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

#[cfg(test)]
pub mod tests {
    use crate::prelude::*;
    use ws_core::Tile;

    #[test]
    pub fn test_calculate_needed_tiles() {
        let level = DesignedLevel::from_tsv_line(
            // spellchecker:disable-next-line
            "Sports	GNDTEUIOKILOASHP	POLO SHOOTING KENDO SAILING LUGE SKIING",
            0,
        );

        let mut found_words: bevy::utils::hashbrown::HashSet<CharsArray> = Default::default();

        let mut expected = GridSet::EMPTY;

        assert_eq!(level.calculate_unneeded_tiles(&found_words), expected);

        found_words.insert(Word::from_str("kendo").unwrap().characters);
        expected.set_bit(&Tile::new_const::<2, 0>(), true);
        let actual: geometrid::tile_set::TileSet16<4, 4, 16> =
            level.calculate_unneeded_tiles(&found_words);
        // for (tile, set) in actual.enumerate(){
        //     println!("{tile} {set}");
        // }
        assert_eq!(actual, expected);
    }
}
