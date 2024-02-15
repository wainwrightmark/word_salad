use bevy::math::Vec2;
use strum::{Display, EnumIter, IntoEnumIterator};
use ws_core::{
    layout::entities::*, LayoutSizing, LayoutStructure, LayoutStructureWithFont,
    LayoutStructureWithTextOrImage, Spacing,
};

use super::{MENU_BUTTON_HEIGHT, MENU_BUTTON_SPACING, MENU_BUTTON_WIDTH};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Display, EnumIter)]
pub enum HintsLayoutEntity {
    Hints5WatchAd = 0,
    Hints25 = 1,
    Hints50 = 2,
    Hints100 = 3,
    Hints500 = 4,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Display)]
pub enum PurchaseMethod {
    WatchAd,
    Money,
}

impl HintsLayoutEntity {
    pub fn index(&self) -> usize {
        *self as usize
    }

    pub fn hint_data(&self) -> (PurchaseMethod, usize) {
        use PurchaseMethod::*;
        match self {
            HintsLayoutEntity::Hints5WatchAd => (WatchAd, 5),

            HintsLayoutEntity::Hints25 => (Money, 25),
            HintsLayoutEntity::Hints50 => (Money, 50),
            HintsLayoutEntity::Hints100 => (Money, 100),
            HintsLayoutEntity::Hints500 => (Money, 500),
        }
    }
}

impl LayoutStructure for HintsLayoutEntity {
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

impl LayoutStructureWithFont for HintsLayoutEntity {
    type FontContext = ();

    fn font_size(&self, _context: &Self::FontContext) -> f32 {
        MENU_BUTTON_FONT_SIZE
    }
}

impl LayoutStructureWithTextOrImage for HintsLayoutEntity {
    fn text_or_image(&self, _context: &Self::Context<'_>) -> ws_core::prelude::TextOrImage {
        match self {
            HintsLayoutEntity::Hints5WatchAd => ws_core::TextOrImage::Text {
                text: "5 Hints (Watch Ad)",
            },
            HintsLayoutEntity::Hints25 => ws_core::TextOrImage::Text { text: "25 Hints" },
            HintsLayoutEntity::Hints50 => ws_core::TextOrImage::Text { text: "50 Hints" },
            HintsLayoutEntity::Hints100 => ws_core::TextOrImage::Text { text: "100 Hints" },
            HintsLayoutEntity::Hints500 => ws_core::TextOrImage::Text { text: "500 Hints" },
        }
    }
}
