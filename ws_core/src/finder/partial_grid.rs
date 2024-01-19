use crate::finder::node::*;
use crate::finder::*;
use crate::{find_solution, Character, Grid, GridSet};

use super::counter::{Counter, SolutionCollector};
use super::helpers::FinderSingleWord;

pub type NodeMap = geometrid::tile_map::TileMap<Node, 16, 1, 16>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PartialGrid {
    pub used_grid: GridSet,
    pub map: NodeTiles,
    pub nodes_to_add: NodeIdSet,
}

impl Default for PartialGrid {
    fn default() -> Self {
        Self {
            used_grid: Default::default(),
            map: Default::default(),
            nodes_to_add: NodeIdSet::EMPTY.negate(),
        }
    }
}

impl PartialGrid {
    pub fn to_grid(&self, nodes: &NodeMap) -> Grid {
        let mut grid: Grid = Grid::from_fn(|_| Character::Blank);
        for node in nodes {
            if let Some(tile) = self.map[node.id] {
                grid[tile] = node.character;
            }
        }
        grid
    }

    pub fn check_matches(
        &self,
        nodes: &NodeMap,
        words: &Vec<FinderSingleWord>,
        exclude_words: &Vec<FinderSingleWord>,
    ) -> bool {
        let solution_grid = self.to_grid(nodes);

        //println!("Solution found:\n{solution_grid}");
        for word in words {
            if find_solution(&word.array, &solution_grid).is_none() {
                return false;
            }
        }

        for word in exclude_words {
            if find_solution(&word.array, &solution_grid).is_some() {
                return false;
            }
        }

        true
    }

    fn get_most_constrained_node(
        &self,
        all_nodes: &NodeMap,
        multi_constraint_map: &MultiConstraintMap,
    ) -> Option<(Node, GridSet)> {
        let allowed_by_symmetry = if self.used_grid == GridSet::EMPTY {
            TOP_LOCATIONS
        } else if self.used_grid.is_subset(&DOWN_RIGHT_DIAGONAL) {
            TOP_RIGHT_LOCATIONS
        } else {
            GridSet::ALL
        };

        self.nodes_to_add
            .iter_true_tiles()
            .map(|tile| {
                let node = all_nodes[tile];
                let set = self.potential_locations(node, allowed_by_symmetry, multi_constraint_map);

                (node, set)
            })
            .min_by(|a, b| {
                a.1.count()
                    .cmp(&b.1.count())
                    .then(b.0.constraint_count.cmp(&a.0.constraint_count))
            })
    }

    pub fn solve_recursive(
        &mut self,
        counter: &mut impl Counter,
        collector: &mut impl SolutionCollector<Self>,
        all_nodes: &NodeMap,
        level: usize,
        words: &Vec<FinderSingleWord>,
        exclude_words: &Vec<FinderSingleWord>,
        multi_constraint_map: &MultiConstraintMap,
    ) {
        // let mut stack: ArrayVec<(Node, GridSet)> = Default::default();

        if !counter.try_increment() {
            return; //Give up
        }

        let Some((node, potential_locations)) =
            self.get_most_constrained_node(all_nodes, multi_constraint_map)
        else {
            //run out of options
            if self.check_matches(all_nodes, words, exclude_words) {
                collector.collect_solution(self.clone());
            }
            return;
        };

        let mut remaining_locations = RemainingLocations::new(potential_locations);

        while let Some(tile) = remaining_locations.next() {
            self.place_node(&node, tile);

            self.solve_recursive(
                counter,
                collector,
                all_nodes,
                level + 1,
                words,
                exclude_words,
                multi_constraint_map,
            );
            if collector.is_full() {
                return;
            }

            self.remove_node(node.id, tile);
        }
    }

