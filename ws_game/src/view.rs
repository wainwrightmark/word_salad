use crate::prelude::*;

pub struct ViewRoot;

impl MavericRootChildren for ViewRoot {
    type Context = NC4<ChosenState, CurrentLevel, FoundWordsState, NC2<Size, AssetServer>>;

    fn set_children(
        context: &<Self::Context as NodeContext>::Wrapper<'_>,
        commands: &mut impl ChildCommands,
    ) {
        //commands.add_child("lines", GridLines, &());
        commands.add_child("cells", GridTiles, context);
        commands.add_child("ui", UI, context);
        commands.add_child("lines", WordLine, context);
    }
}

impl_maveric_root!(ViewRoot);
