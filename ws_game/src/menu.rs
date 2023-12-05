use bevy::prelude::*;

use maveric::{
    helpers::{ChildCommands, MavericContext, UnorderedChildCommands},
    node::MavericNode,
    root::MavericRoot,
};
use strum::EnumIs;

use ws_core::{LayoutStructure, LayoutStructureWithFont, LayoutStructureWithStaticText};
use ws_levels::level_group::LevelGroup;

use crate::{prelude::{
    level_group_layout::LevelGroupLayoutEntity, levels_menu_layout::LevelsMenuLayoutEntity,
    main_menu_layout::MainMenuLayoutEntity, ButtonInteraction, ButtonNode2d, SaladWindowSize, Size,
    ViewContext,
}, menu_layout::main_menu_back_button::MainMenuBackButton};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Resource, EnumIs, MavericContext)]
pub enum MenuState {
    #[default]
    Closed,
    ShowMainMenu,
    ChooseLevelsPage,
    LevelGroupPage(LevelGroup),
}

impl MenuState {
    pub fn toggle(&mut self) {
        *self = if self.is_closed() {
            MenuState::ShowMainMenu
        } else {
            MenuState::Closed
        }
    }

    pub fn close(&mut self) {
        *self = MenuState::Closed
    }

    pub fn go_back(&mut self){
        use MenuState::*;
        *self = match self{
            Closed => Closed,
            ShowMainMenu => Closed,
            ChooseLevelsPage => ShowMainMenu,
            LevelGroupPage(_) => ChooseLevelsPage,
        }
    }
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
                        add_menu_items::<R, MainMenuLayoutEntity>(&(), commands, size, 100);
                    }
                    MenuState::ChooseLevelsPage => {
                        add_menu_items_with_fn::<R, LevelsMenuLayoutEntity>(&(), commands, size, 200, |x| x.get_text(context.7.as_ref()));
                    }
                    MenuState::LevelGroupPage(group) => {
                        add_menu_items_with_fn::<R, LevelGroupLayoutEntity>(&group, commands, size, 300, |x|x.get_text(context.7.as_ref(), group));
                    }
                }


                add_menu_items::<R, MainMenuBackButton>(&(), commands, size, 400);
            });
    }
}

fn add_menu_items<
    R: MavericRoot,
    L: LayoutStructureWithFont + LayoutStructureWithStaticText + Into<ButtonInteraction>,
>(
    context: &<L as LayoutStructure>::Context,
    commands: &mut UnorderedChildCommands<R>,
    size: &Size,
    key_offset: u32,
) {

    for (index, entity) in L::iter_all(context).enumerate() {
        let font_size = size.font_size::<L>(&entity);
        let rect = size.get_rect(&entity, context);
        commands.add_child(
            (index as u32) + key_offset,
            ButtonNode2d {
                font_size,
                rect,
                text: entity.text(context),
                interaction: entity.into(),
            },
            &(),
        );
    }
}

fn add_menu_items_with_fn<
    R: MavericRoot,
    L: LayoutStructureWithFont  + Into<ButtonInteraction>,

>(
    context: &<L as LayoutStructure>::Context,
    commands: &mut UnorderedChildCommands<R>,
    size: &Size,
    key_offset: u32,
    func: impl Fn(&L)-> String
) {
    for (index, entity) in L::iter_all(context).enumerate() {
        let font_size = size.font_size::<L>(&entity);
        let text = func(&entity);

        let rect = size.get_rect(&entity, context);
        commands.add_child(
            (index as u32) + key_offset,
            ButtonNode2d {
                font_size,
                rect,
                text,
                interaction: entity.into(),
            },
            &(),
        );
    }
}
