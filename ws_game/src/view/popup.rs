use crate::{
    prelude::*,
    shapes,
    z_indices::{self, POPUP_BOX_TEXT}, rounding::OTHER_BUTTON_NORMAL,
};
use maveric::{widgets::text2d_node::Text2DNode, with_bundle::CanWithBundle};
use strum::{Display, EnumCount, EnumIter, IntoEnumIterator};
use ws_core::{
    layout::entities::*,
    palette::{POPUP_BOX_BACKGROUND, POPUP_BOX_BORDER},
    LayoutStructure, LayoutStructureWithFont, Spacing,
};

pub struct PopupPlugin;

impl Plugin for PopupPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(PopupState::default());
        app.register_maveric::<PopupStateRoot>();
    }
}

#[derive(Debug, Clone, Copy, Resource, MavericContext, PartialEq, Eq, Default)]
pub enum PopupState {
    #[default]
    None,
    BuyMoreHints,
}

#[derive(Debug, PartialEq, Clone, Copy, Eq, MavericRoot)]
pub struct PopupStateRoot;

impl MavericRootChildren for PopupStateRoot {
    type Context = (MyWindowSize, PopupState);

    fn set_children(
        context: &<Self::Context as NodeContext>::Wrapper<'_>,
        commands: &mut impl ChildCommands,
    ) {
        let size = &context.0;
        match context.1.as_ref() {
            PopupState::None => {}
            PopupState::BuyMoreHints => {
                commands.add_child(
                    "grey out",
                    shapes::box_node1(
                        size.scaled_width,
                        size.scaled_height,
                        Vec3::Z * z_indices::POPUP_BOX_GREY_OUT,
                        Color::GRAY.with_a(0.9),
                        0.0,
                    )
                    .with_transition_in_out::<SmudColorLens>(
                        Color::GRAY.with_a(0.0),
                        Color::GRAY.with_a(0.9),
                        Color::GRAY.with_a(0.0),
                        core::time::Duration::from_millis(500),
                        core::time::Duration::from_millis(500),
                    ),
                    &(),
                );

                for item in BuyMoreHintsLayoutEntity::iter() {
                    let font_size = size.font_size::<BuyMoreHintsLayoutEntity>(&item);
                    let rect: LayoutRectangle = size.get_rect(&item, &());
                    match item {
                        BuyMoreHintsLayoutEntity::Title => {
                            let text = Text2DNode {
                                text: "Need a Hint?",
                                font: TITLE_FONT_PATH,
                                font_size,
                                color: Color::BLACK,
                                alignment: TextAlignment::Center,
                                linebreak_behavior: bevy::text::BreakLineOn::NoWrap,
                                text_2d_bounds: Default::default(),
                                text_anchor: Default::default(),
                            }
                            .with_bundle(Transform::from_translation(
                                rect.centre().extend(POPUP_BOX_TEXT),
                            ));

                            commands.add_child("title text", text, &());
                        }
                        BuyMoreHintsLayoutEntity::BuyMoreHintsButton => {
                            let button = shapes::box_node1(
                                rect.width(),
                                rect.height(),
                                rect.centre().extend(crate::z_indices::POPUP_BOX_BUTTON),
                                Color::BLUE,
                                OTHER_BUTTON_NORMAL,
                            );

                            commands.add_child(
                                "buy_mode_hints_box",
                                button.with_bundle(ButtonInteraction::BuyMoreHints),
                                &(),
                            );

                            let text = Text2DNode {
                                text: "Buy more",
                                font: BUTTONS_FONT_PATH,
                                font_size,
                                color: Color::BLACK,
                                alignment: TextAlignment::Center,
                                linebreak_behavior: bevy::text::BreakLineOn::NoWrap,
                                text_2d_bounds: Default::default(),
                                text_anchor: Default::default(),
                            }
                            .with_bundle(Transform::from_translation(
                                rect.centre().extend(POPUP_BOX_TEXT),
                            ));

                            commands.add_child("buy_mode_hints_text", text, &());
                        }
                        BuyMoreHintsLayoutEntity::SufferAloneButton => {
                            let button = shapes::box_node1(
                                rect.width(),
                                rect.height(),
                                rect.centre().extend(crate::z_indices::POPUP_BOX_BUTTON),
                                Color::GRAY,
                                OTHER_BUTTON_NORMAL,
                            )
                            .with_bundle(ButtonInteraction::ClosePopups);

                            commands.add_child("suffer_alone_box", button, &());

                            let text = Text2DNode {
                                text: "Suffer Alone",
                                font: BUTTONS_FONT_PATH,
                                font_size,
                                color: Color::BLACK,
                                alignment: TextAlignment::Center,
                                linebreak_behavior: bevy::text::BreakLineOn::NoWrap,
                                text_2d_bounds: Default::default(),
                                text_anchor: Default::default(),
                            }
                            .with_bundle(Transform::from_translation(
                                rect.centre().extend(POPUP_BOX_TEXT),
                            ));

                            commands.add_child("suffer_alone_text", text, &());
                        }
                        BuyMoreHintsLayoutEntity::Box => {
                            let node = shapes::box_with_border_node(
                                rect.width(),
                                rect.height(),
                                rect.centre().extend(crate::z_indices::POPUP_BOX_BACKGROUND),
                                POPUP_BOX_BACKGROUND.convert_color(),
                                POPUP_BOX_BORDER.convert_color(),
                                0.1,
                                0.01,
                            );

                            commands.add_child("background", node, &())
                        }
                    }
                }
            }
        }
    }
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, EnumIter, EnumCount, Display,
)]
pub enum BuyMoreHintsLayoutEntity {
    Title = 0,

