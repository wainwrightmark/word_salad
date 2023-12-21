use std::fmt::Debug;

use bevy::prelude::*;
use maveric::{
    helpers::ChildCommands, node::MavericNode, root::MavericRoot, widgets::text2d_node::Text2DNode,
    with_bundle::CanWithBundle,
};

use ws_core::LayoutRectangle;

use crate::prelude::{box_node1, ButtonInteraction, BUTTONS_FONT_PATH};

#[derive(Debug, PartialEq)]
pub struct ButtonNode2d<T: Into<String> + PartialEq + Debug + Send + Sync + Clone + 'static> {
    pub font_size: f32,
    pub rect: LayoutRectangle,
    pub text: T,
    pub interaction: ButtonInteraction,
    pub fill_color: Color,
    pub text_color: Color,
}

impl<T: Into<String> + PartialEq + Debug + Send + Sync + Clone + 'static> MavericNode
    for ButtonNode2d<T>
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
                let ButtonNode2d {
                    font_size,
                    rect,
                    text,
                    interaction,
                    fill_color,
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
                    box_node1(
                        rect.extents.x.abs(),
                        rect.extents.y.abs(),
                        shape_translation,
                        *fill_color,
                        crate::rounding::OTHER_BUTTON_NORMAL,
                    )
                    .with_bundle(*interaction),
                    &(),
                );
            })
    }
}
