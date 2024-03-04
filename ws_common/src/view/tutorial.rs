use std::time::Duration;

use bevy::{math::Vec2, text::Text2dBounds};
use itertools::Itertools;
use maveric::{
    helpers::{ChildCommands, SpatialBundle},
    node::MavericNode,
    widgets::text2d_node::Text2DNode,
    with_bundle::CanWithBundle,
};
use strum::{EnumCount, EnumIs, EnumIter, IntoEnumIterator};
use ws_core::{
    layout::entities::{
        GameLayoutEntity, SelfieMode, GRID_SIZE, LEFT_MARGIN, TUTORIAL_TEXT_FONT_SIZE,
    },
    LayoutStructure,
};

use crate::prelude::*;

pub const POPUP_TRANSITION_IN_SECONDS: f32 = 2.0;
pub const POPUP_TRANSITION_OUT_SECONDS: f32 = 2.0;

#[derive(Debug, PartialEq)]
pub struct TutorialNode {
    pub text: TutorialText,
}

#[derive(Debug, NodeContext)]
pub struct TutorialContext {
    pub window_size: MyWindowSize,
    pub video_resource: VideoResource,
    pub insets: InsetsResource,
}

impl<'a, 'w: 'a> From<&'a ViewContextWrapper<'w>> for TutorialContextWrapper<'w> {
    fn from(value: &'a ViewContextWrapper<'w>) -> Self {
        Self {
            window_size: Res::clone(&value.window_size),
            video_resource: Res::clone(&value.video_resource),
            insets: Res::clone(&value.insets),
        }
    }
}

impl MavericNode for TutorialNode {
    type Context = TutorialContext;

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
                commands.add_child(
                    "top",
                    TutorialPopupNode {
                        text,
                        entity: TutorialLayoutEntity::Top,
                        color: palette::TUTORIAL_TOP_TEXT.convert_color(),
                        align_left: true,
                    }
                    .with_transition_in_out::<TransformScaleLens>(
                        Vec3::ZERO,
                        Vec3::ONE,
                        Vec3::ZERO,
                        Duration::from_secs_f32(POPUP_TRANSITION_IN_SECONDS),
                        Duration::from_secs_f32(POPUP_TRANSITION_OUT_SECONDS),
                        Some(Ease::CubicOut),
                        Some(Ease::CubicOut),
                    ),
                    context,
                );
            }
            if let Some(text) = node.text.middle {
                let transition = TransitionBuilder::default()
                    .then_wait(Duration::from_secs_f32(TRANSITION_WAIT_SECS))
                    .then_ease(Vec3::ONE, (1.0 / TRANSITION_SECS).into(), Ease::CubicOut)
                    .build();

                commands.add_child(
                    "middle",
                    TutorialPopupNode {
                        text,
                        entity: TutorialLayoutEntity::BigTop,
                        color: palette::TUTORIAL_MIDDLE_TEXT.convert_color(),
                        align_left: false,
                    }
                    .with_transition::<TransformScaleLens, ()>(
                        Vec3::ZERO,
                        transition,
                        (),
                    ),
                    context,
                );
            }
            if let Some(text) = node.text.bottom {
                commands.add_child(
                    "bottom",
                    TutorialPopupNode {
                        text,
                        entity: TutorialLayoutEntity::Bottom,
                        color: palette::TUTORIAL_BOTTOM_TEXT.convert_color(),
                        align_left: true,
                    }
                    .with_transition_in_out::<TransformScaleLens>(
                        Vec3::ZERO,
                        Vec3::ONE,
                        Vec3::ZERO,
                        Duration::from_secs_f32(POPUP_TRANSITION_IN_SECONDS),
                        Duration::from_secs_f32(POPUP_TRANSITION_OUT_SECONDS),
                        Some(Ease::CubicOut),
                        Some(Ease::CubicOut),
                    ),
                    context,
                );
            }
        });
    }
}

#[derive(Debug, PartialEq)]
pub struct TutorialPopupNode {
    pub text: &'static str,
    pub entity: TutorialLayoutEntity,
    pub color: Color,
    pub align_left: bool,
}

impl MavericNode for TutorialPopupNode {
    type Context = TutorialContext;

    fn set_components(mut commands: maveric::prelude::SetComponentCommands<Self, Self::Context>) {
        commands.insert_static_bundle(SpatialBundle::default());
    }

