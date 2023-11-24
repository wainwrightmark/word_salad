use serde::{Deserialize, Serialize};
use strum::{Display, EnumCount, EnumIs, EnumIter};

use crate::level_sequence::LevelSequence;

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
pub enum LevelGroup {
    GlobalLocation = 0,
    HistoryMythology = 1,
    Science = 2,
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
            LevelGroup::Science => &[LevelSequence::Scientists, LevelSequence::Insects],
        }
    }

    pub fn get_level_sequence(&self, index: usize) -> LevelSequence {
        let s = self.get_sequences();
        let index = index % s.len();
        s[index]
    }

    pub fn name(&self) -> &'static str {
        match self {
            LevelGroup::GlobalLocation => "Global Location",
            LevelGroup::HistoryMythology => "History / Mythology",
            LevelGroup::Science => "Science",
        }
    }
}
