use app_lifecycle::AppLifecyclePlugin;
use notifications::NotificationPlugin;
use ws_common::prelude::*;

pub mod app_lifecycle;
pub mod notifications;

fn main() {
    ws_common::startup::setup_app(add_mobile_plugins);
}

fn add_mobile_plugins(app: &mut App) {
    app.add_plugins(NotificationPlugin);
    app.add_plugins(AppLifecyclePlugin);
}
