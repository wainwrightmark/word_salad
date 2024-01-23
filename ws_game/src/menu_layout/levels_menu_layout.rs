use bevy::math::Vec2;
use strum::{Display, EnumCount};
use ws_core::{
    layout::entities::*, LayoutSizing, LayoutStructure, LayoutStructureWithFont, Spacing,
};
use ws_levels::level_group::LevelGroup;

use crate::prelude::*;

use super::{MENU_BUTTON_HEIGHT, MENU_BUTTON_SPACING, MENU_BUTTON_WIDTH};

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

    pub fn is_complete(
        &self,
        daily_challenge_completion: &DailyChallengeCompletion,
        sequence_completion: &SequenceCompletion,
        daily_challenges: &DailyChallenges,
    ) -> bool {
        let num_complete = match self {
            LevelsMenuLayoutEntity::WordSalad => {
                daily_challenge_completion.get_daily_challenges_complete()
            }
            LevelsMenuLayoutEntity::AdditionalLevel(group) => {
                sequence_completion.get_number_complete_group(group)
            }
        };
        let total = match self {
            LevelsMenuLayoutEntity::WordSalad => daily_challenges.total_levels(),
            LevelsMenuLayoutEntity::AdditionalLevel(group) => group.total_count(),
        };

        num_complete >= total
    }

    pub fn get_text(
        &self,
        daily_challenge_completion: &DailyChallengeCompletion,
        sequence_completion: &SequenceCompletion,
        daily_challenges: &DailyChallenges,
    ) -> (String, String) {
        let name = self.name();
        let num_complete = match self {
            LevelsMenuLayoutEntity::WordSalad => {
                daily_challenge_completion.get_daily_challenges_complete()
            }
            LevelsMenuLayoutEntity::AdditionalLevel(group) => {
                sequence_completion.get_number_complete_group(group)
            }
        };
        let total = match self {
            LevelsMenuLayoutEntity::WordSalad => daily_challenges.total_levels(),
            LevelsMenuLayoutEntity::AdditionalLevel(group) => group.total_count(),
        };

        let complete = num_complete.min(total);
        let fraction = fmtastic::VulgarFraction::new(complete, total).to_string();

        (name.to_string(), fraction)
    }

    fn name(&self) -> &'static str {
        match self {
            LevelsMenuLayoutEntity::WordSalad => "Word Salad",
            LevelsMenuLayoutEntity::AdditionalLevel(levels) => levels.name(),
        }
    }
}

impl LayoutStructure for LevelsMenuLayoutEntity {
    type Context<'a> = SelfieMode;

    fn size(&self, _context: &Self::Context<'_>, _sizing: &LayoutSizing) -> Vec2 {
        Vec2 {
            x: MENU_BUTTON_WIDTH,
            y: MENU_BUTTON_HEIGHT,
        }
    }

    fn location(&self, context: &Self::Context<'_>, sizing: &LayoutSizing) -> Vec2 {
        Vec2 {
            x: (IDEAL_WIDTH - MENU_BUTTON_WIDTH) / 2.,
            y: (TOP_BAR_HEIGHT_BASE + extra_top_bar_height(sizing, context))
                + Spacing::Centre.apply(
                    IDEAL_HEIGHT - (TOP_BAR_HEIGHT_BASE + extra_top_bar_height(sizing, context)),
                    MENU_BUTTON_HEIGHT + MENU_BUTTON_SPACING,
                    super::MENU_VIRTUAL_CHILDREN,
                    self.index(),
                ),
        }
    }

    fn iter_all(_context: &Self::Context<'_>) -> impl Iterator<Item = Self> {
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
    type FontContext = ();
    fn font_size(&self, _: &()) -> f32 {
        MENU_BUTTON_FONT_SIZE
    }
}
