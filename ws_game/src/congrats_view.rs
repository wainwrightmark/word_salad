use crate::prelude::*;
use maveric::{widgets::text2d_node::Text2DNode, with_bundle::CanWithBundle};
use ws_core::layout::entities::*;
#[derive(Debug, Clone, PartialEq)]
pub struct CongratsView;

impl MavericNode for CongratsView {
    type Context = ViewContext;

    fn set_components(mut commands: SetComponentCommands<Self, Self::Context>) {
        commands.insert_static_bundle(SpatialBundle::default());
    }

    fn set_children<R: MavericRoot>(commands: SetChildrenCommands<Self, Self::Context, R>) {
        commands
            .ignore_node()
            .unordered_children_with_context(|context, commands| {
                let size = &context.3;

                let hints_used_text = match context.2.hints_used1 {
                    0 => "No hints used".to_string(),
                    1 => "1 hint used".to_string(),
                    n => format!("{n} hints used"),
                };

                //let full_rect = size.get_rect(GameLayoutEntity::, context)

                commands.add_child(
                    "hints used",
                    Text2DNode {
                        text: hints_used_text,
                        font_size: size.font_size(&CongratsLayoutEntity::HintsUsed),
                        color: palette::BUTTON_TEXT_COLOR.convert_color(),
                        font: BUTTONS_FONT_PATH,
                        alignment: TextAlignment::Center,
                        linebreak_behavior: bevy::text::BreakLineOn::NoWrap,
                    }
                    .with_bundle(Transform::from_translation(
                        size.get_rect(&CongratsLayoutEntity::HintsUsed, &())
                            .centre()
                            .extend(crate::z_indices::CONGRATS_BUTTON),
                    )),
                    &(),
                );

                commands.add_child(
                    "next level",
                    ButtonNode2d {
                        text: "Next",
                        font_size: size.font_size(&CongratsLayoutEntity::NextButton),
                        rect: size.get_rect(&CongratsLayoutEntity::NextButton, &()),
                        interaction: ButtonInteraction::Congrats(CongratsLayoutEntity::NextButton),
                        text_color: palette::CONGRATS_BUTTON_TEXT.convert_color(),
                        fill_color: palette::CONGRATS_BUTTON_FILL.convert_color(),
                    },
                    &(),
                );

                #[cfg(target_arch = "wasm32")]
                {
                    commands.add_child(
                        "share",
                        ButtonNode2d {
                            text: "Share",
                            font_size: size.font_size(&CongratsLayoutEntity::ShareButton),
                            rect: size.get_rect(&CongratsLayoutEntity::ShareButton, &()),
                            interaction: ButtonInteraction::Congrats(
                                CongratsLayoutEntity::ShareButton,
                            ),
                            text_color: palette::CONGRATS_BUTTON_TEXT.convert_color(),
                            fill_color: palette::CONGRATS_BUTTON_FILL.convert_color(),
                        },
                        &(),
                    );
                }
            });
    }
}
