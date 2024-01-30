use crate::prelude::*;
use glam::Vec2;
use strum::{Display, EnumCount, EnumIter, IntoEnumIterator};

use super::{consts::*, SelfieMode};

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Display, EnumIter, EnumCount,
)]
pub enum LayoutTopBar {
    ToggleRecordingButton,
    WordSaladLogo,
}

impl LayoutStructure for LayoutTopBar {
    type Context<'a> = SelfieMode;

    fn size(&self, context: &Self::Context<'_>, sizing: &LayoutSizing) -> Vec2 {
        use LayoutTopBar::*;
        match self {
            ToggleRecordingButton => Vec2 {
                x: TOP_BAR_ICON_WIDTH,
                y: (TOP_BAR_HEIGHT_BASE + extra_top_bar_height(sizing, context)),
            },
            WordSaladLogo => Vec2 {
                x: (TOP_BAR_HEIGHT_BASE + extra_top_bar_height(sizing, context)),
                y: (TOP_BAR_HEIGHT_BASE + extra_top_bar_height(sizing, context)),
            },
        }
    }

    fn location(&self, context: &Self::Context<'_>, sizing: &LayoutSizing) -> Vec2 {
        match self {
            LayoutTopBar::ToggleRecordingButton => Vec2 {
                x: (IDEAL_WIDTH - GRID_SIZE) * 0.5,
                y: 0.,
            },
            LayoutTopBar::WordSaladLogo => Vec2 {
                x: (IDEAL_WIDTH - (TOP_BAR_HEIGHT_BASE + extra_top_bar_height(sizing, context)))
                    / 2.,
                y: 0.,
            },
        }
    }

    fn iter_all(_context: &Self::Context<'_>) -> impl Iterator<Item = Self> {
        Self::iter()
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Default)]
pub struct HintCount {
    pub count: usize,
}

impl LayoutStructureWithFont for LayoutTopBar {
    type FontContext = ();
    fn font_size(&self, _context: &Self::FontContext) -> f32 {
        match self {
            LayoutTopBar::ToggleRecordingButton => BURGER_FONT_SIZE,
            LayoutTopBar::WordSaladLogo => LOGO_FONT_SIZE,
        }
    }
}
