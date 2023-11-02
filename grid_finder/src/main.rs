use itertools::Itertools;
use log::{info, Log};
use prime_bag::PrimeBag128;
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
    info!(
        "{c} possible combinations found",
        c = possible_combinations.len()
    );
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

        let Some(new_multiplicities)  = multiplicities.try_add_word(&word) else{ panic!("Could not add word to multiplicities");};

        let mut new_must_words = must_words.clone();
        new_must_words.push(word);

        if new_multiplicities.count <= max_size {
            possible_combinations.push(new_must_words.clone());
            get_combinations(
                possible_combinations,
                new_must_words,
                new_multiplicities,
                new_possible_words.clone(),
                max_size,
            );
        }
        // else{
        //     let sum = new_multiplicities.set.into_iter().count();
        //     let elements = new_multiplicities.set.into_iter().map(|x|x.as_char()).join("");
        //     info!("Set too big {} ({sum} elements) '{elements}'", new_multiplicities.count)
        // }

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
    //pub array: CharsArray,
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
            //array,
            counts,
            text,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default, Copy)]
struct Multiplicities {
    set: PrimeBag128<Character>,
    count: u8,
}

impl Multiplicities {
    #[must_use]
    fn try_add_word(&self, word: &FinderWord) -> Option<Self> {
        let union = self.set.try_union(&word.counts)?;

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

        let mut possible_combinations: Vec<Vec<FinderWord>> = Default::default();

        get_combinations(
            &mut possible_combinations,
            vec![],
            Multiplicities::default(),
            words,
            16,
        );

        println!("{:?}", now.elapsed());

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
