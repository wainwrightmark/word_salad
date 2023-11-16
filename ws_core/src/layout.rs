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

const IDEAL_WIDTH: isize = 320;
const IDEAL_HEIGHT: isize = 568;
const IDEAL_RATIO: f32 = IDEAL_WIDTH as f32 / IDEAL_HEIGHT as f32;

const TOP_BAR_ICON_SIZE: isize = 40;
const TEXT_ITEM_HEIGHT: isize = 30;
const TEXT_ITEM_WIDTH: isize = 300;

const TEXT_AREA_HEIGHT: isize = 70;

const GRID_TILE_SIZE: isize = 72;
const GRID_SIZE: isize = 320;

const WORD_LIST_HEIGHT: isize = 138;
const WORD_HEIGHT: isize = 22;
const WORD_WIDTH: isize = 110;
const WORD_LIST_WIDTH: isize = WORD_BETWEEN_PAD + WORD_WIDTH + WORD_WIDTH;
const WORD_BETWEEN_PAD: isize = 20;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Spacing {
    SpaceBetween,
    SpaceAround,
}

impl Spacing {
    const fn apply(
        &self,
        parent_ideal_length: isize,
        child_ideal_length: isize,
        num_children: usize,
        child_index: usize,
    ) -> isize {
        let total_padding = parent_ideal_length - (num_children as isize * child_ideal_length);
        let child_index = child_index as isize;
        let num_children = num_children as isize;
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
    full_size: Size,
    tile_size: Size,
) -> Location {
    let x = h_spacing.apply(
        full_size.width,
        tile_size.width,
        WIDTH as usize,
        tile.x() as usize,
    );
    let y = v_spacing.apply(
        full_size.height,
        tile_size.height,
        HEIGHT as usize,
        tile.y() as usize,
    );

    Location { x, y }
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
    pub const fn pick(point: &Location) -> Option<Self> {
        if !Self::Root
            .ideal_location()
            .contains(&Self::Root.default_size(), point)
        {
            return None;
        }

        let mut current = Self::Root;

        'outer: loop {
            let children = current.children();
            let mut child_index = 0;
            loop {
                if child_index <= children.len() {
                    break 'outer;
                }
                let child = children[child_index];

                if child
                    .ideal_location()
                    .contains(&child.default_size(), point)
                {
                    current = child;
                    continue 'outer;
                }

                child_index += 1;
            }
        }
        Some(current)
    }

