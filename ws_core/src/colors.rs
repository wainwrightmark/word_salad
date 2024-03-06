#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct BasicColor {
    pub red: f32,
    pub green: f32,
    pub blue: f32,
    pub alpha: f32,
}

impl BasicColor {
    pub const fn rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
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

    pub fn try_from_str(hex: &str) -> Option<Self> {
        let hex = hex.strip_prefix('#').unwrap_or(hex);

        let (r, g, b, a) = match *hex.as_bytes() {
            // RGB
            [r, g, b] => {
                let [r, g, b, ..] = Self::decode_hex([r, r, g, g, b, b])?;
                (r, g, b, u8::MAX)
            }
            // RGBA
            [r, g, b, a] => {
                let [r, g, b, a, ..] = Self::decode_hex([r, r, g, g, b, b, a, a])?;
                (r, g, b, a)
            }
            // RRGGBB
            [r1, r2, g1, g2, b1, b2] => {
                let [r, g, b, ..] = Self::decode_hex([r1, r2, g1, g2, b1, b2])?;
                (r, g, b, u8::MAX)
            }
            // RRGGBBAA
            [r1, r2, g1, g2, b1, b2, a1, a2] => {
                let [r, g, b, a, ..] = Self::decode_hex([r1, r2, g1, g2, b1, b2, a1, a2])?;
                (r, g, b, a)
            }
            _ => {
                return None;
            }
        };

        Some(Self::rgba(
            r as f32 / 255.0,
            g as f32 / 255.0,
            b as f32 / 255.0,
            a as f32 / 255.0,
        ))
    }

    const fn decode_hex<const N: usize>(mut bytes: [u8; N]) -> Option<[u8; N]> {
        /// Parse a single hex digit (a-f/A-F/0-9) as a `u8`
        const fn hex_value(b: u8) -> Result<u8, u8> {
            match b {
                b'0'..=b'9' => Ok(b - b'0'),
                b'A'..=b'F' => Ok(b - b'A' + 10),
                b'a'..=b'f' => Ok(b - b'a' + 10),
                // Wrong hex digit
                _ => Err(b),
            }
        }
        let mut i = 0;
        while i < bytes.len() {
            // Convert single hex digit to u8
            let val = match hex_value(bytes[i]) {
                Ok(val) => val,
                Err(..) => return None,
            };
            bytes[i] = val;
            i += 1;
        }
        // Modify the original bytes to give an `N / 2` length result
        i = 0;
        while i < bytes.len() / 2 {
            // Convert pairs of u8 to R/G/B/A
            // e.g `ff` -> [102, 102] -> [15, 15] = 255
            bytes[i] = bytes[i * 2] * 16 + bytes[i * 2 + 1];
            i += 1;
        }
        Some(bytes)
    }
}

pub mod palette {
    use crate::BasicColor as Color;

    pub const ANIMATED_SOLUTION_NEW: Color = GREEN_LIGHT;
    pub const ANIMATED_SOLUTION_OLD: Color = GOLD;

    pub const TOP_BAR_LOGO_SELFIE: Color = MY_WHITE;

    pub const THEME_TITLE_COLOR_INCOMPLETE_NORMAL: Color = GREEN_LIGHT;
    pub const THEME_TITLE_COLOR_COMPLETE_NORMAL: Color = MY_WHITE;
    pub const THEME_TITLE_COLOR_SELFIE: Color = LIGHT_GRAY;

    pub const THEME_INFO_COLOR_INCOMPLETE_NORMAL: Color = MEDIUM_GRAY;
    pub const THEME_INFO_COLOR_COMPLETE_NORMAL: Color = MY_WHITE;
    pub const THEME_INFO_COLOR_SELFIE: Color = LIGHT_GRAY;

    pub const TUTORIAL_TOP_TEXT: Color = MY_BLACK;
    pub const TUTORIAL_BOTTOM_TEXT: Color = MY_BLACK;
    pub const TUTORIAL_MIDDLE_TEXT: Color = MY_WHITE;

    pub const TIMER_COLOR_NORMAL: Color = MEDIUM_GRAY;
    pub const TIMER_COLOR_SELFIE: Color = LIGHT_GRAY;

    pub const CONGRATS_BUTTON_TEXT_NORMAL: Color = MY_WHITE;
    pub const CONGRATS_BUTTON_FILL_NORMAL: Color = GREEN_LIGHT;
    pub const CONGRATS_STATISTIC_TEXT_NORMAL: Color = MY_WHITE;

    pub const HINTS_REMAINING_TEXT_COLOR_NORMAL: Color = MY_BLACK;
    pub const HINTS_REMAINING_TEXT_COLOR_SELFIE: Color = MY_WHITE;

