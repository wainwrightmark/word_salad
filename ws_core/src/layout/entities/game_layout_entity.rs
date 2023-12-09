use crate::prelude::*;
use glam::Vec2;
use strum::{Display, EnumCount, EnumIter, IntoEnumIterator};

use super::consts::*;

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, EnumIter, Display, EnumCount,
)]
pub enum GameLayoutEntity {
    TopBar,
    Theme,
    Grid,
    WordList,
    Timer,
}

impl LayoutStructure for GameLayoutEntity {
    type Context = ();
    type Iterator = <Self as IntoEnumIterator>::Iterator;

    fn iter_all(_context: &Self::Context) -> Self::Iterator {
        Self::iter()
    }

    fn pick(point: Vec2, context: &Self::Context) -> Option<Self> {
        for item in Self::iter() {
            if item.rect(context).contains(point) {
                return Some(item);
            }
        }
        return None;
    }

    //const ROOT: Self = GameLayoutEntity::Root;
    ///The size on a 320x568 canvas
    fn size(&self, _context: &()) -> Vec2 {
        match self {
            GameLayoutEntity::TopBar => Vec2 {
                x: IDEAL_WIDTH,
                y: TOP_BAR_ICON_SIZE,
            },
            GameLayoutEntity::Theme => Vec2 {
                x: THEME_WIDTH,
                y: THEME_HEIGHT,
            },
            GameLayoutEntity::Grid => Vec2 {
                x: GRID_SIZE,
                y: GRID_SIZE,
            },

            GameLayoutEntity::WordList => Vec2 {
                x: WORD_LIST_WIDTH,
                y: WORD_LIST_HEIGHT,
            },
            GameLayoutEntity::Timer => Vec2 {
                x: TIMER_WIDTH,
                y: TIMER_HEIGHT,
            },
        }
    }
    fn location(&self, _context: &()) -> Vec2 {
        match self {
            GameLayoutEntity::TopBar => Vec2::ZERO,

            GameLayoutEntity::Theme => Vec2 {
                x: (IDEAL_WIDTH - THEME_WIDTH) * 0.5,
                y: TOP_BAR_ICON_SIZE,
            },
            GameLayoutEntity::Grid => Vec2 {
                x: (IDEAL_WIDTH - GRID_SIZE) * 0.5,
                y: TOP_BAR_ICON_SIZE + THEME_HEIGHT + (GRID_TILE_SIZE * 0.5),
            },
            GameLayoutEntity::WordList => Vec2 {
                x: (IDEAL_WIDTH - WORD_LIST_WIDTH) / 2.,
                y: TOP_BAR_ICON_SIZE + THEME_HEIGHT + GRID_SIZE + GRID_TILE_SIZE,
            },
            GameLayoutEntity::Timer => Vec2 {
                x: (IDEAL_WIDTH - TIMER_WIDTH) * 0.5,
                y: IDEAL_HEIGHT - TIMER_HEIGHT,
            },
        }
    }
}

impl LayoutStructureWithFont for GameLayoutEntity {
    fn font_size(&self) -> f32 {
        match self {
            GameLayoutEntity::TopBar => f32::NAN,
            GameLayoutEntity::Theme => 32.0,
            GameLayoutEntity::Grid => f32::NAN,
            GameLayoutEntity::WordList => f32::NAN,
            GameLayoutEntity::Timer => 20.0,
        }
    }
}
