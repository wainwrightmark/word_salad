use glam::Vec2;
use strum::{Display, EnumCount, EnumIter, IntoEnumIterator};
use crate::prelude::*;

use super::consts::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, EnumIter, Display, EnumCount)]
pub enum GameLayoutEntity {
    TopBar,
    TextArea,
    Grid,
    WordList,
}

impl LayoutStructure for GameLayoutEntity {
    type Context = ();
    type Iterator = <Self as IntoEnumIterator>::Iterator;

    fn iter_all(context: &Self::Context) -> Self::Iterator {
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
            GameLayoutEntity::TextArea => Vec2 {
                x: IDEAL_WIDTH,
                y: TEXT_AREA_HEIGHT,
            },
            GameLayoutEntity::Grid => Vec2 {
                x: IDEAL_WIDTH,
                y: IDEAL_WIDTH,
            },

            GameLayoutEntity::WordList => Vec2 {
                x: WORD_LIST_WIDTH,
                y: WORD_LIST_HEIGHT,
            },
        }
    }
    fn location(&self, _context: &()) -> Vec2 {
        match self {
            GameLayoutEntity::TopBar => Vec2::ZERO,

            GameLayoutEntity::TextArea => Vec2 {
                x: 0.,
                y: TOP_BAR_ICON_SIZE,
            },
            GameLayoutEntity::Grid => Vec2 {
                x: 0.,
                y: TOP_BAR_ICON_SIZE + TEXT_AREA_HEIGHT,
            },
            GameLayoutEntity::WordList => Vec2 {
                x: (IDEAL_WIDTH - WORD_LIST_WIDTH) / 2.,
                y: TOP_BAR_ICON_SIZE + TEXT_AREA_HEIGHT + GRID_SIZE,
            },
        }
    }
}
