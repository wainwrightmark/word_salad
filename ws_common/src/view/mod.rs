pub mod congrats;
pub mod fireworks;
pub mod game_grid;
pub mod hints;
pub mod logo;
pub mod menu;
pub mod non_level;
pub mod popup;
pub mod recording_button;
pub mod theme_view;
pub mod tutorial;
pub mod wordline;
pub mod words;

pub use congrats::*;
pub use game_grid::*;
pub use hints::*;
pub use logo::*;
pub use menu::*;
pub use non_level::*;
pub use popup::*;
pub use recording_button::*;
pub use theme_view::*;
pub use tutorial::*;
pub use wordline::*;
pub use words::*;

use crate::{completion::*, prelude::*};
use maveric::prelude::*;

#[derive(Debug, NodeContext)]
pub struct ViewContext {
    pub chosen_state: ChosenState,
    pub current_level: CurrentLevel,
    pub found_words_state: FoundWordsState,
    pub window_size: MyWindowSize,
    pub level_time: LevelTime,
    pub menu_state: MenuState,
    pub hint_state: HintState,
    pub daily_challenge_completion: DailyChallengeCompletion,
    pub sequence_completion: SequenceCompletion,
    pub video_resource: VideoResource,
    pub daily_challenges: DailyChallenges,
    pub streak: Streak,
}

#[derive(MavericRoot)]
pub struct ViewRoot;

impl MavericRootChildren for ViewRoot {
    type Context = ViewContext;

    fn set_children(
        context: &<Self::Context as NodeContext>::Wrapper<'_>,
        commands: &mut impl ChildCommands,
    ) {
        let selfie_mode = context.video_resource.selfie_mode();
        let is_level_complete = context.found_words_state.is_level_complete();
        let background_type = background_type_from_resources(
            &context.video_resource,
            &context.current_level,
            &context.found_words_state,
        );

        let pause_type = if !context.menu_state.is_closed() {
            PauseType::Blank
        } else if context.level_time.is_paused() {
            PauseType::BlankWithPlay
        } else {
            PauseType::NotPaused
        };

        commands.add_child(
            "cells",
            GridTiles {
                is_level_complete,
                pause_type,
            },
            &context.into(),
        );

        match context.current_level.level(&context.daily_challenges) {
            itertools::Either::Left(level) => {
                if context.found_words_state.is_level_complete() {
                    commands.add_child("congrats", CongratsView, &context.into());
                } else {
                    commands.add_child("words", WordsNode, &context.into());
                }

                if !context.level_time.is_paused() {
                    let close_to_solution = context
                        .chosen_state
                        .is_close_to_a_solution(level, context.found_words_state.as_ref());

                    commands.add_child(
                        "word_line",
                        WordLine {
                            solution: context.chosen_state.solution.clone(),
                            should_hide: context.chosen_state.is_just_finished,
                            close_to_solution,
                            selfie_mode,
                            special_colors: level.special_colors.clone(),
                        },
                        &context.window_size,
                    );
                }

                if let Some(text) =
                    TutorialText::try_create(&context.current_level, &context.found_words_state)
                {
                    if context.menu_state.is_closed() {
                        commands.add_child("tutorial", TutorialNode { text }, &context.window_size);
                    }
                } else {
                    let full_name = level.full_name();
                    commands.add_child(
                        "ui_theme",
                        ThemeView {
                            full_name,
                            info: level.extra_info,
                            background_type,
                            selfie_mode,
                            is_level_complete,
                        },
                        &context.window_size,
                    );
                }
            }
            itertools::Either::Right(non_level) => {
                if context.menu_state.is_closed() {
                    commands.add_child(
                        "non_level",
                        NonLevelView {
                            non_level,
                            selfie_mode,
                        },
                        &context.window_size,
                    );
                }
            }
        }
    }
}
