use crate::{animated_solutions, prelude::*};
use bevy_param_shaders::*;
use itertools::Either;
use maveric::widgets::text2d_node::Text2DNode;
use maveric::with_bundle::CanWithBundle;
use ws_core::layout::entities::*;
use ws_core::prelude::*;

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
                            selfie_mode
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
    pub selfie_mode: bool
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

            let progress = match node.completion {
                Completion::Unstarted => 0.0,
                Completion::ManualHinted(hints) => {
                    (hints.get() + 1) as f32 / (node.word.graphemes.len() + 2) as f32
                }
                Completion::Complete => 1.0,
            };


            let centre = node.rect.centre();

            let text_translation = centre.extend(crate::z_indices::WORD_TEXT);

            let color = if node.selfie_mode{
                palette::WORD_TEXT_SELFIE
            }else{
                palette::WORD_TEXT_NORMAL
            }.convert_color();

            commands.add_child(
                "text",
                Text2DNode {
                    text,
                    font_size: node.font_size,
                    color,
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

            let second_color = match node.completion{
                Completion::Unstarted => palette::WORD_BACKGROUND_UNSTARTED.convert_color(),
                Completion::ManualHinted(_) => palette::WORD_BACKGROUND_MANUAL_HINT.convert_color(),
                Completion::Complete => palette::WORD_BACKGROUND_COMPLETE.convert_color(),
            };

            commands.add_child(
                "shape_fill",
                (
                    ShaderBundle {
                        shape: ShaderShape::<HorizontalGradientBoxShader>::default(),
                        parameters: (
                            palette::WORD_BACKGROUND_UNSTARTED.convert_color().into(),
                            crate::rounding::WORD_BUTTON_NORMAL.into(),
                            (node.rect.extents.y.abs() / node.rect.extents.x.abs()).into(),
                            ShaderProgress { progress },
                            second_color.into(),
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
