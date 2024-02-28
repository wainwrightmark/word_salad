use crate::prelude::*;
use glam::Vec2;

use super::{consts::*, SelfieMode};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct WordSaladLogo;

impl LayoutStructure for WordSaladLogo {
    type Context<'a> = SelfieMode;

    fn size(&self, _context: &Self::Context<'_>, _sizing: &LayoutSizing) -> Vec2 {
        Vec2 {
            x: (TOP_BAR_HEIGHT_BASE),
            y: (TOP_BAR_HEIGHT_BASE),
        }
    }

    fn location(&self, context: &Self::Context<'_>, sizing: &LayoutSizing) -> Vec2 {
        Vec2 {
            x: (IDEAL_WIDTH - (TOP_BAR_HEIGHT_BASE)) / 2.,
            y: extra_top_bar_height(sizing, context) / 2.,
        }
    }

    fn iter_all(_context: &Self::Context<'_>) -> impl Iterator<Item = Self> {
        [Self].into_iter()
    }
}