    BuyMoreHintsButton = 1,
    SufferAloneButton = 2,
    Box = 3,
}

impl BuyMoreHintsLayoutEntity {
    pub fn index(&self) -> usize {
        *self as usize
    }
}

impl LayoutStructure for BuyMoreHintsLayoutEntity {
    type Context = ();
    type Iterator = <Self as IntoEnumIterator>::Iterator;

    fn pick(point: Vec2, context: &Self::Context) -> Option<Self> {
        Self::iter().find(|&x| x.rect(context).contains(point))
    }

    fn size(&self, _context: &Self::Context) -> Vec2 {
        use BuyMoreHintsLayoutEntity::*;
        match self {
            Title => Vec2 {
                x: HINTS_POPUP_BOX_TITLE_WIDTH,
                y: HINTS_POPUP_BOX_TITLE_HEIGHT,
            },
            BuyMoreHintsButton => Vec2 {
                x: HINTS_POPUP_BOX_BUTTON_WIDTH,
                y: HINTS_POPUP_BOX_BUTTON_HEIGHT,
            },
            SufferAloneButton => Vec2 {
                x: HINTS_POPUP_BOX_BUTTON_WIDTH,
                y: HINTS_POPUP_BOX_BUTTON_HEIGHT,
            },
            Box => Vec2 {
                x: HINTS_POPUP_BOX_WIDTH,
                y: HINTS_POPUP_BOX_HEIGHT,
            },
        }
    }

    fn location(&self, _context: &Self::Context) -> Vec2 {
        use BuyMoreHintsLayoutEntity::*;
        match self {
            Title | BuyMoreHintsButton | SufferAloneButton => Vec2 {
                x: (IDEAL_WIDTH - HINTS_POPUP_BOX_TITLE_WIDTH) / 2.,
                y: HINTS_POPUP_BOX_TOP
                    + Spacing::Centre.apply(
                        HINTS_POPUP_BOX_HEIGHT,
                        HINTS_POPUP_BOX_TITLE_HEIGHT * 1.2,
                        Self::COUNT - 1,
                        self.index(),
                    ),
            },
            Box => Vec2 {
                x: (IDEAL_WIDTH - HINTS_POPUP_BOX_WIDTH) / 2.0,
                y: HINTS_POPUP_BOX_TOP,
            },
        }
    }

    fn iter_all(_context: &Self::Context) -> Self::Iterator {
        Self::iter()
    }
}

impl LayoutStructureWithFont for BuyMoreHintsLayoutEntity {
    fn font_size(&self) -> f32 {
        MENU_BUTTON_FONT_SIZE
    }
}
