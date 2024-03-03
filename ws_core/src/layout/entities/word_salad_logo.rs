use crate::prelude::*;
use glam::Vec2;

use super::{consts::*, GameLayoutEntity, SelfieMode};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct WordSaladLogo;

impl LayoutStructure for WordSaladLogo {
    type Context<'a> = SelfieMode;

    fn size(&self, _context: &Self::Context<'_>, _sizing: &LayoutSizing) -> Vec2 {
        Vec2 {
            x: WORD_SALAD_LOGO_SIZE,
            y: WORD_SALAD_LOGO_SIZE,
        }
    }

    fn location(&self, context: &Self::Context<'_>, sizing: &LayoutSizing) -> Vec2 {
        Vec2 {
            x: LEFT_MARGIN,
            y: GameLayoutEntity::TopBar.location(context, sizing).y,
        }
    }

    fn iter_all(_context: &Self::Context<'_>) -> impl Iterator<Item = Self> {
        [Self].into_iter()
    }
}
