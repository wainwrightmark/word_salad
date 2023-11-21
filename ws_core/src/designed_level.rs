use std::str::FromStr;

use itertools::Itertools;
use serde::{Deserialize, Serialize};
use crate::{finder::helpers::LetterCounts, Grid, prelude::*};


#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DesignedLevel {
    pub name: String,
    pub grid: Grid,
    pub words: Vec<DisplayWord>,
}

impl std::fmt::Display for DesignedLevel{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{grid}\t{name}\t{words}",

    grid = self.grid.iter().join(""), name=self.name, words= self.words.iter().join("\t"))
    }
}

impl DesignedLevel {

    pub fn letter_counts(&self)-> Option<LetterCounts>{
        LetterCounts::try_from_iter(self.grid.iter(). cloned())
    }

    pub fn from_tsv_line(line: &str) -> Self {
        let mut iter = line.split('\t');

        let chars: &str = iter.next().expect("Level should have a grid");
        let name: &str = iter.next().expect("Level should have name");

        let grid = try_make_grid(chars)
            //.map(|x| x.with_flip(FlipAxes::Vertical))
            .expect("Should be able to make grid");

        let words = iter
            .map(|x| x.trim().to_string())
            .flat_map(|x| DisplayWord::from_str(x.as_str()).ok())
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
        //info!("path: {path}");

        if path.is_empty() || path.eq_ignore_ascii_case("/") {
            return None;
        }

        if path.to_ascii_lowercase().starts_with("/game") {
            //info!("Path starts with game");
            let data = path[6..].to_string();
            //info!("{data}");

            use base64::Engine;

            let data = base64::engine::general_purpose::URL_SAFE
                .decode(data)
                .ok()?;

            let data = String::from_utf8(data).ok()?;
            //info!("{data}");

            let level = DesignedLevel::from_tsv_line(&data);

            Some(level)
        } else {
            None
        }
    }
}

impl DesignedLevel {
    pub fn calculate_unneeded_tiles<F: Fn(usize) -> bool>(
        &self,
        mut unneeded_tiles: GridSet,
        is_word_found: F,
    ) -> GridSet {
        let mut needed_characters: LetterCounts = LetterCounts::default();
        for word in self
            .words
            .iter()
            .enumerate()
            .filter(|x| !is_word_found(x.0))
            .map(|x| x.1)
        {
            let Some(characters) = word.letter_counts() else {
                //warn!("Could not get letter counts for word");
                return unneeded_tiles;
            };

            match needed_characters.try_union(&characters) {
                Some(set) => needed_characters = set,
                None => {
                    //warn!("Could not get letter counts for word");
                    return unneeded_tiles;
                }
            }
        }

        let remaining_characters = self
            .grid
            .enumerate()
            .filter(|(tile, _)| !unneeded_tiles.get_bit(tile))
            .map(|x| x.1)
            .filter(|x| !x.is_blank())
            .cloned();
        let Some(remaining_characters) = LetterCounts::try_from_iter(remaining_characters) else {
            //warn!("Could not get letter counts of remaining tiles");
            return unneeded_tiles;
        };

        let Some(potentially_redundant_characters) =
            remaining_characters.try_difference(&needed_characters)
        else {
            //warn!("Remaining characters was not a superset of needed characters"); //todo add log to this crate
            return unneeded_tiles;
        };

        'character_groups: for (character, mut remaining_copies) in
            potentially_redundant_characters.iter_groups()
        {
            let character_tiles = self
                .grid
                .enumerate()
                .filter(|x| x.1 == &character)
                .map(|x| x.0);
            if needed_characters.contains(character) {
                //we have additional copies of this character - try removing them
                'tiles_to_check: for tile in character_tiles {
                    if unneeded_tiles.get_bit(&tile) {
                        //we've already excluded this tile
                        continue 'tiles_to_check;
                    }

                    let mut remaining_grid = self.grid;
                    for t in unneeded_tiles.iter_true_tiles() {
                        remaining_grid[t] = Character::Blank;
                    }
                    remaining_grid[tile] = Character::Blank;

                    for word in self
                        .words
                        .iter()
                        .enumerate()
                        .filter(|x| !is_word_found(x.0))
                        .map(|x| x.1)
                    {
                        if !word.find_solution(&remaining_grid).is_some() {
                            continue 'tiles_to_check;
                        }
                    }
                    //this tile is not needed for any solutions
                    unneeded_tiles.set_bit(&tile, true);
                    remaining_copies -= 1;
                    if remaining_copies == 0 {
                        continue 'character_groups;
                    }
                }
            } else {
                //remove this character completely
                for tile in character_tiles {
                    unneeded_tiles.set_bit(&tile, true);
                }
            }
        }

        unneeded_tiles
    }
}

#[cfg(test)]
pub mod tests {
    use crate::prelude::*;

    #[test]
    pub fn test_calculate_needed_tiles() {
        let level = DesignedLevel::from_tsv_line(
            // spellchecker:disable-next-line
            "ASHPKILOEUIOGNDT\tSports\tPOLO\tSHOOTING\tKENDO\tSAILING\tLUGE\tSKIING",
        );

        // A|S|H|P
        // K|I|L|O
        // E|U|I|O
        // G|N|D|T

        //println!("{}", level.grid);

        let tests = vec![
            GridSet::EMPTY,                                              //all tiles are needed
            GridSet::from_iter([Tile::new_const::<2, 3>()].into_iter()), // kendo
            GridSet::from_iter([Tile::new_const::<0, 2>(), Tile::new_const::<1, 2>()].into_iter()), // luge
            GridSet::from_iter([Tile::new_const::<3, 0>()].into_iter()), // polo
            GridSet::from_iter([Tile::new_const::<0, 0>(), Tile::new_const::<2, 1>()].into_iter()), // sailing
            GridSet::from_iter(
                [
                    Tile::new_const::<2, 0>(),
                    Tile::new_const::<3, 1>(),
                    Tile::new_const::<3, 2>(),
                    Tile::new_const::<3, 3>(),
                ]
                .into_iter(),
            ), // skiing
            GridSet::ALL,
        ];

        let mut current_expected = GridSet::EMPTY;

        for (words_found, to_remove) in tests.into_iter().enumerate() {
            let actual = level.calculate_unneeded_tiles(current_expected, |wi| wi < words_found);

            current_expected = current_expected.union(&to_remove);

            if current_expected != actual {
                println!("Actual: ");
                println!("{actual}");
                println!("Expected: ");
                println!("{current_expected}");

                assert_eq!(actual, current_expected, "Test number {words_found}")
            }
        }
    }
}
