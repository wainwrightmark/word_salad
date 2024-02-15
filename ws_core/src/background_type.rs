use strum::EnumIs;

use crate::{palette, BasicColor};

#[derive(Debug, Clone, Copy, PartialEq, EnumIs)]
pub enum BackgroundType {
    Congrats,
    NonLevel,
    Selfie,
    Normal,
}

impl BackgroundType {
    pub fn color(&self) -> BasicColor {
        match self {
            BackgroundType::Congrats => palette::CLEAR_COLOR_CONGRATS,
            BackgroundType::NonLevel => palette::CLEAR_COLOR_NON_LEVEL,
            BackgroundType::Selfie => palette::CLEAR_COLOR_SELFIE,
            BackgroundType::Normal => palette::CLEAR_COLOR_NORMAL,
        }
    }

    pub fn is_transition_instant(&self) -> bool {
        match self {
            BackgroundType::Congrats => false,
            BackgroundType::NonLevel => true,
            BackgroundType::Selfie => true,
            BackgroundType::Normal => true,
        }
    }

    pub fn menu_button_complete_fill(&self) -> BasicColor {
        match self {
            BackgroundType::Congrats | BackgroundType::NonLevel => palette::NONE,
            BackgroundType::Selfie | BackgroundType::Normal => palette::MENU_BUTTON_COMPLETE_FILL,
        }
    }

    pub fn menu_button_incomplete_fill(&self) -> BasicColor {
        match self {
            BackgroundType::Congrats | BackgroundType::NonLevel => palette::NONE,
            BackgroundType::Selfie | BackgroundType::Normal => palette::MENU_BUTTON_FILL,
        }
    }
}
