use bevy::prelude::*;

use maveric::{
    helpers::{ChildCommands, MavericContext, NodeContext},
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
    prelude::*, shapes, z_indices,
};

use self::{
    level_group_layout::LevelGroupLayoutEntity, level_group_store_layout::LevelGroupStoreLayoutStructure, levels_menu_layout::LevelsMenuLayoutEntity, main_menu_layout::MainMenuLayoutEntity
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
    pub purchases: Purchases,
    pub hint_state: HintState,
    pub current_level: CurrentLevel,
    pub found_words_state: FoundWordsState,
    pub prices: Prices
}

#[derive(Debug, PartialEq, Clone, Copy, MavericRoot)]
pub struct MenuRoot;

impl MavericRootChildren for MenuRoot {
    type Context = MenuContext;

    fn set_children(
        context: &<Self::Context as NodeContext>::Wrapper<'_>,
        commands: &mut impl ChildCommands,
    ) {
        if context.menu_state.is_closed() {
            return;
        }

        let size = &context.window_size;
        let grey_color = palette::GREEN_DARK.convert_color();
        let opacity = 0.95;
        commands.add_child(
            "grey out",
            shapes::basic_box_node1(
                size.scaled_width.max(size.scaled_height),
                size.scaled_width.max(size.scaled_height),
                Vec3::Z * z_indices::GREY_OUT,
                grey_color.with_a(opacity),
                0.0,
            )
            .with_transition_in_out::<ShaderColorLens>(
                grey_color.with_a(0.0),
                grey_color.with_a(opacity),
                grey_color.with_a(0.0),
                core::time::Duration::from_millis(500),
                core::time::Duration::from_millis(500),
                Some(Ease::CubicOut),
                Some(Ease::CubicOut),
            ),
            &(),
        );

        let background_type = background_type_from_resources(
            &context.video_resource,
            &context.current_level,
            &context.found_words_state,
        );

        let border = match background_type {
            BackgroundType::Congrats | BackgroundType::NonLevel => {
                ShaderBorder::from_color(palette::MENU_BUTTON_TEXT_REGULAR.convert_color())
            }

            BackgroundType::Selfie | BackgroundType::Normal => ShaderBorder::NONE,
        };

        let size = context.window_size.as_ref();
        match context.menu_state.as_ref() {
            MenuState::Closed => {}
            MenuState::ShowMainMenu => {
                add_menu_items::<MainMenuLayoutEntity>(
                    &(context.video_resource.selfie_mode(), ()),
                    commands,
                    size,
                    0,
                    palette::MENU_BUTTON_FILL.convert_color(),
                    palette::MENU_BUTTON_TEXT_REGULAR.convert_color(),
                    border,
                );
            }
            MenuState::SettingsPage => {
                add_menu_items::<settings_menu_layout::SettingsLayoutEntity>(
                    &(context.video_resource.selfie_mode(), ()),
                    commands,
                    size,
                    1,
                    palette::MENU_BUTTON_FILL.convert_color(),
                    palette::MENU_BUTTON_TEXT_REGULAR.convert_color(),
                    border,
                );
            }
            MenuState::MainStorePage => {
                add_double_text_menu_items::<store_menu_layout::StoreLayoutStructure>(
                    &(context.video_resource.selfie_mode(), ()),
                    commands,
                    size,
                    2,
                    background_type,
                    border,
                    context,
                );
            }
            MenuState::HintsStorePage => {
                add_double_text_menu_items::<hints_menu_layout::HintsLayoutEntity>(
                    &(context.video_resource.selfie_mode(), ()),
                    commands,
                    size,
                    3,
                    background_type,
                    border,
                    context,
                );
            }
            MenuState::LevelGroupStorePage => {
                add_double_text_menu_items::<LevelGroupStoreLayoutStructure>(
                    &(context.video_resource.selfie_mode(), ()),
                    commands,
                    size,
                    4,
                    background_type,
                    border,
                    context,
                );
            }
            MenuState::ChooseLevelsPage => {
                add_double_text_menu_items::<LevelsMenuLayoutEntity>(
                    &(context.video_resource.selfie_mode(), ()),
                    commands,
                    size,
                    5,
                    background_type,
                    border,
                    context,
                );
            }
            MenuState::LevelGroupPage(group) => {
                add_double_text_menu_items::<LevelGroupLayoutEntity>(
                    &(context.video_resource.selfie_mode(), *group),
                    commands,
                    size,
                    6,
                    background_type,
                    border,
                    context,
                );
            }
            MenuState::WordSaladLevels => add_double_text_menu_items::<WordSaladMenuLayoutEntity>(
                &(context.video_resource.selfie_mode(), ()),
                commands,
                size,
                7,
                background_type,
                border,
                context,
            ),
        }

        add_menu_items::<MainMenuBackButton>(
            &(),
            commands,
            size,
            8,
            palette::MENU_BUTTON_DISCOURAGED_FILL.convert_color(),
            palette::MENU_BUTTON_TEXT_DISCOURAGED.convert_color(),
            border,
        );
    }
}

fn add_menu_items<
    L: LayoutStructureWithFont<FontContext = ()>
        + LayoutStructureWithTextOrImage
        + Into<ButtonInteraction>,
>(
    context: &<L as LayoutStructure>::Context<'_>,
    commands: &mut impl ChildCommands,
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

fn add_double_text_menu_items<L: LayoutStructureDoubleTextButton + Into<ButtonInteraction>>(
    context: &<L as LayoutStructure>::Context<'_>,
    commands: &mut impl ChildCommands,
    size: &Size,
    page: u16,
    background_type: BackgroundType,
    border: ShaderBorder,
    text_context: &<L as LayoutStructureDoubleTextButton>::TextContext<'_>,
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

        let interaction: ButtonInteraction = if entity.is_disabled(context, text_context) {
            ButtonInteraction::None
        } else {
            entity.into()
        };
        commands.add_child(
            (index as u16, page),
            DoubleTextButtonNode {
                font_size,
                rect,
                left_text,
                right_text,
                interaction,
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
