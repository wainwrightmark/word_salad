use crate::prelude::*;
use glam::Vec2;

use super::consts::*;

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct LayoutWordTile(pub usize);

impl From<usize> for LayoutWordTile {
    fn from(value: usize) -> Self {
        Self(value)
    }
}

impl LayoutStructure for LayoutWordTile {
    type Context = Vec<DisplayWord>;
    type Iterator = LayoutWordTileIter;

    fn iter_all(context: &Self::Context) -> Self::Iterator {
        LayoutWordTileIter {
            inner: 0,
            total_count: context.len(),
        }
    }

    fn pick(point: Vec2, context: &Self::Context) -> Option<Self> {
        let parent_loc = super::GameLayoutEntity::WordList.location(&());
        let point = point - parent_loc;
        FlexLayout::Row.try_pick(
            super::GameLayoutEntity::WordList.size(&()),
            point,
            context,
            WORD_MAIN_PAD,
            WORD_CROSS_PAD,
        )
    }

    fn size(&self, context: &Self::Context) -> Vec2 {
        let num_letters = context
            .get(self.0)
            .map(|x| x.graphemes.len())
            .unwrap_or_default();

        let width = WORD_WIDTH_FIXED + (num_letters as f32 * WORD_WIDTH_PER_CHARACTER);

        Vec2 {
            x: width,
            y: WORD_HEIGHT,
        }
    }

    fn location(&self, context: &Self::Context) -> Vec2 {
        let parent_loc = super::GameLayoutEntity::WordList.location(&());

        let offset = FlexLayout::Row.get_location(
            super::GameLayoutEntity::WordList.size(&()),
            self,
            context,
            WORD_MAIN_PAD,
            WORD_CROSS_PAD,
        );

        parent_loc + offset
    }
}

impl LayoutStructureWithFont for LayoutWordTile {
    fn font_size(&self) -> f32 {
        30.0
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LayoutWordTileIter {
    inner: usize,
    total_count: usize,
}

impl Iterator for LayoutWordTileIter {
    type Item = LayoutWordTile;

    fn next(&mut self) -> Option<Self::Item> {
        if self.inner >= self.total_count {
            return None;
        }

        let ret = LayoutWordTile(self.inner);
        self.inner += 1;
        Some(ret)
    }
}
