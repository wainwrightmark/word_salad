use itertools::Itertools;
use log::{info, Log};
use simplelog::*;
use std::collections::BTreeMap;
use ws_core::prelude::*;

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

fn make_words_from_file(text: &'static str) -> Vec<FinderWord> {
    text.lines().map(|x| FinderWord::new(x)).collect_vec()
}

fn create_grid_for_most_words(words: Vec<FinderWord>, max_tries: usize) {
    let mut possible_combinations: Vec<Vec<FinderWord>> = Default::default();

    get_combinations(
        &mut possible_combinations,
        vec![],
        Multiplicities::default(),
        words,
        16,
    );
    info!("");
    info!("{c} possible combinations found",c = possible_combinations.len());
    info!("");

    let groups = possible_combinations
        .into_iter()
        .into_group_map_by(|x| x.len());
    let ordered_groups = groups
        .into_iter()
        .sorted_unstable_by(|a, b| b.0.cmp(&a.0))
        .collect_vec();

    for (size, group) in ordered_groups {
        info!(
            "{len} possible combinations of size {size} found",
            len = group.len()
        )
    }
}

fn get_combinations(
    possible_combinations: &mut Vec<Vec<FinderWord>>,
    must_words: Vec<FinderWord>,
    multiplicities: Multiplicities,
    possible_words: Vec<FinderWord>,
    max_size: u8,
) {
    let mut new_possible_words = possible_words.clone();

    while let Some(word) = new_possible_words.pop() {
        let word_text = word.text;

        let mut new_multiplicities = multiplicities.clone();
        new_multiplicities.add_word(&word);
        let mut new_must_words = must_words.clone();
        new_must_words.push(word);

        if new_multiplicities.sum <= max_size {
            possible_combinations.push(new_must_words.clone());
            get_combinations(
                possible_combinations,
                new_must_words,
                new_multiplicities,
                new_possible_words.clone(),
                max_size,
            );
        }

        if must_words.len() == 0 {
            info!(
                "'{word_text}' eliminated. {found} options found. {remaining} remain",
                found = possible_combinations.len(),
                remaining = new_possible_words.len()
            )
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
struct CharacterCounter([u8; 26]);

#[derive(Debug, Clone, PartialEq)]
struct FinderWord {
    pub text: &'static str,
    pub array: CharsArray,
    pub counts: BTreeMap<Character, u8>,
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
        let counts: BTreeMap<Character, u8> = array
            .clone()
            .into_iter()
            .counts()
            .into_iter()
            .map(|(char, count)| (char, count as u8))
            .collect();
        Self {
            array,
            counts,
            text,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
struct Multiplicities {
    set: BTreeMap<Character, u8>,
    sum: u8,
}

impl Multiplicities {
    fn add_word(&mut self, word: &FinderWord) {
        for (char, count) in word.counts.iter() {
            match self.set.entry(*char) {
                std::collections::btree_map::Entry::Vacant(v) => {
                    v.insert(*count);
                    self.sum += count;
                }
                std::collections::btree_map::Entry::Occupied(mut o) => {
                    match count.checked_sub(*o.get()) {
                        Some(s) => {
                            if s > 0 {
                                o.insert(*count);
                                self.sum += s;
                            }
                        }
                        None => {}
                    }
                }
            }
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    #[test]
    pub fn test() {
        let file = "monkey\ncow\nant\nantelope";

        let words = make_words_from_file(file);

        let mut possible_combinations: Vec<Vec<FinderWord>> = Default::default();

        get_combinations(
            &mut possible_combinations,
            vec![],
            Multiplicities::default(),
            words,
            16,
        );

        let expected = "[antelope]\n\
        [antelope, ant]\n\
        [antelope, ant, cow]\n\
        [antelope, ant, cow, monkey]\n\
        [antelope, ant, monkey]\n\
        [antelope, cow]\n\
        [antelope, cow, monkey]\n\
        [antelope, monkey]\n\
        [ant]\n\
        [ant, cow]\n\
        [ant, cow, monkey]\n\
        [ant, monkey]\n\
        [cow]\n\
        [cow, monkey]\n\
        [monkey]";

        let actual = possible_combinations
            .into_iter()
            .map(|x| format!("[{}]", x.iter().join(", ")))
            .join("\n");

        assert_eq!(expected, actual)
    }
}
