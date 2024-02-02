use crate::{prelude::*, purchases::Purchases, z_indices};
use bevy::{sprite::Anchor, text::Text2dBounds};

use maveric::{widgets::text2d_node::Text2DNode, with_bundle::CanWithBundle};

use rand::rngs::ThreadRng;
use std::time::Duration;
use strum::IntoEnumIterator;
use ws_core::{layout::entities::*, palette::BUTTON_CLICK_FILL};

pub const TRANSITION_WAIT_SECS: f32 = 1.0;
pub const TRANSITION_SECS: f32 = 1.0;
#[derive(Debug, Clone, PartialEq)]
pub struct CongratsView;

#[derive(Debug, NodeContext)]
pub struct CongratsContext {
    pub current_level: CurrentLevel,
    pub found_words_state: FoundWordsState,
    pub window_size: MyWindowSize,
    pub daily_challenge_completion: DailyChallengeCompletion,
    pub sequence_completion: SequenceCompletion,
    pub video_resource: VideoResource,
    pub streak: Streak,
    pub level_time: LevelTime,
}

impl<'a, 'w: 'a> From<&'a ViewContextWrapper<'w>> for CongratsContextWrapper<'w> {
    fn from(value: &'a ViewContextWrapper<'w>) -> Self {
        Self {
            current_level: Res::clone(&value.current_level),
            found_words_state: Res::clone(&value.found_words_state),
            window_size: Res::clone(&value.window_size),
            daily_challenge_completion: Res::clone(&value.daily_challenge_completion),
            sequence_completion: Res::clone(&value.sequence_completion),
            video_resource: Res::clone(&value.video_resource),
            streak: Res::clone(&value.streak),
            level_time: Res::clone(&value.level_time),
        }
    }
}

impl MavericNode for CongratsView {
    type Context = CongratsContext;

    fn set_components(mut commands: SetComponentCommands<Self, Self::Context>) {
        commands.insert_static_bundle(SpatialBundle::default());
    }

    fn on_created(
        &self,
        context: &<Self::Context as NodeContext>::Wrapper<'_>,
        _world: &World,
        entity_commands: &mut bevy::ecs::system::EntityCommands,
    ) {
        if !context.found_words_state.is_changed() || context.found_words_state.is_added() {
            return;
        }

        //SHOW FIREWORKS - only in Selfie mode
        if context.video_resource.is_selfie_mode {
            let size = &context.window_size;

            const SECONDS: f32 = 5.0;
            const NUM_FIREWORKS: usize = 25;

            entity_commands.with_children(|cb| {
                let mut rng = ThreadRng::default();
                for i in 0..NUM_FIREWORKS {
                    fireworks::create_firework(
                        cb,
                        &mut rng,
                        SECONDS,
                        size.as_ref(),
                        TILE_LINGER_SECONDS,
                        i <= 1,
                        context.video_resource.selfie_mode(),
                    );
                }
            });
        }
    }

