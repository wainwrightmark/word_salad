use serde::{Deserialize, Serialize};
use std::str::FromStr;
use ustr::ustr;

use crate::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Word {
    pub characters: CharsArray,
    pub text: Ustr,
}

impl WordTrait for Word {
    fn characters(&self) -> &CharsArray {
        &self.characters
    }
}

impl FromStr for Word {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let characters = normalize_characters_array(s)?;

        if characters.len() <= 3 {
            return Err("Word has 3 or fewer characters");
        }

        Ok(Self {
            characters,
            text: ustr(s),
        })
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

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

        let expected: ArrayVec<Tile, 16> = arrayvec::ArrayVec::from_iter([
            Tile::new_const::<0, 1>(),
            Tile::new_const::<1, 2>(),
            Tile::new_const::<0, 3>(),
            Tile::new_const::<1, 3>(),
            Tile::new_const::<2, 3>(),
        ]);

        assert_eq!(expected, path)
    }

    #[test]
    pub fn test_find_paths() {
        // spellchecker:disable-next-line
        let grid = try_make_grid("SGOPELWODEMKVEEU").expect("Should be able to make grid");
        // spellchecker:disable-next-line
        let pokemon = Word::from_str("eevee").expect("Should be able to make word");

        let paths = pokemon.find_solutions(&grid);

        let expected_0: ArrayVec<Tile, 16> = arrayvec::ArrayVec::from_iter([
            Tile::new_const::<0, 1>(),
            Tile::new_const::<1, 2>(),
            Tile::new_const::<0, 3>(),
            Tile::new_const::<1, 3>(),
            Tile::new_const::<2, 3>(),
        ]);

        assert_eq!(2, paths.len());

        let expected_1: ArrayVec<Tile, 16> = arrayvec::ArrayVec::from_iter([
            Tile::new_const::<2, 3>(),
            Tile::new_const::<1, 3>(),
            Tile::new_const::<0, 3>(),
            Tile::new_const::<1, 2>(),
            Tile::new_const::<0, 1>(),
        ]);

        assert_eq!(expected_0, paths[0]);
        assert_eq!(expected_1, paths[1]);
    }
}
