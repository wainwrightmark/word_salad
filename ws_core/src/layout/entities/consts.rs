use crate::LayoutSizing;

use super::SelfieMode;

pub const IDEAL_WIDTH: f32 = 320.;
pub const IDEAL_HEIGHT: f32 = 568.;
pub const IDEAL_RATIO: f32 = IDEAL_WIDTH as f32 / IDEAL_HEIGHT as f32;

pub const TOP_BAR_OFFSET: f32 = 30.;
pub const TOP_BAR_HEIGHT: f32 = 40.;
pub const WORD_SALAD_LOGO_SIZE: f32 = 30.;
pub const TIMER_HEIGHT: f32 = 20.;
pub const RECORDING_BUTTON_MIN_SIZE: f32 = 25.0;
pub const RECORDING_BUTTON_MAX_SIZE: f32 = 50.0;
//pub const WORD_SALAD_LOGO_WIDTH: f32 = 160.;

pub const THEME_HEIGHT: f32 = 24.;

pub const THEME_INFO_HEIGHT: f32 = 18.;

pub const TIMER_WIDTH: f32 = 60.;

pub const GRID_TILE_SIZE: f32 = 64.;
pub const GRID_GAP: f32 = 12.;
pub const GRID_SIZE: f32 = (GRID_TILE_SIZE * 4.0) + GRID_GAP;

pub const LEFT_MARGIN: f32 = (IDEAL_WIDTH - GRID_SIZE) * 0.5;

pub const GRID_WORD_LIST_SPACER: f32 = GRID_TILE_SIZE * 0.5;
pub const GRID_THEME_SPACER: f32 = GRID_TILE_SIZE * 0.5;
pub const WORD_LIST_EXTRA_WIDTH: f32 = 20.0;

pub const WORD_LIST_HEIGHT: f32 = 124.;
pub const WORD_HEIGHT: f32 = 22.;
pub const WORD_WIDTH_PER_CHARACTER: f32 = 11.;
pub const WORD_WIDTH_FIXED: f32 = 20.;

pub const WORD_LIST_WIDTH: f32 = GRID_SIZE + WORD_LIST_EXTRA_WIDTH;
pub const WORD_MAIN_PAD: f32 = 10.;
pub const WORD_CROSS_PAD: f32 = 5.;

pub const USED_HEIGHT_BASE: f32 = TOP_BAR_OFFSET
    + TOP_BAR_HEIGHT
    + THEME_HEIGHT
    + THEME_INFO_HEIGHT
    + GRID_SIZE
    + GRID_THEME_SPACER
    + GRID_WORD_LIST_SPACER
    + WORD_LIST_HEIGHT;

static_assertions::const_assert_eq!(USED_HEIGHT_BASE, IDEAL_HEIGHT);

pub const GRID_MID_BASE: f32 = TOP_BAR_HEIGHT + TOP_BAR_OFFSET
    + THEME_HEIGHT
    + THEME_INFO_HEIGHT
    + GRID_THEME_SPACER
    + (GRID_SIZE * 0.5);

pub fn extra_top_height(sizing: &LayoutSizing, selfie_mode: &SelfieMode) -> f32 {
    if selfie_mode.is_selfie_mode {
        return 0.0;
    }

    let bottom_padding = sizing.bottom_pad / sizing.size_ratio;
    let total_height = IDEAL_HEIGHT + bottom_padding;
    let mid = total_height * 0.5;
    let result = bottom_padding.min(mid - (GRID_MID_BASE));
    //log::info!("bottom padding: {bottom_padding} total height: {total_height} mid: {mid} result: {result} ");
    result
}

pub fn extra_bottom_space(sizing: &LayoutSizing, selfie_mode: &SelfieMode) -> f32 {
    let bottom_padding = sizing.bottom_pad / sizing.size_ratio;
    let x = extra_top_height(sizing, selfie_mode);

    bottom_padding - x
}

/// Extra top offset while streaming

pub const CONGRATS_ENTITY_STATISTIC_SIZE_NORMAL: f32 = 72.0;
pub const CONGRATS_ENTITY_STATISTIC_SIZE_SELFIE: f32 =
    CONGRATS_ENTITY_STATISTIC_SIZE_NORMAL * 2.0 / 3.0;

pub const CONGRATS_ENTITY_SPACING: f32 = 5.0;

