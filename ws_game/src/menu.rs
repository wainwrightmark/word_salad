use bevy::prelude::*;
use bevy_prototype_lyon::{
    draw::{Fill, Stroke},
    shapes,
};
use maveric::{
    helpers::{ChildCommands, TextNode, UnorderedChildCommands},
    node::MavericNode,
    node_context::NoContext,
    root::MavericRoot,
    widgets::text2d_node::Text2DNode,
};
use strum::EnumIs;

use ws_core::{palette, LayoutRectangle, LayoutStructure, LayoutStructureWithFont, LayoutStructureWithStaticText};
use ws_levels::level_group::LevelGroup;

use crate::prelude::{
    convert_color, level_group_layout::LevelGroupLayout,
    levels_menu_layout::LevelsMenuLayoutEntity, main_menu_layout::MainMenuLayoutEntity,
    LyonShapeNode, SaladWindowSize, Size, ViewContext, MENU_BUTTON_FONT_PATH,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Resource, EnumIs)]
pub enum MenuState {
    #[default]
    Closed,
    ShowMainMenu,
    ChooseLevelsPage,
    LevelGroupPage(LevelGroup),
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Menu;

impl MavericNode for Menu {
    type Context = ViewContext;

    fn set_components(commands: maveric::prelude::SetComponentCommands<Self, Self::Context>) {
        commands
            .ignore_context()
            .ignore_node()
            .insert(SpatialBundle::default());
    }

    fn set_children<R: maveric::prelude::MavericRoot>(
        commands: maveric::prelude::SetChildrenCommands<Self, Self::Context, R>,
    ) {
        commands
            .ignore_node()
            .unordered_children_with_context(|context, commands| {
                let size = context.3.as_ref();
                match context.5.as_ref() {
                    MenuState::Closed => {}
                    MenuState::ShowMainMenu => {
                        add_menu_items::<R, MainMenuLayoutEntity>(&(), commands, size);
                    }
                    MenuState::ChooseLevelsPage => {
                        add_menu_items::<R, LevelsMenuLayoutEntity>(&(), commands, size);
                    }
                    MenuState::LevelGroupPage(group) => {
                        add_menu_items::<R, LevelGroupLayout>(&group, commands, size);
                    }
                }
            });
    }
}

fn add_menu_items<R: MavericRoot, L: LayoutStructureWithFont + LayoutStructureWithStaticText>(
    context: &<L as LayoutStructure>::Context,

    commands: &mut UnorderedChildCommands<R>,
    size: &Size,
) {
    let font_size = size.font_size::<L>();
    for (index, entity) in L::iter_all(context).enumerate() {
        let rect = size.get_rect(&entity, context);
        commands.add_child(index as u32, MenuButton { font_size, rect,text: entity.text(context)}, &());
    }
}

#[derive(Debug, PartialEq)]
pub struct MenuButton {
    pub font_size: f32,
    pub rect: LayoutRectangle,
    pub text: &'static str
}

impl MavericNode for MenuButton {
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
                let MenuButton { font_size, rect, text } = node;
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
                    LyonShapeNode {
                        transform: Transform::from_translation(shape_translation),
                        fill: Fill::color(convert_color(palette::MENU_BUTTON_FILL)),
                        stroke: Stroke::color(convert_color(palette::MENU_BUTTON_STROKE)),
                        shape: rectangle,
                    },
                    &(),
                );
            })
    }
}
