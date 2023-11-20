use crate::prelude::*;
use ws_core::Tile;

#[derive(Debug, Default)]
pub struct GridInputState {
    last_tile: Option<Tile>,
    delete_on_end: bool,
}

impl GridInputState {
    pub fn handle_input_start(
        &mut self,
        chosen_state: &mut ResMut<ChosenState>,
        tile: Tile,
        grid: &Grid,
        found_words: &FoundWordsState,
    ) {
        //todo clicking on the last one should only go back one character
        // todo bug when if you do a path, then cancel, then do that path again that path goes way

        if self.last_tile == Some(tile) {
            self.delete_on_end = true;
            return;
        }
        self.delete_on_end = false;
        self.last_tile = Some(tile);

        if let Some(last) = chosen_state.0.last() {
            if let Some(index) = chosen_state.0.iter().position(|x| *x == tile) {
                // element is already present
                if index + 1 == chosen_state.0.len() {
                    self.delete_on_end = true;
                    //chosen_state.0.clear(); do nothing
                } else {
                    chosen_state.0.truncate(index + 1);
                }
            } else if last.is_adjacent_to(&tile) {
                //element is not already present
                if allow_tile(tile, grid, found_words) {
                    chosen_state.0.push(tile);
                }
            } else {
                *chosen_state.as_mut() = ChosenState::default();
            }
        } else {
            //array is empty
            if allow_tile(tile, grid, found_words) {
                chosen_state.0.push(tile);
            }
        }
    }

    pub fn handle_input_move(
        &mut self,
        chosen_state: &mut ResMut<ChosenState>,
        tile: Tile,
        grid: &Grid,
        found_words: &FoundWordsState,
    ) {
        if self.last_tile == Some(tile) {
            return;
        }
        self.delete_on_end = false;
        self.last_tile = Some(tile);

        if let Some(last) = chosen_state.0.last() {
            if let Some(index) = chosen_state.0.iter().position(|x| *x == tile) {
                // element is already present
                if index + 1 == chosen_state.0.len() {
                    //chosen_state.0.clear(); do nothing
                } else {
                    chosen_state.0.truncate(index + 1);
                }
            } else if last.is_adjacent_to(&tile) {
                //element is not already present
                if allow_tile(tile, grid, found_words) {
                    chosen_state.0.push(tile);
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

fn allow_tile(tile: Tile, grid: &Grid, found_words: &FoundWordsState) -> bool {
    if grid[tile].is_blank() {
        false
    } else {
        !found_words.unneeded_tiles.get_bit(&tile)
    }
}
