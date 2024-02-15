use bevy::{prelude::*, utils::HashMap};
use fixedbitset::FixedBitSet;
use maveric::helpers::MavericContext;
use nice_bevy_utils::TrackableResource;
use serde::{Deserialize, Serialize};
use ws_levels::{level_group::LevelGroup, level_sequence::LevelSequence};

use crate::{
    compatibility::SubmitScoreData,
    level_time::LevelTime,
    prelude::{CurrentLevel, DailyChallenges, FoundWordsState, NonLevel, Streak},
    purchases::Purchases,
};

#[derive(Debug, PartialEq, Serialize, Deserialize, Default, Clone, Resource, MavericContext)]
pub struct SequenceCompletion {
    pub completions: HashMap<LevelSequence, LevelCompletion>,
}

impl TrackableResource for SequenceCompletion {
    const KEY: &'static str = "SequenceCompletion";
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Default, Clone, Resource, MavericContext)]
pub struct TutorialCompletion {
    pub inner: LevelCompletion,
}

impl TrackableResource for TutorialCompletion {
    const KEY: &'static str = "TutorialCompletion";
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct LevelResult {
    pub seconds: u32,
    pub hints_used: u32,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Default, Clone, Resource, MavericContext)]
pub struct DailyChallengeCompletion {
    pub results: HashMap<usize, LevelResult>,
    total_completion: FixedBitSet,
}

impl TrackableResource for DailyChallengeCompletion {
    const KEY: &'static str = "DailyChallengeCompletion";
}
#[derive(Debug, PartialEq, Resource, Serialize, Deserialize, Default, Clone)]
pub struct LevelCompletion {
    #[serde(rename = "t")]
    pub total_complete: usize,
    #[serde(rename = "s")]
    pub current_index: usize,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NextLevelResult {
    Index(usize),
    MustPurchase,
    NoMoreLevels,
}

impl NextLevelResult {
    pub fn to_level(self, sequence: LevelSequence) -> CurrentLevel {
        match self {
            NextLevelResult::Index(index) => CurrentLevel::Fixed {
                level_index: index,
                sequence,
            },
            NextLevelResult::MustPurchase => {
                CurrentLevel::NonLevel(NonLevel::LevelSequenceMustPurchaseGroup(sequence))
            }
            NextLevelResult::NoMoreLevels => {
                CurrentLevel::NonLevel(NonLevel::LevelSequenceAllFinished(sequence))
            }
        }
    }
}

impl SequenceCompletion {
    pub fn get_next_level_sequence(
        &self,
        current: Option<LevelSequence>,
        purchases: &Purchases,
    ) -> Option<(LevelSequence, usize)> {
        let mut current = current;
        loop {
            let next = match current {
                Some(s) => s.get_next()?,
                None => LevelSequence::FIRST,
            };

            if let NextLevelResult::Index(index) = self.get_next_level_index(next, purchases) {
                return Some((next, index));
            }

            current = Some(next)
        }
    }

    pub fn restart_level_sequence_completion(&mut self, sequence: LevelSequence) {
        self.completions.entry(sequence).or_default().current_index = 0;
    }

    pub fn get_next_level_index(
        &self,
        sequence: LevelSequence,
        purchases: &Purchases,
    ) -> NextLevelResult {
        let index = self
            .completions
            .get(&sequence)
            .cloned()
            .unwrap_or_default()
            .current_index;

        if index >= sequence.level_count() {
            NextLevelResult::NoMoreLevels
        } else if index >= sequence.free_level_count()
            && !purchases.groups_purchased.contains(&sequence.group())
        {
            return NextLevelResult::MustPurchase;
        } else {
            NextLevelResult::Index(index)
        }
    }

    pub fn get_number_complete(&self, sequence: &LevelSequence) -> usize {
        self.completions
            .get(sequence)
            .map(|x| x.total_complete)
            .unwrap_or_default()
    }

