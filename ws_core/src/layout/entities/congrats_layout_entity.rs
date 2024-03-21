use glam::Vec2;
use strum::{Display, EnumCount, EnumIs, EnumIter, IntoEnumIterator};

use crate::{level_type::LevelType, prelude::*};

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
    ResetPuzzle = 3,
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumIter, EnumCount, Display)]
pub enum CongratsButton {
    Next = 0,
    MoreLevels = 1,
    ResetPuzzle = 2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumCount, Display, EnumIs)]
pub enum CongratsLayoutEntity {
    Time,
    Statistic(CongratsStatistic),
    Button(CongratsButton),
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct SelfieMode {
    pub is_selfie_mode: bool,
}

impl CongratsLayoutEntity {
    pub fn get_button_count(context: &(SelfieMode, LevelType)) -> usize {
        if context.0.is_selfie_mode || context.1.is_tutorial() {
            1
        } else {
            CongratsButton::COUNT
        }
    }
}

impl LayoutStructure for CongratsLayoutEntity {
    type Context<'a> = (SelfieMode, LevelType);

    fn size(&self, context: &Self::Context<'_>, _sizing: &LayoutSizing) -> Vec2 {
        match self {
            CongratsLayoutEntity::Time => {
                if context.0.is_selfie_mode {
                    Vec2 {
                        x: CONGRATS_ENTITY_BUTTON_WIDTH_SELFIE,
                        y: CONGRATS_ENTITY_STATISTIC_SIZE_SELFIE,
                    }
                } else {
                    Vec2 {
                        x: CONGRATS_ENTITY_BUTTON_WIDTH_NORMAL,
                        y: CONGRATS_ENTITY_STATISTIC_SIZE_NORMAL,
                    }
                }
            }

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

    fn location(&self, context: &Self::Context<'_>, sizing: &LayoutSizing) -> Vec2 {
        if self.is_button() && context.1.is_tutorial() {
            let bottom_padding = sizing.bottom_pad / sizing.size_ratio;
            let total_height = IDEAL_HEIGHT + bottom_padding;

            let size = self.size(context, sizing);

            return Vec2 {
                x: (IDEAL_WIDTH - size.x) / 2.,
                y: (total_height - size.y) / 2.,
            };
        }

        let stat_size = if context.0.is_selfie_mode {
            CONGRATS_ENTITY_STATISTIC_SIZE_SELFIE
        } else {
            CONGRATS_ENTITY_STATISTIC_SIZE_NORMAL
        };

        let button_height = if context.0.is_selfie_mode {
            CONGRATS_ENTITY_BUTTON_HEIGHT + (MENU_BUTTON_SPACING * 2.0)
        } else {
            GRID_SIZE - ((stat_size + CONGRATS_ENTITY_SPACING) * 2.0)
        };

        let top_offset = if context.0.is_selfie_mode {
            IDEAL_HEIGHT
                - (button_height
                    + CONGRATS_BUTTON_GAP_SELFIE
                    + ((stat_size + CONGRATS_ENTITY_SPACING) * 2.0))
        } else {
            TOP_BAR_HEIGHT
                + TOP_BAR_OFFSET
                + extra_top_height(sizing, &context.0)
                + THEME_HEIGHT
                + GRID_WORD_LIST_SPACER
                + GRID_THEME_SPACER
        };
        pub const MENU_BUTTON_SPACING: f32 = 40.0 * 0.1;

        let button_count = Self::get_button_count(context);

        let button_width = if context.0.is_selfie_mode {
            CONGRATS_ENTITY_BUTTON_WIDTH_SELFIE
        } else {
            CONGRATS_ENTITY_BUTTON_WIDTH_NORMAL
        };

        match self {
            CongratsLayoutEntity::Time => Vec2 {
                x: (IDEAL_WIDTH - button_width) * 0.5,
                y: top_offset,
            },

            CongratsLayoutEntity::Statistic(statistic) => Vec2 {
                x: Spacing::SpaceAround.apply(
                    button_width,
                    stat_size,
                    CongratsStatistic::COUNT,
                    *statistic as usize,
                ) + ((IDEAL_WIDTH - button_width) * 0.5),
                y: top_offset + stat_size + CONGRATS_ENTITY_SPACING,
            },
            CongratsLayoutEntity::Button(button) => Vec2 {
                x: (IDEAL_WIDTH - button_width) * 0.5,
                y: top_offset
                    + ((stat_size + CONGRATS_ENTITY_SPACING) * 2.0)
                    + if context.0.is_selfie_mode {
                        CONGRATS_BUTTON_GAP_SELFIE
                    } else {
                        CONGRATS_BUTTON_GAP_NORMAL
                    }
                    + Spacing::Centre.apply(
                        button_height,
                        CONGRATS_ENTITY_BUTTON_HEIGHT + MENU_BUTTON_SPACING,
                        button_count,
                        *button as usize,
                    ),
            },
        }
    }

    fn iter_all(context: &Self::Context<'_>) -> impl Iterator<Item = Self> {
        let button_count = Self::get_button_count(context);
        let take = 3 + button_count;

        let stat_count = if context.1.is_tutorial() { 0 } else { 3 };

        CongratsStatistic::iter()
            .map(Self::Statistic)
            .take(stat_count)
            .chain(CongratsButton::iter().map(Self::Button))
            .take(take)
    }
}

impl LayoutStructureWithFont for CongratsLayoutEntity {
    type FontContext = ();
    fn font_size(&self, _context: &()) -> f32 {
        CONGRATS_BUTTON_FONT_SIZE
    }
}

pub struct CongratsTimer;

impl LayoutStructureWithFont for CongratsTimer {
    type FontContext = SelfieMode;
    fn font_size(&self, context: &SelfieMode) -> f32 {
        if context.is_selfie_mode {
            CONGRATS_TIMER_FONT_SIZE_SELFIE
        } else {
            CONGRATS_TIMER_FONT_SIZE_NORMAL
        }
    }
}

pub struct StatisticNumber;
pub struct StatisticLabel;

impl LayoutStructureWithFont for StatisticNumber {
    type FontContext = SelfieMode;
    fn font_size(&self, context: &SelfieMode) -> f32 {
        if context.is_selfie_mode {
            STATISTIC_NUMBER_FONT_SIZE_SELFIE
        } else {
            STATISTIC_NUMBER_FONT_SIZE_NORMAL
        }
    }
}

impl LayoutStructureWithFont for StatisticLabel {
    type FontContext = SelfieMode;
    fn font_size(&self, context: &SelfieMode) -> f32 {
        if context.is_selfie_mode {
            STATISTIC_LABEL_FONT_SIZE_SELFIE
        } else {
            STATISTIC_LABEL_FONT_SIZE_NORMAL
        }
    }
}