    fn set_children<R: MavericRoot>(commands: SetChildrenCommands<Self, Self::Context, R>) {
        commands
            .ignore_node()
            .unordered_children_with_context(|context, commands| {
                let size = &context.window_size;
                let selfie_mode = context.video_resource.selfie_mode();

                let congrats_context = (selfie_mode, context.current_level.level_type());

                #[derive(Debug, Clone, Copy)]
                enum Data {
                    None,
                    JustHints,
                    TodaysChallenge { streak: usize, longest: usize },
                    Sequence { complete: usize, remaining: usize },
                }

                let data = match context.current_level.as_ref() {
                    CurrentLevel::DailyChallenge { index } => {
                        let today_index = DailyChallenges::get_today_index();
                        if today_index == *index {
                            let streak = context.streak.as_ref();
                            Data::TodaysChallenge {
                                streak: streak.current,
                                longest: streak.longest,
                            }
                        } else {
                            let complete = context
                                .daily_challenge_completion
                                .get_daily_challenges_complete();
                            let total = today_index + 1;
                            let remaining = total.saturating_sub(complete);
                            Data::Sequence {
                                complete,
                                remaining,
                            }
                        }
                    }
                    CurrentLevel::Tutorial { .. } => Data::None,
                    CurrentLevel::Fixed { sequence, .. } => {
                        let complete = context.sequence_completion.get_number_complete(sequence);
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

                let initial_scale = if context.found_words_state.is_changed() {
                    Vec3::ZERO
                } else {
                    Vec3::ONE
                };
                let transition = TransitionBuilder::default()
                    .then_wait(Duration::from_secs_f32(TRANSITION_WAIT_SECS))
                    .then_set_value(Vec3::ONE)
                    //.then_ease(Vec3::ONE, (1.0 / TRANSITION_SECS).into(), Ease::CubicOut)
                    .build();

                let stat_text_color = if selfie_mode.is_selfie_mode {
                    palette::CONGRATS_STATISTIC_TEXT_SELFIE
                } else {
                    palette::CONGRATS_STATISTIC_TEXT_NORMAL
                }
                .convert_color();

                if !context.current_level.is_tutorial() {
                    let rect = size.get_rect(&CongratsLayoutEntity::Time, &congrats_context);

                    commands.add_child(
                        "Timer",
                        TimerNode {
                            text: format_seconds(context.level_time.total_elapsed().as_secs()),
                            text_color: stat_text_color,
                            text_font_size: size.font_size(&CongratsTimer, &selfie_mode),
                        }
                        .with_bundle(Transform::from_translation(
                            rect.centre().extend(z_indices::CONGRATS_BUTTON),
                        ))
                        .with_transition::<TransformScaleLens, ()>(
                            initial_scale,
                            transition.clone(),
                            (),
                        ),
                        &(),
                    );
                }

                let stat_number_font_size = size.font_size(&StatisticNumber, &selfie_mode);
                let stat_text_font_size = size.font_size(&StatisticLabel, &selfie_mode);

                for (index, statistic) in CongratsStatistic::iter().enumerate() {
                    let data = match (statistic, data) {
                        (_, Data::None)
                        | (CongratsStatistic::Left, Data::JustHints)
                        | (CongratsStatistic::Right, Data::JustHints) => None,
                        (CongratsStatistic::Left, _)
                        | (CongratsStatistic::Middle, Data::JustHints) => {
                            Some((context.found_words_state.hints_used, "Hints"))
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
                    let rect = size.get_rect(
                        &CongratsLayoutEntity::Statistic(statistic),
                        &congrats_context,
                    );

                    commands.add_child(
                        (0u16, index as u16),
                        StatisticNode {
                            rect,
                            number,
                            text: label,
                            text_color: stat_text_color,
                            number_font_size: stat_number_font_size,
                            text_font_size: stat_text_font_size,
                        }
                        .with_bundle(Transform::from_translation(
                            rect.centre().extend(z_indices::CONGRATS_BUTTON),
                        ))
                        .with_transition::<TransformScaleLens, ()>(
                            initial_scale,
                            transition.clone(),
                            (),
                        ),
                        &(),
                    );
                }

                let button_count = CongratsLayoutEntity::get_button_count(&congrats_context);

                let button_text_color = if selfie_mode.is_selfie_mode {
                    palette::CONGRATS_BUTTON_TEXT_SELFIE
                } else {
                    palette::CONGRATS_BUTTON_TEXT_NORMAL
                }
                .convert_color();

                let (button_fill_color, border) = if selfie_mode.is_selfie_mode {
                    (
                        palette::CONGRATS_BUTTON_FILL_SELFIE.convert_color(),
                        ShaderBorder::NONE,
                    )
                } else {
                    (Color::NONE, ShaderBorder::from_color(button_text_color))
                };

                for (index, button) in CongratsButton::iter().enumerate().take(button_count) {
                    let text = match button {
                        CongratsButton::Next => match context.current_level.level_type() {
                            ws_core::level_type::LevelType::Tutorial => "Next".to_string(),
                            _ => {
                                let next_level = context.current_level.get_next_level(
                                    &context.daily_challenge_completion,
                                    &context.sequence_completion,
                                    &Purchases::default(), //don't actually worry about purchases here :)
                                );

                                match next_level {
                                    CurrentLevel::Tutorial { .. } => "Next".to_string(),
                                    CurrentLevel::Fixed { .. } => "Next".to_string(),
                                    CurrentLevel::DailyChallenge { index: next_index } => {
                                        if let CurrentLevel::DailyChallenge {
                                            index: current_index,
                                        } = context.current_level.as_ref()
                                        {
                                            if next_index > *current_index
                                                && next_index == DailyChallenges::get_today_index()
                                            {
                                                "Today's Puzzle".to_string()
                                            } else {
                                                format!("Play #{}", next_index + 1)
                                            }
                                        } else {
                                            format!("Play #{}", next_index + 1)
                                        }
                                    }
                                    CurrentLevel::Custom { .. } => "Next".to_string(),
                                    CurrentLevel::NonLevel(NonLevel::LevelSequenceAllFinished(
                                        _,
                                    )) => "Finish".to_string(),
                                    CurrentLevel::NonLevel(_) => "Next".to_string(),
                                }
                            }
                        },
                        CongratsButton::MoreLevels => "More Puzzles".to_string(),
                        #[cfg(target_arch = "wasm32")]
                        CongratsButton::Share => "Share".to_string(),
                    };

                    commands.add_child(
                        (1u16, index as u16),
                        WSButtonNode {
                            text,
                            font_size: size.font_size(&CongratsLayoutEntity::Button(button), &()),
                            rect: size
                                .get_rect(&CongratsLayoutEntity::Button(button), &congrats_context),
                            interaction: ButtonInteraction::Congrats(button),
                            text_color: button_text_color,
                            fill_color: button_fill_color,
                            clicked_fill_color: BUTTON_CLICK_FILL.convert_color(),
                            border,
                        }
                        .with_transition::<TransformScaleLens, ()>(
                            initial_scale,
                            transition.clone(),
                            (),
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
                    y: rect.extents.y * 0.4,
                    z: 1.0,
                })),
                &(),
            );
        });
    }
}

#[derive(Debug, Clone, PartialEq)]
struct TimerNode {
    text: String,
    text_color: Color,
    text_font_size: f32,
}

impl MavericNode for TimerNode {
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
            let TimerNode {
                text,
                text_color,
                text_font_size,
            } = node;

            commands.add_child(
                "label",
                Text2DNode {
                    text: text.clone(),
                    font_size: *text_font_size,
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
        });
    }
}
