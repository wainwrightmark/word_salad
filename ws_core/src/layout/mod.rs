use glam::Vec2;
use std::ops::Add;
use strum::IntoEnumIterator;
use strum::{Display, EnumIter};

use self::{
    layout_structure::LayoutStructure,
    spacing::{tile_offset, Spacing},
};

pub type Tile = geometrid::tile::Tile<4, 4>;
pub type WordTile = geometrid::tile::Tile<2, 5>;

pub mod layout_sizing;
pub mod layout_structure;
pub mod rect;
pub mod spacing;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Display, EnumIter)]
pub enum TopBarButton {
    MenuBurgerButton,
    TimeCounter,
    HintCounter,
}

impl TopBarButton {
    pub const fn index(&self) -> usize {
        match self {
            TopBarButton::MenuBurgerButton => 0,
            TopBarButton::TimeCounter => 1,
            TopBarButton::HintCounter => 2,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Display, EnumIter)]
pub enum TextItem {
    PuzzleTitle,
    PuzzleTheme,
}

impl TextItem {
    pub const fn index(&self) -> usize {
        match self {
            TextItem::PuzzleTitle => 0,
            TextItem::PuzzleTheme => 1,
        }
    }
}

pub const IDEAL_WIDTH: f32 = 320.;
pub const IDEAL_HEIGHT: f32 = 568.;
pub const IDEAL_RATIO: f32 = IDEAL_WIDTH as f32 / IDEAL_HEIGHT as f32;

const TOP_BAR_ICON_SIZE: f32 = 40.;
const TEXT_ITEM_HEIGHT: f32 = 30.;
const TEXT_ITEM_WIDTH: f32 = 300.;

const TEXT_AREA_HEIGHT: f32 = 70.;

const GRID_TILE_SIZE: f32 = 72.;
const GRID_SIZE: f32 = 320.;

const WORD_LIST_HEIGHT: f32 = 138.;
const WORD_HEIGHT: f32 = 22.;
const WORD_WIDTH: f32 = 110.;
const WORD_LIST_WIDTH: f32 = WORD_BETWEEN_PAD + WORD_WIDTH + WORD_WIDTH;
const WORD_BETWEEN_PAD: f32 = 20.;

const CONGRATS_ENTITY_HEIGHT: f32 = 40.0;
const CONGRATS_ENTITY_WIDTH: f32 = 80.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, EnumIter)]
pub enum GameLayoutEntity {
    TopBar,
    TextArea,
    Grid,
    WordList,
}

impl std::fmt::Display for GameLayoutEntity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use GameLayoutEntity::*;
        match self {
            TopBar => write!(f, "TopBar"),
            TextArea => write!(f, "TextArea"),
            Grid => write!(f, "Grid"),
            WordList => write!(f, "WordList"),
        }
    }
}

impl LayoutStructure for GameLayoutEntity {
    type Context = ();
    type Iterator = <Self as IntoEnumIterator>::Iterator;

    fn iter_all() -> Self::Iterator {
        Self::iter()
    }

    fn pick(point: Vec2, context: &Self::Context) -> Option<Self> {
        for item in Self::iter() {
            if item.rect(context).contains(point) {
                return Some(item);
            }
        }
        return None;
    }

    //const ROOT: Self = GameLayoutEntity::Root;
    ///The size on a 320x568 canvas
    fn size(&self, _context: &()) -> Vec2 {
        match self {
            GameLayoutEntity::TopBar => Vec2 {
                x: IDEAL_WIDTH,
                y: TOP_BAR_ICON_SIZE,
            },
            GameLayoutEntity::TextArea => Vec2 {
                x: IDEAL_WIDTH,
                y: TEXT_AREA_HEIGHT,
            },
            GameLayoutEntity::Grid => Vec2 {
                x: IDEAL_WIDTH,
                y: IDEAL_WIDTH,
            },

            GameLayoutEntity::WordList => Vec2 {
                x: WORD_LIST_WIDTH,
                y: WORD_LIST_HEIGHT,
            },
        }
    }
    fn location(&self, _context: &()) -> Vec2 {
        match self {
            GameLayoutEntity::TopBar => Vec2::ZERO,

            GameLayoutEntity::TextArea => Vec2 {
                x: 0.,
                y: TOP_BAR_ICON_SIZE,
            },
            GameLayoutEntity::Grid => Vec2 {
                x: 0.,
                y: TOP_BAR_ICON_SIZE + TEXT_AREA_HEIGHT,
            },
            GameLayoutEntity::WordList => Vec2 {
                x: (IDEAL_WIDTH - WORD_LIST_WIDTH) / 2.,
                y: TOP_BAR_ICON_SIZE + TEXT_AREA_HEIGHT + GRID_SIZE,
            },
        }
    }
}