pub const CONGRATS_BUTTON_GAP_NORMAL: f32 = 100.0;
pub const CONGRATS_BUTTON_GAP_SELFIE: f32 = CONGRATS_ENTITY_SPACING;

pub const CONGRATS_ENTITY_BUTTON_HEIGHT: f32 = 35.0;
pub const CONGRATS_ENTITY_BUTTON_WIDTH_NORMAL: f32 = GRID_SIZE;
pub const CONGRATS_ENTITY_BUTTON_WIDTH_SELFIE: f32 = GRID_SIZE * 2.0 / 3.0;

pub const NON_LEVEL_TEXT_HEIGHT: f32 = 80.0;
pub const NON_LEVEL_TEXT_WIDTH: f32 = 240.0;

pub const NON_LEVEL_BUTTON_HEIGHT: f32 = 35.0;
pub const NON_LEVEL_BUTTON_WIDTH: f32 = GRID_SIZE;

pub const HINTS_REMAINING_HEIGHT: f32 = 20.0;

pub const HINTS_POPUP_BOX_TOP: f32 = 80.;
pub const HINTS_POPUP_BOX_WIDTH: f32 = 300.;
pub const HINTS_POPUP_BOX_HEIGHT: f32 = 300.;

pub const HINTS_POPUP_BOX_TITLE_WIDTH: f32 = 280.;
pub const HINTS_POPUP_BOX_TITLE_HEIGHT: f32 = 40.;

pub const HINTS_POPUP_BOX_BUTTON_HEIGHT: f32 = 40.;
pub const HINTS_POPUP_BOX_BUTTON_WIDTH: f32 = 280.;

pub const SELFIE_POPUP_BOX_TOP: f32 = 180.;
pub const SELFIE_POPUP_BOX_WIDTH: f32 = 300.;
pub const SELFIE_POPUP_BOX_HEIGHT: f32 = 250.;

pub const SELFIE_POPUP_BOX_TITLE_WIDTH: f32 = 280.;
pub const SELFIE_POPUP_BOX_TITLE_HEIGHT: f32 = 60.;

pub const SELFIE_POPUP_BOX_BUTTON_HEIGHT: f32 = 43.;
pub const SELFIE_POPUP_BOX_BUTTON_WIDTH: f32 = 280.;

pub const GRID_TILE_FONT_SIZE: f32 = 40f32;

pub const CONGRATS_TIMER_FONT_SIZE_SELFIE: f32 = 40f32;
pub const CONGRATS_TIMER_FONT_SIZE_NORMAL: f32 = 60f32;

pub const CONGRATS_BUTTON_FONT_SIZE: f32 = 22f32;
pub const STATISTIC_NUMBER_FONT_SIZE_SELFIE: f32 = 22f32;
pub const STATISTIC_NUMBER_FONT_SIZE_NORMAL: f32 = 34f32;
pub const STATISTIC_LABEL_FONT_SIZE_SELFIE: f32 = 10f32;
pub const STATISTIC_LABEL_FONT_SIZE_NORMAL: f32 = 14f32;

pub const THEME_FONT_SIZE: f32 = 22f32;
pub const THEME_FONT_SIZE_SMALL: f32 = 18f32;
pub const THEME_INFO_FONT_SIZE: f32 = 18f32;
pub const TIMER_FONT_SIZE: f32 = 18f32;

pub const WORD_TILE_FONT_SIZE: f32 = 14f32;

pub const NON_LEVEL_TEXT_FONT_SIZE: f32 = 22f32;
pub const NON_LEVEL_COUNTDOWN_FONT_SIZE: f32 = 34f32;

pub const MENU_BUTTON_FONT_SIZE: f32 = 22f32;
pub const MENU_BUTTON_FONT_SIZE_SMALL: f32 = 18f32;

pub const TUTORIAL_TEXT_FONT_SIZE: f32 = 18f32;
pub const HINTS_REMAINING_FONT_SIZE: f32 = 18f32;

pub const MENU_BUTTON_HEIGHT: f32 = 35.0;
pub const MENU_BUTTON_WIDTH: f32 = GRID_SIZE;
pub const MENU_BUTTON_SPACING: f32 = 40.0 * 0.1;
pub const MENU_VIRTUAL_CHILDREN: usize = 9; //eight buttons plus back button
