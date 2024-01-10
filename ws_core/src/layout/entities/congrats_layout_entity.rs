use glam::Vec2;
use strum::{Display, EnumCount, EnumIter, IntoEnumIterator};

use crate::{prelude::*, level_type::LevelType};

use super::consts::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumIter, EnumCount, Display)]
pub enum CongratsStatistic {
    Left = 0,
    Middle = 1,
    Right = 2,
}

#[cfg(target_arch = "wasm32")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumIter, EnumCount, Display)]
pub enum CongratsButton {
    Next = 0,
    MoreLevels = 1,
    Share = 2,
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumIter, EnumCount, Display)]
pub enum CongratsButton {
    Next = 0,
    MoreLevels = 1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumCount, Display)]
pub enum CongratsLayoutEntity {
    Statistic(CongratsStatistic),
    Button(CongratsButton),
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct SelfieMode {
    pub is_selfie_mode: bool,
}

impl CongratsLayoutEntity{
    pub fn get_button_count(context: &(SelfieMode, LevelType))-> usize{
        if context.0.is_selfie_mode || context.1.is_tutorial(){
            1
        }else{
            CongratsButton::COUNT
        }
    }
}

impl LayoutStructure for CongratsLayoutEntity {
    type Context = (SelfieMode, LevelType);

    fn size(&self, context: &Self::Context) -> Vec2 {
        match self {
            CongratsLayoutEntity::Statistic(_) => {
                let stat_size = if context.0.is_selfie_mode {
                    CONGRATS_ENTITY_STATISTIC_SIZE_SELFIE
                } else {
                    CONGRATS_ENTITY_STATISTIC_SIZE_NORMAL
                };
                Vec2 {
                    x: stat_size,
                    y: stat_size,
                }
            }
            CongratsLayoutEntity::Button(_) => Vec2 {
                x: if context.0.is_selfie_mode {
                    CONGRATS_ENTITY_BUTTON_WIDTH_SELFIE
                } else {
                    CONGRATS_ENTITY_BUTTON_WIDTH_NORMAL
                },
                y: CONGRATS_ENTITY_BUTTON_HEIGHT,
            },
        }
    }

    fn location(&self, context: &Self::Context) -> Vec2 {
        let extra_offset = if context.0.is_selfie_mode {
            SELFIE_MODE_CONGRATS_TOP_OFFSET
        } else {
            0.0
        };

        let top_offset = TOP_BAR_HEIGHT + THEME_HEIGHT + GRID_TILE_SIZE + extra_offset;

        pub const MENU_BUTTON_SPACING: f32 = 40.0 * 0.1;

        let button_count = Self:: get_button_count(context);

        let button_width = if context.0.is_selfie_mode {
            CONGRATS_ENTITY_BUTTON_WIDTH_SELFIE
        } else {
            CONGRATS_ENTITY_BUTTON_WIDTH_NORMAL
        };

        let stat_size = if context.0.is_selfie_mode {
            CONGRATS_ENTITY_STATISTIC_SIZE_SELFIE
        } else {
            CONGRATS_ENTITY_STATISTIC_SIZE_NORMAL
        };

        match self {
            CongratsLayoutEntity::Statistic(statistic) => Vec2 {
                x: Spacing::SpaceBetween.apply(
                    button_width,
                    stat_size,
                    CongratsStatistic::COUNT,
                    *statistic as usize,
                ) + ((IDEAL_WIDTH - button_width) * 0.5),
                y: top_offset,
            },
            CongratsLayoutEntity::Button(button) => Vec2 {
                x: (IDEAL_WIDTH - button_width) * 0.5,
                y: top_offset
                    + stat_size
                    + CONGRATS_ENTITY_VERTICAL_GAP
                    + Spacing::Centre.apply(
                        GRID_SIZE - stat_size - CONGRATS_ENTITY_VERTICAL_GAP - extra_offset,
                        CONGRATS_ENTITY_BUTTON_HEIGHT + MENU_BUTTON_SPACING,
                        button_count,
                        *button as usize,
                    ),
            },
        }
    }

    fn iter_all(context: &Self::Context) -> impl Iterator<Item = Self> {
        let button_count = Self::get_button_count(context);
        let take =  3 + button_count;

        CongratsStatistic::iter()
            .map(|x| Self::Statistic(x))
            .chain(CongratsButton::iter().map(|x| Self::Button(x)))
            .take(take)
    }

    fn pick(point: Vec2, context: &Self::Context) -> Option<Self> {
        Self::iter_all(context).find(|x| x.rect(context).contains(point))
    }

    fn rect(&self, context: &Self::Context) -> LayoutRectangle {
        LayoutRectangle {
            top_left: self.location(context),
            extents: self.size(context),
        }
    }
}

impl LayoutStructureWithFont for CongratsLayoutEntity {
    type FontContext = ();
    fn font_size(&self, _context: &()) -> f32 {
        30.0
    }
}

pub struct StatisticNumber;
pub struct StatisticLabel;

impl LayoutStructureWithFont for StatisticNumber {
    type FontContext = SelfieMode;
    fn font_size(&self, context: &SelfieMode) -> f32 {
        if context.is_selfie_mode {
            40.0
        } else {
            60.0
        }
    }
}

impl LayoutStructureWithFont for StatisticLabel {
    type FontContext = SelfieMode;
    fn font_size(&self, context: &SelfieMode) -> f32 {
        if context.is_selfie_mode {
            18.0
        } else {
            24.0
        }
    }
}
