use crate::{find_solution, Character, CharsArray, Grid, GridSet, TileMap};
use itertools::Itertools;
use std::collections::{BTreeMap, BTreeSet, HashMap};

use super::{
    helpers::LetterCounts,
    partial_grid::{NodeMap, PartialGrid},
};
use crate::finder::*;

//todo benchmark more efficient collections, different heuristics

pub struct GridResult {
    pub tries: usize,
    pub grid: Option<Grid>,
}

pub struct Counter {
    pub max: usize,
    pub current: usize,
}

impl Counter {
    pub fn try_increment(&mut self) -> bool {
        if self.current >= self.max {
            return false;
        }
        self.current += 1;
        return true;
    }
}

pub fn try_make_grid_with_blank_filling(
    letters: LetterCounts,
    words: &Vec<CharsArray>,
    first_blank_replacement: Character,
    max_tries: usize,
) -> GridResult {
    let result = try_make_grid(letters, words, max_tries);
    //println!("{} tries", result.tries);
    if result.grid.is_some() {
        return result;
    }
    let mut tries = result.tries;

    if letters.contains(Character::Blank) {
        let ordered_replacements = words
            .iter()
            .flat_map(|x| x)
            .counts()
            .into_iter()
            .filter(|x| !x.0.is_blank())
            .filter(|x| *x.0 >= first_blank_replacement)
            .sorted_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(b.0)));

        for (replacement, count) in ordered_replacements {
            if count <= 1 {
                continue; //no point having two copies of this letter
            }
            let new_letters = letters.clone();
            let new_letters = new_letters
                .try_remove(Character::Blank)
                .expect("prime bag error");
            let new_letters = new_letters
                .try_insert(*replacement)
                .expect("prime bag error");

            let result =
                try_make_grid_with_blank_filling(new_letters, words, *replacement, max_tries);
            match result.grid {
                Some(grid) => {
                    return GridResult {
                        tries: tries + result.tries,
                        grid: Some(grid),
                    };
                }
                None => {
                    tries += result.tries;
                }
            }
        }
    }

    return GridResult { tries, grid: None };
}

