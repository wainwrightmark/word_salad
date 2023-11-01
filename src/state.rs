use crate::prelude::*;
use bevy::utils::HashSet;
use bevy_utils::{CanInitTrackedResource, TrackableResource};
use serde::{Deserialize, Serialize};

pub struct StatePlugin;

impl Plugin for StatePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ChosenState>();
        app.init_tracked_resource::<CurrentLevel>();
        app.init_tracked_resource::<FoundWordsState>();

        app.add_systems(Update, track_found_words);
        // app.add_systems(Update, track_level_change);
    }
}

#[derive(Debug, Clone, Resource, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct ChosenState(pub Solution);

#[derive(Debug, Clone, Resource, PartialEq, Eq, Serialize, Deserialize)]
pub struct CurrentLevel {
    pub level_index: usize, //todo more sophisticated pointer
}

impl TrackableResource for CurrentLevel {
    const KEY: &'static str = "CurrentLevel";
}

#[derive(Debug, Clone, Resource, Default, Serialize, Deserialize)]
pub struct FoundWordsState {
    pub found: HashSet<CharsArray>,
}

impl TrackableResource for FoundWordsState {
    const KEY: &'static str = "FoundWOrds";
}

impl Default for CurrentLevel {
    fn default() -> Self {
        Self { level_index: 0 }
    }
}

fn track_found_words(
    chosen: Res<ChosenState>,
    level: Res<CurrentLevel>,
    mut found_words: ResMut<FoundWordsState>,
) {
    if chosen.is_changed() {
        let grid = level.level().grid;
        let chars: CharsArray = chosen.0.iter().map(|t| grid[*t]).collect();

        if level.level().words_set.contains(&chars) {
            if !found_words.found.contains(&chars) {
                found_words.found.insert(chars);
            }
        }
    }
}
