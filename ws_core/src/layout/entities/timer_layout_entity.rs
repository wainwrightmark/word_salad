use crate::prelude::*;
use glam::Vec2;

use super::{consts::*, GameLayoutEntity, SelfieMode};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TimerLayoutEntity;

impl LayoutStructure for TimerLayoutEntity {
    type Context<'a> = (SelfieMode, Insets);

    fn size(&self, _context: &Self::Context<'_>, _sizing: &LayoutSizing) -> Vec2 {
        Vec2 {
            x: TIMER_WIDTH,
            y: TIMER_HEIGHT,
        }
    }
    fn location(&self, context: &Self::Context<'_>, sizing: &LayoutSizing) -> Vec2 {
        Vec2 {
            x: (IDEAL_WIDTH - TIMER_WIDTH) * 0.5,
            y: GameLayoutEntity::TopBar.location(context, sizing).y + (WORD_SALAD_LOGO_SIZE / 2.),
        }
    }

    fn iter_all(_context: &Self::Context<'_>) -> impl Iterator<Item = Self> {
        [TimerLayoutEntity].into_iter()
    }
}

impl LayoutStructureWithFont for TimerLayoutEntity {
    type FontContext = ();

    fn font_size(&self, _context: &Self::FontContext) -> f32 {
        THEME_INFO_FONT_SIZE
    }
}
