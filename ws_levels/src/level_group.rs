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

    pub fn total_count(&self)-> usize{
        self.get_sequences().iter().map(|x|x.level_count()).sum()
    }

    pub fn get_sequences(&self) -> &'static [LevelSequence] {
        use LevelSequence::*;
        match self {
            LevelGroup::GlobalLocation => &[USStates, EUCapitals, EUCountries],
            LevelGroup::HistoryMythology => &[USPresidents, GreekGods, RomanGods, EgyptianGods, FamousQueens],
            LevelGroup::Science => &[Scientists, Insects, Fruit, Gemstones, Vegetables, Elements],
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
