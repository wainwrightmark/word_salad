use crate::prelude::*;
use maveric::{widgets::text2d_node::Text2DNode, with_bundle::CanWithBundle};
use ws_core::layout::entities::*;
#[derive(Debug, Clone, PartialEq)]
pub struct NonLevelView {
    pub non_level: NonLevel,
    pub selfie_mode: SelfieMode
}

impl MavericNode for NonLevelView {
    type Context = MyWindowSize;

    fn set_components(mut commands: SetComponentCommands<Self, Self::Context>) {
        commands.insert_static_bundle(SpatialBundle::default());
    }

    fn set_children<R: MavericRoot>(commands: SetChildrenCommands<Self, Self::Context, R>) {
        commands.unordered_children_with_node_and_context(|node, context, commands| {
            let size = &context;

            let selfie_mode = node.selfie_mode;

            let text = match node.non_level {
                NonLevel::BeforeTutorial => {
                    "Welcome to Word Salad\nLet's find some Chess Pieces".to_string()
                }
                NonLevel::AfterCustomLevel => "Custom Level Complete".to_string(),
                NonLevel::NoMoreDailyChallenge => {
                    "You have completed\nAll daily challenges".to_string()
                }
                NonLevel::NoMoreLevelSequence(ls) => {
                    format!("You have completed\nAll {}", ls.name())
                }
            };

            let text_color = if selfie_mode.0 {
                palette::CONGRATS_BUTTON_TEXT_SELFIE
            } else {
                palette::CONGRATS_BUTTON_TEXT_NORMAL
            }
            .convert_color();

            commands.add_child(
                "text",
                Text2DNode {
                    text,
                    font_size: size.font_size(&NonLevelLayoutEntity::Text),
                    color: text_color,
                    font: BUTTONS_FONT_PATH,
                    alignment: TextAlignment::Center,
                    linebreak_behavior: bevy::text::BreakLineOn::NoWrap,
                    text_2d_bounds: Default::default(),
                    text_anchor: Default::default(),
                }
                .with_bundle(Transform::from_translation(
                    size.get_rect(&NonLevelLayoutEntity::Text, &())
                        .centre()
                        .extend(crate::z_indices::CONGRATS_BUTTON),
                )),
                &(),
            );

            let interaction_text = match node.non_level {
                NonLevel::BeforeTutorial => "Ok",
                NonLevel::AfterCustomLevel => "Restart",
                NonLevel::NoMoreDailyChallenge => "Reset",
                NonLevel::NoMoreLevelSequence(_) => "Reset",
            };



            let fill_color = if selfie_mode.0 {
                palette::CONGRATS_BUTTON_FILL_SELFIE
            } else {
                palette::CONGRATS_BUTTON_FILL_NORMAL
            }
            .convert_color();

            commands.add_child(
                "interaction",
                WSButtonNode {
                    text: interaction_text,
                    font_size: size.font_size(&NonLevelLayoutEntity::InteractButton),
                    rect: size.get_rect(&NonLevelLayoutEntity::InteractButton, &()),
                    interaction: ButtonInteraction::NonLevelInteractionButton,
                    text_color,
                    fill_color
                },
                &(),
            );
        });
    }
}
