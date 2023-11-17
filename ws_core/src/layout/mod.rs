use std::ops::Add;

use glam::Vec2;
use strum::{Display, EnumIter};

use self::{layout_structure::LayoutStructure, spacing::{Spacing, tile_offset}};

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



#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum GameLayoutEntity {
    Root,

    TopBar,
    TopBarItem(TopBarButton),
    TextArea,
    TextAreaItem(TextItem),

    Grid,
    GridTile(Tile),

    WordList,
    Word(WordTile),
}

impl std::fmt::Display for GameLayoutEntity{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use GameLayoutEntity::*;
            match self {
                Root => write!(f, "Root"),
                TopBar =>  write!(f,"TopBar"),
                TextArea =>  write!(f,"TextArea"),
                Grid =>  write!(f,"Grid"),

                GridTile(tile) => write!(f,"GridTile_{}_{}", tile.x(), tile.y()),
                WordList => write!(f,"WordList"),
                Word(tile) => write!(f,"Word_{}_{}", tile.x(), tile.y()),
                TopBarItem(item) => item.fmt(f),
                TextAreaItem(item) => item.fmt(f),
            }
    }
}

impl LayoutStructure for GameLayoutEntity {

    const ROOT: Self = GameLayoutEntity::Root;
    ///The size on a 320x568 canvas
    fn size(&self) -> Vec2 {
        match self {
            GameLayoutEntity::Root => Vec2 {
                x: IDEAL_WIDTH,
                y: IDEAL_HEIGHT,
            },
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
            GameLayoutEntity::TopBarItem(_) => Vec2 {
                x: TOP_BAR_ICON_SIZE,
                y: TOP_BAR_ICON_SIZE,
            },
            GameLayoutEntity::TextAreaItem(_) => Vec2 {
                x: TEXT_ITEM_WIDTH,
                y: TEXT_ITEM_HEIGHT,
            },
            GameLayoutEntity::GridTile(_) => Vec2 {
                x: GRID_TILE_SIZE,
                y: GRID_TILE_SIZE,
            },
            GameLayoutEntity::WordList => Vec2 {
                x: WORD_LIST_WIDTH,
                y: WORD_LIST_HEIGHT,
            },
            GameLayoutEntity::Word(_) => Vec2 {
                x: WORD_WIDTH,
                y: WORD_HEIGHT,
            },
        }
    }
    fn location(&self) -> Vec2 {
        match self {
            GameLayoutEntity::Root => Vec2::ZERO,
            GameLayoutEntity::TopBar => Vec2::ZERO,
            GameLayoutEntity::TopBarItem(item) => Vec2 {
                x: Spacing::SpaceBetween.apply(IDEAL_WIDTH, TOP_BAR_ICON_SIZE, 3, item.index()),
                y: 0.,
            },
            GameLayoutEntity::TextArea => Vec2 {
                x: 0.,
                y: TOP_BAR_ICON_SIZE,
            },
            GameLayoutEntity::TextAreaItem(item) => Vec2 {
                x: (IDEAL_WIDTH - TEXT_ITEM_WIDTH) / 2.,
                y: TOP_BAR_ICON_SIZE
                    + Spacing::SpaceAround.apply(
                        TEXT_AREA_HEIGHT,
                        TEXT_ITEM_HEIGHT,
                        2,
                        item.index(),
                    ),
            },
            GameLayoutEntity::Grid => Vec2 {
                x: 0.,
                y: TOP_BAR_ICON_SIZE + TEXT_AREA_HEIGHT,
            },
            GameLayoutEntity::GridTile(tile) => Self::Grid.location().add(tile_offset(
                *tile,
                Spacing::SpaceAround,
                Spacing::SpaceAround,
                Self::Grid.size(),
                Self::GridTile(*tile).size(),
            )),
            GameLayoutEntity::WordList => Vec2 {
                x: (IDEAL_WIDTH - WORD_LIST_WIDTH) / 2.,
                y: TOP_BAR_ICON_SIZE + TEXT_AREA_HEIGHT + GRID_SIZE,
            },
            GameLayoutEntity::Word(tile) => Self::WordList.location().add(tile_offset(
                *tile,
                Spacing::SpaceAround,
                Spacing::SpaceAround,
                Self::WordList.size(),
                Self::Word(*tile).size(),
            )),
        }
    }



    fn children(self) -> &'static [Self] {
        use GameLayoutEntity::*;

        let arr: &'static [Self] = match self {
            Root => &[TopBar, TextArea, Grid, WordList],

            TopBar => &[
                TopBarItem(TopBarButton::MenuBurgerButton),
                TopBarItem(TopBarButton::TimeCounter),
                TopBarItem(TopBarButton::HintCounter),
            ],
            TextArea => &[
                TextAreaItem(TextItem::PuzzleTitle),
                TextAreaItem(TextItem::PuzzleTheme),
            ],

            Grid => &ALL_GRID_TILES,
            WordList => &ALL_WORD_TILES,

