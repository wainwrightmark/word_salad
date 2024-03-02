use std::{
    collections::{HashMap, HashSet},
    iter::FusedIterator,
    str::FromStr,
};

use itertools::Itertools;
use prime_bag::PrimeBagElement;
use strum::IntoEnumIterator;

use crate::{layout::rect, Character, CharacterMap, CharsArray, Grid, GridSet, WordTrait};

pub fn do_complete_solve(
    grid: &Grid,
    all_words_text: &str,
    min_word_length: usize,
) -> Vec<CharsArray> {
    let mut wa = WordAutomata::default();
    for line in all_words_text.lines() {
        if let Ok(word) = RawWord::from_str(line) {
            if word.characters.len() >= min_word_length {
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
    pub inner: CharacterMap<Option<usize>>, //todo option<nonzerou32>
}

struct AutomataIterator<'a> {
    automata: &'a WordAutomata,
    stack: Vec<(usize, Character)>,
}

impl<'a> FusedIterator for AutomataIterator<'a> {}

impl<'a> Iterator for AutomataIterator<'a> {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        fn increment_last(stack: &mut Vec<(usize, Character)>) {
            loop {
                match stack.last_mut() {
                    Some((_, Character::Z)) => {
                        stack.pop();
                    }
                    Some(other) => {
                        if other.1.is_blank() {
                            other.1 = Character::from_prime_index(0);
                        } else {
                            other.1 = Character::from_prime_index(other.1.into_prime_index() + 1);
                        }
                        return;
                    }
                    None => return,
                }
            }
        }

        loop {
            let (top_state_index, character) = self.stack.last()?.clone();

            // println!(
            //     "{}",
            //     self.stack.iter().map(|x|x.1.as_char()).join("")
            // );

            match self.automata.slab[top_state_index].inner.get(character) {
                Some(next_state_index) => {
                    if character.is_blank() {
                        //this is a valid word
                        let word = self
                            .stack
                            .iter()
                            .take(self.stack.len() - 1)
                            .map(|x| x.1.as_char())
                            .join("");
                        increment_last(&mut self.stack);

                        return Some(word);
                    } else {
                        //Make the stack bigger, exploring the next state
                        self.stack.push((*next_state_index, Character::Blank));
                    }
                }
                None => {
                    increment_last(&mut self.stack);
                }
            }
        }
    }
}

impl WordAutomata {
    pub fn iter<'a>(&'a self) -> AutomataIterator<'a> {
        AutomataIterator {
            automata: self,
            stack: vec![(0, Character::Blank)],
        }
    }

    //todo prevent adding new entries after compression
    pub fn compress(&mut self) {
        let mut leaves: HashMap<CharacterMap<Option<usize>>, usize> = Default::default();
        let mut replacements: HashMap<usize, usize> = Default::default();
        let mut removed: HashSet<usize> = Default::default();
        loop {
            leaves.clear();
            replacements.clear();

            for (index, state) in self.slab.iter().enumerate() {
                match leaves.entry(state.inner) {
                    std::collections::hash_map::Entry::Occupied(o) => {
                        replacements.insert(index, *o.get());
                        removed.insert(index);
                    }
                    std::collections::hash_map::Entry::Vacant(v) => {
                        v.insert(index);
                    }
                }
            }

            if replacements.is_empty() {
                break;
            }
            let mut changed = false;
            for state in self.slab.iter_mut() {
                for c in Character::iter() {
                    if let Some(old_index) = state.inner.get(c) {
                        if let Some(new_index) = replacements.get(old_index) {
                            state.inner.set(c, Some(*new_index));
                            changed = true;
                        }
                    }
                }
            }
            if !changed {
                break;
            }
        }

        if removed.is_empty() {
            return;
        }

        replacements.clear();
        let mut new_slab: Vec<State> = Default::default();
        let mut next_index = 0;

        for (old_index, state) in self.slab.drain(..).enumerate() {
            if removed.contains(&old_index) {
                continue;
            }

            new_slab.push(state);
            if old_index != next_index {
                replacements.insert(old_index, next_index);
            }
            next_index += 1;
        }
        self.slab = new_slab;

        for state in self.slab.iter_mut() {
            for c in Character::iter() {
                if let Some(old_index) = state.inner.get(c) {
                    if let Some(new_index) = replacements.get(old_index) {
                        state.inner.set(c, Some(*new_index));
                    }
                }
            }
        }
    }

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
    pub fn test_iter() {
        let mut wa = WordAutomata::default();

        for word in [
            "Earth", "Mars", "Neptune", "Pluto", "Saturn", "Uranus", "Venus", "Some", "Random",
            "Word",
        ] {
            let word = RawWord::from_str(word).unwrap();
            wa.add_word(&word);
        }

        wa.compress();

        let v = wa.iter().collect_vec();

        let joined = v.join(", ");

        assert_eq!(
            joined,
            "EARTH, NEPTUNE, SATURN, SOME, RANDOM, URANUS, MARS, WORD, PLUTO, VENUS"
        );
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
        println!("Uncompressed - {} states", wa.slab.len());
        wa.compress();
        println!("Compressed - {} states", wa.slab.len());

        let grid = try_make_grid("VENMOUAULTRSHPEN").unwrap();

        let grid_words = wa.find_all_words(&grid);

        let found_words = grid_words
            .iter()
            .map(|x| x.iter().map(|c| c.as_char()).join(""))
            .join(", ");

        assert_eq!(
            found_words,
            "VENUS, EARTH, MARS, URANUS, SATURN, PLUTO, NEPTUNE"
        )
    }
}
