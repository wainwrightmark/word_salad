use glam::Vec2;
use strum::{Display, EnumCount, EnumIter, IntoEnumIterator};

use crate::prelude::*;

use super::consts::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumIter, EnumCount, Display)]
pub enum CongratsStatistic {
    Left = 0,
    Middle = 1,
    Right = 2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumIter, EnumCount, Display)]
pub enum CongratsButton {
    Next = 0,
    #[cfg(target_arch = "wasm32")]
    Share = 1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumCount, Display)]
pub enum CongratsLayoutEntity {
    Statistic(CongratsStatistic),
    Button(CongratsButton),
}

#[derive(Debug, PartialEq)]
pub struct SelfieMode(pub bool);

impl LayoutStructure for CongratsLayoutEntity {
    type Context = SelfieMode;

    fn size(&self, _context: &Self::Context) -> Vec2 {
        match self {
            CongratsLayoutEntity::Statistic(_) => Vec2 {
                x: CONGRATS_ENTITY_STATISTIC_WIDTH,
                y: CONGRATS_ENTITY_STATISTIC_HEIGHT,
            },
            CongratsLayoutEntity::Button(_) => Vec2 {
                x: CONGRATS_ENTITY_BUTTON_WIDTH,
                y: CONGRATS_ENTITY_BUTTON_HEIGHT,
            },
        }
    }

    fn location(&self, context: &Self::Context) -> Vec2 {
        let top_offset = if context.0 {
            TOP_BAR_HEIGHT + THEME_HEIGHT + GRID_TILE_SIZE + STREAMING_TOP_OFFSET
        } else {
            TOP_BAR_HEIGHT + THEME_HEIGHT + GRID_TILE_SIZE
        };

        match self {
            CongratsLayoutEntity::Statistic(statistic) => Vec2 {
                x: Spacing::SpaceBetween.apply(
                    GRID_SIZE,
                    CONGRATS_ENTITY_STATISTIC_WIDTH * 1.2,
                    CongratsStatistic::COUNT,
                    *statistic as usize,
                ) + ((IDEAL_WIDTH - GRID_SIZE) * 0.5),
                y: top_offset,
            },
            CongratsLayoutEntity::Button(button) => Vec2 {
                x: Spacing::SpaceAround.apply(
                    IDEAL_WIDTH,
                    CONGRATS_ENTITY_BUTTON_WIDTH * 1.2,
                    CongratsButton::COUNT,
                    *button as usize,
                ),
                y: top_offset + CONGRATS_ENTITY_STATISTIC_HEIGHT + CONGRATS_ENTITY_VERTICAL_GAP,
            },
        }
    }

    fn iter_all(_context: &Self::Context) -> impl Iterator<Item = Self> {
        CongratsStatistic::iter()
            .map(|x| Self::Statistic(x))
            .chain(CongratsButton::iter().map(|x| Self::Button(x)))
    }
}

impl LayoutStructureWithFont for CongratsLayoutEntity {
    fn font_size(&self) -> f32 {
        30.0
    }
}