pub fn try_make_grid(
    letters: LetterCounts,
    words: &Vec<CharsArray>,
    max_tries: usize,
) -> GridResult {
    //println!("Try to make grid: {l:?} : {w:?}", l= crate::get_raw_text(&letters), w= crate::write_words(words) );
    let mut nodes_map: BTreeMap<Character, Vec<Node>> = Default::default();

    let mut next_node_id: u8 = 0;
    for character in letters.into_iter() {
        let node = Node {
            id: NodeId::try_from_inner(next_node_id).expect("Should be able to create node id"),
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

    for word in words {
        for (a, b) in word.iter().tuple_windows() {
            let added = try_add_constraint(a, b, &mut nodes_map);
            let added2 = try_add_constraint(b, a, &mut nodes_map);
            if !added && !added2 {
                if let Some(a_first) = nodes_map.get(a).and_then(|v| v.first()) {
                    if a_first
                        .constraints
                        .iter()
                        .all(|c| c.is_to_character(a, &nodes_map))
                    {
                        if let Some(b_first) = nodes_map.get(b).and_then(|v| v.first()) {
                            if b_first
                                .constraints
                                .iter()
                                .all(|c| c.is_to_character(b, &nodes_map))
                            {
                                let a_to_b_constraint = Constraint::Single(b_first.id);
                                let b_to_a_constraint = Constraint::Single(a_first.id);
                                // these nodes are otherwise unconstrained
                                let a_constraints = &mut nodes_map
                                    .get_mut(a)
                                    .unwrap()
                                    .first_mut()
                                    .unwrap()
                                    .constraints;
                                a_constraints.insert(a_to_b_constraint);
                                let b_constraints = &mut nodes_map
                                    .get_mut(b)
                                    .unwrap()
                                    .first_mut()
                                    .unwrap()
                                    .constraints;

                                b_constraints.insert(b_to_a_constraint);
                            }
                        }
                    }
                }
            }
        }
    }

    //todo check that a grid is actually possible with the given constraint multiplicities

    // #[cfg(test)]
    // {
    //     for (_, nodes) in nodes_map.iter() {
    //         for node in nodes {
    //             info!("{}: {} constraints", node.character, node.constraints.len());
    //         }
    //     }
    // }

    let grid: PartialGrid = Default::default();

    //let nodes: NodeMap = NodeMap::from_fn(|z|Node{});

    let mut nodes_by_id: BTreeMap<NodeId, Node> = nodes_map
        .into_values()
        .flat_map(|v| v.into_iter())
        .map(|n| (n.id, n))
        .collect();

    let nodes: NodeMap = NodeMap::from_fn(|tile| nodes_by_id.remove(&tile).expect("Could not find node"));


    let mut counter = Counter {
        max: max_tries,
        current: 0,
    };
    let Some(solution) = grid.solve_recursive(&mut counter, &nodes, 0, words) else {
        return GridResult {
            tries: counter.current,
            grid: None,
        };
    };

    let solution_grid = solution.to_grid(&nodes);

    GridResult {
        tries: counter.current,
        grid: Some(solution_grid),
    }
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
        Constraint::OneOf(geometrid::tile_set::TileSet16::from_iter(
            target_nodes.iter().map(|z| z.id),
        ))
    };

    // let node_id = constraints.get(key)
    let Some(source_nodes) = nodes_map.get_mut(&from) else {
        return false;
    };

    if !(source_nodes.len() == 1 || (source_nodes.len() == 2 && from == to)) {
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

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Node {
    pub id: NodeId,
    pub character: Character,
    pub constraints: BTreeSet<Constraint>,
}

// #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
// pub struct  Tile::<1,16>;// NodeId(u8); //TODO replace with Tile<1,16>

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Constraint {
    //TODO constraint builder
    Single(NodeId),
    OneOf(geometrid::tile_set::TileSet16<16, 1, 16>),
}
// TODO track whether a node is uniquely constrained
// If there are two copies of a character, look for places where the go round another character

impl Constraint {
    pub fn is_to_character(
        &self,
        character: &Character,
        map: &BTreeMap<Character, Vec<Node>>,
    ) -> bool {
        let Self::Single(id) = self else {
            return false;
        };

        let Some(nodes) = map.get(&character) else {
            return false;
        };
        nodes.iter().map(|x| x.id).contains(id)
    }

    pub fn is_met(&self, tile: Tile, map: &NodeTiles) -> Option<bool> {
        match self {
            Constraint::Single(node_id) => match map[*node_id] {
                Some(other_tile) => Some(tile.is_adjacent_to(&other_tile)),
                None => None,
            },
            Constraint::OneOf(ids) => {
                let mut any_maybe = false;

                for node_id in ids.iter_true_tiles() {
                    match map[node_id] {
                        Some(other_tile) => {
                            if tile.is_adjacent_to(&other_tile) {
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

#[cfg(test)]
mod tests {
    use std::time::Instant;

    use super::*;
    use test_case::test_case;

    //#[test_case("DOG, TOAD, PIGEON, OWL, PIG, ANT, CAT, LION, DEER, COW, GOAT, BEE, TIGER")]
    // #[test_case("SILVER, ORANGE, GREEN, IVORY, CORAL, OLIVE, TEAL, GRAY, CYAN, RED")]
    #[test_case("CROATIA, ROMANIA, IRELAND, LATVIA, POLAND, FRANCE, MALTA")]
    // #[test_case("PIEPLATE, STRAINER, TEAPOT, GRATER, APRON, SPOON, POT")]
    // #[test_case("THIRTEEN, FOURTEEN, FIFTEEN, SEVENTY, THIRTY, NINETY, THREE, SEVEN, FORTY, FIFTY, FIFTH, FOUR, NINE, ONE, TEN")]
    // #[test_case("POLO, SHOOTING, KENDO, SAILING, LUGE, SKIING")]
    // #[test_case("IOWA, OHIO, IDAHO, UTAH, HAWAII, INDIANA, MONTANA")]
    // #[test_case("ROSEMARY, CARROT, PARSLEY, SOY, PEANUT, YAM, PEA, BEAN")]
    // #[test_case("WEEDLE, MUK, SLOWPOKE, GOLEM, SEEL, MEW, EEVEE, GLOOM")]
    // #[test_case("POLITICIAN, OPTICIAN, CASHIER, FLORIST, ARTIST, TAILOR, ACTOR")]
    // #[test_case("ALDGATE, ANGEL, ALDGATEEAST, BANK, LANCASTERGATE")]
    // #[test_case("WELLS, LEEDS, ELY, LISBURN, DERBY, NEWRY, SALISBURY")]
    pub fn test_try_make_grid(input: &'static str) {
        let now = Instant::now();
        let words = crate::finder::helpers::make_words_from_file(input);
        let words = words
            .into_iter()
            .flat_map(|x| x.1.into_iter())
            .collect_vec();

        let mut letters = LetterCounts::default();
        for word in words.iter() {
            letters = letters
                .try_union(&word.counts)
                .expect("Should be able to combine letters");
        }
        let letter_count = letters.into_iter().count();
        println!("{} letters {}", letter_count, letters.into_iter().join(""));

        if letter_count > 16 {
            panic!("Too many letters");
        }
        let arrays = words.into_iter().map(|x| x.array.clone()).collect();

        let mut blanks_to_add = 16usize.saturating_sub(letter_count);
        while blanks_to_add > 0 {
            match letters.try_insert(Character::Blank) {
                Some(n) => letters = n,
                None => {
                    println!("Prime bag wont accept more blanks")
                }
            }
            blanks_to_add -= 1;
        }

        let solution = try_make_grid_with_blank_filling(letters, &arrays, Character::E, 1000000);
        println!("{:?}", now.elapsed());
        match solution.grid {
            Some(grid) => {
                println!("Found after {} tries", solution.tries);
                println!("{grid}");
            }
            None => panic!("No Solution found after {} tries", solution.tries),
        }
    }
}
