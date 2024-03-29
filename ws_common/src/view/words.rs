use std::time::Duration;

use crate::{animated_solutions, prelude::*};
use bevy_param_shaders::frame::Frame;
use bevy_param_shaders::parameterized_shader::{
    ExtractToShader, FragmentImport, ParameterizedShader, SDFColorCall,
};
use bevy_param_shaders::*;
use itertools::Either;
use maveric::transition::speed::calculate_speed;
use maveric::widgets::text2d_node::Text2DNode;
use maveric::with_bundle::CanWithBundle;
use ws_core::layout::entities::*;
use ws_core::prelude::*;

pub struct WordsPlugin;

impl Plugin for WordsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ExtractToShaderPlugin::<WordButtonBoxShader>::default());
    }
}

#[derive(Debug, NodeContext)]
pub struct WordsContext {
    pub current_level: CurrentLevel,
    pub found_words_state: FoundWordsState,
    pub window_size: MyWindowSize,
    pub video_resource: VideoResource,
    pub daily_challenges: DailyChallenges,
    pub menu_state: MenuState,
    pub insets: InsetsResource,
}

impl<'a, 'w: 'a> From<&'a ViewContextWrapper<'w>> for WordsContextWrapper<'w> {
    fn from(value: &'a ViewContextWrapper<'w>) -> Self {
        Self {
            current_level: Res::clone(&value.current_level),
            found_words_state: Res::clone(&value.found_words_state),
            window_size: Res::clone(&value.window_size),
            video_resource: Res::clone(&value.video_resource),
            daily_challenges: Res::clone(&value.daily_challenges),
            menu_state: Res::clone(&value.menu_state),
            insets: Res::clone(&value.insets),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct WordsNode;

impl MavericNode for WordsNode {
    type Context = WordsContext;

    fn should_recreate(
        &self,
        _previous: &Self,
        context: &<Self::Context as NodeContext>::Wrapper<'_>,
    ) -> bool {
        context.current_level.is_changed()
    }

    fn set_components(commands: SetComponentCommands<Self, Self::Context>) {
        commands
            .ignore_context()
            .ignore_node()
            .insert(SpatialBundle::default());
    }

    fn set_children<R: MavericRoot>(commands: SetChildrenCommands<Self, Self::Context, R>) {
        commands
            .ignore_node()
            .unordered_children_with_context(|context, commands| {
                let Either::Left(level) = context.current_level.level(&context.daily_challenges)
                else {
                    return;
                };
                let words = level.words.as_slice();

                let selfie_mode = context.video_resource.selfie_mode();

                for (index, word) in words.iter().enumerate() {
                    let completion = context.found_words_state.get_completion(index);
                    let tile = LayoutWordTile(index);
                    let font_size = context.window_size.font_size::<LayoutWordTile>(&tile, &());
                    let rect = context
                        .window_size
                        .get_rect(&tile, &(words, (selfie_mode, context.insets.0)));
                    let level_indices: (u16, u16) = match context.current_level.as_ref() {
                        CurrentLevel::Fixed { level_index, .. } => (0, *level_index as u16),
                        CurrentLevel::Custom { .. } => (1, 0),
                        CurrentLevel::Tutorial { index } => (2, *index as u16),
                        CurrentLevel::DailyChallenge { index } => (3, *index as u16),
                        CurrentLevel::NonLevel(..) => (4, 0),
                    };

                    commands.add_child(
                        (index as u16, level_indices.0, level_indices.1),
                        WordNode {
                            word: word.clone(),
                            tile,
                            completion,
                            rect,
                            font_size,
                            selfie_mode,
                            menu_closed: context.menu_state.is_closed(),
                        },
                        &(),
                    );
                }
            });
    }
}

#[derive(Debug, PartialEq)]
pub struct WordNode {
    pub tile: LayoutWordTile,
    pub word: DisplayWord,
    pub completion: Completion,
    pub rect: LayoutRectangle,
    pub font_size: f32,
    pub selfie_mode: SelfieMode,
    pub menu_closed: bool,
}

impl MavericNode for WordNode {
    type Context = ();

    fn set_components(commands: SetComponentCommands<Self, Self::Context>) {
        commands
            .ignore_node()
            .ignore_context()
            .insert(SpatialBundle::default())
            .finish()
    }

    fn set_children<R: MavericRoot>(commands: SetChildrenCommands<Self, Self::Context, R>) {
        commands.unordered(|args, commands| {
            let node = args.node;

            let completion = node.completion;

            let progress = match completion {
                Completion::Unstarted => 0.0,
                Completion::ManualHinted(_) => 0.0,
                Completion::Complete { .. } => 1.0,
            };

            let centre = node.rect.centre();

            if node.menu_closed {
                let text = match node.completion {
                    Completion::Unstarted => node.word.hidden_text.to_string(),
                    Completion::ManualHinted(hints) => node.word.hinted_text(hints).to_uppercase(),

                    Completion::Complete { .. } => node.word.text.to_uppercase().to_string(),
                };

                let text_translation = centre.extend(crate::z_indices::WORD_TEXT);

                let text_color = match completion {
                    Completion::Unstarted => palette::WORD_TEXT_NUMBER,
                    Completion::ManualHinted(_) | Completion::Complete { .. } => {
                        palette::WORD_TEXT_LETTERS
                    }
                }
                .convert_color();

                commands.add_child(
                    "text",
                    Text2DNode {
                        text,
                        font_size: node.font_size,
                        color: text_color,
                        font: SOLUTIONS_FONT_PATH,
                        justify_text: JustifyText::Center,
                        linebreak_behavior: bevy::text::BreakLineOn::NoWrap,
                        text_2d_bounds: Default::default(),
                        text_anchor: Default::default(),
                    }
                    .with_bundle(Transform::from_translation(text_translation))
                    .with_transition_to::<TextColorLens<0>>(
                        text_color,
                        calculate_speed(
                            &palette::WORD_TEXT_NUMBER.convert_color(),
                            &palette::WORD_TEXT_LETTERS.convert_color(),
                            Duration::from_secs_f32(animated_solutions::TOTAL_SECONDS),
                        ),
                        None,
                    ),
                    &(),
                );
            }

            let shape_translation = centre.extend(crate::z_indices::WORD_BACKGROUND);
            let _shape_border_translation = centre.extend(crate::z_indices::WORD_BACKGROUND + 1.0);

            let transition_speed = match completion {
                Completion::Unstarted => f32::MAX,
                Completion::ManualHinted(_) => f32::MAX,
                Completion::Complete { .. } => 1.0 / animated_solutions::TOTAL_SECONDS,
            };

            let height = node.rect.extents.y.abs();
            let width = node.rect.extents.x.abs();
            let scale = height.max(width);

            commands.add_child(
                "shape_fill",
                (
                    ShaderBundle::<WordButtonBoxShader> {
                        parameters: (
                            WordButtonCompletion {
                                completion,
                                tile: node.tile,
                                previous_completion: args.previous.map(|x| x.completion),
                                menu_open: !node.menu_closed,
                            },
                            ShaderProportions {
                                width: width / scale,
                                height: height / scale,
                            },
                            ShaderProgress { progress },
                        ),
                        transform: Transform {
                            translation: shape_translation,
                            scale: Vec3::ONE * scale * 0.5,
                            ..Default::default()
                        },
                        ..default()
                    },
                    ButtonInteraction::WordButton(node.tile),
                )
                    .with_transition_to::<ProgressLens>(
                        progress,
                        transition_speed.into(),
                        None,
                    ),
                &(),
            );
        })
    }
}

#[derive(Debug, PartialEq, Clone, Component, Default)]
pub struct WordButtonCompletion {
    pub completion: Completion,
    pub previous_completion: Option<Completion>,
    pub tile: LayoutWordTile,
    pub menu_open: bool,
}

pub const WORD_BUTTON_HOLD_SECONDS: f32 = 0.3;

#[repr(C)]
#[derive(Debug, Reflect, Clone, Copy, Default, PartialEq)]
pub struct WordButtonBoxShader;

impl ExtractToShader for WordButtonBoxShader {
    type Shader = Self;
    type ParamsQuery<'a> = (
        &'a WordButtonCompletion,
        &'a ShaderProportions,
        &'a ShaderProgress,
    );
    type ParamsBundle = (WordButtonCompletion, ShaderProportions, ShaderProgress);
    type ResourceParams<'w> = (Res<'w, PressedButton>, Res<'w, Time>);

    fn get_params(
        query_item: <Self::ParamsQuery<'_> as bevy::ecs::query::WorldQuery>::Item<'_>,
        resource: &<Self::ResourceParams<'_> as bevy::ecs::system::SystemParam>::Item<'_, '_>,
    ) -> <Self::Shader as ParameterizedShader>::Params {
        let (
            WordButtonCompletion {
                completion,
                tile,
                previous_completion,
                menu_open,
            },
            ShaderProportions { height, width },
            ShaderProgress { progress },
        ) = query_item;

        let (pressed_button, time) = resource;

        if let Some(pressed_duration) = match pressed_button.as_ref() {
            PressedButton::None | PressedButton::NoInteractionPressed { .. } => None,
            PressedButton::Pressed {
                interaction,
                start_elapsed,
                ..
            } => {
                if interaction == &ButtonInteraction::WordButton(*tile) {
                    Some(time.elapsed().saturating_sub(*start_elapsed))
                } else {
                    None
                }
            }
            PressedButton::PressedAfterActivated { .. } => None,
        } {
            let color = if *menu_open {
                palette::WORD_BACKGROUND_UNSTARTED.convert_color()
            } else {
                match completion {
                    Completion::Unstarted => palette::WORD_BACKGROUND_UNSTARTED.convert_color(),
                    Completion::ManualHinted(_) => {
                        palette::WORD_BACKGROUND_MANUAL_HINT.convert_color()
                    }
                    Completion::Complete { .. } => {
                        palette::WORD_BACKGROUND_COMPLETE.convert_color()
                    }
                }
            };

            let color2 = if *menu_open {
                palette::WORD_BACKGROUND_UNSTARTED.convert_color()
            } else {
                match completion {
                    Completion::Unstarted => palette::WORD_BACKGROUND_PROGRESS.convert_color(),
                    Completion::ManualHinted(_) => {
                        palette::WORD_BACKGROUND_MANUAL_HINT2.convert_color()
                    }
                    Completion::Complete { .. } => {
                        palette::WORD_BACKGROUND_COMPLETE.convert_color()
                    }
                }
            };

            let progress =
                (pressed_duration.as_secs_f32() / WORD_BUTTON_HOLD_SECONDS).clamp(0.0, 1.0);

            HorizontalGradientBoxShaderParams {
                color: color.into(),
                height: *height,
                width: *width,
                progress,
                color2: color2.into(),
                rounding: crate::rounding::WORD_BUTTON_NORMAL,
            }
        } else {
            let color = if *menu_open {
                palette::WORD_BACKGROUND_UNSTARTED.convert_color()
            } else {
                match completion {
                    Completion::Unstarted => palette::WORD_BACKGROUND_UNSTARTED.convert_color(),
                    Completion::ManualHinted(_) => {
                        palette::WORD_BACKGROUND_MANUAL_HINT.convert_color()
                    }
                    Completion::Complete { .. } => {
                        if previous_completion.is_some_and(|x| x.is_manual_hinted()) {
                            palette::WORD_BACKGROUND_MANUAL_HINT.convert_color()
                        } else {
                            palette::WORD_BACKGROUND_UNSTARTED.convert_color()
                        }
                    }
                }
            };

            let color2 = if *menu_open {
                palette::WORD_BACKGROUND_UNSTARTED.convert_color()
            } else {
                match completion {
                    Completion::Unstarted => palette::WORD_BACKGROUND_UNSTARTED.convert_color(),
                    Completion::ManualHinted(_) => {
                        palette::WORD_BACKGROUND_MANUAL_HINT.convert_color()
                    }
                    Completion::Complete { .. } => {
                        palette::WORD_BACKGROUND_COMPLETE.convert_color()
                    }
                }
            };

            HorizontalGradientBoxShaderParams {
                color: color.into(),
                height: *height,
                width: *width,
                progress: *progress,
                color2: color2.into(),
                rounding: crate::rounding::WORD_BUTTON_NORMAL,
            }
        }
    }
}

impl ParameterizedShader for WordButtonBoxShader {
    type Params = HorizontalGradientBoxShaderParams;

    fn fragment_body() -> impl Into<String> {
        SDFColorCall {
            sdf: "shaders::box::sdf(in.pos, in.width, in.height, in.rounding)",
            fill_color:
                "fill::horizontal_gradient::fill(d, in.color, in.pos, in.progress, in.color2)",
        }
    }

    fn imports() -> impl Iterator<Item = FragmentImport> {
        [HORIZONTAL_GRADIENT_FILL, BOX_SDF_IMPORT].into_iter()
    }

    const FRAME: Frame = Frame::square(1.0);

    const UUID: u128 = 0x266b0619b9134ccebe867470ef0b129b;
}
