use bevy::prelude::*;
use fixedbitset::FixedBitSet;
use maveric::helpers::MavericContext;
use nice_bevy_utils::TrackableResource;
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;
use ws_levels::{level_group::LevelGroup, level_sequence::LevelSequence};

use crate::{
    prelude::{CurrentLevel, DailyChallenges, FoundWordsState, Streak},
    state::HintState,
};

#[derive(Debug, PartialEq, Resource, Serialize, Deserialize, Default, Clone, MavericContext)]
pub struct TotalCompletion {
    //TODO think about splitting this up
    completions: Vec<LevelCompletion>,
    daily_challenge_completion: DailyChallengeCompletion,
    tutorial_completion: LevelCompletion,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Default, Clone)]
pub struct DailyChallengeCompletion {
    total_completion: FixedBitSet,
    current_completion: FixedBitSet,
}

#[derive(Debug, PartialEq, Resource, Serialize, Deserialize, Default, Clone)]
pub struct LevelCompletion {
    #[serde(rename = "t")]
    pub total_complete: usize,
    #[serde(rename = "s")]
    pub current_index: usize,
}

impl TrackableResource for TotalCompletion {
    const KEY: &'static str = "TotalCompletion";
}

impl TotalCompletion {
    pub fn get_next_level_sequence(
        &self,
        current: Option<LevelSequence>,
    ) -> Option<(LevelSequence, usize)> {
        let first_index = current.map(|x| x.index() + 1).unwrap_or_default();

        for sequence in LevelSequence::iter().filter(|x| x.index() >= first_index) {
            if let Some(index) = self.get_next_level_index(sequence) {
                return Some((sequence, index));
            }
        }
        None
    }

    pub fn reset_daily_challenge_completion(&mut self) {
        self.daily_challenge_completion.current_completion.clear();
    }
    pub fn restart_level_sequence_completion(&mut self, sequence: LevelSequence) {
        if let Some(lc) = self.completions.get_mut(sequence.index()) {
            lc.current_index = 0;
        }
    }

    pub fn is_daily_challenge_complete(&self, index: usize) -> bool {
        self.daily_challenge_completion
            .total_completion
            .contains(index)
    }

    pub fn level_complete(
        total_completion: &mut ResMut<Self>,
        hints_state: &mut ResMut<HintState>,
        current_level: &CurrentLevel,
        streak: &mut ResMut<Streak>,
    ) {
        match current_level {
            CurrentLevel::Fixed {
                level_index,
                sequence,
            } => {
                let number_complete = level_index + 1;
                let sequence_index = *sequence as usize;
                while total_completion.completions.len() <= sequence_index {
                    total_completion
                        .completions
                        .push(LevelCompletion::default());
                }

                let completion = &mut total_completion.completions[sequence_index];
                completion.current_index = (completion.current_index + 1) % sequence.level_count();
                if completion.total_complete < number_complete {
                    completion.total_complete = number_complete;

                    if number_complete <= sequence.level_count() {
                        hints_state.hints_remaining += 1;
                        hints_state.total_earned_hints += 1;
                    }
                }
            }

            CurrentLevel::Custom { .. } => {
                // No need to track custom level completion
            }
            CurrentLevel::Tutorial { index } => {
                const TUTORIAL_LEVEL_COUNT: usize = 2;

                let number_complete = index + 1;

                let completion = &mut total_completion.tutorial_completion;
                completion.current_index = (completion.current_index + 1) % TUTORIAL_LEVEL_COUNT;
                if completion.total_complete < number_complete {
                    completion.total_complete = number_complete;

                    if number_complete <= TUTORIAL_LEVEL_COUNT {
                        hints_state.hints_remaining += 1;
                        hints_state.total_earned_hints += 1;
                    }
                }
            }

            CurrentLevel::DailyChallenge { index } => {
                if !total_completion
                    .daily_challenge_completion
                    .total_completion
                    .contains(*index)
                {
                    total_completion
                        .daily_challenge_completion
                        .total_completion
                        .grow(index.saturating_add(1));
                    total_completion
                        .daily_challenge_completion
                        .total_completion
                        .insert(*index);
                    hints_state.hints_remaining += 1;
                    hints_state.total_earned_hints += 1;

                    if Some(*index) == DailyChallenges::get_today_index() {
                        if streak.last_completed == Some(*index){
                            warn!("Daily challenge completed for the first time again?");
                        }
                         else if streak.last_completed == index.checked_sub(1) {
                            info!("Streak increased by one");
                            streak.current += 1;
                        } else {
                            info!("Streak set to one");
                            streak.current = 1;
                        }

                        streak.last_completed = Some(*index);
                        streak.longest = streak.current.max(streak.longest);
                    }
                }
                total_completion
                    .daily_challenge_completion
                    .current_completion
                    .grow(index.saturating_add(1));
                total_completion
                    .daily_challenge_completion
                    .current_completion
                    .insert(*index);
            }
            CurrentLevel::NonLevel(..) => {}
        }
    }

    pub fn get_next_level_index(&self, sequence: LevelSequence) -> Option<usize> {
        let sequence_index = sequence as usize;

        let index = self
            .completions
            .get(sequence_index)
            .map(|x| x.current_index)
            .unwrap_or_default();

        if index >= sequence.level_count() {
            return None;
        }
        Some(index)
    }

    pub fn get_number_complete(&self, sequence: &LevelSequence) -> usize {
        let sequence_index = *sequence as usize;
        self.completions
            .get(sequence_index)
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

    pub fn get_next_incomplete_daily_challenge_from_today(&self) -> Option<usize> {
        let today_date_index = DailyChallenges::get_today_index()?;
        self.get_next_incomplete_daily_challenge(today_date_index)
    }

    pub fn get_next_incomplete_daily_challenge(&self, today_date_index: usize) -> Option<usize> {
        if !self
            .daily_challenge_completion
            .current_completion
            .contains(today_date_index)
        {
            return Some(today_date_index);
        }

        let mut set = self.daily_challenge_completion.current_completion.clone();
        set.toggle_range(..);

        set.ones().take(today_date_index).last()
    }

    pub fn get_daily_challenges_complete(&self) -> usize {
        self.daily_challenge_completion
            .total_completion
            .count_ones(..)
    }
}

pub fn track_level_completion(
    mut total_completion: ResMut<TotalCompletion>,
    current_level: Res<CurrentLevel>,
    found_words: Res<FoundWordsState>,
    mut hints_state: ResMut<HintState>,
    mut streak: ResMut<Streak>,
) {
    if !found_words.is_changed() || !found_words.is_level_complete() {
        return;
    }

    TotalCompletion::level_complete(
        &mut total_completion,
        &mut hints_state,
        current_level.as_ref(),
        &mut streak,
    );
}

#[cfg(test)]
pub mod test {
    use super::TotalCompletion;

    #[test]
    pub fn go() {
        let mut completion = TotalCompletion::default();
        completion
            .daily_challenge_completion
            .current_completion
            .grow(4);

        assert_eq!(Some(3), completion.get_next_incomplete_daily_challenge(3));

        completion
            .daily_challenge_completion
            .current_completion
            .set(3, true);
        completion
            .daily_challenge_completion
            .current_completion
            .set(2, true);

        assert_eq!(Some(1), completion.get_next_incomplete_daily_challenge(3));

        completion
            .daily_challenge_completion
            .current_completion
            .set(1, true);
        completion
            .daily_challenge_completion
            .current_completion
            .set(0, true);

        assert_eq!(None, completion.get_next_incomplete_daily_challenge(3));
    }
}
