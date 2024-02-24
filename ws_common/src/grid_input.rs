use crate::prelude::*;
use ws_core::Tile;

#[derive(Debug, Default)]
pub struct GridInputState {
    last_tile: Option<Tile>,
    multi_click: Option<MultiClick>,

    //his4 his1b hie1 his4
    last_truncate: Option<Tile>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum MultiClick {
    DeleteOnEndThenStop,
    DeleteOnEndThenMaybeSwitch(Tile),
    SwitchOnStart(Tile),
}

impl GridInputState {
    pub fn handle_input_start(
        &mut self,
        chosen_state: &mut impl AnyResMut<ChosenState>,
        tile: Tile,
        grid: &Grid,
        found_words: &FoundWordsState,
    ) {
        if self.last_tile == Some(tile) {
            self.multi_click = Some(MultiClick::DeleteOnEndThenStop);
            return;
        }

        let next_multi_click: Option<MultiClick>;

        self.last_tile = Some(tile);

        if chosen_state.is_just_finished {
            *chosen_state.as_mut() = ChosenState::default();
            self.last_truncate = None;
        }

        if let Some(last) = chosen_state.solution.last() {
            if let Some(index) = chosen_state.solution.iter().position(|x| *x == tile) {
                // element is already present
                if index + 1 == chosen_state.solution.len() {
                    if Some(tile) == self.last_truncate {
                        chosen_state.solution.clear();
                        chosen_state.solution.push(tile);
                        self.last_truncate = None;
                        //info!("His1a index: {index}");
                        next_multi_click = None;
                    } else {
                        next_multi_click = Some(MultiClick::DeleteOnEndThenMaybeSwitch(tile));
                        //info!("His1b index: {index}");
                    }

                    self.last_truncate = None;
                } else if index == 0 {
                    //info!("His2 index: {index}");
                    chosen_state.solution.clear();
                    self.last_truncate = None;
                    next_multi_click = None;
                } else {
                    //info!("His3 index: {index}");
                    chosen_state.solution.truncate(index + 1);

                    //info!("His3 index: {index}  cs len {}");
                    self.last_truncate = Some(tile);
                    next_multi_click = None;
                }
            } else if last.is_adjacent_to(&tile) {
                //element is not already present
                if allow_tile(tile, grid, found_words) {
                    //info!("His4");
                    if self.multi_click == Some(MultiClick::SwitchOnStart(tile)) {
                        chosen_state.solution = ArrayVec::from_iter([tile]);
                    } else {
                        chosen_state.solution.push(tile);
                    }
                }
                next_multi_click = None;
            } else {
                //info!("His5");
                *chosen_state.as_mut() = ChosenState::default();
                next_multi_click = None;
            }
        } else {
            next_multi_click = None;
            //array is empty
            if allow_tile(tile, grid, found_words) {
                //info!("His5");
                chosen_state.solution.push(tile);
            }
        }

        self.multi_click = next_multi_click;
    }

    pub fn handle_input_move(
        &mut self,
        chosen_state: &mut impl AnyResMut<ChosenState>,
        tile: Tile,
        grid: &Grid,
        found_words: &FoundWordsState,
    ) {
        if self.last_tile == Some(tile) {
            return;
        }
        self.multi_click = None;
        self.last_tile = Some(tile);

        if chosen_state.is_just_finished {
            *chosen_state.as_mut() = ChosenState::default();
            self.last_truncate = None;
        }

        if let Some(last) = chosen_state.solution.last() {
            if let Some(index) = chosen_state.solution.iter().position(|x| *x == tile) {
                //info!("Him1");
                // element is already present
                chosen_state.solution.truncate(index + 1);
                self.last_truncate = None;
            } else if last.is_adjacent_to(&tile) {
                //element is not already present
                if allow_tile(tile, grid, found_words) {
                    //info!("Him2");
                    chosen_state.solution.push(tile);
                    self.last_truncate = None;
                }
            }
        }
    }

