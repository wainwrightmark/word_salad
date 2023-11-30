use bevy::math::Vec2;
use strum::{Display, EnumCount};
use ws_core::{
    layout::entities::{IDEAL_HEIGHT, IDEAL_WIDTH, TOP_BAR_ICON_SIZE},
    LayoutStructure, LayoutStructureWithFont, LayoutStructureWithStaticText, Spacing,
};
use ws_levels::level_group::LevelGroup;

use super::{MENU_BUTTON_FONT_SIZE, MENU_BUTTON_HEIGHT, MENU_BUTTON_WIDTH};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Display)]
pub enum LevelsMenuLayoutEntity {
    WordSalad,
    AdditionalLevel(LevelGroup),

}

impl LevelsMenuLayoutEntity {
    pub fn index(&self) -> usize {
        match self {
            LevelsMenuLayoutEntity::WordSalad => 0,
            LevelsMenuLayoutEntity::AdditionalLevel(lg) => (*lg as usize) + 1,
        }
    }

    pub const COUNT: usize = 1 + LevelGroup::COUNT;
}

impl LayoutStructure for LevelsMenuLayoutEntity {
    type Context = ();
    type Iterator = std::array::IntoIter<Self, { Self::COUNT }>;

    fn pick(point: Vec2, context: &Self::Context) -> Option<Self> {
        for x in Self::iter_all(context) {
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
                    MENU_BUTTON_HEIGHT * 1.2,
                    Self::COUNT,
                    self.index(),
                ),
        }
    }

    fn iter_all(_context: &Self::Context) -> Self::Iterator {
        [
            Self::WordSalad,
            Self::AdditionalLevel(LevelGroup::GlobalLocation),
            Self::AdditionalLevel(LevelGroup::HistoryMythology),
            Self::AdditionalLevel(LevelGroup::Science),
        ]
        .into_iter()
    }
}

impl LayoutStructureWithFont for LevelsMenuLayoutEntity {
    fn font_size() -> f32 {
        MENU_BUTTON_FONT_SIZE
    }
}

impl LayoutStructureWithStaticText for LevelsMenuLayoutEntity {
    fn text(&self, _context: &Self::Context) -> &'static str {
        match self {
            LevelsMenuLayoutEntity::WordSalad => "Word Salad",
            LevelsMenuLayoutEntity::AdditionalLevel(levels) => levels.name(),

        }
    }
}
