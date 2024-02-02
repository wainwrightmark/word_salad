use crate::{
    prelude::*,
    rounding::OTHER_BUTTON_NORMAL,
    shapes,
    z_indices::{self, POPUP_BOX_TEXT},
};
use maveric::{widgets::text2d_node::Text2DNode, with_bundle::CanWithBundle};
use strum::{Display, EnumCount, EnumIs, EnumIter, IntoEnumIterator};
use ws_core::{
    layout::entities::*,
    palette::{BUTTON_CLICK_FILL, POPUP_BOX_BACKGROUND, POPUP_BOX_BORDER},
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
pub struct PopupState(pub Option<PopupType>);

#[derive(Debug, Clone, Copy, Resource, MavericContext, PartialEq, Eq, EnumIs)]
pub enum PopupType {
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

        let Some(popup_type) = context.1 .0 else {
            return;
        };

        commands.add_child(
            "grey out",
            shapes::basic_box_node1(
                size.scaled_width.max(size.scaled_height),
                size.scaled_width.max(size.scaled_height),
                Vec3::Z * z_indices::POPUP_BOX_GREY_OUT,
                Color::GRAY.with_a(0.9),
                0.0,
            )
            .with_transition_in_out::<ShaderColorLens>(
                Color::GRAY.with_a(0.0),
                Color::GRAY.with_a(0.9),
                Color::GRAY.with_a(0.0),
                core::time::Duration::from_millis(500),
                core::time::Duration::from_millis(500),
                Some(Ease::CubicOut),
                Some(Ease::CubicOut),
            ),
            &(),
        );

        match popup_type {
            PopupType::BuyMoreHints => {
                for item in HintsPopupLayoutEntity::iter() {
                    let font_size = size.font_size::<HintsPopupLayoutEntity>(&item, &());
                    let rect: LayoutRectangle = size.get_rect(&item, &());
                    //info!("{rect:?}");
                    match item {
                        HintsPopupLayoutEntity::Text => {
                            let text = "Need some help?";

                            let text_node = Text2DNode {
                                text,
                                font: POPUP_FONT_PATH,
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

                            commands.add_child("title text", text_node, &());
                        }
                        HintsPopupLayoutEntity::BuyMoreButton => {
                            let button = shapes::button_box_node(
                                rect.width(),
                                rect.height(),
                                rect.centre().extend(crate::z_indices::POPUP_BOX_BUTTON),
                                palette::MENU_BUTTON_FILL.convert_color(),
                                BUTTON_CLICK_FILL.convert_color(),
                                OTHER_BUTTON_NORMAL,
                                ShaderBorder::NONE,
                                ButtonInteraction::Popup(PopupInteraction::HintsBuyMore),
                            );

                            commands.add_child("buy_more_hints_box", button, &());

                            let text = "Buy More Hints";

                            let text_node = Text2DNode {
                                text,
                                font: BUTTONS_FONT_PATH,
                                font_size,
                                color: palette::MENU_BUTTON_TEXT_REGULAR.convert_color(),
                                alignment: TextAlignment::Center,
                                linebreak_behavior: bevy::text::BreakLineOn::NoWrap,
                                text_2d_bounds: Default::default(),
                                text_anchor: Default::default(),
                            }
                            .with_bundle(Transform::from_translation(
                                rect.centre().extend(POPUP_BOX_TEXT),
                            ));

                            commands.add_child("buy_more_hints_text", text_node, &());
                        }
                        HintsPopupLayoutEntity::SufferAloneButton => {
                            let button = shapes::button_box_node(
                                rect.width(),
                                rect.height(),
                                rect.centre().extend(crate::z_indices::POPUP_BOX_BUTTON),
                                palette::MENU_BUTTON_DISCOURAGED_FILL.convert_color(),
                                BUTTON_CLICK_FILL.convert_color(),
                                OTHER_BUTTON_NORMAL,
                                ShaderBorder::NONE,
                                ButtonInteraction::Popup(PopupInteraction::ClickClose),
                            );

                            commands.add_child("secondary_action_box", button, &());

                            let text = "Suffer Alone";

                            let text_box = Text2DNode {
                                text,
                                font: BUTTONS_FONT_PATH,
                                font_size,
                                color: palette::MENU_BUTTON_TEXT_DISCOURAGED.convert_color(),
                                alignment: TextAlignment::Center,
                                linebreak_behavior: bevy::text::BreakLineOn::NoWrap,
                                text_2d_bounds: Default::default(),
                                text_anchor: Default::default(),
                            }
                            .with_bundle(Transform::from_translation(
                                rect.centre().extend(POPUP_BOX_TEXT),
                            ));

                            commands.add_child("secondary_action_text", text_box, &());
                        }
                        HintsPopupLayoutEntity::PopupBox => {
                            let node = shapes::box_with_border_node(
                                rect.width(),
                                rect.height(),
                                rect.centre().extend(crate::z_indices::POPUP_BOX_BACKGROUND),
                                POPUP_BOX_BACKGROUND.convert_color(),
                                0.1,
                                ShaderBorder {
                                    border_color: POPUP_BOX_BORDER.convert_color(),
                                    border: 0.01,
                                },
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
    Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, EnumCount, Display, EnumIter,
)]
pub enum HintsPopupLayoutEntity {
    Text = 0,
    BuyMoreButton = 1,
    SufferAloneButton = 2,
    PopupBox = 3,
}

impl HintsPopupLayoutEntity {
    pub fn index(&self) -> usize {
        *self as usize
    }
}

impl LayoutStructure for HintsPopupLayoutEntity {
    type Context<'a> = ();

    fn size(&self, _context: &Self::Context<'_>, _sizing: &LayoutSizing) -> Vec2 {
        match self {
            Self::Text => Vec2 {
                x: HINTS_POPUP_BOX_TITLE_WIDTH,
                y: HINTS_POPUP_BOX_TITLE_HEIGHT,
            },
            Self::BuyMoreButton | Self::SufferAloneButton => Vec2 {
                x: HINTS_POPUP_BOX_BUTTON_WIDTH,
                y: HINTS_POPUP_BOX_BUTTON_HEIGHT,
            },
            Self::PopupBox => Vec2 {
                x: HINTS_POPUP_BOX_WIDTH,
                y: HINTS_POPUP_BOX_HEIGHT,
            },
        }
    }

    fn location(&self, _context: &Self::Context<'_>, _sizing: &LayoutSizing) -> Vec2 {
        match self {
            Self::Text | Self::BuyMoreButton | Self::SufferAloneButton => Vec2 {
                x: (IDEAL_WIDTH - HINTS_POPUP_BOX_TITLE_WIDTH) / 2.,
                y: HINTS_POPUP_BOX_TOP
                    + Spacing::Centre.apply(
                        HINTS_POPUP_BOX_HEIGHT,
                        HINTS_POPUP_BOX_TITLE_HEIGHT * 1.2,
                        Self::COUNT - 1,
                        self.index(),
                    ),
            },
            Self::PopupBox => Vec2 {
                x: (IDEAL_WIDTH - HINTS_POPUP_BOX_WIDTH) / 2.0,
                y: HINTS_POPUP_BOX_TOP,
            },
        }
    }

    fn iter_all(_context: &Self::Context<'_>) -> impl Iterator<Item = Self> {
        Self::iter()
    }
}

impl LayoutStructureWithFont for HintsPopupLayoutEntity {
    type FontContext = ();
    fn font_size(&self, _context: &Self::FontContext) -> f32 {
        MENU_BUTTON_FONT_SIZE
    }
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, EnumCount, Display, EnumIter,
)]
pub enum SelfiePopupLayoutEntity {
    Text = 0,
    MoreInformationButton = 1,
    OkButton = 2,
    DontShowAgainButton = 3,
    PopupBox = 4,
}

impl SelfiePopupLayoutEntity {
    pub fn index(&self) -> usize {
        *self as usize
    }
}

impl LayoutStructure for SelfiePopupLayoutEntity {
    type Context<'a> = ();

    fn size(&self, _context: &Self::Context<'_>, _sizing: &LayoutSizing) -> Vec2 {
        match self {
            Self::Text => Vec2 {
                x: SELFIE_POPUP_BOX_TITLE_WIDTH,
                y: SELFIE_POPUP_BOX_TITLE_HEIGHT,
            },
            Self::MoreInformationButton | Self::OkButton | Self::DontShowAgainButton => Vec2 {
                x: SELFIE_POPUP_BOX_BUTTON_WIDTH,
                y: SELFIE_POPUP_BOX_BUTTON_HEIGHT,
            },
            Self::PopupBox => Vec2 {
                x: SELFIE_POPUP_BOX_WIDTH,
                y: SELFIE_POPUP_BOX_HEIGHT,
            },
        }
    }

    fn location(&self, _context: &Self::Context<'_>, _sizing: &LayoutSizing) -> Vec2 {
        match self {
            Self::Text => Vec2 {
                x: (IDEAL_WIDTH - SELFIE_POPUP_BOX_TITLE_WIDTH) / 2.,
                y: SELFIE_POPUP_BOX_TOP + (SELFIE_POPUP_BOX_TITLE_HEIGHT * 0.2),
            },
            Self::MoreInformationButton | Self::OkButton | Self::DontShowAgainButton => Vec2 {
                x: (IDEAL_WIDTH - SELFIE_POPUP_BOX_TITLE_WIDTH) / 2.,
                y: SELFIE_POPUP_BOX_TOP
                    + (SELFIE_POPUP_BOX_TITLE_HEIGHT * 1.4)
                    + Spacing::Centre.apply(
                        SELFIE_POPUP_BOX_HEIGHT - (SELFIE_POPUP_BOX_TITLE_HEIGHT * 1.4),
                        SELFIE_POPUP_BOX_BUTTON_HEIGHT * 1.2,
                        Self::COUNT - 2,
                        self.index() - 1,
                    ),
            },
            Self::PopupBox => Vec2 {
                x: (IDEAL_WIDTH - SELFIE_POPUP_BOX_WIDTH) / 2.0,
                y: SELFIE_POPUP_BOX_TOP,
            },
        }
    }

    fn iter_all(_context: &Self::Context<'_>) -> impl Iterator<Item = Self> {
        Self::iter()
    }
}

impl LayoutStructureWithFont for SelfiePopupLayoutEntity {
    type FontContext = ();
    fn font_size(&self, _context: &Self::FontContext) -> f32 {
        MENU_BUTTON_FONT_SIZE_SMALL
    }
}
