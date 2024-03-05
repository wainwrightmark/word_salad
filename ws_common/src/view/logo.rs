use std::time::Duration;

use crate::prelude::*;
use maveric::{define_lens, with_bundle::CanWithBundle};
use ws_core::layout::entities::*;
#[derive(Debug, NodeContext)]
pub struct LogoContext {
    pub window_size: MyWindowSize,
    pub video_resource: VideoResource,
    pub pressed_button: PressedButton,
    pub insets_resource: InsetsResource,
    pub current_level: CurrentLevel,
    pub found_words_state: FoundWordsState,
}

define_lens!(SpriteColorLens, Sprite, Color, color);

#[derive(Debug, PartialEq, Clone, Copy, MavericRoot)]
pub struct WordSaladLogoRoot;

impl MavericRootChildren for WordSaladLogoRoot {
    type Context = LogoContext;

    fn set_children(
        context: &<Self::Context as NodeContext>::Wrapper<'_>,
        commands: &mut impl ChildCommands,
    ) {
        let background_type = background_type_from_resources(
            &context.video_resource,
            &context.current_level,
            &context.found_words_state,
        );

        let size = &context.window_size;

        let logo_rect = size.get_rect(
            &WordSaladLogo,
            &(
                context.video_resource.selfie_mode(),
                context.insets_resource.0,
            ),
        );

        let pressed_multiplier = match context.pressed_button.as_ref() {
            PressedButton::Pressed {
                interaction: ButtonInteraction::WordSaladLogo,
                ..
            } => 1.1,
            _ => 1.0,
        };

        const LOGO_WHITE_PATH: &str =
            r#"embedded://ws_common/../../assets/images/icon-white-circle1024.png"#;
        const LOGO_NORMAL_PATH: &str = r#"embedded://ws_common/../../assets/images/icon1024.png"#;

        let (key, texture_path, in_secs) = match background_type {
            BackgroundType::Congrats => ("logo_white", LOGO_WHITE_PATH, TILE_LINGER_SECONDS),
            BackgroundType::NonLevel => ("logo_white", LOGO_WHITE_PATH, TILE_LINGER_SECONDS),
            BackgroundType::Selfie => ("logo_normal", LOGO_NORMAL_PATH, 0.0),
            BackgroundType::Normal => ("logo_normal", LOGO_NORMAL_PATH, 0.0),
        };

        commands.add_child(
            key,
            SpriteNode {
                texture_path,
                sprite: Sprite {
                    custom_size: Some(logo_rect.extents.abs() * pressed_multiplier),
                    ..Default::default()
                },
            }
            .with_bundle((Transform::from_translation(
                logo_rect.centre().extend(crate::z_indices::TOP_BAR_BUTTON),
            ),))
            .with_transition_in::<SpriteColorLens>(
                Color::NONE,
                Color::WHITE,
                Duration::from_secs_f32(in_secs),

                Some(Ease::ExpoIn),
            ),

            &(),
        );
    }
}
