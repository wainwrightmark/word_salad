use bevy::prelude::*;

use maveric::{
    helpers::{ChildCommands, MavericContext, NodeContext, UnorderedChildCommands},
    node::MavericNode,
    root::MavericRoot,
};
use strum::EnumIs;

use ws_core::{
    palette::{self, BUTTON_CLICK_FILL},
    LayoutStructure, LayoutStructureWithFont, LayoutStructureWithTextOrImage,
};
use ws_levels::level_group::LevelGroup;

use crate::{
    menu_layout::{
        main_menu_back_button::MainMenuBackButton,
        word_salad_menu_layout::WordSaladMenuLayoutEntity,
    },
    prelude::*,
};

use self::{
    level_group_layout::LevelGroupLayoutEntity, levels_menu_layout::LevelsMenuLayoutEntity,
    main_menu_layout::MainMenuLayoutEntity,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Resource, EnumIs, MavericContext)]
pub enum MenuState {
    #[default]
    Closed,
    ShowMainMenu,
    ChooseLevelsPage,
    LevelGroupPage(LevelGroup),
    WordSaladLevels,
    MainStorePage,
    HintsStorePage,
    LevelGroupStorePage,
    SettingsPage,
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
            MainStorePage => ShowMainMenu,
            HintsStorePage => MainStorePage,
            LevelGroupStorePage => MainStorePage,
            SettingsPage => ShowMainMenu,
        }
    }
}

#[derive(Debug, NodeContext)]
pub struct MenuContext {
    pub window_size: MyWindowSize,
    pub menu_state: MenuState,
    pub daily_challenge_completion: DailyChallengeCompletion,
    pub sequence_completion: SequenceCompletion,
    pub video_resource: VideoResource,
    pub daily_challenges: DailyChallenges,
}

