use ws_common::{platform_specific, prelude::*};
// use bevy::{prelude::*, utils::HashSet};
// use nice_bevy_utils::{CanInitTrackedResource, CanRegisterAsyncEvent, TrackableResource};
// use serde::{Deserialize, Serialize};
// use strum::{Display, EnumString, EnumTable};
// use ws_levels::{level_group::LevelGroup, level_sequence::LevelSequence};

pub struct PurchasesPlugin;

impl Plugin for PurchasesPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            handle_request_purchase_events
                .run_if(|x: EventReader<RequestPurchaseEvent>| !x.is_empty()),
        );

        app.add_systems(
            Update,
            handle_refresh_and_restore
                .run_if(|ev: EventReader<RefreshAndRestoreEvent>| !ev.is_empty()),
        );
    }
}

#[allow(unused_variables)]
fn handle_refresh_and_restore(
    mut ev: EventReader<RefreshAndRestoreEvent>,
    get_products_writer: nice_bevy_utils::async_event_writer::AsyncEventWriter<UpdateProductsEvent>,
) {
    for event in ev.read() {
        platform_specific::show_toast_on_web("We would refresh prices and restore purchases here");
    }
}

fn handle_request_purchase_events(
    mut ev: EventReader<RequestPurchaseEvent>,
    mut product_purchased_event_writer_sync: EventWriter<ProductPurchasedEvent>,
) {
    for event in ev.read() {
        let product: Product = event.into();

        {
            show_toast_on_web("In the real app you would pay money");
            product_purchased_event_writer_sync.send(ProductPurchasedEvent { product });
        }
    }
}