    pub fn potential_locations(
        &self,
        node: Node,
        allowed_by_symmetry: GridSet,
        multi_constraint_map: &MultiConstraintMap,
    ) -> GridSet {
        let mut allowed = self.used_grid.negate().intersect(&allowed_by_symmetry);

        match node.constraint_count {
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
        let mut new_allowed = allowed;

        //println!("{new_allowed}");

        for tile in allowed.iter_true_tiles() {
            if !node.are_constraints_met(&tile, self, multi_constraint_map) {
                new_allowed.set_bit(&tile, false);
                // println!("{tile}");
                // println!("{new_allowed}");
            }
        }

        new_allowed
    }

    pub fn remove_node(&mut self, node_id: NodeId, tile: Tile) {
        self.map[node_id] = None;
        self.used_grid.set_bit(&tile, false);
        self.nodes_to_add.set_bit(&node_id, true);
    }

    fn place_node(&mut self, node: &Node, tile: Tile) {
        self.map[node.id] = Some(tile);
        self.used_grid.set_bit(&tile, true);
        self.nodes_to_add.set_bit(&node.id, false);
    }
}

#[derive(Debug, Clone, PartialEq)]
enum RemainingLocations {
    Centre {
        remaining_centres: GridSet,
        potential_locations: GridSet,
    },
    Edge {
        remaining_edges: GridSet,
        potential_locations: GridSet,
    },
    Corner {
        remaining_corners: GridSet,
    },
}

impl RemainingLocations {
    pub const fn new(potential_locations: GridSet) -> Self {
        Self::Centre {
            remaining_centres: potential_locations.intersect(&CENTRE_TILES),
            potential_locations,
        }
    }
    #[must_use]
    fn grid_set_pop(set: &mut GridSet) -> Option<Tile> {
        // if set.into_inner() == 0{
        //     return None;
        // }
        let Some(next) = Tile::try_from_inner(set.into_inner().trailing_zeros() as u8) else {
            return None;
        };
        set.set_bit(&next, false);
        Some(next)
    }
}

impl Iterator for RemainingLocations {
    type Item = Tile;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self {
                RemainingLocations::Centre {
                    ref mut remaining_centres,
                    potential_locations,
                } => {
                    if let Some(next) = Self::grid_set_pop(remaining_centres) {
                        return Some(next);
                    } else {
                        *self = RemainingLocations::Edge {
                            remaining_edges: EDGE_TILES.intersect(&potential_locations),
                            potential_locations: *potential_locations,
                        };
                    }
                }
                RemainingLocations::Edge {
                    ref mut remaining_edges,
                    potential_locations,
                } => {
                    if let Some(next) = Self::grid_set_pop(remaining_edges) {
                        return Some(next);
                    } else {
                        *self = RemainingLocations::Corner {
                            remaining_corners: CORNER_TILES.intersect(&potential_locations),
                        };
                    }
                }
                RemainingLocations::Corner {
                    ref mut remaining_corners,
                } => {
                    return Self::grid_set_pop(remaining_corners);
                }
            }
        }
    }
}

const CENTRE_TILES: GridSet = GridSet::EMPTY
    .with_bit_set(&Tile::new_const::<1, 1>(), true)
    .with_bit_set(&Tile::new_const::<1, 2>(), true)
    .with_bit_set(&Tile::new_const::<2, 1>(), true)
    .with_bit_set(&Tile::new_const::<2, 2>(), true);

const EDGE_TILES: GridSet = GridSet::EMPTY
    .with_bit_set(&Tile::new_const::<0, 1>(), true)
    .with_bit_set(&Tile::new_const::<0, 2>(), true)
    .with_bit_set(&Tile::new_const::<1, 3>(), true)
    .with_bit_set(&Tile::new_const::<2, 3>(), true)
    .with_bit_set(&Tile::new_const::<1, 0>(), true)
    .with_bit_set(&Tile::new_const::<2, 0>(), true)
    .with_bit_set(&Tile::new_const::<3, 1>(), true)
    .with_bit_set(&Tile::new_const::<3, 2>(), true);

const CORNER_TILES: GridSet = GridSet::EMPTY
    .with_bit_set(&Tile::new_const::<0, 0>(), true)
    .with_bit_set(&Tile::new_const::<0, 3>(), true)
    .with_bit_set(&Tile::new_const::<3, 0>(), true)
    .with_bit_set(&Tile::new_const::<3, 3>(), true);

const TOP_LOCATIONS: GridSet = {
    let set = GridSet::EMPTY;
    let set = set.with_bit_set(&Tile::new_const::<0, 0>(), true);
    let set = set.with_bit_set(&Tile::new_const::<0, 1>(), true);
    let set = set.with_bit_set(&Tile::new_const::<1, 1>(), true);

    set
};

const DOWN_RIGHT_DIAGONAL: GridSet = GridSet::EMPTY
    .with_bit_set(&Tile::new_const::<0, 0>(), true)
    .with_bit_set(&Tile::new_const::<1, 1>(), true)
    .with_bit_set(&Tile::new_const::<2, 2>(), true)
    .with_bit_set(&Tile::new_const::<3, 3>(), true);

