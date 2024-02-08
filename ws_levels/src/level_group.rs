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
    Geography,
    NaturalWorld,
}

impl LevelGroup {
    pub fn total_count(&self) -> usize {
        self.get_sequences().iter().map(|x| x.level_count()).sum()
    }

    pub fn get_sequences(&self) -> &'static [LevelSequence] {
        use LevelSequence::*;
        match self {
            LevelGroup::Geography => &[
                USStates,
                EuropeanCountries,
                SouthAndEastAsianCountries,
                MiddleEasternCountries,
                EuropeanCapitals,
                SouthAndEastAsianCapitals,
                MiddleEasternCapitals,
            ],

            LevelGroup::NaturalWorld => &[
                Mammals, Birds, Insects, Fruit, Vegetables, Gemstones, Elements,
            ],
        }
    }

    pub fn get_level_sequence(&self, index: usize) -> LevelSequence {
        let s = self.get_sequences();
        let index = index % s.len();
        s[index]
    }

    pub fn name(&self) -> &'static str {
        match self {
            LevelGroup::Geography => "Geography",
            LevelGroup::NaturalWorld => "Natural World",
        }
    }
}
