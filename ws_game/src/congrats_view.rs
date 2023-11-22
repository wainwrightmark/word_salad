use crate::prelude::*;
use maveric::widgets::text2d_node::Text2DNode;
use ws_core::layout::entities::*;
#[derive(Debug, Clone, PartialEq)]
pub struct CongratsView;


impl MavericNode for CongratsView {
    type Context = ViewContext;

    fn set_components(commands: SetComponentCommands<Self, Self::Context>) {
        commands
            .ignore_context()
            .ignore_node()
            .insert(SpatialBundle::default())
            .finish()
    }

    fn set_children<R: MavericRoot>(commands: SetChildrenCommands<Self, Self::Context, R>) {
        commands
            .ignore_node()
            .unordered_children_with_context(|context, commands| {
                let size = &context.3;
                let font_size = size.font_size::<CongratsLayoutEntity>();

                let hints_used_text = match context.2.hints_used{
                    0 => "No hints used".to_string(),
                    1 => "1 hint used".to_string(),
                    n => format!("{n} hints used")

                };

                commands.add_child(
                    "hints used",
                    Text2DNode {
                        text: TextNode {
                            text: hints_used_text,
                            font_size,
                            color: convert_color(palette::BUTTON_TEXT_COLOR),
                            font: BUTTONS_FONT_PATH,
                            alignment: TextAlignment::Center,
                            linebreak_behavior: bevy::text::BreakLineOn::NoWrap,
                        },
                        transform: Transform::from_translation(
                            size.get_rect(&CongratsLayoutEntity::HintsUsed, &())
                                .centre()
                                .extend(crate::z_indices::CONGRATS_BUTTON),
                        ),
                    },
                    &(),
                );

                commands.add_child(
                    "next level",
                    Text2DNode {
                        text: TextNode {
                            text: "Next",
                            font_size,
                            color: convert_color(palette::BUTTON_TEXT_COLOR),
                            font: BUTTONS_FONT_PATH,
                            alignment: TextAlignment::Center,
                            linebreak_behavior: bevy::text::BreakLineOn::NoWrap,
                        },
                        transform: Transform::from_translation(
                            size.get_rect(&CongratsLayoutEntity::NextButton, &())
                                .centre()
                                .extend(crate::z_indices::CONGRATS_BUTTON),
                        ),
                    },
                    &(),
                );

                #[cfg(target_arch = "wasm32")]
                {
                    commands.add_child(
                        "share",
                        Text2DNode {
                            text: TextNode {
                                text: "Share",
                                font_size,
                                color: convert_color(palette::BUTTON_TEXT_COLOR),
                                font: BUTTONS_FONT_PATH,
                                alignment: TextAlignment::Center,
                                linebreak_behavior: bevy::text::BreakLineOn::NoWrap,
                            },
                            transform: Transform::from_translation(
                                size.get_rect(&CongratsLayoutEntity::ShareButton, &())
                                    .centre()
                                    .extend(crate::z_indices::CONGRATS_BUTTON),
                            ),
                        },
                        &(),
                    );
                }


            });
    }
}
