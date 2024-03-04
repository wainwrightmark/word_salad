use crate::prelude::*;
use maveric::widgets::text2d_node::Text2DNode;
use maveric::with_bundle::CanWithBundle;
use ws_core::layout::entities::level_info_entity::{
    IsLevelComplete, LevelInfoLayoutEntity, ThemeLengths,
};
use ws_core::layout::entities::{SelfieMode, TimerLayoutEntity};
use ws_core::prelude::*;

#[derive(Debug, Clone, PartialEq)]
pub struct TimerView {
    pub background_type: BackgroundType,
    pub selfie_mode: SelfieMode,
    pub insets: Insets,
}

impl MavericNode for TimerView {
    type Context = MyWindowSize;

    fn set_components(mut commands: SetComponentCommands<Self, Self::Context>) {
        commands.insert_static_bundle(SpatialBundle::default());
    }

    fn set_children<R: MavericRoot>(commands: SetChildrenCommands<Self, Self::Context, R>) {
        commands.unordered_children_with_node_and_context(|node, context, commands| {
            let font_size = context.font_size(&TimerLayoutEntity, &());

            let timer_color = match node.background_type {
                BackgroundType::Congrats | BackgroundType::NonLevel => palette::TIMER_COLOR_NORMAL,
                BackgroundType::Selfie => palette::TIMER_COLOR_SELFIE,
                BackgroundType::Normal => palette::TIMER_COLOR_NORMAL,
            }
            .convert_color();

            commands.add_child(
                "timer",
                Text2DNode {
                    text: "00:00",
                    font_size,
                    color: timer_color,
                    font: TIMER_FONT_PATH,
                    justify_text: JustifyText::Center,
                    linebreak_behavior: bevy::text::BreakLineOn::NoWrap,
                    text_2d_bounds: Default::default(),
                    text_anchor: bevy::sprite::Anchor::Center,
                }
                .with_bundle((
                    Transform::from_translation(
                        context
                            .get_rect(&TimerLayoutEntity, &(node.selfie_mode, node.insets))
                            .top_centre()
                            .extend(crate::z_indices::THEME),
                    ),
                    TimeCounterMarker,
                )), //TODO slow fade out
                &(),
            );
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ThemeView {
    pub full_name: Ustr,
    pub info: Option<Ustr>,
    pub background_type: BackgroundType,
    pub is_level_complete: bool,
    pub selfie_mode: SelfieMode,
    pub insets: Insets,
}

impl MavericNode for ThemeView {
    type Context = MyWindowSize;

    fn set_components(mut commands: SetComponentCommands<Self, Self::Context>) {
        commands.insert_static_bundle(SpatialBundle::default());
    }

    fn set_children<R: MavericRoot>(commands: SetChildrenCommands<Self, Self::Context, R>) {
        commands.unordered_children_with_node_and_context(|node, context, commands| {
            let theme_font_size = context.font_size(
                &LevelInfoLayoutEntity::ThemeAndNumber,
                &ThemeLengths {
                    full_name_characters: node.full_name.len(),
                },
            );

            let title_color = match node.background_type {
                BackgroundType::Congrats | BackgroundType::NonLevel => {
                    palette::THEME_TITLE_COLOR_COMPLETE_NORMAL
                }
                BackgroundType::Selfie => palette::THEME_TITLE_COLOR_SELFIE,
                BackgroundType::Normal => palette::THEME_TITLE_COLOR_INCOMPLETE_NORMAL,
            }
            .convert_color();

            let text_anchor = if node.is_level_complete {
                bevy::sprite::Anchor::Center
            } else {
                bevy::sprite::Anchor::CenterLeft
            };

            let origin: Vec2 = context.get_origin(
                &LevelInfoLayoutEntity::ThemeAndNumber,
                &(
                    (node.selfie_mode, node.insets),
                    IsLevelComplete(node.is_level_complete),
                ),
            );

            commands.add_child(
                "theme",
                Text2DNode {
                    text: node.full_name.to_string(),
                    font_size: theme_font_size,
                    color: title_color,
                    font: THEME_FONT_PATH,
                    justify_text: JustifyText::Left,
                    linebreak_behavior: bevy::text::BreakLineOn::NoWrap,
                    text_2d_bounds: Default::default(),
                    text_anchor,
                }
                .with_bundle(Transform::from_translation(
                    origin.extend(crate::z_indices::THEME),
                )),
                &(),
            );

            let info_font_size = context.font_size(
                &LevelInfoLayoutEntity::ThemeInfo,
                &ThemeLengths {
                    full_name_characters: node.full_name.len(),
                },
            );

            if let Some(info) = node.info {
                let theme_color = match node.background_type {
                    BackgroundType::Congrats | BackgroundType::NonLevel => {
                        palette::THEME_INFO_COLOR_COMPLETE_NORMAL
                    }
                    BackgroundType::Selfie => palette::THEME_INFO_COLOR_SELFIE,
                    BackgroundType::Normal => palette::THEME_INFO_COLOR_INCOMPLETE_NORMAL,
                }
                .convert_color();

                let origin: Vec2 = context.get_origin(
                    &LevelInfoLayoutEntity::ThemeInfo,
                    &(
                        (node.selfie_mode, node.insets),
                        IsLevelComplete(node.is_level_complete),
                    ),
                );

                commands.add_child(
                    "info",
                    Text2DNode {
                        text: info.to_string(),
                        font_size: info_font_size,
                        color: theme_color,
                        font: THEME_INFO_FONT_PATH,
                        justify_text: JustifyText::Left,
                        linebreak_behavior: bevy::text::BreakLineOn::NoWrap,
                        text_2d_bounds: Default::default(),
                        text_anchor,
                    }
                    .with_bundle((Transform::from_translation(
                        origin.extend(crate::z_indices::THEME),
                    ),)),
                    &(),
                );
            }
        });
    }
}