    pub fn handle_input_end(
        &mut self,
        chosen_state: &mut impl AnyResMut<ChosenState>,
        location: Tile,
    ) {
        if self.last_tile == Some(location) {
            match self.multi_click {
                Some(MultiClick::DeleteOnEndThenStop) => {
                    //info!("hie1");
                    chosen_state.solution.pop();
                    self.multi_click = None;
                }
                Some(MultiClick::DeleteOnEndThenMaybeSwitch(tile)) => {
                    chosen_state.solution.pop();
                    self.multi_click = if tile == location {
                        //info!("hie2a");
                        Some(MultiClick::SwitchOnStart(tile))
                    } else {
                        //info!("hie2b");
                        None
                    };
                }
                _ => {
                    //info!("hie3");
                    self.multi_click = None;
                }
            }
        } else {
            //info!("hie4");
            self.multi_click = None;
        }
        self.last_tile = None;
    }

    pub fn handle_input_end_no_location(&mut self) {
        //info!("hie no location");
        self.last_tile = None;
        self.multi_click = None;
    }
}

fn allow_tile(tile: Tile, grid: &Grid, found_words: &FoundWordsState) -> bool {
    if grid[tile].is_blank() {
        false
    } else {
        !found_words.unneeded_tiles.get_bit(&tile)
    }
}

#[cfg(test)]
pub mod tests {
    use std::str::FromStr;

    use itertools::Itertools;

    use crate::prelude::*;
    use test_case::test_case;

    #[test_case("", "")]
    #[test_case("s00", "00")]
    #[test_case("s00 e s01", "00 01")]
    #[test_case("s00 e s02", "")]
    #[test_case("s00 m01", "00 01")]
    #[test_case("s00 m01 e", "00 01")]
    #[test_case("s00 m01 m02 e", "00 01 02")]
    #[test_case("s00 m11 m22 e", "00 11 22")]
    #[test_case("s00 m11 m22 m11 e ", "00 11")]
    #[test_case("s00 m01 m02 e s01", "00 01")]
    #[test_case("s00 m01 m02 e s01 e s01", "01")]
    #[test_case("s00 m01 m02 e s03 e03 s03 e03 s03", "03")]
    pub fn test_inputs(input: &str, expected: &str) {
        let input_list = parse_input_list(input);
        let expected = parse_expected_list(expected);

        let mut state = GridInputState::default();

        let found_words = FoundWordsState::default();

        let mut chosen_state = TestResMut {
            value: &mut ChosenState::default(),
            added: false,
            last_changed: None,
        };

        let grid = Grid::from_fn(|_| Character::A);

        for input in input_list.into_iter() {
            match input {
                Input::EndNoLocation => state.handle_input_end_no_location(),
                Input::Start(tile) => {
                    state.handle_input_start(&mut chosen_state, tile, &grid, &found_words)
                }
                Input::Move(tile) => {
                    state.handle_input_move(&mut chosen_state, tile, &grid, &found_words)
                }
                Input::End(tile) => state.handle_input_end(&mut chosen_state, tile),
            }
        }

        assert_eq!(chosen_state.solution, expected);
    }

    fn parse_input_list(input: &str) -> Vec<Input> {
        input
            .split_ascii_whitespace()
            .map(|x| Input::from_str(x).unwrap())
            .collect_vec()
    }

    fn parse_expected_list(expected: &str) -> Solution {
        Solution::from_iter(
            expected
                .split_ascii_whitespace()
                .map(|x| try_parse_tile(x).unwrap()),
        )
    }

    #[derive(Debug, Clone, Copy, PartialEq)]
    enum Input {
        EndNoLocation,
        Start(Tile),
        Move(Tile),
        End(Tile),
    }

    impl FromStr for Input {
        type Err = &'static str;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            if s == "E" || s == "e" {
                return Ok(Self::EndNoLocation);
            }
            let (command, tile) = s.split_at(1);

            let tile = try_parse_tile(tile)?;

            match command {
                "s" | "S" => Ok(Self::Start(tile)),
                "m" | "M" => Ok(Self::Move(tile)),
                "e" | "E" => Ok(Self::End(tile)),
                _ => Err("Unrecognized command"),
            }
        }
    }

    fn try_parse_tile(s: &str) -> Result<Tile, &'static str> {
        if s.len() != 2 {
            return Err("Tile string should be two characters");
        }

        let (col, row) = s.split_at(1);

        let col: u8 = col.parse().map_err(|_| "Could not parse column")?;
        let row: u8 = row.parse().map_err(|_| "Could not parse row")?;

        Tile::try_new(col, row).ok_or("Tile out of range")
    }
}
