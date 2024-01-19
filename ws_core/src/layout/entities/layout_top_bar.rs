use crate::prelude::*;
use glam::Vec2;
use strum::{Display, EnumCount, EnumIter, IntoEnumIterator};

use super::consts::*;

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Display, EnumIter, EnumCount,
)]
pub enum LayoutTopBar {
    MenuBurgerButton,
    WordSaladLogo,
    HintCounter,
}

impl LayoutTopBar {
    pub const fn index(&self) -> usize {
        match self {
            LayoutTopBar::MenuBurgerButton => 0,
            LayoutTopBar::WordSaladLogo => 1,
            LayoutTopBar::HintCounter => 2,
        }
    }
}

impl LayoutStructure for LayoutTopBar {
    type Context<'a> = ();

    fn size(&self, _context: &Self::Context<'_>) -> Vec2 {
        use LayoutTopBar::*;
        match self {
            MenuBurgerButton | HintCounter => Vec2 {
                x: TOP_BAR_ICON_WIDTH,
                y: TOP_BAR_HEIGHT,
            },
            WordSaladLogo => Vec2 {
                x: WORD_SALAD_LOGO_WIDTH,
                y: TOP_BAR_HEIGHT,
            },
        }
    }

    fn location(&self, _context: &Self::Context<'_>, _sizing: &LayoutSizing) -> Vec2 {
        match self {
            LayoutTopBar::MenuBurgerButton => Vec2 {
                x: (IDEAL_WIDTH - GRID_SIZE) * 0.5,
                y: 0.,
            },
            LayoutTopBar::HintCounter => Vec2 {
                x: ((IDEAL_WIDTH + GRID_SIZE) * 0.5) - TOP_BAR_ICON_WIDTH,
                y: 0.,
            },
            LayoutTopBar::WordSaladLogo => Vec2 {
                x: (IDEAL_WIDTH - WORD_SALAD_LOGO_WIDTH) / 2.,
                y: 5.0,
            },
        }
    }

    fn iter_all(_context: &Self::Context<'_>) -> impl Iterator<Item = Self> {
        Self::iter()
    }
}

impl LayoutStructureWithFont for LayoutTopBar {
    type FontContext = ();
    fn font_size(&self, _: &()) -> f32 {
        match self {
            LayoutTopBar::MenuBurgerButton => BURGER_FONT_SIZE,
            LayoutTopBar::WordSaladLogo => LOGO_FONT_SIZE,
            LayoutTopBar::HintCounter => HINT_COUNTER_FONT_SIZE,
        }
    }
}
