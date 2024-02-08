use crate::prelude::*;
use bevy::{prelude::*, utils::HashSet};
use nice_bevy_utils::{CanInitTrackedResource, TrackableResource};
use serde::{Deserialize, Serialize};
use ws_levels::{level_group::LevelGroup, level_sequence::LevelSequence};

pub struct PurchasesPlugin;

impl Plugin for PurchasesPlugin {
    fn build(&self, app: &mut App) {
        app.init_tracked_resource::<Purchases>();
        app.add_event::<PurchaseEvent>();

        app.add_systems(
            Update,
            track_purchase_events.run_if(|x: EventReader<PurchaseEvent>| !x.is_empty()),
        );
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize, MavericContext, Resource, Clone, Default)]
pub struct Purchases {
    /// Level groups which the user has purchased
    pub groups_purchased: HashSet<LevelGroup>,
    /// True is the user has purchased the pack to avoid ads
    pub avoid_ads_purchased: bool,
}

impl TrackableResource for Purchases {
    const KEY: &'static str = "Purchases";
}

fn track_purchase_events(
    mut ev: EventReader<PurchaseEvent>,
    mut purchases: ResMut<Purchases>,
    mut hints: ResMut<HintState>,
    sequence_completion: Res<SequenceCompletion>,
    mut change_level_events: EventWriter<ChangeLevelEvent>,
    mut hint_events: EventWriter<HintEvent>
) {
    for event in ev.read() {
        match event {
            PurchaseEvent::BuyHintsPack1(hint_event) => {
                hints.hints_remaining += 100;
                hints.total_bought_hints += 100;
                show_toast_on_web("In the real app you would pay money");
                hint_events.send(*hint_event);
            }
            PurchaseEvent::BuyHintsPack2(hint_event) => {
                hints.hints_remaining += 1000;
                hints.total_bought_hints += 1000;
                show_toast_on_web("In the real app you would pay money");
                hint_events.send(*hint_event);
            }
            PurchaseEvent::BuyLevelGroup(sequence) => {
                purchases.groups_purchased.insert(sequence.group());
                show_toast_on_web("In the real app you would pay money");

                let level: CurrentLevel = sequence_completion
                    .get_next_level_index(*sequence, &purchases)
                    .to_level(*sequence);

                change_level_events.send(level.into());
            }
            PurchaseEvent::BuyAvoidAds => {
                purchases.avoid_ads_purchased = true;
                show_toast_on_web("In the real app you would pay money");
            }
        }
    }
}

#[derive(Debug, Event, Clone, PartialEq)]
pub enum PurchaseEvent {
    BuyHintsPack1(HintEvent),
    BuyHintsPack2(HintEvent),
    BuyLevelGroup(LevelSequence),
    BuyAvoidAds,
}
