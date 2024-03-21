pub mod background_type;
pub mod character;
pub mod colors;
pub mod complete_solve;
pub mod designed_level;
pub mod display_word;
pub mod finder;
pub mod font_icons;
pub mod insets;
pub mod layout;
pub mod level_trait;
pub mod level_type;
pub mod word;
pub mod word_trait;
pub use crate::prelude::*;

pub mod prelude {

    pub use crate::background_type::*;
    pub use crate::character::*;
    pub use crate::colors::*;
    pub use crate::designed_level::*;
    pub use crate::display_word::*;
    pub use crate::font_icons::*;
    pub use crate::insets::*;
    pub use crate::level_trait::*;
    pub use crate::word::*;
    pub use crate::word_trait::*;

    pub use arrayvec::ArrayVec;
    pub use geometrid::prelude::HasCenter;
    pub use geometrid::prelude::*;
    pub use std::array;
    pub use ustr::Ustr;

    pub use crate::layout::prelude::*;

    pub type Tile = geometrid::tile::Tile<4, 4>;

    pub type CharsArray = ArrayVec<Character, 16>;
    pub type Grid = geometrid::tile_map::TileMap<Character, 4, 4, 16>;
    pub type GridSet = geometrid::tile_set::TileSet16<4, 4, 16>;
    pub type Solution = ArrayVec<Tile, 16>;

    pub fn try_make_grid(text: &str) -> Option<Grid> {
        let mut arr = [Character::Blank; 16];
        for (index, char) in text.chars().enumerate() {
            let c = Character::try_from(char).ok()?;
            *arr.get_mut(index)? = c;
        }

        Some(Grid::from_inner(arr))
    }
}
