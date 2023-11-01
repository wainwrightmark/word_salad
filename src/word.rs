use arrayvec::ArrayVec;
use serde::{Serialize, Deserialize};

use crate::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Word {
    pub characters: CharsArray,
    pub text: String,
}

impl Word {
    pub fn from_static_str(text: &'static str) -> Result<Self, ()> {
        let mut characters = ArrayVec::<Character, 16>::default();

        for c in text.chars() {
            let character = Character::try_from(c)?;
            characters.try_push(character).map_err(|_| ())?;
        }

        Ok(Self { characters, text: text.to_string() })
    }

    pub fn find_solution(&self, grid: &Grid) -> Option<Solution> {
        //TODO more efficient path if word has no duplicate letters
        let Some(first_char) = self.characters.get(0) else {
            return Default::default();
        };

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
                    return Some(path);
                }
            };

            loop {
                if let Some(vector) = Vector::UNITS.get(current_index as usize) {
                    current_index += 1;
                    if let Some(adjacent_tile) = current_tile + vector {
                        if grid[adjacent_tile] == char_to_find {
                            if used_tiles.get_bit(&adjacent_tile) == false {
                                //we need to go deeper
                                path.push(current_tile);
                                used_tiles.set_bit(&current_tile, true);
                                indices.push(current_index);
                                current_index = 0;
                                current_tile = adjacent_tile;
                                char_to_find = match self.characters.get(path.len() + 1) {
                                    Some(c) => *c,
                                    None => {
                                        path.push(current_tile);
                                        return Some(path);
                                    }
                                };
                            }
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

        None
    }

    pub fn is_complete(&self, grid: &Grid) -> bool {
        //todo just use find_path
        let Some(first_char) = self.characters.get(0) else {
            return true;
        };

        for tile in Tile::iter_by_row().filter(|tile| grid[*tile] == *first_char) {
            let mut used = GridSet::default();
            used.set_bit(&tile, true);
            if self.is_complete_helper(grid, 1, tile, used) {
                return true;
            }
        }
        false
    }

    fn is_complete_helper(&self, grid: &Grid, index: usize, previous: Tile, used: GridSet) -> bool {
        let Some(char) = self.characters.get(index) else {
            return true;
        };

        for tile in previous
            .iter_adjacent()
            .filter(|t| &grid[*t] == char)
            .filter(|t| !used.get_bit(t))
        {
            let mut new_used = used.clone();
            new_used.set_bit(&tile, true);

            if self.is_complete_helper(grid, index + 1, tile, new_used) {
                return true;
            }
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use arrayvec::ArrayVec;

    use crate::prelude::*;

    #[test]
    pub fn test_find_path() {
        // SGOP
        // ELWO
        // DEMK
        // VEEU

        let grid = try_make_grid("SGOPELWODEMKVEEU").expect("Should be able to make grid");
        let eevee = Word::from_static_str("eevee").expect("Should be able to make word");

        let path = eevee
            .find_solution(&grid)
            .expect("Should be able to find a path for 'eevee'");

        let expected: ArrayVec<Tile, 16> = arrayvec::ArrayVec::from_iter(
            [
                Tile::new_const::<0, 1>(),
                Tile::new_const::<1, 2>(),
                Tile::new_const::<0, 3>(),
                Tile::new_const::<1, 3>(),
                Tile::new_const::<2, 3>(),
            ]
            .into_iter(),
        );

        assert_eq!(expected, path)
    }
}
