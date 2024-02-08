use crate::prelude::*;
use glam::Vec2;
use strum::{Display, EnumCount, EnumIter, IntoEnumIterator};

use super::{consts::*, GameLayoutEntity, SelfieMode};

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, EnumIter, Display, EnumCount,
)]
pub enum LevelInfoLayoutEntity {
    ThemeAndNumber,
    ThemeInfo,
    Timer,
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
            LevelInfoLayoutEntity::ThemeAndNumber => Vec2 {
                x: GRID_SIZE,
                y: THEME_HEIGHT,
            },
            LevelInfoLayoutEntity::ThemeInfo => Vec2 {
                x: GRID_SIZE,
                y: THEME_INFO_HEIGHT,
            },
            LevelInfoLayoutEntity::Timer => Vec2 {
                x: GRID_SIZE,
                y: THEME_INFO_HEIGHT,
            },
        }
    }
    fn location(&self, context: &Self::Context<'_>, sizing: &LayoutSizing) -> Vec2 {
        let base_location =
            GameLayoutEntity::location(&GameLayoutEntity::LevelInfo, &context, sizing);

        match self {
            LevelInfoLayoutEntity::ThemeAndNumber => Vec2 {
                x: base_location.x,
                y: base_location.y,
            },

            LevelInfoLayoutEntity::ThemeInfo => Vec2 {
                x: base_location.x,
                y: base_location.y + THEME_HEIGHT,
            },

            LevelInfoLayoutEntity::Timer => Vec2 {
                x: base_location.x,
                y: base_location.y + THEME_HEIGHT + THEME_INFO_HEIGHT,
            },
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct ThemeLengths {
    pub full_name_characters: usize,
}

impl LayoutStructureWithFont for LevelInfoLayoutEntity {
    type FontContext = ThemeLengths;

    fn font_size(&self, theme_length: &ThemeLengths) -> f32 {
        match self {
            LevelInfoLayoutEntity::ThemeAndNumber => {
                if theme_length.full_name_characters <= 22 {
                    THEME_FONT_SIZE
                } else {
                    THEME_FONT_SIZE_SMALL
                }
            }
            LevelInfoLayoutEntity::ThemeInfo | LevelInfoLayoutEntity::Timer => THEME_INFO_FONT_SIZE,
        }
    }
}