    pub fn get_number_complete_group(&self, group: &LevelGroup) -> usize {
        group
            .get_sequences()
            .iter()
            .map(|x| self.get_number_complete(x))
            .sum()
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NextDailyChallengeResult {
    Level(usize),
    AllFinished,
    TodayNotLoaded(usize),
}

impl NextDailyChallengeResult {
    pub fn actual_level(&self) -> Option<CurrentLevel> {
        match self {
            NextDailyChallengeResult::Level(index) => {
                Some(CurrentLevel::DailyChallenge { index: *index })
            }
            NextDailyChallengeResult::AllFinished => None,
            NextDailyChallengeResult::TodayNotLoaded{..} => None,
        }
    }

    pub fn level_index(&self) -> Option<usize> {
        match self {
            NextDailyChallengeResult::Level(index) => Some(*index),
            NextDailyChallengeResult::AllFinished => None,
            NextDailyChallengeResult::TodayNotLoaded{..} => None,
        }
    }
}

impl From<NextDailyChallengeResult> for CurrentLevel {
    fn from(val: NextDailyChallengeResult) -> Self {
        match val {
            NextDailyChallengeResult::Level(index) => CurrentLevel::DailyChallenge { index },
            NextDailyChallengeResult::AllFinished => {
                CurrentLevel::NonLevel(NonLevel::DailyChallengeFinished)
            }
            NextDailyChallengeResult::TodayNotLoaded(index) => {
                CurrentLevel::NonLevel(NonLevel::DailyChallengeNotLoaded{goto_level: index})
            }
        }
    }
}

impl DailyChallengeCompletion {
    pub fn reset_daily_challenge_completion(&mut self) {
        self.results.clear();
        //keep total completions
    }

    pub fn is_daily_challenge_complete(&self, index: usize) -> bool {
        self.total_completion.contains(index)
    }

    pub fn get_next_incomplete_daily_challenge_from_today(
        &self,
        daily_challenges: &DailyChallenges,
    ) -> NextDailyChallengeResult {
        let today_date_index = DailyChallenges::get_today_index();
        self.get_next_incomplete_daily_challenge(today_date_index, daily_challenges)
    }

    pub fn get_next_incomplete_daily_challenge(
        &self,
        today_date_index: usize,
        daily_challenges: &DailyChallenges,
    ) -> NextDailyChallengeResult {
        let mut current_index = today_date_index;

        if daily_challenges.levels.get(current_index).is_none() {
            return NextDailyChallengeResult::TodayNotLoaded(current_index);
        }

        loop {
            if !self.results.contains_key(&current_index) {
                return NextDailyChallengeResult::Level(current_index);
            }
            match current_index.checked_sub(1) {
                Some(ci) => current_index = ci,
                None => return NextDailyChallengeResult::AllFinished,
            }
        }
    }

    pub fn get_daily_challenges_complete(&self) -> usize {
        self.total_completion.count_ones(..)
    }
}

pub fn track_level_completion(
    mut sequence_completion: ResMut<SequenceCompletion>,
    mut daily_challenge_completion: ResMut<DailyChallengeCompletion>,
    mut tutorial_completion: ResMut<TutorialCompletion>,
    current_level: Res<CurrentLevel>,
    found_words: Res<FoundWordsState>,
    mut streak: ResMut<Streak>,
    level_time: Res<LevelTime>,
) {
    if !found_words.is_changed()
        || !found_words.is_level_complete()
        || found_words.word_completions.is_empty()
    {
        return;
    }

    match current_level.as_ref() {
        CurrentLevel::Fixed {
            level_index,
            sequence,
        } => {
            let number_complete = level_index + 1;

            let completion = sequence_completion
                .completions
                .entry(*sequence)
                .or_default();
            completion.current_index += 1;
            if completion.total_complete < number_complete {
                completion.total_complete = number_complete;

                if completion.total_complete % 5 == 0 {
                    crate::platform_specific::request_review();
                }
            }
        }

        CurrentLevel::Custom { .. } => {
            // No need to track custom level completion
        }
        CurrentLevel::Tutorial { index } => {
            const TUTORIAL_LEVEL_COUNT: usize = 2;

            let number_complete = index + 1;

            let completion = &mut tutorial_completion.inner;
            completion.current_index = (completion.current_index + 1) % TUTORIAL_LEVEL_COUNT;
            if completion.total_complete < number_complete {
                completion.total_complete = number_complete;
            }
        }

        CurrentLevel::DailyChallenge { index } => {
            if !daily_challenge_completion.total_completion.contains(*index) {
                daily_challenge_completion
                    .total_completion
                    .grow(index.saturating_add(1));
                daily_challenge_completion.total_completion.insert(*index);

                let index = DailyChallenges::get_today_index();
                {
                    if streak.last_completed == Some(index) {
                    } else if streak.last_completed == index.checked_sub(1) {
                        info!("Streak increased by one");
                        streak.current += 1;

                        crate::platform_specific::submit_score(SubmitScoreData {
                            leaderboard_id: "Word_Salad_Daily_Challenge".to_string(),
                            total_score_amount: level_time.total_elapsed().as_secs() as i32,
                        });

                        crate::platform_specific::request_review();
                    } else {
                        info!("Streak set to one");
                        streak.current = 1;
                    }

                    streak.last_completed = Some(index);
                    streak.longest = streak.current.max(streak.longest);
                }
            }
            daily_challenge_completion.results.insert(
                *index,
                LevelResult {
                    seconds: level_time.total_elapsed().as_secs() as u32,
                    hints_used: found_words.hints_used as u32,
                },
            );
        }
        CurrentLevel::NonLevel(..) => {}
    }
}

#[cfg(test)]
pub mod test {
    use crate::prelude::*;

    #[test]
    pub fn test_daily_challenge_completion() {
        let mut completion = DailyChallengeCompletion::default();
        let mut daily_challenges = DailyChallenges::default();
        daily_challenges.levels.push(DesignedLevel::unknown());
        daily_challenges.levels.push(DesignedLevel::unknown());
        daily_challenges.levels.push(DesignedLevel::unknown());
        daily_challenges.levels.push(DesignedLevel::unknown());

        assert_eq!(
            NextDailyChallengeResult::Level(3),
            completion.get_next_incomplete_daily_challenge(3, &daily_challenges)
        );

        completion.results.insert(
            3,
            LevelResult {
                seconds: 0,
                hints_used: 0,
            },
        );
        completion.results.insert(
            2,
            LevelResult {
                seconds: 0,
                hints_used: 0,
            },
        );

        assert_eq!(
            NextDailyChallengeResult::Level(1),
            completion.get_next_incomplete_daily_challenge(3, &daily_challenges)
        );

        completion.results.insert(
            1,
            LevelResult {
                seconds: 0,
                hints_used: 0,
            },
        );
        completion.results.insert(
            0,
            LevelResult {
                seconds: 0,
                hints_used: 0,
            },
        );

        assert_eq!(
            NextDailyChallengeResult::AllFinished,
            completion.get_next_incomplete_daily_challenge(3, &daily_challenges)
        );
    }
}
