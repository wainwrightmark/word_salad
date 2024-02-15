use bevy::math::Vec2;
use strum::{Display, EnumCount, EnumIter, IntoEnumIterator};
use ws_core::{
    layout::entities::*, LayoutSizing, LayoutStructure, LayoutStructureWithFont,
    LayoutStructureWithTextOrImage, Spacing,
};

use super::{MENU_BUTTON_HEIGHT, MENU_BUTTON_SPACING, MENU_BUTTON_WIDTH};

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
    Settings = 5,
    PlaySteks = 6,
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, EnumIter, EnumCount, Display,
)]
pub enum MainMenuLayoutEntity {
    Puzzles = 0,
    SelfieMode = 1,
    Tutorial = 2,
    ResetPuzzle = 3,
    Settings = 4,
}

//#[cfg_attr(target_arch = "wasm32", derive( Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, EnumIter, EnumCount, Display,))]

impl MainMenuLayoutEntity {
    pub fn index(&self) -> usize {
        *self as usize
    }
}

impl LayoutStructure for MainMenuLayoutEntity {
    type Context<'a> = SelfieMode;

    fn size(&self, _context: &Self::Context<'_>, _sizing: &LayoutSizing) -> Vec2 {
        Vec2 {
            x: MENU_BUTTON_WIDTH,
            y: MENU_BUTTON_HEIGHT,
        }
    }

    fn location(&self, context: &Self::Context<'_>, sizing: &LayoutSizing) -> Vec2 {
        Vec2 {
            x: (IDEAL_WIDTH - MENU_BUTTON_WIDTH) / 2.,
            y: (TOP_BAR_HEIGHT_BASE + extra_top_bar_height(sizing, context))
                + Spacing::Centre.apply(
                    IDEAL_HEIGHT - (TOP_BAR_HEIGHT_BASE + extra_top_bar_height(sizing, context)),
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

    fn font_size(&self, _context: &Self::FontContext) -> f32 {
        MENU_BUTTON_FONT_SIZE
    }
}

impl LayoutStructureWithTextOrImage for MainMenuLayoutEntity {
    fn text_or_image(&self, _context: &Self::Context<'_>) -> ws_core::prelude::TextOrImage {
        use MainMenuLayoutEntity::*;

        match self {
            Puzzles => ws_core::TextOrImage::Text { text: "Puzzles" },

            SelfieMode => ws_core::TextOrImage::Text {
                text: "Selfie Mode",
            },
            Tutorial => ws_core::TextOrImage::Text { text: "Tutorial" },
            ResetPuzzle => ws_core::TextOrImage::Text {
                text: "Reset Puzzle",
            },
            #[cfg(target_arch = "wasm32")]
            Store => ws_core::TextOrImage::Text { text: "Store" },
            #[cfg(target_arch = "wasm32")]
            PlaySteks => ws_core::TextOrImage::Image {
                path: "images/steks_button.png",
                color: ws_core::BasicColor::rgba(0.53, 0.68, 0.92, 1.0),
                pressed_color: ws_core::BasicColor::rgba(0.36, 0.55, 0.88, 1.0),
                aspect_ratio: 7168.0 / 1024.0,
            },
            Settings => ws_core::TextOrImage::Text { text: "Settings" },
        }
    }
}
