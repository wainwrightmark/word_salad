use chrono::{DateTime, Utc};
use itertools::Either;
use nice_bevy_utils::TrackableResource;
use serde::{Deserialize, Serialize};
use std::sync::RwLock;
use strum::EnumIs;
use ws_core::level_type::LevelType;
use ws_levels::{all_levels::get_tutorial_level, level_sequence::LevelSequence};

use crate::{
    completion::{DailyChallengeCompletion, SequenceCompletion},
    prelude::*,
};

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
    Custom {
        name: Ustr,
    },
    NonLevel(NonLevel),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, EnumIs)]
pub enum NonLevel {
    BeforeTutorial,
    AfterCustomLevel,
    DailyChallengeFinished,
    DailyChallengeNotLoaded {
        goto_level: usize,
    },
    DailyChallengeLoading {
        goto_level: usize,
    },
    DailyChallengeReset,
    DailyChallengeCountdown {
        todays_index: usize,
    }, //TODO remove

    PleaseBuyTheGame,
    LevelSequenceMustPurchaseGroup(LevelSequence),
    LevelSequenceAllFinished(LevelSequence),
    LevelSequenceReset(LevelSequence),

    AdBreak(NextLevel),
    AdFailed {
        next_level: NextLevel,
        since: Option<DateTime<Utc>>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, EnumIs)]
pub enum NextLevel {
    DailyChallenge {
        index: usize,
    },
    LevelSequence {
        sequence: LevelSequence,
        level_index: usize,
    },
}

impl From<NextLevel> for CurrentLevel {
    fn from(value: NextLevel) -> Self {
        match value {
            NextLevel::DailyChallenge { index } => CurrentLevel::DailyChallenge { index },
            NextLevel::LevelSequence {
                sequence,
                level_index,
            } => CurrentLevel::Fixed {
                level_index,
                sequence,
            },
        }
    }
}

impl<'a> TryFrom<&'a CurrentLevel> for NextLevel {
    type Error = ();

    fn try_from(value: &'a CurrentLevel) -> Result<Self, Self::Error> {
        match value {
            CurrentLevel::Tutorial { .. } => Err(()),
            CurrentLevel::Fixed {
                level_index,
                sequence,
            } => Ok(NextLevel::LevelSequence {
                sequence: *sequence,
                level_index: *level_index,
            }),
            CurrentLevel::DailyChallenge { index } => {
                Ok(NextLevel::DailyChallenge { index: *index })
            }
            CurrentLevel::Custom { .. } => Err(()),
            CurrentLevel::NonLevel(_) => Err(()),
        }
    }
}

impl From<NonLevel> for CurrentLevel {
    fn from(val: NonLevel) -> Self {
        CurrentLevel::NonLevel(val)
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

pub static CUSTOM_LEVEL: RwLock<Option<&'static DesignedLevel>> = RwLock::new(None);

pub fn set_custom_level(level: DesignedLevel) {
    match CUSTOM_LEVEL.write() {
        Ok(mut write) => {
            let leaked_level: &'static DesignedLevel = Box::leak(Box::new(level));
            write.replace(leaked_level);
        }
        Err(..) => {
            error!("Poison error writing custom level")
        }
    }
}

impl CurrentLevel {
    /// Returns true if hints used on this level should be deducted from the users remaining hints
    pub fn should_spend_hints(&self) -> bool {
        match self {
            CurrentLevel::Tutorial { .. } => false,
            CurrentLevel::Fixed { .. } => true,
            CurrentLevel::DailyChallenge { .. } => true,
            CurrentLevel::Custom { .. } => true,
            CurrentLevel::NonLevel(_) => false,
        }
    }

    /// Whether this level should be counted to towards total interstitial ads
    pub fn count_for_interstitial_ads(&self, purchases: &Purchases) -> bool {
        if purchases.remove_ads_purchased {
            return false;
        }

        match self {
            CurrentLevel::Tutorial { .. } => false,
            CurrentLevel::Fixed { sequence, .. } => {
                !purchases.groups_purchased.contains(&sequence.group())
            }
            CurrentLevel::DailyChallenge { .. } => true,
            CurrentLevel::Custom { .. } => false,
            CurrentLevel::NonLevel(_) => false,
        }
    }

    pub fn level_type(&self) -> LevelType {
        match self {
            CurrentLevel::Tutorial { .. } => LevelType::Tutorial,
            CurrentLevel::Fixed { .. } => LevelType::Fixed,
            CurrentLevel::DailyChallenge { .. } => LevelType::DailyChallenge,
            CurrentLevel::Custom { .. } => LevelType::Custom,
            CurrentLevel::NonLevel(_) => LevelType::NonLevel,
        }
    }
    pub fn level<'c>(
        &self,
        daily_challenges: &'c DailyChallenges,
    ) -> Either<&'c DesignedLevel, NonLevel> {
        match self {
            CurrentLevel::Fixed {
                level_index,
                sequence,
            } => match sequence.get_level(*level_index) {
                Some(level) => Either::Left(level),
                None => Either::Right(NonLevel::LevelSequenceAllFinished(*sequence)),
            },
            CurrentLevel::Custom { .. } => match CUSTOM_LEVEL.read() {
                Ok(opt) => match opt.as_ref() {
                    Some(cl) => Either::Left(cl),
                    None => Either::Right(NonLevel::AfterCustomLevel),
                },
                Err(_) => Either::Right(NonLevel::AfterCustomLevel),
            },
            CurrentLevel::Tutorial { index } => match get_tutorial_level(*index) {
                Some(cl) => Either::Left(cl),
                None => Either::Right(NonLevel::BeforeTutorial),
            },
            CurrentLevel::DailyChallenge { index } => match daily_challenges.levels().get(*index) {
                Some(cl) => Either::Left(cl),
                None => Either::Right(NonLevel::DailyChallengeNotLoaded { goto_level: *index }),
            },
            CurrentLevel::NonLevel(nl) => Either::Right(*nl),
        }
    }

    pub fn get_next_level(
        &self,
        daily_challenge_completion: &DailyChallengeCompletion,
        sequence_completion: &SequenceCompletion,
        purchases: &Purchases,
        daily_challenges: &DailyChallenges,
    ) -> CurrentLevel {
        match self {
            CurrentLevel::Tutorial { index } => {
                let index = index.saturating_add(1);
                match get_tutorial_level(index) {
                    Some(_) => CurrentLevel::Tutorial { index },
                    None => daily_challenge_completion
                        .get_next_incomplete_daily_challenge_from_today(daily_challenges)
                        .into(),
                }
            }
            CurrentLevel::Fixed {
                level_index: _,
                sequence,
            } => sequence_completion
                .get_next_level_index(*sequence, purchases)
                .to_level(*sequence),
            CurrentLevel::DailyChallenge { .. } => daily_challenge_completion
                .get_next_incomplete_daily_challenge_from_today(daily_challenges)
                .into(),
            CurrentLevel::Custom { .. } => NonLevel::AfterCustomLevel.into(),
            CurrentLevel::NonLevel(x) => (*x).into(), //No change
        }
    }
}
