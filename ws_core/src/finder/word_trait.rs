use arrayvec::ArrayVec;
use geometrid::vector::Vector;

use super::helpers::LetterCounts;
use crate::{Character, CharsArray, Grid, GridSet, Solution, Tile};
pub trait WordTrait {
    fn characters(&self) -> &CharsArray;

    fn letter_counts(&self) -> Option<LetterCounts> {
        LetterCounts::try_from_iter(self.characters().iter().cloned())
    }

    fn find_solutions(&self, grid: &Grid) -> Vec<Solution> {
        let characters = self.characters();
        //TODO return iter
        //TODO more efficient path if word has no duplicate letters

        let Some(first_char) = characters.get(0) else {
            return Default::default();
        };
        let mut solutions: Vec<Solution> = vec![];

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
                    solutions.push(path.clone());
                    continue;
                }
            };

            loop {
                if let Some(vector) = Vector::UNITS.get(current_index as usize) {
                    current_index += 1;
                    if let Some(adjacent_tile) = current_tile + vector {
                        if grid[adjacent_tile] == char_to_find
                            && !used_tiles.get_bit(&adjacent_tile)
                        {
                            //we need to go deeper
                            path.push(current_tile);

                            match characters.get(path.len() + 1) {
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

                    char_to_find = match characters.get(path.len() + 1) {
                        Some(c) => *c,
                        None => break,
                    };
                }
            }
        }

        solutions
    }

    fn find_solution(&self, grid: &Grid) -> Option<Solution> {
        let characters = self.characters();
        // if characters.iter().all_unique(){

        // }

        //TODO more efficient path if word has no duplicate letters

        let first_char = characters.get(0)?;

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
                        if grid[adjacent_tile] == char_to_find
                            && !used_tiles.get_bit(&adjacent_tile)
                        {
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

    fn find_solution_with_tiles(&self, grid: &Grid, unneeded_tiles: GridSet) -> Option<Solution> {
        let mut grid = grid.clone();
        for tile in unneeded_tiles.iter_true_tiles() {
            grid[tile] = Character::Blank;
        }
        Self::find_solution(self, &grid)
    }


}
