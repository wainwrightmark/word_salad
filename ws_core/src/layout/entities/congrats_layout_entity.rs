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

    MoreLevels = 1,

    #[cfg(target_arch = "wasm32")]
    Share = 2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumCount, Display)]
pub enum CongratsLayoutEntity {
    Statistic(CongratsStatistic),
    Button(CongratsButton),
}

#[derive(Debug, PartialEq, Clone, Copy)]
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

        let extra_offset = if context.0{
            SELFIE_MODE_CONGRATS_TOP_OFFSET
        }else{
            0.0
        };

        let top_offset = TOP_BAR_HEIGHT + THEME_HEIGHT + GRID_TILE_SIZE + extra_offset;

        pub const MENU_BUTTON_SPACING: f32 = 40.0 * 0.1;

        let button_count = if context.0{
            1
        } else{
            CongratsButton::COUNT
        };

        match self {
            CongratsLayoutEntity::Statistic(statistic) => Vec2 {
                x: Spacing::SpaceBetween.apply(
                    CONGRATS_ENTITY_BUTTON_WIDTH,
                    CONGRATS_ENTITY_STATISTIC_WIDTH,
                    CongratsStatistic::COUNT,
                    *statistic as usize,
                ) + ((IDEAL_WIDTH - CONGRATS_ENTITY_BUTTON_WIDTH) * 0.5),
                y: top_offset,
            },
            CongratsLayoutEntity::Button(button) => Vec2 {
                x: (IDEAL_WIDTH - CONGRATS_ENTITY_BUTTON_WIDTH) * 0.5,
                y: top_offset
                    + CONGRATS_ENTITY_STATISTIC_HEIGHT
                    + CONGRATS_ENTITY_VERTICAL_GAP
                    + Spacing::Centre.apply(
                        GRID_SIZE
                            - CONGRATS_ENTITY_STATISTIC_HEIGHT
                            - CONGRATS_ENTITY_VERTICAL_GAP
                            - extra_offset,
                        CONGRATS_ENTITY_BUTTON_HEIGHT + MENU_BUTTON_SPACING,
                        button_count,
                        *button as usize,
                    ),
            },
        }
    }

    fn iter_all(context: &Self::Context) -> impl Iterator<Item = Self> {
        let take =
        if context.0{
            4
        }
        else{
            6
        };

        CongratsStatistic::iter()
            .map(|x| Self::Statistic(x))
            .chain(CongratsButton::iter().map(|x| Self::Button(x))).take(take)
    }
}

impl LayoutStructureWithFont for CongratsLayoutEntity {
    fn font_size(&self) -> f32 {
        30.0
    }
}

pub struct StatisticNumber;
pub struct StatisticLabel;

impl LayoutStructureWithFont for StatisticNumber {
    fn font_size(&self) -> f32 {
        60.0
    }
}

impl LayoutStructureWithFont for StatisticLabel {
    fn font_size(&self) -> f32 {
        24.0
    }
}
