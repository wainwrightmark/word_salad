use strum::{Display, EnumIs};
use ws_core::{font_icons, layout::entities::*};

use crate::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Display, EnumIs)]
pub enum WordSaladMenuLayoutEntity {
    DaysAgo(usize),
    NextPuzzle,
}

pub const DAYS_AGO: usize = 5;

impl MenuButtonsLayout for WordSaladMenuLayoutEntity {
    type Context = ();

    fn index(&self) -> usize {
        match self {
            WordSaladMenuLayoutEntity::DaysAgo(x) => *x,
            WordSaladMenuLayoutEntity::NextPuzzle => DAYS_AGO + 1,
        }
    }

    fn count(_context: &Self::Context) -> usize {
        DAYS_AGO + 2
    }

    fn iter_all(_context: &Self::Context) -> impl Iterator<Item = Self> {
        (0..DAYS_AGO)
            .map(WordSaladMenuLayoutEntity::DaysAgo)
            .chain(std::iter::once(WordSaladMenuLayoutEntity::NextPuzzle))
    }

    const FONT_SIZE_SMALL: bool = true;
}

impl WordSaladMenuLayoutEntity {
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
            WordSaladMenuLayoutEntity::DaysAgo(0) => "Today's Puzzle".to_string(),
            WordSaladMenuLayoutEntity::DaysAgo(1) => "Yesterday's Puzzle".to_string(),
            WordSaladMenuLayoutEntity::DaysAgo(x) => format!("{x} Days Ago Puzzle"),
            WordSaladMenuLayoutEntity::NextPuzzle => "Reset Completion".to_string(),
        };

        (s1.to_string(), "\u{f096}".to_string())
    }

    pub fn is_complete(
        &self,
        completion: &DailyChallengeCompletion,
        daily_challenges: &DailyChallenges,
    ) -> bool {
        let today_index = DailyChallenges::get_today_index();

        let index = match self {
            WordSaladMenuLayoutEntity::DaysAgo(x) => today_index.checked_sub(*x),
            WordSaladMenuLayoutEntity::NextPuzzle => {
                today_index.checked_sub(DAYS_AGO).and_then(|x| {
                    completion
                        .get_next_incomplete_daily_challenge(x, daily_challenges)
                        .level_index()
                })
            }
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
            WordSaladMenuLayoutEntity::DaysAgo(x) => today_index.checked_sub(*x)?,
            WordSaladMenuLayoutEntity::NextPuzzle => completion
                .get_next_incomplete_daily_challenge(
                    today_index.checked_sub(DAYS_AGO)?,
                    daily_challenges,
                )
                .level_index()?,
        };

        let level: &ws_core::prelude::DesignedLevel = daily_challenges.levels().get(index)?;

        let complete = completion.is_daily_challenge_complete(index);
        let name = level.full_name();
        let right = if complete { font_icons::TICK } else { "" }.to_string(); //check boxes

        Some((name.to_string(), right))
    }
}

impl LayoutStructureDoubleTextButton for WordSaladMenuLayoutEntity {
    type TextContext<'a> = MenuContextWrapper<'a>;

    fn double_text(
        &self,
        _context: &Self::Context<'_>,
        text_context: &Self::TextContext<'_>,
    ) -> (String, String) {
        self.get_text(
            text_context.daily_challenge_completion.as_ref(),
            text_context.daily_challenges.as_ref(),
        )
    }

    fn left_font(&self) -> &'static str {
        BUTTONS_FONT_PATH
    }

    fn right_font(&self) -> &'static str {
        ICON_FONT_PATH
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
        background_type: ws_core::prelude::BackgroundType,
        _context: &Self::Context<'_>,
        text_context: &Self::TextContext<'_>,
    ) -> BasicColor {
        if self.is_complete(
            &text_context.daily_challenge_completion,
            &text_context.daily_challenges,
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
