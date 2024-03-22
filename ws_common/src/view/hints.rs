use std::time::Duration;

use bevy::text::Text2dBounds;
use maveric::{widgets::text2d_node::Text2DNode, with_bundle::CanWithBundle};
use ws_core::layout::entities::HintsRemainingLayout;

use crate::{prelude::*, z_indices};

pub struct HintsRemainingPlugin;
impl Plugin for HintsRemainingPlugin {
    fn build(&self, app: &mut App) {
        app.register_maveric::<HintsRemainingRoot>();
    }
}

const LINGER_SECONDS: f32 = 3.0;

#[derive(MavericRoot)]
struct HintsRemainingRoot;

#[allow(dead_code)]
#[derive(Debug, NodeContext)]
struct HintsRemainingContext {
    pub hints: HintState,
    pub window_size: MyWindowSize,
    pub video_resource: VideoResource,
    pub current_level: CurrentLevel,
    pub pressed_button: PressedButton,
    pub menu: MenuState,
    pub insets_resource: InsetsResource,
}

fn is_word_button_pressed(b: &PressedButton) -> bool {
    matches!(
        b,
        PressedButton::Pressed {
            interaction: ButtonInteraction::WordButton(..),
            ..
        } | PressedButton::PressedAfterActivated {
            interaction: ButtonInteraction::WordButton(..),
        },
    )
}

impl MavericRootChildren for HintsRemainingRoot {
    type Context = HintsRemainingContext;

    fn set_children(
        context: &<Self::Context as NodeContext>::Wrapper<'_>,
        commands: &mut impl ChildCommands,
    ) {
        if !context.menu.is_closed() {
            return;
        }

        if !context.current_level.should_spend_hints() {
            return;
        }

        if !is_word_button_pressed(&context.pressed_button) &&!context.current_level.is_changed() {
            return;
        }

        let text = context.hints.as_text();

        let font_size = context.window_size.font_size(&HintsRemainingLayout, &());
        let color = if context.video_resource.is_selfie_mode {
            palette::HINTS_REMAINING_TEXT_COLOR_SELFIE
        } else {
            palette::HINTS_REMAINING_TEXT_COLOR_NORMAL
        }
        .convert_color();

        let rect = context.window_size.get_rect(
            &HintsRemainingLayout,
            &(
                context.video_resource.selfie_mode(),
                context.insets_resource.0,
            ),
        );

        commands.add_child(
            "text",
            Text2DNode {
                text,
                font: THEME_FONT_PATH,
                font_size,
                color,
                justify_text: JustifyText::Center,
                linebreak_behavior: bevy::text::BreakLineOn::AnyCharacter,
                text_anchor: bevy::sprite::Anchor::Center,
                text_2d_bounds: Text2dBounds::UNBOUNDED,
            }
            .with_bundle(Transform::from_translation(
                rect.centre().extend(z_indices::HINTS_REMAINING),
            ))
            .with_transition_in_out::<TextColorLens<0>>(
                color,
                color,
                color.with_a(0.0),
                Duration::ZERO,
                Duration::from_secs_f32(LINGER_SECONDS),
                None,
                Some(Ease::CubicIn),
            ),
            &(),
        );
    }
}
