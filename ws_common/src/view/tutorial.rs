use std::time::Duration;

use bevy::{math::Vec2, text::Text2dBounds};
use itertools::Itertools;
use maveric::{
    helpers::{ChildCommands, SpatialBundle},
    node::MavericNode,
    with_bundle::CanWithBundle,
};
use strum::{EnumCount, EnumIs, EnumIter, IntoEnumIterator};
use ws_core::{layout::entities::*, LayoutStructure};

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
            let font_size = context
                .window_size
                .font_size(&TutorialLayoutEntity::Top, &());
            let font = TUTORIAL_FONT_PATH;

            if node.text.middle.is_none() {
                commands.add_child(
                    "title",
                    Text2DNode {
                        text: "Tutorial",
                        font: BOLD_FONT,
                        font_size: context.window_size.font_size(&TutorialTitleLayoutEntity, &()),
                        color: palette::TUTORIAL_TEXT_LINE2.convert_color(),
                        justify_text: JustifyText::Center,
                        linebreak_behavior: bevy::text::BreakLineOn::NoWrap,
                        text_anchor: bevy::sprite::Anchor::Center,
                        text_2d_bounds: Default::default(),
                    }
                    .with_bundle(Transform::from_translation(
                        context
                            .window_size
                            .get_origin(&TutorialTitleLayoutEntity, &(context.video_resource.selfie_mode(), context.insets.0))
                            .extend(crate::z_indices::TUTORIAL_POPUP_BOX_TEXT),
                    )),
                    &(),
                )
            }

            if let [Some(text1), maybe_text2] = node.text.top {
                commands.add_child(
                    "top",
                    TutorialPopupNode {
                        sections: [
                            Some(TextSectionData {
                                text: text1,
                                font,
                                font_size,
                                color: palette::TUTORIAL_TEXT_LINE1.convert_color(),
                            }),
                            maybe_text2.map(|t| TextSectionData {
                                text: t,
                                font,
                                font_size,
                                color: palette::TUTORIAL_TEXT_LINE2.convert_color(),
                            }),
                        ],
                        entity: TutorialLayoutEntity::Top,
                        //color: palette::TUTORIAL_TOP_TEXT.convert_color(),
                        align_left: true,
                        middle_transition: false,
                    },
                    context,
                );
            }
            if let Some(text) = node.text.middle {
                commands.add_child(
                    "middle",
                    TutorialPopupNode {
                        entity: TutorialLayoutEntity::Middle,
                        sections: [
                            Some(TextSectionData {
                                text,
                                font,
                                font_size,
                                color: palette::TUTORIAL_MIDDLE_TEXT.convert_color(),
                            }),
                            None,
                        ],
                        align_left: false,
                        middle_transition: true,
                    },
                    context,
                );
            }
            if let [Some(text1), maybe_text2] = node.text.bottom {
                commands.add_child(
                    "bottom",
                    TutorialPopupNode {
                        entity: TutorialLayoutEntity::Bottom,
                        sections: [
                            Some(TextSectionData {
                                text: text1,
                                font,
                                font_size,
                                color: palette::TUTORIAL_TEXT_LINE1.convert_color(),
                            }),
                            maybe_text2.map(|t| TextSectionData {
                                text: t,
                                font,
                                font_size,
                                color: palette::TUTORIAL_TEXT_LINE2.convert_color(),
                            }),
                        ],
                        align_left: true,
                        middle_transition: false,
                    },
                    context,
                );
            }
        });
    }
}

#[derive(Debug, PartialEq)]
pub struct TutorialPopupNode {
    pub sections: [Option<TextSectionData<&'static str>>; 2],
    pub entity: TutorialLayoutEntity,
    pub align_left: bool,
    pub middle_transition: bool,
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
                sections,
                entity,
                align_left,
                middle_transition,
            } = node;

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

            let transition_in = if *middle_transition {
                TransitionBuilder::<TransformScaleLens>::default()
                    .then_set_value(Vec3::ZERO)
                    .then_wait(Duration::from_secs_f32(TRANSITION_WAIT_SECS))
                    .then_ease_with_duration(
                        Vec3::ONE,
                        Duration::from_secs_f32(TRANSITION_SECS),
                        Ease::CubicOut,
                    )
                    .build()
            } else {
                TransitionBuilder::<TransformScaleLens>::default()
                    .then_set_value(Vec3 {
                        x: 1.0,
                        y: 0.0,
                        z: 1.0,
                    })
                    .then_ease_with_duration(
                        Vec3::ONE,
                        Duration::from_secs_f32(POPUP_TRANSITION_IN_SECONDS),
                        Ease::CubicOut,
                    )
                    .build()
            };

