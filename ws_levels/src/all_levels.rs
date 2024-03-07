use itertools::Itertools;
use lazy_static::lazy_static;
use ws_core::{DesignedLevel, Numbering, Ustr};

lazy_static! { //todo data_bake
    pub(crate) static ref TUTORIAL: Vec<DesignedLevel> = include_str!("levels/tutorial.tsv")
        .lines()
        .map(DesignedLevel::from_tsv_line)
        .map(|x| x.unwrap())
        .collect_vec();


        pub(crate) static ref US_STATES: Vec<DesignedLevel> = number_levels(
            include_str!("levels/geography/us_states.tsv")
                .lines()
                .map(DesignedLevel::from_tsv_line)
                .map(|x| x.unwrap()),
            "US States"
        );

    pub(crate) static ref EUROPEAN_CAPITALS: Vec<DesignedLevel> = number_levels(
        include_str!("levels/geography/european_capitals.tsv")
            .lines()
            .map(DesignedLevel::from_tsv_line)
            .map(|x| x.unwrap()),
        "European Capitals"
    );
    pub(crate) static ref EUROPEAN_COUNTRIES: Vec<DesignedLevel> = number_levels(
        include_str!("levels/geography/european_countries.tsv")
            .lines()
            .map(DesignedLevel::from_tsv_line)
            .map(|x| x.unwrap()),
        "European Countries"
    );


    pub(crate) static ref SOUTH_AND_EAST_ASIAN_COUNTRIES: Vec<DesignedLevel> = number_levels(
        include_str!("levels/geography/south_and_east_asian_countries.tsv")
            .lines()
            .map(DesignedLevel::from_tsv_line)
            .map(|x| x.unwrap()),
        "South & East Asian Countries"
    );

    pub(crate) static ref MIDDLE_EASTERN_COUNTRIES: Vec<DesignedLevel> = number_levels(
        include_str!("levels/geography/middle_eastern_countries.tsv")
            .lines()
            .map(DesignedLevel::from_tsv_line)
            .map(|x| x.unwrap()),
        "Middle Eastern Countries"
    );

    pub(crate) static ref AFRICAN_COUNTRIES: Vec<DesignedLevel> = number_levels(
        include_str!("levels/geography/african_countries.tsv")
            .lines()
            .map(DesignedLevel::from_tsv_line)
            .map(|x| x.unwrap()),
        "African Countries"
    );

    pub(crate) static ref SOUTH_AND_EAST_ASIAN_CAPITALS: Vec<DesignedLevel> = number_levels(
        include_str!("levels/geography/south_and_east_asian_capitals.tsv")
            .lines()
            .map(DesignedLevel::from_tsv_line)
            .map(|x| x.unwrap()),
        "South & East Asian Capitals"
    );

    pub(crate) static ref MIDDLE_EASTERN_CAPITALS: Vec<DesignedLevel> = number_levels(
        include_str!("levels/geography/middle_eastern_capitals.tsv")
            .lines()
            .map(DesignedLevel::from_tsv_line)
            .map(|x| x.unwrap()),
        "Middle Eastern Capitals"
    );




    pub(crate) static ref INSECTS: Vec<DesignedLevel> = number_levels(
        include_str!("levels/natural_world/insects.tsv")
            .lines()
            .map(DesignedLevel::from_tsv_line)
            .map(|x| x.unwrap()),
        "Insects"
    );
    pub(crate) static ref FRUIT: Vec<DesignedLevel> = number_levels(
        include_str!("levels/natural_world/fruit.tsv")
            .lines()
            .map(DesignedLevel::from_tsv_line)
            .map(|x| x.unwrap()),
        "Fruit"
    );
    pub(crate) static ref GEMSTONES: Vec<DesignedLevel> = number_levels(
        include_str!("levels/natural_world/gemstones.tsv")
            .lines()
            .map(DesignedLevel::from_tsv_line)
            .map(|x| x.unwrap()),
        "Gemstones"
    );
    pub(crate) static ref VEGETABLES: Vec<DesignedLevel> = number_levels(
        include_str!("levels/natural_world/vegetables.tsv")
            .lines()
            .map(DesignedLevel::from_tsv_line)
            .map(|x| x.unwrap()),
        "Vegetables"
    );
    pub(crate) static ref ELEMENTS: Vec<DesignedLevel> = number_levels(
        include_str!("levels/natural_world/elements.tsv")
            .lines()
            .map(DesignedLevel::from_tsv_line)
            .map(|x| x.unwrap()),
        "Elements"
    );

    pub(crate) static ref MAMMALS: Vec<DesignedLevel> = number_levels(
        include_str!("levels/natural_world/mammals.tsv")
            .lines()
            .map(DesignedLevel::from_tsv_line)
            .map(|x| x.unwrap()),
        "Mammals"
    );

    pub(crate) static ref BIRDS: Vec<DesignedLevel> = number_levels(
        include_str!("levels/natural_world/birds.tsv")
            .lines()
            .map(DesignedLevel::from_tsv_line)
            .map(|x| x.unwrap()),
        "Birds"
    );

    pub(crate) static ref REPTILES_AND_AMPHIBIANS: Vec<DesignedLevel> = number_levels(
        include_str!("levels/natural_world/reptiles and amphibians.tsv")
            .lines()
            .map(DesignedLevel::from_tsv_line)
            .map(|x| x.unwrap()),
        "Reptiles & Amphibians"
    );

    pub(crate) static ref NFL_TEAMS: Vec<DesignedLevel> = number_levels(
        include_str!("levels/US Sports/NFL Teams.tsv")
            .lines()
            .map(DesignedLevel::from_tsv_line)
            .map(|x| x.unwrap()),
        "NFL Teams"
    );

    pub(crate) static ref NHL_TEAMS: Vec<DesignedLevel> = number_levels(
        include_str!("levels/US Sports/NHL Teams.tsv")
            .lines()
            .map(DesignedLevel::from_tsv_line)
            .map(|x| x.unwrap()),
        "NHL Teams"
    );

    pub(crate) static ref NBA_TEAMS: Vec<DesignedLevel> = number_levels(
        include_str!("levels/US Sports/NBA Teams.tsv")
            .lines()
            .map(DesignedLevel::from_tsv_line)
            .map(|x| x.unwrap()),
        "NBA Teams"
    );

    pub(crate) static ref MLB_TEAMS: Vec<DesignedLevel> = number_levels(
        include_str!("levels/US Sports/MLB Teams.tsv")
            .lines()
            .map(DesignedLevel::from_tsv_line)
            .map(|x| x.unwrap()),
        "MLB Teams"
    );

    pub static ref DEFAULT_DAILY_CHALLENGE: Vec<DesignedLevel> = include_str!("../../daily.tsv")
            .lines()
            .map(DesignedLevel::from_tsv_line)
            .map(|x| x.unwrap())
            .collect_vec();


    pub static ref DAILY_CHALLENGE_NUMBERED: Vec<DesignedLevel> = {
        let mut levels = (*DEFAULT_DAILY_CHALLENGE).clone();
        number_daily_challenge_levels(&mut levels);
        levels

    };
}

