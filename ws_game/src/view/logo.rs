use crate::prelude::*;
use maveric::with_bundle::CanWithBundle;
use ws_core::layout::entities::*;
#[derive(Debug, NodeContext)]
pub struct LogoContext {
    pub window_size: MyWindowSize,
    pub video_resource: VideoResource,
    pub pressed_button: PressedButton,
}

#[derive(Debug, PartialEq, Clone, Copy, MavericRoot)]
pub struct WordSaladLogoRoot;

impl MavericRootChildren for WordSaladLogoRoot {
    type Context = LogoContext;

    fn set_children(
        context: &<Self::Context as NodeContext>::Wrapper<'_>,
        commands: &mut impl ChildCommands,
    ) {
        let size = &context.window_size;

        let logo_rect = size.get_rect(&WordSaladLogo, &context.video_resource.selfie_mode());

        let pressed_multiplier = match context.pressed_button.as_ref() {
            PressedButton::Pressed {
                interaction: ButtonInteraction::WordSaladLogo,
                ..
            } => 1.1,
            _ => 1.0,
        };

        commands.add_child(
            "Word Salad Icon",
            SpriteNode {
                texture_path: r#"embedded://ws_game/../../assets/images/logo1024.png"#,
                sprite: Sprite {
                    custom_size: Some(logo_rect.extents.abs() * pressed_multiplier),
                    ..Default::default()
                },
            }
            .with_bundle((Transform::from_translation(
                logo_rect.centre().extend(crate::z_indices::TOP_BAR_BUTTON),
            ),)),
            &(),
        );
    }
}

// #[derive(Debug, Clone, Copy, Default, PartialEq)]
// struct LogoImageNodeStyle;

// impl IntoBundle for LogoImageNodeStyle {
//     type B = Style;

//     fn into_bundle(self) -> Self::B {
//         Style {
//             width: Val::Px(100.0),
//             height: Val::Px(100.0),
//             margin: UiRect::DEFAULT,
//             align_self: AlignSelf::Center,
//             justify_self: JustifySelf::Center,
//             ..default()
//         }
//     }
// }
