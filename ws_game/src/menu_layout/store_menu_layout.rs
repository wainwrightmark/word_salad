
use strum::{Display, EnumCount, EnumIter, IntoEnumIterator};
use ws_core::{
    layout::entities::*, palette, LayoutStructureDoubleTextButton,
};
use ws_levels::level_group::LevelGroup;

use crate::{prelude::BUTTONS_FONT_PATH, view::MenuContextWrapper};

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Display, EnumIter, EnumCount,
)]
pub enum StoreLayoutStructure {
    RemoveAds = 0,
    BuyHints = 1,
    LevelGroups = 2,
    RefreshPrices = 4,
    RestorePurchases = 5,
}

impl MenuButtonsLayout for StoreLayoutStructure {
    type Context = ();

    fn index(&self) -> usize {
        *self as usize
    }

    fn count(_context: &Self::Context) -> usize {
        Self::COUNT
    }

    fn iter_all(_context: &Self::Context) -> impl Iterator<Item = Self> {
        Self::iter()
    }
}

impl LayoutStructureDoubleTextButton for StoreLayoutStructure {
    type TextContext<'a> = MenuContextWrapper<'a>;

    fn double_text(
        &self,
        context: &Self::Context<'_>,
        text_context: &Self::TextContext<'_>,
    ) -> (String, String) {

        let left: String;
        let right: String;
        let disabled = self.is_disabled(context, text_context);

        match self {
            StoreLayoutStructure::RemoveAds => {
                left = "Remove Ads".to_string();
                right = if disabled {
                    "Owned".to_string()
                } else {
                    text_context.prices.get_price_string(crate::purchases::Product::RemoveAds)
                };
            }
            StoreLayoutStructure::BuyHints => {
                left = "Buy Hints".to_string();
                right = String::new();
            }
            StoreLayoutStructure::LevelGroups => {
                left = "Buy Addons".to_string();
                let complete = text_context.purchases.groups_purchased.len();

                right = String::new();
            }
            StoreLayoutStructure::RefreshPrices => {
                left = "Refresh Prices".to_string();
                right = String::new();
            }
            StoreLayoutStructure::RestorePurchases => {
                left = "Restore Purchases".to_string();
                right = String::new();
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
            StoreLayoutStructure::RemoveAds => text_context.purchases.remove_ads_purchased,
            StoreLayoutStructure::BuyHints => false,
            StoreLayoutStructure::LevelGroups => {
                text_context.purchases.groups_purchased.len() == LevelGroup::COUNT
            }
            StoreLayoutStructure::RefreshPrices => {
                false
            }
             StoreLayoutStructure::RestorePurchases => {
                false
            }
        }
    }
}
