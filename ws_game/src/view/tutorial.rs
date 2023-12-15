use std::time::Duration;

use bevy::math::Vec2;
use maveric::{
    helpers::{ChildCommands, SpatialBundle},
    node::MavericNode,
    widgets::text2d_node::Text2DNode,
    with_bundle::CanWithBundle,
};
use strum::{EnumCount, EnumIter, IntoEnumIterator};
use ws_core::{LayoutRectangle, LayoutStructure};

use crate::prelude::*;

#[derive(Debug, PartialEq)]
pub struct TutorialNode {
    pub text: TutorialText,
}

impl MavericNode for TutorialNode {
    type Context = MyWindowSize;

    fn set_components(commands: maveric::prelude::SetComponentCommands<Self, Self::Context>) {
        commands
            .ignore_context()
            .ignore_node()
            .insert(SpatialBundle::default())
            .finish()
    }

    fn set_children<R: maveric::prelude::MavericRoot>(
        commands: maveric::prelude::SetChildrenCommands<Self, Self::Context, R>,
    ) {
        commands.ordered_children_with_node_and_context(|node, context, commands| {
            if let Some(text) = node.text.top {
                let rect = context.get_rect(&TutorialLayoutEntity::Top, &());
                let font_size = context.font_size(&TutorialLayoutEntity::Top);
                commands.add_child(
                    "top",
                    TutorialPopupNode {
                        text,
                        rect,
                        font_size,
                    }
                    .with_transition_in::<TransformScaleLens>(
                        Vec3::ZERO,
                        Vec3::ONE,
                        Duration::from_secs_f32(0.5),
                    ),
                    &(),
                );
            }
             if let Some(text) = node.text.middle {
                let rect = context.get_rect(&TutorialLayoutEntity::Middle, &());
                let font_size = context.font_size(&TutorialLayoutEntity::Middle);
                commands.add_child(
                    "middle",
                    TutorialPopupNode {
                        text,
                        rect,
                        font_size,
                    }
                    .with_transition_in::<TransformScaleLens>(
                        Vec3::ZERO,
                        Vec3::ONE,
                        Duration::from_secs_f32(0.5),
                    ),
                    &(),
                );
            }
            if let Some(text) = node.text.bottom {
                let rect = context.get_rect(&TutorialLayoutEntity::Bottom, &());
                let font_size = context.font_size(&TutorialLayoutEntity::Bottom);
                commands.add_child(
                    "bottom",
                    TutorialPopupNode {
                        text,
                        rect,
                        font_size,
                    }
                    .with_transition_in::<TransformScaleLens>(
                        Vec3::ZERO,
                        Vec3::ONE,
                        Duration::from_secs_f32(0.5),
                    ),
                    &(),
                );
            }
        });
    }
}

#[derive(Debug, PartialEq)]
struct TutorialPopupNode {
    text: &'static str,
    rect: LayoutRectangle,
    font_size: f32,
}

impl MavericNode for TutorialPopupNode {
    type Context = ();

    fn set_components(commands: maveric::prelude::SetComponentCommands<Self, Self::Context>) {
        commands
            .ignore_context()
            .ignore_node()
            .insert(SpatialBundle::default())
            .finish()
    }

    fn set_children<R: maveric::prelude::MavericRoot>(
        commands: maveric::prelude::SetChildrenCommands<Self, Self::Context, R>,
    ) {
        commands.unordered_children_with_node(|node, commands| {
            let TutorialPopupNode {
                text,
                rect,
                font_size,
            } = node;
            let background = crate::shapes::box_with_border_node(
                rect.width(),
                rect.height(),
                rect.centre()
                    .extend(crate::z_indices::TUTORIAL_POPUP_BOX_BACKGROUND),
                ws_core::palette::POPUP_BOX_BACKGROUND
                    .convert_color()
                    .with_a(0.8),
                ws_core::palette::POPUP_BOX_BORDER
                    .convert_color()
                    .with_a(0.8),
                0.1,
                0.01,
            );

            commands.add_child("background", background, &());

            let text = Text2DNode {
                text: *text,
                font: TITLE_FONT_PATH,
                font_size: *font_size,
                color: Color::BLACK,
                alignment: TextAlignment::Center,
                linebreak_behavior: bevy::text::BreakLineOn::NoWrap,
            }
            .with_bundle(Transform::from_translation(
                rect.centre()
                    .extend(crate::z_indices::TUTORIAL_POPUP_BOX_TEXT),
            ));

            commands.add_child("title text", text, &());
        });
    }
}

#[derive(Debug, PartialEq)]
pub struct TutorialText {
    top: Option<&'static str>,
    middle: Option<&'static str>,
    bottom: Option<&'static str>,
}

