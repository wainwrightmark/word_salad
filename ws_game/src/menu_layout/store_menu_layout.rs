use bevy::math::Vec2;
use strum::{Display, EnumIter, IntoEnumIterator};
use ws_core::{
    layout::entities::*, LayoutSizing, LayoutStructure, LayoutStructureWithFont,
    LayoutStructureWithTextOrImage, Spacing,
};

use super::{MENU_BUTTON_HEIGHT, MENU_BUTTON_SPACING, MENU_BUTTON_WIDTH};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Display, EnumIter)]
pub enum StoreLayoutEntity {
    RemoveAds,
    BuyHints,
    LevelGroups,
}

impl StoreLayoutEntity {
    pub fn index(&self) -> usize {
        *self as usize
    }
}

impl LayoutStructure for StoreLayoutEntity {
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

impl LayoutStructureWithFont for StoreLayoutEntity {
    type FontContext = ();

    fn font_size(&self, _context: &Self::FontContext) -> f32 {
        MENU_BUTTON_FONT_SIZE
    }
}

impl LayoutStructureWithTextOrImage for StoreLayoutEntity {
    fn text_or_image(&self, _context: &Self::Context<'_>) -> ws_core::prelude::TextOrImage {
        match self {
            StoreLayoutEntity::RemoveAds => ws_core::TextOrImage::Text { text: "Remove Ads" },
            StoreLayoutEntity::BuyHints => ws_core::TextOrImage::Text { text: "Buy Hints" },
            StoreLayoutEntity::LevelGroups => ws_core::TextOrImage::Text { text: "Buy Addons" },
        }
    }
}
