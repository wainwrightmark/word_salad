use crate::prelude::*;
use glam::Vec2;

use super::{consts::*, GameLayoutEntity, SelfieMode};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ToggleRecordingButton;

impl LayoutStructure for ToggleRecordingButton {
    type Context<'a> = (SelfieMode, Insets);

    fn size(&self, context: &Self::Context<'_>, sizing: &LayoutSizing) -> Vec2 {
        let default_size = Vec2 {
            x: RECORDING_BUTTON_MIN_SIZE,
            y: RECORDING_BUTTON_MIN_SIZE,
        };
        let bottom_padding = extra_bottom_space(sizing, &context.0);

        let bottom_size = bottom_padding * 0.75;

        if bottom_size < RECORDING_BUTTON_MIN_SIZE {
            return default_size;
        }

        let size = bottom_size.min(RECORDING_BUTTON_MAX_SIZE);

        Vec2 { x: size, y: size }
    }

    fn location(&self, context: &Self::Context<'_>, sizing: &LayoutSizing) -> Vec2 {
        let extra_top_bar_height = extra_top_height(sizing, &context.0);
        let default_location = Vec2 {
            x: IDEAL_WIDTH - RECORDING_BUTTON_MIN_SIZE - ((IDEAL_WIDTH - GRID_SIZE) * 0.5),
            y: GameLayoutEntity::TopBar.location(context, sizing).y + (RECORDING_BUTTON_MIN_SIZE * 0.5),
        };
        let bottom_padding = extra_bottom_space(sizing, &context.0);

        let bottom_size = bottom_padding * 0.75;

        if bottom_size <= RECORDING_BUTTON_MIN_SIZE {
            return default_location;
        }

        let size = bottom_size.min(RECORDING_BUTTON_MAX_SIZE);

        Vec2 {
            x: (IDEAL_WIDTH - size) * 0.5,
            y: IDEAL_HEIGHT + extra_top_bar_height + (extra_bottom_space(sizing, &context.0) * 0.5) - (size * 0.5),
        }
    }

    fn iter_all(_context: &Self::Context<'_>) -> impl Iterator<Item = Self> {
        [Self].into_iter()
    }
}
