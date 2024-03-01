use crate::prelude::*;
use maveric::{widgets::text2d_node::Text2DNode, with_bundle::CanWithBundle};
use ws_core::{layout::entities::*, palette::BUTTON_CLICK_FILL};

#[derive(Debug, NodeContext)]
pub struct NonLevelContext {
    pub size: MyWindowSize,
    pub prices: Prices,
    pub redraw_marker: RedrawMarker
}

impl<'a, 'w: 'a> From<&'a ViewContextWrapper<'w>> for NonLevelContextWrapper<'w> {
    fn from(value: &'a ViewContextWrapper<'w>) -> Self {
        Self {
            size: Res::clone(&value.window_size),
            prices: Res::clone(&value.prices),
            redraw_marker: Res::clone(&value.redraw_marker)
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NonLevelView {
    pub non_level: NonLevel,
    pub selfie_mode: SelfieMode,

}

#[derive(Debug, Component, PartialEq, Clone, Copy)]
pub struct NonLevelText;

impl MavericNode for NonLevelView {
    type Context = NonLevelContext;

    fn set_components(mut commands: SetComponentCommands<Self, Self::Context>) {
        commands.insert_static_bundle(SpatialBundle::default());
    }

    fn set_children<R: MavericRoot>(commands: SetChildrenCommands<Self, Self::Context, R>) {
        commands.unordered_children_with_node_and_context(|node, context, commands| {
            let size = &context.size;

            let selfie_mode = node.selfie_mode;

            let text = match node.non_level {
                NonLevel::BeforeTutorial => {
                    "Welcome to Word Salad\nLet's find some chess pieces".to_string()
                }
                NonLevel::AfterCustomLevel => "Custom Level Complete".to_string(),
                NonLevel::DailyChallengeFinished => {
                    "Well Done!\nThat's all the daily challenges\nWe'll see you tomorrow".to_string()
                }
                NonLevel::DailyChallengeReset => {
                    "You have completed\nAll daily challenges".to_string()
                }
                NonLevel::LevelSequenceAllFinished(ls) => {
                    //TODO text here samuel please
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
                        "Buy the {} add-on\nTo unlock all {} ad-free levels",
                        ls.group().name(),
                        ls.group().total_count()
                    )
                }
                NonLevel::DailyChallengeNotLoaded { .. } => {
                    "Could not load Daily Challenge".to_string()
                }
                NonLevel::DailyChallengeLoading { .. } => "Loading Daily Challenges".to_string(),

                NonLevel::AdBreak(_)=> "Ad Break".to_string(),
                NonLevel::AdFailed{since,..}=>{

                    if let Some(since) = since{
                        let seconds_remaining = AD_FAILED_SECONDS - chrono::Utc::now().signed_duration_since(since).num_seconds();
                        format!("Hate interruptions?\nRemove ads on the store page\n{seconds_remaining:2}")
                    }else{
                        format!("Hate interruptions?\nRemove ads on the store page\n")
                    }



                },
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
                    justify_text: JustifyText::Center,
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
                NonLevel::BeforeTutorial => Some("Ok".to_string()),
                NonLevel::AfterCustomLevel => Some("Restart".to_string()),
                NonLevel::DailyChallengeFinished => Some("Next".to_string()),
                NonLevel::DailyChallengeNotLoaded { .. } => Some("Retry".to_string()),
                NonLevel::DailyChallengeReset | NonLevel::LevelSequenceReset(_) => {
                    Some("Reset".to_string())
                }
                NonLevel::LevelSequenceAllFinished(_) => Some("Next".to_string()),
                NonLevel::DailyChallengeCountdown { todays_index } => {
                    Some(format!("Replay #{}", todays_index + 1))
                }
                NonLevel::LevelSequenceMustPurchaseGroup(ls) => {
                    context.prices.try_get_price_string(ls.group().into()).or_else(||Some("???".to_string()))
                },
                NonLevel::DailyChallengeLoading { .. } => None,
                NonLevel::AdBreak(_)=> None,
                NonLevel::AdFailed{since, ..}=> {
                    if  since.is_none(){
                        Some("Continue".to_string())
                    }
                    else{
                        None
                    }
                },
            };

            let (fill_color, border) = if selfie_mode.is_selfie_mode {
                (
                    palette::CONGRATS_BUTTON_FILL_SELFIE.convert_color(),
                    ShaderBorder::NONE,
                )
            } else {
                (Color::NONE, ShaderBorder::from_color(text_color))
            };

            if let Some(interaction_text) = interaction_text {
                commands.add_child(
                    "interaction",
                    WSButtonNode {
                        text: interaction_text,
                        font_size: size
                            .font_size(&NonLevelLayoutEntity::InteractButton, &non_level_type),
                        rect: size
                            .get_rect(&NonLevelLayoutEntity::InteractButton, &node.selfie_mode),
                        interaction: ButtonInteraction::NonLevelInteractionButton,
                        text_color,
                        fill_color,
                        clicked_fill_color: BUTTON_CLICK_FILL.convert_color(),
                        border,
                    },
                    &(),
                );
            }
        });
    }
}
