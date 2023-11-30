use crate::prelude::*;
use glam::Vec2;
use strum::{Display, EnumCount, EnumIter, IntoEnumIterator};

use super::consts::*;

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Display, EnumIter, EnumCount,
)]
pub enum LayoutTopBarButton {
    MenuBurgerButton,
    WordSaladButton,
    HintCounter,
}

impl LayoutTopBarButton {
    pub const fn index(&self) -> usize {
        match self {
            LayoutTopBarButton::MenuBurgerButton => 0,
            LayoutTopBarButton::WordSaladButton => 1,
            LayoutTopBarButton::HintCounter => 2,
        }
    }
}

impl LayoutStructure for LayoutTopBarButton {
    type Context = ();
    type Iterator = <Self as IntoEnumIterator>::Iterator;

    fn pick(point: Vec2, context: &Self::Context) -> Option<Self> {
        for x in Self::iter() {
            if x.rect(context).contains(point) {
                return Some(x);
            }
        }
        return None;
    }

    fn size(&self, _context: &Self::Context) -> Vec2 {
        use LayoutTopBarButton::*;
        match self {
            MenuBurgerButton | HintCounter => Vec2 {
                x: TOP_BAR_ICON_SIZE,
                y: TOP_BAR_ICON_SIZE,
            },
            WordSaladButton => Vec2 {
                x: WORD_SALAD_LOGO_WIDTH,
                y: TOP_BAR_ICON_SIZE,
            },
        }
    }

    fn location(&self, _context: &Self::Context) -> Vec2 {

        match self{
            LayoutTopBarButton::MenuBurgerButton | LayoutTopBarButton::HintCounter => {
                Vec2 {
                    x: Spacing::SpaceBetween.apply(IDEAL_WIDTH, TOP_BAR_ICON_SIZE, 3, self.index()),
                    y: 0.,
                }
            },
            LayoutTopBarButton::WordSaladButton => {
                Vec2{
                    x: (IDEAL_WIDTH - WORD_SALAD_LOGO_WIDTH) / 2.,
                    y: 0.0
                }
            },
        }


    }

    fn iter_all(_context: &Self::Context) -> Self::Iterator {
        Self::iter()
    }
}

impl LayoutStructureWithFont for LayoutTopBarButton {
    fn font_size() -> f32 {
        28.0
    }
}
