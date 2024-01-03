use std::time::Duration;

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

#[derive(Debug, PartialEq, Clone, Resource, Serialize, Deserialize, EnumIs, MavericContext)]
pub enum LevelTime {
    Running {
        since: DateTime<Utc>,
        additional: Duration,
    },
    Paused {
        elapsed: Duration,
    },
    Finished {
        elapsed: Duration,
    },
}

impl LevelTime {
    pub fn total_elapsed(&self) -> std::time::Duration {
        match self {
            LevelTime::Running { since, additional } => {
                let now = chrono::Utc::now();

                let additional =
                    chrono::Duration::from_std(*additional).unwrap_or(chrono::Duration::zero());
                //info!("{now:?}");
                let elapsed = now.signed_duration_since(since) + additional;

                let elapsed = elapsed.to_std().unwrap_or_default();
                elapsed
            }
            LevelTime::Paused { elapsed } => *elapsed,
            LevelTime::Finished { elapsed } => *elapsed,
        }
    }

    pub fn pause_timer(&mut self) {
        *self = LevelTime::Paused {
            elapsed: self.total_elapsed(),
        }
    }

    pub fn resume_timer(&mut self) {
        *self = LevelTime::Running {
            since: chrono::Utc::now(),
            additional: self.total_elapsed(),
        }
    }
}

const FLUSH_SECONDS: i64 = 15;

impl TrackableResource for LevelTime {
    const KEY: &'static str = "Timer";

    fn on_loaded(&mut self) {
        if let LevelTime::Running { since, additional } = self {
            let now = chrono::Utc::now();
            let flush_time = chrono::Duration::seconds(FLUSH_SECONDS);
            if now.signed_duration_since(since) > flush_time {
                let new_additional = *additional + Duration::from_secs(FLUSH_SECONDS as u64);
                *self = LevelTime::Running {
                    since: now,
                    additional: new_additional,
                }
            } else {
            }
        }
    }
}

impl Default for LevelTime {
    fn default() -> Self {
        let since: DateTime<Utc> = chrono::Utc::now();
        LevelTime::Running {
            since,
            additional: Duration::ZERO,
        }
    }
}

fn manage_timer(
    mut timer: ResMut<LevelTime>,
    current_level: Res<CurrentLevel>,
    found_words: Res<FoundWordsState>,
    menu_state: Res<MenuState>,
    chosen_state: Res<ChosenState>,
) {
    if !current_level.is_added() && current_level.is_changed() {
        *timer.as_mut() = LevelTime::default();

        //debug!("{timer:?}");
    }

    if found_words.is_changed() {
        if found_words.is_level_complete() {
            *timer.as_mut() = LevelTime::Finished {
                elapsed: timer.total_elapsed(),
            };
        } else if timer.is_paused() {
            timer.resume_timer()
        }
    }

    if chosen_state.is_changed() && timer.is_paused() {
        timer.resume_timer()
    }

    if menu_state.is_changed() {
        if menu_state.is_closed() {
            timer.resume_timer();
        } else {
            timer.pause_timer();
        }
    }

    if let LevelTime::Running { since, .. } = timer.as_ref() {
        if chrono::Utc::now().signed_duration_since(since)
            >= chrono::Duration::seconds(FLUSH_SECONDS)
        {
            timer.resume_timer();
        }
    }
}

fn count_up(mut query: Query<&mut Text, With<TimeCounterMarker>>, timer: Res<LevelTime>) {
    if !timer.is_running() {
        return;
    }

    let elapsed = timer.total_elapsed();

    for mut t in query.iter_mut() {
        if let Some(section) = t.sections.first_mut() {
            let total_seconds = elapsed.as_secs();
            section.value = format_seconds(total_seconds);
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Component)]
pub struct TimeCounterMarker;

pub fn format_seconds(total_seconds: u64) -> String {
    if total_seconds >= 3600 {
        return "60:00".to_string();
    } else {
        let mm = (total_seconds / 60) % 60;
        let ss = total_seconds % 60;

        let time_str = format!("{mm:02}:{ss:02}");

        time_str
    }
}
