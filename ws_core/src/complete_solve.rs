use std::str::FromStr;

use crate::{CharacterMap, CharsArray, Grid, GridSet,  WordTrait};

pub fn do_complete_solve(grid: &Grid, all_words_text: &str, min_word_length: usize) -> Vec<CharsArray> {
    let mut wa = WordAutomata::default();
    for line in all_words_text.lines() {
        if let Ok(word) = RawWord::from_str(line){
            if word.characters.len() >= min_word_length{
                wa.add_word(&word);
            }
        }


    }

    wa.find_all_words(grid)
}

#[derive(Debug)]
struct WordAutomata {
    pub slab: Vec<State>,
}

impl Default for WordAutomata {
    fn default() -> Self {
        Self {
            slab: vec![State::default()],
        }
    }
}

#[derive(Debug, Default)]
struct State {
    pub inner: CharacterMap<Option<usize>>,
}

impl WordAutomata {
    pub fn contains(&self, w: &impl WordTrait) -> bool {
        let mut state = self.slab.get(0).unwrap();

        for c in w.characters().iter() {
            match state.inner.get(*c) {
                Some(a) => state = self.slab.get(*a).unwrap(),
                None => return false,
            }
        }
        state.inner.get(crate::Character::Blank).is_some()
    }

    pub fn find_all_words(&self, grid: &Grid) -> Vec<CharsArray> {
        fn find_words_inner(
            wa: &WordAutomata,
            results: &mut Vec<CharsArray>,
            current_index: usize,
            grid: &Grid,
            new_tile: crate::Tile,
            used_tiles: GridSet,
            previous_chars: &CharsArray,
        ) {
            let character = grid[new_tile];

            if let Some(next_index) = wa.slab[current_index].inner.get(character) {
                let state = &wa.slab[*next_index];
                let mut next_chars = previous_chars.clone();
                next_chars.push(character);

                let next_used_tiles = used_tiles.with_bit_set(&new_tile, true);

                for tile in new_tile.iter_adjacent().filter(|x| !used_tiles.get_bit(&x)) {
                    find_words_inner(
                        wa,
                        results,
                        *next_index,
                        grid,
                        tile,
                        next_used_tiles,
                        &next_chars,
                    );
                }

                if state.inner.get(crate::Character::Blank).is_some() {
                    results.push(next_chars);
                }
            }
        }

        let mut result: Vec<CharsArray> = vec![];

        for tile in crate::Tile::iter_by_row() {
            find_words_inner(
                self,
                &mut result,
                0,
                grid,
                tile,
                GridSet::EMPTY,
                &CharsArray::new(),
            )
        }

        return result;
    }

    /// Returns true if the word was added
    pub fn add_word(&mut self, w: &impl WordTrait) -> bool {
        let mut state_index: usize = 0;

        for c in w.characters().iter() {
            match self.slab[state_index].inner.get(*c) {
                Some(a) => {
                    state_index = *a;
                }
                None => {
                    let new_state = State::default();
                    let new_state_index = self.slab.len();
                    self.slab.push(new_state);

                    self.slab[state_index].inner.set(*c, Some(new_state_index));
                    state_index = new_state_index;
                }
            }
        }

        match self
            .slab
            .get(state_index)
            .unwrap()
            .inner
            .get(crate::Character::Blank)
        {
            Some(_) => return false, //word was already present
            None => {
                self.slab[state_index]
                    .inner
                    .set(crate::Character::Blank, Some(0));
                return true; //word was added
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RawWord {
    pub characters: CharsArray,
}

impl WordTrait for RawWord {
    fn characters(&self) -> &CharsArray {
        &self.characters
    }
}

impl std::str::FromStr for RawWord {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let characters = crate::normalize_characters_array(s)?;

        Ok(Self { characters })
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use itertools::Itertools;

    use crate::try_make_grid;

    use super::*;
    #[test]
    pub fn test_word_automata() {
        let mark = RawWord::from_str("Mark").unwrap();
        let mar = RawWord::from_str("Mar").unwrap();

        let mut wa = WordAutomata::default();

        assert!(!wa.contains(&mar));
        assert!(!wa.contains(&mark));

        assert!(wa.add_word(&mar));

        assert!(wa.contains(&mar));
        assert!(!wa.contains(&mark));

        assert!(wa.add_word(&mark));

        assert!(wa.contains(&mar));
        assert!(wa.contains(&mark));
    }

    #[test]
    pub fn test_on_grid() {
        let mut wa = WordAutomata::default();

        for word in [
            "Earth", "Mars", "Neptune", "Pluto", "Saturn", "Uranus", "Venus", "Some", "Random",
            "Word",
        ] {
            let word = RawWord::from_str(word).unwrap();
            wa.add_word(&word);
        }

        let grid = try_make_grid("VENMOUAULTRSHPEN").unwrap();

        let grid_words = wa.find_all_words(&grid);

        let found_words = grid_words
            .iter()
            .map(|x| x.iter().map(|c| c.as_char().to_ascii_lowercase()).join(""))
            .join(", ");

        assert_eq!(
            found_words,
            "VENUS, EARTH, MARS, URANUS, SATURN, PLUTO, NEPTUNE"
        )
    }
}
