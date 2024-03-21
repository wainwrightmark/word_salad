use bevy::prelude::*;
use ws_common::purchase_common::*;

use self::purchase_api::{PurchasePlatform, Transaction};

pub struct PurchasesPlugin;

impl Plugin for PurchasesPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            handle_request_purchase_events
                .run_if(|x: EventReader<RequestPurchaseEvent>| !x.is_empty()),
        );

        app.add_systems(Startup, on_startup);

        app.add_systems(
            Update,
            handle_refresh_and_restore
                .run_if(|ev: EventReader<RefreshAndRestoreEvent>| !ev.is_empty()),
        );
    }
}

#[allow(unused_variables)]
fn on_startup(
    get_products_writer: nice_bevy_utils::async_event_writer::AsyncEventWriter<UpdateProductsEvent>,
    product_purchased_event_writer_async: nice_bevy_utils::async_event_writer::AsyncEventWriter<
        ProductPurchasedEvent,
    >,
) {
    let platform: PurchasePlatform;

    #[cfg(feature = "android")]
    {
        platform = PurchasePlatform::GooglePlay;
    }
    #[cfg(all(not(feature = "android"), feature = "ios"))]
    {
        platform = PurchasePlatform::AppleAppstore;
    }

    ws_common::asynchronous::spawn_and_run(purchase_api::initialize_and_get_products(
        platform,
        get_products_writer,
        move |t: Transaction| {
            bevy::log::info!("Transaction Approved: {t:?}");

            //std::sync::mpsc::channel()

            for transaction_product in t.products.iter() {
                if let Ok(product) = transaction_product.try_into() {
                    product_purchased_event_writer_async
                        .send_or_panic(ProductPurchasedEvent { product })
                }
            }
        },
    ));
}

#[allow(unused_variables)]
fn handle_refresh_and_restore(
    mut ev: EventReader<RefreshAndRestoreEvent>,
    get_products_writer: nice_bevy_utils::async_event_writer::AsyncEventWriter<UpdateProductsEvent>,
) {
    for event in ev.read() {
        let writer = get_products_writer.clone();
        ws_common::asynchronous::spawn_and_run(purchase_api::refresh_and_get_products(writer));
    }
}

fn handle_request_purchase_events(mut ev: EventReader<RequestPurchaseEvent>) {
    for event in ev.read() {
        let product: Product = event.into();

        ws_common::asynchronous::spawn_and_run(purchase_api::purchase_product_async(product));
    }
}

mod purchase_api {
    #[allow(unused_imports)]
    use capacitor_bindings::error::Error;
    use serde::{Deserialize, Serialize};
    use std::str::FromStr;
    #[allow(unused_imports)]
    use wasm_bindgen_futures::wasm_bindgen::{self, closure::Closure, JsValue};

