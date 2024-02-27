use crate::{platform_specific, prelude::*};
use bevy::utils::HashSet;
use nice_bevy_utils::{CanInitTrackedResource, CanRegisterAsyncEvent, TrackableResource};
use serde::{Deserialize, Serialize};
use strum::{Display, EnumString, EnumTable};
use ws_levels::{level_group::LevelGroup, level_sequence::LevelSequence};

pub struct PurchaseCommonPlugin;

impl Plugin for PurchaseCommonPlugin {
    fn build(&self, app: &mut App) {
        app.init_tracked_resource::<Purchases>();
        app.init_resource::<Prices>();
        app.add_event::<RequestPurchaseEvent>();
        app.add_event::<RefreshAndRestoreEvent>();

        app.add_systems(
            Update,
            update_products.run_if(|ev: EventReader<UpdateProductsEvent>| !ev.is_empty()),
        );

        app.add_systems(
            Update,
            handle_product_purchased
                .run_if(|ev: EventReader<ProductPurchasedEvent>| !ev.is_empty()),
        );

        app.register_async_event::<UpdateProductsEvent>();
        app.register_async_event::<ProductPurchasedEvent>();
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, EnumString, EnumTable, Display)]
pub enum Product {
    //spellchecker:disable
    #[strum(serialize = "removeads")]
    RemoveAds,
    #[strum(serialize = "naturalworldpack")]
    NaturalWorldPack,
    #[strum(serialize = "geographypack")]
    GeographyPack,
    #[strum(serialize = "ussports")]
    USSportsPack,
    #[strum(serialize = "hints500")]
    Hints500,
    #[strum(serialize = "hints100")]
    Hints100,
    #[strum(serialize = "hints50")]
    Hints50,
    #[strum(serialize = "hints25")]
    Hints25,
    //spellchecker:enable
}

impl From<LevelGroup> for Product {
    fn from(value: LevelGroup) -> Self {
        match value {
            LevelGroup::Geography => Product::GeographyPack,
            LevelGroup::NaturalWorld => Product::NaturalWorldPack,
            LevelGroup::USSports => Product::USSportsPack,
        }
    }
}

#[derive(Debug, Clone, Event)]
pub struct RefreshAndRestoreEvent;

#[derive(Debug, PartialEq, Serialize, Deserialize, MavericContext, Resource, Clone, Default)]
pub struct Purchases {
    /// Level groups which the user has purchased
    pub groups_purchased: HashSet<LevelGroup>,
    /// True is the user has purchased the pack to avoid ads
    pub remove_ads_purchased: bool,
}

impl TrackableResource for Purchases {
    const KEY: &'static str = "Purchases";
}

#[derive(Debug, PartialEq, MavericContext, Resource, Clone, Default)]
pub struct Prices {
    pub product_prices: ProductTable<Option<String>>,
}

impl Prices {
    pub fn get_price_string(&self, product: Product) -> String {
        match &self.product_prices[product] {
            Some(s) => s.to_string(),
            None => "???".to_string(),
        }
    }
}

#[derive(Debug, Clone, Event)]
pub struct UpdateProductsEvent {
    pub product_prices: ProductTable<Option<String>>,
    pub owned_products: Vec<Product>,
}

#[derive(Debug, Clone, Event)]
pub struct ProductPurchasedEvent {
    pub product: Product,
}

