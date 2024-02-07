use itertools::Either;
use nice_bevy_utils::TrackableResource;
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;
use strum::EnumIs;
use ws_core::level_type::LevelType;
use ws_levels::{all_levels::get_tutorial_level, level_sequence::LevelSequence};

use crate::{
    completion::{DailyChallengeCompletion, SequenceCompletion},
    prelude::*,
    purchases::Purchases,
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
        name: String,
    },
    NonLevel(NonLevel),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, EnumIs)]
pub enum NonLevel {
    BeforeTutorial,
    AfterCustomLevel,
    DailyChallengeFinished,
    DailyChallengeNotLoaded,
    DailyChallengeLoading,
    DailyChallengeReset,
    DailyChallengeCountdown { todays_index: usize },

    LevelSequenceMustPurchaseGroup(LevelSequence),
    LevelSequenceAllFinished(LevelSequence),
    LevelSequenceReset(LevelSequence),
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

pub static CUSTOM_LEVEL: OnceLock<DesignedLevel> = OnceLock::new();

impl CurrentLevel {

    /// Returns true if hints used on this level should be deducted from the users remaining hints
    pub fn should_spend_hints(&self)-> bool{
        match self{
            CurrentLevel::Tutorial { .. } => false,
            CurrentLevel::Fixed { .. } => true,
            CurrentLevel::DailyChallenge { .. } => true,
            CurrentLevel::Custom { .. } => true,
            CurrentLevel::NonLevel(_) => false,
        }
    }

    /// Whether this level should be counted to towards total interstitial ads
    pub fn count_for_interstitial_ads(&self, purchases: &Purchases) -> bool {
        if purchases.avoid_ads_purchased {
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
            CurrentLevel::Custom { .. } => match CUSTOM_LEVEL.get() {
                Some(cl) => Either::Left(cl),
                None => Either::Right(NonLevel::AfterCustomLevel),
            },
            CurrentLevel::Tutorial { index } => match get_tutorial_level(*index) {
                Some(cl) => Either::Left(cl),
                None => Either::Right(NonLevel::BeforeTutorial),
            },
            CurrentLevel::DailyChallenge { index } => match daily_challenges.levels.get(*index) {
                Some(cl) => Either::Left(cl),
                None => Either::Right(NonLevel::DailyChallengeNotLoaded),
            },
            CurrentLevel::NonLevel(nl) => Either::Right(*nl),
        }
    }

    pub fn get_next_level(
        &self,
        daily_challenge_completion: &DailyChallengeCompletion,
        sequence_completion: &SequenceCompletion,
        purchases: &Purchases,
    ) -> CurrentLevel {
        match self {
            CurrentLevel::Tutorial { index } => {
                let index = index.saturating_add(1);
                match get_tutorial_level(index) {
                    Some(_) => CurrentLevel::Tutorial { index },
                    None => match daily_challenge_completion
                        .get_next_incomplete_daily_challenge_from_today()
                    {
                        Some(index) => CurrentLevel::DailyChallenge { index },
                        None => CurrentLevel::NonLevel(NonLevel::DailyChallengeFinished),
                    },
                }
            }
            CurrentLevel::Fixed {
                level_index: _,
                sequence,
            } => sequence_completion
                .get_next_level_index(*sequence, purchases)
                .to_level(*sequence),
            CurrentLevel::DailyChallenge { .. } => {
                match daily_challenge_completion.get_next_incomplete_daily_challenge_from_today() {
                    Some(index) => CurrentLevel::DailyChallenge { index },
                    None => CurrentLevel::NonLevel(NonLevel::DailyChallengeFinished),
                }
            }
            CurrentLevel::Custom { .. } => NonLevel::AfterCustomLevel.into(),
            CurrentLevel::NonLevel(x) => (*x).into(), //No change
        }
    }
}
