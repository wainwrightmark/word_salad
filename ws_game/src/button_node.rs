use bevy::prelude::*;
use bevy_prototype_lyon::{
    draw::{Fill, Stroke},
    shapes,
};
use maveric::{
    helpers::{ChildCommands, TextNode},
    node::MavericNode,
    node_context::NoContext,
    root::MavericRoot,
    widgets::text2d_node::Text2DNode,
    with_bundle::WithBundle,
};

use ws_core::{palette, LayoutRectangle};

use crate::prelude::{convert_color, ButtonInteraction, LyonShapeNode, MENU_BUTTON_FONT_PATH};

#[derive(Debug, PartialEq)]
pub struct ButtonNode2d {
    pub font_size: f32,
    pub rect: LayoutRectangle,
    pub text: &'static str,
    pub interaction: ButtonInteraction,
}

impl MavericNode for ButtonNode2d {
    type Context = NoContext;

    fn set_components(commands: maveric::prelude::SetComponentCommands<Self, Self::Context>) {
        commands
            .ignore_node()
            .ignore_context()
            .insert(SpatialBundle::default());
    }

    fn set_children<R: MavericRoot>(
        commands: maveric::prelude::SetChildrenCommands<Self, Self::Context, R>,
    ) {
        commands
            .ignore_context()
            .unordered_children_with_node(|node, commands| {
                let ButtonNode2d {
                    font_size,
                    rect,
                    text,
                    interaction,
                } = node;
                let centre = rect.centre();
                let text_translation = centre.extend(crate::z_indices::MENU_BUTTON_TEXT);

                commands.add_child(
                    "text",
                    Text2DNode {
                        text: TextNode {
                            text: *text,
                            font_size: *font_size,
                            color: convert_color(palette::MENU_BUTTON_TEXT),
                            font: MENU_BUTTON_FONT_PATH,
                            alignment: TextAlignment::Center,
                            linebreak_behavior: bevy::text::BreakLineOn::NoWrap,
                        },
                        transform: Transform::from_translation(text_translation),
                    },
                    &(),
                );

                let shape_translation = centre.extend(crate::z_indices::MENU_BUTTON_BACKGROUND);

                let e = rect.extents * 0.5;

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
                commands.add_child(
                    "shape",
                    WithBundle {
                        node: LyonShapeNode {
                            transform: Transform::from_translation(shape_translation),
                            fill: Fill::color(convert_color(palette::MENU_BUTTON_FILL)),
                            stroke: Stroke::color(convert_color(palette::MENU_BUTTON_STROKE)),
                            shape: rectangle,
                        },
                        bundle: (*interaction),
                    },
                    &(),
                );
            })
    }
}
