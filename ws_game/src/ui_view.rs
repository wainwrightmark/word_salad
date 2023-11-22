use crate::prelude::*;
use maveric::transition::speed::{LinearSpeed, ScalarSpeed};
use maveric::{widgets::text2d_node::Text2DNode, with_bundle::WithBundle};
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
                let title = context.1.level().name.trim().to_string();
                let size = &context.3;
                let text_font_size = size.font_size::<LayoutTextItem>();
                commands.add_child(
                    "title",
                    Text2DNode {
                        text: TextNode {
                            text: title,
                            font_size: text_font_size,
                            color: convert_color(palette::BUTTON_TEXT_COLOR),
                            font: TITLE_FONT_PATH,
                            alignment: TextAlignment::Center,
                            linebreak_behavior: bevy::text::BreakLineOn::NoWrap,
                        },
                        transform: Transform::from_translation(
                            size.get_rect(&LayoutTextItem::PuzzleTitle, &())
                                .centre()
                                .extend(crate::z_indices::TEXT_AREA_TEXT),
                        ),
                    },
                    &(),
                );

                commands.add_child(
                    "theme",
                    Text2DNode {
                        text: TextNode {
                            text: "Theme",
                            font_size: text_font_size,
                            color: convert_color(palette::BUTTON_TEXT_COLOR),
                            font: TITLE_FONT_PATH,
                            alignment: TextAlignment::Center,
                            linebreak_behavior: bevy::text::BreakLineOn::NoWrap,
                        },
                        transform: Transform::from_translation(
                            size.get_rect(&LayoutTextItem::PuzzleTheme, &())
                                .centre()
                                .extend(crate::z_indices::TEXT_AREA_TEXT),
                        ),
                    },
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
                let font_size = context.3.font_size::<LayoutWordTile>();
                for (index, word) in words.iter().enumerate() {
                    let completion = context.2.get_completion(index);
                    let tile = LayoutWordTile(index);
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
    type Context = NoContext;

    fn set_components(commands: SetComponentCommands<Self, Self::Context>) {
        commands
            .ignore_node()
            .ignore_context()
            .insert(SpatialBundle::default())
            .finish()

        // commands
        //     .map_args(|x| x.completion.color())
        //     .ignore_context()
        //     .animate_on_node::<BackgroundColorLens>(Some(ScalarSpeed::new(1.0)));
    }

    fn set_children<R: MavericRoot>(commands: SetChildrenCommands<Self, Self::Context, R>) {
        commands.unordered_children_with_node(|node, commands| {
            let text = match node.completion {
                Completion::Unstarted => node.word.hidden_text.clone(),
                Completion::ManualHinted(hints) | Completion::AutoHinted(hints) => {
                    node.word.hinted_text(hints)
                }

                Completion::Complete => node.word.text.to_string(),
            };

            let centre = node.rect.centre();

            let text_translation = centre.extend(crate::z_indices::WORD_TEXT);
            //let font_size = size.font_size::<LayoutWordTile>();

            commands.add_child(
                "text",
                Text2DNode {
                    text: TextNode {
                        text,
                        font_size: node.font_size,
                        color: convert_color(palette::BUTTON_TEXT_COLOR),
                        font: SOLUTIONS_FONT_PATH,
                        alignment: TextAlignment::Center,
                        linebreak_behavior: bevy::text::BreakLineOn::NoWrap,
                    },
                    transform: Transform::from_translation(text_translation),
                },
                &(),
            );

            let shape_translation = centre.extend(crate::z_indices::WORD_BACKGROUND);

            let e = node.rect.extents * 0.5;

            let rectangle = shapes::RoundedPolygon {
                points: vec![
                    e,
                    Vec2 {
                        x: e.x,
                        y: e.y * -1.0,
                    },
                    e * -1.0,
                    Vec2 {
                        x: e.x * -1.0,
                        y: e.y,
                    },
                ],
                radius: e.y.abs() * 0.5,
                closed: true,
            };

            let fill_color = node.completion.color();
            let amount_per_second = node
                .completion
                .is_unstarted()
                .then_some(10.0)
                .unwrap_or(1.0);
            commands.add_child(
                "shape",
                LyonShapeNode {
                    transform: Transform::from_translation(shape_translation),
                    fill: Fill::color(convert_color(palette::WORD_BACKGROUND_UNSTARTED)),
                    stroke: Stroke::color(convert_color(palette::WORD_BORDER)),
                    shape: rectangle,
                }
                .with_transition_to::<FillColorLens>(
                    *fill_color,
                    ScalarSpeed { amount_per_second },
                ),
                &(),
            );
        })
    }
}

#[derive(PartialEq)]
pub struct LyonShapeNode<G: Geometry + PartialEq + Send + Sync + 'static> {
    pub shape: G,
    pub transform: Transform,
    pub fill: Fill,
    pub stroke: Stroke,
}

impl<G: Geometry + PartialEq + Send + Sync + 'static> MavericNode for LyonShapeNode<G> {
    type Context = NoContext;

    fn set_components(mut commands: SetComponentCommands<Self, Self::Context>) {
        commands.scope(|commands| {
            commands
                .map_args(|x| &x.shape)
                .insert_with_node(|node| {
                    (
                        GeometryBuilder::build_as(node),
                        bevy::sprite::Mesh2dHandle::default(),
                        ShapeBundle::default().material,
                        VisibilityBundle::default(),
                        GlobalTransform::default(),
                    )
                })
                .finish()
        });

        commands.scope(|c| c.map_args(|x| &x.fill).insert_bundle().finish());
        commands.scope(|c| c.map_args(|x| &x.stroke).insert_bundle().finish());
        commands.scope(|c| c.map_args(|x| &x.transform).insert_bundle().finish());
    }

    fn set_children<R: MavericRoot>(commands: SetChildrenCommands<Self, Self::Context, R>) {
        commands.no_children()
    }
}
