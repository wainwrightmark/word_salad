use super::word_trait::WordTrait;
use crate::{finder::helpers::LetterCounts, Character, Grid, GridSet};

pub trait LevelTrait {
    type Word: WordTrait;
    fn grid(&self) -> Grid;

    fn words(&self) -> &[Self::Word];

    fn calculate_unneeded_tiles<F: Fn(usize) -> bool>(
        &self,
        mut unneeded_tiles: GridSet,
        is_word_found: F,
    ) -> GridSet {
        let mut needed_characters: LetterCounts = LetterCounts::default();
        for word in self
            .words()
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

        let grid = self.grid();

        let remaining_characters = grid
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

        'character_groups: for (character, mut remaining_copies) in potentially_redundant_characters
            .iter_groups()
            .map(|x| (x.0, x.1.get()))
        {
            let character_tiles = grid.enumerate().filter(|x| x.1 == &character).map(|x| x.0);
            if needed_characters.contains(character) {
                //we have additional copies of this character - try removing them
                'tiles_to_check: for tile in character_tiles {
                    if unneeded_tiles.get_bit(&tile) {
                        //we've already excluded this tile
                        continue 'tiles_to_check;
                    }

                    let mut remaining_grid = self.grid();
                    for t in unneeded_tiles.iter_true_tiles() {
                        remaining_grid[t] = Character::Blank;
                    }
                    remaining_grid[tile] = Character::Blank;

                    for word in self
                        .words()
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
