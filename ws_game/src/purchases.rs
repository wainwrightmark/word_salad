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
            PurchaseEvent::BuyHintsPack { hint_event, number_of_hints } => {
                hints.hints_remaining += number_of_hints;
                hints.total_bought_hints += number_of_hints;
                show_toast_on_web("In the real app you would pay money");
                if let Some(hint_event) = hint_event{
                    hint_events.send(*hint_event);
                }

            }
            PurchaseEvent::BuyLevelGroupBySequence(sequence) => {
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
            PurchaseEvent::BuyLevelGroup(lg) => {
                purchases.groups_purchased.insert(*lg);
                show_toast_on_web("In the real app you would pay money");
                let sequence = lg.get_sequences()[0];

                let level: CurrentLevel = sequence_completion
                    .get_next_level_index(sequence, &purchases)
                    .to_level(sequence);

                change_level_events.send(level.into());
            },
        }
    }
}

#[derive(Debug, Event, Clone, PartialEq)]
pub enum PurchaseEvent {
    BuyHintsPack{
        hint_event: Option<HintEvent>,
        number_of_hints: usize
    },
    BuyLevelGroupBySequence(LevelSequence),
    BuyLevelGroup(LevelGroup),
    BuyAvoidAds,
}
