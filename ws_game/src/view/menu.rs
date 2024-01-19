use bevy::prelude::*;

use maveric::{
    helpers::{ChildCommands, MavericContext, UnorderedChildCommands},
    node::MavericNode,
    root::MavericRoot,
};
use strum::EnumIs;

use ws_core::{
    palette::{self, BUTTON_CLICK_FILL},
    LayoutStructure, LayoutStructureWithFont, LayoutStructureWithStaticText,
};
use ws_levels::level_group::LevelGroup;

use crate::{
    menu_layout::{
        main_menu_back_button::MainMenuBackButton,
        word_salad_menu_layout::WordSaladMenuLayoutEntity,
    },
    prelude::{
        level_group_layout::LevelGroupLayoutEntity, levels_menu_layout::LevelsMenuLayoutEntity,
        main_menu_layout::MainMenuLayoutEntity, ButtonInteraction, ConvertColor,
        DoubleTextButtonNode, SaladWindowSize, Size, ViewContext, WSButtonNode, BUTTONS_FONT_PATH,
        ICON_FONT_PATH,
    },
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Resource, EnumIs, MavericContext)]
pub enum MenuState {
    #[default]
    Closed,
    ShowMainMenu,
    ChooseLevelsPage,
    LevelGroupPage(LevelGroup),
    WordSaladLevels,
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

    pub fn go_back(&mut self) {
        use MenuState::*;
        *self = match self {
            Closed => Closed,
            ShowMainMenu => Closed,
            ChooseLevelsPage => ShowMainMenu,
            LevelGroupPage(_) => ChooseLevelsPage,
            WordSaladLevels => ChooseLevelsPage,
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
                        add_menu_items::<R, MainMenuLayoutEntity>(
                            &(),
                            commands,
                            size,
                            0,
                            palette::MENU_BUTTON_FILL.convert_color(),
                        );
                    }
                    MenuState::ChooseLevelsPage => {
                        add_double_text_menu_items::<R, LevelsMenuLayoutEntity>(
                            &(),
                            commands,
                            size,
                            1,
                            |x| x.get_text(context.7.as_ref(), context.9.as_ref()),
                            |x| get_variable_fill(x.is_complete(&context.7, context.9.as_ref())),
                            BUTTONS_FONT_PATH,
                            BUTTONS_FONT_PATH,
                        );
                    }
                    MenuState::LevelGroupPage(group) => {
                        add_double_text_menu_items::<R, LevelGroupLayoutEntity>(
                            group,
                            commands,
                            size,
                            2,
                            |x| x.get_text(context.7.as_ref(), group),
                            |x| get_variable_fill(x.is_complete(&context.7, group)),
                            BUTTONS_FONT_PATH,
                            BUTTONS_FONT_PATH,
                        );
                    }
                    MenuState::WordSaladLevels => {
                        add_double_text_menu_items::<R, WordSaladMenuLayoutEntity>(
                            &(),
                            commands,
                            size,
                            5,
                            |x| x.get_text(context.7.as_ref(), context.9.as_ref()),
                            |x| get_variable_fill(x.is_complete(&context.7)),
                            BUTTONS_FONT_PATH,
                            ICON_FONT_PATH,
                        )
                    }
                }

                add_menu_items::<R, MainMenuBackButton>(
                    &(),
                    commands,
                    size,
                    4,
                    palette::MENU_BUTTON_DISCOURAGED_FILL.convert_color(),
                );
            });
    }
}

fn get_variable_fill(is_complete: bool) -> Color {
    if is_complete {
        palette::MENU_BUTTON_COMPLETE_FILL.convert_color()
    } else {
        palette::MENU_BUTTON_FILL.convert_color()
    }
}
fn add_menu_items<
    R: MavericRoot,
    L: LayoutStructureWithFont<FontContext = ()>
        + LayoutStructureWithStaticText
        + Into<ButtonInteraction>,
>(
    context: &<L as LayoutStructure>::Context<'_>,
    commands: &mut UnorderedChildCommands<R>,
    size: &Size,
    page: u16,
    fill_color: Color,
) {
    for (index, entity) in L::iter_all(context).enumerate() {
        let font_size = size.font_size::<L>(&entity, &());
        let rect = size.get_rect(&entity, context);
        commands.add_child(
            (index as u16, page),
            WSButtonNode {
                font_size,
                rect,
                text: entity.text(context),
                interaction: entity.into(),
                text_color: palette::MENU_BUTTON_TEXT.convert_color(),
                fill_color,
                clicked_fill_color: BUTTON_CLICK_FILL.convert_color(),
            },
            &(),
        );
    }
}

fn add_double_text_menu_items<
    R: MavericRoot,
    L: LayoutStructureWithFont<FontContext = ()> + LayoutStructure + Into<ButtonInteraction>,
>(
    context: &<L as LayoutStructure>::Context<'_>,
    commands: &mut UnorderedChildCommands<R>,
    size: &Size,
    page: u16,
    text_func: impl Fn(&L) -> (String, String),
    fill_color_func: impl Fn(&L) -> Color,
    left_font: &'static str,
    right_font: &'static str,
) {
    for (index, entity) in L::iter_all(context).enumerate() {
        let font_size = size.font_size::<L>(&entity, &());
        let (left_text, right_text) = text_func(&entity);
        let fill_color = fill_color_func(&entity);

        let rect = size.get_rect(&entity, context);
        commands.add_child(
            (index as u16, page),
            DoubleTextButtonNode {
                font_size,
                rect,
                left_text,
                right_text,
                interaction: entity.into(),
                text_color: palette::MENU_BUTTON_TEXT.convert_color(),
                fill_color,
                left_font,
                right_font,
                clicked_fill_color: BUTTON_CLICK_FILL.convert_color(),
            },
            &(),
        );
    }
}