impl LayoutStructure for TextItem {
    type Context = ();
    type Iterator = <Self as IntoEnumIterator>::Iterator;

    fn iter_all() -> Self::Iterator {
        Self::iter()
    }

    fn pick(point: Vec2, context: &Self::Context) -> Option<Self> {
        for x in Self::iter() {
            if x.rect(context).contains(point) {
                return Some(x);
            }
        }
        return None;
    }

    fn size(&self, _context: &Self::Context) -> Vec2 {
        Vec2 {
            x: TEXT_ITEM_WIDTH,
            y: TEXT_ITEM_HEIGHT,
        }
    }

    fn location(&self, _context: &Self::Context) -> Vec2 {
        Vec2 {
            x: (IDEAL_WIDTH - TEXT_ITEM_WIDTH) / 2.,
            y: TOP_BAR_ICON_SIZE
                + Spacing::SpaceAround.apply(TEXT_AREA_HEIGHT, TEXT_ITEM_HEIGHT, 2, self.index()),
        }
    }
}

impl LayoutStructure for TopBarButton {
    type Context = ();
    type Iterator = <Self as IntoEnumIterator>::Iterator;

    fn pick(point: Vec2, context: &Self::Context) -> Option<Self> {
        for x in Self::iter() {
            if x.rect(context).contains(point) {
                return Some(x);
            }
        }
        return None;
    }

    fn size(&self, _context: &Self::Context) -> Vec2 {
        Vec2 {
            x: TOP_BAR_ICON_SIZE,
            y: TOP_BAR_ICON_SIZE,
        }
    }

    fn location(&self, _context: &Self::Context) -> Vec2 {
        Vec2 {
            x: Spacing::SpaceBetween.apply(IDEAL_WIDTH, TOP_BAR_ICON_SIZE, 3, self.index()),
            y: 0.,
        }
    }

    fn iter_all() -> Self::Iterator {
        Self::iter()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, EnumIter)]
pub enum CongratsLayoutEntity {
    Time,
    ShareButton,
    NextButton,
}

impl CongratsLayoutEntity {
    pub const fn index(&self) -> usize {
        match self {
            CongratsLayoutEntity::Time => 0,
            CongratsLayoutEntity::ShareButton => 1,
            CongratsLayoutEntity::NextButton => 2,
        }
    }
}

impl LayoutStructure for CongratsLayoutEntity {
    type Context = ();
    type Iterator = <Self as IntoEnumIterator>::Iterator;

    fn pick(point: Vec2, context: &Self::Context) -> Option<Self> {
        for x in Self::iter() {
            if x.rect(context).contains(point) {
                return Some(x);
            }
        }
        return None;
    }

    fn size(&self, _context: &Self::Context) -> Vec2 {
        Vec2 {
            x: CONGRATS_ENTITY_WIDTH,
            y: CONGRATS_ENTITY_HEIGHT,
        }
    }

    fn location(&self, _context: &Self::Context) -> Vec2 {
        Vec2 {
            x: (IDEAL_WIDTH - CONGRATS_ENTITY_WIDTH) / 2.,
            y: TOP_BAR_ICON_SIZE
                + TEXT_AREA_HEIGHT
                + Spacing::Centre.apply(GRID_SIZE, CONGRATS_ENTITY_HEIGHT, 3, self.index()),
        }
    }

