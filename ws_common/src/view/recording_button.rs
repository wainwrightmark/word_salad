use crate::prelude::*;
use bevy_param_shaders::ShaderBundle;
use ws_core::layout::entities::recording_button::ToggleRecordingButton;

#[derive(Debug, NodeContext)]
pub struct RecordingButtonContext {
    pub window_size: MyWindowSize,
    pub video_resource: VideoResource,
    pub pressed_button: PressedButton,
    pub insets: InsetsResource
}

#[derive(Debug, PartialEq, Clone, Copy, MavericRoot)]
pub struct RecordingButtonRoot;

impl MavericRootChildren for RecordingButtonRoot {
    type Context = RecordingButtonContext;

    fn set_children(
        context: &<Self::Context as NodeContext>::Wrapper<'_>,
        commands: &mut impl ChildCommands,
    ) {
        if !context.video_resource.show_recording_button() {
            return;
        }

        let size = &context.window_size;

        let rect = size.get_rect(
            &ToggleRecordingButton,
            &(context.video_resource.selfie_mode(), context.insets.0),
        );

        let pressed_multiplier = match context.pressed_button.as_ref() {
            PressedButton::Pressed {
                interaction: ButtonInteraction::ToggleRecordingButton,
                ..
            } => 1.1,
            _ => 1.0,
        };

        let inner_color: Color;
        let outer_color: Color = if context.video_resource.is_selfie_mode {
            palette::RECORDING_BUTTON_SELFIE.convert_color()
        } else {
            palette::RECORDING_BUTTON_NORMAL.convert_color()
        };
        let inner_rounding: f32;

        if context.video_resource.is_recording() {
            inner_color = palette::RECORDING_BUTTON_RECORDING.convert_color();
            inner_rounding = 0.1;
        } else {
            inner_color = outer_color;
            inner_rounding = 0.9;
        }

        commands.add_child(
            "OuterCircle",
            (
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
                        rect.centre().extend(crate::z_indices::TOP_BAR_BUTTON),
                    )
                    .with_scale(Vec3::ONE * rect.width() * 0.5),

                    ..Default::default()
                },
                ButtonInteraction::ToggleRecordingButton,
            ),
            &(),
        );

        let inner_scale = Vec3::ONE * rect.width() * 0.25 * pressed_multiplier;

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
                    rect.centre().extend(crate::z_indices::TOP_BAR_BUTTON),
                )
                .with_scale(inner_scale),

                ..Default::default()
            }
            .with_transition_to::<RoundingLens>(inner_rounding, 5.0.into(), None),
            &(),
        );
    }
}
