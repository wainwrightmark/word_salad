use clap::Parser;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use indicatif_log_bridge::LogWrapper;
use itertools::Itertools;
use log::{debug, info};
use rayon::prelude::*;
use std::{
    collections::{BTreeMap, BTreeSet, HashSet},
    io,
    sync::atomic::AtomicUsize,
};
use ws_core::{
    finder::{counter::FakeCounter, helpers::*},
    prelude::*,
};

#[derive(Parser, Debug)]
#[command()]
struct Options {
    #[arg(short, long, default_value = "data.txt")]
    pub path: String,
    #[arg(short, long, default_value_t = false)]
    pub output: bool,
}

fn main() {
    let logger =
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).build();
    let multi = MultiProgress::new();

    LogWrapper::new(multi.clone(), logger).try_init().unwrap();

    let options = Options::parse();

    info!("Starting up");

    let file = std::fs::read_to_string(options.path.clone())
        .expect("There should be a file called data.txt");
    let file = file.leak(); //cheeky

    //let file = include_str!("colors.txt");
    let word_map = make_words_from_file(file);

    if options.output {
        output_saved_data(word_map);
    } else {
        create_grid_for_most_words(word_map);
    }

    info!("Finished... Press enter");
    io::stdin().read_line(&mut String::new()).unwrap();
}

const DB_PATH: &'static str = "/word_salad_grids";

fn output_saved_data(word_map: WordMultiMap) {
    let word_set: HashSet<&str> = word_map
        .into_iter()
        .flat_map(|x| x.1.into_iter())
        .map(|w| w.text)
        .collect();
    let db = sled::open(DB_PATH).expect("Could not open database");

    let mut solutions: BTreeSet<(usize, String)> = Default::default();

    'results: for result in db.into_iter() {
        let (key, value) = result.unwrap();
        let key = String::from_utf8_lossy(&key);
        let value = String::from_utf8_lossy(&value);

        let trimmed_key = key
            .trim_end_matches(')')
            .trim_end_matches(char::is_numeric)
            .trim_end_matches('(');

        let words: Vec<_> = trimmed_key.split(", ").collect();

        if !words.iter().all(|w| word_set.contains(w)) {
            continue 'results;
        }

        solutions.insert((words.len(), format!("{value}: {key}")));
    }

    for (_, text) in solutions.into_iter().rev() {
        info!("{text}");
    }
}

fn create_grid_for_most_words(word_map: WordMultiMap) {
    let word_letters: Vec<LetterCounts> = word_map.keys().cloned().sorted().collect_vec();
    let possible_combinations: BTreeMap<LetterCounts, usize> = get_combinations(
        Multiplicities::default(),
        word_letters.as_slice(),
        16,
        &word_map,
    );

    info!(
        "{c} possible combinations found",
        c = possible_combinations.len()
    );
    // info!("");

    let groups = possible_combinations.into_iter().into_group_map_by(|x| x.1);
    let ordered_groups = groups.into_iter().sorted_unstable().rev().collect_vec();

    // for (size, group) in ordered_groups.iter() {
    //     info!(
    //         "{len} possible combinations of size {size} found",
    //         len = group.len()
    //     )
    // }

    let db = sled::open(DB_PATH).expect("Could not open database");
    let mut previous_solutions: BTreeSet<LetterCounts> = Default::default();

    let (sender, receiver) = std::sync::mpsc::channel::<LetterCounts>();
    for (size, group) in ordered_groups {
        let solution_count = AtomicUsize::new(0);
        let impossible_count = AtomicUsize::new(0);
        let redundant_count = AtomicUsize::new(0);
        let pb = ProgressBar::new(group.len() as u64).with_style(ProgressStyle::with_template("{msg} {wide_bar} {pos:4}/{len:4}").unwrap()) .with_message(format!("Groups of size {size}"));
        //let latest_solution = ProgressBar::new_spinner();
        group.par_iter().for_each(|(letters, _)| {
            if previous_solutions
                .range(letters..)
                .any(|prev| prev.is_superset(letters))
            {
                pb.inc(1);
                redundant_count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                return;
            }

            let mut counter = FakeCounter;

            let letter_words: Vec<ArrayVec<Character, 16>> = word_map
                .iter()
                .flat_map(|s| s.1)
                .filter(|word| letters.is_superset(&word.counts))
                .map(|z| z.array.clone())
                .collect();
            //let raw_text = get_raw_text(&letters);
            let words_text = get_possible_words_text(&letters, &word_map);
            let result = ws_core::finder::node::try_make_grid_with_blank_filling(
                *letters,
                &letter_words,
                Character::E,
                &mut counter,
            );
            pb.inc(1);
            match result.grid {
                Some(solution) => {
                    solution_count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                    let value = solution.iter().join("");
                    let key = words_text.clone();

                    db.insert(key.as_str(), value.as_str())
                        .expect("Could not insert data");

                    sender.send(*letters).expect("Could not send solution");
                    //latest_solution.set_message(format!("Solution found:\n{words_text}\n{solution}"));

                }
                None => {
                    // debug!(
                    //     "No solution possible for {raw_text}: ({count}) ({words_text})",
                    //     count = letters.into_iter().count()
                    // );
                    impossible_count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                }
            }
        });

        let solution_count = solution_count.into_inner();
        let impossible_count = impossible_count.into_inner();
        let redundant_count = redundant_count.into_inner();

        pb.finish_with_message(format!("{size:2} words: {solution_count:4} Solutions. {impossible_count:4} Impossible. {redundant_count:4} Redundant."));
        //latest_solution.finish();

        previous_solutions.extend(receiver.try_iter());
        db.flush().expect("Could not flush db");
    }
}

