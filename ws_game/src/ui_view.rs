use crate::prelude::*;
use itertools::Itertools;
use maveric::{transition::speed::ScalarSpeed, widgets::text2d_node::Text2DNode};
#[derive(Debug, Clone, PartialEq)]
pub struct UI;

pub const BUTTON_FONT_SIZE: f32 = 22.0;
pub const BUTTON_TEXT_COLOR: Color = Color::rgb(0.1, 0.1, 0.1);
pub const TEXT_BUTTON_WIDTH: f32 = 140.;
pub const TEXT_BUTTON_HEIGHT: f32 = 30.;
pub const UI_BORDER_WIDTH: Val = Val::Px(3.0);

impl MavericNode for UI {
    type Context = NC4<ChosenState, CurrentLevel, FoundWordsState, NC2<Size, AssetServer>>;

    fn set_components(commands: SetComponentCommands<Self, Self::Context>) {
        commands
            .ignore_context()
            .ignore_node()
            .insert((TransformBundle::default(), VisibilityBundle::default()))
            .finish()
    }

    fn set_children<R: MavericRoot>(commands: SetChildrenCommands<Self, Self::Context, R>) {
        commands
            .ignore_node()
            .unordered_children_with_context(|context, commands| {
                let size = &context.3 .0;
                let asset_server = &context.3 .1;

                commands.add_child(
                    "Burger",
                    Text2DNode {
                        text: TextNode {
                            text: "\u{f0c9}",
                            font_size: BUTTON_FONT_SIZE,
                            color: BUTTON_TEXT_COLOR,
                            font: MENU_BUTTON_FONT_PATH,
                            alignment: TextAlignment::Center,
                            linebreak_behavior: bevy::text::BreakLineOn::NoWrap,
                        },

                        transform: Transform::from_translation(
                            size.get_rect(LayoutEntity::TopBarItem(TopBarButton::MenuBurgerButton))
                                .centre()
                                .extend(crate::z_indices::TOP_BAR_BUTTON),
                        ),
                    },
                    asset_server,
                );

                commands.add_child(
                    "hints",
                    Text2DNode {
                        text: TextNode {
                            text: context.2.hint_count().to_string(),
                            font_size: BUTTON_FONT_SIZE,
                            color: BUTTON_TEXT_COLOR,
                            font: BUTTONS_FONT_PATH,
                            alignment: TextAlignment::Center,
                            linebreak_behavior: bevy::text::BreakLineOn::NoWrap,
                        },
                        transform: Transform::from_translation(
                            size.get_rect(LayoutEntity::TopBarItem(TopBarButton::HintCounter))
                                .centre()
                                .extend(crate::z_indices::TOP_BAR_BUTTON),
                        ),
                    },
                    &context.3 .1,
                );

                let title = context.1.level().name.trim().to_string();

                commands.add_child(
                    "title",
                    Text2DNode {
                        text: TextNode {
                            text: title,
                            font_size: 32.0,
                            color: BUTTON_TEXT_COLOR,
                            font: TITLE_FONT_PATH,
                            alignment: TextAlignment::Center,
                            linebreak_behavior: bevy::text::BreakLineOn::NoWrap,
                        },
                        transform: Transform::from_translation(
                            size.get_rect(LayoutEntity::TextAreaItem(TextItem::PuzzleTitle))
                                .centre()
                                .extend(crate::z_indices::TEXT_AREA_TEXT),
                        )
                    },
                    &context.3 .1,
                );

                commands.add_child(
                    "theme",
                    Text2DNode {
                        text: TextNode {
                            text: "Theme",
                            font_size: 32.0,
                            color: BUTTON_TEXT_COLOR,
                            font: TITLE_FONT_PATH,
                            alignment: TextAlignment::Center,
                            linebreak_behavior: bevy::text::BreakLineOn::NoWrap,
                        },
                        transform: Transform::from_translation(
                            size.get_rect(LayoutEntity::TextAreaItem(TextItem::PuzzleTheme))
                                .centre()
                                .extend(crate::z_indices::TEXT_AREA_TEXT),
                        )
                    },
                    &context.3 .1,
                );

                commands.add_child("words", WordsNode, context);
            });
    }
}

#[derive(Debug, PartialEq)]
pub struct WordsNode;

impl MavericNode for WordsNode {
    type Context = NC4<ChosenState, CurrentLevel, FoundWordsState, NC2<Size, AssetServer>>;

    fn set_components(commands: SetComponentCommands<Self, Self::Context>) {
        commands
            .ignore_context()
            .ignore_node()
            .insert((TransformBundle::default(), VisibilityBundle::default()));
    }

    fn set_children<R: MavericRoot>(commands: SetChildrenCommands<Self, Self::Context, R>) {
        commands
            .ignore_node()
            .unordered_children_with_context(|context, commands| {
                for (index, word) in context.1.level().words.iter().enumerate() {
                    let completion = context.2.get_completion(&word.characters);

                    if let Some(tile) = WordTile::try_from_usize(index) {
                        commands.add_child(
                            index as u32,
                            WordNode {
                                word: word.clone(),
                                tile,
                                completion,
                            },
                            &context.3,
                        )
                    }
                }
            });
    }
}

#[derive(Debug, PartialEq)]
pub struct WordNode {
    pub tile: WordTile,
    pub word: Word,
    pub completion: Completion,
}

impl MavericNode for WordNode {
    type Context = NC2<Size, AssetServer>;

    fn set_components(mut commands: SetComponentCommands<Self, Self::Context>) {
        commands.scope(|commands| {
            commands
                .ignore_node()
                .ignore_context()
                .insert((TransformBundle::default(), VisibilityBundle::default()))
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
                Completion::Incomplete => node.word.characters.len().to_string(),
                Completion::Hinted(hints) => {
                    let hinted_characters = node.word.text.chars().take(hints);
                    let question_marks = std::iter::repeat('?');

                    std::iter::Iterator::chain(hinted_characters, question_marks)
                        .take(node.word.characters.len())
                        .join("")
                }

                Completion::Complete => node.word.text.to_string(),
            };
            let text_transform = context
                .0
                .get_rect(LayoutEntity::Word(node.tile))
                .centre()
                .extend(crate::z_indices::WORD_TEXT);

            commands.add_child(
                0,
                Text2DNode {
                    text: TextNode {
                        text,
                        font_size: BUTTON_FONT_SIZE,
                        color: BUTTON_TEXT_COLOR,
                        font: SOLUTIONS_FONT_PATH,
                        alignment: TextAlignment::Center,
                        linebreak_behavior: bevy::text::BreakLineOn::NoWrap,
                    },
                    transform: Transform::from_translation(text_transform),
                },
                &context.1,
            );
        })
    }
}
