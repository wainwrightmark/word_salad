use crate::{prelude, Character, Grid};
use itertools::Itertools;
use std::{cell::Cell, num::NonZeroU8, str::FromStr};

use super::{
    counter::{Counter, SolutionCollector},
    helpers::{FinderSingleWord, LetterCounts},
    partial_grid::{NodeMap, PartialGrid},
};
use crate::finder::*;

//todo benchmark more efficient collections, different heuristics

#[derive(Debug, Clone)]
pub struct GridResult {
    pub grid: Grid,
    pub letters: LetterCounts,
    pub words: Vec<FinderSingleWord>,
}

impl FromStr for GridResult {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut iter = s.split('\t');

        let chars: &str = iter.next().ok_or("Level should have a grid")?;
        let _name: &str = iter.next().ok_or("Level should have name")?;

        let grid = prelude::try_make_grid(chars).ok_or("Should be able to make grid")?;

        let mut words: Vec<FinderSingleWord> = iter
            .map(|x| FinderSingleWord::from_str(x.trim()))
            .try_collect()?;

        words.sort_by_cached_key(|x| x.text.to_ascii_lowercase());

        // .sorted_by_cached_key(|x| x.text.to_ascii_lowercase())
        // .collect();

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

pub fn try_make_grid_with_blank_filling<Collector: SolutionCollector<GridResult>>(
    letters: LetterCounts,
    words: &Vec<FinderSingleWord>,
    exclude_words: &Vec<FinderSingleWord>,
    first_blank_replacement: Character,
    counter: &mut impl Counter,
    collector: &mut Collector,
) {
    try_make_grid(letters, words, exclude_words, counter, collector);

    if collector.is_full() {
        return;
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

            try_make_grid_with_blank_filling(
                new_letters,
                words,
                exclude_words,
                replacement,
                counter,
                collector,
            );
            if collector.is_full() {
                return;
            }
        }
    }
}

struct WordUniquenessHelper {
    constraining_words: CharacterSet<Option<NonZeroU8>>,
    pub character_multi_constraints: CharacterMultiConstraints,
    pub character_nodes: CharacterNodes,
}

impl WordUniquenessHelper {
    pub fn new(
        words: &[FinderSingleWord],
        character_nodes: CharacterNodes,
        character_multi_constraints: CharacterMultiConstraints,
    ) -> Self {
        let mut constraining_words: CharacterSet<Option<NonZeroU8>> = Default::default();

        for (character, set) in character_nodes.enumerate() {
            if set.count() > 1 {
                let word_index = words
                    .iter()
                    .enumerate()
                    .max_by_key(|w| helpers::AdjacencyStrength::calculate(w.1, character))
                    .unwrap()
                    .0 as u8;
                constraining_words.set(character, NonZeroU8::MIN.checked_add(word_index));
            }
        }

        Self {
            constraining_words,
            character_multi_constraints,
            character_nodes,
        }
    }

    pub fn check_letter(
        &self,
        buffered_index: usize,
        word: &FinderSingleWord,
        word_index: NonZeroU8,
    ) -> WordLetterResult {
        let Some(true_index) = buffered_index.checked_sub(1) else {
            return WordLetterResult::Buffer;
        };

        let Some(character) = word.array.get(true_index) else {
            return WordLetterResult::Buffer;
        };

        let node_id_set = self.character_nodes.get(*character);

        match node_id_set.count() {
            0 => WordLetterResult::Buffer,
            1 => WordLetterResult::UniqueLetter(
                *character,
                node_id_set.iter_true_tiles().next().unwrap(),
            ),
            _ => {
                if *self.constraining_words.get(*character) == Some(word_index) {
                    let node_index = word.array[0..true_index]
                        .iter()
                        .filter(|x| *x == character)
                        .count();
                    let node = node_id_set
                        .iter_true_tiles()
                        .skip(node_index)
                        .next()
                        .expect("Should be able to get node by index");
                    return WordLetterResult::UniqueLetter(*character, node);
                } else {
                    WordLetterResult::DuplicateLetter(
                        *character,
                        self.character_multi_constraints
                            .get(*character)
                            .expect("Character should have multi-constraint"),
                    )
                }
            }
        }
    }
}
enum WordLetterResult {
    Buffer,
    UniqueLetter(Character, NodeId),
    DuplicateLetter(Character, MultiConstraintId),
}

