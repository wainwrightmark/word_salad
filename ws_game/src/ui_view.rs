use crate::prelude::*;
use itertools::Itertools;
use maveric::transition::speed::ScalarSpeed;

#[derive(Debug, Clone, PartialEq)]
pub struct UI;

pub const BUTTON_FONT_SIZE: f32 = 22.0;
pub const BUTTON_TEXT_COLOR: Color = Color::rgb(0.1, 0.1, 0.1);
pub const TEXT_BUTTON_WIDTH: f32 = 140.;
pub const TEXT_BUTTON_HEIGHT: f32 = 30.;
pub const UI_BORDER_WIDTH: Val = Val::Px(3.0);

#[derive(Debug, Clone, PartialEq, Default)]
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

impl MavericNode for UI {
    type Context = NC4<ChosenState, CurrentLevel, FoundWordsState, NC2<Size, AssetServer>>;

    fn set_components(commands: SetComponentCommands<Self, Self::Context>) {
        let mapped: SetComponentCommands<(), Size> =
            commands.ignore_node().map_context(|x| &x.3 .0);

        mapped.insert_with_context(|size: &Res<Size>| {
            let y = size.ui_top();
            let left = (size.scaled_width - (size.scale() * 4.0)) * 0.5;
            NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    top: Val::Px(y),
                    left: Val::Px(left),
                    width: Val::Px(size.scale() * 4.0),
                    display: Display::Flex,
                    flex_direction: FlexDirection::Column,

                    ..Default::default()
                },

                ..Default::default()
            }
        });
    }

    fn set_children<R: MavericRoot>(commands: SetChildrenCommands<Self, Self::Context, R>) {
        commands
            .ignore_node()
            .unordered_children_with_context(|context, commands| {
                let level = context.1.level();
                let current_string: String = context
                    .0
                     .0
                    .iter()
                    .map(|tile| level.grid[*tile])
                    .map(|c| c.as_char())
                    .collect();
                let current_string: String = format!("{current_string:^24}");

                commands.add_child(
                    "current",
                    TextNode {
                        text: current_string,
                        font_size: 32.0,
                        color: BUTTON_TEXT_COLOR,
                        font: CURRENT_STRING_FONT_PATH,
                        alignment: TextAlignment::Center,
                        linebreak_behavior: bevy::text::BreakLineOn::NoWrap,
                    },
                    &context.3 .1,
                );

                let title = context.1.level().name.trim().to_string();

                commands.add_child(
                    "title",
                    TextNode {
                        text: title,
                        font_size: 32.0,
                        color: BUTTON_TEXT_COLOR,
                        font: TITLE_FONT_PATH,
                        alignment: TextAlignment::Center,
                        linebreak_behavior: bevy::text::BreakLineOn::NoWrap,
                    },
                    &context.3 .1,
                );

                let hints = context.2.hint_count();
                let hints = format!("Used {hints} hints");

                commands.add_child(
                    "hints",
                    TextNode {
                        text: hints,
                        font_size: 10.0,
                        color: BUTTON_TEXT_COLOR,
                        font: TITLE_FONT_PATH,
                        alignment: TextAlignment::Center,
                        linebreak_behavior: bevy::text::BreakLineOn::NoWrap,
                    },
                    &context.3 .1,
                );

                commands.add_child("words", WordsNode, context);

                commands.add_child("buttons", ButtonsNode, &context.3 .1);
            });
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ButtonsNode;

impl MavericNode for ButtonsNode {
    type Context = AssetServer;

    fn set_components(commands: SetComponentCommands<Self, Self::Context>) {
        commands.ignore_node().ignore_context().insert(NodeBundle {
            style: Style {
                display: Display::Flex,
                flex_direction: FlexDirection::Row,
                flex_wrap: FlexWrap::Wrap,
                ..Default::default()
            },
            ..Default::default()
        });
    }

    fn set_children<R: MavericRoot>(commands: SetChildrenCommands<Self, Self::Context, R>) {
        commands
            .ignore_node()
            .unordered_children_with_context(|context, commands| {


                commands.add_child(
                    "Menu",
                    ButtonNode {
                        style: TextButtonStyle::default(),
                        visibility: Visibility::Visible,
                        border_color: Color::BLACK,
                        background_color: Color::NONE,
                        marker: ButtonAction::OpenMenu,
                        children: (TextNode {
                            text: "Menu",
                            font_size: BUTTON_FONT_SIZE,
                            color: BUTTON_TEXT_COLOR,
                            font: BUTTONS_FONT_PATH,
                            alignment: TextAlignment::Center,
                            linebreak_behavior: bevy::text::BreakLineOn::NoWrap,
                        },),
                    },
                    &context,
                );

                // commands.add_child(
                //     "next",
                //     ButtonNode {
                //         style: TextButtonStyle::default(),
                //         visibility: Visibility::Visible,
                //         border_color: Color::BLACK,
                //         background_color: Color::NONE,
                //         marker: ButtonMarker::NextLevel,
                //         children: (TextNode {
                //             text: "Next",
                //             font_size: BUTTON_FONT_SIZE,
                //             color: BUTTON_TEXT_COLOR,
                //             font: BUTTONS_FONT_PATH,
                //             alignment: TextAlignment::Center,
                //             linebreak_behavior: bevy::text::BreakLineOn::NoWrap,
                //         },),
                //     },
                //     &context,
                // );

                // commands.add_child(
                //     "reset",
                //     ButtonNode {
                //         style: TextButtonStyle::default(),
                //         visibility: Visibility::Visible,
                //         border_color: Color::BLACK,
                //         background_color: Color::NONE,
                //         marker: ButtonMarker::Reset,
                //         children: (TextNode {
                //             text: "Reset",
                //             font_size: BUTTON_FONT_SIZE,
                //             color: BUTTON_TEXT_COLOR,
                //             font: BUTTONS_FONT_PATH,
                //             alignment: TextAlignment::Center,
                //             linebreak_behavior: bevy::text::BreakLineOn::NoWrap,
                //         },),
                //     },
                //     &context,
                // );

                commands.add_child(
                    "hint",
                    ButtonNode {
                        style: TextButtonStyle::default(),
                        visibility: Visibility::Visible,
                        border_color: Color::BLACK,
                        background_color: Color::NONE,
                        marker: ButtonAction::Hint,
                        children: (TextNode {
                            text: "Hint",
                            font_size: BUTTON_FONT_SIZE,
                            color: BUTTON_TEXT_COLOR,
                            font: BUTTONS_FONT_PATH,
                            alignment: TextAlignment::Center,
                            linebreak_behavior: bevy::text::BreakLineOn::NoWrap,
                        },),
                    },
                    &context,
                );
            });
    }
}



#[derive(Debug, PartialEq)]
pub struct WordsNode;

impl MavericNode for WordsNode {
    type Context = NC4<ChosenState, CurrentLevel, FoundWordsState, NC2<Size, AssetServer>>;

    fn set_components(commands: SetComponentCommands<Self, Self::Context>) {
        commands.ignore_context().ignore_node().insert(NodeBundle {
            style: Style {
                display: Display::Flex,
                flex_direction: FlexDirection::Row,
                flex_wrap: FlexWrap::Wrap,
                ..Default::default()
            },
            ..Default::default()
        });
    }

    fn set_children<R: MavericRoot>(commands: SetChildrenCommands<Self, Self::Context, R>) {
        commands
            .ignore_node()
            .unordered_children_with_context(|context, commands| {
                for (index, word) in context.1.level().words.iter().enumerate() {
                    let completion = context.2.get_completion(&word.characters);

                    commands.add_child(
                        index as u32,
                        WordNode {
                            word: word.clone(),
                            completion,
                        },
                        &context.3 .1,
                    )
                }
            });
    }
}



#[derive(Debug, PartialEq)]
pub struct WordNode {
    pub word: Word,
    pub completion: Completion,
}

impl MavericNode for WordNode {
    type Context = AssetServer;

    fn set_components(mut commands: SetComponentCommands<Self, Self::Context>) {
        commands.scope(|commands| {
            commands
                .ignore_node()
                .ignore_context()
                .insert(NodeBundle {
                    style: Style {
                        margin: UiRect::all(Val::Px(10.0)),

                        //border: UiRect::all(Val::Px(1.0)),
                        ..Default::default()
                    },
                    background_color: BackgroundColor(Color::rgb(0.9, 0.9, 0.9)),
                    ..Default::default()
                })
                .finish()
        });

        commands
            .map_args(|x| x.completion.color())
            .ignore_context()
            .animate_on_node::<BackgroundColorLens>(Some(ScalarSpeed::new(1.0)));
    }

    fn set_children<R: MavericRoot>(commands: SetChildrenCommands<Self, Self::Context, R>) {
        commands.unordered_children_with_node_and_context(|node, context, commands| {
            let text = match node.completion {
                Completion::Incomplete => str::repeat("?", node.word.characters.len()),
                Completion::Hinted(hints) => node
                    .word
                    .text
                    .chars()
                    .take(hints)
                    .chain(std::iter::repeat('?'))
                    .take(node.word.characters.len())
                    .join(""),
                Completion::Complete => node.word.text.to_string(),
            };

            commands.add_child(
                0,
                TextNode {
                    text,
                    font_size: BUTTON_FONT_SIZE,
                    color: BUTTON_TEXT_COLOR,
                    font: SOLUTIONS_FONT_PATH,
                    alignment: TextAlignment::Center,
                    linebreak_behavior: bevy::text::BreakLineOn::NoWrap,
                },
                &context,
            );
        })
    }
}
