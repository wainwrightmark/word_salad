use nice_bevy_utils::TrackableResource;
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;
use ws_levels::{all_levels::get_tutorial_level, level_sequence::LevelSequence};

use crate::{completion::TotalCompletion, prelude::*};

#[derive(Debug, Clone, Resource, PartialEq, Eq, Serialize, Deserialize, MavericContext)]
pub enum CurrentLevel {
    Tutorial {
        index: usize,
    },
    Fixed {
        level_index: usize,
        sequence: LevelSequence,
    },
    DailyChallenge {
        index: usize,
    },
    Custom,
    Unknown,
}

impl TrackableResource for CurrentLevel {
    const KEY: &'static str = "CurrentLevel";
}

impl Default for CurrentLevel {
    fn default() -> Self {
        Self::Tutorial { index: 0 }
    }
}

pub static CUSTOM_LEVEL: OnceLock<DesignedLevel> = OnceLock::new();

impl CurrentLevel {
    pub fn level<'c>(&self, daily_challenges: &'c DailyChallenges) -> Option<&'c DesignedLevel> {
        match self {
            CurrentLevel::Fixed {
                level_index,
                sequence,
            } => sequence.get_level(*level_index),
            CurrentLevel::Custom => CUSTOM_LEVEL.get(),
            CurrentLevel::Tutorial { index } => get_tutorial_level(*index),
            CurrentLevel::DailyChallenge { index } => daily_challenges.levels.get(*index),
            CurrentLevel::Unknown => None,
        }
    }

    pub fn get_next_level(&self, total_completion: &TotalCompletion) -> CurrentLevel {
        match self {
            CurrentLevel::Tutorial { index } => {
                let index = index.saturating_add(1);
                match get_tutorial_level(index) {
                    Some(_) => CurrentLevel::Tutorial { index },
                    None => match total_completion.get_next_incomplete_daily_challenge_from_today()
                    {
                        Some(index) => CurrentLevel::DailyChallenge { index },
                        None => CurrentLevel::Unknown,
                    },
                }
            }
            CurrentLevel::Fixed {
                level_index: _,
                sequence,
            } => {
                let mut sequence = Some(*sequence);

                while let Some(seq) = sequence {
                    if let Some(index) = total_completion.get_next_level_index(seq) {
                        if let Some(..) = seq.get_level(index) {
                            return Self::Fixed {
                                level_index: index,
                                sequence: seq,
                            };
                        }
                    }

                    sequence = seq.get_next();
                }
                CurrentLevel::Unknown
            }
            CurrentLevel::DailyChallenge { .. } => {
                match total_completion.get_next_incomplete_daily_challenge_from_today() {
                    Some(index) => CurrentLevel::DailyChallenge { index },
                    None => CurrentLevel::Unknown,
                }
            }
            CurrentLevel::Custom => CurrentLevel::Unknown,
            CurrentLevel::Unknown => CurrentLevel::Unknown,
        }
    }
}
