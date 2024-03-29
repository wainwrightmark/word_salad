pub mod ads;
pub mod purchases;
#[cfg(feature = "recording")]
pub mod recording;

use ads::AdsPlugin;

use purchases::PurchasesPlugin;
use ws_common::prelude::*;

fn main() {
    ws_common::startup::setup_app(add_web_plugins);
}

fn add_web_plugins(app: &mut App) {
    app.add_plugins(PurchasesPlugin);
    app.add_plugins(AdsPlugin);

    #[cfg(feature = "recording")]
    {
        app.add_plugins(crate::recording::RecordingPlugin);
    }
}
