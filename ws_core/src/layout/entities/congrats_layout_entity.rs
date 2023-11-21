use glam::Vec2;
use strum::{Display, EnumCount, EnumIter, IntoEnumIterator};

use crate::prelude::*;

use super::consts::*;

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, EnumIter, EnumCount, Display,
)]
pub enum CongratsLayoutEntity {
    LevelTime = 0,
    HintsUsed = 1,
    NextButton = 2,
    ShareButton = 3,

}

impl CongratsLayoutEntity {
    pub const fn index(&self) -> usize {
        *self as usize
    }
}

impl LayoutStructure for CongratsLayoutEntity {
    type Context = ();
    type Iterator = <Self as IntoEnumIterator>::Iterator;

    fn pick(point: Vec2, context: &Self::Context) -> Option<Self> {
        for x in Self::iter() {
            if x.rect(context).contains(point) {
                return Some(x);
            }
        }
        return None;
    }

    fn size(&self, _context: &Self::Context) -> Vec2 {
        Vec2 {
            x: CONGRATS_ENTITY_WIDTH,
            y: CONGRATS_ENTITY_HEIGHT,
        }
    }

    fn location(&self, _context: &Self::Context) -> Vec2 {
        Vec2 {
            x: (IDEAL_WIDTH - CONGRATS_ENTITY_WIDTH) / 2.,
            y: TOP_BAR_ICON_SIZE
                + TEXT_AREA_HEIGHT
                + Spacing::Centre.apply(
                    GRID_SIZE,
                    CONGRATS_ENTITY_HEIGHT,
                    Self::COUNT,
                    self.index(),
                ),
        }
    }

    fn iter_all(_context: &Self::Context) -> Self::Iterator {
        Self::iter()
    }
}

impl LayoutStructureWithText for CongratsLayoutEntity {
    fn font_size() -> f32 {
        22.0
    }
}
