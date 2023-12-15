use crate::{completion::TotalCompletion, prelude::*};

pub type ViewContext = (
    ChosenState,
    CurrentLevel,
    FoundWordsState,
    MyWindowSize,
    LevelTime,
    MenuState,
    HintState,
    TotalCompletion,
    VideoResource,
    DailyChallenges,
);
#[derive(MavericRoot)]
pub struct ViewRoot;

impl MavericRootChildren for ViewRoot {
    type Context = ViewContext;

    fn set_children(
        context: &<Self::Context as NodeContext>::Wrapper<'_>,
        commands: &mut impl ChildCommands,
    ) {
        commands.add_child("Top Bar", TopBar, context);

        if context.5.is_closed() {
            let level_complete = context.2.is_level_complete();

            commands.add_child("cells", GridTiles { level_complete }, context);
            commands.add_child("words", WordsNode, context);

            match context.1.level(&context.9) {
                itertools::Either::Left(level) => {
                    let close_to_solution =
                        context.0.is_close_to_a_solution(level, context.2.as_ref());
                    commands.add_child(
                        "word_line",
                        WordLine {
                            solution: context.0.solution.clone(),
                            should_hide: context.0.is_just_finished,
                            close_to_solution,
                        },
                        &context.3,
                    );

                    if context.2.is_level_complete() {
                        commands.add_child("congrats", CongratsView, context);
                    }

                    if let Some(text) = TutorialText::try_create(&context.1, &context.2) {
                        commands.add_child("tutorial", TutorialNode { text }, &context.3);
                    } else {
                        let theme = level.name.clone();
                        commands.add_child("ui_theme", UITheme { theme }, &context.3);

                        let time_text = match context.4.as_ref() {
                            LevelTime::Started(..) => "00:00".to_string(),
                            LevelTime::Finished { total_seconds } => format_seconds(*total_seconds),
                        };
                        commands.add_child("ui_timer", UITimer { time_text }, &context.3);
                    }
                }
                itertools::Either::Right(non_level) => {
                    commands.add_child("non_level", NonLevelView { non_level }, &context.3);
                }
            }
        } else {
            commands.add_child("menu", Menu, context);
        }
    }
}
