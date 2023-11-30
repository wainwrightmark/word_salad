use bevy::math::Vec2;
use strum::{Display, EnumCount, EnumIter, IntoEnumIterator};
use ws_core::{
    layout::entities::{IDEAL_HEIGHT, IDEAL_WIDTH, TOP_BAR_ICON_SIZE},
    LayoutStructure, LayoutStructureWithFont, LayoutStructureWithStaticText, Spacing,
};

use super::{MENU_BUTTON_FONT_SIZE, MENU_BUTTON_HEIGHT, MENU_BUTTON_WIDTH, MENU_BUTTON_PADDING_RATIO};

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, EnumIter, EnumCount, Display,
)]
pub enum MainMenuLayoutEntity {
    Puzzles = 0,
    Store = 1,
    SelfieMode = 2,
    Tutorial = 3,
    ResetPuzzle = 4
}

impl MainMenuLayoutEntity {
    pub fn index(&self) -> usize {
        *self as usize
    }
}

impl LayoutStructure for MainMenuLayoutEntity {
    type Context = ();
    type Iterator = <Self as IntoEnumIterator>::Iterator;

    fn pick(point: Vec2, context: &Self::Context) -> Option<Self> {
        for x in Self::iter() {
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
            y: TOP_BAR_ICON_SIZE
                + Spacing::Centre.apply(
                    IDEAL_HEIGHT - TOP_BAR_ICON_SIZE - (MENU_BUTTON_HEIGHT * MENU_BUTTON_PADDING_RATIO),
                    MENU_BUTTON_HEIGHT * MENU_BUTTON_PADDING_RATIO,
                    Self::COUNT,
                    self.index(),
                ),
        }
    }

    fn iter_all(_context: &Self::Context) -> Self::Iterator {
        Self::iter()
    }
}

impl LayoutStructureWithFont for MainMenuLayoutEntity {
    fn font_size() -> f32 {
        MENU_BUTTON_FONT_SIZE
    }
}

impl LayoutStructureWithStaticText for MainMenuLayoutEntity {
    fn text(&self, _context: &Self::Context) -> &'static str {
        use MainMenuLayoutEntity::*;

        match self{
            Puzzles => "Puzzles",
            Store => "Store",
            SelfieMode => "Selfie Mode",
            Tutorial => "Tutorial",
            ResetPuzzle => "Reset Puzzle",
        }

    }
}
