use crate::prelude::*;

pub type ViewContext = NC6<ChosenState, CurrentLevel, FoundWordsState, Size, LevelTime, MenuState>;
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
            commands.add_child("word_line", WordLine{
                solution: context.0 .solution.clone(),
                should_hide: context.0.is_just_finished
            }, &context.3);
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

impl_maveric_root!(ViewRoot);
