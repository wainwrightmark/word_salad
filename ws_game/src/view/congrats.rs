use std::time::Duration;

use crate::{prelude::*, z_indices};
use bevy::{reflect::TypeUuid, sprite::Anchor, text::Text2dBounds};
use bevy_param_shaders::prelude::*;
use maveric::{widgets::text2d_node::Text2DNode, with_bundle::CanWithBundle};
use num_traits::Zero;
use rand::{rngs::ThreadRng, Rng};
use ws_core::layout::entities::*;
#[derive(Debug, Clone, PartialEq)]
pub struct CongratsView;

fn create_firework(
    cb: &mut ChildBuilder,
    rng: &mut impl Rng,
    total_seconds: f32,
    size: &Size,
    no_delay: bool,
) {
    let rect = size.get_rect(&ws_core::layout::entities::GameLayoutEntity::Grid, &());

    let delay = if no_delay {
        0.0
    } else {
        rng.gen_range(0.0..(total_seconds - 1.0))
    };
    let color = Color::hsl(rng.gen_range(0.0..=360.0), 1.0, 0.75);

    let position = rect.top_left
        + Vec2 {
            x: rng.gen_range(0.0..=(rect.extents.x.abs())),
            y: rng.gen_range(rect.extents.y..=0.0),
        };

    let bundle = (
        ShaderBundle::<FireworksShader> {
            parameters: (color.into(), 0.0.into()),
            transform: Transform {
                translation: position.extend(z_indices::FIREWORKS),

                scale: Vec3::ONE * rect.extents.x.max(rect.extents.y),
                ..default()
            },
            ..default()
        },
        ScheduledForDeletion {
            remaining: Duration::from_secs_f32(1.0),
        },
        TransitionBuilder::<ProgressLens>::default()
            .then_tween(1.0, 1.0.into())
            .build(),
    );

    if delay.is_zero() {
        cb.spawn(bundle);
    } else {
        cb.spawn(ScheduledChange {
            remaining: Duration::from_secs_f32(delay),
            boxed_change: Box::new(move |ec| {
                ec.insert(bundle);
            }),
        });
    }
}

impl MavericNode for CongratsView {
    type Context = ViewContext;

    fn set_components(mut commands: SetComponentCommands<Self, Self::Context>) {
        commands.insert_static_bundle(SpatialBundle::default());
    }

    fn on_created(
        &self,
        context: &<Self::Context as NodeContext>::Wrapper<'_>,
        _world: &World,
        entity_commands: &mut bevy::ecs::system::EntityCommands,
    ) {
        if !context.2.is_changed() || context.2.is_added() {
            return;
        }

        //SHOW FIREWORKS
        let size = &context.3;

        const SECONDS: f32 = 5.0;
        const NUM_FIREWORKS: usize = 25;


        entity_commands.with_children(|cb| {
            let mut rng = ThreadRng::default();
            for i in 0..NUM_FIREWORKS {
                create_firework(cb, &mut rng, SECONDS, size.as_ref(), i <= 1);
            }
        });
    }

    fn set_children<R: MavericRoot>(commands: SetChildrenCommands<Self, Self::Context, R>) {
        commands
            .ignore_node()
            .unordered_children_with_context(|context, commands| {
                let size = &context.3;

                let hints_used_text = match context.2.hints_used1 {
                    0 => "No hints used".to_string(),
                    1 => "1 hint used".to_string(),
                    n => format!("{n} hints used"),
                };
                let selfie_mode = SelfieMode(context.8.is_selfie_mode);

                //let full_rect = size.get_rect(GameLayoutEntity::, context)

                commands.add_child(
                    "hints used",
                    Text2DNode {
                        text: hints_used_text,
                        font_size: size.font_size(&CongratsLayoutEntity::HintsUsed),
                        color: palette::BUTTON_TEXT_COLOR.convert_color(),
                        font: BUTTONS_FONT_PATH,
                        alignment: TextAlignment::Center,
                        linebreak_behavior: bevy::text::BreakLineOn::NoWrap,
                        text_2d_bounds: Text2dBounds::default(),
                        text_anchor: Anchor::default(),
                    }
                    .with_bundle(Transform::from_translation(
                        size.get_rect(&CongratsLayoutEntity::HintsUsed, &selfie_mode)
                            .centre()
                            .extend(crate::z_indices::CONGRATS_BUTTON),
                    ))
                    .with_transition_in::<TransformScaleLens>(
                        Vec3::ZERO,
                        Vec3::ONE,
                        Duration::from_secs_f32(1.0),
                    ),
                    &(),
                );

                commands.add_child(
                    "next level",
                    WSButtonNode {
                        text: "Next",
                        font_size: size.font_size(&CongratsLayoutEntity::NextButton),
                        rect: size.get_rect(&CongratsLayoutEntity::NextButton, &selfie_mode),
                        interaction: ButtonInteraction::Congrats(CongratsLayoutEntity::NextButton),
                        text_color: palette::CONGRATS_BUTTON_TEXT.convert_color(),
                        fill_color: palette::CONGRATS_BUTTON_FILL.convert_color(),
                    }
                    .with_transition_in::<TransformScaleLens>(
                        Vec3::ZERO,
                        Vec3::ONE,
                        Duration::from_secs_f32(1.0),
                    ),
                    &(),
                );

                #[cfg(target_arch = "wasm32")]
                {
                    commands.add_child(
                        "share",
                        WSButtonNode {
                            text: "Share",
                            font_size: size.font_size(&CongratsLayoutEntity::ShareButton),
                            rect: size.get_rect(&CongratsLayoutEntity::ShareButton, &selfie_mode),
                            interaction: ButtonInteraction::Congrats(
                                CongratsLayoutEntity::ShareButton,
                            ),
                            text_color: palette::CONGRATS_BUTTON_TEXT.convert_color(),
                            fill_color: palette::CONGRATS_BUTTON_FILL.convert_color(),
                        }
                        .with_transition_in::<TransformScaleLens>(
                            Vec3::ZERO,
                            Vec3::ONE,
                            Duration::from_secs_f32(1.0),
                        ),
                        &(),
                    );
                }
            });
    }
}

#[repr(C)]
#[derive(Debug, Reflect, Clone, Copy, TypeUuid, Default, PartialEq)]
#[uuid = "3b76cd37-5f82-4fcc-9d26-ea6f60d616e3"]
pub struct FireworksShader;

impl ParameterizedShader for FireworksShader {
    type Params = FireworksParams;
    type ParamsQuery<'a> = (&'a ShaderColor, &'a ShaderProgress);
    type ParamsBundle = (ShaderColor, ShaderProgress);

    fn get_params<'w, 'a>(
        query_item: <Self::ParamsQuery<'a> as bevy::ecs::query::WorldQuery>::Item<'w>,
    ) -> Self::Params {
        FireworksParams {
            color: query_item.0.color.into(),
            progress: query_item.1.progress,
        }
    }

    fn fragment_body() -> impl Into<String> {
        "return fill::fireworks::fill(in.color, in.pos, in.progress);"
    }

    fn imports() -> impl Iterator<Item = bevy_param_shaders::prelude::FragmentImport> {
        [FIREWORKS_IMPORT].into_iter()
    }

    const FRAME: Frame = Frame::square(0.25);
}


#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Default, Reflect, bytemuck::Pod, bytemuck::Zeroable)]
pub struct FireworksParams {
    pub color: LinearRGB,
    pub progress: f32,
}

impl ShaderParams for FireworksParams {}

const FIREWORKS_IMPORT: FragmentImport = FragmentImport {
    path: "shaders/fill/fireworks.wgsl",
    import_path: "fill::fireworks",
};
