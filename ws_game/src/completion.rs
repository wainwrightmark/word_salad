use bevy::prelude::*;
use nice_bevy_utils::TrackableResource;
use serde::{Deserialize, Serialize};
use ws_levels::level_sequence::LevelSequence;

use crate::{
    prelude::{CurrentLevel, FoundWordsState},
    state::HintState,
};

#[derive(Debug, PartialEq, Resource, Serialize, Deserialize, Default, Clone)]
pub struct TotalCompletion {
    completions: Vec<usize>,
}

impl TrackableResource for TotalCompletion {
    const KEY: &'static str = "TotalCompletion";
}

impl TotalCompletion {
    pub fn level_complete(
        total_completion: &mut ResMut<Self>,
        hints_state: &mut ResMut<HintState>,
        current_level: &CurrentLevel,
    ) {
        match current_level {
            CurrentLevel::Fixed {
                level_index,
                sequence,
            } => {
                let number_complete = level_index + 1;
                let sequence_index = *sequence as usize;
                while total_completion.completions.len() <= sequence_index {
                    total_completion.completions.push(0);
                }

                let completion = total_completion.completions[sequence_index];
                if completion < number_complete {
                    total_completion.completions[sequence_index] = number_complete;

                    if number_complete <= sequence.level_count()
                    {
                        hints_state.hints_remaining += 2;
                    }

                }
            }
            CurrentLevel::Custom(_) => {}
        }
    }

    pub fn next_level(&self, sequence: LevelSequence) -> usize {
        let index = sequence as usize;

        self.completions.get(index).cloned().unwrap_or_default()
    }
}

pub fn track_level_completion(
    mut total_completion: ResMut<TotalCompletion>,
    current_level: Res<CurrentLevel>,
    found_words: Res<FoundWordsState>,
    mut hints_state: ResMut<HintState>,
) {
    if !found_words.is_changed() || !found_words.is_level_complete() {
        return;
    }

    TotalCompletion::level_complete(
        &mut total_completion,
        &mut hints_state,
        current_level.as_ref(),
    );
}