            GridTile { .. } => &[],
            Word { .. } => &[],
            TopBarItem(_) => &[],
            TextAreaItem(_) => &[],
        };

        arr
    }
}

const ALL_WORD_TILES: [GameLayoutEntity; 10] = [
    GameLayoutEntity::Word(WordTile::new_const::<0, 0>()),
    GameLayoutEntity::Word(WordTile::new_const::<0, 1>()),
    GameLayoutEntity::Word(WordTile::new_const::<0, 2>()),
    GameLayoutEntity::Word(WordTile::new_const::<0, 3>()),
    GameLayoutEntity::Word(WordTile::new_const::<0, 4>()),
    GameLayoutEntity::Word(WordTile::new_const::<1, 0>()),
    GameLayoutEntity::Word(WordTile::new_const::<1, 1>()),
    GameLayoutEntity::Word(WordTile::new_const::<1, 2>()),
    GameLayoutEntity::Word(WordTile::new_const::<1, 3>()),
    GameLayoutEntity::Word(WordTile::new_const::<1, 4>()),
];

const ALL_GRID_TILES: [GameLayoutEntity; 16] = [
    GameLayoutEntity::GridTile(Tile::new_const::<0, 0>()),
    GameLayoutEntity::GridTile(Tile::new_const::<0, 1>()),
    GameLayoutEntity::GridTile(Tile::new_const::<0, 2>()),
    GameLayoutEntity::GridTile(Tile::new_const::<0, 3>()),
    GameLayoutEntity::GridTile(Tile::new_const::<1, 0>()),
    GameLayoutEntity::GridTile(Tile::new_const::<1, 1>()),
    GameLayoutEntity::GridTile(Tile::new_const::<1, 2>()),
    GameLayoutEntity::GridTile(Tile::new_const::<1, 3>()),
    GameLayoutEntity::GridTile(Tile::new_const::<2, 0>()),
    GameLayoutEntity::GridTile(Tile::new_const::<2, 1>()),
    GameLayoutEntity::GridTile(Tile::new_const::<2, 2>()),
    GameLayoutEntity::GridTile(Tile::new_const::<2, 3>()),
    GameLayoutEntity::GridTile(Tile::new_const::<3, 0>()),
    GameLayoutEntity::GridTile(Tile::new_const::<3, 1>()),
    GameLayoutEntity::GridTile(Tile::new_const::<3, 2>()),
    GameLayoutEntity::GridTile(Tile::new_const::<3, 3>()),
];





#[cfg(test)]
mod tests {
    use crate::{layout::*, layout_sizing::LayoutSizing};

    // TODO check that all children are contained within parents
    // TODO check that all siblings do not intersect each other
    // TODO check that each item can be picked

    // #[test]
    // fn test_picking(){
    //     for entity in LayoutEntity::all(){
    //         let rect = entity.rect();

    //         let top_left_expected =  LayoutEntity::pick(&rect.top_left);

    //         assert_eq!(Some(entity), top_left_expected, "Top left");

    //         // let bottom_right_expected = LayoutEntity::pick(&(rect.top_left + rect.extents));

    //         // assert_eq!(Some(entity), bottom_right_expected, "Bottom right");

    //         let centre_expected = LayoutEntity::pick(&(rect.top_left + (rect.extents / 2)));

    //         assert_eq!(Some(entity), centre_expected, "Centre");
    //     }
    // }

    #[test]
    fn svg() {
        let size = Vec2 {
            x: (IDEAL_WIDTH) as f32,
            y: (IDEAL_HEIGHT) as f32,
        };

        let layout = LayoutSizing::from_page_size(size, IDEAL_RATIO, IDEAL_WIDTH );

        let mut svg = format!(
            r#"
        <svg version="1.1" id="Layer_1" xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink"
            viewBox="0 0 {} {}" xml:space="preserve">
        "#,
            size.x, size.y
        );

        for layout_entity in GameLayoutEntity::all() {
            let layout_size = layout.get_size(layout_entity);
            let (width, height) = (layout_size.x, layout_size.y);
            let Vec2 { x, y } = layout.get_location(layout_entity);

            let color = match layout_entity {
                GameLayoutEntity::Root => "black",
                GameLayoutEntity::TopBar => "blue",
                GameLayoutEntity::TopBarItem(_) => "beige",

                GameLayoutEntity::TextArea => "coral",
                GameLayoutEntity::TextAreaItem(_) => "deeppink",
                GameLayoutEntity::Grid => "indigo",
                GameLayoutEntity::GridTile { .. } => "lightpink",
                GameLayoutEntity::WordList => "mediumblue",
                GameLayoutEntity::Word { .. } => "mediumspringgreen",
            };

            let id = layout_entity.to_string();

            svg.push_str(format!(r#"<rect id="{id}" x="{x}" y="{y}" width="{width}" height="{height}" fill="{color}" opacity="0.8" />"#).as_str());
            svg.push('\n');
        }

        svg.push_str("</svg>");

        println!("{svg}");
    }
}
