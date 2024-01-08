use glam::Vec2;
use strum::{Display, EnumCount, EnumIter, IntoEnumIterator};

use crate::prelude::*;

use super::consts::*;

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
    type Context = ();

    fn size(&self, _context: &Self::Context) -> Vec2 {
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

    fn location(&self, _context: &Self::Context) -> Vec2 {
        let top_offset = GRID_TILE_SIZE + TOP_BAR_HEIGHT;

        match self {
            NonLevelLayoutEntity::Text => Vec2 {
                x: (IDEAL_WIDTH - NON_LEVEL_TEXT_WIDTH) / 2.,
                y: top_offset,
            },
            NonLevelLayoutEntity::InteractButton => Vec2 {
                x: (IDEAL_WIDTH - NON_LEVEL_BUTTON_WIDTH) / 2.,
                y: top_offset + NON_LEVEL_TEXT_HEIGHT,
            },
        }
    }

    fn iter_all(_context: &Self::Context) -> impl Iterator<Item = Self> {
        Self::iter()
    }
}

impl LayoutStructureWithFont for NonLevelLayoutEntity {
    type FontContext = ();
    fn font_size(&self,_: &()) -> f32 {
        30.0
    }
}
