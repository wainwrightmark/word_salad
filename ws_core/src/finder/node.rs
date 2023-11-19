use crate::{prelude, Character, Grid};
use itertools::Itertools;
use std::{
    cell::{Cell, RefCell},
    collections::{BTreeMap, BTreeSet},
    str::FromStr,
};

use super::{
    counter::Counter,
    helpers::{FinderWord, LetterCounts},
    partial_grid::{NodeMap, PartialGrid},
};
use crate::finder::*;

//todo benchmark more efficient collections, different heuristics

#[derive(Debug, Clone)]
pub struct GridResult {
    pub grid: Grid,
    pub letters: LetterCounts,
    pub words: Vec<FinderWord>,
}

impl FromStr for GridResult {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut iter = s.split('\t');

        let chars: &str = iter.next().expect("Level should have a grid");
        let _name: &str = iter.next().expect("Level should have name");

        let grid = prelude::try_make_grid(chars).expect("Should be able to make grid");

        let words = iter
            .map(|x| x.trim().to_string())
            .flat_map(|x| FinderWord::try_new(x.as_str()))
            .sorted_by_cached_key(|x| x.text.to_ascii_lowercase())
            .collect();

        let mut letters = LetterCounts::default();
        for c in grid.iter() {
            letters = letters.try_insert(*c).ok_or("Prime bag is too big")?;
        }

        Ok(Self {
            grid,
            letters,
            words,
        })
    }
}

impl std::fmt::Display for GridResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let words_text = self
            .words
            .iter()
            .map(|x| format!("{:8}", x.text))
            .sorted()
            .join("\t");
        let solution = self.grid.iter().join("");
        let size = self.words.len();

        write!(f, "{solution}\t{size}\t{words_text}")
    }
}

pub fn try_make_grid_with_blank_filling(
    letters: LetterCounts,
    words: &Vec<FinderWord>,
    first_blank_replacement: Character,
    counter: &mut impl Counter,
) -> Option<GridResult> {
    let result = try_make_grid(letters, words, counter);
    //println!("{} tries", result.tries);
    if result.is_some() {
        return result;
    }

    if letters.contains(Character::Blank) {
        let ordered_replacements = words
            .iter()
            .flat_map(|x| x.array.clone())
            .counts()
            .into_iter()
            .filter(|x| !x.0.is_blank())
            .filter(|x| x.0 >= first_blank_replacement)
            .sorted_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));

        for (replacement, count) in ordered_replacements {
            if count <= 1 {
                continue; //no point having two copies of this letter
            }
            let new_letters = letters;
            let new_letters = new_letters
                .try_remove(Character::Blank)
                .expect("prime bag error");
            let new_letters = new_letters
                .try_insert(replacement)
                .expect("prime bag error");

            let result = try_make_grid_with_blank_filling(new_letters, words, replacement, counter);
            if result.is_some() {
                return result;
            }
        }
    }

    None
}

struct WordUniquenessHelper {
    constraining_words: BTreeMap<Character, FinderWord>,
}

impl WordUniquenessHelper {
    pub fn new(words: &[FinderWord], nodes_map: &BTreeMap<Character, Vec<NodeBuilder>>) -> Self {
        let constraining_words: BTreeMap<Character, FinderWord> = nodes_map
            .iter()
            .filter(|(_, nodes)| nodes.len() > 1)
            .map(|x| x.0)
            .map(|char| {
                (
                    *char,
                    words
                        .iter()
                        .max_by_key(|w| helpers::count_adjacent_indexes(w, *char))
                        .unwrap()
                        .clone(),
                )
            })
            .collect();

        Self { constraining_words }
    }

