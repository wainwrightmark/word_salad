use crate::prelude::*;
use bevy::{prelude::*, utils::HashSet};
use nice_bevy_utils::{CanInitTrackedResource, CanRegisterAsyncEvent, TrackableResource};
use serde::{Deserialize, Serialize};
use strum::{EnumString, EnumTable};
use ws_levels::{level_group::LevelGroup, level_sequence::LevelSequence};

pub struct PurchasesPlugin;

impl Plugin for PurchasesPlugin {
    fn build(&self, app: &mut App) {
        app.init_tracked_resource::<Purchases>();
        app.init_resource::<Prices>();
        app.add_event::<PurchaseEvent>();

        app.add_systems(
            Update,
            track_purchase_events.run_if(|x: EventReader<PurchaseEvent>| !x.is_empty()),
        );

        app.add_systems(Startup, on_startup);
        app.add_systems(
            Update,
            update_products
                .run_if(|ev: EventReader<UpdateProductsEvent>| !ev.is_empty()),
        );

        app.register_async_event::<UpdateProductsEvent>();
    }
}

#[allow(unused_variables)]
fn on_startup(
    get_products_writer: nice_bevy_utils::async_event_writer::AsyncEventWriter<
        UpdateProductsEvent,
    >,
) {
    #[cfg(all(target_arch = "wasm32", any(feature = "android", feature = "ios")))]
    {
        crate::asynchronous::spawn_and_run(purchase_api::get_products(get_products_writer));
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize, MavericContext, Resource, Clone, Default)]
pub struct Purchases {
    /// Level groups which the user has purchased
    pub groups_purchased: HashSet<LevelGroup>,
    /// True is the user has purchased the pack to avoid ads
    pub remove_ads_purchased: bool,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, EnumString, EnumTable)]
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

impl TrackableResource for Purchases {
    const KEY: &'static str = "Purchases";
}

