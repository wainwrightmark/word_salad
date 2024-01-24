use crate::prelude::*;
use bevy::{prelude::*, utils::HashSet};
use nice_bevy_utils::{CanInitTrackedResource, TrackableResource};
use ws_levels::level_group::LevelGroup;
use serde::{Serialize, Deserialize};

pub struct PurchasesPlugin;

impl Plugin for PurchasesPlugin{
    fn build(&self, app: &mut App) {
        app.init_tracked_resource::<Purchases>();
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize, MavericContext, Resource, Clone, Default)]
pub struct Purchases{
    pub groups_purchased: HashSet<LevelGroup>
}

impl TrackableResource for Purchases{
    const KEY: &'static str = "Purchases";
}