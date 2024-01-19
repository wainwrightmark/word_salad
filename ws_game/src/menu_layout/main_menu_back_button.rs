use bevy::math::Vec2;
use ws_core::{
    layout::entities::{IDEAL_HEIGHT, IDEAL_WIDTH, MENU_BUTTON_FONT_SIZE},
    LayoutSizing, LayoutStructure, LayoutStructureWithFont, LayoutStructureWithStaticText,
};

use super::{MENU_BUTTON_HEIGHT, MENU_BUTTON_SPACING, MENU_BUTTON_WIDTH};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub struct MainMenuBackButton;

impl LayoutStructure for MainMenuBackButton {
    type Context<'a> = ();

    fn size(&self, _context: &Self::Context<'_>) -> Vec2 {
        Vec2 {
            x: MENU_BUTTON_WIDTH,
            y: MENU_BUTTON_HEIGHT,
        }
    }

    fn location(&self, _context: &Self::Context<'_>, _sizing: &LayoutSizing) -> Vec2 {
        Vec2 {
            x: (IDEAL_WIDTH - MENU_BUTTON_WIDTH) / 2.,
            y: IDEAL_HEIGHT - ((MENU_BUTTON_HEIGHT + MENU_BUTTON_SPACING) * 2.0),
        }
    }

    fn iter_all(_context: &Self::Context<'_>) -> impl Iterator<Item = Self> {
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
    fn text(&self, _context: &Self::Context<'_>) -> &'static str {
        "Back"
    }
}
