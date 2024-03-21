use std::ops::Add;

use glam::Vec2;

use crate::prelude::*;
use crate::LayoutStructure;

use super::GameLayoutEntity;
use super::SelfieMode;
use super::GRID_TILE_SIZE;

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct LayoutAnimatedTile {
    pub index: usize,
}

impl LayoutStructure for LayoutAnimatedTile {
    type Context<'a> = ((SelfieMode, Insets), usize); //number of tiles

    fn size(&self, context: &Self::Context<'_>, _sizing: &crate::LayoutSizing) -> glam::Vec2 {
        let size = if context.1 <= 4 {
            GRID_TILE_SIZE
        } else {
            (GRID_TILE_SIZE * 4.0) / (context.1 as f32)
        };

        Vec2 { x: size, y: size }
    }

    fn location(&self, context: &Self::Context<'_>, sizing: &crate::LayoutSizing) -> glam::Vec2 {
        let my_size = self.size(context, sizing);
        let grid_size = GameLayoutEntity::Grid.size(&context.0, sizing);
        let tile_offset = {
            let x = Spacing::SpaceBetween.apply(grid_size.x, my_size.x, context.1, self.index);
            let y = grid_size.y * 0.5;

            Vec2 { x, y }
        };

        GameLayoutEntity::Grid
            .location(&context.0, sizing)
            .add(tile_offset)
    }

    fn iter_all(context: &Self::Context<'_>) -> impl Iterator<Item = Self> {
        (0..context.1).map(|index| Self { index })
    }
}
