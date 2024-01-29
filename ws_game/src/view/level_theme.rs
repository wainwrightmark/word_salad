use crate::prelude::*;
use maveric::widgets::text2d_node::Text2DNode;
use maveric::with_bundle::CanWithBundle;
use ws_core::layout::entities::level_info_entity::{LevelInfoLayoutEntity, ThemeLengths};
use ws_core::prelude::*;

#[derive(Debug, NodeContext)]
pub struct ThemeContext {
    pub found_words_state: FoundWordsState,
    pub window_size: MyWindowSize,
    pub video_resource: VideoResource,
}

impl<'a, 'w: 'a> From<&'a ViewContextWrapper<'w>> for ThemeContextWrapper<'w> {
    fn from(value: &'a ViewContextWrapper<'w>) -> Self {
        Self {
            found_words_state: Res::clone(&value.found_words_state),
            window_size: Res::clone(&value.window_size),

            video_resource: Res::clone(&value.video_resource),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LevelName {
    pub full_name: Ustr,
    // pub daily_challenge_number: Option<usize>,
}

impl MavericNode for LevelName {
    type Context = ThemeContext;

    fn set_components(commands: SetComponentCommands<Self, Self::Context>) {
        commands
            .ignore_context()
            .ignore_node()
            .insert(SpatialBundle::default())
            .finish()
    }

    fn set_children<R: MavericRoot>(commands: SetChildrenCommands<Self, Self::Context, R>) {
        commands.unordered_children_with_node_and_context(|node, context, commands| {
            let theme_font_size = context.window_size.font_size(
                &LevelInfoLayoutEntity::ThemeAndNumber,
                &ThemeLengths {
                    full_name_characters: node.full_name.len(),
                },
            );

            let color = if context.video_resource.is_selfie_mode {
                palette::THEME_TEXT_COLOR_SELFIE
            } else if context.found_words_state.is_level_complete() {
                palette::THEME_TEXT_COLOR_COMPLETE_NORMAL
            }else{
                palette::THEME_TEXT_COLOR_INCOMPLETE_NORMAL
            }
            .convert_color();

            commands.add_child(
                "theme",
                Text2DNode {
                    text: node.full_name.to_string(),
                    font_size: theme_font_size,
                    color,
                    font: THEME_FONT_PATH,
                    alignment: TextAlignment::Center,
                    linebreak_behavior: bevy::text::BreakLineOn::NoWrap,
                    text_2d_bounds: Default::default(),
                    text_anchor: bevy::sprite::Anchor::Center,
                }
                .with_bundle(Transform::from_translation(
                    context.window_size
                        .get_rect(&LevelInfoLayoutEntity::ThemeAndNumber, &context.video_resource.selfie_mode())
                        .centre()
                        .extend(crate::z_indices::THEME),
                )),
                &(),
            );

            // if let Some(dcn) = node.daily_challenge_number {
            //     commands.add_child(
            //         "daily_challenge_number",
            //         Text2DNode {
            //             text: format!("#{dcn}",),
            //             font_size: theme_font_size,
            //             color,
            //             font: THEME_FONT_PATH,
            //             alignment: TextAlignment::Right,
            //             linebreak_behavior: bevy::text::BreakLineOn::NoWrap,
            //             text_2d_bounds: Default::default(),
            //             text_anchor: bevy::sprite::Anchor::CenterRight,
            //         }
            //         .with_bundle(Transform::from_translation(
            //             context
            //                 .get_rect(
            //                     &LevelInfoLayoutEntity::DailyChallengeNumber,
            //                     &node.selfie_mode,
            //                 )
            //                 .centre_right()
            //                 .extend(crate::z_indices::THEME),
            //         )),
            //         &(),
            //     );
            // }
        });
    }
}
