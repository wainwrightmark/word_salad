use crate::prelude::*;
use glam::Vec2;

use super::{consts::*, SelfieMode};

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct LayoutWordTile(pub usize);

impl From<usize> for LayoutWordTile {
    fn from(value: usize) -> Self {
        Self(value)
    }
}

impl LayoutStructure for LayoutWordTile {
    type Context<'a> = (&'a [DisplayWord], SelfieMode);

    fn iter_all(context: &Self::Context<'_>) -> impl Iterator<Item = Self> {
        LayoutWordTileIter {
            inner: 0,
            total_count: context.0.len(),
        }
    }

    fn pick(point: Vec2, context: &Self::Context<'_>, sizing: &LayoutSizing) -> Option<Self> {
        let parent_loc = super::GameLayoutEntity::WordList.location(&context.1, sizing);
        let point = point - parent_loc;
        FlexLayout::Row.try_pick(
            super::GameLayoutEntity::WordList.size(&context.1, sizing),
            point,
            context,
            WORD_MAIN_PAD,
            WORD_CROSS_PAD,
            sizing
        )
    }

    fn size(&self, context: &Self::Context<'_>, _sizing: &LayoutSizing) -> Vec2 {
        let num_letters = context
            .0
            .get(self.0)
            .map(|x| x.graphemes.len())
            .unwrap_or_default();

        let width = WORD_WIDTH_FIXED + (num_letters as f32 * WORD_WIDTH_PER_CHARACTER);

        Vec2 {
            x: width,
            y: WORD_HEIGHT,
        }
    }

    fn location(&self, context: &Self::Context<'_>, sizing: &LayoutSizing) -> Vec2 {
        let parent_loc = super::GameLayoutEntity::WordList.location(&context.1, sizing);

        let offset = FlexLayout::Row.get_location(
            super::GameLayoutEntity::WordList.size(&context.1, sizing),
            self,
            context,
            WORD_MAIN_PAD,
            WORD_CROSS_PAD,
            sizing
        );

        parent_loc + offset
    }
}

impl LayoutStructureWithFont for LayoutWordTile {
    type FontContext = ();
    fn font_size(&self, _: &()) -> f32 {
        WORD_TILE_FONT_SIZE
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
