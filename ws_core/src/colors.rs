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
    /// <div style="background-color:rgb(0%, 0%, 100%); width: 10px; padding: 10px; border: 1px solid;"></div>
    pub(crate) const BLUE: BasicColor = BasicColor::rgb(0.0, 0.0, 1.0);
    /// <div style="background-color:rgb(25%, 25%, 25%); width: 10px; padding: 10px; border: 1px solid;"></div>
    pub(crate) const DARK_GRAY: BasicColor = BasicColor::rgb(0.25, 0.25, 0.25);
    /// <div style="background-color:rgb(100%, 84%, 0%); width: 10px; padding: 10px; border: 1px solid;"></div>
    pub(crate) const GOLD: BasicColor = BasicColor::rgb(1.0, 0.84, 0.0);

    /// <div style="background-color:rgb(0%, 100%, 0%); width: 10px; padding: 10px; border: 1px solid;"></div>
    pub(crate) const GREEN: BasicColor = BasicColor::rgb(0.0, 1.0, 0.0);

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

    pub const ANIMATED_SOLUTION_NEW: Color = Color::rgb(0., 1.0, 0.0);
    pub const ANIMATED_SOLUTION_OLD: Color = Color::rgb(1., 1.0, 0.0);

    pub const BUTTON_TEXT_COLOR: Color = Color::rgb(0.1, 0.1, 0.1);
    pub const BUTTON_BORDER: Color = Color::BLACK;

    pub const ICON_BUTTON_BACKGROUND: Color = Color::NONE;
    pub const TEXT_BUTTON_BACKGROUND: Color = Color::WHITE;

    pub const GAME_BACKGROUND: Color = Color::ALICE_BLUE;

    pub const WORD_BACKGROUND_UNSTARTED: Color = Color::ALICE_BLUE;
    pub const WORD_BACKGROUND_MANUAL_HINT: Color = Color::rgb(0.3, 0.3, 0.9);
    pub const WORD_BACKGROUND_AUTO_HINT: Color = Color::SILVER;
    pub const WORD_BACKGROUND_COMPLETE: Color = Color::GREEN;
    pub const WORD_BORDER: Color = Color::DARK_GRAY;

    pub const WORD_LINE_COLOR: Color = Color::rgba(0.9, 0.25, 0.95, 0.9);

    pub const GRID_TILE_STROKE: Color = Color::DARK_GRAY;
    pub const GRID_TILE_FILL_SELECTABLE: Color = Color::rgb(0.7, 0.7, 0.7);
    pub const GRID_TILE_FILL_INADVISABLE: Color = GRID_TILE_FILL_SELECTABLE;//  Color::rgb(0.4, 0.5, 0.5);
    pub const GRID_TILE_FILL_UNSELECTABLE: Color = Color::rgb(0.4, 0.4, 0.4);
    pub const GRID_TILE_FILL_SELECTED: Color = Color::ALICE_BLUE;

    pub const GRID_LETTER: Color = Color::DARK_GRAY;

    pub const MANUAL_HINT_GLOW: Color = Color::GOLD;
    pub const AUTO_HINT_GLOW: Color = Color::BLUE;

    pub const MENU_BUTTON_TEXT: Color = Color::BLACK;
    pub const MENU_BUTTON_FILL: Color = Color::WHITE;
    pub const MENU_BUTTON_STROKE: Color = Color::BLACK;
}
