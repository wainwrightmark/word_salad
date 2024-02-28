use crate::{prelude::*, startup};
use bevy::prelude::*;
use chrono::{DateTime, Utc};
use nice_bevy_utils::{CanInitTrackedResource, TrackableResource};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use strum::EnumIs;

pub struct LevelTimePlugin;

impl Plugin for LevelTimePlugin {
    fn build(&self, app: &mut App) {
        app.init_tracked_resource::<LevelTime>();
        app.add_systems(PostUpdate, manage_timer);
        app.add_systems(
            Update,
            count_up.run_if(|timer: Res<LevelTime>| timer.is_running()),
        );
        app.add_systems(
            Update,
            count_down.run_if(|level: Res<CurrentLevel>| {
                matches!(
                    level.as_ref(),
                    CurrentLevel::NonLevel(NonLevel::DailyChallengeCountdown { .. },)
                )
            }),
        );
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

                elapsed.to_std().unwrap_or_default()
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
    found_words: Res<FoundWordsState>,
    menu_state: Res<MenuState>,
    chosen_state: Res<ChosenState>,
) {
    //a different system sets the time on level changed

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
            if !found_words.is_level_complete() {
                timer.resume_timer();
            }
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
    let elapsed = timer.total_elapsed();

    for mut text in query.iter_mut() {
        if let Some(section) = text.sections.first_mut() {
            let total_seconds = elapsed.as_secs();
            section.value = format_seconds(total_seconds);
        }
    }
}

fn count_down(
    mut query: Query<&mut Text, With<NonLevelText>>,
    time: Res<Time>,
    mut elapsed: Local<Duration>,
    current_level: Res<CurrentLevel>,
    mut change_level_events: EventWriter<ChangeLevelEvent>,
) {
    let todays_index = match current_level.as_ref() {
        CurrentLevel::NonLevel(NonLevel::DailyChallengeCountdown { todays_index }) => todays_index,
        _ => {
            return;
        }
    };
    *elapsed += time.elapsed();

    if elapsed.as_secs() >= 1 {
        *elapsed = Duration::ZERO;

        if let Some(new_text) = DailyChallenges::time_until_challenge_string(*todays_index) {
            for mut text in query.iter_mut() {
                text.sections[0].value = new_text.clone();
            }
        } else {
            let index = DailyChallenges::get_today_index();
            change_level_events.send(CurrentLevel::DailyChallenge { index }.into());
        }
        startup::ADDITIONAL_TRACKING.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Component)]
pub struct TimeCounterMarker;

pub fn format_seconds(total_seconds: u64) -> String {
    let hh = total_seconds / 3600;
    let mm = (total_seconds / 60) % 60;
    let ss = total_seconds % 60;

    if hh > 0 {
        format!("{hh:02}:{mm:02}:{ss:02}")
    } else {
        format!("{mm:02}:{ss:02}")
    }
}