impl<'a, 'w: 'a> From<&'a ViewContextWrapper<'w>> for MenuContextWrapper<'w> {
    fn from(value: &'a ViewContextWrapper<'w>) -> Self {
        Self {
            window_size: Res::clone(&value.window_size),
            video_resource: Res::clone(&value.video_resource),
            daily_challenges: Res::clone(&value.daily_challenges),
            menu_state: Res::clone(&value.menu_state),
            daily_challenge_completion: Res::clone(&value.daily_challenge_completion),
            sequence_completion: Res::clone(&value.sequence_completion),
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Menu {
    pub background_type: BackgroundType,
}

impl MavericNode for Menu {
    type Context = MenuContext;

    fn set_components(commands: maveric::prelude::SetComponentCommands<Self, Self::Context>) {
        commands
            .ignore_context()
            .ignore_node()
            .insert(SpatialBundle::default());
    }

    fn set_children<R: maveric::prelude::MavericRoot>(
        commands: maveric::prelude::SetChildrenCommands<Self, Self::Context, R>,
    ) {
        commands.unordered_children_with_node_and_context(|node, context, commands| {
            let border = match node.background_type {
                BackgroundType::Congrats | BackgroundType::NonLevel => {
                    ShaderBorder::from_color(palette::MENU_BUTTON_TEXT_REGULAR.convert_color())
                }

                BackgroundType::Selfie | BackgroundType::Normal => ShaderBorder::NONE,
            };

            let size = context.window_size.as_ref();
            match context.menu_state.as_ref() {
                MenuState::Closed => {}
                MenuState::ShowMainMenu => {
                    add_menu_items::<R, MainMenuLayoutEntity>(
                        &context.video_resource.selfie_mode(),
                        commands,
                        size,
                        0,
                        palette::MENU_BUTTON_FILL.convert_color(),
                        palette::MENU_BUTTON_TEXT_REGULAR.convert_color(),
                        border,
                    );
                }
                MenuState::SettingsPage => {
                    add_menu_items::<R, settings_menu_layout::SettingsLayoutEntity>(
                        &context.video_resource.selfie_mode(),
                        commands,
                        size,
                        0,
                        palette::MENU_BUTTON_FILL.convert_color(),
                        palette::MENU_BUTTON_TEXT_REGULAR.convert_color(),
                        border,
                    );
                }
                MenuState::MainStorePage => {
                    add_menu_items::<R, store_menu_layout::StoreLayoutEntity>(
                        &context.video_resource.selfie_mode(),
                        commands,
                        size,
                        0,
                        palette::MENU_BUTTON_FILL.convert_color(),
                        palette::MENU_BUTTON_TEXT_REGULAR.convert_color(),
                        border,
                    );
                }
                MenuState::HintsStorePage => {
                    add_double_text_menu_items::<R, hints_menu_layout::HintsLayoutEntity>(
                        &context.video_resource.selfie_mode(),
                        commands,
                        size,
                        0,
                        node.background_type,
                        border,
                        &context,
                    );
                }
                MenuState::LevelGroupStorePage => {
                    add_double_text_menu_items::<
                        R,
                        level_group_store_layout::LevelGroupStoreLayoutStructure,
                    >(
                        &context.video_resource.selfie_mode(),
                        commands,
                        size,
                        1,
                        node.background_type,
                        border,
                        &context,
                    );
                }
                MenuState::ChooseLevelsPage => {
                    add_double_text_menu_items::<R, LevelsMenuLayoutEntity>(
                        &context.video_resource.selfie_mode(),
                        commands,
                        size,
                        2,
                        node.background_type,
                        border,
                        &context,
                    );
                }
                MenuState::LevelGroupPage(group) => {
                    add_double_text_menu_items::<R, LevelGroupLayoutEntity>(
                        &(context.video_resource.selfie_mode(), *group),
                        commands,
                        size,
                        3,
                        node.background_type,
                        border,
                        &context,
                    );
                }
                MenuState::WordSaladLevels => {
                    add_double_text_menu_items::<R, WordSaladMenuLayoutEntity>(
                        &context.video_resource.selfie_mode(),
                        commands,
                        size,
                        4,
                        node.background_type,
                        border,
                        &context,
                    )
                }
            }

            add_menu_items::<R, MainMenuBackButton>(
                &(),
                commands,
                size,
                4,
                palette::MENU_BUTTON_DISCOURAGED_FILL.convert_color(),
                palette::MENU_BUTTON_TEXT_DISCOURAGED.convert_color(),
                border,
            );
        });
    }
}

fn add_menu_items<
    R: MavericRoot,
    L: LayoutStructureWithFont<FontContext = ()>
        + LayoutStructureWithTextOrImage
        + Into<ButtonInteraction>,
>(
    context: &<L as LayoutStructure>::Context<'_>,
    commands: &mut UnorderedChildCommands<R>,
    size: &Size,
    page: u16,
    fill_color: Color,
    text_color: Color,
    border: ShaderBorder,
) {
    for (index, entity) in L::iter_all(context).enumerate() {
        let rect = size.get_rect(&entity, context);
        match entity.text_or_image(context) {
            TextOrImage::Text { text } => {
                let font_size = size.font_size::<L>(&entity, &());

                commands.add_child(
                    (index as u16, page),
                    WSButtonNode {
                        font_size,
                        rect,
                        text,
                        interaction: entity.into(),
                        text_color,
                        fill_color,
                        clicked_fill_color: BUTTON_CLICK_FILL.convert_color(),
                        border,
                    },
                    &(),
                );
            }
            TextOrImage::Image {
                path,
                color,
                pressed_color,
                aspect_ratio,
            } => {
                commands.add_child(
                    (index as u16, page),
                    WsImageButtonNode {
                        rect,
                        image_path: path,
                        interaction: entity.into(),
                        fill_color: color.convert_color(),
                        clicked_fill_color: pressed_color.convert_color(),
                        image_aspect_ratio: aspect_ratio,
                        border,
                    },
                    &(),
                );
            }
        }
    }
}

fn add_double_text_menu_items<
    R: MavericRoot,
    L: LayoutStructureDoubleText + Into<ButtonInteraction>,
>(
    context: &<L as LayoutStructure>::Context<'_>,
    commands: &mut UnorderedChildCommands<R>,
    size: &Size,
    page: u16,
    background_type: BackgroundType,
    // fill_color_func: impl Fn(&L) -> Color,
    // text_color: Color,
    border: ShaderBorder,
    text_context: &<L as LayoutStructureDoubleText>::TextContext<'_>,
) {
    for (index, entity) in L::iter_all(context).enumerate() {
        let font_size = size.font_size::<L>(&entity, &());
        let (left_text, right_text) = entity.double_text(context, text_context);
        let left_font = entity.left_font();
        let right_font = entity.right_font();
        let text_color = entity.text_color(context, text_context).convert_color();
        let fill_color = entity
            .fill_color(background_type, context, text_context)
            .convert_color();

        let rect = size.get_rect(&entity, context);
        commands.add_child(
            (index as u16, page),
            DoubleTextButtonNode {
                font_size,
                rect,
                left_text,
                right_text,
                interaction: entity.into(),
                text_color,
                fill_color,
                left_font,
                right_font,
                clicked_fill_color: BUTTON_CLICK_FILL.convert_color(),
                border,
            },
            &(),
        );
    }
}
