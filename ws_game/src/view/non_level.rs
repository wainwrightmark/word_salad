use crate::prelude::*;
use maveric::{widgets::text2d_node::Text2DNode, with_bundle::CanWithBundle};
use ws_core::{layout::entities::*, palette::BUTTON_CLICK_FILL};
#[derive(Debug, Clone, PartialEq)]
pub struct NonLevelView {
    pub non_level: NonLevel,
    pub selfie_mode: SelfieMode,
}

#[derive(Debug, Component, PartialEq, Clone, Copy)]
pub struct NonLevelText;

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
                NonLevel::DailyChallengeFinished => {
                    "You have completed\nAll daily challenges".to_string()
                }
                NonLevel::DailyChallengeReset => {
                    "You have completed\nAll daily challenges".to_string()
                }
                NonLevel::LevelSequenceAllFinished(ls) => {
                    format!("You have completed\nAll {}", ls.name())
                }
                NonLevel::LevelSequenceReset(ls) => {
                    format!("You have completed\nAll {}", ls.name())
                }
                NonLevel::DailyChallengeCountdown { todays_index } => {
                    DailyChallenges::time_until_challenge_string(todays_index)
                        .unwrap_or_else(|| "00:00:00".to_string())
                }
                NonLevel::LevelSequenceMustPurchaseGroup(ls) => {
                    format!(
                        "Buy the {} addon
                    \nTo unlock all {} levels",
                        ls.group().name(),
                        ls.group().total_count()
                    )
                }
            };

            let text_color = if selfie_mode.is_selfie_mode {
                palette::NON_LEVEL_TEXT_SELFIE
            } else {
                palette::NON_LEVEL_TEXT_NORMAL
            }
            .convert_color();

            let non_level_type = match node.non_level {
                NonLevel::DailyChallengeCountdown { .. } => NonLevelType::Countdown,
                _ => NonLevelType::Normal,
            };

            commands.add_child(
                "text",
                Text2DNode {
                    text,
                    font_size: size.font_size(&NonLevelLayoutEntity::Text, &non_level_type),
                    color: text_color,
                    font: BUTTONS_FONT_PATH,
                    alignment: TextAlignment::Center,
                    linebreak_behavior: bevy::text::BreakLineOn::NoWrap,
                    text_2d_bounds: Default::default(),
                    text_anchor: Default::default(),
                }
                .with_bundle((
                    Transform::from_translation(
                        size.get_rect(&NonLevelLayoutEntity::Text, &node.selfie_mode)
                            .centre()
                            .extend(crate::z_indices::CONGRATS_BUTTON),
                    ),
                    NonLevelText,
                )),
                &(),
            );

            let interaction_text = match node.non_level {
                NonLevel::BeforeTutorial => "Ok".to_string(),
                NonLevel::AfterCustomLevel => "Restart".to_string(),
                NonLevel::DailyChallengeFinished => "Next".to_string(),
                NonLevel::DailyChallengeReset | NonLevel::LevelSequenceReset(_) => {
                    "Reset".to_string()
                }
                NonLevel::LevelSequenceAllFinished(_) => "Next".to_string(),
                NonLevel::DailyChallengeCountdown { todays_index } => {
                    format!("Replay #{}", todays_index + 1)
                }
                NonLevel::LevelSequenceMustPurchaseGroup(_) => "Purchase".to_string(),
            };

            let (fill_color, border) = if selfie_mode.is_selfie_mode {
                (
                    palette::CONGRATS_BUTTON_FILL_SELFIE.convert_color(),
                    ShaderBorder::NONE,
                )
            } else {
                (Color::NONE, ShaderBorder::from_color(text_color))
            };

            commands.add_child(
                "interaction",
                WSButtonNode {
                    text: interaction_text,
                    font_size: size
                        .font_size(&NonLevelLayoutEntity::InteractButton, &non_level_type),
                    rect: size.get_rect(&NonLevelLayoutEntity::InteractButton, &node.selfie_mode),
                    interaction: ButtonInteraction::NonLevelInteractionButton,
                    text_color,
                    fill_color,
                    clicked_fill_color: BUTTON_CLICK_FILL.convert_color(),
                    border,
                },
                &(),
            );
        });
    }
}
