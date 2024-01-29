use crate::prelude::*;
use maveric::widgets::text2d_node::Text2DNode;
use maveric::with_bundle::CanWithBundle;
use ws_core::layout::entities::level_info_entity::{LevelInfoLayoutEntity, ThemeLengths};
use ws_core::prelude::*;

#[derive(Debug, Clone, PartialEq)]
pub struct LevelExtraInfo {
    pub info: Ustr,
    pub full_name_characters: usize,

}

impl MavericNode for LevelExtraInfo {
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
                &LevelInfoLayoutEntity::ThemeInfoAndTimer,
                &ThemeLengths {
                    full_name_characters: node.full_name_characters,
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
                "info",
                Text2DNode {
                    text: node.info.to_string(),
                    font_size: theme_font_size,
                    color,
                    font: THEME_INFO_FONT_PATH,
                    alignment: TextAlignment::Center,
                    linebreak_behavior: bevy::text::BreakLineOn::NoWrap,
                    text_2d_bounds: Default::default(),
                    text_anchor: bevy::sprite::Anchor::Center,
                }
                .with_bundle((Transform::from_translation(
                    context.window_size
                        .get_rect(&LevelInfoLayoutEntity::ThemeInfoAndTimer, &context.video_resource.selfie_mode())
                        .centre()
                        .extend(crate::z_indices::THEME),
                ), TimeCounterMarker{theme_info: node.info})),
                &(),
            );
        });
    }
}
