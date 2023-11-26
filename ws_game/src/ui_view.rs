use crate::prelude::*;
use maveric::transition::speed::ScalarSpeed;
use maveric::widgets::text2d_node::Text2DNode;
use maveric::with_bundle::{CanWithBundle, WithBundle};
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
                        text: title,
                        font_size: text_font_size,
                        color: convert_color(palette::BUTTON_TEXT_COLOR),
                        font: TITLE_FONT_PATH,
                        alignment: TextAlignment::Center,
                        linebreak_behavior: bevy::text::BreakLineOn::NoWrap,
                    }
                    .with_bundle(Transform::from_translation(
                        size.get_rect(&LayoutTextItem::PuzzleTitle, &())
                            .centre()
                            .extend(crate::z_indices::TEXT_AREA_TEXT),
                    )),
                    &(),
                );

                commands.add_child(
                    "theme",
                    Text2DNode {
                        text: "Theme",
                        font_size: text_font_size,
                        color: convert_color(palette::BUTTON_TEXT_COLOR),
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
            //let font_size = size.font_size::<LayoutWordTile>();

            commands.add_child(
                "text",
                Text2DNode {
                    text,
                    font_size: node.font_size,
                    color: convert_color(palette::BUTTON_TEXT_COLOR),
                    font: SOLUTIONS_FONT_PATH,
                    alignment: TextAlignment::Center,
                    linebreak_behavior: bevy::text::BreakLineOn::NoWrap,
                }
                .with_bundle(Transform::from_translation(text_translation)),
                &(),
            );

            let shape_translation = centre.extend(crate::z_indices::WORD_BACKGROUND);
            let shape_border_translation = centre.extend(crate::z_indices::WORD_BACKGROUND + 1.0);

            // let e = node.rect.extents * 0.5;

            // let rectangle = shapes::RoundedPolygon {
            //     points: vec![
            //         e,
            //         Vec2 {
            //             x: e.x,
            //             y: e.y * -1.0,
            //         },
            //         e * -1.0,
            //         Vec2 {
            //             x: e.x * -1.0,
            //             y: e.y,
            //         },
            //     ],
            //     radius: e.y.abs() * 0.5,
            //     closed: true,
            // };

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
                    convert_color(palette::WORD_BACKGROUND_UNSTARTED),
                    0.1,
                )
                .with_bundle(ButtonInteraction::WordButton(node.tile))
                .with_transition_to::<SmudColorLens>(*fill_color, amount_per_second.into()),
                &(),
            );

            // commands.add_child( //todo use a different shader rather than a border
            //     "shape_border",
            //     box_border_node(
            //         node.rect.extents.x.abs(),
            //         node.rect.extents.y.abs(),
            //         shape_border_translation,
            //         convert_color(palette::WORD_BORDER),
            //         0.1,
            //         0.025,
            //     ),
            //     &(),
            // );
        })
    }
}

// #[derive(PartialEq)]
// pub struct LyonShapeNode<G: Geometry + PartialEq + Send + Sync + 'static> {
//     pub shape: G,
//     pub transform: Transform,
//     pub fill: Fill,
//     pub stroke: Stroke,
// }

// impl<G: Geometry + PartialEq + Send + Sync + 'static> MavericNode for LyonShapeNode<G> {
//     type Context = ();

//     fn set_components(mut commands: SetComponentCommands<Self, Self::Context>) {
//         commands.scope(|commands| {
//             commands
//                 .map_node(|x| &x.shape)
//                 .insert_with_node(|node| (GeometryBuilder::build_as(node),))
//                 .finish()
//         });
//         commands.node_to_bundle(|x| &x.transform);
//         commands.node_to_bundle(|x| &x.fill);
//         commands.node_to_bundle(|x| &x.stroke);

//         commands.insert_static_bundle((
//             bevy::sprite::Mesh2dHandle::default(),
//             VisibilityBundle::default(),
//             GlobalTransform::default(),
//             ShapeBundle::default().material,
//         ));
//     }

//     fn set_children<R: MavericRoot>(commands: SetChildrenCommands<Self, Self::Context, R>) {
//         commands.no_children()
//     }
// }
