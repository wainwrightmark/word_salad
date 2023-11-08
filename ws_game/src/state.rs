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

#[derive(Debug, Clone, Resource, Default, Serialize, Deserialize)]
pub struct FoundWordsState {
    pub found: HashSet<CharsArray>,
    pub unneeded_tiles: GridSet,
}

impl TrackableResource for FoundWordsState {
    const KEY: &'static str = "FoundWOrds";
}

fn track_found_words(
    mut commands: Commands,
    mut chosen: ResMut<ChosenState>,
    level: Res<CurrentLevel>,
    level_data: Res<LazyLevelData>,
    mut found_words: ResMut<FoundWordsState>,
    asset_server: Res<AssetServer>,
) {
    if chosen.is_changed() {
        let grid = level.level().grid;
        let chars: CharsArray = chosen.0.iter().map(|t| grid[*t]).collect();

        if let Some(word) = level_data.words_map.get(&chars) {
            let is_first_time = !found_words.found.contains(&chars);

            if let Some(last_tile) = chosen.0.last() {
                crate::animated_solutions::animate_solution(
                    &mut commands,
                    *last_tile,
                    word,
                    is_first_time,
                    &asset_server,
                );
            }

            if is_first_time {
                found_words.found.insert(chars);

                found_words.unneeded_tiles =
                    level_data.calculate_unneeded_tiles(&found_words.found);

                if chosen
                    .0
                    .iter()
                    .any(|x| found_words.unneeded_tiles.get_bit(x))
                {
                    *chosen = ChosenState::default();
                }
            }
        }
    }
}
