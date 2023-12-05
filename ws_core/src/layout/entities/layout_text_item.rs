use crate::prelude::*;
use glam::Vec2;
use strum::{Display, EnumCount, EnumIter, IntoEnumIterator};

use super::consts::*;

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Display, EnumIter, EnumCount,
)]
pub enum LayoutTextItem {
    Timer,
    PuzzleTheme,
    FoundWordAnimation,
}

impl LayoutTextItem {
    pub const fn index(&self) -> usize {
        match self {
            LayoutTextItem::Timer => 0,
            LayoutTextItem::PuzzleTheme => 1,
            LayoutTextItem::FoundWordAnimation => 2,
        }
    }
}

impl LayoutStructure for LayoutTextItem {
    type Context = ();
    type Iterator = <Self as IntoEnumIterator>::Iterator;

    fn iter_all(_context: &Self::Context) -> Self::Iterator {
        Self::iter()
    }

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
            x: TEXT_ITEM_WIDTH,
            y: TEXT_ITEM_HEIGHT,
        }
    }

    fn location(&self, _context: &Self::Context) -> Vec2 {
        Vec2 {
            x: (IDEAL_WIDTH - TEXT_ITEM_WIDTH) / 2.,
            y: TOP_BAR_ICON_SIZE
                + Spacing::SpaceAround.apply(
                    TEXT_AREA_HEIGHT,
                    TEXT_ITEM_HEIGHT,
                    Self::COUNT,
                    self.index(),
                ),
        }
    }
}

impl LayoutStructureWithFont for LayoutTextItem {
    fn font_size(&self) -> f32 {
        match self{
            LayoutTextItem::Timer => 24.0,
            LayoutTextItem::PuzzleTheme => 32.0,
            LayoutTextItem::FoundWordAnimation => 32.0,
        }

    }
}