    ///The size on a 320x568 canvas
    pub const fn default_size(&self) -> Size {
        match self {
            LayoutEntity::Root => Size {
                width: IDEAL_WIDTH,
                height: IDEAL_HEIGHT,
            },
            LayoutEntity::TopBar => Size {
                width: IDEAL_WIDTH,
                height: TOP_BAR_ICON_SIZE,
            },
            LayoutEntity::TextArea => Size {
                width: IDEAL_WIDTH,
                height: TEXT_AREA_HEIGHT,
            },
            LayoutEntity::Grid => Size {
                width: IDEAL_WIDTH,
                height: IDEAL_WIDTH,
            },
            LayoutEntity::TopBarItem(_) => Size {
                width: TOP_BAR_ICON_SIZE,
                height: TOP_BAR_ICON_SIZE,
            },
            LayoutEntity::TextAreaItem(_) => Size {
                width: TEXT_ITEM_WIDTH,
                height: TEXT_ITEM_HEIGHT,
            },
            LayoutEntity::GridTile(_) => Size {
                width: GRID_TILE_SIZE,
                height: GRID_TILE_SIZE,
            },
            LayoutEntity::WordList => Size {
                width: WORD_LIST_WIDTH,
                height: WORD_LIST_HEIGHT,
            },
            LayoutEntity::Word(_) => Size {
                width: WORD_WIDTH,
                height: WORD_HEIGHT,
            },
        }
    }
    pub const fn ideal_location(&self) -> Location {
        match self {
            LayoutEntity::Root => Location { x: 0, y: 0 },
            LayoutEntity::TopBar => Location { x: 0, y: 0 },
            LayoutEntity::TopBarItem(item) => Location {
                x: Spacing::SpaceBetween.apply(IDEAL_WIDTH, TOP_BAR_ICON_SIZE, 3, item.index()),
                y: 0,
            },
            LayoutEntity::TextArea => Location {
                x: 0,
                y: TOP_BAR_ICON_SIZE,
            },
            LayoutEntity::TextAreaItem(item) => Location {
                x: (IDEAL_WIDTH - TEXT_ITEM_WIDTH) / 2,
                y: TOP_BAR_ICON_SIZE
                    + Spacing::SpaceAround.apply(
                        TEXT_AREA_HEIGHT,
                        TEXT_ITEM_HEIGHT,
                        2,
                        item.index(),
                    ),
            },
            LayoutEntity::Grid => Location {
                x: 0,
                y: TOP_BAR_ICON_SIZE + TEXT_AREA_HEIGHT,
            },
            LayoutEntity::GridTile(tile) => Self::Grid.ideal_location().add(tile_offset(
                *tile,
                Spacing::SpaceAround,
                Spacing::SpaceAround,
                Self::Grid.default_size(),
                Self::GridTile(*tile).default_size(),
            )),
            LayoutEntity::WordList => Location {
                x: (IDEAL_WIDTH - WORD_LIST_WIDTH) / 2,
                y: TOP_BAR_ICON_SIZE + TEXT_AREA_HEIGHT + GRID_SIZE,
            },
            LayoutEntity::Word(tile) => Self::WordList.ideal_location().add(tile_offset(
                *tile,
                Spacing::SpaceAround,
                Spacing::SpaceAround,
                Self::WordList.default_size(),
                Self::Word(*tile).default_size(),
            )),
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

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Location {
    ///distance from the left
    pub x: isize,
    /// distance from the top
    pub y: isize,
}

impl Location {
    pub const fn add(self, other: Location) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }

    pub const fn contains(&self, size: &Size, point: &Self) -> bool {
        const fn contains_1d(min: isize, length: isize, p: isize) -> bool {
            p > min && p < min + length
        }

        contains_1d(self.x, size.width, point.x) && contains_1d(self.y, size.height, point.y)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Size {
    pub width: isize,
    pub height: isize,
}

impl std::ops::Mul<f32> for Size {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        Self {
            width: (self.width as f32 * rhs).round() as isize,
            height: (self.height as f32 * rhs).round() as isize,
        }
    }
}

pub struct Layout {
    pub size_ratio: f32,
    pub left_pad: isize,
}

impl Layout {
    pub fn from_page_size(page_size: Size) -> Self {
        let ratio = page_size.width as f32 / page_size.height as f32;

        //let used_height: isize;
        let used_width: f32;

        if ratio >= IDEAL_RATIO {
            // There is additional width, so just left pad everything
            //used_height = page_size.height;
            used_width = page_size.height as f32 * IDEAL_RATIO;
        } else {
            // There is additional height, so don't use the bottom area
            used_width = page_size.width as f32;
            //used_height = page_size.width / IDEAL_RATIO;
        }

        let left_pad = ((page_size.width as f32 - used_width) / 2.).round() as isize;
        let size_ratio = used_width / IDEAL_WIDTH as f32;

        Self {
            size_ratio,
            left_pad,
        }
    }

    pub fn pick_entity(&self, location: Location) -> Option<LayoutEntity> {
        let x = location.x - self.left_pad;
        let y = location.y;

        let x = (x as f32 / self.size_ratio).round() as isize;
        let y = (y as f32 / self.size_ratio).round() as isize;

        let adjusted_location = Location { x, y };

        LayoutEntity::pick(&adjusted_location)
    }

    pub fn get_size(&self, entity: LayoutEntity) -> Size {
        entity.default_size() * self.size_ratio
    }

    pub fn get_location(&self, entity: LayoutEntity) -> Location {
        let Location { x, y } = entity.ideal_location();

        Location {
            x: self.left_pad + (self.size_ratio * x as f32).round() as isize,
            y: (self.size_ratio * y as f32).round() as isize,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::*;

    // TODO check that all children are contained within parents
    // TODO check that all siblings do not intersect each other
    // TODO check that each item can be picked

    #[test]
    fn svg() {
        let size = Size {
            width: IDEAL_WIDTH + 100,
            height: IDEAL_HEIGHT + 100,
        };

        let layout = Layout::from_page_size(size);

        let mut svg = format!(
            r#"
        <svg version="1.1" id="Layer_1" xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink"
            viewBox="0 0 {} {}" xml:space="preserve">
        "#,
            size.width, size.height
        );

        for layout_entity in LayoutEntity::all() {
            let Size { width, height } = layout.get_size(layout_entity);
            let Location { x, y } = layout.get_location(layout_entity);

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

            svg.push_str(format!(r#"<rect id="{id}" width="{width}" height="{height}" x="{x}" y="{y}" fill="{color}" opacity="0.8" />"#).as_str());
            svg.push('\n');
        }

        svg.push_str("</svg>");

        println!("{svg}");
    }
}