    pub const CONGRATS_BUTTON_TEXT_SELFIE: Color = MY_WHITE.with_a(0.95);
    pub const CONGRATS_BUTTON_FILL_SELFIE: Color = GREEN_LIGHT.with_a(0.5);

    pub const CONGRATS_STATISTIC_TEXT_SELFIE: Color = LIGHT_GRAY.with_a(0.95);

    pub const ICON_BUTTON_BACKGROUND: Color = Color::NONE;
    pub const TEXT_BUTTON_BACKGROUND: Color = Color::WHITE;

    pub const WORD_BACKGROUND_UNSTARTED: Color = LIGHT_GRAY;
    pub const WORD_BACKGROUND_MANUAL_HINT: Color = GREEN_DARK;
    pub const WORD_BACKGROUND_MANUAL_HINT2: Color = GREEN_OTHER;
    pub const WORD_BACKGROUND_COMPLETE: Color = GREEN_LIGHT;
    pub const WORD_BACKGROUND_PROGRESS: Color = GREEN_DARK;

    pub const GRID_TILE_FILL_NORMAL: Color = LIGHT_GRAY;

    pub const GRID_LETTER_NORMAL: Color = MY_BLACK;
    pub const GRID_LETTER_SELFIE: Color = LIGHT_GRAY.with_a(0.95);
    pub const GRID_LETTER_SELECTED: Color = GOLD;

    pub const GRID_TILE_FILL_SELFIE: Color = MEDIUM_GRAY.with_a(0.5);

    pub const MENU_BUTTON_TEXT_REGULAR: Color = MY_WHITE;
    pub const MENU_BUTTON_TEXT_DISCOURAGED: Color = MY_BLACK;
    pub const MENU_BUTTON_FILL: Color = GREEN_LIGHT;
    pub const BUTTON_CLICK_FILL: Color = GREEN_DARK;
    pub const MENU_BUTTON_DISCOURAGED_FILL: Color = LIGHT_GRAY;
    pub const MENU_BUTTON_COMPLETE_FILL: Color = GREEN_OTHER;
    pub const MENU_BUTTON_STROKE: Color = MY_BLACK;

    pub const POPUP_BOX_BACKGROUND: Color = Color::WHITE;
    pub const POPUP_BOX_BORDER: Color = MY_BLACK;

    pub const HINT_COUNTER_COLOR: Color = GREEN_LIGHT;
    pub const HINT_TEXT_COLOR: Color = MY_WHITE;

    pub const WORD_TEXT_LETTERS: Color = MY_WHITE;
    pub const WORD_TEXT_NUMBER: Color = MY_BLACK;

    pub const NON_LEVEL_TEXT_NORMAL: Color = MY_WHITE;
    pub const NON_LEVEL_TEXT_SELFIE: Color = MY_WHITE;

    pub const CLEAR_COLOR_SELFIE: Color = Color::rgba(0.1, 0.1, 0.1, 0.1);
    pub const CLEAR_COLOR_NORMAL: Color = MY_WHITE;
    pub const CLEAR_COLOR_NON_LEVEL: Color = GREEN_LIGHT;
    pub const CLEAR_COLOR_CONGRATS: Color = GREEN_LIGHT;

    pub const RECORDING_BUTTON_SELFIE: Color = MY_WHITE;
    pub const RECORDING_BUTTON_NORMAL: Color = MY_BLACK;
    pub const RECORDING_BUTTON_RECORDING: Color = Color::rgba(1.0, 0.14, 0.09, 1.0);

    const MY_BLACK: Color = Color::rgba(0.12, 0., 0., 1.);
    const MY_WHITE: Color = Color::rgba(1.0, 1.0, 1.0, 1.);

    const LIGHT_GRAY: Color = Color::rgba(0.96, 0.95, 0.95, 1.);

    const MEDIUM_GRAY: Color = Color::rgba(0.4, 0.42, 0.44, 1.0);

    const GREEN_LIGHT: Color = Color::rgb(0.16, 0.66, 0.28);
    pub const GREEN_DARK: Color = Color::rgb(0.09, 0.34, 0.27);
    const GREEN_OTHER: Color = Color::rgb(0.01, 0.53, 0.22);
    const GOLD: Color = Color::rgba(1., 0.94, 0.62, 1.);
    #[allow(dead_code)]
    pub const TRANSPARENT: Color = Color::rgba(0., 0., 0., 0.);
}




#[cfg(test)]
mod tests {
    use crate::BasicColor;

    #[test]
    pub fn test_parse() {
        let hex = "#165b33";

        let actual = BasicColor::try_from_str(hex);

        assert_eq!(actual, Some(BasicColor::rgb(0.08627451, 0.35686275, 0.2)));
    }
}
