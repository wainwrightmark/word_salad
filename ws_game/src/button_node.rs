use std::fmt::Debug;

use bevy::prelude::*;
use maveric::{
    helpers::{ChildCommands, SpriteNode},
    node::MavericNode,
    root::MavericRoot,
    widgets::text2d_node::Text2DNode,
    with_bundle::CanWithBundle,
};

use ws_core::LayoutRectangle;

use crate::{
    prelude::{button_box_node, ButtonInteraction, BUTTONS_FONT_PATH},
    shapes::ShaderBorder,
};

#[derive(Debug, PartialEq)]
pub struct WsImageButtonNode {
    pub image_path: &'static str,
    pub rect: LayoutRectangle,
    pub interaction: ButtonInteraction,
    pub fill_color: Color,

    pub clicked_fill_color: Color,
    pub image_aspect_ratio: f32,
    pub border: ShaderBorder,
}

impl MavericNode for WsImageButtonNode {
    type Context = ();

    fn set_components(mut commands: maveric::prelude::SetComponentCommands<Self, Self::Context>) {
        commands.insert_static_bundle(SpatialBundle::default());
    }

    fn set_children<R: MavericRoot>(
        commands: maveric::prelude::SetChildrenCommands<Self, Self::Context, R>,
    ) {
        commands
            .ignore_context()
            .unordered_children_with_node(|node, commands| {
                let WsImageButtonNode {
                    image_path,
                    rect,
                    interaction,
                    image_aspect_ratio,
                    fill_color,
                    clicked_fill_color,
                    border,
                } = node;
                let centre = rect.centre();
                let ratio = 1.0 - (border.border * 2.0);
                let height = rect.height() * ratio;
                let width = image_aspect_ratio * height;

                let text_translation = centre.extend(crate::z_indices::MENU_BUTTON_TEXT);

                commands.add_child(
                    "text",
                    SpriteNode {
                        texture_path: image_path,
                        sprite: Sprite {
                            color: Color::WHITE,
                            flip_x: false,
                            flip_y: false,
                            rect: None,
                            anchor: Default::default(),
                            custom_size: Some(Vec2 {
                                x: width,
                                y: height,
                            }),
                        },
                    }
                    .with_bundle(Transform::from_translation(text_translation)),
                    &(),
                );

                let shape_translation = centre.extend(crate::z_indices::MENU_BUTTON_BACKGROUND);
                commands.add_child(
                    "shape_fill",
                    button_box_node(
                        rect.extents.x.abs(),
                        rect.extents.y.abs(),
                        shape_translation,
                        *fill_color,
                        *clicked_fill_color,
                        crate::rounding::OTHER_BUTTON_NORMAL,
                        *border,
                        *interaction,
                    ),
                    &(),
                );
            })
    }
}

#[derive(Debug, PartialEq)]
pub struct WSButtonNode<T: Into<String> + PartialEq + Debug + Send + Sync + Clone + 'static> {
    pub font_size: f32,
    pub rect: LayoutRectangle,
    pub text: T,
    pub interaction: ButtonInteraction,
    pub fill_color: Color,

    pub clicked_fill_color: Color,
    pub border: ShaderBorder,
    pub text_color: Color,
}

impl<T: Into<String> + PartialEq + Debug + Send + Sync + Clone + 'static> MavericNode
    for WSButtonNode<T>
{
    type Context = ();

    fn set_components(mut commands: maveric::prelude::SetComponentCommands<Self, Self::Context>) {
        commands.insert_static_bundle(SpatialBundle::default());
    }

    fn set_children<R: MavericRoot>(
        commands: maveric::prelude::SetChildrenCommands<Self, Self::Context, R>,
    ) {
        commands
            .ignore_context()
            .unordered_children_with_node(|node, commands| {
                let WSButtonNode {
                    font_size,
                    rect,
                    text,
                    interaction,
                    fill_color,
                    clicked_fill_color,
                    border,
                    text_color,
                } = node;
                let centre = rect.centre();
                let text_translation = centre.extend(crate::z_indices::MENU_BUTTON_TEXT);

                commands.add_child(
                    "text",
                    Text2DNode {
                        text: text.clone(),
                        font_size: *font_size,
                        color: *text_color,
                        font: BUTTONS_FONT_PATH,
                        alignment: TextAlignment::Center,
                        linebreak_behavior: bevy::text::BreakLineOn::NoWrap,
                        text_2d_bounds: Default::default(),
                        text_anchor: Default::default(),
                    }
                    .with_bundle(Transform::from_translation(text_translation)),
                    &(),
                );

                let shape_translation = centre.extend(crate::z_indices::MENU_BUTTON_BACKGROUND);
                commands.add_child(
                    "shape_fill",
                    button_box_node(
                        rect.extents.x.abs(),
                        rect.extents.y.abs(),
                        shape_translation,
                        *fill_color,
                        *clicked_fill_color,
                        crate::rounding::OTHER_BUTTON_NORMAL,
                        *border,
                        *interaction,
                    ),
                    &(),
                );
            })
    }
}

