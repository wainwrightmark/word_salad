use itertools::Either;
use nice_bevy_utils::TrackableResource;
use serde::{Deserialize, Serialize};
use strum::EnumIs;
use std::sync::OnceLock;
use ws_levels::{all_levels::get_tutorial_level, level_sequence::LevelSequence};

use crate::{completion::TotalCompletion, prelude::*};

#[derive(Debug, Clone, Resource, PartialEq, Eq, Serialize, Deserialize, MavericContext, EnumIs)]
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
    Custom {name: String},
    NonLevel(NonLevel),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, EnumIs)]
pub enum NonLevel{
    BeforeTutorial,
    AfterCustomLevel,
    NoMoreDailyChallenge,
    NoMoreLevelSequence(LevelSequence)
}

impl Into<CurrentLevel> for NonLevel {
    fn into(self) -> CurrentLevel {
        CurrentLevel::NonLevel(self)
    }
}

impl TrackableResource for CurrentLevel {
    const KEY: &'static str = "CurrentLevel";
}

impl Default for CurrentLevel {
    fn default() -> Self {
        Self::NonLevel(NonLevel::BeforeTutorial)
    }
}

pub static CUSTOM_LEVEL: OnceLock<DesignedLevel> = OnceLock::new();

impl CurrentLevel {
    pub fn level<'c>(&self, daily_challenges: &'c DailyChallenges) -> Either<&'c DesignedLevel, NonLevel> {
        match self {
            CurrentLevel::Fixed {
                level_index,
                sequence,
            } => {
                match sequence.get_level(*level_index){
                    Some(level) => Either::Left(level),
                    None => Either::Right(NonLevel::NoMoreLevelSequence(*sequence)),
                }
            },
            CurrentLevel::Custom { .. } => {
                match  CUSTOM_LEVEL.get(){
                    Some(cl) => Either::Left(cl),
                    None => Either::Right(NonLevel::AfterCustomLevel),
                }
            },
            CurrentLevel::Tutorial { index } => {
                match get_tutorial_level(*index){
                    Some(cl) => Either::Left(cl),
                    None => Either::Right(NonLevel::BeforeTutorial),
                }
            },
            CurrentLevel::DailyChallenge { index } => {
                match daily_challenges.levels.get(*index){
                    Some(cl) => Either::Left(cl),
                    None => Either::Right(NonLevel::NoMoreDailyChallenge),
                }
            },
            CurrentLevel::NonLevel(nl) => Either::Right(*nl),
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
                        None => CurrentLevel::NonLevel(NonLevel::NoMoreDailyChallenge),
                    },
                }
            }
            CurrentLevel::Fixed {
                level_index: _,
                sequence,
            } => {

                if let Some(index) = total_completion.get_next_level_index(*sequence) {
                    if index > 0 {
                        if let Some(..) = sequence.get_level(index) {
                            return Self::Fixed {
                                level_index: index,
                                sequence: *sequence,
                            };
                        }
                    }

                }

                return NonLevel::NoMoreLevelSequence(*sequence).into();
            }
            CurrentLevel::DailyChallenge { .. } => {
                match total_completion.get_next_incomplete_daily_challenge_from_today() {
                    Some(index) => CurrentLevel::DailyChallenge { index },
                    None => CurrentLevel::NonLevel(NonLevel::NoMoreDailyChallenge),
                }
            }
            CurrentLevel::Custom{..} => NonLevel::AfterCustomLevel.into(),
            CurrentLevel::NonLevel(x) => (*x).into(),//No change

        }
    }
}
