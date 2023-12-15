use crate::prelude::*;
use maveric::widgets::text2d_node::Text2DNode;
use maveric::with_bundle::CanWithBundle;
use ws_core::layout::entities::*;
use ws_core::prelude::*;

#[derive(Debug, Clone, PartialEq)]
pub struct LevelName {
    pub theme: String,
}

impl MavericNode for LevelName {
    type Context = MyWindowSize;

    fn set_components(commands: SetComponentCommands<Self, Self::Context>) {
        commands
            .ignore_context()
            .ignore_node()
            .insert(SpatialBundle::default())
            .finish()
    }

    fn set_children<R: MavericRoot>(commands: SetChildrenCommands<Self, Self::Context, R>) {
        commands.unordered_children_with_node_and_context(|node, context, commands| {
            let theme_font_size = context.font_size(&GameLayoutEntity::Theme);

            commands.add_child(
                "theme",
                Text2DNode {
                    text: node.theme.clone(),
                    font_size: theme_font_size,
                    color: palette::BUTTON_TEXT_COLOR.convert_color(),
                    font: TITLE_FONT_PATH,
                    alignment: TextAlignment::Left,
                    linebreak_behavior: bevy::text::BreakLineOn::NoWrap,
                }
                .with_bundle(Transform::from_translation(
                    context
                        .get_rect(&GameLayoutEntity::Theme, &())
                        .centre_left()
                        .extend(crate::z_indices::THEME),
                )),
                &(),
            );
        });
    }
}