impl TutorialText {
    pub fn try_create(current_level: &CurrentLevel, found_words: &FoundWordsState) -> Option<Self> {
        let level_index = match current_level {
            CurrentLevel::Tutorial {
                index,
            } => {
                *index % 2
            }
            _=> {return None;}
        };

        let completed_words = found_words
            .word_completions
            .iter()
            .filter(|x| x.is_complete())
            .count();
        let result = if level_index == 0 {
            //Chess Pieces
            match completed_words {
                0 => Self {
                    top: Some(
                        "\
                        Let's start by finding 'Pawn'\n\
                        Tap or swipe adjacent tiles\n\
                        to make the word",
                    ),
                    middle: None,
                    bottom: None,
                },
                1 => Self {
                    top: Some(
                        "\
                        Words can be made diagonally\n\
                        Like 'Queen'",
                    ),
                    middle: None,
                    bottom: Some(
                        "\
                        These labels show the word lengths\n\
                        Four more to go",
                    ),
                },
                2 => Self {
                    top: Some(
                        "\
                        Find the final three\n\
                        Chess Pieces\n\
                        to finish the puzzle",
                    ),
                    middle: None,
                    bottom: Some(
                        "\
                        Labels are listed alphabetically\n\
                        Use this to your advantage",
                    ),
                },
                3 => Self {
                    top: Some(
                        "\
                        Find the final two\n\
                        Chess Pieces\n\
                        to finish the puzzle",
                    ),
                    middle: None,
                    bottom: Some(
                        "\
                        Labels are listed alphabetically\n\
                        Use this to your advantage",
                    ),
                },
                4 => Self {
                    top: Some(
                        "\
                        Just one Chess Piece left",
                    ),
                    middle: None,
                    bottom: Some(
                        "\
                        Labels are listed alphabetically\n\
                        Use this to your advantage",
                    ),
                },

                _ => {
                    //Completed
                    Self {
                        top: None,
                        middle: Some(
                            "\
                            You completed your first\n\
                            Word Salad\n\
                            You've earned two hints\n\
                            Hints reveal a letter\n\
                            from a word of your choosing",
                        ),
                        bottom: None,
                    }
                }
            }
        } else {
            //Planets
            match completed_words {
                0 => Self {
                    top: Some(
                        "\
                    Your line can cross\n\
                    over itself\n\
                    Find 'Mars'",
                    ),
                    middle: None,
                    bottom: None,
                },
                1..=3 => Self {
                    top: Some("Find the other planets"),
                    middle: None,
                    bottom: Some(
                        "\
                    To use a hint\n\
                     click a word you haven't found yet",
                    ),
                },
                4 => Self {
                    top: Some("Find the other planets"),
                    middle: None,
                    bottom: Some(
                        "\
                    You can hint a word more than once\n\
                    to reveal more letters",
                    ),
                },
                5 => Self {
                    top: Some("You're a Word Salad expert"),
                    middle: None,
                    bottom: Some(
                        "\
                    You can hint a word more than once\n\
                    to reveal more letters",
                    ),
                },
                _ => {
                    //Completed
                    Self {
                        top: None,
                        middle: Some(
                            "\
                        Wanna film yourself playing?\n\
                        Use Selfie Mode in the menu\n\
                        Then use your device's\n\
                        Screen Recorder\n\
                        Remember to tag us!",
                        ),

                        bottom: None,
                    }
                }
            }
        };

        return Some(result);
    }
}

#[derive(Debug, EnumCount, EnumIter, PartialEq, Clone, Copy)]
enum TutorialLayoutEntity {
    Top,
    Middle,
    Bottom,
}

impl LayoutStructure for TutorialLayoutEntity {
    type Context = ();

    type Iterator = <Self as IntoEnumIterator>::Iterator;

    fn pick(point: bevy::prelude::Vec2, context: &Self::Context) -> Option<Self> {
        for x in Self::iter() {
            if x.rect(context).contains(point) {
                return Some(x);
            }
        }
        return None;
    }

    fn size(&self, _context: &Self::Context) -> bevy::prelude::Vec2 {
        match self {
            TutorialLayoutEntity::Top => Vec2 { x: 300.0, y: 70.0 },
            TutorialLayoutEntity::Middle => Vec2 { x: 300.0, y: 140.0 },
            TutorialLayoutEntity::Bottom => Vec2 { x: 300.0, y: 40.0 },
        }
    }

    fn location(&self, _context: &Self::Context) -> bevy::prelude::Vec2 {
        match self {
            TutorialLayoutEntity::Top => Vec2 { x: 10.0, y: 52.0 },
            TutorialLayoutEntity::Middle => Vec2 { x: 10.0, y: 52.0 },
            TutorialLayoutEntity::Bottom => Vec2 { x: 10.0, y: 500.0 },
        }
    }

    fn iter_all(_context: &Self::Context) -> Self::Iterator {
        Self::iter()
    }
}

impl LayoutStructureWithFont for TutorialLayoutEntity {
    fn font_size(&self) -> f32 {
        match self {
            TutorialLayoutEntity::Top => 20.0,
            TutorialLayoutEntity::Middle => 20.0,
            TutorialLayoutEntity::Bottom => 16.0,
        }
    }
}