fn track_purchase_events(
    mut ev: EventReader<PurchaseEvent>,
    mut purchases: ResMut<Purchases>,
    mut hints: ResMut<HintState>,
    sequence_completion: Res<SequenceCompletion>,
    mut change_level_events: EventWriter<ChangeLevelEvent>,
    mut hint_events: EventWriter<HintEvent>,
) {
    for event in ev.read() {
        match event {
            PurchaseEvent::BuyHintsPack {
                hint_event,
                number_of_hints,
            } => {
                hints.hints_remaining += number_of_hints;
                hints.total_bought_hints += number_of_hints;
                show_toast_on_web("In the real app you would pay money");
                if let Some(hint_event) = hint_event {
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
                purchases.remove_ads_purchased = true;
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
            }
        }
    }
}

#[derive(Debug, Event, Clone, PartialEq)]
pub enum PurchaseEvent {
    BuyHintsPack {
        hint_event: Option<HintEvent>,
        number_of_hints: usize,
    },
    BuyLevelGroupBySequence(LevelSequence),
    BuyLevelGroup(LevelGroup),
    BuyAvoidAds,
}

mod purchase_api {
    use serde::{Deserialize, Serialize};

    #[cfg(all(target_arch = "wasm32", any(feature = "android", feature = "ios")))]
    use wasm_bindgen_futures::wasm_bindgen::JsValue;

    use super::*;

    #[cfg(all(target_arch = "wasm32", any(feature = "android", feature = "ios")))]
    #[wasm_bindgen::prelude::wasm_bindgen(module = "/purchase.js")]
    extern "C" {
        #[wasm_bindgen(catch, final, js_name = "get_products")]
        async fn get_products_extern() -> Result<JsValue, JsValue>;
    }

    #[cfg(all(target_arch = "wasm32", any(feature = "android", feature = "ios")))]
    pub async fn get_products(
        writer: nice_bevy_utils::async_event_writer::AsyncEventWriter<
            super::UpdateProductsEvent,
        >,
    ) {
        let result: Result<Vec<ExternProduct>, capacitor_bindings::error::Error> =
            capacitor_bindings::helpers::run_unit_value(get_products_extern).await;

        match result {
            Ok(products) => {
                for product in products.iter() {
                    bevy::log::info!("{product:?}");
                }
                let event = ExternProduct::make_product_price_event(products);
                writer.send_async(event).await.unwrap();
            }
            Err(e) => {
                bevy::log::error!("get_products error: {e}");
                crate::platform_specific::show_toast_async("Could not load store products").await;
            }
        }
    }

    #[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
    pub struct ExternProduct {
        /// Platform this product is available from
        #[serde(rename = "platform")]
        pub platform: PurchasePlatform,
        /// Type of product (subscription, consumable, etc.)
        #[serde(rename = "type")]
        pub r#type: ProductType,

        /// Product identifier on the store (unique per platform)
        #[serde(rename = "id")]
        pub id: String,

        /// List of offers available for this product
        #[serde(rename = "offers")]
        pub offers: Vec<Offer>,

        /// Product title from the store.
        #[serde(rename = "title")]
        pub title: String,
        /// Product full description from the store.
        #[serde(rename = "description")]
        pub description: String,

        /// Group the product is member of.
        ///
        /// Only 1 product of a given group can be owned. This is generally used
        /// to provide different levels for subscriptions, for example: silver
        /// and gold.
        ///
        /// Purchasing a different level will replace the previously owned one.
        #[serde(rename = "group")]
        #[serde(default)]
        pub group: Option<String>,

        /// Whether the product is owned
        #[serde(rename = "owned")]
        pub owned: bool,
    }

    #[allow(dead_code)]
    impl ExternProduct {
        pub fn pricing(&self) -> Option<String> {
            self.offers
                .iter()
                .flat_map(|x| x.pricing_phases.iter())
                .map(|x| &x.price)
                .next()
                .cloned()
        }

        pub fn as_product(&self) -> Option<Product> {
            use std::str::FromStr;
            Product::from_str(self.id.as_str()).ok()
        }

        pub fn make_product_price_event(products: Vec<Self>) -> UpdateProductsEvent {
            let mut product_prices: ProductTable<Option<String>> = Default::default();
            let mut owned_products = vec![];

            for extern_product in products.into_iter() {
                if let Some(product) = extern_product.as_product() {
                    if extern_product.owned{
                        owned_products.push(product);
                    }

                    product_prices[product] = extern_product.pricing();
                }
            }

            UpdateProductsEvent { product_prices, owned_products }
        }
    }

    #[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
    pub enum PurchasePlatform {
        #[serde(rename = "test")]
        Test,

        #[serde(rename = "ios-appstore")]
        AppleAppstore,
        #[serde(rename = "android-playstore")]
        GooglePlay,
        #[serde(rename = "windows-store-transaction")]
        WindowsStore,
        #[serde(rename = "braintree")]
        Braintree,
    }

    #[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]

    pub enum ProductType {
        /** Type: An consumable product, that can be purchased multiple time */
        #[serde(rename = "consumable")]
        CONSUMABLE,
        /** Type: A non-consumable product, that can purchased only once and the user keeps forever */
        #[serde(rename = "non consumable")]
        NonConsumable,
        /** @deprecated use PAID_SUBSCRIPTION */
        #[serde(rename = "free subscription")]
        FreeSubscription,
        /** Type: An auto-renewable subscription */
        #[serde(rename = "paid subscription")]
        PaidSubscription,
        /** Type: An non-renewing subscription */
        #[serde(rename = "non renewing subscription")]
        NonRenewingSubscription,
        /** Type: The application bundle */
        #[serde(rename = "application")]
        APPLICATION,
    }

    #[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
    #[serde(default)]

    pub struct Offer {
        #[serde(rename = "pricingPhases")]
        pub pricing_phases: Vec<PricingPhase>,
    }

    #[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
    #[serde(default)]

    pub struct PricingPhase {
        /// Price formatted for humans
        #[serde(rename = "price")]
        pub price: String,

        /// Price in micro-units (divide by 1000000 to get numeric price)
        #[serde(rename = "priceMicros")]
        pub price_micros: u64,

        /// Currency code
        #[serde(rename = "currency")]
        #[serde(default)]
        pub currency: Option<String>,
        //TODO  billingPeriod, billingCycles, recurrenceMode, paymentMode

        // /// ISO 8601 duration of the period (https://en.wikipedia.org/wiki/ISO_8601#Durations)
        // #[serde(rename ="billingPeriod")]
        // #[serde(default)]
        // pub billing_period: Option<String>,

        // /// Number of recurrence cycles (if recurrenceMode is FINITE_RECURRING)
        // #[serde(rename ="billingCycles")]
        // #[serde(default)]
        // pub billing_cycles: Option<u64>,
    }
}
