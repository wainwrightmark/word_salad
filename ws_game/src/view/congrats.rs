use std::time::Duration;

use crate::{prelude::*, z_indices};
use bevy::{reflect::TypeUuid, sprite::Anchor, text::Text2dBounds};
use bevy_param_shaders::prelude::*;
use maveric::{widgets::text2d_node::Text2DNode, with_bundle::CanWithBundle};
use num_traits::Zero;
use rand::{rngs::ThreadRng, Rng};
use strum::IntoEnumIterator;
use ws_core::layout::entities::*;
#[derive(Debug, Clone, PartialEq)]
pub struct CongratsView;

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
                let selfie_mode = SelfieMode(context.8.is_selfie_mode);

                const TRANSITION_SECS: f32 = 1.0;

                #[derive(Debug, Clone, Copy)]
                enum Data {
                    None,
                    JustHints,
                    TodaysChallenge { streak: usize, longest: usize },
                    Sequence { complete: usize, remaining: usize },
                }

                let data = match context.1.as_ref() {
                    CurrentLevel::DailyChallenge { index } => {
                        let today_index = DailyChallenges::get_today_index();
                        if today_index == Some(*index) {
                            let streak = context.10.as_ref();
                            Data::TodaysChallenge {
                                streak: streak.current,
                                longest: streak.longest,
                            }
                        } else {
                            let complete = context.7.get_daily_challenges_complete();
                            let total = today_index.unwrap_or_default() + 1;
                            let remaining = total.saturating_sub(complete);
                            Data::Sequence {
                                complete,
                                remaining,
                            }
                        }
                    }
                    CurrentLevel::Tutorial { .. } => Data::None,
                    CurrentLevel::Fixed { sequence, .. } => {
                        let complete = context.7.get_number_complete(sequence);
                        let total = sequence.level_count();
                        let remaining = total.saturating_sub(complete);
                        Data::Sequence {
                            complete,
                            remaining,
                        }
                    }
                    CurrentLevel::Custom { .. } => Data::JustHints,
                    CurrentLevel::NonLevel(_) => Data::None,
                };

                for (index, statistic) in CongratsStatistic::iter().enumerate() {
                    let data = match (statistic, data) {
                        (_, Data::None)
                        | (CongratsStatistic::Left, Data::JustHints)
                        | (CongratsStatistic::Right, Data::JustHints) => None,
                        (CongratsStatistic::Left, _)
                        | (CongratsStatistic::Middle, Data::JustHints) => {
                            Some((context.2.hints_used, "Hints"))
                        }
                        (CongratsStatistic::Middle, Data::TodaysChallenge { streak, .. }) => {
                            Some((streak, "Streak"))
                        }
                        (CongratsStatistic::Middle, Data::Sequence { complete, .. }) => {
                            Some((complete, "Complete"))
                        }
                        (CongratsStatistic::Right, Data::TodaysChallenge { longest, .. }) => {
                            Some((longest, "Longest"))
                        }
                        (CongratsStatistic::Right, Data::Sequence { remaining, .. }) => {
                            Some((remaining, "Remaining"))
                        }
                    };

                    let Some((number, label)) = data else {
                        continue;
                    };
                    let rect =
                        size.get_rect(&CongratsLayoutEntity::Statistic(statistic), &selfie_mode);
                    let number_font_size = size.font_size(&StatisticNumber);
                    let text_font_size = size.font_size(&StatisticLabel);

                    let text_color = if selfie_mode.0 {
                        palette::CONGRATS_STATISTIC_TEXT_SELFIE
                    } else {
                        palette::CONGRATS_STATISTIC_TEXT_NORMAL
                    }
                    .convert_color();

                    let fill_color = if selfie_mode.0 {
                        palette::CONGRATS_STATISTIC_FILL_SELFIE
                    } else {
                        palette::CONGRATS_STATISTIC_FILL_NORMAL
                    }
                    .convert_color();

                    commands.add_child(
                        (0u16, index as u16),
                        StatisticNode {
                            rect,
                            number,
                            text: label,
                            text_color,
                            fill_color,
                            number_font_size,
                            text_font_size,
                        }
                        .with_bundle(Transform::from_translation(
                            rect.centre().extend(z_indices::CONGRATS_BUTTON),
                        ))
                        .with_transition_in::<TransformScaleLens>(
                            Vec3::ZERO,
                            Vec3::ONE,
                            Duration::from_secs_f32(TRANSITION_SECS),
                        ),
                        &(),
                    );
                }

                let button_count  = if selfie_mode.0{
                    1
                } else{
                     3
                };

                for (index, button) in CongratsButton::iter().enumerate().take(button_count) {
                    let text = match button {
                        CongratsButton::Next => {
                            let next_level = context.1.get_next_level(context.7.as_ref());

                            match next_level {
                                CurrentLevel::Tutorial { .. } => "Next".to_string(),
                                CurrentLevel::Fixed { .. } => "Next".to_string(),
                                CurrentLevel::DailyChallenge { index: next_index } => {
                                    if let CurrentLevel::DailyChallenge {
                                        index: current_index,
                                    } = context.1.as_ref()
                                    {
                                        if next_index > *current_index {
                                            "Today's Puzzle".to_string()
                                        } else {
                                            format!("Play #{}", next_index + 1)
                                        }
                                    } else {
                                        format!("Play #{}", next_index + 1)
                                    }
                                }
                                CurrentLevel::Custom { .. } => "Next".to_string(),
                                CurrentLevel::NonLevel(_) => "Finish".to_string(),
                            }
                        }
                        CongratsButton::MoreLevels => "More Puzzles".to_string(),
                        #[cfg(target_arch = "wasm32")]
                        CongratsButton::Share => "Share".to_string(),
                    };

                    let text_color = if selfie_mode.0 {
                        palette::CONGRATS_BUTTON_TEXT_SELFIE
                    } else {
                        palette::CONGRATS_BUTTON_TEXT_NORMAL
                    }
                    .convert_color();

                    let fill_color = if selfie_mode.0 {
                        palette::CONGRATS_BUTTON_FILL_SELFIE
                    } else {
                        palette::CONGRATS_BUTTON_FILL_NORMAL
                    }
                    .convert_color();

                    commands.add_child(
                        (1u16, index as u16),
                        WSButtonNode {
                            text,
                            font_size: size.font_size(&CongratsLayoutEntity::Button(button)),
                            rect: size
                                .get_rect(&CongratsLayoutEntity::Button(button), &selfie_mode),
                            interaction: ButtonInteraction::Congrats(button),
                            text_color,
                            fill_color,
                        }
                        .with_transition_in::<TransformScaleLens>(
                            Vec3::ZERO,
                            Vec3::ONE,
                            Duration::from_secs_f32(TRANSITION_SECS),
                        ),
                        &(),
                    );
                }
            });
    }
}

