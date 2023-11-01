use crate::prelude::*;

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

    pub fn handle_input_move(
        &mut self,
        chosen_state: &mut ResMut<ChosenState>,
        location: Tile,
        grid: &Grid,
    ) {
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