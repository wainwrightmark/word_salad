use crate::prelude::*;
use maveric::widgets::text2d_node::Text2DNode;
use maveric::with_bundle::CanWithBundle;
use ws_core::layout::entities::*;
use ws_core::{font_icons, prelude::*};

#[derive(Debug, NodeContext)]
pub struct TopBarContext {
    pub window_size: MyWindowSize,
    // pub hint_state: HintState,
    pub video_resource: VideoResource,
}

impl<'a, 'w: 'a> From<&'a ViewContextWrapper<'w>> for TopBarContextWrapper<'w> {
    fn from(value: &'a ViewContextWrapper<'w>) -> Self {
        Self {
            // hint_state: Res::clone(&value.hint_state),
            window_size: Res::clone(&value.window_size),
            video_resource: Res::clone(&value.video_resource),
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct TopBar;

impl MavericNode for TopBar {
    type Context = TopBarContext;

    fn set_components(commands: SetComponentCommands<Self, Self::Context>) {
        commands
            .ignore_context()
            .ignore_node()
            .insert(SpatialBundle::default())
            .finish()
    }

    fn set_children<R: MavericRoot>(commands: SetChildrenCommands<Self, Self::Context, R>) {
        commands
            .ignore_node()
            .unordered_children_with_context(|context, commands| {
                let size = &context.window_size;

                let selfie = context.video_resource.selfie_mode().is_selfie_mode;

                //play / record button
                if selfie {
                    let (text, color) = if context.video_resource.is_recording {
                        let color = Color::RED;
                        (font_icons::STOP_CIRCLED, color)
                    } else {
                        let color = (if selfie {
                            palette::TOP_BAR_BURGER_SELFIE
                        } else {
                            palette::TOP_BAR_BURGER_NORMAL
                        })
                        .convert_color();

                        (font_icons::RECORD_CIRCLED, color)
                    };

                    commands.add_child(
                        "ToggleRecording",
                        Text2DNode {
                            text,
                            font_size: size.font_size::<LayoutTopBar>(
                                &LayoutTopBar::ToggleRecordingButton,
                                &(),
                            ),
                            color,
                            font: ICON_FONT_PATH,
                            alignment: TextAlignment::Left,
                            linebreak_behavior: bevy::text::BreakLineOn::NoWrap,
                            text_2d_bounds: Default::default(),
                            text_anchor: bevy::sprite::Anchor::CenterLeft,
                        }
                        .with_bundle(Transform::from_translation(
                            size.get_rect(
                                &LayoutTopBar::ToggleRecordingButton,
                                &context.video_resource.selfie_mode(),
                            )
                            .centre_left()
                            .extend(crate::z_indices::TOP_BAR_BUTTON),
                        )),
                        &(),
                    );
                }

                // commands.add_child(
                //     "hints",
                //     HintsViewNode {
                //         hint_state: context.hint_state.clone(),
                //         selfie_mode: context.video_resource.selfie_mode(),
                //     },
                //     size,
                // );

                let logo_rect = size.get_rect(
                    &LayoutTopBar::WordSaladLogo,
                    &context.video_resource.selfie_mode(),
                );

                commands.add_child(
                    "Word Salad Icon",
                    SpriteNode {
                        texture_path: r#"images/logo1024.png"#,
                        sprite: Sprite {
                            custom_size: Some(logo_rect.extents.abs()),
                            ..Default::default()
                        },
                    }
                    .with_bundle((Transform::from_translation(
                        logo_rect.centre().extend(crate::z_indices::TOP_BAR_BUTTON),
                    ),)),
                    &(),
                );

                // if node.background_type.is_non_level() || node.background_type.is_congrats() {
                //     let rect = logo_rect;
                //     let circle_bundle = ShaderBundle::<CircleShader> {
                //         transform: Transform::from_translation(
                //             rect.centre()
                //             .extend(crate::z_indices::TOP_BAR_BUTTON - 1.0),
                //         ).with_scale(Vec3::ONE * rect.width().abs() * 0.5),
                //         parameters: ShaderColor {
                //             color: palette::TOP_BAR_LOGO_CIRCLE.convert_color(),
                //         },
                //         ..default()
                //     };

                //     commands.add_child("Icon Circle", circle_bundle, &());
                // }
            });
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
struct LogoImageNodeStyle;

impl IntoBundle for LogoImageNodeStyle {
    type B = Style;

    fn into_bundle(self) -> Self::B {
        Style {
            width: Val::Px(100.0),
            height: Val::Px(100.0),
            margin: UiRect::DEFAULT,
            align_self: AlignSelf::Center,
            justify_self: JustifySelf::Center,
            ..default()
        }
    }
}
