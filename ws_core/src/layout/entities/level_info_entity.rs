use crate::prelude::*;
use glam::Vec2;
use strum::{Display, EnumCount, EnumIter, IntoEnumIterator};

use super::{consts::*, GameLayoutEntity, SelfieMode};

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, EnumIter, Display, EnumCount,
)]
pub enum LevelInfoLayoutEntity {
    Theme,
    DailyChallengeNumber,
    TimerRight,
    TimerLeft,
    ThemeInfo,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct IsDailyChallenge(bool);

impl LayoutStructure for LevelInfoLayoutEntity {
    type Context<'a> = SelfieMode;

    fn iter_all(_context: &Self::Context<'_>) -> impl Iterator<Item = Self> {
        Self::iter()
    }
    ///The size on a 320x568 canvas
    fn size(&self, _context: &Self::Context<'_>, _sizing: &LayoutSizing) -> Vec2 {
        match self {
            LevelInfoLayoutEntity::Theme => Vec2 {
                x: THEME_WIDTH,
                y: THEME_HEIGHT,
            },

            LevelInfoLayoutEntity::DailyChallengeNumber => Vec2 {
                x: GRID_SIZE - THEME_WIDTH,
                y: THEME_HEIGHT,
            },
            LevelInfoLayoutEntity::ThemeInfo => Vec2 {
                x: THEME_INFO_WIDTH,
                y: THEME_INFO_HEIGHT,
            },
            LevelInfoLayoutEntity::TimerRight => Vec2 {
                x: TIMER_WIDTH,
                y: TIMER_HEIGHT,
            },
            LevelInfoLayoutEntity::TimerLeft => Vec2 {
                x: TIMER_WIDTH,
                y: TIMER_HEIGHT,
            },
        }
    }
    fn location(&self, context: &Self::Context<'_>, sizing: &LayoutSizing) -> Vec2 {
        let base_location =
            GameLayoutEntity::location(&GameLayoutEntity::LevelInfo, &context, sizing);

        match self {
            LevelInfoLayoutEntity::Theme => Vec2 {
                x: base_location.x,
                y: base_location.y,
            },

            LevelInfoLayoutEntity::DailyChallengeNumber => Vec2 {
                x: base_location.x + GRID_SIZE  - DAILY_CHALLENGE_NUMBER_WIDTH,
                y: base_location.y,
            },
            LevelInfoLayoutEntity::ThemeInfo => Vec2 {
                x: base_location.x,
                y: base_location.y + THEME_HEIGHT,
            },

            LevelInfoLayoutEntity::TimerRight => Vec2 {
                x: base_location.x + GRID_SIZE - TIMER_WIDTH,
                y: base_location.y + THEME_HEIGHT,
            },
            LevelInfoLayoutEntity::TimerLeft => Vec2 {
                x: base_location.x,
                y: base_location.y + THEME_HEIGHT,
            },
        }
    }
}

impl LayoutStructureWithFont for LevelInfoLayoutEntity {
    type FontContext = ();

    fn font_size(&self, _: &()) -> f32 {
        match self {
            LevelInfoLayoutEntity::Theme => THEME_FONT_SIZE,
            LevelInfoLayoutEntity::DailyChallengeNumber => THEME_FONT_SIZE,
            LevelInfoLayoutEntity::TimerLeft => TIMER_FONT_SIZE,
            LevelInfoLayoutEntity::TimerRight => TIMER_FONT_SIZE,
            LevelInfoLayoutEntity::ThemeInfo => THEME_INFO_FONT_SIZE,
        }
    }
}