            let transition_out = if *middle_transition {
                TransitionBuilder::<TransformScaleLens>::default()
                    .then_set_value(Vec3::ZERO)
                    .build()
            } else {
                TransitionBuilder::<TransformScaleLens>::default()
                    .then_set_value(Vec3::ONE)
                    .then_ease_with_duration(
                        Vec3 {
                            x: 1.0,
                            y: 0.0,
                            z: 1.0,
                        },
                        Duration::from_secs_f32(POPUP_TRANSITION_OUT_SECONDS),
                        Ease::CubicOut,
                    )
                    .build()
            };

            let text = MultiText2DNode {
                sections: sections.clone(),
                justify_text,
                linebreak_behavior: bevy::text::BreakLineOn::NoWrap,
                text_2d_bounds: Text2dBounds::default(),
                text_anchor,
            }
            .with_bundle(Transform::from_translation(
                translation.extend(crate::z_indices::TUTORIAL_POPUP_BOX_TEXT),
            ))
            .with_transition(Vec3::ZERO, transition_in, transition_out);

            commands.add_child("title text", text, &());
        });
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TutorialTitleLayoutEntity;

impl LayoutStructure for TutorialTitleLayoutEntity {
    type Context<'a> = (SelfieMode, Insets);

    fn size(&self, _context: &Self::Context<'_>, _sizing: &LayoutSizing) -> Vec2 {
        Vec2 {
            x: TIMER_WIDTH,
            y: TIMER_HEIGHT,
        }
    }
    fn location(&self, context: &Self::Context<'_>, sizing: &LayoutSizing) -> Vec2 {
        Vec2 {
            x: (IDEAL_WIDTH - TIMER_WIDTH) * 0.5,
            y: GameLayoutEntity::TopBar.location(context, sizing).y + (WORD_SALAD_LOGO_SIZE / 2.),
        }
    }

    fn iter_all(_context: &Self::Context<'_>) -> impl Iterator<Item = Self> {
        [Self].into_iter()
    }
}

impl LayoutStructureWithOrigin for TutorialTitleLayoutEntity{
    fn origin(&self, _context: &Self::Context<'_>, _sizing: &LayoutSizing)-> Origin {
        Origin::TopCenter
    }
}

impl LayoutStructureWithFont for TutorialTitleLayoutEntity {
    type FontContext = ();
    fn font_size(&self, _: &()) -> f32 {
        TUTORIAL_TITLE_FONT_SIZE
    }
}

