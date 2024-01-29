pub mod congrats;
pub mod fireworks;
pub mod game_grid;
pub mod hints;
pub mod level_extra_info;
pub mod level_theme;
pub mod menu;
pub mod non_level;
pub mod popup;
pub mod timer;
pub mod top_bar;
pub mod tutorial;
pub mod wordline;
pub mod words;

pub use congrats::*;
pub use game_grid::*;
pub use hints::*;
pub use level_extra_info::*;
pub use level_theme::*;
pub use menu::*;
pub use non_level::*;
pub use popup::*;
pub use top_bar::*;
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
        commands.add_child("Top Bar", TopBar, &context.into());

        let selfie_mode = context.video_resource.selfie_mode();

        if !context.menu_state.is_closed() {
            commands.add_child("menu", Menu, &context.into());
        }

        let level_complete = context.found_words_state.is_level_complete();

        if context.menu_state.is_closed() {
            commands.add_child("cells", GridTiles { level_complete }, &context.into());

        }

        match context.current_level.level(&context.daily_challenges) {
            itertools::Either::Left(level) => {
                if context.menu_state.is_closed() {
                    let close_to_solution = context
                        .chosen_state
                        .is_close_to_a_solution(level, context.found_words_state.as_ref());

                    if !context.level_time.is_paused() {
                        commands.add_child(
                            "word_line",
                            WordLine {
                                solution: context.chosen_state.solution.clone(),
                                should_hide: context.chosen_state.is_just_finished,
                                close_to_solution,
                                selfie_mode,
                            },
                            &context.window_size,
                        );
                    }

                    if context.found_words_state.is_level_complete() {
                        commands.add_child("congrats", CongratsView, &context.into());
                    }
                    else{
                        commands.add_child("words", WordsNode, &context.into());
                    }
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
                        LevelName {
                            full_name,
                            selfie_mode,
                        },
                        &context.window_size,
                    );

                    if let Some(info) = &level.extra_info {
                        commands.add_child(
                            "ui_theme_info",
                            LevelExtraInfo {
                                info: *info,
                                selfie_mode,
                                full_name_characters: full_name.len()

                            },
                            &context.window_size,
                        );
                    }
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
