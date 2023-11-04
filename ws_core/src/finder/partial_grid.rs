use crate::finder::node::*;
use crate::finder::*;
use crate::{find_solution, Character, CharsArray, Grid, GridSet, TileMap};

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

/// Whether a tile is a corner
fn is_tile_corner(tile: &Tile) -> bool {
    matches!(tile.x(), 0 | 3) && matches!(tile.y(), 0 | 3)
}

/// Whether a tile is an edge or a corner
fn is_tile_edge(tile: &Tile) -> bool {
    matches!(tile.x(), 0 | 3) || matches!(tile.y(), 0 | 3)
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

    pub fn check_matches(&self, nodes: &NodeMap, words: &Vec<CharsArray>) -> bool {
        let solution_grid = self.to_grid(&nodes);

        //info!("Solution found:\n{solution_grid}");
        for word in words {
            if find_solution(word, &solution_grid).is_none() {
                return false;
            }
        }
        return true;
    }

    pub fn solve_recursive(
        //change to an iterator
        &self, //TODO mut self
        counter: &mut Counter,
        all_nodes: &NodeMap,
        level: usize,
        words: &Vec<CharsArray>,
    ) -> Option<Self> {
        if !counter.try_increment() {
            return None;
        }

        //info!("{g}\n\n", g = self.to_grid(all_nodes));

        let Some((node, potential_locations)) = self
            .nodes_to_add
            .iter_true_tiles()
            .map(|tile| {
                let node = &all_nodes[tile];
                let set = self.potential_locations(node);
                (node, set)
            })
            // .inspect(|f| {
            //     #[cfg(test)]
            //     if level == 0 {
            //         info!("{} possible locations:\n{}", f.1.character, f.2)
            //     }
            // })
            .min_by(|a, b| {
                a.1.count()
                    .cmp(&b.1.count())
                    .then(b.0.constraints.len().cmp(&a.0.constraints.len()))
            })
        else {
            //run out of options
            if self.check_matches(all_nodes, words) {
                return Some(self.clone());
            } else {
                return None;
            }
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

        for tile in ORDERED_GOOD_LOCATIONS
            .iter()
            .filter(|t| potential_locations.get_bit(t))
        {
            let Some(new_grid) = self.try_place_node(&node, *tile) else {
                continue;
            };

            if let Some(result) = new_grid.solve_recursive(counter, &all_nodes, level + 1, words) {
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
                    match self.map[*adjacent_node] {
                        Some(tile) => {
                            let adjacent = get_adjacent_tiles(&tile);
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
        if self.map[node.id].is_some() {
            return None;
        }
        if self.used_grid.get_bit(&tile) {
            return None;
        }

        let new_map = {
            let mut nm = self.map.clone();
            nm[node.id] = Some(tile);
            nm
        };

        for constraint in node.constraints.iter() {
            match constraint.is_met(tile, &new_map) {
                Some(true) => {}
                Some(false) => {
                    return None;
                }
                None => {} // todo only push unidirectional constraints
            }
        }

        let new_grid = {
            let mut ng = self.used_grid.clone();
            ng.set_bit(&tile, true);
            ng
        };

        //check adjacent nodes aren't locked out (e.g. if an adjacent node is on an edge and has five constraints but isn't connected to this, the grid is now invalid)

        let mut new_nodes_to_add = self.nodes_to_add.clone();
        new_nodes_to_add.set_bit(&node.id, false);

        Some(Self {
            used_grid: new_grid,
            map: new_map,
            nodes_to_add: new_nodes_to_add,
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

    #[test]
    pub fn test_adjacent_tiles() {
        let tiles = get_adjacent_tiles(&Tile::new_const::<1, 1>());

        assert_eq!("***_\n*_*_\n***_\n____", tiles.to_string())
    }
}
