use crate::prelude::*;
use glam::Vec2;
use strum::{Display, EnumCount, EnumIter, IntoEnumIterator};

use super::{consts::*, SelfieMode};

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, EnumIter, Display, EnumCount,
)]
pub enum GameLayoutEntity {
    TopBar,
    LevelInfo,
    Grid,
    WordList,
}

impl LayoutStructure for GameLayoutEntity {
    type Context<'a> = SelfieMode;

    fn iter_all(_context: &Self::Context<'_>) -> impl Iterator<Item = Self> {
        Self::iter()
    }

    //const ROOT: Self = GameLayoutEntity::Root;
    ///The size on a 320x568 canvas
    fn size(&self, _context: &Self::Context<'_>) -> Vec2 {
        match self {
            GameLayoutEntity::TopBar => Vec2 {
                x: IDEAL_WIDTH,
                y: TOP_BAR_HEIGHT,
            },
            GameLayoutEntity::LevelInfo => Vec2 {
                x: GRID_SIZE,
                y: THEME_HEIGHT + TIMER_HEIGHT,
            },
            GameLayoutEntity::Grid => Vec2 {
                x: GRID_SIZE,
                y: GRID_SIZE,
            },

            GameLayoutEntity::WordList => Vec2 {
                x: WORD_LIST_WIDTH,
                y: WORD_LIST_HEIGHT,
            },
        }
    }
    fn location(&self, context: &Self::Context<'_>, sizing: &LayoutSizing) -> Vec2 {
        match self {
            GameLayoutEntity::TopBar => Vec2::ZERO,

            GameLayoutEntity::LevelInfo => Vec2 {
                x: (IDEAL_WIDTH - GRID_SIZE) * 0.5,
                y: THEME_HEIGHT + TIMER_HEIGHT,
            },

            GameLayoutEntity::Grid => Vec2 {
                x: (IDEAL_WIDTH - GRID_SIZE) * 0.5,
                y: TOP_BAR_HEIGHT + THEME_HEIGHT + TIMER_HEIGHT + (GRID_TILE_SIZE * 0.5),
            },
            GameLayoutEntity::WordList => {
                let y = if context.is_selfie_mode {
                    IDEAL_HEIGHT + (sizing.bottom_pad / sizing.size_ratio) - WORD_LIST_HEIGHT
                } else {
                    TOP_BAR_HEIGHT + THEME_HEIGHT + TIMER_HEIGHT + GRID_SIZE + GRID_TILE_SIZE
                };

                Vec2 {
                    x: (IDEAL_WIDTH - GRID_SIZE) / 2.,
                    y,
                }
            }
        }
    }
}