const NOT_CORNERS: GridSet = GridSet::ALL
    .with_bit_set(&Tile::NORTH_EAST, false)
    .with_bit_set(&Tile::NORTH_WEST, false)
    .with_bit_set(&Tile::SOUTH_EAST, false)
    .with_bit_set(&Tile::SOUTH_WEST, false);

const INNER_TILES: GridSet = GridSet::EMPTY
    .with_bit_set(&Tile::new_const::<1, 1>(), true)
    .with_bit_set(&Tile::new_const::<1, 2>(), true)
    .with_bit_set(&Tile::new_const::<2, 1>(), true)
    .with_bit_set(&Tile::new_const::<2, 2>(), true);

const TOP_RIGHT_LOCATIONS: GridSet = GridSet::ALL
    .with_bit_set(&Tile::new_const::<0, 1>(), false)
    .with_bit_set(&Tile::new_const::<0, 2>(), false)
    .with_bit_set(&Tile::new_const::<0, 3>(), false)
    .with_bit_set(&Tile::new_const::<1, 2>(), false)
    .with_bit_set(&Tile::new_const::<1, 3>(), false)
    .with_bit_set(&Tile::new_const::<2, 3>(), false);

#[cfg(test)]
mod tests {
    use itertools::Itertools;

    use super::*;

    #[test]
    pub fn test_potential_locations() {
        let multi_constraint_map: MultiConstraintMap = MultiConstraintMap::default();
        let mut grid = PartialGrid::default();

        let a_id = NodeId::try_from_inner(0).unwrap();
        let a_node = NodeBuilder::new(a_id, Character::A).into();

        let a_locations = grid.potential_locations(a_node, GridSet::ALL, &multi_constraint_map);

        let mut expected = GridSet::EMPTY.negate();

        assert_sets_eq(a_locations, expected);
        let a_tile = Tile::new_const::<1, 1>();

        grid.place_node(&a_node, a_tile);

        let b_id = NodeId::try_from_inner(1).unwrap();

        let b_node = NodeBuilder::new(b_id, Character::B).into();

        let b_location_1 = grid.potential_locations(b_node, GridSet::ALL, &multi_constraint_map);

        expected.set_bit(&a_tile, false);

        assert_sets_eq(b_location_1, expected);

        let c_id = NodeId::try_from_inner(2).unwrap();

        let c_node = NodeBuilder::new(c_id, Character::C);

        c_node.add_single_constraint(a_id, &multi_constraint_map);
        let c_node: Node = c_node.into();

        let c_location = grid.potential_locations(c_node, GridSet::ALL, &multi_constraint_map);

        expected = GridSet::from_fn(|t| a_tile.is_adjacent_to(&t));
        assert_sets_eq(c_location, expected);
    }

    #[test]
    fn test_remaining_locations() {
        const ORDERED_GOOD_LOCATIONS: [Tile; 16] = [
            //centre
            Tile::new_const::<1, 1>(),
            Tile::new_const::<2, 1>(),
            Tile::new_const::<1, 2>(),
            Tile::new_const::<2, 2>(),
            //edges
            Tile::new_const::<1, 0>(),
            Tile::new_const::<2, 0>(),
            Tile::new_const::<0, 1>(),
            Tile::new_const::<3, 1>(),
            Tile::new_const::<0, 2>(),
            Tile::new_const::<3, 2>(),
            Tile::new_const::<1, 3>(),
            Tile::new_const::<2, 3>(),
            //corners
            Tile::new_const::<0, 0>(),
            Tile::new_const::<3, 0>(),
            Tile::new_const::<0, 3>(),
            Tile::new_const::<3, 3>(),
        ];

        let remaining_locations = RemainingLocations::new(GridSet::ALL);

        let actual = remaining_locations.collect_vec();
        let expected = ORDERED_GOOD_LOCATIONS.into_iter().collect_vec();

        assert_eq!(expected, actual);
    }

    #[track_caller]
    fn assert_sets_eq(actual: GridSet, expected: GridSet) {
        if actual == expected {
            return;
        }

        panic!("Grid Sets do not match\n\nExpected:\n{expected}\n\nActual:\n{actual}");
    }
}