pub fn number_daily_challenge_levels(levels: &mut Vec<DesignedLevel>) {
    for (index, level) in levels.iter_mut().enumerate() {
        level.numbering = Some(Numbering::WordSaladNumber(index + 1));
    }
}

pub fn number_levels(
    levels: impl Iterator<Item = DesignedLevel>,
    category: &'static str,
) -> Vec<DesignedLevel> {
    let mut r = vec![];
    let mut index = 1;
    for mut l in levels {
        l.numbering = Some(Numbering::SequenceNumber(index));
        l.name = Ustr::from(category);
        index += 1;
        r.push(l)
    }

    r
}

pub fn get_tutorial_level(index: usize) -> Option<&'static DesignedLevel> {
    TUTORIAL.get(index)
}

#[cfg(test)]
pub mod tests {

    use std::{collections::HashSet, str::FromStr};

    use strum::IntoEnumIterator;
    use ws_core::{
        finder::{
            cluster::*, falling_probability, helpers::FinderSingleWord, node::GridResult,
            orientation,
        },
        prelude::*,
    };

    use crate::prelude::LevelSequence;

    use super::*;

    pub fn get_all_levels() -> Vec<DesignedLevel> {
        [
            TUTORIAL.iter(),
            US_STATES.iter(),
            EUROPEAN_COUNTRIES.iter(),
            EUROPEAN_CAPITALS.iter(),
            SOUTH_AND_EAST_ASIAN_COUNTRIES.iter(),
            MIDDLE_EASTERN_COUNTRIES.iter(),
            SOUTH_AND_EAST_ASIAN_CAPITALS.iter(),
            MIDDLE_EASTERN_CAPITALS.iter(),
            INSECTS.iter(),
            FRUIT.iter(),
            GEMSTONES.iter(),
            VEGETABLES.iter(),
            ELEMENTS.iter(),
            MAMMALS.iter(),
            BIRDS.iter(),
        ]
        .iter()
        .cloned()
        .flat_map(|x| x)
        .cloned()
        .collect_vec()
    }

