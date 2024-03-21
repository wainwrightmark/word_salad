use bevy::{prelude::*, utils::HashMap};
use maveric::helpers::MavericContext;
use nice_bevy_utils::TrackableResource;
use serde::{Deserialize, Serialize};
use ws_levels::{level_group::LevelGroup, level_sequence::LevelSequence};

use crate::{
    ads_common::InterstitialProgressState,
    compatibility::SubmitScoreData,
    level_time::LevelTime,
    prelude::{CurrentLevel, DailyChallenges, FoundWordsState, NonLevel, Purchases, Streak},
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
                if cfg!(feature = "web") {
                    CurrentLevel::NonLevel(NonLevel::PleaseBuyTheGame)
                } else {
                    CurrentLevel::NonLevel(NonLevel::LevelSequenceMustPurchaseGroup(sequence))
                }
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
            NextDailyChallengeResult::TodayNotLoaded { .. } => None,
        }
    }

    pub fn level_index(&self) -> Option<usize> {
        match self {
            NextDailyChallengeResult::Level(index) => Some(*index),
            NextDailyChallengeResult::AllFinished => None,
            NextDailyChallengeResult::TodayNotLoaded { .. } => None,
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
                CurrentLevel::NonLevel(NonLevel::DailyChallengeNotLoaded { goto_level: index })
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
        self.results.contains_key(&index)
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

        if daily_challenges.levels().get(current_index).is_none() {
            //info!("Today not loaded {current_index} / {}", daily_challenges.levels.len());
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
        self.results.len()
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
    daily_challenges: Res<DailyChallenges>,
    mut ips: ResMut<InterstitialProgressState>,
    purchases: Res<Purchases>,
) {
    if !found_words.is_changed()
        || !found_words.is_level_complete()
        || found_words.word_completions.is_empty()
    {
        return;
    }

    let first_time: bool;

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
                first_time = true;
                completion.total_complete = number_complete;

                if completion.total_complete % 5 == 0 {
                    crate::platform_specific::request_review();
                }
            } else {
                first_time = false;
            }
        }

        CurrentLevel::Custom { .. } => {
            // No need to track custom level completion
            crate::platform_specific::request_review();
            first_time = false;
        }
        CurrentLevel::Tutorial { index } => {
            const TUTORIAL_LEVEL_COUNT: usize = 2;

            let number_complete = index + 1;

            let completion = &mut tutorial_completion.inner;
            completion.current_index = (completion.current_index + 1) % TUTORIAL_LEVEL_COUNT;
            if completion.total_complete < number_complete {
                first_time = true;
                completion.total_complete = number_complete;
            } else {
                first_time = false;
            }
        }

        CurrentLevel::DailyChallenge { index } => {
            if !daily_challenge_completion.results.contains_key(index) {
                first_time = true;
                // daily_challenge_completion
                //     .total_completion
                //     .grow(index.saturating_add(1));

                //daily_challenge_completion.total_completion.insert(*index);

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

                        #[cfg(feature = "web")]
                        {
                            crate::platform_specific::show_toast_on_web(
                                capacitor_bindings::toast::ShowOptions {
                                    text: "The full app has now been released on IOS and Android"
                                        .to_string(),
                                    duration: capacitor_bindings::toast::ToastDuration::Long,
                                    position: capacitor_bindings::toast::ToastPosition::Bottom,
                                },
                            );
                        }
                        #[cfg(not(feature = "web"))]
                        {
                            crate::platform_specific::request_review();
                        }
                    } else {
                        info!("Streak set to one");
                        streak.current = 1;
                    }

                    streak.last_completed = Some(index);
                    streak.longest = streak.current.max(streak.longest);
                }
            } else {
                first_time = false;
            }
            daily_challenge_completion.results.insert(
                *index,
                LevelResult {
                    seconds: level_time.total_elapsed().as_secs() as u32,
                    hints_used: found_words.hints_used as u32,
                },
            );
        }
        CurrentLevel::NonLevel(..) => first_time = false,
    }
    if !current_level.is_changed() {
        if current_level.count_for_interstitial_ads(&purchases) {
            ips.levels_without_interstitial += 1;
        }

        // if current level was changed, it was because we loaded the game
        let level = current_level.level(&daily_challenges);
        match level {
            itertools::Either::Left(designed_level) => {
                let seconds = level_time.total_elapsed().as_secs();
                let event = crate::logging::LoggableEvent::FinishGameLevel {
                    level: designed_level.full_name().to_string(),
                    seconds,
                    hints_used: found_words.hints_used,
                    word_order: found_words.completed_words_ordered(designed_level),
                    first_time,
                };
                event.try_log1();
            }
            itertools::Either::Right(..) => {}
        }
    }
}

#[cfg(test)]
pub mod test {

    use crate::prelude::*;

    #[test]
    pub fn test_daily_challenge_completion_serde() {
        let old_form = r#"{"results":{"16":{"seconds":12,"hints_used":0},"18":{"seconds":29,"hints_used":0},"17":{"seconds":11,"hints_used":0},"19":{"seconds":15,"hints_used":0},"15":{"seconds":24,"hints_used":0}},"total_completion":{"data":[1015808],"length":20}}"#;
        let new_form = r#"{"results":{"19":{"seconds":15,"hints_used":0},"17":{"seconds":11,"hints_used":0},"18":{"seconds":29,"hints_used":0},"15":{"seconds":24,"hints_used":0},"16":{"seconds":12,"hints_used":0}}}"#;

        let old_form_completion: DailyChallengeCompletion =
            serde_json::from_str(old_form).expect("Should be able to deserialize old form");

        assert_eq!(5, old_form_completion.get_daily_challenges_complete());

        let new_form_completion: DailyChallengeCompletion =
            serde_json::from_str(new_form).expect("Should be able to deserialize new form");

        assert_eq!(old_form_completion, new_form_completion);

        let serialized = serde_json::to_string(&new_form_completion)
            .expect("Should be able to serialize new form");

        let restored: DailyChallengeCompletion =
            serde_json::from_str(&serialized).expect("Should be able to round trip");

        assert_eq!(restored, new_form_completion);
    }

    #[test]
    pub fn test_daily_challenge_completion() {
        let mut completion = DailyChallengeCompletion::default();
        let mut daily_challenges = DailyChallenges::default();

        let levels = vec![DesignedLevel::unknown(); 4];
        daily_challenges.levels = Some(levels);

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
