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
    layout::entities::{GRID_SIZE, IDEAL_WIDTH},
    LayoutStructure,
};

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
                commands.add_child(
                    "top",
                    TutorialPopupNode {
                        text,
                        entity: TutorialLayoutEntity::Top,
                    }
                    .with_transition_in::<TransformScaleLens>(
                        Vec3::ZERO,
                        Vec3::ONE,
                        Duration::from_secs_f32(0.5),
                    ),
                    context,
                );
            }
            if let Some(text) = node.text.middle {
                commands.add_child(
                    "middle",
                    TutorialPopupNode {
                        text,
                        entity: TutorialLayoutEntity::Middle,
                    }
                    .with_transition_in::<TransformScaleLens>(
                        Vec3::ZERO,
                        Vec3::ONE,
                        Duration::from_secs_f32(0.5),
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
                    }
                    .with_transition_in::<TransformScaleLens>(
                        Vec3::ZERO,
                        Vec3::ONE,
                        Duration::from_secs_f32(0.5),
                    ),
                    context,
                );
            }
        });
    }
}

#[derive(Debug, PartialEq)]
struct TutorialPopupNode {
    text: &'static str,
    entity: TutorialLayoutEntity,
}

impl MavericNode for TutorialPopupNode {
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
        commands.unordered_children_with_node_and_context(|node, context, commands| {
            let TutorialPopupNode { text, entity } = node;

            let rect = context.get_rect(entity, &());
            let font_size = context.font_size(&TutorialTextLayoutEntity(*entity), &());
            let text_rect = context.get_rect(&TutorialTextLayoutEntity(*entity), &());

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
                font_size: font_size,
                color: Color::BLACK,
                alignment: TextAlignment::Left,
                linebreak_behavior: bevy::text::BreakLineOn::WordBoundary,
                text_2d_bounds: Text2dBounds::default(),
                text_anchor: bevy::sprite::Anchor::CenterLeft,
            }
            .with_bundle(Transform::from_translation(
                text_rect
                    .centre_left()
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
                            You've earned two hints\n\
                            Spend a hint to reveal a letter",
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
                    To beat this puzzle, find six planets\n\
                    The 4-letter word is 'Mars'",
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
                    Click a label to reveal a letter", //TODO actually do one
                    ),
                },
                2 => Self {
                    top: None,
                    middle: None,
                    bottom: Some(
                        "\
                        \n\
                    Want help? Use a hint\n\
                    Click a label to reveal a letter", //TODO actually do one
                    ),
                },
                3..=4 => Self {
                    top: Some("Your remaining hints are shown\nIn the green circle"),
                    middle: None,
                    bottom: Some(
                        "\
                        \n\
                    You earn hints by completing levels\n\
                    Don't be afraid to spend them!",
                    ),
                },
                5 => Self {
                    top: Some("One planet to go!"),
                    middle: None,
                    bottom: None,
                },
                _ => {
                    //Completed
                    Self {
                        top: None,
                        middle: Some(
                            "\
                        Tap 'Word Salad' for today's puzzle\n\
                        Open the menu for extra puzzles\n\
                        Why not try out Selfie Mode?",
                        ),

                        bottom: None,
                    }
                }
            }
        };

        return Some(result);
    }
}

#[derive(Debug, EnumCount, EnumIter, EnumIs, PartialEq, Clone, Copy)]
enum TutorialLayoutEntity {
    Top,
    Middle,
    Bottom,
}

const BOX_WIDTH: f32 = GRID_SIZE + 20.0;

impl LayoutStructure for TutorialLayoutEntity {
    type Context = ();

    fn size(&self, _context: &Self::Context) -> bevy::prelude::Vec2 {
        match self {
            TutorialLayoutEntity::Top => Vec2 {
                x: BOX_WIDTH,
                y: 70.0,
            },
            TutorialLayoutEntity::Middle => Vec2 {
                x: BOX_WIDTH,
                y: 105.0,
            },
            TutorialLayoutEntity::Bottom => Vec2 {
                x: BOX_WIDTH,
                y: 140.0,
            },
        }
    }

    fn location(&self, _context: &Self::Context) -> bevy::prelude::Vec2 {
        match self {
            TutorialLayoutEntity::Top => Vec2 {
                x: (IDEAL_WIDTH - BOX_WIDTH) * 0.5,
                y: 52.0,
            },
            TutorialLayoutEntity::Middle => Vec2 {
                x: (IDEAL_WIDTH - BOX_WIDTH) * 0.5,
                y: 70.0,
            },
            TutorialLayoutEntity::Bottom => Vec2 {
                x: (IDEAL_WIDTH - BOX_WIDTH) * 0.5,
                y: 410.0,
            },
        }
    }

    fn iter_all(_context: &Self::Context) -> impl Iterator<Item = Self> {
        Self::iter()
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct TutorialTextLayoutEntity(TutorialLayoutEntity);

const TEXT_LEFT_MARGIN: f32 = 15.0;
const TEXT_RIGHT_MARGIN: f32 = 5.0;
const BOTTOM_TEXT_TOP_OFFSET: f32 = 40.0;

impl LayoutStructure for TutorialTextLayoutEntity {
    type Context = ();

    fn size(&self, context: &Self::Context) -> Vec2 {
        let x = TEXT_LEFT_MARGIN + TEXT_RIGHT_MARGIN;
        let y = if self.0.is_bottom() {
            BOTTOM_TEXT_TOP_OFFSET
        } else {
            0.0
        };
        self.0.size(context) - Vec2 { x, y }
    }

    fn location(&self, context: &Self::Context) -> Vec2 {
        let x = TEXT_LEFT_MARGIN; //note this is different from in 'size'
        let y = if self.0.is_bottom() {
            BOTTOM_TEXT_TOP_OFFSET
        } else {
            0.0
        };
        self.0.location(context) + Vec2 { x, y }
    }

    fn iter_all(_context: &Self::Context) -> impl Iterator<Item = Self> {
        TutorialLayoutEntity::iter().map(|x| Self(x))
    }
}

impl LayoutStructureWithFont for TutorialTextLayoutEntity {
    type FontContext = ();
    fn font_size(&self,_: &()) -> f32 {
        30.0
    }
}
