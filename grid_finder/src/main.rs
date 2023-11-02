use itertools::Itertools;
use log::{info, Log};
use prime_bag::PrimeBag128;
use simplelog::*;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use ws_core::prelude::*;

type LetterCounts = PrimeBag128<Character>;
type WordMultiMap = HashMap<LetterCounts, Vec<FinderWord>>;

fn main() {
    CombinedLogger::init(vec![
        TermLogger::new(
            LevelFilter::Info,
            Config::default(),
            TerminalMode::Mixed,
            ColorChoice::Auto,
        ),
        //WriteLogger::new(LevelFilter::Info, Config::default(), File::create("my_rust_binary.log").unwrap()),
    ])
    .unwrap();

    info!("Starting up");

    let file = include_str!("animals.txt");
    let words = make_words_from_file(file);

    create_grid_for_most_words(words, 100);
}

fn make_words_from_file(text: &'static str) -> WordMultiMap {
    text.lines()
        .map(|x| FinderWord::new(x))
        .into_group_map_by(|x| x.counts)
}

fn create_grid_for_most_words(words: WordMultiMap, max_tries: usize) {
    let mut possible_combinations: BTreeMap<LetterCounts, usize> = Default::default();

    let word_letters: Vec<LetterCounts> = words.keys().cloned().sorted().collect_vec();

    get_combinations(
        &mut possible_combinations,
        vec![],
        Multiplicities::default(),
        word_letters,
        16,
        &words,
    );
    info!("");
    info!(
        "{c} possible combinations found",
        c = possible_combinations.len()
    );
    info!("");

    let groups = possible_combinations.into_iter().into_group_map_by(|x| x.1);
    let ordered_groups = groups.into_iter().sorted_unstable().rev().collect_vec();

    for (size, group) in ordered_groups {
        info!(
            "{len} possible combinations of size {size} found",
            len = group.len()
        )
    }
}

fn get_combinations(
    possible_combinations: &mut BTreeMap<LetterCounts, usize>,
    used_words: Vec<LetterCounts>,
    multiplicities: Multiplicities,
    possible_words: Vec<LetterCounts>,
    max_size: u8,
    multi_map: &WordMultiMap,
) {
    let mut new_possible_words = possible_words.clone();

    while let Some(word) = new_possible_words.pop() {
        //let word_text = word.text;

        let Some(new_multiplicities) = multiplicities.try_add_word(&word) else {
            panic!("Could not add word to multiplicities");
        };

        let mut new_used_words = used_words.clone();
        new_used_words.push(word);

        if new_multiplicities.count <= max_size {
            match possible_combinations.entry(new_multiplicities.set) {
                std::collections::btree_map::Entry::Vacant(v) => {
                    v.insert(new_used_words.len());
                }
                std::collections::btree_map::Entry::Occupied(mut o) => {
                    if o.get() < &new_used_words.len() {
                        o.insert(new_used_words.len());
                    }
                }
            };

            get_combinations(
                possible_combinations,
                new_used_words,
                new_multiplicities,
                new_possible_words.clone(),
                max_size,
                multi_map,
            );
        }
        // else{
        //     let sum = new_multiplicities.set.into_iter().count();
        //     let elements = new_multiplicities.set.into_iter().map(|x|x.as_char()).join("");
        //     info!("Set too big {} ({sum} elements) '{elements}'", new_multiplicities.count)
        // }

        if used_words.len() == 0 {
            info!(
                "'{word_text}' eliminated. {found} options found. {remaining} remain",
                word_text = get_text(&word, multi_map),
                found = possible_combinations.len(),
                remaining = new_possible_words.len()
            )
        }
    }
}

fn get_text(counts: &LetterCounts, word_map: &WordMultiMap) -> String {
    word_map
        .get(counts)
        .map(|x| x.iter().map(|w| w.text).join(", "))
        .unwrap_or_else(|| "?????".to_string())
}

fn get_possible_words_text(counts: &LetterCounts, word_map: &WordMultiMap) -> String {
    word_map
        .iter()
        .filter(|(c, _w)| counts.is_superset(c))
        .flat_map(|(_c, words)| words.iter().map(|w| w.text))
        .sorted()
        .join(", ")
}

// fn try_create()-> Option<Grid>
// {

// }

#[derive(Debug, Clone, PartialEq, Default)]
struct CharacterCounter([u8; 26]);

#[derive(Debug, Clone, PartialEq)]
struct FinderWord {
    pub text: &'static str,
    pub array: CharsArray,
    pub counts: PrimeBag128<Character>,
}

impl std::fmt::Display for FinderWord {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.text)
    }
}

impl FinderWord {
    fn new(text: &'static str) -> Self {
        let array = match Word::from_static_str(text) {
            Ok(w) => w.characters,
            Err(..) => panic!("'{text}' is not a valid word"),
        };

        let counts: PrimeBag128<Character> =
            PrimeBag128::try_from_iter(array.iter().cloned()).expect("Could not make prime bag");

        // let counts: BTreeMap<Character, u8> = array
        //     .clone()
        //     .into_iter()
        //     .counts()
        //     .into_iter()
        //     .map(|(char, count)| (char, count as u8))
        //     .collect();
        Self {
            array,
            counts,
            text,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default, Copy, Ord, PartialOrd, Eq)]
struct Multiplicities {
    count: u8,
    set: LetterCounts,
}

impl Multiplicities {
    #[must_use]
    fn try_add_word(&self, word: &LetterCounts) -> Option<Self> {
        let union = self.set.try_union(&word)?;

        if union == self.set {
            Some(*self)
        } else {
            let diff = union.try_difference(&self.set)?;
            let new_elements = diff.into_iter().count() as u8;
            Some(Self {
                set: union,
                count: self.count + new_elements,
            })
        }
    }
}

#[cfg(test)]
pub mod tests {
    use std::time::Instant;

    use super::*;
    #[test]
    pub fn test() {
        let file = "monkey\ncow\nant\nantelope";

        let now = Instant::now();

        let words = make_words_from_file(file);
        let word_letters: Vec<LetterCounts> = words.keys().cloned().collect_vec();

        let mut possible_combinations: BTreeMap<LetterCounts, usize> = Default::default();

        get_combinations(
            &mut possible_combinations,
            vec![],
            Multiplicities::default(),
            word_letters,
            16,
            &words,
        );

        println!("{:?}", now.elapsed());

        let expected = "[ant]\n[cow]\n[ant, cow]\n[ant, antelope]\n[monkey]\n[ant, monkey]\n[ant, antelope, cow]\n[cow, monkey]\n[ant, cow, monkey]\n[ant, antelope, monkey]\n[ant, antelope, cow, monkey]";

        let actual = possible_combinations
            .into_iter()
            .map(|x| format!("[{}]", get_possible_words_text(&x.0, &words)))
            .join("\n");

        assert_eq!(expected, actual)
    }
}
