use bevy::prelude::*;

use maveric::{
    helpers::{ChildCommands, MavericContext, NodeContext, UnorderedChildCommands},
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
            let (button_fill_color_complete, button_fill_color_incomplete, border) =
                match node.background_type {
                    BackgroundType::Congrats | BackgroundType::NonLevel => (
                        Color::NONE,
                        Color::NONE,
                        ShaderBorder::from_color(palette::MENU_BUTTON_TEXT_REGULAR.convert_color()),
                    ),
                    BackgroundType::Selfie => (
                        palette::MENU_BUTTON_COMPLETE_FILL.convert_color(),
                        palette::MENU_BUTTON_FILL.convert_color(),
                        ShaderBorder::NONE,
                    ),
                    BackgroundType::Normal => (
                        palette::MENU_BUTTON_COMPLETE_FILL.convert_color(),
                        palette::MENU_BUTTON_FILL.convert_color(),
                        ShaderBorder::NONE,
                    ),
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
                MenuState::ChooseLevelsPage => {
                    add_double_text_menu_items::<R, LevelsMenuLayoutEntity>(
                        &context.video_resource.selfie_mode(),
                        commands,
                        size,
                        1,
                        |x| {
                            x.get_text(
                                context.daily_challenge_completion.as_ref(),
                                context.sequence_completion.as_ref(),
                                context.daily_challenges.as_ref(),
                            )
                        },
                        |x| {
                            if x.is_complete(
                                &context.daily_challenge_completion,
                                &context.sequence_completion,
                                context.daily_challenges.as_ref(),
                            ) {
                                button_fill_color_complete
                            } else {
                                button_fill_color_incomplete
                            }
                        },
                        BUTTONS_FONT_PATH,
                        BUTTONS_FONT_PATH,
                        palette::MENU_BUTTON_TEXT_REGULAR.convert_color(),
                        border,
                    );
                }
                MenuState::LevelGroupPage(group) => {
                    add_double_text_menu_items::<R, LevelGroupLayoutEntity>(
                        &(context.video_resource.selfie_mode(), *group),
                        commands,
                        size,
                        2,
                        |x| x.get_text(context.sequence_completion.as_ref(), group),
                        |x| {
                            if x.is_complete(&context.sequence_completion, group) {
                                button_fill_color_complete
                            } else {
                                button_fill_color_incomplete
                            }
                        },
                        BUTTONS_FONT_PATH,
                        BUTTONS_FONT_PATH,
                        palette::MENU_BUTTON_TEXT_REGULAR.convert_color(),
                        border,
                    );
                }
                MenuState::WordSaladLevels => {
                    add_double_text_menu_items::<R, WordSaladMenuLayoutEntity>(
                        &context.video_resource.selfie_mode(),
                        commands,
                        size,
                        5,
                        |x| {
                            x.get_text(
                                context.daily_challenge_completion.as_ref(),
                                context.daily_challenges.as_ref(),
                            )
                        },
                        |x| {
                            if x.is_complete(&context.daily_challenge_completion) {
                                button_fill_color_complete
                            } else {
                                button_fill_color_incomplete
                            }
                        },
                        BUTTONS_FONT_PATH,
                        ICON_FONT_PATH,
                        palette::MENU_BUTTON_TEXT_REGULAR.convert_color(),
                        border,
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
        + LayoutStructureWithStaticText
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
        let font_size = size.font_size::<L>(&entity, &());
        let rect = size.get_rect(&entity, context);
        commands.add_child(
            (index as u16, page),
            WSButtonNode {
                font_size,
                rect,
                text: entity.text(context),
                interaction: entity.into(),
                text_color,
                fill_color,
                clicked_fill_color: BUTTON_CLICK_FILL.convert_color(),
                border,
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
    text_color: Color,
    border: ShaderBorder,
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
