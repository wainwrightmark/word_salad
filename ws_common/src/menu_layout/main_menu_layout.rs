use strum::{Display, EnumCount, EnumIter, IntoEnumIterator};
use ws_core::{layout::entities::*, LayoutStructureWithTextOrImage};

#[cfg(all(target_arch = "wasm32", feature = "web"))]
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, EnumIter, EnumCount, Display,
)]
pub enum MainMenuLayoutEntity {
    Puzzles = 0,

    SelfieMode = 1,
    Tutorial = 2,
    ResetPuzzle = 3,
    PlaySteks = 4,
    GetFullGame = 5,
}

#[cfg(all(target_arch = "wasm32", not(feature = "web")))]
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, EnumIter, EnumCount, Display,
)]
pub enum MainMenuLayoutEntity {
    Puzzles = 0,

    Store = 1,
    SelfieMode = 2,
    Tutorial = 3,
    ResetPuzzle = 4,
    Settings = 5,
    PlaySteks = 6,
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, EnumIter, EnumCount, Display,
)]
pub enum MainMenuLayoutEntity {
    Puzzles = 0,
    SelfieMode = 1,
    Tutorial = 2,
    ResetPuzzle = 3,
}

impl MenuButtonsLayout for MainMenuLayoutEntity {
    type Context = ();

    fn index(&self) -> usize {
        *self as usize
    }

    fn count(_context: &Self::Context) -> usize {
        Self::COUNT
    }

    fn iter_all(_context: &Self::Context) -> impl Iterator<Item = Self> {
        Self::iter()
    }
}

impl LayoutStructureWithTextOrImage for MainMenuLayoutEntity {
    fn text_or_image(&self, _context: &Self::Context<'_>) -> ws_core::prelude::TextOrImage {


        match self {
            MainMenuLayoutEntity::Puzzles => ws_core::TextOrImage::Text { text: "Puzzles" },

            MainMenuLayoutEntity::SelfieMode => ws_core::TextOrImage::Text {
                text: "Selfie Mode",
            },
            MainMenuLayoutEntity::Tutorial => ws_core::TextOrImage::Text { text: "Tutorial" },
            MainMenuLayoutEntity::ResetPuzzle => ws_core::TextOrImage::Text {
                text: "Reset Puzzle",
            },
            #[cfg(all(target_arch = "wasm32", not(feature = "web")))]
            MainMenuLayoutEntity::Store => ws_core::TextOrImage::Text { text: "Store" },
            #[cfg(target_arch = "wasm32")]
            MainMenuLayoutEntity::PlaySteks => ws_core::TextOrImage::Image {
                path: "embedded://ws_common/../../assets/images/steks_button.png",
                color: ws_core::BasicColor::rgba(0.53, 0.68, 0.92, 1.0),
                pressed_color: ws_core::BasicColor::rgba(0.36, 0.55, 0.88, 1.0),
                aspect_ratio: 7168.0 / 1024.0,
            },
            #[cfg(all(target_arch = "wasm32", not(feature = "web")))]
            MainMenuLayoutEntity::Settings => ws_core::TextOrImage::Text { text: "Settings" },

            #[cfg(all(target_arch = "wasm32", feature = "web"))]
            MainMenuLayoutEntity::GetFullGame => ws_core::TextOrImage::Text { text: "Full Game" },
        }
    }
}
