use crate::prelude::*;

use maveric::widgets::text2d_node::Text2DNode;
use maveric::with_bundle::CanWithBundle;
use ws_core::layout::entities::*;
use ws_core::prelude::*;

#[derive(Debug, Clone, PartialEq)]
pub struct UI;

impl MavericNode for UI {
    type Context = ViewContext;

    fn set_components(commands: SetComponentCommands<Self, Self::Context>) {
        commands
            .ignore_context()
            .ignore_node()
            .insert(SpatialBundle::default())
            .finish()
    }

    fn set_children<R: MavericRoot>(commands: SetChildrenCommands<Self, Self::Context, R>) {
        commands
            .ignore_node()
            .unordered_children_with_context(|context, commands| {
                let theme = context.1.level().name.trim().to_string();
                let size = &context.3;

                let time_text = match context.4.as_ref() {
                    LevelTime::Started(..) => "00:00".to_string(),
                    LevelTime::Finished { total_seconds } => format_seconds(*total_seconds),
                };

                commands.add_child(
                    "timer",
                    Text2DNode {
                        text: time_text,
                        font_size: size.font_size::<LayoutTextItem>(&LayoutTextItem::Timer),
                        color: palette::BUTTON_TEXT_COLOR.convert_color(),
                        font: TITLE_FONT_PATH,
                        alignment: TextAlignment::Center,
                        linebreak_behavior: bevy::text::BreakLineOn::NoWrap,
                    }
                    .with_bundle((Transform::from_translation(
                        size.get_rect(&LayoutTextItem::Timer, &())
                            .centre()
                            .extend(crate::z_indices::TEXT_AREA_TEXT),
                    ), TimeCounterMarker)),
                    &(),
                );

                commands.add_child(
                    "theme",
                    Text2DNode {
                        text: theme,
                        font_size: size.font_size::<LayoutTextItem>(&LayoutTextItem::PuzzleTheme),
                        color: palette::BUTTON_TEXT_COLOR.convert_color(),
                        font: TITLE_FONT_PATH,
                        alignment: TextAlignment::Center,
                        linebreak_behavior: bevy::text::BreakLineOn::NoWrap,
                    }
                    .with_bundle(Transform::from_translation(
                        size.get_rect(&LayoutTextItem::PuzzleTheme, &())
                            .centre()
                            .extend(crate::z_indices::TEXT_AREA_TEXT),
                    )),
                    &(),
                );

                commands.add_child("words", WordsNode, context);
            });
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
                let words = &context.1.level().words;

                for (index, word) in words.iter().enumerate() {

                    let completion = context.2.get_completion(index);
                    let tile = LayoutWordTile(index);
                    let font_size = context.3.font_size::<LayoutWordTile>(&tile);
                    let rect = context.3.get_rect(&tile, words);
                    commands.add_child(
                        index as u32,
                        WordNode {
                            word: word.clone(),
                            tile,
                            completion,
                            rect,
                            font_size,
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
                Completion::Unstarted => node.word.hidden_text.clone(),
                Completion::ManualHinted(hints) => node.word.hinted_text(hints),

                Completion::Complete => node.word.text.to_string(),
            };

            let centre = node.rect.centre();

            let text_translation = centre.extend(crate::z_indices::WORD_TEXT);

            commands.add_child(
                "text",
                Text2DNode {
                    text,
                    font_size: node.font_size,
                    color: palette::BUTTON_TEXT_COLOR.convert_color(),
                    font: SOLUTIONS_FONT_PATH,
                    alignment: TextAlignment::Center,
                    linebreak_behavior: bevy::text::BreakLineOn::NoWrap,
                }
                .with_bundle(Transform::from_translation(text_translation)),
                &(),
            );

            let shape_translation = centre.extend(crate::z_indices::WORD_BACKGROUND);
            let _shape_border_translation = centre.extend(crate::z_indices::WORD_BACKGROUND + 1.0);

            let fill_color = node.completion.color();
            let amount_per_second = node
                .completion
                .is_unstarted()
                .then_some(10.0)
                .unwrap_or(1.0);

            commands.add_child(
                "shape_fill",
                box_node(
                    node.rect.extents.x.abs(),
                    node.rect.extents.y.abs(),
                    shape_translation,
                    palette::WORD_BACKGROUND_UNSTARTED.convert_color(),
                    0.1,
                )
                .with_bundle(ButtonInteraction::WordButton(node.tile))
                .with_transition_to::<SmudColorLens>(*fill_color, amount_per_second.into()),
                &(),
            );
        })
    }
}
