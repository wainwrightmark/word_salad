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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct IsLevelComplete(pub bool);

impl LayoutStructure for LevelInfoLayoutEntity {
    type Context<'a> = ((SelfieMode, Insets), IsLevelComplete);

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
        }
    }
    fn location(&self, context: &Self::Context<'_>, sizing: &LayoutSizing) -> Vec2 {
        let base_location =
            GameLayoutEntity::location(&GameLayoutEntity::LevelInfo, &context.0, sizing);

        let extra_y = if context.1 .0 {
            THEME_HEIGHT + THEME_INFO_HEIGHT
        } else {
            0.0
        };

        match self {
            LevelInfoLayoutEntity::ThemeAndNumber => Vec2 {
                x: base_location.x,
                y: base_location.y + extra_y,
            },

            LevelInfoLayoutEntity::ThemeInfo => Vec2 {
                x: base_location.x,
                y: base_location.y + THEME_HEIGHT + extra_y,
            },
        }
    }
}

impl LayoutStructureWithOrigin for LevelInfoLayoutEntity {
    fn origin(&self, context: &Self::Context<'_>, _sizing: &LayoutSizing) -> Origin {
        if context.1 .0 {
            Origin::Center
        } else {
            Origin::CenterLeft
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
            LevelInfoLayoutEntity::ThemeInfo => THEME_INFO_FONT_SIZE,
        }
    }
}
