use nice_bevy_utils::TrackableResource;
use serde::{Deserialize, Serialize};
use ws_levels::level_sequence::LevelSequence;

use crate::{completion::TotalCompletion, prelude::*};

#[derive(Debug, Clone, Resource, PartialEq, Eq, Serialize, Deserialize)]
pub enum CurrentLevel {
    Fixed {
        level_index: usize,
        sequence: LevelSequence,
    },
    Custom(DesignedLevel), //todo more sophisticated pointer
}

impl TrackableResource for CurrentLevel {
    const KEY: &'static str = "CurrentLevel";
}

impl Default for CurrentLevel {
    fn default() -> Self {
        Self::Fixed {
            level_index: 0,
            sequence: LevelSequence::Tutorial,
        }
    }
}

impl CurrentLevel {
    pub fn level(&self) -> &DesignedLevel {
        match self {
            CurrentLevel::Fixed {
                level_index,
                sequence,
            } => sequence.get_level(*level_index),
            CurrentLevel::Custom(level) => level,
        }
    }

    pub fn to_level(
        &mut self,
        sequence: LevelSequence,
        total_completion: &TotalCompletion,
        found_words: &mut FoundWordsState,
        chosen_state: &mut ChosenState,
    ) {
        let level_index = total_completion.next_level(sequence);
        *self = CurrentLevel::Fixed {
            level_index,
            sequence,
        };
        *found_words = FoundWordsState::new_from_level(&self);
        *chosen_state = ChosenState::default();
    }

    pub fn to_next_level(
        &mut self,
        total_completion: &TotalCompletion,
        found_words: &mut FoundWordsState,
        chosen_state: &mut ChosenState,
    ) {
        let sequence = match *self {
            CurrentLevel::Fixed { sequence, .. } => sequence,
            CurrentLevel::Custom(_) => LevelSequence::DailyChallenge,
        };

        self.to_level(sequence, total_completion, found_words, chosen_state);
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
