#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct BasicColor {
    pub red: f32,
    pub green: f32,
    pub blue: f32,
    pub alpha: f32,
}

impl BasicColor {
    pub(crate) const fn rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self {
            red: r,
            green: g,
            blue: b,
            alpha: a,
        }
    }
    pub(crate) const fn rgb(r: f32, g: f32, b: f32) -> Self {
        Self {
            red: r,
            green: g,
            blue: b,
            alpha: 1.0,
        }
    }

    pub(crate) const fn with_a(self, alpha: f32) -> Self {
        Self { alpha, ..self }
    }

    /// <div style="background-color:rgba(0%, 0%, 0%, 0%); width: 10px; padding: 10px; border: 1px solid;"></div>
    #[doc(alias = "transparent")]
    pub(crate) const NONE: BasicColor = BasicColor::rgba(0.0, 0.0, 0.0, 0.0);
    /// <div style="background-color:rgb(100%, 100%, 100%); width: 10px; padding: 10px; border: 1px solid;"></div>
    pub(crate) const WHITE: BasicColor = BasicColor::rgb(1.0, 1.0, 1.0);
}

pub mod palette {
    use crate::BasicColor as Color;

    pub const ANIMATED_SOLUTION_NEW: Color = FULL_GREEN;
    pub const ANIMATED_SOLUTION_OLD: Color = MY_BLUE;

    pub const TOP_BAR_BURGER: Color = MY_BLACK;
    pub const TOP_BAR_LOGO: Color = MY_BLACK;

    pub const THEME_TEXT_COLOR_NORMAL: Color = MY_BLACK;
    pub const THEME_TEXT_COLOR_SELFIE: Color = LIGHT_GRAY;

    pub const CONGRATS_BUTTON_TEXT_NORMAL: Color = MY_BLACK;
    pub const CONGRATS_BUTTON_FILL_NORMAL: Color = LIGHT_GREEN;

    pub const CONGRATS_BUTTON_TEXT_SELFIE: Color = MY_BLACK.with_a(0.95);
    pub const CONGRATS_BUTTON_FILL_SELFIE: Color = LIGHT_GREEN.with_a(0.5);

    pub const CONGRATS_STATISTIC_TEXT_SELFIE: Color = LIGHT_GRAY.with_a(0.95);
    pub const CONGRATS_STATISTIC_FILL_SELFIE: Color = DARK_GRAY.with_a(0.5);

    pub const CONGRATS_STATISTIC_TEXT_NORMAL: Color = MY_BLACK;
    pub const CONGRATS_STATISTIC_FILL_NORMAL: Color = LIGHT_GRAY;

    pub const BUTTON_CLICK_FILL: Color = MY_BLUE;

    pub const ICON_BUTTON_BACKGROUND: Color = Color::NONE;
    pub const TEXT_BUTTON_BACKGROUND: Color = Color::WHITE;

    pub const WORD_BACKGROUND_UNSTARTED: Color = LIGHT_GRAY;
    pub const WORD_BACKGROUND_MANUAL_HINT: Color = LIGHT_GREEN;
    pub const WORD_BACKGROUND_COMPLETE: Color = FULL_GREEN;
    pub const WORD_BACKGROUND_PROGRESS: Color = MY_BLUE;

    pub const WORD_LINE_COLOR: Color = FULL_GREEN.with_a(1.0);

    pub const GRID_TILE_FILL_NORMAL: Color = LIGHT_GRAY;

    pub const GRID_LETTER_NORMAL: Color = MY_BLACK;
    pub const GRID_LETTER_SELFIE: Color = LIGHT_GRAY.with_a(0.95);

    pub const GRID_TILE_FILL_SELFIE: Color = DARK_GRAY.with_a(0.5);

    pub const MENU_BUTTON_TEXT: Color = MY_BLACK;
    pub const MENU_BUTTON_FILL: Color = LIGHT_GREEN;
    pub const MENU_BUTTON_STROKE: Color = MY_BLACK;

    pub const POPUP_BOX_BACKGROUND: Color = Color::WHITE;
    pub const POPUP_BOX_BORDER: Color = MY_BLACK;

    pub const HINT_COUNTER_COLOR: Color = LIGHT_GREEN;
    pub const HINT_TEXT_COLOR: Color = MY_BLACK;

    pub const WORD_TEXT_NORMAL: Color = MY_BLACK;
    pub const WORD_TEXT_SELFIE: Color = MY_BLACK.with_a(0.95);

    pub const MY_BLACK: Color = Color::rgba(0.12, 0., 0., 1.);
    pub const MY_WHITE: Color = Color::rgba(1.0, 1.0, 1.0, 1.);

    pub const MY_BLUE: Color = Color::rgba(0.17, 0.48, 0.71, 1.0);
    pub const FULL_GREEN: Color = Color::rgba(0.17, 0.71, 0.35, 1.);
    pub const LIGHT_GREEN: Color = Color::rgba(0.463, 0.851, 0.596, 1.);
    pub const LIGHT_GRAY: Color = Color::rgba(0.91, 0.89, 0.89, 1.);
    pub const DARK_GRAY: Color = Color::rgba(0.4, 0.4, 0.4, 1.);
}
