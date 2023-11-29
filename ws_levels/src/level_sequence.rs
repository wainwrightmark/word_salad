use crate::all_levels::*;
use num_derive::FromPrimitive;
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
    Tutorial = 0,
    DailyChallenge = 1,
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
}

impl LevelSequence {
    pub fn get_level(&self, index: usize) -> &DesignedLevel {
        let levels = self.levels();

        let index = index % levels.len();

        levels
            .get(index)
            .expect("All level sequences should have at least one level")
    }

    pub fn level_count(&self)-> usize{
        let levels = self.levels();
        levels.len()
    }

    fn levels(&self)-> &Vec<DesignedLevel>{
        let levels = match self {
            LevelSequence::Tutorial => &*TUTORIAL,
            LevelSequence::DailyChallenge => &*DAILY_CHALLENGE,
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
        };
        levels
    }

    pub fn name(&self) -> &'static str {
        match self {
            LevelSequence::Tutorial => "Tutorial",
            LevelSequence::DailyChallenge => "Word Salad",
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
        }
    }
}
