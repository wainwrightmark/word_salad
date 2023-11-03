pub mod node;
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

    let file = include_str!("colors.txt");
    let words = make_words_from_file(file);

    for word in words.iter().flat_map(|(_, v)| v) {
        println!("'{word}'");
    }

    create_grid_for_most_words(words);
}

fn make_words_from_file(text: &'static str) -> WordMultiMap {
    text.lines().flat_map(|x|x.split(','))
        .map(|x| FinderWord::new(x))
        .into_group_map_by(|x| x.counts)
}

fn create_grid_for_most_words(word_map: WordMultiMap) {
    let mut possible_combinations: BTreeMap<LetterCounts, usize> = Default::default();

    let word_letters: Vec<LetterCounts> = word_map.keys().cloned().sorted().collect_vec();

    get_combinations(
        &mut possible_combinations,
        vec![],
        Multiplicities::default(),
        word_letters,
        16,
        &word_map,
    );
    info!("");
    info!(
        "{c} possible combinations found",
        c = possible_combinations.len()
    );
    info!("");

    let groups = possible_combinations.into_iter().into_group_map_by(|x| x.1);
    let ordered_groups = groups.into_iter().sorted_unstable().rev().collect_vec();

    for (size, group) in ordered_groups.iter() {
        info!(
            "{len} possible combinations of size {size} found",
            len = group.len()
        )
    }

    for (_, group) in ordered_groups {
        for (letters, _) in group {
            //todo if there are blanks, try to fill them with needed characters

            let letter_words: Vec<ArrayVec<Character, 16>> = word_map
                .iter()
                .flat_map(|s| s.1)
                .filter(|word| letters.is_superset(&word.counts))
                .map(|z| z.array.clone())
                .collect();
            let raw_text = get_raw_text(&letters);
            let words_text = get_possible_words_text(&letters, &word_map);
            let result = node::try_make_grid_with_blank_filling(letters, &letter_words);
            let tries = result.tries;
            match result.grid {
                Some(solution) => {
                    info!("Solution found after {tries} tries:\n{words_text}\n{solution}")
                }
                None => {
                    if tries >= 1000000 {
                        info!(
                            "Gave up looking for {raw_text} ({tries} tries): ({count}) ({words_text})",
                            count = letters.into_iter().count()
                        )
                    } else {
                        info!(
                            "No solution possible for {raw_text} ({tries} tries): ({count}) ({words_text})",
                            count = letters.into_iter().count()
                        )
                    }
                }
            }
        }
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

pub fn get_raw_text(counts: &LetterCounts) -> String {
    counts.into_iter().join("")
}

pub fn write_words(word: &Vec<CharsArray>) -> String {
    word.iter().map(|c| c.iter().join("")).join(", ")
}

fn get_text(counts: &LetterCounts, word_map: &WordMultiMap) -> String {
    word_map
        .get(counts)
        .map(|x| x.iter().map(|w| w.text).join(", "))
        .unwrap_or_else(|| "?????".to_string())
}

fn get_possible_words_text(counts: &LetterCounts, word_map: &WordMultiMap) -> String {
    let words = word_map.iter().filter(|(c, _w)| counts.is_superset(c));
    let suffix = format!("({})", words.clone().count());
    words
        .flat_map(|(_c, words)| words.iter().map(|w| w.text))
        .sorted()
        .join(", ")
        + suffix.as_str()
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
        //println!("'{text}'");
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
    use super::*;
    use std::time::Instant;
    use test_case::test_case;

    #[test]
    pub fn test() {
        let input = "monkey\ncow\nant\nantelope";

        let now = Instant::now();

        let words = make_words_from_file(input);
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

        let expected = "[ant(1)]\n[cow(1)]\n[ant, cow(2)]\n[ant, antelope(2)]\n[monkey(1)]\n[ant, monkey(2)]\n[ant, antelope, cow(3)]\n[cow, monkey(2)]\n[ant, cow, monkey(3)]\n[ant, antelope, monkey(3)]\n[ant, antelope, cow, monkey(4)]";

        let actual = possible_combinations
            .into_iter()
            .map(|x| format!("[{}]", get_possible_words_text(&x.0, &words)))
            .join("\n");

        assert_eq!(expected, actual)
    }

    #[test_case("monkey\ncow\nant\nantelope", "monkey\ncow\nant\nantelope")]
    #[test_case("POLITICIAN, OPTICIAN, CASHIER, FLORIST, ARTIST, TAILOR, ACTOR", "POLITICIAN, OPTICIAN, CASHIER, FLORIST, ARTIST, TAILOR, ACTOR")]
    #[test_case("SILVER, ORANGE, GREEN, IVORY, CORAL, OLIVE, TEAL, GRAY, CYAN, RED", "SILVER, ORANGE, GREEN, IVORY, CORAL, OLIVE, TEAL, GRAY, CYAN, RED")]
    pub fn test_membership(input: &'static str, expected_member: &'static str) {
        let now = Instant::now();

        let expected_words = make_words_from_file(expected_member);
        let mut expected = LetterCounts::default();
        for (w, _) in expected_words {
            expected = expected
                .try_union(&w)
                .expect("Should be able to union expected");
        }

        let words = make_words_from_file(input);

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

        let contains_expected = possible_combinations.contains_key(&expected);

        if !contains_expected {
            let actual = possible_combinations
                .into_iter()
                .map(|x| format!("[{}]", get_possible_words_text(&x.0, &words)))
                .join("\n");

            println!("{actual}");
        }

        assert!(contains_expected);
    }
}
