
use clap::Parser;
use itertools::Itertools;
use log::info;
use rayon::prelude::*;
use simplelog::*;
use std::{
    collections::{BTreeMap, BTreeSet},
    io,
    sync::atomic::AtomicUsize, ops::{RangeInclusive, Range, RangeFrom},
};
use ws_core::{prelude::*, finder::helpers::*};



#[derive(Parser, Debug)]
#[command()]
struct Options {
    #[arg(short, long, default_value = "data.txt")]
    pub path: String,
    #[arg(short, long, default_value_t = 1000000)]
    pub tries: usize,

    #[arg(short, long, default_value_t = false)]
    pub verbose: bool,
}

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

    let options = Options::parse();

    //todo parallel

    info!("Starting up");

    let file = std::fs::read_to_string(options.path.clone())
        .expect("There should be a file called data.txt");
    let file = file.leak(); //cheeky

    //let file = include_str!("colors.txt");
    let words = make_words_from_file(file);

    create_grid_for_most_words(words, &options);

    println!("Finished... Press enter");
    io::stdin().read_line(&mut String::new()).unwrap();
}



fn create_grid_for_most_words(word_map: WordMultiMap, options: &Options) {
    let mut possible_combinations: BTreeMap<LetterCounts, usize> = Default::default();

    let word_letters: Vec<LetterCounts> = word_map.keys().cloned().sorted().collect_vec();

    get_combinations(
        &mut possible_combinations,
        0,
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

    let db = sled::open("/word_salad_grids").expect("Could not open database");
    let mut previous_solutions: BTreeSet<LetterCounts> = Default::default();

    let (sender, receiver) = std::sync::mpsc::channel::<LetterCounts>();
    for (size, group) in ordered_groups {


        let solution_count = AtomicUsize::new(0);
        let impossible_count = AtomicUsize::new(0);
        let given_up_count = AtomicUsize::new(0);
        let redundant_count = AtomicUsize::new(0);

        group.par_iter()

        .for_each(|(letters, _)| {
            if previous_solutions.range(letters..).any(|prev|prev.is_superset(letters)){
                redundant_count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                return;
            }

            //todo if there are blanks, try to fill them with needed characters

            let letter_words: Vec<ArrayVec<Character, 16>> = word_map
                .iter()
                .flat_map(|s| s.1)
                .filter(|word| letters.is_superset(&word.counts))
                .map(|z| z.array.clone())
                .collect();
            let raw_text = get_raw_text(&letters);
            let words_text = get_possible_words_text(&letters, &word_map);
            let result = ws_core::finder::node::try_make_grid_with_blank_filling(
                *letters,
                &letter_words,
                Character::E,
                options.tries,
            );
            let tries = result.tries;
            match result.grid {
                Some(solution) => {
                    solution_count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                    let value = solution.iter().join("");
                    let key = words_text.clone();

                    db.insert(key.as_str(), value.as_str()).expect("Could not insert data");

                    sender.send(*letters).expect("Could not send solution");
                    info!("Solution found after {tries} tries:\n{words_text}\n{solution}")
                }
                None => {
                    if tries >= options.tries {
                        if options.verbose {
                            info!(
                "Gave up looking for {raw_text} ({tries} tries): ({count}) ({words_text})",
                count = letters.into_iter().count()
            )
                        }
                        given_up_count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                    } else {
                        if options.verbose {
                            info!(
                "No solution possible for {raw_text} ({tries} tries): ({count}) ({words_text})",
                count = letters.into_iter().count()
            )
                        }
                        impossible_count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                    }
                }
            }
        });

        let solution_count = solution_count.into_inner();
        let impossible_count = impossible_count.into_inner();
        let given_up_count = given_up_count.into_inner();
        let redundant_count = redundant_count.into_inner();
        info!("Groups of {size} words: {solution_count} Solutions. {impossible_count} Impossible. {given_up_count} Given Up On. {redundant_count} Redundant.");

        previous_solutions.extend(receiver.try_iter());
        db.flush().expect("Could not flush db");
    }
}


fn get_combinations(
    possible_combinations: &mut BTreeMap<LetterCounts, usize>,
    word_count: usize,
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

        let new_word_count = word_count + 1;


        if new_multiplicities.count <= max_size {
            match possible_combinations.entry(new_multiplicities.set) {
                std::collections::btree_map::Entry::Vacant(v) => {
                    v.insert(new_word_count);
                }
                std::collections::btree_map::Entry::Occupied(mut o) => {
                    if *o.get() < new_word_count {
                        o.insert(new_word_count);
                    }
                }
            };

            get_combinations(
                possible_combinations,
                new_word_count,
                new_multiplicities,
                new_possible_words.clone(),
                max_size,
                multi_map,
            );
        }

        if new_word_count == 1 {
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
            0,
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
    #[test_case(
        "POLITICIAN, OPTICIAN, CASHIER, FLORIST, ARTIST, TAILOR, ACTOR",
        "POLITICIAN, OPTICIAN, CASHIER, FLORIST, ARTIST, TAILOR, ACTOR"
    )]
    #[test_case(
        "SILVER, ORANGE, GREEN, IVORY, CORAL, OLIVE, TEAL, GRAY, CYAN, RED",
        "SILVER, ORANGE, GREEN, IVORY, CORAL, OLIVE, TEAL, GRAY, CYAN, RED"
    )]
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
            0,
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
