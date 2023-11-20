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
        let level_complete = context.2.is_level_complete();

        commands.add_child("cells", GridTiles{level_complete}, context);
        if context.2.is_level_complete() {
            commands.add_child("congrats", CongratsView, context);
        } else {

            commands.add_child("hints", HintGlows, context);
            if !context.0 .0.is_empty() {
                commands.add_child("word_line", WordLine(context.0.0.clone()), &context.3);
            }
        }
    }
}

impl_maveric_root!(ViewRoot);
