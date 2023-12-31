use bevy::math::Vec2;
use strum::{Display, EnumCount, EnumIter, IntoEnumIterator};
use ws_core::{
    layout::entities::{IDEAL_HEIGHT, IDEAL_WIDTH, TOP_BAR_HEIGHT},
    LayoutStructure, LayoutStructureWithFont, LayoutStructureWithStaticText, Spacing,
};

use super::{MENU_BUTTON_FONT_SIZE, MENU_BUTTON_HEIGHT, MENU_BUTTON_SPACING, MENU_BUTTON_WIDTH};

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, EnumIter, EnumCount, Display,
)]
pub enum MainMenuLayoutEntity {
    Puzzles = 0,
    Store = 1,
    SelfieMode = 2,
    Tutorial = 3,
    ResetPuzzle = 4,
    #[cfg(target_arch= "wasm32")]
    PlaySteks = 5
}

impl MainMenuLayoutEntity {
    pub fn index(&self) -> usize {
        *self as usize
    }
}

impl LayoutStructure for MainMenuLayoutEntity {
    type Context = ();

    fn size(&self, _context: &Self::Context) -> Vec2 {
        Vec2 {
            x: MENU_BUTTON_WIDTH,
            y: MENU_BUTTON_HEIGHT,
        }
    }

    fn location(&self, _context: &Self::Context) -> Vec2 {
        Vec2 {
            x: (IDEAL_WIDTH - MENU_BUTTON_WIDTH) / 2.,
            y: TOP_BAR_HEIGHT
                + Spacing::Centre.apply(
                    IDEAL_HEIGHT - TOP_BAR_HEIGHT,
                    MENU_BUTTON_HEIGHT + MENU_BUTTON_SPACING,
                    super::MENU_VIRTUAL_CHILDREN,
                    self.index(),
                ),
        }
    }

    fn iter_all(_context: &Self::Context) -> impl Iterator<Item = Self> {
        Self::iter()
    }
}

impl LayoutStructureWithFont for MainMenuLayoutEntity {
    fn font_size(&self) -> f32 {
        MENU_BUTTON_FONT_SIZE
    }
}

impl LayoutStructureWithStaticText for MainMenuLayoutEntity {
    fn text(&self, _context: &Self::Context) -> &'static str {
        use MainMenuLayoutEntity::*;

        match self {
            Puzzles => "Puzzles",
            Store => "Store",
            SelfieMode => "Selfie Mode",
            Tutorial => "Tutorial",
            ResetPuzzle => "Reset Puzzle",
            PlaySteks => "Play Steks"
        }
    }
}
