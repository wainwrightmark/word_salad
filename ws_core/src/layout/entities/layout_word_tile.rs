use std::ops::Add;

use glam::Vec2;
use crate::prelude::*;

use super::consts::*;


#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct LayoutWordTile(pub WordTile);

impl LayoutStructure for LayoutWordTile {
    type Context = ();
    type Iterator = LayoutWordTileIter;

    fn iter_all() -> Self::Iterator {
        LayoutWordTileIter::default()
    }

    fn pick(point: Vec2, context: &Self::Context) -> Option<Self> {
        for x in Self::iter_all() {
            if x.rect(context).contains(point) {
                return Some(x);
            }
        }
        return None;
    }

    fn size(&self, _context: &Self::Context) -> Vec2 {
        Vec2 {
            x: WORD_WIDTH,
            y: WORD_HEIGHT,
        }
    }

    fn location(&self, context: &Self::Context) -> Vec2 {
        //todo use flex
        super::GameLayoutEntity::WordList
            .location(context)
            .add(tile_offset(
                self.0,
                Spacing::SpaceAround,
                Spacing::SpaceAround,
                super::GameLayoutEntity::WordList.size(context),
                self.size(context),
            ))
    }
}

impl LayoutStructureWithText for LayoutWordTile{
    fn font_size()-> f32 {
        22.0
    }
}


#[derive(Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)] //TODO use version in geometrid
pub struct LayoutWordTileIter {
    inner: u8,
}

impl Iterator for LayoutWordTileIter {
    type Item = LayoutWordTile;

    fn next(&mut self) -> Option<Self::Item> {
        let ret = geometrid::tile::Tile::<2, 5>::try_from_inner(self.inner)?;
        self.inner = self.inner.saturating_add(1);
        Some(LayoutWordTile(ret))
    }
}