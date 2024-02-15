use bevy::math::Vec2;
use strum::IntoEnumIterator;
use ws_core::{
    layout::entities::*, LayoutSizing, LayoutStructure, LayoutStructureWithFont,
    LayoutStructureWithTextOrImage, Spacing,
};
use ws_levels::level_group::LevelGroup;

use super::{MENU_BUTTON_HEIGHT, MENU_BUTTON_SPACING, MENU_BUTTON_WIDTH};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct LevelGroupStoreLayoutEntity(pub LevelGroup);

impl LevelGroupStoreLayoutEntity {
    pub fn index(&self) -> usize {
        self.0 as usize
    }
}

impl LayoutStructure for LevelGroupStoreLayoutEntity {
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
        LevelGroup::iter().map(|lg| Self(lg))
    }
}

impl LayoutStructureWithFont for LevelGroupStoreLayoutEntity {
    type FontContext = ();

    fn font_size(&self, _context: &Self::FontContext) -> f32 {
        MENU_BUTTON_FONT_SIZE
    }
}

impl LayoutStructureWithTextOrImage for LevelGroupStoreLayoutEntity {
    fn text_or_image(&self, _context: &Self::Context<'_>) -> ws_core::prelude::TextOrImage {
        ws_core::TextOrImage::Text {
            text: self.0.name(),
        }
    }
}
