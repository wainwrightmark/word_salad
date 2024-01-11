use bevy::math::Vec2;
use ws_core::{
    layout::entities::{IDEAL_HEIGHT, IDEAL_WIDTH},
    LayoutStructure, LayoutStructureWithFont, LayoutStructureWithStaticText, LayoutSizing,
};

use super::{MENU_BUTTON_FONT_SIZE, MENU_BUTTON_HEIGHT, MENU_BUTTON_SPACING, MENU_BUTTON_WIDTH};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub struct MainMenuBackButton;

impl LayoutStructure for MainMenuBackButton {
    type Context = ();

    fn size(&self, _context: &Self::Context) -> Vec2 {
        Vec2 {
            x: MENU_BUTTON_WIDTH,
            y: MENU_BUTTON_HEIGHT,
        }
    }

    fn location(&self, _context: &Self::Context, _sizing: &LayoutSizing) -> Vec2 {
        Vec2 {
            x: (IDEAL_WIDTH - MENU_BUTTON_WIDTH) / 2.,
            y: IDEAL_HEIGHT - ((MENU_BUTTON_HEIGHT + MENU_BUTTON_SPACING) * 2.0),
        }
    }

    fn iter_all(_context: &Self::Context) -> impl Iterator<Item = Self> {
        std::iter::once(MainMenuBackButton)
    }
}

impl LayoutStructureWithFont for MainMenuBackButton {
    type FontContext = ();
    fn font_size(&self, _: &()) -> f32 {
        MENU_BUTTON_FONT_SIZE
    }
}

impl LayoutStructureWithStaticText for MainMenuBackButton {
    fn text(&self, _context: &Self::Context) -> &'static str {
        "Back"
    }
}
