use crate::prelude::*;
use bevy::prelude::*;
use nice_bevy_utils::{CanInitTrackedResource, TrackableResource};
use serde::{Deserialize, Serialize};

pub struct StreakPlugin;

impl Plugin for StreakPlugin {
    fn build(&self, app: &mut App) {
        app.init_tracked_resource::<Streak>();
    }
}

#[derive(Debug, Clone, Resource, Serialize, Deserialize, MavericContext, PartialEq, Default)]
pub struct Streak {
    pub current: usize,
    pub longest: usize,
    pub last_completed: Option<usize>,
}

impl TrackableResource for Streak {
    const KEY: &'static str = "Streak";
}
