use crate::prelude::*;
use maveric::widgets::text2d_node::Text2DNode;
use maveric::with_bundle::CanWithBundle;
use ws_core::layout::entities::*;
use ws_core::prelude::*;

#[derive(Debug, Clone, PartialEq)]
pub struct LevelExtraInfo {
    pub info: String,
}

impl MavericNode for LevelExtraInfo {
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
            let theme_font_size = context.font_size(&GameLayoutEntity::ThemeInfo);

            commands.add_child(
                "info",
                Text2DNode {
                    text: node.info.clone(),
                    font_size: theme_font_size,
                    color: palette::BUTTON_TEXT_COLOR.convert_color(),
                    font: TITLE_FONT_PATH,
                    alignment: TextAlignment::Right,
                    linebreak_behavior: bevy::text::BreakLineOn::NoWrap,
                    text_2d_bounds: Default::default(),
                    text_anchor: bevy::sprite::Anchor::CenterRight,
                }
                .with_bundle(Transform::from_translation(
                    context
                        .get_rect(&GameLayoutEntity::ThemeInfo, &())
                        .centre_right()
                        .extend(crate::z_indices::THEME),
                )),
                &(),
            );
        });
    }
}