fn get_combinations(
    multiplicities: Multiplicities,
    possible_words: &[LetterCounts],
    max_size: u8,
    multi_map: &WordMultiMap,
) -> BTreeMap<LetterCounts, usize> {
    let pb = ProgressBar::new(possible_words.len() as u64).with_style(ProgressStyle::with_template("{msg} {wide_bar} {pos}/{len}").unwrap()) .with_message("Getting word combinations");

    let upper_bounds = 1..(possible_words.len());
    let result = upper_bounds
        .into_iter()
        .map(|upper| &possible_words[0..=upper])
        .par_bridge()
        .map(|words| {
            let mut possible_combinations: BTreeMap<LetterCounts, usize> = BTreeMap::default();
            get_combinations_inner(
                &mut possible_combinations,
                0,
                multiplicities,
                words,
                max_size,
                multi_map,
            );
            possible_combinations
        })
        .reduce(
            || BTreeMap::<LetterCounts, usize>::default(),
            |a, b| {
                pb.inc(1);
                let (mut big, small) = if a.len() >= b.len() { (a, b) } else { (b, a) };
                if small.is_empty() {
                    return big;
                }

                for (key, value) in small.into_iter() {
                    match big.entry(key) {
                        std::collections::btree_map::Entry::Vacant(v) => {
                            v.insert(value);
                        }
                        std::collections::btree_map::Entry::Occupied(mut o) => {
                            if *o.get() < value {
                                *o.get_mut() = value;
                            }
                        }
                    }
                }

                big
            },
        );

    pb.finish();
    result
}

fn get_combinations_inner(
    possible_combinations: &mut BTreeMap<LetterCounts, usize>,
    word_count: usize,
    multiplicities: Multiplicities,
    mut possible_words: &[LetterCounts],
    max_size: u8,
    multi_map: &WordMultiMap,
) {
    loop {
        let Some((word, npw)) = possible_words.split_last() else {
            break;
        };
        possible_words = npw;

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

            get_combinations_inner(
                possible_combinations,
                new_word_count,
                new_multiplicities,
                possible_words,
                max_size,
                multi_map,
            );
        }
    }
}

pub fn get_raw_text(counts: &LetterCounts) -> String {
    counts.into_iter().join("")
}

pub fn write_words(word: &Vec<CharsArray>) -> String {
    word.iter().map(|c| c.iter().join("")).join(", ")
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

        let possible_combinations: BTreeMap<LetterCounts, usize> = get_combinations(

            Multiplicities::default(),
            word_letters.as_slice(),
            16,
            &words,
        );

        info!("{:?}", now.elapsed());

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

        let possible_combinations: BTreeMap<LetterCounts, usize> = get_combinations(

            Multiplicities::default(),
            word_letters.as_slice(),
            16,
            &words,
        );

        info!("{:?}", now.elapsed());

        let contains_expected = possible_combinations.contains_key(&expected);

        if !contains_expected {
            let actual = possible_combinations
                .into_iter()
                .map(|x| format!("[{}]", get_possible_words_text(&x.0, &words)))
                .join("\n");

            info!("{actual}");
        }

        assert!(contains_expected);
    }
}
