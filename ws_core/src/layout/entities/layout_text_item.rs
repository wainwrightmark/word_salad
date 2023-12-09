// use crate::prelude::*;
// use glam::Vec2;
// use strum::{Display, EnumCount, EnumIter, IntoEnumIterator};

// use super::consts::*;

// #[derive(
//     Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Display, EnumIter, EnumCount,
// )]
// pub enum LayoutPuzzleTheme {
//     PuzzleTheme = 0,
// }

// impl LayoutPuzzleTheme {
//     pub const fn index(&self) -> usize {
//         *self as usize
//     }
// }

// impl LayoutStructure for LayoutPuzzleTheme {
//     type Context = ();
//     type Iterator = <Self as IntoEnumIterator>::Iterator;

//     fn iter_all(_context: &Self::Context) -> Self::Iterator {
//         Self::iter()
//     }

//     fn pick(point: Vec2, context: &Self::Context) -> Option<Self> {
//         for x in Self::iter() {
//             if x.rect(context).contains(point) {
//                 return Some(x);
//             }
//         }
//         return None;
//     }

//     fn size(&self, _context: &Self::Context) -> Vec2 {
//         Vec2 {
//             x: TEXT_ITEM_WIDTH,
//             y: TEXT_ITEM_HEIGHT,
//         }
//     }

//     fn location(&self, _context: &Self::Context) -> Vec2 {
//         Vec2 {
//             x: (IDEAL_WIDTH - TEXT_ITEM_WIDTH) / 2.,
//             y: TOP_BAR_ICON_SIZE,
//         }
//     }
// }

// impl LayoutStructureWithFont for LayoutPuzzleTheme {
//     fn font_size(&self) -> f32 {
//         match self {
//             //LayoutTextItem::Timer => 24.0,
//             LayoutPuzzleTheme::PuzzleTheme => 32.0,
//         }
//     }
// }



// #[derive(
//     Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Display, EnumIter, EnumCount,
// )]
// pub enum LayoutTimer {
//     Timer = 0,
// }

// impl LayoutTimer {
//     pub const fn index(&self) -> usize {
//         *self as usize
//     }
// }

// impl LayoutStructure for LayoutTimer {
//     type Context = ();
//     type Iterator = <Self as IntoEnumIterator>::Iterator;

//     fn iter_all(_context: &Self::Context) -> Self::Iterator {
//         Self::iter()
//     }

//     fn pick(point: Vec2, context: &Self::Context) -> Option<Self> {
//         for x in Self::iter() {
//             if x.rect(context).contains(point) {
//                 return Some(x);
//             }
//         }
//         return None;
//     }

//     fn size(&self, _context: &Self::Context) -> Vec2 {
//         Vec2 {
//             x: TEXT_ITEM_WIDTH,
//             y: TEXT_ITEM_HEIGHT,
//         }
//     }

//     fn location(&self, _context: &Self::Context) -> Vec2 {
//         Vec2 {
//             x: (IDEAL_WIDTH - TEXT_ITEM_WIDTH) / 2.,
//             y: TOP_BAR_ICON_SIZE,
//         }
//     }
// }

// impl LayoutStructureWithFont for LayoutTimer {
//     fn font_size(&self) -> f32 {
//         match self {
//             LayoutTimer::Timer => 32.0,
//         }
//     }
// }
