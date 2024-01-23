use crate::all_levels::*;
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
)]

pub enum LevelSequence {
    USStates,
    EuropeanCountries,
    EuropeanCapitals,

    Mammals,
    Birds,
    Insects,
    Fruit,
    Vegetables,
    Gemstones,
    Elements,
}

impl LevelSequence {
    pub const FIRST: Self = LevelSequence::EuropeanCapitals;

    pub fn get_next(self) -> Option<Self> {
        use LevelSequence::*;
        let r = match self {
            USStates => EuropeanCountries,
            EuropeanCountries => EuropeanCapitals,
            EuropeanCapitals => Mammals,
            Mammals => Birds,
            Birds => Insects,
            Insects => Fruit,
            Fruit => Vegetables,
            Vegetables => Gemstones,
            Gemstones => Elements,
            Elements => return None,
        };
        return Some(r);
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
            LevelSequence::EuropeanCapitals => &*EU_CAPITALS,
            LevelSequence::EuropeanCountries => &*EU_COUNTRIES,
            LevelSequence::USStates => &*US_STATES,
            LevelSequence::Insects => &*INSECTS,
            LevelSequence::Fruit => &*FRUIT,
            LevelSequence::Gemstones => &*GEMSTONES,
            LevelSequence::Vegetables => &*VEGETABLES,

            LevelSequence::Elements => &*ELEMENTS,
            LevelSequence::Mammals => &*MAMMALS,
            LevelSequence::Birds => &*BIRDS,
        };
        levels
    }

    pub fn name(self) -> &'static str {
        match self {
            LevelSequence::EuropeanCapitals => "European Capitals",
            LevelSequence::EuropeanCountries => "European Countries",
            LevelSequence::USStates => "US States",
            LevelSequence::Insects => "Insects",
            LevelSequence::Gemstones => "Gemstones",
            LevelSequence::Vegetables => "Vegetables",
            LevelSequence::Elements => "Elements",
            LevelSequence::Mammals => "Mammals",
            LevelSequence::Birds => "Birds",
            LevelSequence::Fruit => "Fruit",
        }
    }
}
