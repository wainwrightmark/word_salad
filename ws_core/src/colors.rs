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

    /// <div style="background-color:rgb(94%, 97%, 100%); width: 10px; padding: 10px; border: 1px solid;"></div>
    pub(crate) const ALICE_BLUE: BasicColor = BasicColor::rgb(0.94, 0.97, 1.0);
    /// <div style="background-color:rgb(0%, 0%, 0%); width: 10px; padding: 10px; border: 1px solid;"></div>
    pub(crate) const BLACK: BasicColor = BasicColor::rgb(0.0, 0.0, 0.0);
    /// <div style="background-color:rgb(100%, 84%, 0%); width: 10px; padding: 10px; border: 1px solid;"></div>
    pub(crate) const GOLD: BasicColor = BasicColor::rgb(1.0, 0.84, 0.0);


    /// <div style="background-color:rgb(75%, 75%, 75%); width: 10px; padding: 10px; border: 1px solid;"></div>
    pub(crate) const SILVER: BasicColor = BasicColor::rgb(0.75, 0.75, 0.75);
    /// <div style="background-color:rgba(0%, 0%, 0%, 0%); width: 10px; padding: 10px; border: 1px solid;"></div>
    #[doc(alias = "transparent")]
    pub(crate) const NONE: BasicColor = BasicColor::rgba(0.0, 0.0, 0.0, 0.0);
    /// <div style="background-color:rgb(100%, 100%, 100%); width: 10px; padding: 10px; border: 1px solid;"></div>
    pub(crate) const WHITE: BasicColor = BasicColor::rgb(1.0, 1.0, 1.0);
}

pub mod palette {
    use crate::BasicColor as Color;

    pub const ANIMATED_SOLUTION_NEW: Color = Color::rgb(72.0 /255.0,163.0 /255.0,87.0 /255.0);
    pub const ANIMATED_SOLUTION_OLD: Color = Color::rgb(251.0 /255.,247.0 /255.,159.0 /255.);

    pub const BUTTON_TEXT_COLOR: Color = Color::rgb(0.1, 0.1, 0.1);
    pub const BUTTON_BORDER: Color = Color::BLACK;


    pub const CONGRATS_BUTTON_TEXT: Color = Color::rgb(0.1, 0.1, 0.1);
    pub const CONGRATS_BUTTON_FILL: Color = Color::rgba(0.7,0.7,0.7,0.4);

    pub const ICON_BUTTON_BACKGROUND: Color = Color::NONE;
    pub const TEXT_BUTTON_BACKGROUND: Color = Color::WHITE;

    pub const GAME_BACKGROUND: Color = Color::ALICE_BLUE;

    pub const WORD_BACKGROUND_UNSTARTED: Color = Color::rgb(0.7, 0.7, 0.7);
    pub const WORD_BACKGROUND_MANUAL_HINT: Color = Color::rgb(0.3, 0.3, 0.9);
    pub const WORD_BACKGROUND_AUTO_HINT: Color = Color::SILVER;
    pub const WORD_BACKGROUND_COMPLETE: Color = Color::rgb(72.0 /255.0,163.0 /255.0,87.0 /255.0);


    pub const WORD_LINE_COLOR: Color = Color::rgba(0.9, 0.25, 0.95, 0.9);

    pub const GRID_TILE_STROKE_NORMAL: Color = Color::rgba(0.25, 0.25, 0.25, 0.7);
    pub const GRID_TILE_FILL_NORMAL: Color = Color::rgba(0.7, 0.7, 0.7, 0.9);
    pub const GRID_LETTER_NORMAL: Color = Color::rgba(0.25, 0.25, 0.25, 0.85);

    pub const GRID_TILE_STROKE_SELFIE: Color = Color::rgba(0.25, 0.25, 0.25, 0.95);
    pub const GRID_TILE_FILL_SELFIE: Color = Color::rgba(0.7, 0.7, 0.7, 0.7);
    pub const GRID_LETTER_SELFIE: Color = Color::rgba(0.25, 0.25, 0.25, 0.95);

    pub const MANUAL_HINT_GLOW: Color = Color::GOLD;

    pub const MENU_BUTTON_TEXT: Color = Color::BLACK;
    pub const MENU_BUTTON_FILL: Color = Color::rgb(0.8, 0.8, 0.8);
    pub const MENU_BUTTON_STROKE: Color = Color::BLACK;

    pub const POPUP_BOX_BACKGROUND: Color = Color::WHITE;
    pub const POPUP_BOX_BORDER: Color = Color::BLACK;

    pub const BACKGROUND_COLOR_1: Color = Color::rgb(234. / 255., 177. / 255., 138. / 255.);
    pub const BACKGROUND_COLOR_2: Color = Color::rgb(158. / 255., 216. / 255., 112. / 255.);

    pub const HINT_COUNTER_COLOR: Color = Color::rgb(51. / 255., 138. / 255., 225. / 255.);
}
