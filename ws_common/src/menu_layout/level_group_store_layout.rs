use strum::{EnumCount, IntoEnumIterator};
use ws_core::{
    layout::entities::*, palette, LayoutStructureDoubleTextButton, LayoutStructureWithTextOrImage,
};
use ws_levels::level_group::LevelGroup;

use crate::{prelude::{Product, BUTTONS_FONT_PATH},  view::MenuContextWrapper};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct LevelGroupStoreLayoutStructure(pub LevelGroup);

impl LevelGroupStoreLayoutStructure {
    pub fn index(&self) -> usize {
        self.0 as usize
    }
}

impl MenuButtonsLayout for LevelGroupStoreLayoutStructure {
    type Context = ();
    fn index(&self) -> usize {
        self.0 as usize
    }

    fn count(_context: &Self::Context) -> usize {
        LevelGroup::COUNT
    }

    fn iter_all(_context: &Self::Context) -> impl Iterator<Item = Self> {
        LevelGroup::iter().map(Self)
    }
}

impl LayoutStructureWithTextOrImage for LevelGroupStoreLayoutStructure {
    fn text_or_image(&self, _context: &Self::Context<'_>) -> ws_core::prelude::TextOrImage {
        ws_core::TextOrImage::Text {
            text: self.0.name(),
        }
    }
}

impl LayoutStructureDoubleTextButton for LevelGroupStoreLayoutStructure {
    type TextContext<'a> = MenuContextWrapper<'a>;

    fn double_text(
        &self,
        context: &Self::Context<'_>,
        text_context: &Self::TextContext<'_>,
    ) -> (String, String) {
        let name = self.0.name();
        let price = if self.is_disabled(context, text_context) {
            "Owned".to_string()
        } else {
            let product: Product = self.0.into();
            let price = text_context.prices.get_price_string(product);
            price
        };
        (name.to_string(), price)
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
        text_context.purchases.groups_purchased.contains(&self.0)
    }
}
