use crate::prelude::*;
use maveric::widgets::text2d_node::Text2DNode;
use maveric::with_bundle::CanWithBundle;
use ws_core::layout::entities::*;
use ws_core::prelude::*;

#[derive(Debug, Clone, PartialEq)]
pub struct UITimer {
    pub time_text: String,
    pub selfie_mode: SelfieMode,
}

impl MavericNode for UITimer {
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
            let timer_font_size = context.font_size(&GameLayoutEntity::Timer, &());

            let color = if node.selfie_mode.is_selfie_mode {
                palette::THEME_TEXT_COLOR_SELFIE
            } else {
                palette::THEME_TEXT_COLOR_NORMAL
            }
            .convert_color();
            commands.add_child(
                "timer",
                Text2DNode {
                    text: node.time_text.clone(),
                    font_size: timer_font_size,
                    color,
                    font: TIMER_FONT_PATH,
                    alignment: TextAlignment::Right,
                    linebreak_behavior: bevy::text::BreakLineOn::NoWrap,
                    text_2d_bounds: Default::default(),
                    text_anchor: bevy::sprite::Anchor::CenterRight,
                }
                .with_bundle((
                    Transform::from_translation(
                        context
                            .get_rect(&GameLayoutEntity::Timer, &node.selfie_mode)
                            .centre_right()
                            .extend(crate::z_indices::TIMER),
                    ),
                    TimeCounterMarker,
                )),
                &(),
            );
        });
    }
}
