use std::time::Duration;

use crate::prelude::*;
use bevy::sprite::Anchor;
use bevy::text::Text2dBounds;
use bevy_param_shaders::ShaderBundle;
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

const ANIMATE_SECONDS: f32 = 2.0;

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

        let size = context.as_ref();
        let hints_rect = size.get_rect(&LayoutTopBar::HintCounter, &());
        let hint_font_size = size.font_size::<LayoutTopBar>(&LayoutTopBar::WordSaladLogo);

        let final_translation = (hints_rect.centre() + (Vec2::X * hint_font_size * 0.03))
            .extend(crate::z_indices::TOP_BAR_BUTTON - 1.0);
        let initial_translation = Vec2::ZERO.extend(crate::z_indices::TOP_BAR_BUTTON - 1.0);
        let speed = calculate_speed(
            &initial_translation,
            &final_translation,
            Duration::from_secs_f32(ANIMATE_SECONDS),
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

        let circle_bundle = ShaderBundle::<CircleShader> {
            transform: circle_transform,
            parameters: ShaderColor {
                color: palette::HINT_COUNTER_COLOR.convert_color(),
            },
            ..default()
        };

        let circle_bundle = (
            circle_bundle,
            ScheduledForDeletion {
                remaining: Duration::from_secs_f32(ANIMATE_SECONDS),
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
                remaining: Duration::from_secs_f32(ANIMATE_SECONDS),
            },
            TransitionBuilder::<TransformTranslationLens>::default()
                .then_tween(final_translation, speed)
                .build(),
        );

        entity_commands.with_children(|cb| {
            cb.spawn(circle_bundle);
            cb.spawn(text_bundle);
        });

        if let Some(children) = world.get::<Children>(entity_commands.id()) {
            for child in children {
                if let Some(text) = world.get::<Text>(*child) {
                    let mut new_text = text.clone();

                    if let Some(t) = new_text.sections.get_mut(0) {
                        t.value = self.hint_state.hints_remaining.to_string();
                    }

                    entity_commands
                        .commands()
                        .entity(*child)
                        .insert(ScheduledChange {
                            remaining: Duration::from_secs_f32(ANIMATE_SECONDS),
                            boxed_change: Box::new(|ec| {
                                ec.insert(new_text);
                            }),
                        });
                    break;
                }
            }
        }

        //entity_commands.commands().entity(entity)
    }

    fn set_components(commands: SetComponentCommands<Self, Self::Context>) {
        commands
            .ignore_context()
            .ignore_node()
            .insert(SpatialBundle::default())
            .finish()
    }

    fn set_children<R: MavericRoot>(commands: SetChildrenCommands<Self, Self::Context, R>) {
        {
            commands.unordered(|a, commands| {
                let node = a.node;
                let context = a.context;

                let size = context.as_ref();
                let hints_rect = size.get_rect(&LayoutTopBar::HintCounter, &());
                let hint_font_size = size.font_size::<LayoutTopBar>(&LayoutTopBar::HintCounter);

                let text: String = if let Some(prev) = a.previous.filter(|p| {
                    p.hint_state.total_earned_hints < node.hint_state.total_earned_hints
                }) {
                    prev.hint_state.hints_remaining.to_string()
                } else {
                    node.hint_state.hints_remaining.to_string()
                };

                commands.add_child(
                    "hints",
                    Text2DNode {
                        text,
                        font_size: hint_font_size,
                        color: palette::BUTTON_TEXT_COLOR.convert_color(),
                        font: BUTTONS_FONT_PATH,
                        alignment: TextAlignment::Center,
                        linebreak_behavior: bevy::text::BreakLineOn::NoWrap,
                        text_anchor: Anchor::default(),
                        text_2d_bounds: Text2dBounds::default(),
                    }
                    .with_bundle((Transform::from_translation(
                        hints_rect.centre().extend(crate::z_indices::TOP_BAR_BUTTON),
                    ),)),
                    &(),
                );

                commands.add_child(
                    "hints_box",
                    ShaderBundle::<CircleShader> {
                        transform: Transform {
                            translation: (hints_rect.centre() + (Vec2::X * hint_font_size * 0.03))
                                .extend(crate::z_indices::TOP_BAR_BUTTON - 1.0),
                            scale: Vec3::ONE * hints_rect.width() * CIRCLE_SCALE,
                            rotation: Default::default(),
                        },
                        parameters: ShaderColor {
                            color: palette::HINT_COUNTER_COLOR.convert_color(),
                        },
                        ..default()
                    },
                    &(),
                );
            })
        }
    }
}
