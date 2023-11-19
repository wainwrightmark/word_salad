use bevy::prelude::*;
use nice_bevy_utils::async_event_writer::AsyncEventWriter;
use maveric::transition::prelude::*;
use maveric::{impl_maveric_root, prelude::*};

use std::string::ToString;
use std::time::Duration;
use strum::{Display, EnumIs};

use crate::constants::{level_count, level_name, CurrentLevel, VideoResource, VideoEvent, MENU_BUTTON_FONT_PATH};
use crate::state::{ChosenState, FoundWordsState};

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MenuState>()
            .add_systems(Update, button_system);

        app.register_transition::<StyleLeftLens>();
        app.register_transition::<TransformScaleLens>();
        app.register_transition::<BackgroundColorLens>();

        app.register_maveric::<MenuRoot>();
    }
}

#[derive(Clone, Copy, Debug, Display, PartialEq, Eq, Component)]
pub enum ButtonAction {
    OpenMenu,
    Resume,
    ChooseLevel,
    GotoLevel { level: u32 },

    NextLevelsPage,
    PreviousLevelsPage,
    Hint,
    ResetLevel,

    Video,

    None,
}

impl ButtonAction {
    pub fn main_buttons() -> &'static [Self] {
        use ButtonAction::*;
        &[Resume,
        ChooseLevel,
        ResetLevel,
        #[cfg(target_arch = "wasm32")]
        Video

        ]
    }

    pub fn icon(&self) -> String {
        use ButtonAction::*;
        match self {
            OpenMenu => "\u{f0c9}".to_string(),    // "Menu",
            Resume => "\u{e817}".to_string(),      // "Menu",
            ChooseLevel => "\u{e812}".to_string(), // "\u{e812};".to_string(),
            GotoLevel { level } => level.to_string(),
            PreviousLevelsPage => "\u{e81b}".to_string(),
            NextLevelsPage => "\u{e81a}".to_string(),

            None => "".to_string(),
            Hint => "H".to_string(),
            ResetLevel => "R".to_string(),
            Video => "V".to_string(),
        }
    }

    pub fn text(&self) -> String {
        use ButtonAction::*;
        match self {
            OpenMenu => "Menu".to_string(),
            Resume => "Resume".to_string(),
            ChooseLevel => "Choose Level".to_string(),
            GotoLevel { level } => level_name(*level),
            NextLevelsPage => "Next Levels".to_string(),
            PreviousLevelsPage => "Previous Levels".to_string(),
            ResetLevel => "Reset Level".to_string(),
            Hint => "Hint".to_string(),
            Video => "Video".to_string(),
            None => "".to_string(),
        }
    }
}

