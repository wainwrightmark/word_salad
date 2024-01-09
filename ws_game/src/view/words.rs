use crate::{animated_solutions, prelude::*};
use bevy::reflect::TypeUuid;
use bevy_param_shaders::parameterized_shader::{FragmentImport, ParameterizedShader, SDFColorCall};
use bevy_param_shaders::*;
use itertools::Either;
use maveric::widgets::text2d_node::Text2DNode;
use maveric::with_bundle::CanWithBundle;
use ws_core::layout::entities::*;
use ws_core::palette::WORD_BACKGROUND_PROGRESS;
use ws_core::prelude::*;

pub struct WordsPlugin;

impl Plugin for WordsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ParamShaderPlugin::<WordButtonBoxShader>::default());
    }
}

#[derive(Debug, PartialEq)]
pub struct WordsNode;

impl MavericNode for WordsNode {
    type Context = ViewContext;

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
                let Either::Left(level) = context.1.level(&context.9) else {
                    return;
                };
                let words = &level.words;

                for (index, word) in words.iter().enumerate() {
                    let completion = context.2.get_completion(index);
                    let tile = LayoutWordTile(index);
                    let font_size = context.3.font_size::<LayoutWordTile>(&tile, &());
                    let rect = context.3.get_rect(&tile, words);
                    let level_indices: (u16, u16) = match context.1.as_ref() {
                        CurrentLevel::Fixed { level_index, .. } => (0, *level_index as u16),
                        CurrentLevel::Custom { .. } => (1, 0),
                        CurrentLevel::Tutorial { index } => (2, *index as u16),
                        CurrentLevel::DailyChallenge { index } => (3, *index as u16),
                        CurrentLevel::NonLevel(..) => (4, 0),
                    };
                    let selfie_mode = context.8.is_selfie_mode;

                    commands.add_child(
                        (index as u16, level_indices.0, level_indices.1),
                        WordNode {
                            word: word.clone(),
                            tile,
                            completion,
                            rect,
                            font_size,
                            selfie_mode,
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
    pub selfie_mode: bool,
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
        commands.unordered_children_with_node(|node, commands| {
            let text = match node.completion {
                Completion::Unstarted => node.word.hidden_text.to_string(),
                Completion::ManualHinted(hints) => node.word.hinted_text(hints).to_uppercase(),

                Completion::Complete => node.word.text.to_uppercase().to_string(),
            };

            let completion = node.completion;

            let progress = match completion {
                Completion::Unstarted => 0.0,
                Completion::ManualHinted(hints) => {
                    (hints.get() + 1) as f32 / (node.word.graphemes.len() + 2) as f32
                }
                Completion::Complete => 1.0,
            };

            let centre = node.rect.centre();

            let text_translation = centre.extend(crate::z_indices::WORD_TEXT);

            let text_color = if node.selfie_mode {
                palette::WORD_TEXT_SELFIE
            } else {
                palette::WORD_TEXT_NORMAL
            }
            .convert_color();

            commands.add_child(
                "text",
                Text2DNode {
                    text,
                    font_size: node.font_size,
                    color: text_color,
                    font: SOLUTIONS_FONT_PATH,
                    alignment: TextAlignment::Center,
                    linebreak_behavior: bevy::text::BreakLineOn::NoWrap,
                    text_2d_bounds: Default::default(),
                    text_anchor: Default::default(),
                }
                .with_bundle(Transform::from_translation(text_translation)),
                &(),
            );

            let shape_translation = centre.extend(crate::z_indices::WORD_BACKGROUND);
            let _shape_border_translation = centre.extend(crate::z_indices::WORD_BACKGROUND + 1.0);

            commands.add_child(
                "shape_fill",
                (
                    ShaderBundle {
                        shape: ShaderShape::<WordButtonBoxShader>::default(),
                        parameters: (
                            WordButtonCompletion { completion, tile: node.tile },
                            (node.rect.extents.y.abs() / node.rect.extents.x.abs()).into(),
                            ShaderProgress { progress },
                        ),
                        transform: Transform {
                            translation: shape_translation,
                            scale: Vec3::ONE * node.rect.extents.x.abs() * 0.5,
                            ..Default::default()
                        },
                        ..default()
                    },
                    ButtonInteraction::WordButton(node.tile),
                )
                    .with_transition_to::<ProgressLens>(
                        progress,
                        (1.0 / animated_solutions::TOTAL_SECONDS).into(),
                    ),
                &(),
            );
        })
    }
}

#[derive(Debug, PartialEq, Clone, Component, Default)]
pub struct WordButtonCompletion {
    pub completion: Completion,
    pub tile: LayoutWordTile,
}

pub const WORD_BUTTON_HOLD_SECONDS:f32 = 1.0;

#[repr(C)]
#[derive(Debug, Reflect, Clone, Copy, TypeUuid, Default, PartialEq)]
#[uuid = "266b0619-b913-4cce-be86-7470ef0b129b"]
pub struct WordButtonBoxShader;

impl ParameterizedShader for WordButtonBoxShader {
    type Params = HorizontalGradientBoxShaderParams;
    type ParamsQuery<'a> = (
        &'a WordButtonCompletion,
        &'a ShaderAspectRatio,
        &'a ShaderProgress,
    );
    type ParamsBundle = (WordButtonCompletion, ShaderAspectRatio, ShaderProgress);
    type ResourceParams<'w> = Res<'w, PressedButton>;

    fn fragment_body() -> impl Into<String> {
        SDFColorCall {
            sdf: "shaders::box::sdf(in.pos, in.height, in.rounding)",
            fill_color:
                "fill::horizontal_gradient::fill(d, in.color, in.pos, in.progress, in.color2)",
        }
    }

    fn imports() -> impl Iterator<Item = FragmentImport> {
        [HORIZONTAL_GRADIENT_FILL, BOX_SDF_IMPORT].into_iter()
    }

    fn get_params<'w, 'a>(
        query_item: <Self::ParamsQuery<'a> as bevy::ecs::query::WorldQuery>::Item<'w>,
        pressed_button: &Res<PressedButton>,
    ) -> Self::Params {
        let (
            WordButtonCompletion { completion, tile },
            ShaderAspectRatio { height },
            ShaderProgress { progress },
        ) = query_item;

        if let Some(pressed_duration) = match pressed_button.as_ref() {
            PressedButton::None => None,
            PressedButton::Pressed {
                interaction,
                duration,
            } => {
                if interaction == &ButtonInteraction::WordButton(*tile) {
                    Some(duration)
                } else {
                    None
                }
            }
            PressedButton::PressedAfterActivated { .. } => None,
        } {
            let color = palette::WORD_BACKGROUND_UNSTARTED.convert_color().into();

            let color2 = WORD_BACKGROUND_PROGRESS.convert_color();

            let progress = (pressed_duration.as_secs_f32() / WORD_BUTTON_HOLD_SECONDS).clamp(0.0, 1.0);

            HorizontalGradientBoxShaderParams {
                color,
                height: *height,
                progress,
                color2: color2.into(),
                rounding: crate::rounding::WORD_BUTTON_NORMAL.into(),
            }
        } else {
            let color = palette::WORD_BACKGROUND_UNSTARTED.convert_color().into();

            let color2 = match completion {
                Completion::Unstarted => palette::WORD_BACKGROUND_UNSTARTED.convert_color(),
                Completion::ManualHinted(_) => palette::WORD_BACKGROUND_MANUAL_HINT.convert_color(),
                Completion::Complete => palette::WORD_BACKGROUND_COMPLETE.convert_color(),
            };

            HorizontalGradientBoxShaderParams {
                color,
                height: *height,
                progress: *progress,
                color2: color2.into(),
                rounding: crate::rounding::WORD_BUTTON_NORMAL.into(),
            }
        }
    }

    const FRAME: Frame = Frame::square(1.0);
}
