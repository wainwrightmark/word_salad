use crate::prelude::*;

pub type ViewContext = (
    ChosenState,
    CurrentLevel,
    FoundWordsState,
    MyWindowSize,
    LevelTime,
    MenuState,
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
            commands.add_child("ui", UI, context);
            let level_complete = context.2.is_level_complete();

            commands.add_child("cells", GridTiles { level_complete }, context);

            let close_to_solution = context
                .0
                .is_close_to_a_solution(context.1.level(), context.2.as_ref());
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
            } else {
                commands.add_child("hints", HintGlows, context);
            }
        } else {
            commands.add_child("menu", Menu, context);
        }
    }
}
