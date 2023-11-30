use bevy::math::Vec2;
use ws_core::{
    layout::entities::{IDEAL_HEIGHT, IDEAL_WIDTH},
    LayoutStructure, LayoutStructureWithFont, LayoutStructureWithStaticText,
};

use super::{
    MENU_BUTTON_FONT_SIZE, MENU_BUTTON_HEIGHT, MENU_BUTTON_PADDING_RATIO, MENU_BUTTON_WIDTH,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub struct MainMenuBackButton;

impl LayoutStructure for MainMenuBackButton {
    type Context = ();
    type Iterator = std::iter::Once<Self>;

    fn pick(point: Vec2, context: &Self::Context) -> Option<Self> {
        for x in Self::iter_all(context) {
            if x.rect(context).contains(point) {
                return Some(x);
            }
        }
        return None;
    }

    fn size(&self, _context: &Self::Context) -> Vec2 {
        Vec2 {
            x: MENU_BUTTON_WIDTH,
            y: MENU_BUTTON_HEIGHT,
        }
    }

    fn location(&self, _context: &Self::Context) -> Vec2 {
        Vec2 {
            x: (IDEAL_WIDTH - MENU_BUTTON_WIDTH) / 2.,
            y: IDEAL_HEIGHT - (MENU_BUTTON_HEIGHT * MENU_BUTTON_PADDING_RATIO)
        }
    }

    fn iter_all(_context: &Self::Context) -> Self::Iterator {
        std::iter::once(MainMenuBackButton)
    }
}

impl LayoutStructureWithFont for MainMenuBackButton {
    fn font_size() -> f32 {
        MENU_BUTTON_FONT_SIZE
    }
}

impl LayoutStructureWithStaticText for MainMenuBackButton {
    fn text(&self, _context: &Self::Context) -> &'static str {
        "Back"
    }
}
