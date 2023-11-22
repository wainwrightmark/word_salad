pub mod animated_solutions;
pub mod asynchronous;
pub mod constants;
pub mod current_level;
pub mod grid_input;
pub mod top_bar_view;
pub mod congrats_view;
pub mod game_grid_view;
pub mod input;
pub mod level_time;
pub mod menu;
pub mod startup;
pub mod state;
pub mod ui_view;
pub mod video;
pub mod view;
pub mod z_indices;
pub mod menu_layout;
#[cfg(target_arch = "wasm32")]
pub mod wasm;


fn main() {
    crate::startup::go();
}

pub mod prelude {

    pub use crate::animated_solutions::*;
    pub use crate::asynchronous::*;
    pub use crate::congrats_view::*;
    pub use crate::constants::*;
    pub use crate::current_level::*;
    pub use crate::game_grid_view::*;
    pub use crate::grid_input::*;
    pub use crate::level_time::*;
    pub use crate::menu::*;
    pub use crate::menu_layout::*;
    pub use crate::state::*;
    pub use crate::top_bar_view::*;
    pub use crate::ui_view::*;
    pub use crate::video::*;
    pub use crate::view::*;

    pub use std::array;

    //pub use bevy::prelude::*;

    pub use geometrid::prelude::*;

    pub use geometrid::prelude::HasCenter;
    pub use maveric::prelude::*;

    pub use ws_core::prelude::*;

    pub use ws_core::Tile;

    pub const fn convert_color(c: BasicColor)-> Color{
        let BasicColor { red, green, blue, alpha } = c;
        Color::Rgba { red, green, blue, alpha }
    }
}
