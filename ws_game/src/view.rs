use crate::prelude::*;

pub struct ViewRoot;

impl MavericRootChildren for ViewRoot {
    type Context = NC5<ChosenState, CurrentLevel, FoundWordsState, Size, LevelTime>;

    fn set_children(
        context: &<Self::Context as NodeContext>::Wrapper<'_>,
        commands: &mut impl ChildCommands,
    ) {
        //commands.add_child("lines", GridLines, &());

        commands.add_child("ui", UI, context);

        if context.2.is_level_complete(&context.1) {
            commands.add_child("congrats", CongratsView, context);
        } else {
            commands.add_child("cells", GridTiles, context);
            if !context.0 .0.is_empty() {
                commands.add_child("word_line", WordLine, context);
            }
        }
    }
}

impl_maveric_root!(ViewRoot);
