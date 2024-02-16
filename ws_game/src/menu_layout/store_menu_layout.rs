use bevy::math::Vec2;
use strum::{Display, EnumCount, EnumIter, IntoEnumIterator};
use ws_core::{
    layout::entities::*, palette, LayoutSizing, LayoutStructure, LayoutStructureDoubleTextButton,
    LayoutStructureWithFont, Spacing,
};
use ws_levels::level_group::LevelGroup;

use crate::{prelude::BUTTONS_FONT_PATH, view::MenuContextWrapper};

use super::{MENU_BUTTON_HEIGHT, MENU_BUTTON_SPACING, MENU_BUTTON_WIDTH};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Display, EnumIter)]
pub enum StoreLayoutStructure {
    RemoveAds,
    BuyHints,
    LevelGroups,
}

impl StoreLayoutStructure {
    pub fn index(&self) -> usize {
        *self as usize + 3
    }
}

impl LayoutStructure for StoreLayoutStructure {
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

impl LayoutStructureWithFont for StoreLayoutStructure {
    type FontContext = ();

    fn font_size(&self, _context: &Self::FontContext) -> f32 {
        MENU_BUTTON_FONT_SIZE
    }
}

impl LayoutStructureDoubleTextButton for StoreLayoutStructure {
    type TextContext<'a> = MenuContextWrapper<'a>;

    fn double_text(
        &self,
        context: &Self::Context<'_>,
        text_context: &Self::TextContext<'_>,
    ) -> (String, String) {
        //todo get price from store

        let left: String;
        let right: String;
        let disabled = self.is_disabled(context, text_context);

        match self {
            StoreLayoutStructure::RemoveAds => {
                left = "Remove Ads".to_string();
                right = if disabled {
                    "Owned".to_string()
                } else {
                    "£2.99".to_string()
                };
            }
            StoreLayoutStructure::BuyHints => {
                left = "Buy Hints".to_string();
                right = text_context.hint_state.as_text()
            }
            StoreLayoutStructure::LevelGroups => {
                left = "Buy Addons".to_string();
                let complete = text_context.purchases.groups_purchased.len();
                let total = LevelGroup::COUNT;
                right = format!("{:#}", fmtastic::VulgarFraction::new(complete, total));
            }
        }
        (left.to_string(), right.to_string())
    }

    fn left_font(&self) -> &'static str {
        BUTTONS_FONT_PATH
    }

    fn right_font(&self) -> &'static str {
        BUTTONS_FONT_PATH
    }

    fn text_color(
        &self,
        _context: &Self::Context<'_>,
        _text_context: &Self::TextContext<'_>,
    ) -> ws_core::prelude::BasicColor {
        palette::MENU_BUTTON_TEXT_REGULAR
    }

    fn fill_color(
        &self,
        _background_type: ws_core::prelude::BackgroundType,
        context: &Self::Context<'_>,
        text_context: &Self::TextContext<'_>,
    ) -> ws_core::prelude::BasicColor {
        if self.is_disabled(context, text_context) {
            palette::MENU_BUTTON_COMPLETE_FILL
        } else {
            palette::MENU_BUTTON_FILL
        }
    }

    fn is_disabled(
        &self,
        _context: &Self::Context<'_>,
        text_context: &Self::TextContext<'_>,
    ) -> bool {
        match self {
            StoreLayoutStructure::RemoveAds => text_context.purchases.avoid_ads_purchased,
            StoreLayoutStructure::BuyHints => false,
            StoreLayoutStructure::LevelGroups => {
                text_context.purchases.groups_purchased.len() == LevelGroup::COUNT
            }
        }
    }
}
