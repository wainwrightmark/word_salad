use crate::finder::node::*;
use crate::finder::*;
use crate::{find_solution, Character, Grid, GridSet};
use arrayvec::ArrayVec;

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
        for (node_id, node) in nodes.enumerate() {
            if let Some(tile) = self.map[node_id] {
                grid[tile] = node.character;
            }
        }
        grid
    }

    pub fn check_matches(
        &self,
        nodes: &NodeMap,
        words: &[FinderSingleWord],
        exclude_words: &[FinderSingleWord],
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
    ) -> Option<(NodeId, GridSet)> {
        let allowed_by_symmetry = if self.used_grid == GridSet::EMPTY {
            TOP_LOCATIONS
        } else if self.used_grid.is_subset(&DOWN_RIGHT_DIAGONAL) {
            TOP_RIGHT_LOCATIONS
        } else {
            GridSet::ALL
        };

        self.nodes_to_add
            .iter_true_tiles()
            .map(|node_id| {
                let node = all_nodes[node_id];
                let set = self.potential_locations(node, allowed_by_symmetry, multi_constraint_map);

                (node_id, node, set)
            })
            .min_by(|a, b| {
                a.2.count()
                    .cmp(&b.2.count())
                    .then(
                        b.1.multiple_constraints
                            .count()
                            .cmp(&a.1.multiple_constraints.count())
                            .reverse(),
                    ) //as many multiple constraints as possible
                    .then(
                        b.1.single_constraints
                            .count()
                            .cmp(&a.1.single_constraints.count()),
                    ) //as few single constraints as possible
            })
            .map(|x| (x.0, x.2))
    }

    pub fn solve(
        &mut self,
        counter: &mut impl Counter,
        collector: &mut impl SolutionCollector<Self>,
        all_nodes: &NodeMap,
        words: &[FinderSingleWord],
        exclude_words: &[FinderSingleWord],
        multi_constraint_map: &MultiConstraintMap,
    ) {
        struct Frame {
            node_id: NodeId,
            prev_node: NodeId,
            remaining_locations: RemainingLocations,
        }

        let mut stack: ArrayVec<Frame, 16> = Default::default();

        let Some((n_id, pl)) = self.get_most_constrained_node(all_nodes, multi_constraint_map)
        else {
            return;
        };

        stack.push(Frame {
            node_id: n_id,
            prev_node: NodeId::default(),
            remaining_locations: RemainingLocations::new(pl),
        });

        while let Some(Frame {
            node_id,
            prev_node,
            remaining_locations,
        }) = stack.last_mut()
        {
            if let Some(tile) = remaining_locations.next() {
                if !counter.try_increment() {
                    return; //Give up
                }

                self.place_node(*node_id, tile);

                if let Some((next_node_id, potential_locations)) =
                    self.get_most_constrained_node(all_nodes, multi_constraint_map)
                {
                    let prev_node_id = *node_id;

                    stack.push(Frame {
                        node_id: next_node_id,
                        prev_node: prev_node_id,
                        remaining_locations: RemainingLocations::new(potential_locations),
                    })
                } else {
                    if self.check_matches(all_nodes, words, exclude_words) {
                        collector.collect_solution(self.clone());
                        if collector.is_full() {
                            return;
                        }
                    }
                    self.remove_node(*node_id)
                }
            } else {
                self.remove_node(*prev_node);
                stack.pop();
            }
        }
    }

    pub fn potential_locations(
        &self,
        node: Node,
        allowed_by_symmetry: GridSet,
        multi_constraint_map: &MultiConstraintMap,
    ) -> GridSet {
        let mut allowed = self.used_grid.negate().intersect(&allowed_by_symmetry);

        match node.constraint_count() {
            ..=3 => {}
            4..=5 => {
                allowed = allowed.intersect(&NOT_CORNERS);
            }
            6..=8 => {
                allowed = allowed.intersect(&INNER_TILES);
            }
            _ => {
                return GridSet::EMPTY;
            }
        };

        if allowed.is_empty() {
            return allowed;
        }

        //println!("{new_allowed}");

        for tile in allowed.iter_true_tiles() {
            if !node.are_constraints_met(&tile, self, multi_constraint_map) {
                allowed.set_bit(&tile, false);
                // println!("{tile}");
                // println!("{new_allowed}");
            }
        }

        allowed
    }

    pub fn remove_node(&mut self, node_id: NodeId) {
        self.nodes_to_add.set_bit(&node_id, true);

        if let Some(tile) = self.map[node_id] {
            self.used_grid.set_bit(&tile, false);
        }
        self.map[node_id] = None;
    }

    fn place_node(&mut self, node_id: NodeId, tile: Tile) {
        self.map[node_id] = Some(tile);
        self.used_grid.set_bit(&tile, true);
        self.nodes_to_add.set_bit(&node_id, false);
    }
}

#[derive(Debug, Clone, PartialEq)]
struct RemainingLocations {
    potential_locations: GridSet,
}

impl RemainingLocations {
    pub const fn new(potential_locations: GridSet) -> Self {
        Self {
            potential_locations,
        }
    }

    fn grid_set_first(set: GridSet) -> Option<Tile> {
        Tile::try_from_inner(set.into_inner().trailing_zeros() as u8)
    }
}

impl Iterator for RemainingLocations {
    type Item = Tile;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(next) = Self::grid_set_first(self.potential_locations.intersect(&CENTRE_TILES))
        {
            self.potential_locations.set_bit(&next, false);
            return Some(next);
        }
        if let Some(next) = Self::grid_set_first(self.potential_locations.intersect(&EDGE_TILES)) {
            self.potential_locations.set_bit(&next, false);
            return Some(next);
        }
        if let Some(next) = Self::grid_set_first(self.potential_locations.intersect(&CORNER_TILES))
        {
            self.potential_locations.set_bit(&next, false);
            return Some(next);
        }

        None
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

        grid.place_node(a_id, a_tile);

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
