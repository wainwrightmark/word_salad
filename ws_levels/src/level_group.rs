use serde::{Deserialize, Serialize};
use strum::{Display, EnumCount, EnumIs, EnumIter, EnumMessage};

use crate::level_sequence::LevelSequence;

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
)]
pub enum LevelGroup {
    GlobalLocation = 0,
    HistoryMythology = 1,
}

impl LevelGroup {
    pub fn get_sequences(&self) -> &'static [LevelSequence] {
        match self {
            LevelGroup::GlobalLocation => &[
                LevelSequence::USStates,
                LevelSequence::EUCapitals,
                LevelSequence::EUCountries,
            ],
            LevelGroup::HistoryMythology => {
                &[LevelSequence::USPresidents, LevelSequence::GreekGods]
            }
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            LevelGroup::GlobalLocation => "Global Location",
            LevelGroup::HistoryMythology => "History / Mythology",
        }
    }
}
