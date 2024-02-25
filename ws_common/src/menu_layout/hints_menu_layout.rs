use strum::{Display, EnumCount, EnumIter, IntoEnumIterator};
use ws_core::{
    layout::entities::*, palette, LayoutStructureDoubleTextButton, LayoutStructureWithTextOrImage,
};

use crate::{prelude::{Product, BUTTONS_FONT_PATH},  view::MenuContextWrapper};

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Display, EnumIter, EnumCount,
)]
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

impl MenuButtonsLayout for HintsLayoutEntity {
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

impl LayoutStructureDoubleTextButton for HintsLayoutEntity {
    type TextContext<'a> = MenuContextWrapper<'a>;

    fn double_text(
        &self,
        _context: &Self::Context<'_>,
        text_context: &Self::TextContext<'_>,
    ) -> (String, String) {
        let name = match self {
            HintsLayoutEntity::Hints5WatchAd => "  5 Hints",
            HintsLayoutEntity::Hints25 => " 25 Hints",
            HintsLayoutEntity::Hints50 => " 50 Hints",
            HintsLayoutEntity::Hints100 => "100 Hints",
            HintsLayoutEntity::Hints500 => "500 Hints",
        }
        .to_string();

        let price = match self {
            HintsLayoutEntity::Hints5WatchAd => "Watch Ad".to_string(),

            HintsLayoutEntity::Hints25 => text_context.prices.get_price_string(Product::Hints25),
            HintsLayoutEntity::Hints50 => text_context.prices.get_price_string(Product::Hints50),
            HintsLayoutEntity::Hints100 => text_context.prices.get_price_string(Product::Hints100),
            HintsLayoutEntity::Hints500 => text_context.prices.get_price_string(Product::Hints500),
        };

        (name, price)
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
        _context: &Self::Context<'_>,
        _text_context: &Self::TextContext<'_>,
    ) -> ws_core::prelude::BasicColor {
        palette::MENU_BUTTON_FILL
    }

    fn is_disabled(
        &self,
        _context: &Self::Context<'_>,
        _text_context: &Self::TextContext<'_>,
    ) -> bool {
        false
    }
}