    fn set_children<R: maveric::prelude::MavericRoot>(
        commands: maveric::prelude::SetChildrenCommands<Self, Self::Context, R>,
    ) {
        commands.unordered_children_with_node_and_context(|node, context, commands| {
            let TutorialPopupNode {
                text,
                entity,
                color,
                align_left,
            } = node;

            let font_size = context.window_size.font_size(entity, &());
            let text_rect = context.window_size.get_rect(
                entity,
                &(context.video_resource.selfie_mode(), context.insets.0),
            );

            let (justify_text, translation, text_anchor) = if *align_left {
                (
                    JustifyText::Left,
                    text_rect.centre_left(),
                    bevy::sprite::Anchor::CenterLeft,
                )
            } else {
                (
                    JustifyText::Center,
                    text_rect.centre(),
                    bevy::sprite::Anchor::Center,
                )
            };

            let text = Text2DNode {
                text: *text,
                font: TUTORIAL_FONT_PATH,
                font_size,
                color: *color,
                justify_text,
                linebreak_behavior: bevy::text::BreakLineOn::NoWrap,
                text_2d_bounds: Text2dBounds::default(),
                text_anchor,
            }
            .with_bundle(Transform::from_translation(
                translation.extend(crate::z_indices::TUTORIAL_POPUP_BOX_TEXT),
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
            CurrentLevel::Tutorial { index } => *index % 2,
            _ => {
                return None;
            }
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
                        Make words by linking tiles",
                    ),
                    middle: None,
                    bottom: None,
                },
                1 => {
                    let found_index = found_words
                        .word_completions
                        .iter()
                        .find_position(|completion| completion.is_complete())
                        .map(|x| x.0)
                        .unwrap_or_default();

                    let bottom = match found_index {
                        4 => {
                            "\
                        \n\
                        These labels show the word lengths\n\
                        The first 6-letter word is 'Bishop'"
                        }
                        _ => {
                            "\
                        \n\
                        These labels show the word lengths\n\
                        The 5-letter word is 'Queen'"
                        }
                    };

                    Self {
                        top: Some(
                            "\
                        Letters vanish when no longer needed\n\
                        Every remaining letter is needed",
                        ),
                        middle: None,
                        bottom: Some(bottom),
                    }
                }
                2 => Self {
                    top: Some(
                        "\
                        Find the final three Chess Pieces\n",
                    ),
                    middle: None,
                    bottom: None,
                },
                3 => Self {
                    top: None,
                    middle: None,
                    bottom: Some(
                        "\
                        \n\
                        Labels are listed alphabetically\n\
                        This can help you find first letters",
                    ),
                },
                4 => {
                    let incomplete_index = found_words
                        .word_completions
                        .iter()
                        .find_position(|completion| !completion.is_complete())
                        .map(|x| x.0)
                        .unwrap_or_default();

                    let bottom = match incomplete_index {
                        0 =>
                        //bishop
                        {
                            "\
                        \n\
                        Because the labels are alphabetical\n\
                        This word starts with 'B', 'H', or 'I'"
                        }

                        1 =>
                        //king
                        {
                            "\
                        \n\
                        Because the labels are alphabetical\n\
                        This word can't start with an 'N'"
                        }

                        2 =>
                        //knight
                        {
                            "\
                        \n\
                        Because the labels are alphabetical\n\
                        This word must start with 'K' or 'N'"
                        }

                        3 =>
                        //pawn
                        {
                            "\
                        \n\
                        Because the labels are alphabetical\n\
                        This word must start with 'P' or 'N'"
                        }

                        _ =>
                        // queen
                        {
                            "\
                            \n\
                            Because the labels are alphabetical\n\
                            This word must start with 'Q'"
                        }
                    };

                    Self {
                        top: Some(
                            "\
                            Just one Chess Piece left\n\
                            See below for a clue!",
                        ),
                        middle: None,
                        bottom: Some(bottom),
                    }
                }

                _ => {
                    //Completed
                    Self {
                        top: None,
                        middle: Some(
                            "\
                            You completed your first Word Salad\n\
                            The next puzzle is about Planets",
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
                    Find six planets\n\
                    The 4-letter planet is 'Mars'",
                    ),
                    middle: None,
                    bottom: None,
                },
                1 => Self {
                    top: Some("Find the other planets"),
                    middle: None,
                    bottom: Some(
                        "\
                        \n\
                    Want help? Use a hint\n\
                    Press a label to reveal a letter",
                    ),
                },
                2 => Self {
                    top: Some("Hints are free in the tutorial"),
                    middle: None,
                    bottom: Some(
                        "\
                        \n\
                    Want help? Use a hint\n\
                    Press a label to reveal a letter",
                    ),
                },
                3..=4 => Self {
                    top: Some("Hints are free in the tutorial"),
                    middle: None,
                    bottom: None,
                },
                5 => {
                    let incomplete_index = found_words
                        .word_completions
                        .iter()
                        .find_position(|completion| !completion.is_complete())
                        .map(|x| x.0)
                        .unwrap_or_default();

                    let bottom = match incomplete_index {
                        0 =>
                        // Mars
                        {
                            "\
                        \n\
                        Because the labels are alphabetical\n\
                        This word starts with 'A' or 'MA'"
                        }

                        1 =>
                        // Mercury
                        {
                            "\
                        \n\
                        Because the labels are alphabetical\n\
                        This word starts with 'M'"
                        }

                        2 =>
                        // Neptune
                        {
                            "\
                        \n\
                        Because the labels are alphabetical\n\
                        This word must start with 'N' or 'P'"
                        }

                        3 =>
                        // Saturn
                        {
                            "\
                        \n\
                        Because the labels are alphabetical\n\
                        This word cannot start with 'A'"
                        }

                        4 =>
                        // Uranus
                        {
                            "\
                            \n\
                            Because the labels are alphabetical\n\
                            This word must start with 'S' or 'U'"
                        }

                        _ =>
                        // Venus
                        {
                            "\
                            \n\
                            Because the labels are alphabetical\n\
                            This word must start with 'U' or 'V'"
                        }
                    };

                    Self {
                        top: Some(
                            "\
                            Just one Chess Piece left\n\
                            See below for a clue!",
                        ),
                        middle: None,
                        bottom: Some(bottom),
                    };

                    Self {
                        top: Some("One planet to go!"),
                        middle: None,
                        bottom: Some(bottom),
                    }
                }
                _ => {
                    //Completed
                    Self {
                        top: None,
                        middle: Some(
                            "\
                        Tap 'Next' for today's puzzle\n\
                        Open the menu for extra puzzles\n\
                        Why not try out Selfie Mode?",
                        ),

                        bottom: None,
                    }
                }
            }
        };

        Some(result)
    }
}

#[derive(Debug, EnumCount, EnumIter, EnumIs, PartialEq, Clone, Copy)]
pub enum TutorialLayoutEntity {
    Top,
    BigTop,
    Bottom,
}

const BOX_WIDTH: f32 = GRID_SIZE;

impl LayoutStructure for TutorialLayoutEntity {
    type Context<'a> = (SelfieMode, Insets);

