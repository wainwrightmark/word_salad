use crate::{platform_specific, prelude::*};
use bevy::{prelude::*, utils::HashSet};
use nice_bevy_utils::{CanInitTrackedResource, CanRegisterAsyncEvent, TrackableResource};
use serde::{Deserialize, Serialize};
use strum::{Display, EnumString, EnumTable};
use ws_levels::{level_group::LevelGroup, level_sequence::LevelSequence};

pub struct PurchasesPlugin;

impl Plugin for PurchasesPlugin {
    fn build(&self, app: &mut App) {
        app.init_tracked_resource::<Purchases>();
        app.init_resource::<Prices>();
        app.add_event::<RequestPurchaseEvent>();
        app.add_event::<RefreshAndRestoreEvent>();

        app.add_systems(
            Update,
            handle_request_purchase_events
                .run_if(|x: EventReader<RequestPurchaseEvent>| !x.is_empty()),
        );

        app.add_systems(Startup, on_startup);
        app.add_systems(
            Update,
            update_products.run_if(|ev: EventReader<UpdateProductsEvent>| !ev.is_empty()),
        );

        app.add_systems(
            Update,
            handle_product_purchased
                .run_if(|ev: EventReader<ProductPurchasedEvent>| !ev.is_empty()),
        );

        app.add_systems(
            Update,
            handle_refresh_and_restore
                .run_if(|ev: EventReader<RefreshAndRestoreEvent>| !ev.is_empty()),
        );

        app.register_async_event::<UpdateProductsEvent>();
        app.register_async_event::<ProductPurchasedEvent>();
    }
}

#[allow(unused_variables)]
fn on_startup(
    get_products_writer: nice_bevy_utils::async_event_writer::AsyncEventWriter<UpdateProductsEvent>,
) {
    #[cfg(all(target_arch = "wasm32", any(feature = "android", feature = "ios")))]
    {
        crate::asynchronous::spawn_and_run(purchase_api::get_products(get_products_writer));
    }
}

#[derive(Debug, Clone, Event)]
pub struct  RefreshAndRestoreEvent;

