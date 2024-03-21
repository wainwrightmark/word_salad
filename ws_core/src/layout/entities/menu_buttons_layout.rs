use std::fmt::Debug;

use glam::Vec2;

use crate::{
    layout::entities::{SelfieMode, MENU_BUTTON_FONT_SIZE},
    LayoutStructure, LayoutStructureWithFont, Spacing,
};

use crate::layout::entities::consts::*;

pub trait MenuButtonsLayout: Debug + PartialEq + Sized {
    type Context;
    fn index(&self) -> usize;
    fn count(context: &Self::Context) -> usize;

    fn iter_all(context: &Self::Context) -> impl Iterator<Item = Self>;

    const FONT_SIZE_SMALL: bool = false;
}

impl<T: MenuButtonsLayout> LayoutStructure for T {
    type Context<'a> = (SelfieMode, T::Context);

    fn size(&self, _context: &Self::Context<'_>, _sizing: &crate::prelude::LayoutSizing) -> Vec2 {
        Vec2 {
            x: MENU_BUTTON_WIDTH,
            y: MENU_BUTTON_HEIGHT,
        }
    }

    fn location(&self, context: &Self::Context<'_>, sizing: &crate::prelude::LayoutSizing) -> Vec2 {
        Vec2 {
            x: (IDEAL_WIDTH - MENU_BUTTON_WIDTH) / 2.,
            y: (TOP_BAR_HEIGHT + TOP_BAR_OFFSET + extra_top_height(sizing, &context.0))
                + Spacing::Centre.apply(
                    IDEAL_HEIGHT
                        - (TOP_BAR_HEIGHT + TOP_BAR_OFFSET + extra_top_height(sizing, &context.0)),
                    (MENU_BUTTON_HEIGHT + MENU_BUTTON_SPACING) * 0.5,
                    super::MENU_VIRTUAL_CHILDREN * 2,
                    (self.index() * 2) + super::MENU_VIRTUAL_CHILDREN
                        - (Self::count(&context.1) + 1),
                ),
        }
    }

    fn iter_all(context: &Self::Context<'_>) -> impl Iterator<Item = Self> {
        Self::iter_all(&context.1)
    }
}

impl<T: MenuButtonsLayout> LayoutStructureWithFont for T {
    type FontContext = ();

    fn font_size(&self, _context: &Self::FontContext) -> f32 {
        if Self::FONT_SIZE_SMALL {
            MENU_BUTTON_FONT_SIZE_SMALL
        } else {
            MENU_BUTTON_FONT_SIZE
        }
    }
}
