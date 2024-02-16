use bevy::math::Vec2;
use strum::{Display, EnumIter, IntoEnumIterator};
use ws_core::{
    layout::entities::*, palette, LayoutSizing, LayoutStructure, LayoutStructureDoubleTextButton,
    LayoutStructureWithFont, LayoutStructureWithTextOrImage, Spacing,
};

use crate::{prelude::BUTTONS_FONT_PATH, view::MenuContextWrapper};

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

impl LayoutStructureDoubleTextButton for HintsLayoutEntity {
    type TextContext<'a> = MenuContextWrapper<'a>;

    fn double_text(
        &self,
        _context: &Self::Context<'_>,
        _text_context: &Self::TextContext<'_>,
    ) -> (String, String) {
        //todo get price from store
        let (left, right) = match self {
            HintsLayoutEntity::Hints5WatchAd => ("  5 Hints", "Watch Ad"),
            HintsLayoutEntity::Hints25 => (" 25 Hints", "£0.99"),
            HintsLayoutEntity::Hints50 => (" 50 Hints", "£1.49"),
            HintsLayoutEntity::Hints100 => ("100 Hints", "£1.99"),
            HintsLayoutEntity::Hints500 => ("500 Hints", "£2.99"),
        };
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
        _context: &Self::Context<'_>,
        _text_context: &Self::TextContext<'_>,
    ) -> ws_core::prelude::BasicColor {
        palette::MENU_BUTTON_FILL
    }

    fn is_disabled(
        &self,
        _context: &Self::Context<'_>,
        _text_context: &Self::TextContext<'_>,
    )-> bool {
        false
    }
}