fn handle_product_purchased(
    mut events: EventReader<ProductPurchasedEvent>,
    mut purchases: ResMut<Purchases>,
    mut hints: ResMut<HintState>,
    sequence_completion: Res<SequenceCompletion>,
    mut change_level_events: EventWriter<ChangeLevelEvent>,
) {
    fn set_level_group(
        purchases: &mut ResMut<Purchases>,
        lg: LevelGroup,
        change_level_events: &mut EventWriter<ChangeLevelEvent>,
        sequence_completion: &SequenceCompletion,
    ) {
        if !purchases.groups_purchased.contains(&lg) {
            purchases.groups_purchased.insert(lg);

            let sequence = lg.get_sequences()[0];

            let level: CurrentLevel = sequence_completion
                .get_next_level_index(sequence, &purchases)
                .to_level(sequence);

            platform_specific::show_toast_sync(format!("{lg} Addon Purchased"));

            change_level_events.send(level.into());
        }
    }

    fn add_hints(hints: &mut ResMut<HintState>, number: usize) {
        hints.hints_remaining += number;
        hints.total_bought_hints += number;

        platform_specific::show_toast_sync(format!("{number} Hints Purchased"));
    }

    for ev in events.read() {
        match ev.product {
            Product::RemoveAds => {
                if !purchases.remove_ads_purchased {
                    purchases.remove_ads_purchased = true;
                    platform_specific::show_toast_sync("Remove Ads purchased");
                }
            }
            Product::NaturalWorldPack => set_level_group(
                &mut purchases,
                LevelGroup::NaturalWorld,
                &mut change_level_events,
                &sequence_completion,
            ),
            Product::GeographyPack => set_level_group(
                &mut purchases,
                LevelGroup::Geography,
                &mut change_level_events,
                &sequence_completion,
            ),
            Product::USSportsPack => set_level_group(
                &mut purchases,
                LevelGroup::USSports,
                &mut change_level_events,
                &sequence_completion,
            ),
            Product::Hints500 => add_hints(&mut hints, 500),
            Product::Hints100 => add_hints(&mut hints, 100),
            Product::Hints50 => add_hints(&mut hints, 50),
            Product::Hints25 => add_hints(&mut hints, 25),
        }
    }
}

fn update_products(
    mut events: EventReader<UpdateProductsEvent>,
    mut prices: ResMut<Prices>,
    mut purchases: ResMut<Purchases>,
) {
    fn set_level_group(p: &mut ResMut<Purchases>, lg: LevelGroup) {
        if !p.groups_purchased.contains(&lg) {
            p.groups_purchased.insert(lg);
        }
    }

    for ev in events.read() {
        info!("Updating products");

        prices.product_prices = ev.product_prices.clone();

        for product in ev.owned_products.iter() {
            match product {
                Product::RemoveAds => {
                    if !purchases.remove_ads_purchased {
                        purchases.remove_ads_purchased = true;
                    }
                }
                Product::NaturalWorldPack => {
                    set_level_group(&mut purchases, LevelGroup::NaturalWorld)
                }
                Product::GeographyPack => set_level_group(&mut purchases, LevelGroup::Geography),
                Product::USSportsPack => set_level_group(&mut purchases, LevelGroup::USSports),
                Product::Hints500 | Product::Hints100 | Product::Hints50 | Product::Hints25 => {}
            }
        }
    }
}

/// Event that is sent when a user requests a purchase
#[derive(Debug, Event, Clone, PartialEq)]
pub enum RequestPurchaseEvent {
    BuyHintsPack { number_of_hints: usize },
    BuyLevelGroupBySequence(LevelSequence),
    BuyLevelGroup(LevelGroup),
    BuyAvoidAds,
}

impl<'a> From<&'a RequestPurchaseEvent> for Product {
    fn from(val: &'a RequestPurchaseEvent) -> Self {
        match val {
            RequestPurchaseEvent::BuyHintsPack { number_of_hints } => match number_of_hints {
                25 => Product::Hints25,
                50 => Product::Hints50,
                100 => Product::Hints100,
                500 => Product::Hints500,
                _ => {
                    warn!("Unexpected number of hints ({})", number_of_hints);
                    Product::Hints25
                }
            },
            RequestPurchaseEvent::BuyLevelGroupBySequence(ls) => ls.group().into(),
            RequestPurchaseEvent::BuyLevelGroup(lg) => (*lg).into(),
            RequestPurchaseEvent::BuyAvoidAds => Product::RemoveAds,
        }
    }
}
