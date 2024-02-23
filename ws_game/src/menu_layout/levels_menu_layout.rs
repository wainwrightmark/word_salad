use strum::{Display, EnumCount};
use ws_core::layout::entities::*;
use ws_levels::level_group::LevelGroup;

use crate::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Display)]
pub enum LevelsMenuLayoutEntity {
    WordSalad,
    AdditionalLevel(LevelGroup),
}

impl MenuButtonsLayout for LevelsMenuLayoutEntity {
    type Context = ();
    fn index(&self) -> usize {
        match self {
            LevelsMenuLayoutEntity::WordSalad => 0,
            LevelsMenuLayoutEntity::AdditionalLevel(lg) => (*lg as usize) + 1,
        }
    }

    fn count(_context: &Self::Context) -> usize {
        1 + LevelGroup::COUNT
    }

    fn iter_all(_context: &Self::Context) -> impl Iterator<Item = Self> {
        [
            Self::WordSalad,
            Self::AdditionalLevel(LevelGroup::Geography),
            Self::AdditionalLevel(LevelGroup::NaturalWorld),
            Self::AdditionalLevel(LevelGroup::USSports),
        ]
        .into_iter()
    }
}

impl LevelsMenuLayoutEntity {
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
            LevelsMenuLayoutEntity::WordSalad => "Daily Word Salad",
            LevelsMenuLayoutEntity::AdditionalLevel(levels) => levels.name(),
        }
    }
}

impl LayoutStructureDoubleTextButton for LevelsMenuLayoutEntity {
    type TextContext<'a> = MenuContextWrapper<'a>;

    fn double_text(
        &self,
        _context: &Self::Context<'_>,
        text_context: &Self::TextContext<'_>,
    ) -> (String, String) {
        self.get_text(
            text_context.daily_challenge_completion.as_ref(),
            text_context.sequence_completion.as_ref(),
            text_context.daily_challenges.as_ref(),
        )
    }

    fn left_font(&self) -> &'static str {
        BUTTONS_FONT_PATH
    }

    fn right_font(&self) -> &'static str {
        BUTTONS_FONT_PATH
    }

    fn text_color(
        &self,
        _context: &Self::Context<'_>,
        _text_context: &Self::TextContext<'_>,
    ) -> BasicColor {
        palette::MENU_BUTTON_TEXT_REGULAR
    }

    fn fill_color(
        &self,
        background_type: BackgroundType,
        _context: &Self::Context<'_>,
        text_context: &Self::TextContext<'_>,
    ) -> BasicColor {
        if self.is_complete(
            &text_context.daily_challenge_completion,
            &text_context.sequence_completion,
            text_context.daily_challenges.as_ref(),
        ) {
            background_type.menu_button_complete_fill()
        } else {
            background_type.menu_button_incomplete_fill()
        }
    }

    fn is_disabled(
        &self,
        _context: &Self::Context<'_>,
        _text_context: &Self::TextContext<'_>,
    ) -> bool {
        false
    }
}