#[allow(unused_variables)]
fn handle_refresh_and_restore(
    mut ev: EventReader<RefreshAndRestoreEvent>,
    get_products_writer: nice_bevy_utils::async_event_writer::AsyncEventWriter<UpdateProductsEvent>,
){
    for event in ev.read(){
        #[cfg(all(target_arch = "wasm32", any(feature = "android", feature = "ios")))]
        {
            let writer = get_products_writer.clone();
            crate::asynchronous::spawn_and_run(purchase_api::refresh_and_get_products(writer));
        }
        platform_specific::show_toast_on_web("We would refresh prices and restore purchases here");
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
        }

        let sequence = lg.get_sequences()[0];

        let level: CurrentLevel = sequence_completion
            .get_next_level_index(sequence, &purchases)
            .to_level(sequence);

        change_level_events.send(level.into());
    }

    fn add_hints(hints: &mut ResMut<HintState>, number: usize) {
        hints.hints_remaining += number;
        hints.total_bought_hints += number;
    }

    for ev in events.read() {
        match ev.product {
            Product::RemoveAds => purchases.remove_ads_purchased = true,
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

impl TrackableResource for Purchases {
    const KEY: &'static str = "Purchases";
}

#[cfg(all(target_arch = "wasm32", any(feature = "android", feature = "ios")))]
fn handle_request_purchase_events(
    mut ev: EventReader<RequestPurchaseEvent>,
    mut product_purchased_event_writer_async: nice_bevy_utils::async_event_writer::AsyncEventWriter<
        ProductPurchasedEvent,
    >,
) {
    for event in ev.read() {
        let product: Product = event.into();
        let writer = product_purchased_event_writer_async.clone();

        crate::asynchronous::spawn_and_run(purchase_api::purchase_product_async(
            product, product, writer,
        ));
    }
}

#[cfg(not(all(target_arch = "wasm32", any(feature = "android", feature = "ios"))))]
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

mod purchase_api {
    use serde::{Deserialize, Serialize};

    #[cfg(all(target_arch = "wasm32", any(feature = "android", feature = "ios")))]
    use wasm_bindgen_futures::wasm_bindgen::JsValue;

    use super::*;

    #[cfg(all(target_arch = "wasm32", feature = "android"))]
    #[wasm_bindgen::prelude::wasm_bindgen(module = "/android_purchase.js")]
    extern "C" {
        #[wasm_bindgen(catch, final, js_name = "get_products")]
        async fn get_products_extern() -> Result<JsValue, JsValue>;

        #[wasm_bindgen(catch, final, js_name = "refresh_and_get_products")]
        async fn refresh_and_get_products_extern() -> Result<JsValue, JsValue>;

        #[wasm_bindgen(catch, final, js_name = "purchase_product")]
        async fn purchase_product_extern(
            product_purchase_options: JsValue,
        ) -> Result<JsValue, JsValue>;
    }

    #[cfg(all(target_arch = "wasm32",  feature = "ios"))]
    #[wasm_bindgen::prelude::wasm_bindgen(module = "/ios_purchase.js")]
    extern "C" {
        #[wasm_bindgen(catch, final, js_name = "get_products")]
        async fn get_products_extern() -> Result<JsValue, JsValue>;

        #[wasm_bindgen(catch, final, js_name = "refresh_and_get_products")]
        async fn refresh_and_get_products_extern() -> Result<JsValue, JsValue>;

        #[wasm_bindgen(catch, final, js_name = "purchase_product")]
        async fn purchase_product_extern(
            product_purchase_options: JsValue,
        ) -> Result<JsValue, JsValue>;
    }

    #[cfg(all(target_arch = "wasm32", any(feature = "android", feature = "ios")))]
    pub async fn purchase_product_async(
        product: Product,
        options: impl Into<ProductPurchaseOptions>,
        writer: nice_bevy_utils::async_event_writer::AsyncEventWriter<super::ProductPurchasedEvent>,
    ) {
        let result: Result<ProductPurchaseResult, capacitor_bindings::error::Error> =
            capacitor_bindings::helpers::run_value_value(options, purchase_product_extern).await;

        match result {
            Ok(result) => {
                if result.purchased {
                    writer
                        .send_async(ProductPurchasedEvent { product })
                        .await
                        .unwrap();
                } else {
                    crate::platform_specific::show_toast_async("Product was not purchased").await;
                }
            }
            Err(e) => {
                bevy::log::error!("purchase_product error: {e}");
                crate::platform_specific::show_toast_async("Could not purchase product").await;
            }
        }
    }

    #[cfg(all(target_arch = "wasm32", any(feature = "android", feature = "ios")))]
    pub async fn get_products(
        writer: nice_bevy_utils::async_event_writer::AsyncEventWriter<super::UpdateProductsEvent>,
    ) {
        let result: Result<Vec<ExternProduct>, capacitor_bindings::error::Error> =
            capacitor_bindings::helpers::run_unit_value(get_products_extern).await;

        match result {
            Ok(products) => {
                for product in products.iter() {
                    bevy::log::info!("Got product: {product:?}");
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

    #[cfg(all(target_arch = "wasm32", any(feature = "android", feature = "ios")))]
    pub async fn refresh_and_get_products(
        writer: nice_bevy_utils::async_event_writer::AsyncEventWriter<super::UpdateProductsEvent>,
    ) {
        let result: Result<Vec<ExternProduct>, capacitor_bindings::error::Error> =
            capacitor_bindings::helpers::run_unit_value(refresh_and_get_products_extern).await;

        match result {
            Ok(products) => {
                for product in products.iter() {
                    bevy::log::info!("Got product: {product:?}");
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
    pub struct ProductPurchaseOptions {
        #[serde(rename = "id")]
        pub id: String,
    }

    impl From<Product> for ProductPurchaseOptions {
        fn from(value: Product) -> Self {
            ProductPurchaseOptions {
                id: value.to_string(),
            }
        }
    }

    #[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
    pub struct ProductPurchaseResult {
        #[serde(rename = "purchased")]
        pub purchased: bool,
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
                    if extern_product.owned {
                        owned_products.push(product);
                    }

                    product_prices[product] = extern_product.pricing();
                }
            }

            UpdateProductsEvent {
                product_prices,
                owned_products,
            }
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