#[derive(Debug, Clone, PartialEq)]
struct StatisticNode {
    rect: LayoutRectangle,
    number: usize,
    text: &'static str,
    text_color: Color,
    fill_color: Color,
    number_font_size: f32,
    text_font_size: f32,
}

impl MavericNode for StatisticNode {
    type Context = ();

    fn set_components(mut commands: SetComponentCommands<Self, Self::Context>) {
        commands.insert_static_bundle((
            GlobalTransform::default(),
            Transform::default(),
            VisibilityBundle::default(),
        ));
    }

    fn set_children<R: MavericRoot>(commands: SetChildrenCommands<Self, Self::Context, R>) {
        commands.unordered_children_with_node(|node, commands| {
            let StatisticNode {
                rect,
                number,
                text,
                text_color,
                fill_color: background_color,
                number_font_size,
                text_font_size,
            } = node;

            commands.add_child(
                "number",
                Text2DNode {
                    text: number.to_string(),
                    font_size: *number_font_size,
                    color: *text_color,
                    font: BUTTONS_FONT_PATH,
                    alignment: TextAlignment::Center,
                    linebreak_behavior: bevy::text::BreakLineOn::NoWrap,
                    text_2d_bounds: Text2dBounds::default(),
                    text_anchor: Anchor::Center,
                }
                .with_bundle(Transform::from_translation(Vec3 {
                    x: 0.0,
                    y: 0.0,
                    z: 1.0,
                })),
                &(),
            );

            commands.add_child(
                "label",
                Text2DNode {
                    text: *text,
                    font_size: *text_font_size,
                    color: *text_color,
                    font: BUTTONS_FONT_PATH,
                    alignment: TextAlignment::Center,
                    linebreak_behavior: bevy::text::BreakLineOn::NoWrap,
                    text_2d_bounds: Text2dBounds::default(),
                    text_anchor: Anchor::BottomCenter,
                }
                .with_bundle(Transform::from_translation(Vec3 {
                    x: 0.0,
                    y: rect.extents.y * 0.5,
                    z: 1.0,
                })),
                &(),
            );



            commands.add_child(
                "box",
                ShaderBundle::<BoxShader> {
                    parameters: (
                        (*background_color).into(),

                        crate::rounding::OTHER_BUTTON_NORMAL.into(),
                        (rect.width() / rect.height()).into(),
                    ),

                    transform: Transform::from_scale(Vec3::ONE * node.rect.height() * 0.5),
                    ..Default::default()
                },
                &(),
            );
        });
    }
}

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

#[repr(C)]
#[derive(Debug, Reflect, Clone, Copy, TypeUuid, Default, PartialEq)]
#[uuid = "3b76cd37-5f82-4fcc-9d26-ea6f60d616e3"]
pub struct FireworksShader;

impl ParameterizedShader for FireworksShader {
    type Params = FireworksParams;
    type ParamsQuery<'a> = (&'a ShaderColor, &'a ShaderProgress);
    type ParamsBundle = (ShaderColor, ShaderProgress);
    type ResourceParams<'w> = ();

    fn get_params<'w, 'a>(
        query_item: <Self::ParamsQuery<'a> as bevy::ecs::query::WorldQuery>::Item<'w>,
        _r: &(),
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
