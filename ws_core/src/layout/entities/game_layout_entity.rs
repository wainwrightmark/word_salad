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
    fn size(&self, _context: &Self::Context<'_>, _sizing: &LayoutSizing) -> Vec2 {
        match self {
            GameLayoutEntity::TopBar => Vec2 {
                x: IDEAL_WIDTH,
                y: (TOP_BAR_HEIGHT),
            },
            GameLayoutEntity::LevelInfo => Vec2 {
                x: GRID_SIZE,
                y: THEME_HEIGHT + THEME_INFO_HEIGHT,
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
            GameLayoutEntity::TopBar => Vec2 { x: 0.0, y: TOP_BAR_OFFSET },

            GameLayoutEntity::LevelInfo => Vec2 {
                x: LEFT_MARGIN,
                y: TOP_BAR_HEIGHT + TOP_BAR_OFFSET + (extra_top_height(sizing, context) * 0.25),
            },

            GameLayoutEntity::Grid => Vec2 {
                x: LEFT_MARGIN,
                y: (TOP_BAR_HEIGHT + TOP_BAR_OFFSET + extra_top_height(sizing, context))
                    + THEME_HEIGHT
                    + THEME_INFO_HEIGHT
                    + GRID_THEME_SPACER,
            },
            GameLayoutEntity::WordList => {
                let y = (TOP_BAR_HEIGHT + TOP_BAR_OFFSET + extra_top_height(sizing, context))
                    + THEME_HEIGHT
                    + THEME_INFO_HEIGHT
                    + GRID_SIZE
                    + GRID_THEME_SPACER
                    + GRID_WORD_LIST_SPACER;

                Vec2 {
                    x: LEFT_MARGIN,
                    y,
                }
            }
        }
    }
}
