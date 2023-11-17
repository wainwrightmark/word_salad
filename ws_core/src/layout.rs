use glam::{IVec2, Vec2};
use strum::{Display, EnumIter};

pub type Tile = geometrid::tile::Tile<4, 4>;
pub type WordTile = geometrid::tile::Tile<2, 5>;

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

const IDEAL_WIDTH: i32 = 320;
const IDEAL_HEIGHT: i32 = 568;
const IDEAL_RATIO: f32 = IDEAL_WIDTH as f32 / IDEAL_HEIGHT as f32;

const TOP_BAR_ICON_SIZE: i32 = 40;
const TEXT_ITEM_HEIGHT: i32 = 30;
const TEXT_ITEM_WIDTH: i32 = 300;

const TEXT_AREA_HEIGHT: i32 = 70;

const GRID_TILE_SIZE: i32 = 72;
const GRID_SIZE: i32 = 320;

const WORD_LIST_HEIGHT: i32 = 138;
const WORD_HEIGHT: i32 = 22;
const WORD_WIDTH: i32 = 110;
const WORD_LIST_WIDTH: i32 = WORD_BETWEEN_PAD + WORD_WIDTH + WORD_WIDTH;
const WORD_BETWEEN_PAD: i32 = 20;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Spacing {
    SpaceBetween,
    SpaceAround,
}

impl Spacing {
    const fn apply(
        &self,
        parent_ideal_length: i32,
        child_ideal_length: i32,
        num_children: usize,
        child_index: usize,
    ) -> i32 {
        let total_padding = parent_ideal_length - (num_children as i32 * child_ideal_length);
        let child_index = child_index as i32;
        let num_children = num_children as i32;
        match self {
            Spacing::SpaceBetween => {
                if num_children == 0 {
                    return 0;
                } else if num_children == 1 {
                    return total_padding / 2;
                } else {
                    let padding_between_children = total_padding / num_children.saturating_sub(1);
                    (padding_between_children + child_ideal_length) * child_index
                }
            }
            Spacing::SpaceAround => {
                if num_children == 0 {
                    return 0;
                } else {
                    let left_or_right_padding = total_padding / (num_children * 2);

                    let paddings = 1 + (child_index * 2);

                    (paddings * left_or_right_padding) + (child_index * child_ideal_length)
                }
            }
        }
    }
}

