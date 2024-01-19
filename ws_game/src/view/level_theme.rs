use crate::prelude::*;
use maveric::widgets::text2d_node::Text2DNode;
use maveric::with_bundle::CanWithBundle;
use ws_core::layout::entities::level_info_entity::LevelInfoLayoutEntity;
use ws_core::layout::entities::*;
use ws_core::prelude::*;

#[derive(Debug, Clone, PartialEq)]
pub struct LevelName {
    pub theme: Ustr,
    pub daily_challenge_number: Option<usize>,
    pub selfie_mode: SelfieMode,
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
            let theme_font_size = context.font_size(&LevelInfoLayoutEntity::Theme, &());

            let color = if node.selfie_mode.is_selfie_mode {
                palette::THEME_TEXT_COLOR_SELFIE
            } else {
                palette::THEME_TEXT_COLOR_NORMAL
            }
            .convert_color();

            commands.add_child(
                "theme",
                Text2DNode {
                    text: node.theme.to_string(),
                    font_size: theme_font_size,
                    color,
                    font: THEME_FONT_PATH,
                    alignment: TextAlignment::Left,
                    linebreak_behavior: bevy::text::BreakLineOn::NoWrap,
                    text_2d_bounds: Default::default(),
                    text_anchor: bevy::sprite::Anchor::CenterLeft,
                }
                .with_bundle(Transform::from_translation(
                    context
                        .get_rect(&LevelInfoLayoutEntity::Theme, &node.selfie_mode)
                        .centre_left()
                        .extend(crate::z_indices::THEME),
                )),
                &(),
            );

            if let Some(dcn) = node.daily_challenge_number {
                commands.add_child(
                    "daily_challenge_number",
                    Text2DNode {
                        text: format!("#{dcn}",),
                        font_size: theme_font_size,
                        color,
                        font: THEME_FONT_PATH,
                        alignment: TextAlignment::Right,
                        linebreak_behavior: bevy::text::BreakLineOn::NoWrap,
                        text_2d_bounds: Default::default(),
                        text_anchor: bevy::sprite::Anchor::CenterRight,
                    }
                    .with_bundle(Transform::from_translation(
                        context
                            .get_rect(&LevelInfoLayoutEntity::DailyChallengeNumber, &node.selfie_mode)
                            .centre_right()
                            .extend(crate::z_indices::THEME),
                    )),
                    &(),
                );
            }
        });
    }
}
