use lazy_static::lazy_static;
use nice_bevy_utils::TrackableResource;
use serde::{Deserialize, Serialize};

use crate::prelude::*;

#[derive(Debug, Clone, Resource, PartialEq, Eq, Serialize, Deserialize)]
pub enum CurrentLevel {
    Fixed { level_index: usize },
    Custom(DesignedLevel), //todo more sophisticated pointer
}

impl TrackableResource for CurrentLevel {
    const KEY: &'static str = "CurrentLevel";
}

impl Default for CurrentLevel {
    fn default() -> Self {
        Self::Fixed { level_index: 0 }
    }
}

impl CurrentLevel {
    pub fn level(&self) -> &DesignedLevel {
        match self {
            CurrentLevel::Fixed { level_index } => {
                let index = level_index % LEVELS.len();
                &LEVELS[index]
            }
            CurrentLevel::Custom(level) => level,
        }
    }

    pub fn to_next_level(&mut self, found_words: &mut FoundWordsState) {
        let next_index = match *self {
            CurrentLevel::Fixed { level_index } => level_index.saturating_add(1),
            CurrentLevel::Custom(_) => 0,
        };

        *self = CurrentLevel::Fixed {
            level_index: next_index,
        };
        *found_words = FoundWordsState::new_from_level(&self);
    }
}




const LEVEL_LINES: &str = include_str!("levels.tsv");
lazy_static! {
    static ref LEVELS: Vec<DesignedLevel> = {
        let lines = LEVEL_LINES.lines();

        let r: Vec<DesignedLevel> = lines.map(DesignedLevel::from_tsv_line).collect();
        r
    };
}

pub fn level_name(index: u32) -> String {
    let index = (index as usize) % LEVELS.len();

    LEVELS[index].name.clone()
}

pub fn level_count() -> u32 {
    LEVEL_LINES.len() as u32
}
