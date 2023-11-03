use crate::LetterCounts;
use itertools::Itertools;
use std::collections::{BTreeMap, BTreeSet};
use ws_core::{find_solution, ArrayVec, Character, CharsArray, Grid, GridSet, TileMap};

#[cfg(not(test))]
use log::{info, warn}; // Use log crate when building application

#[cfg(test)]
use std::{println as info, println as warn}; // Workaround to use prinltn! for logs.

type Tile = geometrid::tile::Tile<4, 4>;

pub fn try_make_grid(letters: LetterCounts, words: &Vec<CharsArray>) -> Option<Grid> {
    //info!("Try to make grid\n{letters:?}\n{words:?}", );
    let mut nodes_map: BTreeMap<Character, Vec<Node>> = Default::default();

    let mut next_node_id: u8 = 0;
    for character in letters.into_iter() {
        let node = Node {
            id: NodeId(next_node_id),
            character,
            constraints: Default::default(),
        };
        next_node_id += 1;
        match nodes_map.entry(character) {
            std::collections::btree_map::Entry::Vacant(v) => {
                v.insert(vec![node]);
            }
            std::collections::btree_map::Entry::Occupied(mut o) => {
                o.get_mut().push(node);
            }
        }
    }

    //add constraints from words
    let mut all_constraints_representable = true;

    for word in words {
        for (a, b) in word.iter().tuple_windows() {
            let added = try_add_constraint(a, b, &mut nodes_map);
            let added2 = try_add_constraint(b, a, &mut nodes_map);
            if !added && !added2 {
                all_constraints_representable = false
            }
        }
    }

    // #[cfg(test)]
    // {
    //     for (_, nodes) in nodes_map.iter() {
    //         for node in nodes {
    //             info!("{}: {} constraints", node.character, node.constraints.len());
    //         }
    //     }
    // }

    let grid: PartialGrid = Default::default();

    let nodes: Vec<Node> = nodes_map
        .into_iter()
        .flat_map(|x| x.1.into_iter())
        .sorted_by_key(|x| x.constraints.len())
        .collect_vec();

    let solution = grid.solve_recursive(&nodes, &nodes, 0)?;

    let solution_grid = solution.to_grid(&nodes);

    //info!("Solution found:\n{solution_grid}");
    for word in words {
        if find_solution(word, &solution_grid).is_none() {
            return None;
        }
    }

    Some(solution_grid)
}