    use super::*;

    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen::prelude::wasm_bindgen(module = "/purchase.js")]
    extern "C" {
        #[wasm_bindgen(catch, final, js_name = "initialize_and_get_products")]
        async fn initialize_and_get_products_extern(
            platform: JsValue,
            func: &Closure<dyn Fn(JsValue)>,
        ) -> Result<JsValue, JsValue>;

        #[wasm_bindgen(catch, final, js_name = "refresh_and_get_products")]
        async fn refresh_and_get_products_extern() -> Result<JsValue, JsValue>;

        #[wasm_bindgen(catch, final, js_name = "purchase_product")]
        async fn purchase_product_extern(
            product_purchase_options: JsValue,
        ) -> Result<JsValue, JsValue>;
    }

    // #[cfg(not(target_arch = "wasm32"))]
    // async fn initialize_and_get_products_extern(_platform: JsValue) -> Result<JsValue, JsValue> {
    //     panic!("Purchase plugin only works on wasm");
    // }

    #[cfg(not(target_arch = "wasm32"))]
    async fn refresh_and_get_products_extern() -> Result<JsValue, JsValue> {
        panic!("Purchase plugin only works on wasm");
    }

    #[cfg(not(target_arch = "wasm32"))]
    async fn purchase_product_extern(
        _product_purchase_options: JsValue,
    ) -> Result<JsValue, JsValue> {
        panic!("Purchase plugin only works on wasm");
    }

    pub async fn purchase_product_async(options: impl Into<ProductPurchaseOptions>) {
        let result: Result<ProductPurchaseResult, capacitor_bindings::error::Error> =
            capacitor_bindings::helpers::run_value_value(options, purchase_product_extern).await;

        match result {
            Ok(result) => {
                if result.success {
                    ws_common::platform_specific::show_toast_async("Purchase Pending").await;
                } else {
                    ws_common::platform_specific::show_toast_async("Purchase Cancelled").await;
                }
            }
            Err(e) => {
                bevy::log::error!("purchase_product error: {e}");
                ws_common::platform_specific::show_toast_async("Purchase Error").await;
            }
        }
    }

    #[allow(unused_variables)]
    pub async fn initialize_and_get_products(
        platform: PurchasePlatform,
        writer: nice_bevy_utils::async_event_writer::AsyncEventWriter<super::UpdateProductsEvent>,
        func: impl Fn(Transaction) + 'static,
    ) {
        let result: Result<Vec<ExternProduct>, capacitor_bindings::error::Error>;
        #[cfg(not(target_arch = "wasm32"))]
        {
            panic!("Purchase plugin only works on wasm");
        }

        #[cfg(target_arch = "wasm32")]
        {
            let js_input_value: JsValue = serde_wasm_bindgen::to_value(&platform)
                .map_err(|e| Error::serializing::<PurchasePlatform>(e))
                .unwrap();

            let func2 = move |js_value: JsValue| {
                let schema: Transaction = serde_wasm_bindgen::from_value(js_value).unwrap(); //deserialize should always succeed assuming I have done everything else right
                func(schema)
            };
            let closure = Closure::new(func2);
            let box_closure = Box::new(closure);
            let closure_ref = Box::leak(box_closure);

            result = initialize_and_get_products_extern(js_input_value, closure_ref)
                .await
                .map_err(|e| Error::from(e))
                .and_then(|js_output_value| {
                    let o: Result<Vec<ExternProduct>, capacitor_bindings::error::Error> =
                        serde_wasm_bindgen::from_value(js_output_value)
                            .map_err(|e| Error::deserializing::<Vec<ExternProduct>>(e));
                    o
                });
        }

        #[allow(unreachable_code)]
        match result {
            Ok(products) => {
                bevy::log::info!("get_products success - found {} products", products.len());
                for product in products.iter() {
                    bevy::log::info!("Got product: {product:?}");
                }
                let event = ExternProduct::make_update_products_event(products);
                writer.send_or_panic(event);
            }
            Err(e) => {
                bevy::log::error!("get_products error: {e}");
                ws_common::platform_specific::show_toast_async("Could not load store products")
                    .await;
            }
        }
    }

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
                let event = ExternProduct::make_update_products_event(products);
                writer.send_or_panic(event);
            }
            Err(e) => {
                bevy::log::error!("get_products error: {e}");
                ws_common::platform_specific::show_toast_async("Could not load store products")
                    .await;
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
        #[serde(rename = "success")]
        pub success: bool,
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
            Product::from_str(self.id.as_str()).ok()
        }

        pub fn make_update_products_event(products: Vec<Self>) -> UpdateProductsEvent {
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

    #[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
    pub enum TransactionState {
        #[serde(rename = "initiated")]
        Initiated,
        #[serde(rename = "pending")]
        Pending,
        #[serde(rename = "approved")]
        Approved,
        #[serde(rename = "cancelled")]
        Cancelled,
        #[serde(rename = "finished")]
        Finished,
        #[default]
        UnknownState,
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

    #[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
    pub struct Transaction {
        /** Platform this transaction was created on */
        #[serde(rename = "platform")]
        pub platform: PurchasePlatform,
        /** Transaction identifier. */
        #[serde(rename = "transactionId")]
        pub transaction_id: String,
        /** Identifier for the purchase this transaction is a part of. */
        #[serde(rename = "purchaseId")]
        #[serde(default)]
        pub purchase_id: Option<String>,

        /** True when the transaction has been acknowledged to the platform. */
        #[serde(rename = "isAcknowledged")]
        #[serde(default)]
        pub is_acknowledged: Option<bool>,

        /** True when the transaction is still pending payment. */
        #[serde(rename = "isPending")]
        #[serde(default)]
        pub is_pending: Option<bool>,

        /** True when the transaction was consumed. */
        #[serde(rename = "isConsumed")]
        #[serde(default)]
        pub is_consumed: Option<bool>,

        /** State this transaction is in */
        #[serde(rename = "state")]
        #[serde(default)]
        pub state: TransactionState,

        /** Purchased products */
        #[serde(rename = "products")]
        #[serde(default)]
        pub products: Vec<TransactionProduct>,
    }

    #[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
    pub struct TransactionProduct {
        /** Product identifier */
        #[serde(rename = "id")]
        pub id: String,
        /** Offer identifier, if known */
        #[serde(rename = "offer_id")]
        #[serde(default)]
        pub offer_id: Option<String>,
    }

    // impl<'a> Into<Product> for &'a TransactionProduct {
    //     fn into(self) -> Product {
    //         Product::from_str(&self.id).unwrap()
    //     }
    // }

    impl<'a> TryInto<Product> for &'a TransactionProduct {
        type Error = strum::ParseError;

        fn try_into(self) -> Result<Product, Self::Error> {
            Product::from_str(&self.id)
        }
    }
}
