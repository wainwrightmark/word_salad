use bevy::math::Vec2;
use strum::{Display, EnumCount, EnumIs, EnumIter, IntoEnumIterator};
use ws_core::{
    layout::entities::*, LayoutSizing, LayoutStructure, LayoutStructureWithFont, Spacing,
};
use ws_levels::level_group::LevelGroup;

use crate::prelude::*;

use super::{MENU_BUTTON_HEIGHT, MENU_BUTTON_SPACING, MENU_BUTTON_WIDTH};

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Display, EnumIs, EnumCount, EnumIter,
)]
pub enum WordSaladMenuLayoutEntity {
    TodayPuzzle = 0,
    YesterdayPuzzle = 1,
    EreYesterdayPuzzle = 2,
    NextPuzzle = 4,
}

impl WordSaladMenuLayoutEntity {
    pub fn index(&self) -> usize {
        *self as usize
    }

    pub const COUNT: usize = 1 + LevelGroup::COUNT;

    pub fn get_text(
        &self,
        completion: &DailyChallengeCompletion,
        daily_challenges: &DailyChallenges,
    ) -> (String, String) {
        if let Some(result) = self.try_get_text(completion, daily_challenges) {
            return result;
        }

        let s1 = match self {
            //TODO better text
            WordSaladMenuLayoutEntity::TodayPuzzle => "Today's Puzzle",
            WordSaladMenuLayoutEntity::YesterdayPuzzle => "Yesterday's Puzzle",
            WordSaladMenuLayoutEntity::EreYesterdayPuzzle => "Ere-yesterday's Puzzle",
            WordSaladMenuLayoutEntity::NextPuzzle => "Next Incomplete",
        };

        (s1.to_string(), "\u{f096}".to_string())
    }

    pub fn is_complete(&self, completion: &DailyChallengeCompletion) -> bool {
        let today_index = DailyChallenges::get_today_index();

        let index = match self {
            WordSaladMenuLayoutEntity::TodayPuzzle => Some(today_index),
            WordSaladMenuLayoutEntity::YesterdayPuzzle => today_index.checked_sub(1),
            WordSaladMenuLayoutEntity::EreYesterdayPuzzle => today_index.checked_sub(2),
            WordSaladMenuLayoutEntity::NextPuzzle => today_index
                .checked_sub(3)
                .and_then(|x| completion.get_next_incomplete_daily_challenge(x)),
        };

        let Some(index) = index else {
            return false;
        };

        completion.is_daily_challenge_complete(index)
    }

    pub fn try_get_text(
        &self,
        completion: &DailyChallengeCompletion,
        daily_challenges: &DailyChallenges,
    ) -> Option<(String, String)> {
        let today_index = DailyChallenges::get_today_index();

        let index = match self {
            WordSaladMenuLayoutEntity::TodayPuzzle => today_index,
            WordSaladMenuLayoutEntity::YesterdayPuzzle => today_index.checked_sub(1)?,
            WordSaladMenuLayoutEntity::EreYesterdayPuzzle => today_index.checked_sub(2)?,
            WordSaladMenuLayoutEntity::NextPuzzle => {
                completion.get_next_incomplete_daily_challenge(today_index.checked_sub(3)?)?
            }
        };

        let level: &ws_core::prelude::DesignedLevel = daily_challenges.levels.get(index)?;

        let complete = completion.is_daily_challenge_complete(index);
        let name = level.full_name();
        let right = if complete { "\u{e802}" } else { "" }.to_string(); //check boxes

        Some((name.to_string(), right))
    }
}

impl LayoutStructure for WordSaladMenuLayoutEntity {
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
        Self::iter()
    }
}

impl LayoutStructureWithFont for WordSaladMenuLayoutEntity {
    type FontContext = ();
    fn font_size(&self, _: &()) -> f32 {
        MENU_BUTTON_FONT_SIZE_SMALL
    }
}
