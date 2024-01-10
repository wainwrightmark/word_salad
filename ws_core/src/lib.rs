pub mod character;
pub mod colors;
pub mod designed_level;
pub mod display_word;
pub mod finder;
pub mod layout;
pub mod word;
pub mod level_type;
pub use crate::prelude::*;

pub mod prelude {

    pub use crate::character::*;
    pub use crate::colors::*;
    pub use crate::designed_level::*;
    pub use crate::display_word::*;
    pub use crate::word::*;

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
    // pub type Vertex = geometrid::vertex::Vertex<4, 4>;
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
