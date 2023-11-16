use bevy_utils::window_size::*;
use ws_core::layout;

pub use crate::prelude::*;

pub type Size = WindowSize<SaladWindowBreakPoints>;

#[derive(Debug, Default)]
pub struct SaladWindowBreakPoints;

impl Breakpoints for SaladWindowBreakPoints {
    fn size_scale(_: f32, _: f32) -> f32 {
        1.0
    }
}

pub trait SaladWindowSize {
    fn scale(&self) -> f32;
    // fn tile_size(&self) -> f32;
    // fn grid_top_left(&self) -> Vec2;
    // fn ui_top(&self) -> f32;

    // fn try_pick_tile(&self, position: Vec2, tolerance: f32) -> Option<DynamicTile>;
    // fn pick_tile(&self, position: Vec2) -> DynamicTile;

    // /// Get the position of the centre of the given tile
    // fn tile_position(&self, tile: &Tile) -> Vec2;

    // fn adjust_cursor_position(&self, p: Vec2) -> Vec2;

    fn tile_font_size(&self) -> f32;
    fn layout(&self) -> ws_core::layout::Layout; //TODO include Layout on the window_size object

    fn get_rect(&self, entity: LayoutEntity) -> layout::Rect;
    /// tolerance is between 0.0 (always returns none) and 1.0 (always returns a tile)
    fn try_pick(&self, p: Vec2, tolerance: f32) -> Option<LayoutEntity>;
}


impl SaladWindowSize for Size {
    fn layout(&self) -> ws_core::layout::Layout {
        Layout::from_page_size(Vec2 {
            x: self.scaled_width,
            y: self.scaled_height,
        })
    }

    fn get_rect(&self, entity: LayoutEntity) -> layout::Rect {
        let mut rect = self.layout().get_rect(entity);

        rect.top_left =  Vec2{
            x: (self.scaled_width * -0.5) + rect.top_left.x,
            y: (self.scaled_height * 0.5) - rect.top_left.y
        };

        rect.extents.y = rect.extents.y * -1.0;

        rect
    }

    fn try_pick(&self, p: Vec2, tolerance: f32) -> Option<LayoutEntity> {
        //let p = adjust_cursor_position(&self, p);

        let entity = self.layout().try_pick_entity(p, tolerance);

        //info!("{p} {entity:?}");
        entity
    }

    fn scale(&self) -> f32 {
        (self.scaled_width / 4.0).min(self.scaled_height / 8.0)
    }

    fn tile_font_size(&self) -> f32 {
        (self.scale() * 0.1875).ceil() * 4.0
    }

    //     fn grid_top_left(&self) -> Vec2 {
    //         let spare_width = self.scaled_width - (self.scale() * 4.0);

    //         Vec2 {
    //             x: (self.scaled_width * -0.5) + (spare_width * 0.5) + (self.scale() * 0.5),
    //             y: (self.scaled_height * 0.5) - self.scale() * 4.0,
    //         }
    //     }

    //     fn ui_top(&self) -> f32 {
    //         self.grid_top_left().y + (self.scale() * 4.0) + 40.0
    //     }

    //     fn try_pick_tile(&self, position: Vec2, tolerance: f32) -> Option<DynamicTile> {
    //         let scale = self.scale();
    //         let relative_position = position - self.grid_top_left();
    //         let dv = DynamicVertex::from_center(&relative_position, scale);

    //         let tile_centre = dv.get_center(scale);
    //         let dist = tile_centre.distance_squared(relative_position);
    //         let radius_squared = (tolerance * scale).powi(2);

    //         //info!("dist {dist} rs {radius_squared}");

    //         if dist < radius_squared {
    //             let dt = dv.get_tile(&Corner::SouthEast);
    //             Some(dt)
    //         } else {
    //             None
    //         }
    //     }

    //     fn pick_tile(&self, position: Vec2) -> DynamicTile {
    //         let relative_position = position - self.grid_top_left();

    //         let dv = DynamicVertex::from_center(&relative_position, self.scale());

    //         let dt = dv.get_tile(&Corner::SouthEast);
    //         dt
    //     }

    //     fn tile_position(&self, tile: &Tile) -> Vec2 {
    //         let location = tile.get_north_west_vertex().get_center(self.scale()) + self.grid_top_left();
    //         location
    //     }
}

pub const DEFAULT_WINDOW_WIDTH: f32 = 400f32;
pub const DEFAULT_WINDOW_HEIGHT: f32 = 800f32;

pub const TILE_MULTIPLIER: f32 = 0.9;

pub const TILE_FONT_PATH: &str = "fonts/MartianMono_SemiExpanded-Bold.ttf";
pub const CURRENT_STRING_FONT_PATH: &str = "fonts/MartianMono_SemiExpanded-Bold.ttf";
pub const TITLE_FONT_PATH: &str = "fonts/FiraMono-Medium.ttf";
pub const BUTTONS_FONT_PATH: &str = "fonts/FiraMono-Medium.ttf";
pub const SOLUTIONS_FONT_PATH: &str = "fonts/MartianMono_SemiCondensed-Regular.ttf";

pub const ICON_BUTTON_SIZE: f32 = 40f32; //40 pixels
pub const TOOLBAR_SIZE: f32 = 40f32; //40 pixels
