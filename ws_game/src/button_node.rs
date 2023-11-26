use bevy::prelude::*;
use maveric::{
    helpers::ChildCommands, node::MavericNode, root::MavericRoot, widgets::text2d_node::Text2DNode,
    with_bundle::CanWithBundle,
};

use ws_core::{palette, LayoutRectangle};

use crate::prelude::{box_node, convert_color, ButtonInteraction, MENU_BUTTON_FONT_PATH};

#[derive(Debug, PartialEq)]
pub struct ButtonNode2d {
    pub font_size: f32,
    pub rect: LayoutRectangle,
    pub text: &'static str,
    pub interaction: ButtonInteraction,
}

impl MavericNode for ButtonNode2d {
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
                } = node;
                let centre = rect.centre();
                let text_translation = centre.extend(crate::z_indices::MENU_BUTTON_TEXT);

                commands.add_child(
                    "text",
                    Text2DNode {
                        text: *text,
                        font_size: *font_size,
                        color: convert_color(palette::MENU_BUTTON_TEXT),
                        font: MENU_BUTTON_FONT_PATH,
                        alignment: TextAlignment::Center,
                        linebreak_behavior: bevy::text::BreakLineOn::NoWrap,
                    }
                    .with_bundle(Transform::from_translation(text_translation)),
                    &(),
                );

                let shape_translation = centre.extend(crate::z_indices::MENU_BUTTON_BACKGROUND);
                //let shape_border_translation = centre.extend(crate::z_indices::MENU_BUTTON_BACKGROUND + 1.0);
                commands.add_child(
                    "shape_fill",
                    box_node(
                        rect.extents.x.abs(),
                        rect.extents.y.abs(),
                        shape_translation,
                        convert_color(palette::MENU_BUTTON_FILL),
                        0.1,
                    )
                    .with_bundle(*interaction),
                    &(),
                );

                // commands.add_child(
                //     "shape_border",
                //     box_border_node(
                //         rect.extents.x.abs(),
                //         rect.extents.y.abs(),
                //         shape_border_translation,
                //         convert_color(palette::MENU_BUTTON_STROKE),
                //         0.1,
                //         0.02,
                //     )
                //     .with_bundle(*interaction),
                //     &(),
                // );
            })
    }
}
