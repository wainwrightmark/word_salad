use std::ops::Add;

use crate::{
    asynchronous,
    prelude::*,
    startup::{self},
};
use bevy::prelude::*;
use chrono::{Datelike, Duration, Timelike};
use itertools::Itertools;
use nice_bevy_utils::{
    async_event_writer::AsyncEventWriter, CanInitTrackedResource, CanRegisterAsyncEvent,
    TrackableResource,
};
use serde::{Deserialize, Serialize};

pub struct DailyChallengePlugin;

impl Plugin for DailyChallengePlugin {
    fn build(&self, app: &mut App) {
        app.init_tracked_resource::<DailyChallenges>();
        app.register_async_event::<DailyChallengeDataLoadedEvent>();

        app.add_systems(PostStartup, load_levels);
        app.add_systems(
            Update,
            handle_daily_challenge_data_loaded
                .run_if(|ev: EventReader<DailyChallengeDataLoadedEvent>| !ev.is_empty()),
        );
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Resource, Default, MavericContext)]
pub struct DailyChallenges {
    level_data: String,
    #[serde(skip)]
    pub levels: Vec<DesignedLevel>,
} //todo make the default be something

impl TrackableResource for DailyChallenges {
    const KEY: &'static str = "DailyChallenges";

    fn on_loaded(&mut self) {
        //let now = chrono::Utc::now();
        self.levels = self
            .level_data
            .lines()
            .map(|x| {
                DesignedLevel::from_tsv_line(x).unwrap_or_else(|err| {
                    error!("{err}");
                    DesignedLevel::unknown()
                })
            })
            .collect();

        for (index, level) in self.levels.iter_mut().enumerate() {
            level.numbering = Some(Numbering::WordSaladNumber(index + 1));
        }
    }
}

pub const DAYS_OFFSET: i32 = 738865;

#[derive(Debug, PartialEq, Event, Clone)]
pub struct DailyChallengeDataLoadedEvent {
    pub data: String,
}

impl DailyChallenges {
    const OFFSET_HOURS: i64 = -6;

    pub fn get_today_index() -> usize {
        //todo make u16
        let today = get_today_date();
        usize::try_from(today.num_days_from_ce() - DAYS_OFFSET)
            .ok()
            .unwrap_or_default()
    }

    fn time_until_next_challenge(today_index: usize) -> Option<std::time::Duration> {
        if Self::get_today_index() > today_index {
            return None;
        }

        let today = chrono::offset::Utc::now();
        let today_eastern = today.add(Duration::hours(Self::OFFSET_HOURS));

        let secs_today: u32 =
            (today_eastern.hour() * 3600) + (today_eastern.minute() * 60) + today_eastern.second();

        let remaining: u32 = 86400u32.checked_sub(secs_today)?;

        Some(std::time::Duration::new(remaining as u64, 0))
    }

    pub fn time_until_challenge_string(today_index: usize) -> Option<String> {
        let remaining = DailyChallenges::time_until_next_challenge(today_index)?.as_secs();

        let secs = remaining % 60;
        let minutes = (remaining / 60) % 60;
        let hours = remaining / 3600;

        Some(format!("{hours:02}:{minutes:02}:{secs:02}"))
    }

    pub fn total_levels(&self) -> usize {
        (Self::get_today_index().saturating_add(1)).min(self.levels.len())
    }
}

fn get_today_date() -> chrono::NaiveDate {
    let today = chrono::offset::Utc::now();
    let today_eastern = today.add(Duration::hours(DailyChallenges::OFFSET_HOURS));
    today_eastern.date_naive()
}

fn load_levels(writer: AsyncEventWriter<DailyChallengeDataLoadedEvent>, dc: Res<DailyChallenges>) {
    if DailyChallenges::get_today_index() < dc.total_levels() {
        return;
    }

    asynchronous::spawn_and_run(load_levels_async(writer, false));
}

pub async fn load_levels_async(
    writer: AsyncEventWriter<DailyChallengeDataLoadedEvent>,
    user_requested: bool,
) {
    info!("Loading levels");
    let url = "https://wordsalad.online/daily.tsv".to_string();

    let res = reqwest::get(url).await;
    let data = match res {
        Ok(response) => response.text().await,
        Err(err) => {
            if user_requested {
                crate::platform_specific::show_toast_async("Could not load daily challenges").await;
            }

            error!("{err}");
            return;
        }
    };

    let data = match data {
        Ok(data) => data,
        Err(err) => {
            if user_requested {
                crate::platform_specific::show_toast_async("Could not load daily challenges").await;
            }

            error!("{err}");
            return;
        }
    };

    startup::ADDITIONAL_TRACKING.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

    match writer
        .send_async(DailyChallengeDataLoadedEvent { data })
        .await
    {
        Ok(()) => {}
        Err(err) => {
            error!("{err}");
        }
    }
}

fn handle_daily_challenge_data_loaded(
    mut daily_challenges: ResMut<DailyChallenges>,
    mut ev: EventReader<DailyChallengeDataLoadedEvent>,
    current_level: Res<CurrentLevel>,
    mut change_level_events: EventWriter<ChangeLevelEvent>,
) {
    for event in ev.read() {
        //info!("Daily challenge data loaded '{}'", event.data);
        let mut levels = event
            .data
            .lines()
            .map(DesignedLevel::from_tsv_line)
            .flat_map(|x| x.ok())
            .collect_vec();

        for (index, level) in levels.iter_mut().enumerate() {
            level.numbering = Some(Numbering::WordSaladNumber(index + 1));
        }

        if levels.len() > daily_challenges.levels.len() {
            info!(
                "Downloaded {} levels (previously had {})",
                levels.len(),
                daily_challenges.levels.len()
            );
            daily_challenges.level_data = event.data.clone();
            daily_challenges.levels = levels;

            let should_change_to: Option<usize> = match current_level.as_ref() {
                CurrentLevel::NonLevel(NonLevel::DailyChallengeFinished) => {
                    let index = DailyChallenges::get_today_index();
                    Some(index)
                }
                CurrentLevel::NonLevel(
                    NonLevel::DailyChallengeLoading { goto_level }
                    | NonLevel::DailyChallengeNotLoaded { goto_level },
                ) => Some(*goto_level),
                _ => None,
            };

            if let Some(index) = should_change_to {
                let new_current_level = CurrentLevel::DailyChallenge { index };

                if new_current_level.level(daily_challenges.as_mut()).is_left() {
                    change_level_events.send(new_current_level.into());
                }
            }
        } else if levels.len() < daily_challenges.levels.len() {
            warn!(
                "Downloaded {} levels (but previously had {})",
                levels.len(),
                daily_challenges.levels.len()
            );
        }
    }
}