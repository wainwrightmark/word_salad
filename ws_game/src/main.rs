pub mod animated_solutions;
pub mod asynchronous;
pub mod constants;
pub mod designed_level;
pub mod input;

pub mod startup;
pub mod state;
pub mod view;
pub mod menu;
pub mod video;
#[cfg(target_arch = "wasm32")]
pub mod wasm;

fn main() {
    crate::startup::go();
}

pub mod prelude {

    pub use crate::animated_solutions::*;
    pub use crate::asynchronous::*;
    pub use crate::constants::*;
    pub use crate::designed_level::*;
    pub use crate::input::*;
    pub use crate::state::*;
    pub use crate::menu::*;
    pub use crate::view::*;
    pub use crate::video::*;

    pub use std::array;

    pub use bevy::prelude::*;

    pub use geometrid::prelude::*;

    pub use bevy_prototype_lyon::prelude::*;
    pub use geometrid::prelude::HasCenter;
    pub use maveric::prelude::*;

    pub use ws_core::prelude::*;

    pub use ws_core::Tile;
}
