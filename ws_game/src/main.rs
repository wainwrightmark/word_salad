pub mod animated_solutions;
pub mod asynchronous;
pub mod background;
pub mod button;
pub mod button_node;
pub mod chosen_state;
pub mod compatibility;
pub mod completion;
pub mod congrats_view;
pub mod constants;
pub mod current_level;
pub mod game_grid_view;
pub mod grid_input;
pub mod input;
pub mod level_time;
pub mod logging;
pub mod menu;
pub mod menu_layout;
pub mod popup;
pub mod scheduled_component;
pub mod shapes;
pub mod startup;
pub mod state;
pub mod top_bar_view;
pub mod tutorial;
pub mod ui_view;
pub mod video;
pub mod view;
#[cfg(target_arch = "wasm32")]
pub mod wasm;
pub mod wordline;
pub mod z_indices;
pub mod hints_view;
pub mod daily_challenge;
pub mod non_level_view;

fn main() {
    crate::startup::go();
}

pub mod prelude {

    pub use crate::animated_solutions::*;
    pub use crate::asynchronous::*;
    pub use crate::background::*;
    pub use crate::button::*;
    pub use crate::button_node::*;
    pub use crate::chosen_state::*;
    pub use crate::compatibility::*;
    pub use crate::congrats_view::*;
    pub use crate::constants::*;
    pub use crate::current_level::*;
    pub use crate::daily_challenge::*;
    pub use crate::game_grid_view::*;
    pub use crate::grid_input::*;
    pub use crate::level_time::*;
    pub use crate::logging::*;
    pub use crate::menu::*;
    pub use crate::menu_layout::*;
    pub use crate::popup::*;
    pub use crate::scheduled_component::*;
    pub use crate::shapes::*;
    pub use crate::state::*;
    pub use crate::top_bar_view::*;
    pub use crate::ui_view::*;
    pub use crate::video::*;
    pub use crate::video::*;
    pub use crate::view::*;
    pub use crate::wordline::*;
    pub use crate::tutorial::*;
    pub use crate::hints_view::*;
    pub use crate::daily_challenge::*;
    pub use crate::non_level_view::*;

    pub use std::array;

    //pub use bevy::prelude::*;

    pub use geometrid::prelude::*;

    pub use geometrid::prelude::HasCenter;
    pub use maveric::prelude::*;

    pub use ws_core::prelude::*;

    pub use ws_core::Tile;

    pub trait ConvertColor {
        fn convert_color(self) -> Color;
    }

    impl ConvertColor for BasicColor {
        fn convert_color(self) -> Color {
            let BasicColor {
                red,
                green,
                blue,
                alpha,
            } = self;
            Color::Rgba {
                red,
                green,
                blue,
                alpha,
            }
        }
    }

    pub const fn convert_color_const(color: BasicColor) -> Color {
        let BasicColor {
            red,
            green,
            blue,
            alpha,
        } = color;
        Color::Rgba {
            red,
            green,
            blue,
            alpha,
        }
    }
}