    pub fn check_letter<'a>(
        &self,
        buffered_index: usize,
        word: &FinderWord,
        nodes_map: &'a BTreeMap<Character, Vec<NodeBuilder>>,
    ) -> WordLetterResult<'a> {
        let Some(true_index) = buffered_index.checked_sub(1) else {
            return WordLetterResult::Buffer;
        };

        let Some(character) = word.array.get(true_index) else {
            return WordLetterResult::Buffer;
        };

        let vec = nodes_map
            .get(character)
            .expect("Character not associated with any nodes");

        if vec.len() == 1 {
            return WordLetterResult::UniqueLetter(*character, &vec[0]);
        } else if let Some(constraining_word) = self.constraining_words.get(character) {
            if word == constraining_word {
                let node_index = word.array[0..true_index]
                    .iter()
                    .filter(|x| *x == character)
                    .count();
                let node = vec
                    .get(node_index)
                    .expect("Should be able to get node by index");
                return WordLetterResult::UniqueLetter(*character, node);
            }
        }

        WordLetterResult::DuplicateLetter(*character, vec)
    }
}
enum WordLetterResult<'a> {
    Buffer,
    UniqueLetter(Character, &'a NodeBuilder),
    DuplicateLetter(Character, &'a Vec<NodeBuilder>),
}

pub fn try_make_grid(
    letters: LetterCounts,
    words: &Vec<FinderWord>,
    counter: &mut impl Counter,
) -> Option<GridResult> {
    //println!("Try to make grid: {l:?} : {w:?}", l= crate::get_raw_text(&letters), w= crate::write_words(words) );
    let mut nodes_map: BTreeMap<Character, Vec<NodeBuilder>> = Default::default();

    for (node_id, character) in letters.into_iter().enumerate() {
        let id = NodeId::try_from_inner(node_id as u8).expect("Should be able to create node id");
        let node = NodeBuilder::new(id, character);
        match nodes_map.entry(character) {
            std::collections::btree_map::Entry::Vacant(v) => {
                v.insert(vec![node]);
            }
            std::collections::btree_map::Entry::Occupied(mut o) => {
                o.get_mut().push(node);
            }
        }
    }

    let helper: WordUniquenessHelper = WordUniquenessHelper::new(words, &nodes_map);

    for word in words {
        let range = 0..(word.array.len() + 2);
        for (a_index, b_index, c_index) in range.tuple_windows() {
            //we are only adding constraints to b here

            let a = helper.check_letter(a_index, word, &nodes_map);
            let b = helper.check_letter(b_index, word, &nodes_map);
            let c = helper.check_letter(c_index, word, &nodes_map);

            match b {
                WordLetterResult::Buffer => {}
                WordLetterResult::UniqueLetter(_, b_node) => {
                    match a {
                        WordLetterResult::Buffer => {}
                        WordLetterResult::UniqueLetter(_, a_node) => {
                            b_node.add_single_constraint(a_node.id);
                        }
                        WordLetterResult::DuplicateLetter(a_char, a_nodes) => {
                            if a_nodes.len() == 2 {
                                if let WordLetterResult::DuplicateLetter(c_char, ..) = c {
                                    if a_char == c_char {
                                        //there are two copies of this character and both must be connected to b
                                        for a_node in a_nodes {
                                            b_node.add_single_constraint(a_node.id);
                                            a_node.add_single_constraint(b_node.id);
                                        }
                                        continue;
                                    }
                                }
                            }

                            b_node.add_multiple_constraint(a_nodes);
                        }
                    }

                    match c {
                        WordLetterResult::Buffer => {}
                        WordLetterResult::UniqueLetter(_, c_node) => {
                            b_node.add_single_constraint(c_node.id);
                        }
                        WordLetterResult::DuplicateLetter(_, c_nodes) => {
                            b_node.add_multiple_constraint(c_nodes);
                        }
                    }
                }
                WordLetterResult::DuplicateLetter(b_char, b_nodes) => {
                    //todo handle pair of same character nodes next door to each other
                    if b_nodes.len() == 2 {
                        if let WordLetterResult::DuplicateLetter(c_char, ..) = c {
                            if c_char == b_char {
                                let b0 = &b_nodes[0];
                                let b1 = &b_nodes[1];

                                b0.add_single_constraint(b1.id);
                                b1.add_single_constraint(b0.id);
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
    //             let node: Node = node.clone().into();
    //             println!("{}: {} constraints", node.character, node.constraint_count);
    //         }
    //     }
    // }

    let mut grid: PartialGrid = Default::default();

    let mut nodes_by_id: BTreeMap<NodeId, Node> = nodes_map
        .into_values()
        .flat_map(|v| v.into_iter())
        .map(|n| (n.id, n.into()))
        .collect();

    let nodes: NodeMap = NodeMap::from_fn(|tile| {
        nodes_by_id
            .remove(&tile)
            .unwrap_or_else(|| NodeBuilder::new(tile, Character::Blank).into())
    });

    let Some(solution) = grid.solve_recursive(counter, &nodes, 0, words) else {
        return None;
    };

    let solution_grid = solution.to_grid(&nodes);
    Some(GridResult {
        grid: solution_grid,
        letters,
        words: words.clone(),
    })
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct NodeBuilder {
    pub id: NodeId,
    pub character: Character,
    single_constraints: Cell<NodeIdSet>,
    multiple_constraints: RefCell<BTreeSet<NodeIdSet>>,
}

impl NodeBuilder {
    pub fn new(id: NodeId, character: Character) -> Self {
        Self {
            id,
            character,
            single_constraints: Default::default(),
            multiple_constraints: Default::default(),
        }
    }

    pub fn add_single_constraint(&self, other: NodeId) {
        let mut s = self.single_constraints.get();
        s.set_bit(&other, true);
        self.single_constraints.set(s);

        self.multiple_constraints
            .borrow_mut()
            .retain(|mc| mc.intersect(&s) == NodeIdSet::EMPTY);
    }

    pub fn add_multiple_constraint(&self, nodes: &[NodeBuilder]) {
        let mut constraint = NodeIdSet::from_iter(nodes.iter().map(|n| n.id));
        constraint.set_bit(&self.id, false);

        match constraint.count() {
            0 => {}
            1 => self.add_single_constraint(constraint.iter_true_tiles().next().unwrap()),
            _ => {
                if self.single_constraints.get().intersect(&constraint) == NodeIdSet::EMPTY {
                    self.multiple_constraints.borrow_mut().insert(constraint);
                }
            }
        }
    }
}

impl From<NodeBuilder> for Node {
    fn from(val: NodeBuilder) -> Self {
        let constraint_count = val.single_constraints.get().count() as u8
            + val.multiple_constraints.borrow().len() as u8;

        Node {
            id: val.id,
            character: val.character,
            single_constraints: val.single_constraints.get(),
            multiple_constraints: val.multiple_constraints.into_inner(),
            constraint_count,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Node {
    pub id: NodeId,
    pub character: Character,
    single_constraints: NodeIdSet,
    multiple_constraints: BTreeSet<NodeIdSet>,
    pub constraint_count: u8,
}

impl Node {
    pub fn are_all_constraints_to_character(
        &self,
        character: &Character,
        nodes_map: &BTreeMap<Character, Vec<Node>>,
    ) -> bool {
        let character_nodes = match nodes_map.get(character) {
            Some(nodes) => NodeIdSet::from_iter(nodes.iter().map(|x| x.id)),
            None => NodeIdSet::default(),
        };

        let other_nodes = character_nodes.negate();

        if self.single_constraints.intersect(&other_nodes) != NodeIdSet::EMPTY {
            return false;
        }

        for constraint in self.multiple_constraints.iter() {
            if constraint.intersect(&other_nodes) != NodeIdSet::EMPTY {
                return false;
            }
        }

        true
    }

    pub fn are_constraints_met(&self, tile: &Tile, grid: &PartialGrid) -> bool {
        let placed_nodes = grid.nodes_to_add.negate();
        let placed_constraints = placed_nodes.intersect(&self.single_constraints);
        // single constraints
        for placed in placed_constraints.iter_true_tiles() {
            if let Some(placed_tile) = grid.map[placed] {
                if !placed_tile.is_adjacent_to(tile) {
                    return false;
                }
            }
        }
        // multiple constraints
        'constraints: for constraint in &self.multiple_constraints {
            if constraint.intersect(&grid.nodes_to_add.negate()) == *constraint {
                //are all possible nodes of this constraint placed

                for placed_id in constraint.iter_true_tiles() {
                    if let Some(placed_tile) = grid.map[placed_id] {
                        if placed_tile.is_adjacent_to(tile) {
                            continue 'constraints;
                        }
                    }
                }
                return false;
            }
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use std::time::Instant;

    use crate::finder::counter::RealCounter;

    use super::*;
    use test_case::test_case;

    #[test_case("DOG, TOAD, PIGEON, OWL, PIG, ANT, CAT, LION, DEER, COW, GOAT, BEE, TIGER")]
    #[test_case("SILVER, ORANGE, GREEN, IVORY, CORAL, OLIVE, TEAL, GRAY, CYAN, RED")]
    #[test_case("CROATIA, ROMANIA, IRELAND, LATVIA, POLAND, FRANCE, MALTA, LIL")]
    #[test_case("CROATIA, ROMANIA, IRELAND, LATVIA, POLAND, FRANCE, MALTA")]
    // spellchecker:disable-next-line
    #[test_case("PIEPLATE, STRAINER, TEAPOT, GRATER, APRON, SPOON, POT")]
    #[test_case("THIRTEEN, FOURTEEN, FIFTEEN, SEVENTY, THIRTY, NINETY, THREE, SEVEN, FORTY, FIFTY, FIFTH, FOUR, NINE, ONE, TEN")]
    #[test_case("POLO, SHOOTING, KENDO, SAILING, LUGE, SKIING")]
    #[test_case("IOWA, OHIO, IDAHO, UTAH, HAWAII, INDIANA, MONTANA")]
    #[test_case("ROSEMARY, CARROT, PARSLEY, SOY, PEANUT, YAM, PEA, BEAN")]
    // spellchecker:disable-next-line
    #[test_case("WEEDLE, MUK, SLOWPOKE, GOLEM, SEEL, MEW, EEVEE, GLOOM")]
    #[test_case("POLITICIAN, OPTICIAN, CASHIER, FLORIST, ARTIST, TAILOR, ACTOR")]
    // spellchecker:disable-next-line
    #[test_case("ALDGATE, ANGEL, ALDGATEEAST, BANK, LANCASTERGATE")]
    #[test_case("WELLS, LEEDS, ELY, LISBURN, DERBY, NEWRY, SALISBURY")]
    #[test_case("Cat, Dog")]
    //#[test_case("Utah, Ohio, Maine, Idaho, Indiana, Montana, Arizona")] //TODO make this case fast
    #[test_case("Teal, Wheat, White, Green, Cyan, Gray, Coral, Orange, Magenta")]
    pub fn test_try_make_grid(input: &'static str) {
        let now = Instant::now();
        let words = crate::finder::helpers::make_words_vec_from_file(input);

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

        let mut counter = RealCounter {
            max: 1000000000,
            current: 0,
        };

        let solution =
            try_make_grid_with_blank_filling(letters, &words, Character::E, &mut counter);
        println!("{:?}", now.elapsed());
        match solution {
            Some(GridResult { grid, .. }) => {
                println!("Found after {} tries", counter.current);
                println!("{grid}");
            }
            None => panic!("No Solution found after {} tries", counter.current),
        }
    }
}
