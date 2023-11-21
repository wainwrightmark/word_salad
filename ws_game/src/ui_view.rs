use crate::prelude::*;
use itertools::Itertools;
use maveric::transition::speed::LinearSpeed;
use maveric::{
    transition::speed::ScalarSpeed, widgets::text2d_node::Text2DNode, with_bundle::WithBundle,
};
use ws_core::layout::entities::*;
use ws_core::prelude::*;

#[derive(Debug, Clone, PartialEq)]
pub struct UI;

//pub const BUTTON_FONT_SIZE: f32 = 22.0;
pub const BUTTON_TEXT_COLOR: Color = Color::rgb(0.1, 0.1, 0.1);
pub const TEXT_BUTTON_WIDTH: f32 = 140.;
pub const TEXT_BUTTON_HEIGHT: f32 = 30.;
pub const UI_BORDER_WIDTH: Val = Val::Px(3.0);

impl MavericNode for UI {
    type Context = NC5<ChosenState, CurrentLevel, FoundWordsState, Size, LevelTime>;

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
                let size = &context.3;
                let top_bar_font_size = size.font_size::<LayoutTopBarButton>();
                let text_font_size = size.font_size::<LayoutTextItem>();
                commands.add_child(
                    "Burger",
                    Text2DNode {
                        text: TextNode {
                            text: "\u{f0c9}",
                            font_size: top_bar_font_size,
                            color: BUTTON_TEXT_COLOR,
                            font: MENU_BUTTON_FONT_PATH,
                            alignment: TextAlignment::Center,
                            linebreak_behavior: bevy::text::BreakLineOn::NoWrap,
                        },

                        transform: Transform::from_translation(
                            size.get_rect(&LayoutTopBarButton::MenuBurgerButton, &())
                                .centre()
                                .extend(crate::z_indices::TOP_BAR_BUTTON),
                        ),
                    },
                    &(),
                );

                let time_text = match context.4.as_ref() {
                    LevelTime::Started(..) => "00:00".to_string(),
                    LevelTime::Finished { total_seconds } => format_seconds(*total_seconds),
                };

                let time_translation = if context.2.is_level_complete() {
                    size.get_rect(&CongratsLayoutEntity::LevelTime, &())
                        .centre()
                        .extend(crate::z_indices::TOP_BAR_BUTTON)
                } else {
                    size.get_rect(&LayoutTopBarButton::TimeCounter, &())
                        .centre()
                        .extend(crate::z_indices::TOP_BAR_BUTTON)
                };

                commands.add_child(
                    "TimeCounter",
                    WithBundle {
                        node: Text2DNode {
                            text: TextNode {
                                text: time_text,
                                font_size: top_bar_font_size,
                                color: BUTTON_TEXT_COLOR,
                                font: MENU_BUTTON_FONT_PATH,
                                alignment: TextAlignment::Center,
                                linebreak_behavior: bevy::text::BreakLineOn::NoWrap,
                            },

                            transform: Transform::from_translation(
                                size.get_rect(&LayoutTopBarButton::TimeCounter, &())
                                    .centre()
                                    .extend(crate::z_indices::TOP_BAR_BUTTON),
                            ),
                        },
                        bundle: TimeCounterMarker,
                    }
                    .with_transition_to::<TransformTranslationLens>(
                        //TODO improve this animation
                        time_translation,
                        LinearSpeed {
                            units_per_second: 100.0,
                        },
                    ),
                    &(),
                );

                commands.add_child(
                    "hints",
                    Text2DNode {
                        text: TextNode {
                            text: context.2.hints_used.to_string(),
                            font_size: top_bar_font_size,
                            color: BUTTON_TEXT_COLOR,
                            font: BUTTONS_FONT_PATH,
                            alignment: TextAlignment::Center,
                            linebreak_behavior: bevy::text::BreakLineOn::NoWrap,
                        },
                        transform: Transform::from_translation(
                            size.get_rect(&LayoutTopBarButton::HintCounter, &())
                                .centre()
                                .extend(crate::z_indices::TOP_BAR_BUTTON),
                        ),
                    },
                    &(),
                );

                let title = context.1.level().name.trim().to_string();

                commands.add_child(
                    "title",
                    Text2DNode {
                        text: TextNode {
                            text: title,
                            font_size: text_font_size,
                            color: BUTTON_TEXT_COLOR,
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
                            color: BUTTON_TEXT_COLOR,
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
    type Context = NC5<ChosenState, CurrentLevel, FoundWordsState, Size, LevelTime>;

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
                    let rect = context.3.get_rect(&tile, words);
                    commands.add_child(
                        index as u32,
                        WordNode {
                            word: word.clone(),
                            tile,
                            completion,
                            rect,
                        },
                        &context.3,
                    );
                }
            });
    }
}

#[derive(Debug, PartialEq)]
pub struct WordNode {
    pub tile: LayoutWordTile,
    pub word: Word,
    pub completion: Completion,
    pub rect: LayoutRectangle,
}

impl MavericNode for WordNode {
    type Context = Size;

    fn set_components(mut commands: SetComponentCommands<Self, Self::Context>) {
        commands.scope(|commands| {
            commands
                .ignore_node()
                .ignore_context()
                .insert(SpatialBundle::default())
                .finish()
        });

        commands
            .map_args(|x| x.completion.color())
            .ignore_context()
            .animate_on_node::<BackgroundColorLens>(Some(ScalarSpeed::new(1.0)));
    }

    fn set_children<R: MavericRoot>(commands: SetChildrenCommands<Self, Self::Context, R>) {
        commands.unordered_children_with_node_and_context(|node, size, commands| {
            let text = match node.completion {
                Completion::Unstarted => node.word.characters.len().to_string(), //todo show word structure
                Completion::ManualHinted(hints) | Completion::AutoHinted(hints) => {
                    let hinted_characters = node.word.text.chars().take(hints.get());
                    let question_marks = std::iter::repeat('?'); //todo show word structure

                    std::iter::Iterator::chain(hinted_characters, question_marks)
                        .take(node.word.characters.len())
                        .join("")
                }

                Completion::Complete => node.word.text.to_string(),
            };

            let centre = node.rect.centre();

            let text_translation = centre.extend(crate::z_indices::WORD_TEXT);
            let font_size = size.font_size::<LayoutWordTile>();

            commands.add_child(
                "text",
                Text2DNode {
                    text: TextNode {
                        text,
                        font_size,
                        color: BUTTON_TEXT_COLOR,
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
                radius: size.tile_size() * 0.15,
                closed: true,
            };

            let fill_color = node.completion.color();

            commands.add_child(
                "shape",
                LyonShapeNode {
                    transform: Transform::from_translation(shape_translation),
                    fill: Fill::color(*fill_color),
                    stroke: Stroke::color(Color::DARK_GRAY),
                    shape: rectangle,
                },
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
