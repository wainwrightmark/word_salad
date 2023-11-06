use arrayvec::ArrayVec;
use serde::{Deserialize, Serialize};

use crate::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Word {
    pub characters: CharsArray,
    pub text: String,
}

pub fn find_solution(characters: &CharsArray, grid: &Grid) -> Option<Solution> {
    //TODO more efficient path if word has no duplicate letters

    let Some(first_char) = characters.get(0) else {
        return Default::default();
    };

    for first_tile in Tile::iter_by_row().filter(|tile| grid[*tile] == *first_char) {
        let mut path: ArrayVec<Tile, 16> = Default::default();
        let mut used_tiles: GridSet = Default::default();
        let mut indices: ArrayVec<u8, 16> = Default::default();

        let mut current_index: u8 = 0;
        let mut current_tile: Tile = first_tile;
        let mut char_to_find: Character = match characters.get(1) {
            Some(c) => *c,
            None => {
                path.push(current_tile);
                return Some(path);
            }
        };

        loop {
            if let Some(vector) = Vector::UNITS.get(current_index as usize) {
                current_index += 1;
                if let Some(adjacent_tile) = current_tile + vector {
                    if grid[adjacent_tile] == char_to_find && !used_tiles.get_bit(&adjacent_tile) {
                        //we need to go deeper
                        path.push(current_tile);
                        used_tiles.set_bit(&current_tile, true);
                        indices.push(current_index);
                        current_index = 0;
                        current_tile = adjacent_tile;
                        char_to_find = match characters.get(path.len() + 1) {
                            Some(c) => *c,
                            None => {
                                path.push(current_tile);
                                return Some(path);
                            }
                        };
                    }
                }
            } else {
                //we have run out of options to try - go up a level
                let Some(ct) = path.pop() else {
                    break;
                };

                used_tiles.set_bit(&ct, false);
                current_tile = ct;
                let Some(ci) = indices.pop() else {
                    break;
                };
                current_index = ci;

                char_to_find = match characters.get(path.len() + 1) {
                    Some(c) => *c,
                    None => break,
                };
            }
        }
    }

    None
}

impl Word {
    pub fn from_str(text: &str) -> Result<Self, &'static str> {
        let mut characters = ArrayVec::<Character, 16>::default();

        for c in text.chars() {
            let character = Character::try_from(c)?;
            if !character.is_blank() {
                characters.try_push(character).map_err(|_| "Word is too long")?;
            }
        }

        Ok(Self {
            characters,
            text: text.to_string(),
        })
    }

    pub fn find_solution(&self, grid: &Grid) -> Option<Solution> {
        //TODO more efficient path if word has no duplicate letters

        find_solution(&self.characters, grid)
    }

    pub fn find_solutions(&self, grid: &Grid) -> Vec<Solution> {
        //TODO more efficient path if word has no duplicate letters

        let Some(first_char) = self.characters.get(0) else {
            return Default::default();
        };
        let mut solutions: Vec<Solution> = vec![];

        for first_tile in Tile::iter_by_row().filter(|tile| grid[*tile] == *first_char) {
            let mut path: ArrayVec<Tile, 16> = Default::default();
            let mut used_tiles: GridSet = Default::default();
            let mut indices: ArrayVec<u8, 16> = Default::default();

            let mut current_index: u8 = 0;
            let mut current_tile: Tile = first_tile;
            let mut char_to_find: Character = match self.characters.get(1) {
                Some(c) => *c,
                None => {
                    path.push(current_tile);
                    solutions.push(path.clone());
                    continue;
                }
            };

            loop {
                if let Some(vector) = Vector::UNITS.get(current_index as usize) {
                    current_index += 1;
                    if let Some(adjacent_tile) = current_tile + vector {
                        if grid[adjacent_tile] == char_to_find && !used_tiles.get_bit(&adjacent_tile) {
                            //we need to go deeper
                            path.push(current_tile);

                            match self.characters.get(path.len() + 1) {
                                Some(c) => {
                                    used_tiles.set_bit(&current_tile, true);
                                    indices.push(current_index);
                                    current_index = 0;
                                    current_tile = adjacent_tile;
                                    char_to_find = *c;
                                }
                                None => {
                                    //we have found all the characters we need to find
                                    let mut final_path = path.clone();
                                    final_path.push(adjacent_tile);

                                    solutions.push(final_path);
                                    path.pop();
                                }
                            };
                        }
                    }
                } else {
                    //we have run out of options to try - go up a level
                    let Some(ct) = path.pop() else {
                        break;
                    };

                    used_tiles.set_bit(&ct, false);
                    current_tile = ct;
                    let Some(ci) = indices.pop() else {
                        break;
                    };
                    current_index = ci;

                    char_to_find = match self.characters.get(path.len() + 1) {
                        Some(c) => *c,
                        None => break,
                    };
                }
            }
        }

        solutions
    }
}

#[cfg(test)]
mod tests {
    use arrayvec::ArrayVec;

    use crate::prelude::*;

    #[test]
    pub fn test_find_path() {
        // spellchecker:disable-next-line
        let grid = try_make_grid("SGOPELWODEMKVEEU").expect("Should be able to make grid");
        // spellchecker:disable-next-line
        let pokemon = Word::from_str("eevee").expect("Should be able to make word");

        let path = pokemon
            .find_solution(&grid)
            // spellchecker:disable-next-line
            .expect("Should be able to find a path for 'eevee'");

        let expected: ArrayVec<Tile, 16> = arrayvec::ArrayVec::from_iter(
            [
                Tile::new_const::<0, 1>(),
                Tile::new_const::<1, 2>(),
                Tile::new_const::<0, 3>(),
                Tile::new_const::<1, 3>(),
                Tile::new_const::<2, 3>(),
            ],
        );

        assert_eq!(expected, path)
    }

    #[test]
    pub fn test_find_paths() {
        // spellchecker:disable-next-line
        let grid = try_make_grid("UOEVFRNEHITSNTFY").expect("Should be able to make grid");
        let one = Word::from_str("one").expect("Should be able to make word");

        let paths = one.find_solutions(&grid);

        assert_eq!(2, paths.len());

        let expected_0: ArrayVec<Tile, 16> = arrayvec::ArrayVec::from_iter(
            [
                Tile::new_const::<1, 0>(),
                Tile::new_const::<2, 1>(),
                Tile::new_const::<2, 0>(),
            ],
        );

        let expected_1: ArrayVec<Tile, 16> = arrayvec::ArrayVec::from_iter(
            [
                Tile::new_const::<1, 0>(),
                Tile::new_const::<2, 1>(),
                Tile::new_const::<3, 1>(),
            ],
        );

        assert_eq!(expected_0, paths[0]);
        assert_eq!(expected_1, paths[1]);
    }
}