    #[test]
    pub fn test_all_levels_unique() {
        let mut set: HashSet<_> = Default::default();

        let mut all_errors: Vec<String> = Default::default();

        for level in get_all_levels()
            .iter()
            .chain((*DEFAULT_DAILY_CHALLENGE).iter())
        {
            let words = level
                .words
                .iter()
                .map(|x: &DisplayWord| x.characters.clone())
                .sorted()
                .collect_vec();

            if !set.insert(words) {
                all_errors.push(format!("Level '{}' is a duplicate", level.to_string()));
            }
        }

        for error in all_errors.iter() {
            println!("{error}")
        }

        assert!(all_errors.is_empty())
    }

    #[test]
    pub fn test_all_levels_valid() {
        let levels = get_all_levels();

        assert!(levels.len() > 5);

        let mut all_errors: Vec<String> = Default::default();

        for level in levels {
            let name = &level.name;
            assert!(level.words.len() > 0, "Level {name} should have words");
            for word in level.words.iter() {
                let solution = word.find_solution(&level.grid);
                if solution.is_none() {
                    panic!(
                        "Level '{name}' has no solution for '{word}'",
                        word = word.text
                    )
                }
            }

            if let Err(err) = test_grid_not_taboo(&level) {
                if !TABOO_EXCEPTIONS.contains(
                    &(level
                        .grid
                        .into_iter()
                        .map(|x| x.as_char().to_ascii_uppercase())
                        .join("")
                        .as_str()),
                ) {
                    all_errors.push(err);
                }
            }

            test_word_ordering(&level, &mut all_errors);
        }

        for error in all_errors.iter() {
            println!("{error}")
        }

        assert!(all_errors.is_empty())
    }

    #[test]
    pub fn test_daily_challenge_levels_valid() {
        assert!(DEFAULT_DAILY_CHALLENGE.len() > 10);

        let mut all_errors: Vec<String> = Default::default();

        for level in DEFAULT_DAILY_CHALLENGE.clone().into_iter() {
            let name = &level.name;
            if level.words.len() < 4 {
                all_errors.push(format!("Level {name} should have at least 4 words"))
            }

            for word in level.words.iter() {
                let solution = word.find_solution(&level.grid);
                if solution.is_none() {
                    all_errors.push(format!(
                        "Level '{name}' has no solution for '{word}'",
                        word = word.text
                    ));
                }
            }

            if let Err(err) = test_grid_not_taboo(&level) {
                if !TABOO_EXCEPTIONS.contains(
                    &(level
                        .grid
                        .into_iter()
                        .map(|x| x.as_char().to_ascii_uppercase())
                        .join("")
                        .as_str()),
                ) {
                    all_errors.push(err);
                }
            }

            test_word_ordering(&level, &mut all_errors);

            if let Some(colors) = level.special_colors {
                if colors.len() < 4 {
                    all_errors.push(format!(
                        "Level {name} has custom colors but fewer than four"
                    ));
                }
            }
        }

        for error in all_errors.iter() {
            println!("{error}")
        }

        assert!(all_errors.is_empty())
    }

