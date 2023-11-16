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

    fn tile_font_size(&self) -> f32;
    fn tile_size(&self)-> f32;

    fn get_rect(&self, entity: LayoutEntity) -> layout::Rect;
    fn try_pick(&self, p: Vec2, tolerance: f32) -> Option<LayoutEntity>;
}

fn layout(size: &Size) -> ws_core::layout::Layout {
    //TODO include Layout on the window_size object
    Layout::from_page_size(Vec2 {
        x: size.scaled_width,
        y: size.scaled_height,
    })
}

impl SaladWindowSize for Size {
    fn get_rect(&self, entity: LayoutEntity) -> layout::Rect {
        let mut rect = layout(self).get_rect(entity);

        rect.top_left = Vec2 {
            x: (self.scaled_width * -0.5) + rect.top_left.x,
            y: (self.scaled_height * 0.5) - rect.top_left.y,
        };

        rect.extents.y = rect.extents.y * -1.0;

        rect
    }

    fn try_pick(&self, p: Vec2, tolerance: f32) -> Option<LayoutEntity> {
        let entity = layout(self).try_pick_entity(p, tolerance);
        entity
    }

    fn scale(&self) -> f32 {
        (self.scaled_width / 4.0).min(self.scaled_height / 8.0)
    }

    fn tile_font_size(&self) -> f32 {
        (self.scale() * 0.1875).ceil() * 4.0
    }

    fn tile_size(&self)-> f32 {
        layout(self).get_size(LayoutEntity::GridTile(Default::default())).x
    }
}

pub const DEFAULT_WINDOW_WIDTH: f32 = 400f32;
pub const DEFAULT_WINDOW_HEIGHT: f32 = 800f32;

pub const TILE_MULTIPLIER: f32 = 0.9;

pub const TILE_FONT_PATH: &str = "fonts/MartianMono_SemiExpanded-Bold.ttf";
pub const CURRENT_STRING_FONT_PATH: &str = "fonts/MartianMono_SemiExpanded-Bold.ttf";
pub const TITLE_FONT_PATH: &str = "fonts/FiraMono-Medium.ttf";
pub const BUTTONS_FONT_PATH: &str = "fonts/FiraMono-Medium.ttf";
pub const SOLUTIONS_FONT_PATH: &str = "fonts/MartianMono_SemiCondensed-Regular.ttf";
pub const MENU_BUTTON_FONT_PATH: &str = "fonts/merged-font.ttf";

pub const ICON_FONT_PATH: &str = "";

pub const ICON_BUTTON_SIZE: f32 = 40f32; //40 pixels
pub const TOOLBAR_SIZE: f32 = 40f32; //40 pixels
