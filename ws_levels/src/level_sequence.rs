use crate::all_levels::*;
use num_derive::FromPrimitive;
use serde::{Deserialize, Serialize};
use strum::{Display, EnumCount, EnumIs, EnumIter, EnumMessage};
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
    EnumMessage,
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
    #[strum(message = "Tutorial")]
    Tutorial = 0,
    #[strum(message = "Word Salad")]
    DailyChallenge = 1,
    #[strum(message = "EU Capitals")]
    EUCapitals = 2,
    #[strum(message = "EU Countries")]
    EUCountries = 3,
    #[strum(message = "US States")]
    USStates = 4,
    #[strum(message = "Greek Gods")]
    GreekGods = 5,
    #[strum(message = "US Presidents")]
    USPresidents = 6,
}

impl LevelSequence {
    pub fn get_level(&self, index: usize) -> &DesignedLevel {
        let levels = match self {
            LevelSequence::Tutorial => &*TUTORIAL,
            LevelSequence::DailyChallenge => &*DAILY_CHALLENGE,
            LevelSequence::EUCapitals => &*EU_CAPITALS,
            LevelSequence::EUCountries => &*EU_COUNTRIES,
            LevelSequence::USStates => &*US_STATES,
            LevelSequence::GreekGods => &*GREEK_GODS,
            LevelSequence::USPresidents => &*US_PRESIDENTS,
        };

        let index = index % levels.len();

        levels
            .get(index)
            .expect("All level sequences should have at least one level")
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
        }
    }
}
