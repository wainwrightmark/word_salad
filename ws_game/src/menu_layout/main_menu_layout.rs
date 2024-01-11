use bevy::math::Vec2;
use strum::{Display, EnumCount, EnumIter, IntoEnumIterator};
use ws_core::{
    layout::entities::{IDEAL_HEIGHT, IDEAL_WIDTH, TOP_BAR_HEIGHT},
    LayoutStructure, LayoutStructureWithFont, LayoutStructureWithStaticText, Spacing, LayoutSizing,
};

use super::{MENU_BUTTON_FONT_SIZE, MENU_BUTTON_HEIGHT, MENU_BUTTON_SPACING, MENU_BUTTON_WIDTH};

#[cfg(target_arch = "wasm32")]
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, EnumIter, EnumCount, Display,
)]
pub enum MainMenuLayoutEntity {
    Puzzles = 0,
    Store = 1,
    SelfieMode = 2,
    Tutorial = 3,
    ResetPuzzle = 4,
    PlaySteks = 5,
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, EnumIter, EnumCount, Display,
)]
pub enum MainMenuLayoutEntity {
    Puzzles = 0,
    Store = 1,
    SelfieMode = 2,
    Tutorial = 3,
    ResetPuzzle = 4,
}

//#[cfg_attr(target_arch = "wasm32", derive( Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, EnumIter, EnumCount, Display,))]

impl MainMenuLayoutEntity {
    pub fn index(&self) -> usize {
        *self as usize
    }
}

impl LayoutStructure for MainMenuLayoutEntity {
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
            y: TOP_BAR_HEIGHT
                + Spacing::Centre.apply(
                    IDEAL_HEIGHT - TOP_BAR_HEIGHT,
                    MENU_BUTTON_HEIGHT + MENU_BUTTON_SPACING,
                    super::MENU_VIRTUAL_CHILDREN,
                    self.index(),
                ),
        }
    }

    fn iter_all(_context: &Self::Context<'_>) -> impl Iterator<Item = Self> {
        Self::iter()
    }
}

impl LayoutStructureWithFont for MainMenuLayoutEntity {
    type FontContext = ();
    fn font_size(&self, _: &()) -> f32 {
        MENU_BUTTON_FONT_SIZE
    }
}

impl LayoutStructureWithStaticText for MainMenuLayoutEntity {
    fn text(&self, _context: &Self::Context<'_>) -> &'static str {
        use MainMenuLayoutEntity::*;

        match self {
            Puzzles => "Puzzles",
            Store => "Store",
            SelfieMode => "Selfie Mode",
            Tutorial => "Tutorial",
            ResetPuzzle => "Reset Puzzle",
            #[cfg(target_arch = "wasm32")]
            PlaySteks => "Play Steks",
        }
    }
}
