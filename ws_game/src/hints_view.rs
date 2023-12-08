use std::time::Duration;

use crate::prelude::*;
use bevy_smud::param_usage::{ShaderParamUsage, ShaderParameter};
use bevy_smud::{ShapeBundle, SmudShaders};
use maveric::transition::speed::calculate_speed;
use maveric::widgets::text2d_node::Text2DNode;
use maveric::with_bundle::CanWithBundle;
pub use maveric::*;
use ws_core::layout::entities::*;
use ws_core::prelude::*;

#[derive(Debug, PartialEq)]
pub struct HintsViewNode {
    pub hint_state: HintState,
}

const CIRCLE_SCALE: f32 = 0.4;

impl MavericNode for HintsViewNode {
    type Context = MyWindowSize;

    fn on_changed(
        &self,
        previous: &Self,
        context: &<Self::Context as NodeContext>::Wrapper<'_>,
        world: &World,
        entity_commands: &mut bevy::ecs::system::EntityCommands,
    ) {
        if previous.hint_state.total_earned_hints >= self.hint_state.total_earned_hints {
            return;
        }

        const SECONDS: f32 = 1.0;

        let size = context.as_ref();
        let hints_rect = size.get_rect(&LayoutTopBarButton::HintCounter, &());
        let hint_font_size =
            size.font_size::<LayoutTopBarButton>(&LayoutTopBarButton::WordSaladButton);

        let final_translation = (hints_rect.centre() + (Vec2::X * hint_font_size * 0.03))
            .extend(crate::z_indices::TOP_BAR_BUTTON - 1.0);
        let initial_translation = Vec2::ZERO.extend(crate::z_indices::TOP_BAR_BUTTON - 1.0);
        let speed = calculate_speed(
            &initial_translation,
            &final_translation,
            Duration::from_secs_f32(SECONDS),
        );

        let transform = Transform {
            translation: initial_translation,
            scale: Vec3::ONE * hints_rect.width() * CIRCLE_SCALE,
            rotation: Default::default(),
        };

        let Some(asset_server) = world.get_resource::<AssetServer>() else {
            return;
        };

        let sdf: Handle<_> = asset_server.load(CIRCLE_SHADER_PATH);
        let fill = asset_server.load(SIMPLE_FILL_SHADER_PATH);

        let bundle = ShapeBundle::<SHAPE_F_PARAMS, SHAPE_U_PARAMS> {
            shape: bevy_smud::SmudShape {
                color: palette::HINT_COUNTER_COLOR.convert_color(),
                frame: bevy_smud::Frame::Quad(1.0),
                f_params: [0.0; SHAPE_F_PARAMS],
                u_params: [0; SHAPE_U_PARAMS],
            },
            shaders: SmudShaders {
                sdf,
                fill,
                sdf_param_usage: ShaderParamUsage::NO_PARAMS,
                fill_param_usage: ShaderParamUsage::NO_PARAMS,
            },
            transform,
            ..default()
        };

        let bundle = (
            bundle,
            ScheduledForDeletion {
                timer: Timer::from_seconds(SECONDS, TimerMode::Once),
            },
            Transition::<TransformTranslationLens>::new(TransitionStep::new_arc(
                final_translation,
                Some(speed),
                NextStep::None,
            )),
        );

        entity_commands.with_children(|x| {
            x.spawn(bundle);
        });
    }

    fn set_components(commands: SetComponentCommands<Self, Self::Context>) {
        commands
            .ignore_context()
            .ignore_node()
            .insert(SpatialBundle::default())
            .finish()
    }

    fn set_children<R: MavericRoot>(commands: SetChildrenCommands<Self, Self::Context, R>) {
        commands.unordered_children_with_node_and_context(|node, context, commands| {
            let size = context.as_ref();
            let hints_rect = size.get_rect(&LayoutTopBarButton::HintCounter, &());
            let hint_font_size =
                size.font_size::<LayoutTopBarButton>(&LayoutTopBarButton::WordSaladButton);

            commands.add_child(
                "hints",
                Text2DNode {
                    text: node.hint_state.hints_remaining.to_string(),
                    font_size: hint_font_size,
                    color: palette::BUTTON_TEXT_COLOR.convert_color(),
                    font: BUTTONS_FONT_PATH,
                    alignment: TextAlignment::Center,
                    linebreak_behavior: bevy::text::BreakLineOn::NoWrap,
                }
                .with_bundle(Transform::from_translation(
                    hints_rect.centre().extend(crate::z_indices::TOP_BAR_BUTTON),
                )),
                &(),
            );

            commands.add_child(
                "hints_box",
                SmudShapeNode {
                    color: palette::HINT_COUNTER_COLOR.convert_color(),
                    sdf: CIRCLE_SHADER_PATH,
                    fill: SIMPLE_FILL_SHADER_PATH,
                    frame_size: 1.0,
                    f_params: [0.0; SHAPE_F_PARAMS],
                    u_params: [0; SHAPE_U_PARAMS],
                    sdf_param_usage: ShaderParamUsage::NO_PARAMS,
                    fill_param_usage: ShaderParamUsage::NO_PARAMS,
                }
                .with_bundle(Transform {
                    translation: (hints_rect.centre() + (Vec2::X * hint_font_size * 0.03))
                        .extend(crate::z_indices::TOP_BAR_BUTTON - 1.0),
                    scale: Vec3::ONE * hints_rect.width() * CIRCLE_SCALE,
                    rotation: Default::default(),
                }),
                &(),
            );

            const SPARKLE_FILL_PARAMETERS: &[ShaderParameter] = &[
                ShaderParameter::f32(0),
                ShaderParameter::f32(1),
                ShaderParameter::f32(2),
            ];

            commands.add_child(
                "hints_sparkle",
                SmudShapeNode {
                    color: palette::HINT_COUNTER_COLOR.convert_color(),
                    sdf: CIRCLE_SHADER_PATH,
                    fill: SPARKLE_SHADER_PATH,
                    frame_size: 0.9,
                    f_params: [3.0, 2.0, 56789.0, 0.0, 0.0, 0.0],
                    u_params: [0; SHAPE_U_PARAMS],
                    sdf_param_usage: ShaderParamUsage::NO_PARAMS,
                    fill_param_usage: ShaderParamUsage(SPARKLE_FILL_PARAMETERS),
                }
                .with_bundle(Transform {
                    translation: (hints_rect.centre() + (Vec2::X * hint_font_size * 0.03))
                        .extend(crate::z_indices::TOP_BAR_BUTTON - 0.5),
                    scale: Vec3::ONE * hints_rect.width() * 0.4,
                    rotation: Default::default(),
                }),
                &(),
            );
        });
    }
}