    fn iter_all() -> Self::Iterator {
        Self::iter()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct LayoutWordTile(pub WordTile);

impl LayoutStructure for LayoutWordTile {
    type Context = ();
    type Iterator = LayoutWordTileIter;

    fn iter_all() -> Self::Iterator {
        LayoutWordTileIter::default()
    }

    fn pick(point: Vec2, context: &Self::Context) -> Option<Self> {
        for x in Self::iter_all() {
            if x.rect(context).contains(point) {
                return Some(x);
            }
        }
        return None;
    }

    fn size(&self, _context: &Self::Context) -> Vec2 {
        Vec2 {
            x: WORD_WIDTH,
            y: WORD_HEIGHT,
        }
    }

    fn location(&self, context: &Self::Context) -> Vec2 {
        //todo use flex
        GameLayoutEntity::WordList
            .location(context)
            .add(tile_offset(
                self.0,
                Spacing::SpaceAround,
                Spacing::SpaceAround,
                GameLayoutEntity::WordList.size(context),
                self.size(context),
            ))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct LayoutGridTile(pub Tile);

impl LayoutStructure for LayoutGridTile {
    type Context = ();
    type Iterator = LayoutGridTileIter;

    fn pick(point: Vec2, context: &Self::Context) -> Option<Self> {
        let grid_rect = GameLayoutEntity::Grid.rect(context);

        let scaled = grid_rect.scaled_inside(point)?;

        let x = (scaled.x * 4.0).floor() as u8;
        let y = (scaled.y * 4.0).floor() as u8;

        let tile = Self(Tile::try_new(x, y)?);

        if tile.rect(context).contains(point) {
            return Some(tile);
        }
        return None;
    }

    fn size(&self, _context: &Self::Context) -> Vec2 {
        Vec2 {
            x: GRID_TILE_SIZE,
            y: GRID_TILE_SIZE,
        }
    }

    fn location(&self, context: &Self::Context) -> Vec2 {
        GameLayoutEntity::Grid.location(context).add(tile_offset(
            self.0,
            Spacing::SpaceAround,
            Spacing::SpaceAround,
            GameLayoutEntity::Grid.size(context),
            self.size(context),
        ))
    }

    fn iter_all() -> Self::Iterator {
        LayoutGridTileIter::default()
    }
}

#[derive(Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)] //TODO use version in geometrid
pub struct LayoutWordTileIter {
    inner: u8,
}

impl Iterator for LayoutWordTileIter {
    type Item = LayoutWordTile;

    fn next(&mut self) -> Option<Self::Item> {
        let ret = geometrid::tile::Tile::<2, 5>::try_from_inner(self.inner)?;
        self.inner = self.inner.saturating_add(1);
        Some(LayoutWordTile(ret))
    }
}

#[derive(Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)] //TODO use version in geometrid
pub struct LayoutGridTileIter {
    inner: u8,
}

impl Iterator for LayoutGridTileIter {
    type Item = LayoutGridTile;

    fn next(&mut self) -> Option<Self::Item> {
        let ret = geometrid::tile::Tile::<4, 4>::try_from_inner(self.inner)?;
        self.inner = self.inner.saturating_add(1);
        Some(LayoutGridTile(ret))
    }
}

#[cfg(test)]
mod tests {
    use crate::{layout::*, layout_sizing::LayoutSizing};

    // TODO check that all children are contained within parents
    // TODO check that all siblings do not intersect each other

    #[test]
    fn test_picking_all() {
        test_picking::<GameLayoutEntity>(&());
        test_picking::<TopBarButton>(&());
        test_picking::<TextItem>(&());
        test_picking::<LayoutGridTile>(&());
        test_picking::<LayoutWordTile>(&());
    }

    fn test_picking<T: LayoutStructure + Copy>(context: &T::Context) {
        for entity in T::iter_all() {
            let rect = entity.rect(context);

            // let top_left_expected = T::pick(rect.top_left, context);

            // assert_eq!(Some(entity), top_left_expected, "Top left");

            let centre_expected = T::pick(rect.centre(), context);

            assert_eq!(Some(entity), centre_expected, "Centre");
        }
    }

    #[test]
    fn svg() {
        let size = Vec2 {
            x: (IDEAL_WIDTH) as f32,
            y: (IDEAL_HEIGHT) as f32,
        };

        let layout = LayoutSizing::from_page_size(size, IDEAL_RATIO, IDEAL_WIDTH);

        let mut svg = format!(
            r#"
        <svg version="1.1" id="Layer_1" xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink"
            viewBox="0 0 {} {}" xml:space="preserve">
        "#,
            size.x, size.y
        );

        for layout_entity in GameLayoutEntity::iter() {
            let layout_size = layout.get_size(&layout_entity, &());
            let (width, height) = (layout_size.x, layout_size.y);
            let Vec2 { x, y } = layout.get_location(&layout_entity, &());

            let color = match layout_entity {
                GameLayoutEntity::TopBar => "blue",

                GameLayoutEntity::TextArea => "coral",

                GameLayoutEntity::Grid => "indigo",

                GameLayoutEntity::WordList => "mediumblue",
            };

            let id = layout_entity.to_string();

            svg.push_str(format!(r#"<rect id="{id}" x="{x}" y="{y}" width="{width}" height="{height}" fill="{color}" opacity="0.8" />"#).as_str());
            svg.push('\n');
        }

        svg.push_str("</svg>");

        println!("{svg}");
    }
}
