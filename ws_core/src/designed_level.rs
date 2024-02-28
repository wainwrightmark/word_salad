use std::str::FromStr;

use crate::{finder::helpers::LetterCounts, prelude::*, Grid};
use itertools::Itertools;
use log::warn;
use ustr::Ustr;

#[derive(Debug, Clone, PartialEq)]
pub struct DesignedLevel {
    pub name: Ustr,
    pub numbering: Option<Numbering>,

    // Attribution
    pub extra_info: Option<Ustr>,
    pub grid: Grid,
    pub words: Vec<DisplayWord>,
    pub special_colors: Option<Vec<BasicColor>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Numbering {
    WordSaladNumber(usize),
    SequenceNumber(usize),
}

impl std::fmt::Display for DesignedLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{grid}\t{name}\t{words}",
            grid = self.grid.iter().join(""),
            name = self.name,
            words = self.words.iter().join("\t")
        )
    }
}

impl DesignedLevel {
    pub fn full_name(&self) -> Ustr {
        match self.numbering {
            Some(Numbering::SequenceNumber(num)) => {
                Ustr::from(format!("{} {num}", self.name).as_str())
            }
            Some(Numbering::WordSaladNumber(num)) => {
                Ustr::from(format!("#{num} {}", self.name).as_str())
            }
            None => self.name,
        }
    }

    pub fn unknown() -> Self {
        Self {
            name: Ustr::from("Unknown"),
            numbering: None,
            extra_info: None,
            grid: Grid::from_inner([Character::Blank; 16]),
            words: vec![],
            special_colors: None,
        }
    }

    pub fn letter_counts(&self) -> Option<LetterCounts> {
        LetterCounts::try_from_iter(self.grid.iter().cloned())
    }

    pub fn from_tsv_line(line: &str) -> Result<Self, String> {
        let mut iter = line.split('\t');

        let chars: &str = iter
            .next()
            .ok_or_else(|| format!("Level '{line}' should have a grid"))?;
        let name: &str = iter
            .next()
            .ok_or_else(|| format!("Level '{line}' should have a name"))?;

        let grid = try_make_grid(chars)
            .ok_or_else(|| format!("Level '{line}' should be able to make grid"))?;

        let mut words: Vec<DisplayWord> = iter
            .map(|x| {
                DisplayWord::from_str(x.trim()).map_err(|e| format!("Word '{x}' is not valid {e}"))
            })
            .try_collect()?;

        words.sort();

        let mut name = name;

        let special_colors = if name.ends_with('}') {
            if let Some(index) = name.find('{') {
                let (prefix, colors) = name.split_at(index);
                name = prefix.trim_end();
                let colors = &colors[1..(colors.len() - 1)];
                let mut colors_vec = Vec::<BasicColor>::default();

                for c in colors.split(',') {
                    if let Some(color) = BasicColor::try_from_str(c) {
                        colors_vec.push(color);
                    } else {
                        warn!("Could not parse color '{c}'");
                    }
                }
                if colors_vec.is_empty() {
                    None
                } else {
                    Some(colors_vec)
                }
            } else {
                None
            }
        } else {
            None
        };

        let extra_info = if name.ends_with(']') {
            if let Some(index) = name.find('[') {
                let (prefix, extra_info) = name.split_at(index);
                name = prefix.trim_end();
                let extra_info = Ustr::from(&extra_info[1..(extra_info.len() - 1)]);
                Some(extra_info)
            } else {
                None
            }
        } else {
            None
        };

        let name = Ustr::from(name.trim_end());

        Ok(Self {
            name,
            numbering: None,
            extra_info,
            grid,
            words,
            special_colors,
        })
    }
}

impl LevelTrait for DesignedLevel {
    type Word = DisplayWord;

    fn grid(&self) -> Grid {
        self.grid
    }

    fn words(&self) -> &[Self::Word] {
        self.words.as_slice()
    }
}

#[cfg(test)]
pub mod tests {
    use crate::prelude::*;

    #[test]
    pub fn test_calculate_needed_tiles() {
        let level = DesignedLevel::from_tsv_line(
            // spellchecker:disable-next-line
            "ASHPKILOEUIOGNDT\tSports\tPOLO\tSHOOTING\tKENDO\tSAILING\tLUGE\tSKIING",
        )
        .unwrap();

        // A|S|H|P
        // K|I|L|O
        // E|U|I|O
        // G|N|D|T

        //println!("{}", level.grid);

        let tests = vec![
            GridSet::EMPTY,                                              //all tiles are needed
            GridSet::from_iter([Tile::new_const::<2, 3>()].into_iter()), // kendo
            GridSet::from_iter([Tile::new_const::<0, 2>(), Tile::new_const::<1, 2>()].into_iter()), // luge
            GridSet::from_iter([Tile::new_const::<3, 0>()].into_iter()), // polo
            GridSet::from_iter([Tile::new_const::<0, 0>(), Tile::new_const::<2, 1>()].into_iter()), // sailing
            GridSet::from_iter(
                [
                    Tile::new_const::<2, 0>(),
                    Tile::new_const::<3, 1>(),
                    Tile::new_const::<3, 2>(),
                    Tile::new_const::<3, 3>(),
                ]
                .into_iter(),
            ), // skiing
            GridSet::ALL,
        ];

        let mut current_expected = GridSet::EMPTY;

        for (words_found, to_remove) in tests.into_iter().enumerate() {
            let actual = level.calculate_unneeded_tiles(current_expected, |wi| wi < words_found);

            current_expected = current_expected.union(&to_remove);

            if current_expected != actual {
                println!("Actual: ");
                println!("{actual}");
                println!("Expected: ");
                println!("{current_expected}");

                assert_eq!(actual, current_expected, "Test number {words_found}")
            }
        }
    }
}
