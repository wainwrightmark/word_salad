use std::time::Duration;

use crate::prelude::*;
use bevy::sprite::Anchor;
use bevy::text::Text2dBounds;
use bevy_smud::param_usage::ShaderParamUsage;
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

const CIRCLE_SCALE: f32 = 0.5;

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
        let hints_rect = size.get_rect(&LayoutTopBar::HintCounter, &());
        let hint_font_size = size.font_size::<LayoutTopBar>(&LayoutTopBar::WordSaladLogo);

        let final_translation = (hints_rect.centre() + (Vec2::X * hint_font_size * 0.03))
            .extend(crate::z_indices::TOP_BAR_BUTTON - 1.0);
        let initial_translation = Vec2::ZERO.extend(crate::z_indices::TOP_BAR_BUTTON - 1.0);
        let speed = calculate_speed(
            &initial_translation,
            &final_translation,
            Duration::from_secs_f32(SECONDS),
        );

        let circle_transform = Transform {
            translation: initial_translation,
            scale: Vec3::ONE * hints_rect.width() * CIRCLE_SCALE,
            rotation: Default::default(),
        };

        let Some(asset_server) = world.get_resource::<AssetServer>() else {
            return;
        };
        let font = asset_server.load(BUTTONS_FONT_PATH);

        let sdf: Handle<_> = asset_server.load(CIRCLE_SHADER_PATH);
        let fill = asset_server.load(SIMPLE_FILL_SHADER_PATH);

        let circle_bundle = ShapeBundle::<SHAPE_F_PARAMS, SHAPE_U_PARAMS> {
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
            transform: circle_transform,
            ..default()
        };

        let circle_bundle = (
            circle_bundle,
            ScheduledForDeletion {
                remaining: Duration::from_secs_f32(SECONDS),
            },
            TransitionBuilder::<TransformTranslationLens>::default()
                .then_tween(final_translation, speed)
                .build(),
        );

        let text_transform = Transform {
            translation: initial_translation + Vec3::Z,
            scale: Vec3::ONE,
            rotation: Default::default(),
        };

        let text_bundle = Text2dBundle {
            text: Text::from_section(
                "2",
                TextStyle {
                    font_size: hint_font_size,
                    color: palette::BUTTON_TEXT_COLOR.convert_color(),
                    font,
                },
            )
            .with_alignment(TextAlignment::Center)
            .with_no_wrap(),

            text_anchor: Anchor::default(),
            text_2d_bounds: Text2dBounds::default(),

            transform: text_transform,
            ..Default::default()
        };

        let text_bundle = (
            text_bundle,
            ScheduledForDeletion {
                remaining: Duration::from_secs_f32(SECONDS),
            },
            TransitionBuilder::<TransformTranslationLens>::default()
                .then_tween(final_translation, speed)
                .build(),
        );

        entity_commands.with_children(|cb| {
            cb.spawn(circle_bundle);
            cb.spawn(text_bundle);
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
            let hints_rect = size.get_rect(&LayoutTopBar::HintCounter, &());
            let hint_font_size = size.font_size::<LayoutTopBar>(&LayoutTopBar::HintCounter);

            commands.add_child(
                "hints",
                Text2DNode {
                    text: node.hint_state.hints_remaining.to_string(),
                    font_size: hint_font_size,
                    color: palette::BUTTON_TEXT_COLOR.convert_color(),
                    font: BUTTONS_FONT_PATH,
                    alignment: TextAlignment::Center,
                    linebreak_behavior: bevy::text::BreakLineOn::NoWrap,
                    text_anchor: Anchor::default(),
                    text_2d_bounds: Text2dBounds::default(),
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
        });
    }
}