const fn tile_offset<const WIDTH: u8, const HEIGHT: u8>(
    tile: geometrid::tile::Tile<WIDTH, HEIGHT>,
    h_spacing: Spacing,
    v_spacing: Spacing,
    full_size: IVec2,
    tile_size: IVec2,
) -> IVec2 {
    let x = h_spacing.apply(full_size.x, tile_size.x, WIDTH as usize, tile.x() as usize);
    let y = v_spacing.apply(full_size.y, tile_size.y, HEIGHT as usize, tile.y() as usize);

    IVec2 { x, y }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum LayoutEntity {
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

impl LayoutEntity {
    pub const fn pick(point: &IVec2) -> Option<Self> {
        if !Self::Root.rect().contains(point) {
            return None;
        }

        let mut current = Self::Root;

        'outer: loop {
            let children = current.children();
            let mut child_index = 0;
            loop {
                if child_index >= children.len() {
                    break 'outer;
                }
                let child = children[child_index];

                if child.rect().contains(point) {
                    current = child;
                    continue 'outer;
                }

                child_index += 1;
            }
        }
        Some(current)
    }

    pub const fn rect(&self) -> IRect {
        IRect {
            top_left: self.location(),
            extents: self.size(),
        }
    }

    ///The size on a 320x568 canvas
    pub const fn size(&self) -> IVec2 {
        match self {
            LayoutEntity::Root => IVec2 {
                x: IDEAL_WIDTH,
                y: IDEAL_HEIGHT,
            },
            LayoutEntity::TopBar => IVec2 {
                x: IDEAL_WIDTH,
                y: TOP_BAR_ICON_SIZE,
            },
            LayoutEntity::TextArea => IVec2 {
                x: IDEAL_WIDTH,
                y: TEXT_AREA_HEIGHT,
            },
            LayoutEntity::Grid => IVec2 {
                x: IDEAL_WIDTH,
                y: IDEAL_WIDTH,
            },
            LayoutEntity::TopBarItem(_) => IVec2 {
                x: TOP_BAR_ICON_SIZE,
                y: TOP_BAR_ICON_SIZE,
            },
            LayoutEntity::TextAreaItem(_) => IVec2 {
                x: TEXT_ITEM_WIDTH,
                y: TEXT_ITEM_HEIGHT,
            },
            LayoutEntity::GridTile(_) => IVec2 {
                x: GRID_TILE_SIZE,
                y: GRID_TILE_SIZE,
            },
            LayoutEntity::WordList => IVec2 {
                x: WORD_LIST_WIDTH,
                y: WORD_LIST_HEIGHT,
            },
            LayoutEntity::Word(_) => IVec2 {
                x: WORD_WIDTH,
                y: WORD_HEIGHT,
            },
        }
    }
    pub const fn location(&self) -> IVec2 {
        match self {
            LayoutEntity::Root => IVec2 { x: 0, y: 0 },
            LayoutEntity::TopBar => IVec2 { x: 0, y: 0 },
            LayoutEntity::TopBarItem(item) => IVec2 {
                x: Spacing::SpaceBetween.apply(IDEAL_WIDTH, TOP_BAR_ICON_SIZE, 3, item.index()),
                y: 0,
            },
            LayoutEntity::TextArea => IVec2 {
                x: 0,
                y: TOP_BAR_ICON_SIZE,
            },
            LayoutEntity::TextAreaItem(item) => IVec2 {
                x: (IDEAL_WIDTH - TEXT_ITEM_WIDTH) / 2,
                y: TOP_BAR_ICON_SIZE
                    + Spacing::SpaceAround.apply(
                        TEXT_AREA_HEIGHT,
                        TEXT_ITEM_HEIGHT,
                        2,
                        item.index(),
                    ),
            },
            LayoutEntity::Grid => IVec2 {
                x: 0,
                y: TOP_BAR_ICON_SIZE + TEXT_AREA_HEIGHT,
            },
            LayoutEntity::GridTile(tile) => {
                Self::Grid.location().saturating_add(tile_offset(
                    *tile,
                    Spacing::SpaceAround,
                    Spacing::SpaceAround,
                    Self::Grid.size(),
                    Self::GridTile(*tile).size(),
                ))
            }
            LayoutEntity::WordList => IVec2 {
                x: (IDEAL_WIDTH - WORD_LIST_WIDTH) / 2,
                y: TOP_BAR_ICON_SIZE + TEXT_AREA_HEIGHT + GRID_SIZE,
            },
            LayoutEntity::Word(tile) => {
                Self::WordList.location().saturating_add(tile_offset(
                    *tile,
                    Spacing::SpaceAround,
                    Spacing::SpaceAround,
                    Self::WordList.size(),
                    Self::Word(*tile).size(),
                ))
            }
        }
    }

    pub fn display(&self) -> String {
        use LayoutEntity::*;
        match self {
            Root => "Root".to_string(),
            TopBar => "TopBar".to_string(),
            TextArea => "TextArea".to_string(),
            Grid => "Grid".to_string(),

            GridTile(tile) => format!("GridTile_{}_{}", tile.x(), tile.y()),
            WordList => "WordList".to_string(),
            Word(tile) => format!("Word_{}_{}", tile.x(), tile.y()),
            TopBarItem(item) => item.to_string(),
            TextAreaItem(item) => item.to_string(),
        }
    }

    pub fn all() -> Vec<Self> {
        let mut vec = vec![Self::Root];

        let mut index = 0;

        while let Some(node) = vec.get(index) {
            let node = node.clone();
            vec.extend(node.children());
            index += 1;
        }
        vec
    }

    pub const fn children<'t>(&'t self) -> &'static [Self] {
        use LayoutEntity::*;

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

const ALL_WORD_TILES: [LayoutEntity; 10] = [
    LayoutEntity::Word(WordTile::new_const::<0, 0>()),
    LayoutEntity::Word(WordTile::new_const::<0, 1>()),
    LayoutEntity::Word(WordTile::new_const::<0, 2>()),
    LayoutEntity::Word(WordTile::new_const::<0, 3>()),
    LayoutEntity::Word(WordTile::new_const::<0, 4>()),
    LayoutEntity::Word(WordTile::new_const::<1, 0>()),
    LayoutEntity::Word(WordTile::new_const::<1, 1>()),
    LayoutEntity::Word(WordTile::new_const::<1, 2>()),
    LayoutEntity::Word(WordTile::new_const::<1, 3>()),
    LayoutEntity::Word(WordTile::new_const::<1, 4>()),
];

const ALL_GRID_TILES: [LayoutEntity; 16] = [
    LayoutEntity::GridTile(Tile::new_const::<0, 0>()),
    LayoutEntity::GridTile(Tile::new_const::<0, 1>()),
    LayoutEntity::GridTile(Tile::new_const::<0, 2>()),
    LayoutEntity::GridTile(Tile::new_const::<0, 3>()),
    LayoutEntity::GridTile(Tile::new_const::<1, 0>()),
    LayoutEntity::GridTile(Tile::new_const::<1, 1>()),
    LayoutEntity::GridTile(Tile::new_const::<1, 2>()),
    LayoutEntity::GridTile(Tile::new_const::<1, 3>()),
    LayoutEntity::GridTile(Tile::new_const::<2, 0>()),
    LayoutEntity::GridTile(Tile::new_const::<2, 1>()),
    LayoutEntity::GridTile(Tile::new_const::<2, 2>()),
    LayoutEntity::GridTile(Tile::new_const::<2, 3>()),
    LayoutEntity::GridTile(Tile::new_const::<3, 0>()),
    LayoutEntity::GridTile(Tile::new_const::<3, 1>()),
    LayoutEntity::GridTile(Tile::new_const::<3, 2>()),
    LayoutEntity::GridTile(Tile::new_const::<3, 3>()),
];

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct IRect {
    pub top_left: IVec2,
    pub extents: IVec2,
}

impl IRect {
    pub const fn contains(&self, point: &IVec2) -> bool {
        const fn contains_1d(min: i32, length: i32, p: i32) -> bool {
            p >= min && p < min + length
        }

        contains_1d(self.top_left.x, self.extents.x, point.x)
            && contains_1d(self.top_left.y, self.extents.y, point.y)
    }
}

impl Into<Rect> for IRect{
    fn into(self) -> Rect {
        Rect { top_left: self.top_left.as_vec2(), extents: self.extents.as_vec2() }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    pub top_left: Vec2,
    pub extents: Vec2,
}

impl Rect {
    pub fn contains(&self, point: &Vec2) -> bool {
        fn contains_1d(min: f32, length: f32, p: f32) -> bool {
            p > min && p < min + length
        }

        contains_1d(self.top_left.x, self.extents.x, point.x)
            && contains_1d(self.top_left.y, self.extents.y, point.y)
    }

    pub fn centre(&self) -> Vec2 {
        Vec2 {
            x: self.top_left.x + (self.extents.x * 0.5),
            y: self.top_left.y + (self.extents.y * 0.5),
        }
    }
}


pub struct Layout {
    pub size_ratio: f32,
    pub left_pad: f32,
}

impl Layout {
    pub fn from_page_size(page_size: Vec2) -> Self {
        let ratio = page_size.x / page_size.y;

        //let used_y: i32;
        let used_x: f32;

        if ratio >= IDEAL_RATIO {
            // There is additional x, so just left pad everything
            //used_y = page_size.y;
            used_x = page_size.y as f32 * IDEAL_RATIO;
        } else {
            // There is additional y, so don't use the bottom area
            used_x = page_size.x as f32;
            //used_y = page_size.x / IDEAL_RATIO;
        }

        let left_pad = ((page_size.x as f32 - used_x) / 2.).round();
        let size_ratio = used_x / IDEAL_WIDTH as f32;

        Self {
            size_ratio,
            left_pad,
        }
    }

    pub fn try_pick_entity(&self, position: Vec2, tolerance: f32) -> Option<LayoutEntity> {
        let x = position.x - self.left_pad;
        let y = position.y;

        let x = (x / self.size_ratio).round() as i32;
        let y = (y / self.size_ratio).round() as i32;

        let location = IVec2 { x, y };

        let entity = LayoutEntity::pick(&location)?;

        if tolerance >= 1.0 {
            return Some(entity);
        }

        let rect: Rect = entity.rect().into();

        let dist = rect.centre().distance(location.as_vec2());
        let size_squared = rect.extents.length();

        if dist / size_squared < tolerance {
            return Some(entity);
        }
        return None;
    }

    pub fn get_size(&self, entity: LayoutEntity) -> Vec2 {
        let v2: Vec2 = entity.size().as_vec2();
        v2 * self.size_ratio
    }

    pub fn get_location(&self, entity: LayoutEntity) -> glam::Vec2 {
        let Vec2 { x, y } = entity.location().as_vec2();

        Vec2 {
            x: self.left_pad + (self.size_ratio * x as f32),
            y: (self.size_ratio * y as f32),
        }
    }

    pub fn get_rect(&self, entity: LayoutEntity)-> Rect{
        Rect { top_left: self.get_location(entity), extents: self.get_size(entity) }
    }
}

#[cfg(test)]
mod tests {
    use crate::layout::*;

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
            x: (IDEAL_WIDTH ) as f32,
            y: (IDEAL_HEIGHT ) as f32,
        };

        let layout = Layout::from_page_size(size);

        let mut svg = format!(
            r#"
        <svg version="1.1" id="Layer_1" xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink"
            viewBox="0 0 {} {}" xml:space="preserve">
        "#,
            size.x, size.y
        );

        for layout_entity in LayoutEntity::all() {
            let layout_size = layout.get_size(layout_entity);
            let (width, height) = (layout_size.x, layout_size.y);
            let Vec2 { x, y } = layout.get_location(layout_entity);

            let color = match layout_entity {
                LayoutEntity::Root => "black",
                LayoutEntity::TopBar => "blue",
                LayoutEntity::TopBarItem(_) => "beige",

                LayoutEntity::TextArea => "coral",
                LayoutEntity::TextAreaItem(_) => "deeppink",
                LayoutEntity::Grid => "indigo",
                LayoutEntity::GridTile { .. } => "lightpink",
                LayoutEntity::WordList => "mediumblue",
                LayoutEntity::Word { .. } => "mediumspringgreen",
            };

            let id = layout_entity.display();

            svg.push_str(format!(r#"<rect id="{id}" x="{x}" y="{y}" width="{width}" height="{height}" fill="{color}" opacity="0.8" />"#).as_str());
            svg.push('\n');
        }

        svg.push_str("</svg>");

        println!("{svg}");
    }
}
