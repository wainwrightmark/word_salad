use crate::prelude::*;
use bevy_param_shaders::ShaderBundle;
use ws_core::layout::entities::recording_button::ToggleRecordingButton;

#[derive(Debug, NodeContext)]
pub struct RecordingButtonContext {
    pub window_size: MyWindowSize,
    pub video_resource: VideoResource,
}

impl<'a, 'w: 'a> From<&'a ViewContextWrapper<'w>> for RecordingButtonContextWrapper<'w> {
    fn from(value: &'a ViewContextWrapper<'w>) -> Self {
        Self {
            window_size: Res::clone(&value.window_size),
            video_resource: Res::clone(&value.video_resource),
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct RecordingButtonNode;

impl MavericNode for RecordingButtonNode {
    type Context = RecordingButtonContext;

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

                let rect = size.get_rect(
                    &ToggleRecordingButton,
                    &context.video_resource.selfie_mode(),
                );

                let inner_color: Color;
                let outer_color: Color = if context.video_resource.is_selfie_mode {
                    palette::RECORDING_BUTTON_SELFIE.convert_color()
                } else {
                    palette::RECORDING_BUTTON_NORMAL.convert_color()
                };
                let inner_rounding: f32;

                if context.video_resource.is_recording {
                    inner_color = palette::RECORDING_BUTTON_RECORDING.convert_color();
                    inner_rounding = 0.0;
                } else {
                    inner_color = outer_color;
                    inner_rounding = 1.0;
                }

                commands.add_child(
                    "OuterCircle",
                    ShaderBundle::<BoxWithBorderShader> {
                        parameters: (
                            Color::NONE.into(),
                            ShaderRounding { rounding: 1.0 },
                            ShaderProportions::default(),
                            ShaderBorder {
                                border_color: outer_color,
                                border: 0.1,
                            },
                        ),
                        transform: Transform::from_translation(
                            rect.centre_left().extend(crate::z_indices::TOP_BAR_BUTTON),
                        )
                        .with_scale(Vec3::ONE * rect.width()),

                        ..Default::default()
                    },
                    &(),
                );

                commands.add_child(
                    "InnerShape",
                    ShaderBundle::<BasicBoxShaderExtraction> {
                        parameters: (
                            inner_color.into(),
                            ShaderRounding {
                                rounding: inner_rounding,
                            },
                            ShaderProportions::default(),
                        ),
                        transform: Transform::from_translation(
                            rect.centre_left().extend(crate::z_indices::TOP_BAR_BUTTON),
                        )
                        .with_scale(Vec3::ONE * rect.width() * 0.5),

                        ..Default::default()
                    },
                    &(),
                );
            });
    }
}
