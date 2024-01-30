use std::time::Duration;

use bevy::text::Text2dBounds;
use maveric::{widgets::text2d_node::Text2DNode, with_bundle::CanWithBundle};
use ws_core::layout::entities::HintsRemainingLayout;

use crate::{prelude::*, z_indices};

#[derive(MavericRoot)]
pub struct HintsRemainingRoot;

#[derive(Debug, NodeContext)]
pub struct HintsRemainingContext {
    pub hints: HintState,
    pub pressed: PressedButton,
    pub window_size: MyWindowSize,
    pub video_resource: VideoResource,
}

impl MavericRootChildren for HintsRemainingRoot {
    type Context = HintsRemainingContext;

    fn set_children(
        context: &<Self::Context as NodeContext>::Wrapper<'_>,
        commands: &mut impl ChildCommands,
    ) {
        if !matches!(
            context.pressed.as_ref(),
            PressedButton::Pressed {
                interaction: ButtonInteraction::WordButton(..),
                ..
            } | PressedButton::PressedAfterActivated {
                interaction: ButtonInteraction::WordButton(..),
            },
        ) {
            return;
        }

        let text = match context.hints.hints_remaining {
            0 => "No Hints Left".to_string(),
            1 => " 1 Hint  Left".to_string(),
            n => format!("{n:>2} Hints Left"),
        };

        let font_size = context.window_size.font_size(&HintsRemainingLayout, &());
        let color = if context.video_resource.is_selfie_mode {
            palette::HINTS_REMAINING_TEXT_COLOR_SELFIE
        } else {
            palette::HINTS_REMAINING_TEXT_COLOR_NORMAL
        }
        .convert_color();

        let rect = context
            .window_size
            .get_rect(&HintsRemainingLayout, &context.video_resource.selfie_mode());

        commands.add_child(
            "text",
            Text2DNode {
                text,
                font: THEME_FONT_PATH,
                font_size,
                color,
                alignment: TextAlignment::Center,
                linebreak_behavior: bevy::text::BreakLineOn::AnyCharacter,
                text_anchor: bevy::sprite::Anchor::Center,
                text_2d_bounds: Text2dBounds::UNBOUNDED,
            }
            .with_bundle(Transform::from_translation(
                rect.centre().extend(z_indices::HINTS_REMAINING),
            ))
            .with_transition_in_out::<TextColorLens<0>>(
                color.with_a(0.9),
                color,
                color.with_a(0.0),
                Duration::from_secs_f32(0.01),
                Duration::from_secs(3),
                None,
                None,
            ),
            &(),
        );
    }
}
