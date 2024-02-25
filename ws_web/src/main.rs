pub mod purchases;

use ws_common::prelude::*;
fn main() {
    ws_common::startup::setup_app(add_web_plugins);
}

fn add_web_plugins(_app: &mut App) {
}