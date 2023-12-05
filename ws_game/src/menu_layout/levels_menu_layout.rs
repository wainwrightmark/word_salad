use bevy::math::Vec2;
use strum::{Display, EnumCount};
use ws_core::{
    layout::entities::{IDEAL_HEIGHT, IDEAL_WIDTH, TOP_BAR_ICON_SIZE},
    LayoutStructure, LayoutStructureWithFont, Spacing,
};
use ws_levels::{level_group::LevelGroup, level_sequence::LevelSequence};

use crate::completion::TotalCompletion;

use super::{
    MENU_BUTTON_DOUBLE_HEIGHT, MENU_BUTTON_FONT_SIZE, MENU_BUTTON_SPACING, MENU_BUTTON_WIDTH,
};

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

    pub fn get_text(&self, completion: &TotalCompletion)-> String{
        let name =self.name();
        let complete = match self {
            LevelsMenuLayoutEntity::WordSalad => completion.get_number_complete(&ws_levels::level_sequence::LevelSequence::DailyChallenge),
            LevelsMenuLayoutEntity::AdditionalLevel(group) => completion.get_number_complete_group(group),
        };
        let total = match self {
            LevelsMenuLayoutEntity::WordSalad => LevelSequence::DailyChallenge.level_count(),
            LevelsMenuLayoutEntity::AdditionalLevel(group) => group.total_count(),
        };

        let complete = complete.min(total);

        format!("{name}\n{complete:3}/{total:3}")
    }

    fn name(&self) -> &'static str {
        match self {
            LevelsMenuLayoutEntity::WordSalad => "Word Salad",
            LevelsMenuLayoutEntity::AdditionalLevel(levels) => levels.name(),

        }
    }
}

impl LayoutStructure for LevelsMenuLayoutEntity {
    type Context = ();
    type Iterator = std::array::IntoIter<Self, { Self::COUNT }>;

    fn pick(point: Vec2, context: &Self::Context) -> Option<Self> {
        Self::iter_all(context).find(|&x| x.rect(context).contains(point))
    }

    fn size(&self, _context: &Self::Context) -> Vec2 {
        Vec2 {
            x: MENU_BUTTON_WIDTH,
            y: MENU_BUTTON_DOUBLE_HEIGHT,
        }
    }

    fn location(&self, _context: &Self::Context) -> Vec2 {
        Vec2 {
            x: (IDEAL_WIDTH - MENU_BUTTON_WIDTH) / 2.,
            y: TOP_BAR_ICON_SIZE
                + Spacing::Centre.apply(
                    IDEAL_HEIGHT - TOP_BAR_ICON_SIZE,
                    MENU_BUTTON_DOUBLE_HEIGHT + MENU_BUTTON_SPACING,
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
    fn font_size(&self) -> f32 {
        MENU_BUTTON_FONT_SIZE
    }
}