fn try_add_constraint(
    from: &Character,
    to: &Character,
    nodes_map: &mut BTreeMap<Character, Vec<Node>>,
) -> bool {
    let Some(target_nodes) = nodes_map.get(&to) else {
        return false;
    };
    if target_nodes.len() == 0 {
        return false;
    }
    let constraint = if target_nodes.len() == 1 {
        Constraint::Single(target_nodes[0].id)
    } else {
        Constraint::OneOf(target_nodes.iter().map(|z| z.id).collect())
    };

    // let node_id = constraints.get(key)
    let Some(source_nodes) = nodes_map.get_mut(&from) else {
        return false;
    };
    if source_nodes.len() != 1 {
        return false;
    }

    match source_nodes.get_mut(0) {
        Some(source_node) => {
            source_node.constraints.insert(constraint);
            return true;
        }
        None => {
            return false;
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Node {
    pub id: NodeId,
    pub character: Character,
    pub constraints: BTreeSet<Constraint>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NodeId(u8);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Constraint {
    Single(NodeId),
    OneOf(ArrayVec<NodeId, 4>), //Use something like a bitset
}

impl Constraint {
    pub fn is_met(&self, tile: Tile, map: &BTreeMap<NodeId, Tile>) -> Option<bool> {
        match self {
            Constraint::Single(node_id) => match map.get(node_id) {
                Some(other_tile) => Some(tile.is_adjacent_to(other_tile)),
                None => None,
            },
            Constraint::OneOf(ids) => {
                let mut any_maybe = false;

                for node_id in ids {
                    match map.get(node_id) {
                        Some(other_tile) => {
                            if tile.is_adjacent_to(other_tile) {
                                return Some(true);
                            }
                        }
                        None => {
                            any_maybe = true;
                        }
                    }
                }

                if any_maybe {
                    None
                } else {
                    Some(false)
                }
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PartialGrid {
    pub used_grid: GridSet,
    pub map: BTreeMap<NodeId, Tile>,
    pub unchecked_constraints: Vec<(Tile, Constraint)>,
}

impl Default for PartialGrid {
    fn default() -> Self {
        Self {
            used_grid: Default::default(),
            map: Default::default(),
            unchecked_constraints: Default::default(),
        }
    }
}

/// Whether a tile is a corner
fn is_tile_corner(tile: &Tile) -> bool {
    matches!(tile.x(), 0 | 3) && matches!(tile.y(), 0 | 3)
}

/// Whether a tile is an edge or a corner
fn is_tile_edge(tile: &Tile) -> bool {
    matches!(tile.x(), 0 | 3) || matches!(tile.y(), 0 | 3)
}

impl PartialGrid {
    pub fn to_grid(&self, nodes: &Vec<Node>) -> Grid {
        let mut grid: Grid = Grid::from_fn(|_| Character::Blank);
        for node in nodes {
            if let Some(tile) = self.map.get(&node.id) {
                grid[*tile] = node.character;
            }
        }
        grid
    }

    pub fn solve_recursive(
        //change to an iterator
        &self,
        all_nodes: &Vec<Node>,
        nodes_to_add: &Vec<Node>,
        level: usize,
    ) -> Option<Self> {
        //info!("{g}\n\n", g = self.to_grid(all_nodes));

        let Some((index, node, potential_locations)) = nodes_to_add
            .iter()
            .enumerate()
            .map(|(index, node)| {
                let set = self.potential_locations(node);
                (index, node, set)
            })
            // .inspect(|f| {
            //     #[cfg(test)]
            //     if level == 0 {
            //         info!("{} possible locations:\n{}", f.1.character, f.2)
            //     }
            // })
            .min_by(|a, b| {
                a.2.count()
                    .cmp(&b.2.count())
                    .then(b.1.constraints.len().cmp(&a.1.constraints.len()))
            })
        else {
            return Some(self.clone());
        };

        lazy_static::lazy_static! {
            static ref TOP_LOCATIONS: GridSet = GridSet::from_fn(|t| matches!((t.x(), t.y()), (0,0) | (1,1) | (0,1)) );

            static ref TOP_RIGHT_LOCATIONS: GridSet = GridSet::from_fn(|t| t.x() >= t.y());

            static ref ORDERED_GOOD_LOCATIONS: [Tile; 16] = [
                //centre
                Tile::new_const::<1,1>(),
                Tile::new_const::<1,2>(),
                Tile::new_const::<2,1>(),
                Tile::new_const::<2,2>(),
                //edges
                Tile::new_const::<0,1>(),
                Tile::new_const::<0,2>(),
                Tile::new_const::<1,0>(),
                Tile::new_const::<2,0>(),

                Tile::new_const::<3,1>(),
                Tile::new_const::<3,2>(),
                Tile::new_const::<1,3>(),
                Tile::new_const::<2,3>(),

                //corners
                Tile::new_const::<0,0>(),
                Tile::new_const::<0,3>(),
                Tile::new_const::<3,0>(),
                Tile::new_const::<3,3>(),
            ];
        }

        let potential_locations = if level == 0 {
            potential_locations.intersect(&TOP_LOCATIONS)
        } else if level == 1 && self.used_grid.get_bit(&Tile::new_const::<1, 1>()) {
            potential_locations.intersect(&TOP_RIGHT_LOCATIONS) //todo additional symmetry preventions
        } else {
            potential_locations
        };

        // #[cfg(test)]
        // {
        //     info!("{}", node.character);
        //     info!("{}", potential_locations);
        // }

        if potential_locations == GridSet::EMPTY {
            // #[cfg(test)]
            // {
            //     info!("Nowhere to place {}", node.character);
            // }
            return None;
        }
        let new_nodes = {
            let mut n = nodes_to_add.clone();
            n.remove(index);
            n
        };

        for tile in ORDERED_GOOD_LOCATIONS
            .iter()
            .filter(|t| potential_locations.get_bit(t))
        {
            let Some(new_grid) = self.try_place_node(node, *tile) else {
                continue;
            };

            if let Some(result) = new_grid.solve_recursive(&all_nodes, &new_nodes, level + 1) {
                return Some(result);
            }
        }
        // #[cfg(test)]
        // {
        //     info!("No solution for placing {}", node.character);
        // }

        None
    }

    pub fn potential_locations(&self, node: &Node) -> GridSet {
        let mut allowed = self.used_grid.negate();

        lazy_static::lazy_static! {
            /// This is an example for using doc comment attributes
            static ref NOT_CORNERS: GridSet = GridSet::from_fn(|t|!is_tile_corner(&t));
            static ref INNER_TILES: GridSet = GridSet::from_fn(|t|!is_tile_edge(&t));
        }

        match node.constraints.len() {
            ..=3 => {}
            4..=5 => {
                allowed = allowed.intersect(&NOT_CORNERS);
                if allowed == GridSet::EMPTY {
                    return allowed;
                }
            }
            6..=8 => {
                allowed = allowed.intersect(&INNER_TILES);
                if allowed == GridSet::EMPTY {
                    return allowed;
                }
            }
            _ => {
                return GridSet::EMPTY;
            }
        };

        for constraint in node.constraints.iter() {
            match constraint {
                Constraint::Single(adjacent_node) => {
                    match self.map.get(&adjacent_node) {
                        Some(tile) => {
                            let adjacent = get_adjacent_tiles(tile);
                            allowed = allowed.intersect(&adjacent);
                            if allowed == GridSet::EMPTY {
                                return allowed;
                            }
                        }
                        None => {
                            // do nothing - this tile hasn't been placed yet
                        }
                    }
                }
                Constraint::OneOf(_) => {
                    // Ignore this constraint for now //todo improve
                }
            }
        }

        allowed
    }

    pub fn try_place_node(&self, node: &Node, tile: Tile) -> Option<Self> {
        if self.map.contains_key(&node.id) {
            return None;
        }
        if self.used_grid.get_bit(&tile) {
            return None;
        }

        let new_map = {
            let mut nm = self.map.clone();
            nm.insert(node.id, tile);
            nm
        };

        let mut new_unchecked_constraints: Vec<(Tile, Constraint)> = vec![];

        for constraint in node.constraints.iter() {
            match constraint.is_met(tile, &new_map) {
                Some(true) => {}
                Some(false) => {
                    return None;
                }
                None => new_unchecked_constraints.push((tile, constraint.clone())),
            }
        }

        for (tile, constraint) in self.unchecked_constraints.iter() {
            match constraint.is_met(*tile, &new_map) {
                Some(true) => {}
                Some(false) => {
                    return None;
                }
                None => new_unchecked_constraints.push((*tile, constraint.clone())),
            }
        }

        let new_grid = {
            let mut ng = self.used_grid.clone();
            ng.set_bit(&tile, true);
            ng
        };

        Some(Self {
            used_grid: new_grid,
            map: new_map,
            unchecked_constraints: new_unchecked_constraints,
        })
    }
}

lazy_static::lazy_static! {
    /// This is an example for using doc comment attributes
    static ref ADJACENT_TILES: TileMap<GridSet,4,4,16> = {
        TileMap::<GridSet,4,4,16>::from_fn(|tile|{
            GridSet::from_fn(|x|x.is_adjacent_to(&tile))
        })
    };
}

fn get_adjacent_tiles(tile: &Tile) -> GridSet {
    ADJACENT_TILES[*tile]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::FinderWord;

    #[test]
    pub fn test_adjacent_tiles() {
        let tiles = get_adjacent_tiles(&Tile::new_const::<1, 1>());

        assert_eq!("***_\n*_*_\n***_\n____", tiles.to_string())
    }

    #[test]
    pub fn test_try_make_grid() {
        let words: &[FinderWord] = &[
            // FinderWord::new("ant"),
            // FinderWord::new("Bear"),
            // FinderWord::new("fish"),
            // FinderWord::new("goat"),

            FinderWord::new("croatia"),
            FinderWord::new("france"),
            FinderWord::new("ireland"),
            FinderWord::new("latvia"),
            FinderWord::new("malta"),
            FinderWord::new("poland"),
            FinderWord::new("romania"),
            FinderWord::new("lil"), //TO make it aware it needs two ls
        ];

        let mut letters = LetterCounts::default();
        for word in words.iter() {
            letters = letters
                .try_union(&word.counts)
                .expect("Should be able to combine letters");
        }

        println!("{} letters {}", letters.into_iter().count(), letters.into_iter().join("") );
        let arrays = words.into_iter().map(|x| x.array.clone()).collect();

        let solution = try_make_grid(letters, &arrays);

        match solution {
            Some(solution) => {
                info!("{solution}");
            }
            None => panic!("No Solution found"),
        }
    }
}
