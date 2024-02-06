use crate::prelude::*;
use glam::Vec2;

use super::{consts::*, SelfieMode};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct WordSaladLogo;

impl LayoutStructure for WordSaladLogo {
    type Context<'a> = SelfieMode;

    fn size(&self, context: &Self::Context<'_>, sizing: &LayoutSizing) -> Vec2 {
        Vec2 {
            x: (TOP_BAR_HEIGHT_BASE + extra_top_bar_height(sizing, context)),
            y: (TOP_BAR_HEIGHT_BASE + extra_top_bar_height(sizing, context)),
        }
    }

    fn location(&self, context: &Self::Context<'_>, sizing: &LayoutSizing) -> Vec2 {
        Vec2 {
            x: (IDEAL_WIDTH - (TOP_BAR_HEIGHT_BASE + extra_top_bar_height(sizing, context))) / 2.,
            y: 0.,
        }
    }

    fn iter_all(_context: &Self::Context<'_>) -> impl Iterator<Item = Self> {
        [Self].into_iter()
    }
}
