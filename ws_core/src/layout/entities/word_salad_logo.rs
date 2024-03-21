use crate::prelude::*;
use glam::Vec2;

use super::{consts::*, level_info_entity::IsLevelComplete, GameLayoutEntity, SelfieMode};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct WordSaladLogo;

impl LayoutStructure for WordSaladLogo {
    type Context<'a> = ((SelfieMode, Insets), IsLevelComplete);

    fn size(&self, context: &Self::Context<'_>, _sizing: &LayoutSizing) -> Vec2 {
        if context.1 .0 {
            Vec2::splat(WORD_SALAD_LOGO_SIZE + THEME_HEIGHT)
        } else {
            Vec2 {
                x: WORD_SALAD_LOGO_SIZE,
                y: WORD_SALAD_LOGO_SIZE,
            }
        }
    }

    fn location(&self, context: &Self::Context<'_>, sizing: &LayoutSizing) -> Vec2 {
        if context.1 .0 {
            //level complete
            let width = self.size(context, sizing).x;
            Vec2 {
                x: (IDEAL_WIDTH - width) * 0.5,
                y: GameLayoutEntity::TopBar.location(&context.0, sizing).y,
            }
        } else {
            Vec2 {
                x: LEFT_MARGIN,
                y: GameLayoutEntity::TopBar.location(&context.0, sizing).y,
            }
        }
    }

    fn iter_all(_context: &Self::Context<'_>) -> impl Iterator<Item = Self> {
        [Self].into_iter()
    }
}
