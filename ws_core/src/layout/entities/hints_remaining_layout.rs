use glam::Vec2;

use crate::{layout::spacing, LayoutStructure, LayoutStructureWithFont};

use super::{
    GameLayoutEntity, SelfieMode, GRID_SIZE, GRID_WORD_LIST_SPACER, HINTS_REMAINING_FONT_SIZE, HINTS_REMAINING_HEIGHT, IDEAL_WIDTH
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HintsRemainingLayout;

impl LayoutStructure for HintsRemainingLayout {
    type Context<'a> = SelfieMode;

    fn size(&self, _context: &Self::Context<'_>, _sizing: &crate::LayoutSizing) -> glam::Vec2 {
        Vec2 {
            x: GRID_SIZE,
            y: HINTS_REMAINING_HEIGHT,
        }
    }

    fn location(&self, context: &Self::Context<'_>, sizing: &crate::LayoutSizing) -> glam::Vec2 {
        let top = GameLayoutEntity::WordList.location(context, sizing).y - GRID_WORD_LIST_SPACER;



        let y = top + spacing::Spacing::SpaceAround.apply(
            GRID_WORD_LIST_SPACER,
            HINTS_REMAINING_HEIGHT,
            1,
            0,
        );

        Vec2 { x: (IDEAL_WIDTH - GRID_SIZE) * 0.5, y }
    }

    fn iter_all(_context: &Self::Context<'_>) -> impl Iterator<Item = Self> {
        std::iter::once(Self)
    }
}

impl LayoutStructureWithFont for HintsRemainingLayout {
    type FontContext = ();

    fn font_size(&self, _context: &Self::FontContext) -> f32 {
        HINTS_REMAINING_FONT_SIZE
    }
}
