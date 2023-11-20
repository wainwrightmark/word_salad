use super::*;
use crate::prelude::*;
use glam::Vec2;
use std::ops::Add;

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct LayoutGridTile(pub Tile);

impl LayoutStructure for LayoutGridTile {
    type Context = ();
    type Iterator = LayoutGridTileIter;

    fn pick(point: Vec2, context: &Self::Context) -> Option<Self> {
        let grid_rect = GameLayoutEntity::Grid.rect(context);

        let scaled = grid_rect.scaled_inside(point)?;

        let x = (scaled.x * 4.0).floor() as u8;
        let y = (scaled.y * 4.0).floor() as u8;

        let tile = Self(Tile::try_new(x, y)?);

        if tile.rect(context).contains(point) {
            return Some(tile);
        }
        return None;
    }

    fn size(&self, _context: &Self::Context) -> Vec2 {
        Vec2 {
            x: GRID_TILE_SIZE,
            y: GRID_TILE_SIZE,
        }
    }

    fn location(&self, context: &Self::Context) -> Vec2 {
        GameLayoutEntity::Grid.location(context).add(tile_offset(
            self.0,
            Spacing::SpaceAround,
            Spacing::SpaceAround,
            GameLayoutEntity::Grid.size(context),
            self.size(context),
        ))
    }

    fn iter_all(_context: &Self::Context) -> Self::Iterator {
        LayoutGridTileIter::default()
    }
}

impl LayoutStructureWithText for LayoutGridTile {
    fn font_size() -> f32 {
        60.0
    }
}

#[derive(Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LayoutGridTileIter {
    inner: u8,
}

impl Iterator for LayoutGridTileIter {
    type Item = LayoutGridTile;

    fn next(&mut self) -> Option<Self::Item> {
        let ret = geometrid::tile::Tile::<4, 4>::try_from_inner(self.inner)?;
        self.inner = self.inner.saturating_add(1);
        Some(LayoutGridTile(ret))
    }
}
