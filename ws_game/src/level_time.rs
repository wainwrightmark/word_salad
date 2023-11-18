use bevy::prelude::*;
use bevy_utils::{TrackableResource, CanInitTrackedResource};
use chrono::{DateTime, Utc};
use strum::EnumIs;
use crate::prelude::*;
use serde::{Serialize, Deserialize};

pub struct LevelTimePlugin;

impl Plugin for LevelTimePlugin{
    fn build(&self, app: &mut App) {
        app.init_tracked_resource::<LevelTime>();
        app.add_systems(Update, manage_timer);
    }
}

#[derive(Debug, PartialEq, Clone, Resource, Serialize, Deserialize, EnumIs)]
pub enum LevelTime{
    Started(DateTime<Utc>),
    Finished{
        total_seconds: u64
    },

}

impl TrackableResource for LevelTime{
    const KEY: &'static str = "Timer";
}

impl Default for LevelTime{
    fn default() -> Self {
        let time = chrono::Utc::now();
        LevelTime::Started(time)
    }
}

fn manage_timer(mut timer: ResMut<LevelTime>, current_level: Res<CurrentLevel>, found_words: Res<FoundWordsState>, ){
    if current_level.is_changed(){
        *timer.as_mut() = LevelTime::default();

        info!("{timer:?}");
    }

    match timer.as_ref(){
        LevelTime::Started(started) => {
            if  found_words.is_changed() && found_words.is_level_complete(&current_level){
                let now =  chrono::Utc::now();

                info!("{now:?}");
                let diff = now.signed_duration_since(started);
                let total_seconds = diff.num_seconds().max(0) as u64;

                *timer.as_mut() = LevelTime::Finished { total_seconds };

                info!("{timer:?}");

            }
        },
        LevelTime::Finished { .. } => {

        },
    }


}

