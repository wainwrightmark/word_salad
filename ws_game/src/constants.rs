use bevy_utils::window_size::*;

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
    fn tile_size(&self) -> f32;
    fn grid_top_left(&self) -> Vec2;
    fn ui_top_left(&self) -> Vec2;
    fn pick_tile(&self, position: Vec2) -> DynamicTile;

    /// Get the position of the centre of the given tile
    fn tile_position(&self, tile: &Tile) -> Vec2;

    fn adjust_cursor_position(&self, p: Vec2) -> Vec2;

    fn tile_font_size(&self)-> f32;
}

impl SaladWindowSize for Size {
    fn scale(&self) -> f32 {
        (self.scaled_width / 4.0).min(self.scaled_height / 8.0)
    }

    fn tile_size(&self) -> f32 {
        self.scale()
    }

    fn tile_font_size(&self)-> f32 {
        (self.scale() * 0.1875).ceil() * 4.0
    }

    fn adjust_cursor_position(&self, p: Vec2) -> Vec2 {
        Vec2 {
            x: p.x - (self.scaled_width * 0.5),
            y: (self.scaled_height * 0.5) - p.y,
        }
    }

    fn grid_top_left(&self) -> Vec2 {

        let spare_width = self.scaled_width - (self.tile_size() * 4.0);

        Vec2 {
            x: (self.scaled_width * -0.5) + (spare_width * 0.5) + (self.tile_size() * 0.5),
            y: (self.scaled_height * 0.5) - self.tile_size() * 4.0,
        }
    }

    fn ui_top_left(&self) -> Vec2 {
        Vec2 {
            x: 0.0,
            y: self.grid_top_left().y + (self.tile_size() * 4.0) + 100.0,
        }
    }

    fn pick_tile(&self, position: Vec2) -> DynamicTile {
        let relative_position = position - self.grid_top_left();// + (self.tile_size() * 0.5);

        let dv = DynamicVertex ::from_center(&relative_position, self.scale());
        //todo return none if too far from the vertex
        let dt = dv.get_tile(&Corner::SouthEast);
        dt
    }

    fn tile_position(&self, tile: &Tile) -> Vec2 {
        let location = tile.get_north_west_vertex().get_center(self.scale()) + self.grid_top_left();
        location
    }


}

pub const DEFAULT_WINDOW_WIDTH: f32 = 400f32;
pub const DEFAULT_WINDOW_HEIGHT: f32 = 800f32;

//pub const SCALE: f32 = WINDOW_SIZE / 5.0;
//pub const TILE_SIZE: f32 = SCALE * TILE_MULTIPLIER;
pub const TILE_MULTIPLIER: f32 = 0.9;
