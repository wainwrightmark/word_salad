use bevy::math::Vec2;
use strum::{Display, EnumCount, EnumIter, EnumMessage, IntoEnumIterator};
use ws_core::{
    layout::entities::{IDEAL_HEIGHT, IDEAL_WIDTH, TOP_BAR_ICON_SIZE},
    LayoutStructure, LayoutStructureWithFont, Spacing, LayoutStructureWithStaticText,
};

use super::{MENU_BUTTON_FONT_SIZE, MENU_BUTTON_HEIGHT, MENU_BUTTON_WIDTH};

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    EnumIter,
    EnumCount,
    Display,
)]
pub enum MainMenuLayoutEntity {
    Resume = 0,
    ChooseLevel = 1,
    ResetLevel = 2,
    Video = 3,
}

impl MainMenuLayoutEntity {
    pub fn index(&self) -> usize {
        *self as usize
    }
}

impl LayoutStructure for MainMenuLayoutEntity {
    type Context = ();
    type Iterator = <Self as IntoEnumIterator>::Iterator;

    fn pick(point: Vec2, context: &Self::Context) -> Option<Self> {
        for x in Self::iter() {
            if x.rect(context).contains(point) {
                return Some(x);
            }
        }
        return None;
    }

    fn size(&self, _context: &Self::Context) -> Vec2 {
        Vec2 {
            x: MENU_BUTTON_WIDTH,
            y: MENU_BUTTON_HEIGHT,
        }
    }

    fn location(&self, _context: &Self::Context) -> Vec2 {
        Vec2 {
            x: (IDEAL_WIDTH - MENU_BUTTON_WIDTH) / 2.,
            y: TOP_BAR_ICON_SIZE
                + Spacing::Centre.apply(
                    IDEAL_HEIGHT - TOP_BAR_ICON_SIZE,
                    MENU_BUTTON_HEIGHT* 1.2,
                    Self::COUNT,
                    self.index(),
                ),
        }
    }

    fn iter_all(_context: &Self::Context) -> Self::Iterator {
        Self::iter()
    }
}

impl LayoutStructureWithFont for MainMenuLayoutEntity {
    fn font_size() -> f32 {
        MENU_BUTTON_FONT_SIZE
    }
}


impl LayoutStructureWithStaticText for MainMenuLayoutEntity{
    fn text(&self, context: &Self::Context) -> &'static str {
        match self {
            MainMenuLayoutEntity::Resume => "Resume",
            MainMenuLayoutEntity::ChooseLevel => "Levels",
            MainMenuLayoutEntity::ResetLevel => "Reset",
            MainMenuLayoutEntity::Video => "Video",
        }
    }
}