fn button_system(
    mut interaction_query: Query<
        (&Interaction, &ButtonAction),
        (Changed<Interaction>, With<Button>),
    >,
    mut state: ResMut<MenuState>,

    mut current_level: ResMut<CurrentLevel>,
    mut found_words: ResMut<FoundWordsState>,
    mut chosen_state: ResMut<ChosenState>,
    video_state: Res<VideoResource>,
    video_events: AsyncEventWriter<VideoEvent>
) {
    for (interaction, action) in &mut interaction_query {
        if *interaction != Interaction::Pressed {
            continue;
        }

        match action {
            ButtonAction::OpenMenu => *state = MenuState::ShowMainMenu,
            ButtonAction::ChooseLevel => *state = MenuState::ShowLevelsPage(0),
            ButtonAction::NextLevelsPage => {
                match state.as_ref() {
                    MenuState::ShowLevelsPage(x) => *state = MenuState::ShowLevelsPage(x + 1),
                    _ => {}
                };
            }
            ButtonAction::PreviousLevelsPage => {
                match state.as_ref() {
                    MenuState::ShowLevelsPage(x) => {
                        *state = MenuState::ShowLevelsPage(x.saturating_sub(1))
                    }
                    _ => {}
                };
            }
            ButtonAction::None => {}

            ButtonAction::Video => {
                video_state.toggle_video_streaming(video_events.clone());
                *state = MenuState::Closed;
            }

            ButtonAction::Resume => {
                *state = MenuState::Closed;
            }
            ButtonAction::GotoLevel { level } => {
                *current_level = CurrentLevel::Fixed {
                    level_index: *level as usize,
                };

                *found_words = FoundWordsState::default();
                *chosen_state = ChosenState::default();

                *state = MenuState::Closed;
            }
            ButtonAction::ResetLevel => {
                current_level.set_changed();
                *found_words = FoundWordsState::default();
                *chosen_state = ChosenState::default();

                *state = MenuState::Closed;
            }
            ButtonAction::Hint => {
                found_words.try_hint(current_level.as_ref());
            } //_ =>
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Resource, EnumIs)]
pub enum MenuState {
    #[default]
    Closed,
    ShowMainMenu,
    ShowLevelsPage(u32),
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct MenuRoot;

impl_maveric_root!(MenuRoot);

impl MavericRootChildren for MenuRoot {
    type Context = NC2<MenuState, AssetServer>;

    fn set_children(
        context: &<Self::Context as NodeContext>::Wrapper<'_>,
        commands: &mut impl ChildCommands,
    ) {
        let transition_duration: Duration = Duration::from_secs_f32(0.5);

        fn get_carousel_child(page: u32) -> Option<MainOrLevelMenu> {
            Some(if let Some(page) = page.checked_sub(1) {
                MainOrLevelMenu::Level(page)
            } else {
                MainOrLevelMenu::Main
            })
        }

        let carousel = match context.0.as_ref() {
            MenuState::Closed => {
                //commands.add_child("open_icon", menu_button_node(), &context.1);
                return;
            }
            MenuState::ShowMainMenu => Carousel::new(0, get_carousel_child, transition_duration),
            MenuState::ShowLevelsPage(n) => {
                Carousel::new(n + 1_u32, get_carousel_child, transition_duration)
            }
        };

        commands.add_child("carousel", carousel, context);
    }
}



fn icon_button_node(button_action: ButtonAction) -> impl MavericNode<Context = AssetServer> {
    ButtonNode {
        style: IconNodeStyle,
        visibility: Visibility::Visible,
        border_color: BUTTON_BORDER,
        background_color: ICON_BUTTON_BACKGROUND,
        marker: button_action,
        children: (TextNode {
            text: button_action.icon(),
            font: MENU_BUTTON_FONT_PATH,
            font_size: ICON_FONT_SIZE,
            color: BUTTON_TEXT_COLOR,
            alignment: TextAlignment::Center,
            linebreak_behavior: bevy::text::BreakLineOn::NoWrap,
        },),
    }
}

fn text_button_node(button_action: ButtonAction) -> impl MavericNode<Context = AssetServer> {
    ButtonNode {
        style: TextButtonStyle,
        visibility: Visibility::Visible,
        border_color: BUTTON_BORDER,
        background_color: TEXT_BUTTON_BACKGROUND,
        marker: button_action,
        children: (TextNode {
            text: button_action.text(),
            font: MENU_BUTTON_FONT_PATH,
            font_size: BUTTON_FONT_SIZE,
            color: BUTTON_TEXT_COLOR,
            alignment: TextAlignment::Center,
            linebreak_behavior: bevy::text::BreakLineOn::NoWrap,
        },),
    }
}

fn text_and_image_button_node(
    button_action: ButtonAction,
    image_path: &'static str,
) -> impl MavericNode<Context = AssetServer> {
    ButtonNode {
        style: TextButtonStyle,
        visibility: Visibility::Visible,
        border_color: BUTTON_BORDER,
        background_color: TEXT_BUTTON_BACKGROUND,
        marker: button_action,
        children: (
            TextNode {
                text: button_action.text(),
                font: MENU_BUTTON_FONT_PATH,
                font_size: BUTTON_FONT_SIZE,
                color: BUTTON_TEXT_COLOR,
                alignment: TextAlignment::Center,
                linebreak_behavior: bevy::text::BreakLineOn::NoWrap,
            },
            ImageNode {
                path: image_path,
                background_color: Color::WHITE,
                style: SmallImageNodeStyle,
            },
        ),
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MainOrLevelMenu {
    Main,
    Level(u32),
}

impl MavericNode for MainOrLevelMenu {
    type Context = NC2<MenuState, AssetServer>;

    fn set_components(commands: SetComponentCommands<Self, Self::Context>) {
        commands.ignore_node().ignore_context().insert(NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                left: Val::Percent(50.0),  // Val::Px(MENU_OFFSET),
                right: Val::Percent(50.0), // Val::Px(MENU_OFFSET),
                top: Val::Px(MENU_OFFSET),
                display: Display::Flex,
                flex_direction: FlexDirection::Column,

                ..Default::default()
            },
            z_index: ZIndex::Global(10),
            ..Default::default()
        });
    }

    fn set_children<R: MavericRoot>(commands: SetChildrenCommands<Self, Self::Context, R>) {
        commands.unordered_children_with_node_and_context(|args, context, commands| match args {
            MainOrLevelMenu::Main => {
                for (key, action) in ButtonAction::main_buttons().iter().enumerate() {
                    let button = text_button_node(*action);
                    let button = button.with_transition_in::<BackgroundColorLens>(
                        Color::WHITE.with_a(0.0),
                        Color::WHITE,
                        Duration::from_secs_f32(1.0),
                    );

                    commands.add_child(key as u32, button, &context.1)
                }
            }
            MainOrLevelMenu::Level(page) => {
                let start = page * LEVELS_PER_PAGE;
                let end = start + LEVELS_PER_PAGE;

                for (key, level) in (start..end).enumerate() {
                    commands.add_child(
                        key as u32,
                        text_and_image_button_node(
                            ButtonAction::GotoLevel { level },
                            r#"images/MedalsBlack.png"#,
                        ),
                        &context.1,
                    )
                }

                commands.add_child("buttons", LevelMenuArrows(*page), &context.1);
            }
        });
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct LevelMenuArrows(u32);

impl MavericNode for LevelMenuArrows {
    type Context = AssetServer;

    fn set_components(commands: SetComponentCommands<Self, Self::Context>) {
        commands.ignore_node().ignore_context().insert(NodeBundle {
            style: Style {
                position_type: PositionType::Relative,
                left: Val::Percent(0.0),
                display: Display::Flex,
                flex_direction: FlexDirection::Row,

                width: Val::Px(TEXT_BUTTON_WIDTH),
                height: Val::Px(TEXT_BUTTON_HEIGHT),
                margin: UiRect {
                    left: Val::Auto,
                    right: Val::Auto,
                    top: Val::Px(5.0),
                    bottom: Val::Px(5.0),
                },
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_grow: 0.0,
                flex_shrink: 0.0,
                border: UiRect::all(UI_BORDER_WIDTH),

                ..Default::default()
            },
            background_color: BackgroundColor(TEXT_BUTTON_BACKGROUND),
            border_color: BorderColor(BUTTON_BORDER),
            ..Default::default()
        });
    }

    fn set_children<R: MavericRoot>(commands: SetChildrenCommands<Self, Self::Context, R>) {
        commands.unordered_children_with_node_and_context(|args, context, commands| {
            if args.0 == 0 {
                commands.add_child("left", icon_button_node(ButtonAction::OpenMenu), context)
            } else {
                commands.add_child(
                    "left",
                    icon_button_node(ButtonAction::PreviousLevelsPage),
                    context,
                )
            }

            let number_of_pages = (level_count() / LEVELS_PER_PAGE) + 1;

            if args.0 < number_of_pages {
                commands.add_child(
                    "right",
                    icon_button_node(ButtonAction::NextLevelsPage),
                    context,
                )
            } else {
                commands.add_child("right", icon_button_node(ButtonAction::None), context)
            }
        });
    }
}

pub const ICON_BUTTON_WIDTH: f32 = 65.;
pub const ICON_BUTTON_HEIGHT: f32 = 65.;

pub const TEXT_BUTTON_WIDTH: f32 = 360.;
pub const TEXT_BUTTON_HEIGHT: f32 = 60.;

pub const MENU_OFFSET: f32 = 10.;

pub const UI_BORDER_WIDTH: Val = Val::Px(3.0);



pub const ICON_FONT_SIZE: f32 = 30.0;
pub const BUTTON_FONT_SIZE: f32 = 22.0;

const LEVELS_PER_PAGE: u32 = 8;

pub const BUTTON_BORDER: Color = Color::BLACK;
pub const BUTTON_TEXT_COLOR: Color = Color::rgb(0.1, 0.1, 0.1);

pub const ICON_BUTTON_BACKGROUND: Color = Color::NONE;
pub const TEXT_BUTTON_BACKGROUND: Color = Color::WHITE;

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct BigImageNodeStyle;

impl IntoBundle for BigImageNodeStyle {
    type B = Style;

    fn into_bundle(self) -> Self::B {
        Style {
            width: Val::Px(TEXT_BUTTON_HEIGHT * 2.0),
            height: Val::Px(TEXT_BUTTON_HEIGHT),
            margin: UiRect {
                left: Val::Auto,
                right: Val::Auto,
                top: Val::Px(5.0),
                bottom: Val::Px(5.0),
            },
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            flex_grow: 0.0,
            flex_shrink: 0.0,
            border: UiRect::all(UI_BORDER_WIDTH),
            ..default()
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct SmallImageNodeStyle;

impl IntoBundle for SmallImageNodeStyle {
    type B = Style;

    fn into_bundle(self) -> Self::B {
        Style {
            width: Val::Px((TEXT_BUTTON_HEIGHT - 10.0) * 2.0),
            height: Val::Px(TEXT_BUTTON_HEIGHT - 10.0),
            margin: UiRect {
                left: Val::Auto,
                right: Val::Px(0.0),
                top: Val::Px(5.0),
                bottom: Val::Px(5.0),
            },
            align_self: AlignSelf::Center,
            justify_self: JustifySelf::End,
            ..default()
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct IconNodeStyle;

impl IntoBundle for IconNodeStyle {
    type B = Style;

    fn into_bundle(self) -> Self::B {
        Style {
            width: Val::Px(ICON_BUTTON_WIDTH),
            height: Val::Px(ICON_BUTTON_HEIGHT),
            margin: UiRect::all(Val::Auto),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            flex_grow: 0.0,
            flex_shrink: 0.0,

            ..Default::default()
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct OpenMenuButtonStyle;

impl IntoBundle for OpenMenuButtonStyle {
    type B = Style;

    fn into_bundle(self) -> Self::B {
        Style {
            width: Val::Px(ICON_BUTTON_WIDTH),
            height: Val::Px(ICON_BUTTON_HEIGHT),
            margin: UiRect::DEFAULT,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            flex_grow: 0.0,
            flex_shrink: 0.0,
            left: Val::Px(40.0),
            top: Val::Px(40.0),

            ..Default::default()
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct TextButtonStyle;

impl IntoBundle for TextButtonStyle {
    type B = Style;

    fn into_bundle(self) -> Self::B {
        Style {
            width: Val::Px(TEXT_BUTTON_WIDTH),
            height: Val::Px(TEXT_BUTTON_HEIGHT),
            margin: UiRect {
                left: Val::Auto,
                right: Val::Auto,
                top: Val::Px(5.0),
                bottom: Val::Px(5.0),
            },
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            flex_grow: 0.0,
            flex_shrink: 0.0,
            border: UiRect::all(UI_BORDER_WIDTH),

            ..Default::default()
        }
    }
}
