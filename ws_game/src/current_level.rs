
use nice_bevy_utils::TrackableResource;
use serde::{Deserialize, Serialize};
use ws_levels::level_sequence::LevelSequence;

use crate::prelude::*;

#[derive(Debug, Clone, Resource, PartialEq, Eq, Serialize, Deserialize)]
pub enum CurrentLevel {
    Fixed { level_index: usize, sequence: LevelSequence },
    Custom(DesignedLevel), //todo more sophisticated pointer
}

impl TrackableResource for CurrentLevel {
    const KEY: &'static str = "CurrentLevel";
}

impl Default for CurrentLevel {
    fn default() -> Self {
        Self::Fixed { level_index: 0, sequence: LevelSequence::Tutorial }
    }
}

impl CurrentLevel {
    pub fn level(&self) -> &DesignedLevel {
        match self {
            CurrentLevel::Fixed { level_index, sequence } => {
                sequence.get_level(*level_index)
            }
            CurrentLevel::Custom(level) => level,
        }
    }

    pub fn to_level(&mut self, found_words: &mut FoundWordsState, sequence: LevelSequence ){
        *self = CurrentLevel::Fixed {
            level_index: 0,
            sequence
        };
        *found_words = FoundWordsState::new_from_level(&self);
    }

    pub fn to_next_level(&mut self, found_words: &mut FoundWordsState) {
        let (next_index, sequence) = match *self {
            CurrentLevel::Fixed { level_index,sequence } => (level_index.saturating_add(1), sequence),
            CurrentLevel::Custom(_) => (0, LevelSequence::DailyChallenge),
        };

        *self = CurrentLevel::Fixed {
            level_index: next_index,
            sequence
        };
        *found_words = FoundWordsState::new_from_level(&self);
    }
}




// const LEVEL_LINES: &str = include_str!("levels.tsv");
// lazy_static! {
//     static ref LEVELS: Vec<DesignedLevel> = {
//         let lines = LEVEL_LINES.lines();

//         let r: Vec<DesignedLevel> = lines.map(DesignedLevel::from_tsv_line).map(|x|x.unwrap()) .collect();
//         r
//     };
// }

// pub fn level_name(index: u32) -> String {
//     let index = (index as usize) % LEVELS.len();

//     LEVELS[index].name.clone()
// }

// pub fn level_count() -> u32 {
//     LEVEL_LINES.len() as u32
// }