#[derive(Debug, EnumCount, EnumIter, EnumIs, PartialEq, Clone, Copy)]
pub enum TutorialLayoutEntity {
    Top,
    Middle,
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
            TutorialLayoutEntity::Middle => Vec2 {
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
                y: GameLayoutEntity::Grid.location(context, sizing).y
                    - self.size(context, sizing).y -10.0,
            },
            TutorialLayoutEntity::Middle => Vec2 {
                x: LEFT_MARGIN,
                y: GameLayoutEntity::LevelInfo.location(context, sizing).y,
            },
            TutorialLayoutEntity::Bottom => Vec2 {
                x: LEFT_MARGIN,
                y: GameLayoutEntity::WordList.location(context, sizing).y ,
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

#[derive(Debug, PartialEq)]
pub struct TutorialText {
    top: [Option<&'static str>; 2],
    middle: Option<&'static str>,
    bottom: [Option<&'static str>; 2],
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
                    top: [
                        Some("Let's start by finding 'Pawn'\n"),
                        Some("Make words by linking tiles"),
                    ],
                    middle: None,
                    bottom: [None; 2],
                },
                1 => {
                    let found_index = found_words
                        .word_completions
                        .iter()
                        .find_position(|completion| completion.is_complete())
                        .map(|x| x.0)
                        .unwrap_or_default();

                    let bottom = match found_index {
                        4 => [
                            Some("These labels show the word lengths\n"),
                            Some("The first 6-letter word is 'Bishop'"),
                        ],
                        _ => [
                            Some("These labels show the word lengths\n"),
                            Some("The 5-letter word is 'Queen'"),
                        ],
                    };

                    Self {
                        top: [
                            Some("Letters vanish when no longer needed\n"),
                            Some("Every remaining letter is needed"),
                        ],
                        middle: None,
                        bottom,
                    }
                }
                2 => Self {
                    top: [Some("Find the final three Chess Pieces"), None],
                    middle: None,
                    bottom: [None; 2],
                },
                3 => Self {
                    top: [None; 2],
                    middle: None,
                    bottom: [
                        Some("Labels are listed alphabetically\n"),
                        Some("This can help you find first letters"),
                    ],
                },
                4 => {
                    let incomplete_index = found_words
                        .word_completions
                        .iter()
                        .find_position(|completion| !completion.is_complete())
                        .map(|x| x.0)
                        .unwrap_or_default();

                    let hint = match incomplete_index {
                        0 =>
                        //bishop
                        {
                            "This word starts with 'B', 'H', or 'I'"
                        }

                        1 =>
                        //king
                        {
                            "This word can't start with an 'N'"
                        }

                        2 =>
                        //knight
                        {
                            "This word must start with 'K' or 'N'"
                        }

                        3 =>
                        //pawn
                        {
                            "This word must start with 'P' or 'N'"
                        }

                        _ =>
                        // queen
                        {
                            "This word must start with 'Q'"
                        }
                    };

                    Self {
                        top: [
                            Some("Just one Chess Piece left\n"),
                            Some("See below for a clue!"),
                        ],
                        middle: None,
                        bottom: [Some("The labels are listed alphabetically\n"), Some(hint)],
                    }
                }

                _ => {
                    //Completed
                    Self {
                        top: [None; 2],
                        middle: Some(
                            "You completed your first Word Salad\nThe next puzzle is about Planets",
                        ),
                        bottom: [None; 2],
                    }
                }
            }
        } else {
            //Planets
            match completed_words {
                0 => Self {
                    top: [
                        Some("Find six planets\n"),
                        Some("The 4-letter planet is 'Mars'"),
                    ],
                    middle: None,
                    bottom: [None; 2],
                },
                1 => Self {
                    top: [Some("Find the other planets"), None],
                    middle: None,
                    bottom: [
                        Some("Want help? Use a hint\n"),
                        Some("Press a label to reveal a letter"),
                    ],
                },
                2 => Self {
                    top: [Some("Hints are free in the tutorial"), None],
                    middle: None,
                    bottom: [
                        Some("Want help? Use a hint\n"),
                        Some("Press a label to reveal a letter"),
                    ],
                },
                3..=4 => Self {
                    top: [Some("Hints are free in the tutorial"), None],
                    middle: None,
                    bottom: [None; 2],
                },
                5 => {
                    let incomplete_index = found_words
                        .word_completions
                        .iter()
                        .find_position(|completion| !completion.is_complete())
                        .map(|x| x.0)
                        .unwrap_or_default();

                    let hint = match incomplete_index {
                        0 =>
                        // Mars
                        {
                            "This word starts with 'A' or 'MA'"
                        }

                        1 =>
                        // Mercury
                        {
                            "This word starts with 'M'"
                        }

                        2 =>
                        // Neptune
                        {
                            "This word must start with 'N' or 'P'"
                        }

                        3 =>
                        // Saturn
                        {
                            "This word cannot start with 'A'"
                        }

                        4 =>
                        // Uranus
                        {
                            "This word must start with 'S' or 'U'"
                        }

                        _ =>
                        // Venus
                        {
                            "This word must start with 'U' or 'V'"
                        }
                    };

                    Self {
                        top: [Some("One planet to go!"), None],
                        middle: None,
                        bottom: [Some("The labels are listed alphabetically\n"), Some(hint)],
                    }
                }
                _ => {
                    //Completed
                    Self {
                        top: [None; 2],
                        middle: Some(
                            "\
                        Tap 'Next' for today's puzzle\n\
                        Open the menu for extra puzzles\n\
                        Why not try out Selfie Mode?",
                        ),

                        bottom: [None; 2],
                    }
                }
            }
        };

        Some(result)
    }
}