pub(crate) type NodeBuilders = geometrid::tile_map::TileMap<NodeBuilder, 16, 1, 16>;

pub fn try_make_grid<Collector: SolutionCollector<GridResult>>(
    letters: LetterCounts,
    words: &Vec<FinderSingleWord>,
    exclude_words: &Vec<FinderSingleWord>,
    counter: &mut impl Counter,
    collector: &mut Collector,
) {
    //todo use a bump allocator
    let mut node_builders: NodeBuilders = NodeBuilders::from_fn(|id| NodeBuilder {
        id,
        character: Character::Blank,
        single_constraints: Default::default(),
        multiple_constraints: Default::default(),
    });
    let mut character_nodes: CharacterNodes = CharacterNodes::default();

    for (node_id, character) in letters.into_iter().enumerate() {
        let id = NodeId::try_from_inner(node_id as u8).expect("Should be able to create node id");
        node_builders[id].character = character;
        character_nodes.get_mut(character).set_bit(&id, true);
    }

    let node_builders: NodeBuilders = node_builders;

    let mut multi_constraint_map: MultiConstraintMap = MultiConstraintMap::default();
    let mut character_multi_constraints: CharacterMultiConstraints =
        CharacterMultiConstraints::default();
    let mut next_constraint_id: MultiConstraintId = Default::default();
    for (character, tile_set) in character_nodes.enumerate() {
        if tile_set.count() > 1 {
            multi_constraint_map[next_constraint_id] = *tile_set;
            character_multi_constraints.set(character, Some(next_constraint_id));
            next_constraint_id = next_constraint_id
                .try_next()
                .expect("Should be at most 8 multi-constraints");
        }
    }

    let helper: WordUniquenessHelper =
        WordUniquenessHelper::new(words, character_nodes, character_multi_constraints);

    //let now = std::time::Instant::now();
    for (word_index, word) in words.iter().enumerate() {
        let word_index = NonZeroU8::MIN.saturating_add(word_index as u8);
        let range = 0..(word.array.len() + 2);
        for (a_index, b_index, c_index) in range.tuple_windows() {
            //we are only adding constraints to b here

            let a = helper.check_letter(a_index, word, word_index);
            let b = helper.check_letter(b_index, word, word_index);
            let c = helper.check_letter(c_index, word, word_index);

            match b {
                WordLetterResult::Buffer => {}
                WordLetterResult::UniqueLetter(_, b_node) => {
                    match a {
                        WordLetterResult::Buffer => {}
                        WordLetterResult::UniqueLetter(_, a_node) => {
                            node_builders[b_node]
                                .add_single_constraint(a_node, &multi_constraint_map);
                        }
                        WordLetterResult::DuplicateLetter(a_char, constraint_id) => {
                            if let WordLetterResult::DuplicateLetter(c_char, ..) = c {
                                if a_char == c_char {
                                    let a_nodes = multi_constraint_map[constraint_id];
                                    if a_nodes.count() == 2 {
                                        //there are two copies of this character and both must be connected to b
                                        for a_node in a_nodes.iter_true_tiles() {
                                            node_builders[b_node].add_single_constraint(
                                                a_node,
                                                &multi_constraint_map,
                                            );
                                            node_builders[a_node].add_single_constraint(
                                                b_node,
                                                &multi_constraint_map,
                                            );
                                        }
                                        continue; //we have already added all need constraints
                                    }
                                }
                            }

                            node_builders[b_node]
                                .add_multiple_constraint(constraint_id, &multi_constraint_map);
                        }
                    }

                    match c {
                        WordLetterResult::Buffer => {}
                        WordLetterResult::UniqueLetter(_, c_node) => {
                            node_builders[b_node]
                                .add_single_constraint(c_node, &multi_constraint_map);
                        }
                        WordLetterResult::DuplicateLetter(_, constraint_id) => {
                            node_builders[b_node]
                                .add_multiple_constraint(constraint_id, &multi_constraint_map);
                        }
                    }
                }
                WordLetterResult::DuplicateLetter(b_char, b_nodes) => {
                    let b_nodes = multi_constraint_map[b_nodes];
                    if b_nodes.count() == 2 {
                        if let WordLetterResult::DuplicateLetter(c_char, ..) = c {
                            if c_char == b_char {
                                if let Some((b0, b1)) = b_nodes.iter_true_tiles().next_tuple() {
                                    node_builders[b0]
                                        .add_single_constraint(b1, &multi_constraint_map);
                                    node_builders[b1]
                                        .add_single_constraint(b0, &multi_constraint_map);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    let mut grid: PartialGrid = Default::default();

    let nodes: NodeMap = NodeMap::from_fn(|tile| node_builders[tile].clone().into());

    // for node in nodes.iter() {
    //     println!(
    //         "Node '{}', single constraints {} multiple constraints {}",
    //         node.character,
    //         node.single_constraints.count(),
    //         node.multiple_constraints.count()
    //     );

    //     for tile in node.single_constraints.iter_true_tiles() {
    //         let char = nodes.iter().find(|x| x.id == tile).unwrap().character;
    //         println!("Single to {char}");
    //     }

    //     println!()
    // }

    // println!("Made nodes in {}micros", now.elapsed().as_micros());

    let mut nodes_with_4_or_more = 0usize;
    let mut nodes_with_6_or_more = 0usize;

    for node in nodes.iter() {
        match node.constraint_count() {
            0..=3 => {}
            4..=5 => {
                nodes_with_4_or_more += 1;
            }
            6..=8 => {
                nodes_with_4_or_more += 1;
                nodes_with_6_or_more += 1;
            }
            _ => {
                //warn!("Grid has a node with more than 8 constraints");
                return;
            }
        }
    }

    if nodes_with_4_or_more > 12 {
        //warn!("Grid has more than 12 nodes with more than 3 constraints");
        return;
    }
    if nodes_with_6_or_more > 4 {
        //warn!("Grid has more than 4 nodes with more than 5 constraints");
        return;
    }

    for word in exclude_words.iter() {
        if is_word_inevitable(&word.array, &nodes, &character_nodes, &multi_constraint_map) {
            // let words = words.iter().map(|x|x.text).join(", ");
            // warn!("Excluded word '{}' is inevitable when finding {words}", word.text);
            return;
        }
    }

    let mut mapped_collector = Collector::Mapped::default();
    grid.solve(
        counter,
        &mut mapped_collector,
        &nodes,
        words,
        exclude_words,
        &multi_constraint_map,
    );

    //println!("Grid solved in {}micros", now.elapsed().as_micros());

    collector.collect_mapped(mapped_collector, |solution| {
        let solution_grid = solution.to_grid(&nodes);
        GridResult {
            grid: solution_grid,
            letters,
            words: words.clone(),
        }
    })
}

/// will it always be possible to find this word is all node restrictions are met
fn is_word_inevitable(
    characters: &[Character],
    nodes: &NodeMap,
    character_nodes: &CharacterNodes,
    multiple_constraints: &MultiConstraintMap,
) -> bool {
    /// will it always be possible to find this word is all node restrictions are met
    fn is_word_inevitable_inner(
        previous: NodeId,
        used: NodeIdSet,
        characters: &[Character],
        nodes: &NodeMap,
        character_nodes: &CharacterNodes,
        multiple_constraints: &MultiConstraintMap,
    ) -> bool {
        let Some((first, rem)) = characters.split_first() else {
            return true;
        };
        let previous = &nodes[previous];
        let first_node_ids = character_nodes.get(*first).intersect(&used.negate());

        if !first_node_ids
            .intersect(&previous.single_constraints.negate())
            .is_empty()
        {
            return false;
        }

        for first_id in first_node_ids.iter_true_tiles() {
            if !is_word_inevitable_inner(
                first_id,
                used.with_bit_set(&first_id, true),
                rem,
                nodes,
                character_nodes,
                multiple_constraints,
            ) {
                return false;
            }
        }

        true
    }

    let Some((first, rem)) = characters.split_first() else {
        return true;
    };

    let first_node_ids = character_nodes.get(*first);

    for id in first_node_ids.iter_true_tiles() {
        if !is_word_inevitable_inner(
            id,
            NodeIdSet::default().with_bit_set(&id, true),
            rem,
            nodes,
            character_nodes,
            multiple_constraints,
        ) {
            return false;
        }
    }

    return true;
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct NodeBuilder {
    pub id: NodeId,
    pub character: Character,
    single_constraints: Cell<NodeIdSet>,
    multiple_constraints: Cell<MultiConstraintIdSet>,
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

    pub fn add_single_constraint(
        &self,
        other_node_id: NodeId,
        multi_constraint_map: &MultiConstraintMap,
    ) {
        let s = self
            .single_constraints
            .get()
            .with_bit_set(&other_node_id, true);

        self.single_constraints.set(s);

        let mut nc = self.multiple_constraints.get();

        //remove all multiple constraints which reference this node
        for mc in nc.iter_true_tiles() {
            let constraint_node_ids = multi_constraint_map[mc];
            if constraint_node_ids.get_bit(&other_node_id) {
                nc.set_bit(&mc, false);
                self.multiple_constraints.set(nc);
            }
        }
    }

    pub fn add_multiple_constraint(
        &self,
        constraint_id: MultiConstraintId,
        multi_constraint_map: &MultiConstraintMap,
    ) {
        //add this constraint unless there is a single constraint to one of the nodes
        let id_set = multi_constraint_map[constraint_id];
        if self.single_constraints.get().intersect(&id_set).is_empty() {
            let new_set = self
                .multiple_constraints
                .get()
                .with_bit_set(&constraint_id, true);
            self.multiple_constraints.set(new_set);
        }
    }
}

impl From<NodeBuilder> for Node {
    fn from(val: NodeBuilder) -> Self {
        Node {
            id: val.id,
            character: val.character,
            single_constraints: val.single_constraints.into_inner(),
            multiple_constraints: val.multiple_constraints.into_inner(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Node {
    single_constraints: NodeIdSet,
    multiple_constraints: MultiConstraintIdSet,
    pub id: NodeId,
    pub character: Character,

}

impl Node {
    pub fn constraint_count(&self) -> u32 {
        self.single_constraints.count() + self.multiple_constraints.count()
    }

    pub fn are_all_constraints_to_character(
        &self,
        character: &Character,
        nodes_map: &CharacterNodes,
        multi_constraint_map: &MultiConstraintMap,
    ) -> bool {
        let other_nodes = nodes_map.get(*character).negate();

        if !self.single_constraints.intersect(&other_nodes).is_empty() {
            return false;
        }

        for multi_constraint in self
            .multiple_constraints
            .iter_true_tiles()
            .map(|x| multi_constraint_map[x])
        {
            if !multi_constraint.intersect(&other_nodes).is_empty() {
                return false;
            }
        }

        true
    }

    pub fn are_constraints_met(
        &self,
        tile: &Tile,
        grid: &PartialGrid,
        multi_constraint_map: &MultiConstraintMap,
    ) -> bool {
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
        'constraints: for constraint in self
            .multiple_constraints
            .iter_true_tiles()
            .map(|x| multi_constraint_map[x])
        {
            if constraint.intersect(&grid.nodes_to_add.negate()) == constraint {
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
    // spellchecker:disable
    #[test_case("SILVER\nORANGE\nGREEN\nIVORY\nCORAL\nOLIVE\nTEAL\nGRAY\nCYAN\nRED")]
    #[test_case("CROATIA\nROMANIA\nIRELAND\nLATVIA\nPOLAND\nFRANCE\nMALTA\nLIL")]
    #[test_case("CROATIA\nROMANIA\nIRELAND\nLATVIA\nPOLAND\nFRANCE\nMALTA")]
    #[test_case("PIEPLATE\nSTRAINER\nTEAPOT\nGRATER\nAPRON\nSPOON\nPOT")]
    #[test_case("THIRTEEN\nFOURTEEN\nFIFTEEN\nSEVENTY\nTHIRTY\nNINETY\nTHREE\nSEVEN\nFORTY\nFIFTY\nFIFTH\nFOUR\nNINE\nONE\nTEN")]
    #[test_case("POLO\nSHOOTING\nKENDO\nSAILING\nLUGE\nSKIING")]
    #[test_case("IOWA\nOHIO\nIDAHO\nUTAH\nHAWAII\nINDIANA\nMONTANA")]
    #[test_case("ROSEMARY\nCARROT\nPARSLEY\nSOY\nPEANUT\nYAM\nPEA\nBEAN")]
    #[test_case("WEEDLE\nMUK\nSLOWPOKE\nGOLEM\nSEEL\nMEW\nEEVEE\nGLOOM")]
    #[test_case("POLITICIAN\nOPTICIAN\nCASHIER\nFLORIST\nARTIST\nTAILOR\nACTOR")]
    #[test_case("ALDGATE\nANGEL\nALDGATEEAST\nBANK\nLANCASTERGATE")]
    #[test_case("WELLS\nLEEDS\nELY\nLISBURN\nDERBY\nNEWRY\nSALISBURY")]
    #[test_case("Sporty\nScary")]
    #[test_case("Teal\nWheat\nWhite\nGreen\nCyan\nGray\nCoral\nOrange\nMagenta")]
    #[test_case("Utah\nOhio\nMaine\nIdaho\nIndiana\nMontana\nArizona")] //slow case
    #[test_case(
        "Abra\nDratini\nArbok\nNidoran\nNidorina\nNidorino\nDragonite\nNidoking\nDragonair"
    )] //slow case
    #[test_case("Teal\nSage\nGreen\nCyan\nOlive\nGray\nClaret\nMagenta\nSilver")] //slow case
    pub fn test_try_make_grid(input: &'static str) {
        let now = Instant::now();
        let words = crate::finder::helpers::make_finder_group_vec_from_file(input);
        let words: Vec<FinderSingleWord> = words.into_iter().flat_map(|x| x.words).collect();

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
        let exclude_words = vec![];
        let mut solution = None;
        try_make_grid_with_blank_filling(
            letters,
            &words,
            &exclude_words,
            Character::E,
            &mut counter,
            &mut solution,
        );
        println!("{:?}", now.elapsed());
        match solution {
            Some(GridResult { grid, .. }) => {
                println!("Found after {} tries", counter.current);

                for word in words.into_iter() {
                    if crate::word::find_solution(&word.array, &grid).is_none() {
                        panic!("No solution for word '{}'", word.text)
                    }
                }
                println!("{grid}");
            }
            None => panic!("No Solution found after {} tries", counter.current),
        }
    }

    #[test_case("Bishop\npawn\nKing\nKnight\nQueen")]
    pub fn test_try_make_many_grids(input: &'static str) {
        let words = crate::finder::helpers::make_finder_group_vec_from_file(input);

        let words: Vec<FinderSingleWord> = words.into_iter().flat_map(|x| x.words).collect();

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
        let exclude_words = vec![];
        let mut solutions = vec![];
        try_make_grid_with_blank_filling(
            letters,
            &words,
            &exclude_words,
            Character::E,
            &mut counter,
            &mut solutions,
        );

        assert!(!solutions.is_empty());

        for mut s in solutions.iter_mut() {
            let _ = crate::finder::orientation::try_optimize_orientation(&mut s);
        }

        // for grid in solutions{
        //     if let Some(word) = orientation::find_single_row_word(&grid){
        //         println!("{word}\n{}\n\n", grid.grid);
        //     }
        // }

        for (grid, (word, score)) in solutions
            .into_iter()
            .map(|grid| {
                let q = orientation::calculate_best_word(&grid);
                (grid, q)
            })
            .sorted_by_key(|x| x.1 .1)
            .rev()
        {
            println!("{word} {score}\n{}\n\n", grid.grid);
        }
    }
}
