use crate::prelude::*;
use bevy::sprite::Anchor;
use maveric::widgets::text2d_node::Text2DNode;
use maveric::with_bundle::CanWithBundle;
use ws_core::layout::entities::level_info_entity::LevelInfoLayoutEntity;
use ws_core::layout::entities::*;
use ws_core::prelude::*;

#[derive(Debug, Clone, PartialEq)]
pub struct UITimer {
    pub time_text: String,
    pub selfie_mode: SelfieMode,
    pub is_daily_challenge: bool,
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
            let (entity, alignment, text_anchor) = if node.is_daily_challenge {
                (
                    LevelInfoLayoutEntity::TimerRight,
                    TextAlignment::Right,
                    Anchor::CenterRight,
                )
            } else {
                (
                    LevelInfoLayoutEntity::TimerLeft,
                    TextAlignment::Left,
                    Anchor::CenterLeft,
                )
            };

            let timer_font_size = context.font_size(&entity, &());

            let color = if node.selfie_mode.is_selfie_mode {
                palette::THEME_TEXT_COLOR_SELFIE
            } else {
                palette::THEME_TEXT_COLOR_NORMAL
            }
            .convert_color();

            let rect = context.get_rect(&entity, &node.selfie_mode);

            let position = if node.is_daily_challenge {
                rect.centre_right()
            } else {
                rect.centre_left()
            };

            commands.add_child(
                "timer",
                Text2DNode {
                    text: node.time_text.clone(),
                    font_size: timer_font_size,
                    color,
                    font: TIMER_FONT_PATH,
                    alignment,
                    linebreak_behavior: bevy::text::BreakLineOn::NoWrap,
                    text_2d_bounds: Default::default(),
                    text_anchor,
                }
                .with_bundle((
                    Transform::from_translation(position.extend(crate::z_indices::TIMER)),
                    TimeCounterMarker,
                )),
                &(),
            );
        });
    }
}
