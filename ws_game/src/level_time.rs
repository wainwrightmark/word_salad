use crate::prelude::*;
use bevy::prelude::*;
use chrono::{DateTime, Utc};
use nice_bevy_utils::{CanInitTrackedResource, TrackableResource};
use serde::{Deserialize, Serialize};
use strum::EnumIs;

pub struct LevelTimePlugin;

impl Plugin for LevelTimePlugin {
    fn build(&self, app: &mut App) {
        app.init_tracked_resource::<LevelTime>();
        app.add_systems(Update, manage_timer);
        app.add_systems(Update, count_up);
    }
}

#[derive(Debug, PartialEq, Clone, Resource, Serialize, Deserialize, EnumIs)]
pub enum LevelTime {
    Started(DateTime<Utc>),
    Finished { total_seconds: u64 },
}

impl TrackableResource for LevelTime {
    const KEY: &'static str = "Timer";
}

impl Default for LevelTime {
    fn default() -> Self {
        let time = chrono::Utc::now();
        LevelTime::Started(time)
    }
}

fn manage_timer(
    mut timer: ResMut<LevelTime>,
    current_level: Res<CurrentLevel>,
    found_words: Res<FoundWordsState>,
) {
    if !current_level.is_added() && current_level.is_changed() {
        *timer.as_mut() = LevelTime::default();

        //debug!("{timer:?}");
    }

    match timer.as_ref() {
        LevelTime::Started(started) => {
            if found_words.is_changed() && found_words.is_level_complete(&current_level) {
                let now = chrono::Utc::now();

                //info!("{now:?}");
                let diff = now.signed_duration_since(started);
                let total_seconds = diff.num_seconds().max(0) as u64;

                *timer.as_mut() = LevelTime::Finished { total_seconds };

                //info!("{timer:?}");
            }
        }
        LevelTime::Finished { .. } => {}
    }
}

fn count_up(mut query: Query<&mut Text, With<TimeCounterMarker>>, timer: Res<LevelTime>) {
    let LevelTime::Started(started) = timer.as_ref() else {
        return;
    };

    for mut t in query.iter_mut() {
        if let Some(section) = t.sections.first_mut() {
            let now = chrono::Utc::now();
            let diff = now.signed_duration_since(started);
            let total_seconds = diff.num_seconds().max(0) as u64;

            section.value = format_seconds(total_seconds);
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Component)]
pub struct TimeCounterMarker;

pub fn format_seconds(total_seconds: u64) -> String {
    let hh = total_seconds / 3600;
    let mm = (total_seconds / 60) % 60;
    let ss = total_seconds % 60;

    let time_str = format!("{hh:02}:{mm:02}:{ss:02}");

    time_str
}
