use crate::prelude::*;
use itertools::Itertools;
use maveric::{transition::speed::ScalarSpeed, widgets::text2d_node::Text2DNode};
#[derive(Debug, Clone, PartialEq)]
pub struct CongratsView;

pub const BUTTON_FONT_SIZE: f32 = 22.0;
pub const BUTTON_TEXT_COLOR: Color = Color::rgb(0.1, 0.1, 0.1);
pub const TEXT_BUTTON_WIDTH: f32 = 140.;
pub const TEXT_BUTTON_HEIGHT: f32 = 30.;
pub const UI_BORDER_WIDTH: Val = Val::Px(3.0);

impl MavericNode for CongratsView {
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
                    "time",
                    Text2DNode {
                        text: TextNode {
                            text: "06:09",
                            font_size: BUTTON_FONT_SIZE,
                            color: BUTTON_TEXT_COLOR,
                            font: BUTTONS_FONT_PATH,
                            alignment: TextAlignment::Center,
                            linebreak_behavior: bevy::text::BreakLineOn::NoWrap,
                        },
                        transform: Transform::from_translation(
                            size.get_rect(&CongratsLayoutEntity::Time)
                                .centre()
                                .extend(crate::z_indices::CONGRATS_BUTTON),
                        ),
                    },
                    &asset_server,
                );

                commands.add_child(
                    "share",
                    Text2DNode {
                        text: TextNode {
                            text: "Share",
                            font_size: BUTTON_FONT_SIZE,
                            color: BUTTON_TEXT_COLOR,
                            font: BUTTONS_FONT_PATH,
                            alignment: TextAlignment::Center,
                            linebreak_behavior: bevy::text::BreakLineOn::NoWrap,
                        },
                        transform: Transform::from_translation(
                            size.get_rect(&CongratsLayoutEntity::ShareButton)
                                .centre()
                                .extend(crate::z_indices::CONGRATS_BUTTON),
                        ),
                    },
                    &asset_server,
                );

                commands.add_child(
                    "next level",
                    Text2DNode {
                        text: TextNode {
                            text: "Next",
                            font_size: BUTTON_FONT_SIZE,
                            color: BUTTON_TEXT_COLOR,
                            font: BUTTONS_FONT_PATH,
                            alignment: TextAlignment::Center,
                            linebreak_behavior: bevy::text::BreakLineOn::NoWrap,
                        },
                        transform: Transform::from_translation(
                            size.get_rect(&CongratsLayoutEntity::NextButton)
                                .centre()
                                .extend(crate::z_indices::CONGRATS_BUTTON),
                        ),
                    },
                    &asset_server,
                );
            });
    }
}
