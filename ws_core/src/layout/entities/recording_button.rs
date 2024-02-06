use crate::prelude::*;
use glam::Vec2;

use super::{consts::*, SelfieMode};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ToggleRecordingButton;

impl LayoutStructure for ToggleRecordingButton {
    type Context<'a> = SelfieMode;

    fn size(&self, context: &Self::Context<'_>, sizing: &LayoutSizing) -> Vec2 {
        let default_size = Vec2 {
            x: RECORDING_BUTTON_MIN_SIZE,
            y: RECORDING_BUTTON_MIN_SIZE,
        };
        let bottom_padding = extra_bottom_space(sizing, context);

        if bottom_padding < RECORDING_BUTTON_MIN_SIZE {
            return default_size;
        }

        let size = bottom_padding * 0.5;

        Vec2 { x: size, y: size }
    }

    fn location(&self, selfie_mode: &Self::Context<'_>, sizing: &LayoutSizing) -> Vec2 {
        let extra_top_bar_height = extra_top_bar_height(sizing, selfie_mode);
        let default_location = Vec2 {
            x: ((IDEAL_WIDTH - GRID_SIZE) * 0.5) + RECORDING_BUTTON_MIN_SIZE,
            y: (TOP_BAR_HEIGHT_BASE + extra_top_bar_height - RECORDING_BUTTON_MIN_SIZE) * 0.5,
        };
        let bottom_padding = (sizing.bottom_pad / sizing.size_ratio) * 0.5;

        if bottom_padding < RECORDING_BUTTON_MIN_SIZE {
            return default_location;
        }

        Vec2 {
            x: IDEAL_WIDTH * 0.5,
            y: IDEAL_HEIGHT + extra_top_bar_height,
        }
    }

    fn iter_all(_context: &Self::Context<'_>) -> impl Iterator<Item = Self> {
        [Self].into_iter()
    }
}
