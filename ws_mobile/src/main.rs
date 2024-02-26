use ads::AdsPlugin;
use app_lifecycle::AppLifecyclePlugin;
use notifications::NotificationPlugin;
use purchases::PurchasesPlugin;
use ws_common::prelude::*;

pub mod app_lifecycle;
pub mod notifications;
pub mod ads;
pub mod purchases;

fn main() {
    ws_common::startup::setup_app(add_mobile_plugins);
}

fn add_mobile_plugins(app: &mut App) {
    app.add_plugins(NotificationPlugin);
    app.add_plugins(AppLifecyclePlugin);
    app.add_plugins(AdsPlugin);
    app.add_plugins(PurchasesPlugin);
}
