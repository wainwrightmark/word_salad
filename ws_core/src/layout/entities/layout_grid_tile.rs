use super::*;
use crate::prelude::*;
use glam::Vec2;
use std::ops::Add;

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct LayoutGridTile(pub Tile);

impl LayoutStructure for LayoutGridTile {
    type Context<'a> = (SelfieMode, Insets);

    fn pick(point: Vec2, context: &Self::Context<'_>, sizing: &LayoutSizing) -> Option<Self> {
        let grid_rect = GameLayoutEntity::Grid.rect(context, sizing);

        let scaled = grid_rect.scaled_inside(point)?;

        let x = (scaled.x * 4.0).floor() as u8;
        let y = (scaled.y * 4.0).floor() as u8;

        let tile = Self(Tile::try_new(x, y)?);

        if tile.rect(context, sizing).contains(point) {
            return Some(tile);
        }
        None
    }

    fn size(&self, _context: &Self::Context<'_>, _sizing: &LayoutSizing) -> Vec2 {
        Vec2 {
            x: GRID_TILE_SIZE,
            y: GRID_TILE_SIZE,
        }
    }

    fn location(&self, context: &Self::Context<'_>, sizing: &LayoutSizing) -> Vec2 {
        GameLayoutEntity::Grid
            .location(context, sizing)
            .add(tile_offset(
                self.0,
                Spacing::SpaceBetween,
                Spacing::SpaceBetween,
                GameLayoutEntity::Grid.size(context, sizing),
                self.size(context, sizing),
            ))
    }

    fn iter_all(_context: &Self::Context<'_>) -> impl Iterator<Item = Self> {
        LayoutGridTileIter::default()
    }
}

impl LayoutStructureWithFont for LayoutGridTile {
    type FontContext = ();

    fn font_size(&self, _: &()) -> f32 {
        GRID_TILE_FONT_SIZE
    }
}

#[derive(Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LayoutGridTileIter {
    //TODO replace with array iter
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
