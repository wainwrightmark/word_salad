use crate::prelude::*;
use glam::Vec2;

use super::{consts::*, SelfieMode};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ToggleRecordingButton;

impl LayoutStructure for ToggleRecordingButton {
    type Context<'a> = SelfieMode;

    fn size(&self, context: &Self::Context<'_>, sizing: &LayoutSizing) -> Vec2 {
        let default_size = Vec2 {
            x: RECORDING_BUTTON_MIN_SIZE * 2.0,
            y: RECORDING_BUTTON_MIN_SIZE * 2.0,
        };
        let bottom_padding = extra_bottom_space(sizing, context);

        if bottom_padding < RECORDING_BUTTON_MIN_SIZE {
            return default_size;
        }

        let size = bottom_padding;

        Vec2 { x: size, y: size }
    }

    fn location(&self, context: &Self::Context<'_>, sizing: &LayoutSizing) -> Vec2 {
        let extra_top_bar_height = extra_top_bar_height(sizing, context);
        let default_location = Vec2 {
            x: RECORDING_BUTTON_MIN_SIZE * 0.5,
            y: ((TOP_BAR_HEIGHT_BASE + extra_top_bar_height) * 0.5) - RECORDING_BUTTON_MIN_SIZE,
        };
        let bottom_padding = extra_bottom_space(sizing, context);

        if bottom_padding < RECORDING_BUTTON_MIN_SIZE {
            return default_location;
        }

        Vec2 {
            x: (IDEAL_WIDTH - bottom_padding) * 0.5,
            y: IDEAL_HEIGHT + extra_top_bar_height,
        }
    }

    fn iter_all(_context: &Self::Context<'_>) -> impl Iterator<Item = Self> {
        [Self].into_iter()
    }
}
