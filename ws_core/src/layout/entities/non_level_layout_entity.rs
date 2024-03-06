use glam::Vec2;
use strum::{Display, EnumCount, EnumIter, IntoEnumIterator};

use crate::prelude::*;

use super::{consts::*, SelfieMode};

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, EnumIter, EnumCount, Display,
)]
pub enum NonLevelLayoutEntity {
    Text = 0,
    InteractButton = 1,
}

impl NonLevelLayoutEntity {
    pub const fn index(&self) -> usize {
        *self as usize
    }
}

impl LayoutStructure for NonLevelLayoutEntity {
    type Context<'a> = SelfieMode;

    fn size(&self, _context: &Self::Context<'_>, _sizing: &LayoutSizing) -> Vec2 {
        match self {
            NonLevelLayoutEntity::Text => Vec2 {
                x: NON_LEVEL_TEXT_WIDTH,
                y: NON_LEVEL_TEXT_HEIGHT,
            },
            NonLevelLayoutEntity::InteractButton => Vec2 {
                x: NON_LEVEL_BUTTON_WIDTH,
                y: NON_LEVEL_BUTTON_HEIGHT,
            },
        }
    }

    fn location(&self, context: &Self::Context<'_>, sizing: &LayoutSizing) -> Vec2 {
        let top_offset =
            GRID_TILE_SIZE + (TOP_BAR_HEIGHT + TOP_BAR_OFFSET + extra_top_height(sizing, context));

        match self {
            NonLevelLayoutEntity::Text => Vec2 {
                x: (IDEAL_WIDTH - NON_LEVEL_TEXT_WIDTH) / 2.,
                y: top_offset,
            },
            NonLevelLayoutEntity::InteractButton => {
                let bottom_padding = sizing.bottom_pad / sizing.size_ratio;
                let total_height = IDEAL_HEIGHT + bottom_padding;

                Vec2 {
                    x: (IDEAL_WIDTH - NON_LEVEL_BUTTON_WIDTH) / 2.,
                    y: (total_height - NON_LEVEL_BUTTON_HEIGHT) / 2.,
                }
            }
        }
    }

    fn iter_all(_context: &Self::Context<'_>) -> impl Iterator<Item = Self> {
        Self::iter()
    }
}

pub enum NonLevelType {
    Normal,
    Countdown,
}

impl LayoutStructureWithFont for NonLevelLayoutEntity {
    type FontContext = NonLevelType;
    fn font_size(&self, context: &Self::FontContext) -> f32 {
        match self {
            NonLevelLayoutEntity::Text => match context {
                NonLevelType::Normal => NON_LEVEL_TEXT_FONT_SIZE,
                NonLevelType::Countdown => NON_LEVEL_COUNTDOWN_FONT_SIZE,
            },
            NonLevelLayoutEntity::InteractButton => NON_LEVEL_TEXT_FONT_SIZE,
        }
    }
}