#[derive(Debug, PartialEq)]
pub struct DoubleTextButtonNode<T: Into<String> + PartialEq + Debug + Send + Sync + Clone + 'static>
{
    pub font_size: f32,
    pub rect: LayoutRectangle,
    pub left_text: T,
    pub right_text: T,
    pub interaction: ButtonInteraction,
    pub fill_color: Color,
    pub clicked_fill_color: Color,
    pub border: ShaderBorder,
    pub text_color: Color,
    pub left_font: &'static str,
    pub right_font: &'static str,
}

impl<T: Into<String> + PartialEq + Debug + Send + Sync + Clone + 'static> MavericNode
    for DoubleTextButtonNode<T>
{
    type Context = ();

    fn set_components(mut commands: maveric::prelude::SetComponentCommands<Self, Self::Context>) {
        commands.insert_static_bundle(SpatialBundle::default());
    }

    fn set_children<R: MavericRoot>(
        commands: maveric::prelude::SetChildrenCommands<Self, Self::Context, R>,
    ) {
        commands
            .ignore_context()
            .unordered_children_with_node(|node, commands| {
                let DoubleTextButtonNode {
                    font_size,
                    rect,
                    left_text,
                    right_text,
                    interaction,
                    fill_color,
                    clicked_fill_color,

                    text_color,
                    left_font,
                    right_font,
                    border,
                } = node;
                let centre = rect.centre();
                let left_text_translation = (rect.centre_left()
                    + Vec2 {
                        x: rect.extents.x * 0.05,
                        y: 0.0,
                    })
                .extend(crate::z_indices::MENU_BUTTON_TEXT);
                let right_text_translation = (rect.centre_right()
                    - Vec2 {
                        x: rect.extents.x * 0.05,
                        y: 0.0,
                    })
                .extend(crate::z_indices::MENU_BUTTON_TEXT);

                commands.add_child(
                    "left_text",
                    Text2DNode {
                        text: left_text.clone(),
                        font_size: *font_size,
                        color: *text_color,
                        font: left_font,
                        alignment: TextAlignment::Left,
                        linebreak_behavior: bevy::text::BreakLineOn::NoWrap,
                        text_2d_bounds: Default::default(),
                        text_anchor: bevy::sprite::Anchor::CenterLeft,
                    }
                    .with_bundle(Transform::from_translation(left_text_translation)),
                    &(),
                );

                commands.add_child(
                    "right_text",
                    Text2DNode {
                        text: right_text.clone(),
                        font_size: *font_size,
                        color: *text_color,
                        font: right_font,
                        alignment: TextAlignment::Right,
                        linebreak_behavior: bevy::text::BreakLineOn::NoWrap,
                        text_2d_bounds: Default::default(),
                        text_anchor: bevy::sprite::Anchor::CenterRight,
                    }
                    .with_bundle(Transform::from_translation(right_text_translation)),
                    &(),
                );

                let shape_translation = centre.extend(crate::z_indices::MENU_BUTTON_BACKGROUND);
                commands.add_child(
                    "shape_fill",
                    button_box_node(
                        rect.extents.x.abs(),
                        rect.extents.y.abs(),
                        shape_translation,
                        *fill_color,
                        *clicked_fill_color,
                        crate::rounding::OTHER_BUTTON_NORMAL,
                        *border,
                        *interaction,
                    )
                    .with_bundle(*interaction),
                    &(),
                );
            })
    }
}
