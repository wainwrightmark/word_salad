use crate::all_levels::*;
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use serde::{Deserialize, Serialize};
use strum::{Display, EnumCount, EnumIs, EnumIter};
use ws_core::DesignedLevel;

#[repr(u8)]
#[derive(
    Debug,
    Clone,
    Copy,
    Display,
    EnumCount,
    EnumIter,
    EnumIs,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Serialize,
    Deserialize,
    Hash,
    FromPrimitive,
)]

pub enum LevelSequence {
    //DO NOT CHANGE THESE NUMBERS - THEY ARE USED FOR COMPLETION TRACKING
    EUCapitals = 2,
    EUCountries = 3,
    USStates = 4,
    GreekGods = 5,
    USPresidents = 6,
    Scientists = 7,
    Insects = 8,
    Fruit = 9,
    Gemstones = 10,
    Vegetables = 11,

    Elements = 12,
    FamousQueens = 13,

    RomanGods = 14,
    EgyptianGods = 15,
}

impl LevelSequence {
    pub fn index(&self) -> usize {
        *self as usize
    }
    pub fn get_next(self) -> Option<Self> {
        let current_index = self as usize;
        let next_index = (current_index + 1) % Self::COUNT;

        FromPrimitive::from_usize(next_index)
    }

    pub fn get_level(self, index: usize) -> Option<&'static DesignedLevel> {
        let levels = self.levels();

        let index = index;
        levels.get(index)
    }

    pub fn level_count(self) -> usize {
        let levels = self.levels();
        levels.len()
    }

    fn levels(self) -> &'static Vec<DesignedLevel> {
        let levels = match self {
            LevelSequence::EUCapitals => &*EU_CAPITALS,
            LevelSequence::EUCountries => &*EU_COUNTRIES,
            LevelSequence::USStates => &*US_STATES,
            LevelSequence::GreekGods => &*GREEK_GODS,
            LevelSequence::USPresidents => &*US_PRESIDENTS,
            LevelSequence::Scientists => &*SCIENTISTS,
            LevelSequence::Insects => &*INSECTS,
            LevelSequence::Fruit => &*FRUIT,
            LevelSequence::Gemstones => &*GEMSTONES,
            LevelSequence::Vegetables => &*VEGETABLES,

            LevelSequence::Elements => &*ELEMENTS,
            LevelSequence::FamousQueens => &*QUEENS,
            LevelSequence::RomanGods => &*ROMAN_GODS,
            LevelSequence::EgyptianGods => &*EGYPTIAN_GODS,
        };
        levels
    }

    pub fn name(self) -> &'static str {
        match self {
            LevelSequence::EUCapitals => "EU Capitals",
            LevelSequence::EUCountries => "EU Countries",
            LevelSequence::USStates => "US States",
            LevelSequence::GreekGods => "Greek Gods",
            LevelSequence::USPresidents => "US Presidents",
            LevelSequence::Scientists => "Scientists",
            LevelSequence::Insects => "Insects",
            LevelSequence::Fruit => "Fruit",
            LevelSequence::Gemstones => "Gemstones",
            LevelSequence::Vegetables => "Vegetables",

            LevelSequence::FamousQueens => "Famous Queens",
            LevelSequence::RomanGods => "Roman Gods",
            LevelSequence::EgyptianGods => "Egyptian Gods",
            LevelSequence::Elements => "Elements",
        }
    }
}
