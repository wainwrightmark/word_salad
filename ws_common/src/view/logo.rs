use crate::prelude::*;
use maveric::with_bundle::CanWithBundle;
use nice_bevy_utils::window_size::WindowSize;
use ws_core::layout::entities::{level_info_entity::IsLevelComplete, *};

pub fn logo(
    window_size: &WindowSize<SaladWindowBreakPoints>,
    video_resource: &VideoResource,
    insets_resource: &InsetsResource,
    background_type: BackgroundType,
    is_level_complete: bool,
) -> impl MavericNode<Context = ()> {
    //todo scale by 1.1 when button pressed
    const LOGO_WHITE_PATH: &str =
        r#"embedded://ws_common/../../assets/images/icon-white-circle1024.png"#;
    const LOGO_NORMAL_PATH: &str = r#"embedded://ws_common/../../assets/images/icon1024.png"#;

    let texture_path = match background_type {
        BackgroundType::Congrats => LOGO_WHITE_PATH,
        BackgroundType::NonLevel => LOGO_WHITE_PATH,
        BackgroundType::Selfie => LOGO_NORMAL_PATH,
        BackgroundType::Normal => LOGO_NORMAL_PATH,
    };

    let logo_rect = window_size.get_rect(
        &WordSaladLogo,
        &(
            (video_resource.selfie_mode(), insets_resource.0),
            IsLevelComplete(is_level_complete),
        ),
    );

    SpriteNode {
        texture_path,
        sprite: Sprite {
            custom_size: Some(logo_rect.extents.abs()),
            ..Default::default()
        },
    }
    .with_bundle((Transform::from_translation(
        logo_rect.centre().extend(crate::z_indices::TOP_BAR_BUTTON),
    ),))
}