    #[test]
    pub fn test_sequence_clustering() {
        let mut text: String = String::default();

        for sequence in LevelSequence::iter() {
            let cluster = Cluster::from_levels(&sequence.levels());

            text.push_str(format!("{:50} {}\n", sequence.name(), cluster.header()).as_str());
        }

        insta::assert_snapshot!(text);
    }

    #[test]
    pub fn test_daily_challenge_data() {
        let mut text = "Name\tInfo\tWord Count\tUtilization\tChance to fall after 1st word\tChance to fall after 2nd word\tChance to have fallen after 2 words\tWords\n".to_string();

        for level in (*DEFAULT_DAILY_CHALLENGE).iter() {
            let fa1 = falling_probability::calculate_falling_probability_1(level) * 100.0;
            let fa2 = falling_probability::calculate_falling_probability_2(level) * 100.0;
            let fac2 =
                falling_probability::calculate_cumulative_falling_probability_2(level) * 100.0;
            let words_count = level.words.len();
            let utilization = level
                .words
                .iter()
                .map(|x| x.characters.len())
                .sum::<usize>() as f32
                / level.grid.iter().filter(|x| !x.is_blank()).count() as f32;
            let data =
                format!("{words_count:2}\t{utilization:1.2}\t{fa1:3.1}%\t{fa2:3.1}%\t{fac2:3.1}%");

            let words = level.words.iter().map(|x| x.text).join("\t");

            text.push_str(
                format!(
                    "{name:30}\t{info:20}\t{data}\t{words}\n",
                    name = level.full_name().as_str(),
                    info = level.extra_info.unwrap_or_default().as_str()
                )
                .as_str(),
            );
        }

        insta::assert_snapshot!(text);
    }

    fn test_word_ordering(level: &DesignedLevel, errors: &mut Vec<String>) {
        for (a, b) in level.words.iter().tuple_windows() {
            if a > b {
                errors.push(format!("{b} should come before {a}"));
            }
        }
    }

    fn test_grid_not_taboo(level: &DesignedLevel) -> Result<(), String> {
        if let Some(taboo_word) = orientation::find_taboo_word(&level.grid) {
            let mut gr = GridResult {
                grid: level.grid,
                words: level
                    .words
                    .iter()
                    .map(|x| FinderSingleWord::from_str(&x.text).unwrap())
                    .collect_vec(),
                letters: Default::default(), //doesn't matter
            };

            let optimize_result = orientation::try_optimize_orientation(&mut gr);

            match optimize_result {
                Ok(_) => {
                    return Err(format!(
                        "Level '{:<26}' Grid '{:?}' contains taboo word '{taboo_word:?}'. Try {}",
                        level.name.to_string(),
                        level.grid.iter().join(""),
                        gr.grid.iter().join("")
                    ));
                }
                Err(message) => {
                    return Err(format!(
                        "Level '{:<26}' Grid '{:?}' contains taboo word '{taboo_word:?}'. {}",
                        level.name.to_string(),
                        level.grid.iter().join(""),
                        message
                    ));
                }
            }
        }
        Ok(())
    }

    //spellchecker:disable
    pub const TABOO_EXCEPTIONS: &[&str] = &["ZEUTMSEIORHDAPON"];
    //spellchecker:enable
}