    fn size(&self, _context: &Self::Context<'_>, _sizing: &LayoutSizing) -> bevy::prelude::Vec2 {
        match self {
            TutorialLayoutEntity::Top => Vec2 {
                x: BOX_WIDTH,
                y: 70.0,
            },
            TutorialLayoutEntity::BigTop => Vec2 {
                x: BOX_WIDTH,
                y: 105.0,
            },
            TutorialLayoutEntity::Bottom => Vec2 {
                x: BOX_WIDTH,
                y: 160.0,
            },
        }
    }

    fn location(&self, context: &Self::Context<'_>, sizing: &LayoutSizing) -> bevy::prelude::Vec2 {
        match self {
            TutorialLayoutEntity::Top => Vec2 {
                x: LEFT_MARGIN,
                y: GameLayoutEntity::LevelInfo.location(&context, sizing).y - 10.0,
            },
            TutorialLayoutEntity::BigTop => Vec2 {
                x: LEFT_MARGIN,
                y: GameLayoutEntity::LevelInfo.location(&context, sizing).y,
            },
            TutorialLayoutEntity::Bottom => Vec2 {
                x: LEFT_MARGIN,
                y: GameLayoutEntity::WordList.location(&context, sizing).y - 10.0,
            },
        }
    }

    fn iter_all(_context: &Self::Context<'_>) -> impl Iterator<Item = Self> {
        Self::iter()
    }
}

impl LayoutStructureWithFont for TutorialLayoutEntity {
    type FontContext = ();
    fn font_size(&self, _: &()) -> f32 {
        TUTORIAL_TEXT_FONT_SIZE
    }
}
