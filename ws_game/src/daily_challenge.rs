use crate::{prelude::*, asynchronous};
use bevy::prelude::*;
use chrono::Datelike;
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
        app.add_systems(Update, handle_daily_challenge_data_loaded);
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Resource, Default, MavericContext)]
pub struct DailyChallenges {
    pub levels: Vec<DesignedLevel>,
} //todo make the default be something

impl TrackableResource for DailyChallenges {
    const KEY: &'static str = "DailyChallenges";
}

pub const DAYS_OFFSET: i32 = 738865;

#[derive(Debug, PartialEq, Event)]
pub struct DailyChallengeDataLoadedEvent {
    pub data: String,
}

impl DailyChallenges{
    pub fn get_today_index() -> Option<usize> {
        let today = get_today_date();
        usize::try_from(today.num_days_from_ce() - DAYS_OFFSET).ok()
    }

    pub fn total_levels(&self)-> usize{
        (Self::get_today_index().unwrap_or_default().saturating_add(1)) .min(self.levels.len())
    }
}


fn get_today_date() -> chrono::NaiveDate {
    let today = chrono::offset::Utc::now();
    today.date_naive()
}

fn load_levels(writer: AsyncEventWriter<DailyChallengeDataLoadedEvent>){
    asynchronous::spawn_and_run(load_levels_async(writer));
}

async fn load_levels_async(writer: AsyncEventWriter<DailyChallengeDataLoadedEvent>) {
    info!("Loading levels");
    let url = "https://wordsalad.online/daily.tsv".to_string();

    let res = reqwest::get(url).await;
    let data = match res {
        Ok(response) => response.text().await,
        Err(err) => {
            error!("{err}");
            return;
        }
    };

    let data = match data {
        Ok(data) => data,
        Err(err) => {
            error!("{err}");
            return;
        }
    };

    match writer
        .send_async(DailyChallengeDataLoadedEvent { data })
        .await
    {
        Ok(()) => {}
        Err(err) => {
            error!("{err}");
            return;
        }
    }
}

fn handle_daily_challenge_data_loaded(
    mut challenges: ResMut<DailyChallenges>,
    mut ev: EventReader<DailyChallengeDataLoadedEvent>,
) {
    for event in ev.read() {
        info!("Daily challenge data loaded '{}'", event.data);
        let levels = event
            .data
            .lines()
            .map(DesignedLevel::from_tsv_line)
            .flat_map(|x| x.ok())
            .collect_vec();

        if levels.len() > challenges.levels.len() {
            info!(
                "Downloaded {} levels (previously had {})",
                levels.len(),
                challenges.levels.len()
            );
            challenges.levels = levels;
        } else if levels.len() < challenges.levels.len() {
            warn!(
                "Downloaded {} levels (but previously had {})",
                levels.len(),
                challenges.levels.len()
            );
        }
    }
}
