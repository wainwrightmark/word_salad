use crate::prelude::*;
use bevy::{prelude::*, utils::HashSet};
use nice_bevy_utils::{CanInitTrackedResource, TrackableResource};
use serde::{Deserialize, Serialize};
use ws_levels::level_group::LevelGroup;

pub struct PurchasesPlugin;

impl Plugin for PurchasesPlugin {
    fn build(&self, app: &mut App) {
        app.init_tracked_resource::<Purchases>();
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize, MavericContext, Resource, Clone, Default)]
pub struct Purchases {
    /// Level groups which the user has purchased
    pub groups_purchased: HashSet<LevelGroup>,
    /// True is the user has purchased the pack to avoid ads
    pub avoid_ads_purchased: bool
}

impl TrackableResource for Purchases {
    const KEY: &'static str = "Purchases";
}
