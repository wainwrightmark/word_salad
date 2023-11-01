pub use crate::prelude::*;



pub const WINDOW_WIDTH: f32 = 400f32;
pub const WINDOW_HEIGHT: f32 = 800f32;
pub const WINDOW_SIZE: f32 = if WINDOW_HEIGHT < WINDOW_WIDTH {
    WINDOW_HEIGHT
} else {
    WINDOW_WIDTH
};
pub const SCALE: f32 = WINDOW_SIZE / 5.0;
pub const TILE_SIZE: f32 = SCALE * TILE_MULTIPLIER;
const TILE_MULTIPLIER: f32 = 0.9;

pub const TOP_LEFT: Vec2 = Vec2 {
    x: (WINDOW_WIDTH * -0.5) + TILE_SIZE,
    y: (WINDOW_HEIGHT * -0.5) + TILE_SIZE,
};
