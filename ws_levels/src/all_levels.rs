use itertools::Itertools;
use lazy_static::lazy_static;
use ws_core::{DesignedLevel, Numbering, Ustr};

lazy_static! { //todo data_bake
    pub(crate) static ref TUTORIAL: Vec<DesignedLevel> = include_str!("levels/tutorial.tsv")
        .lines()
        .map(DesignedLevel::from_tsv_line)
        .map(|x| x.unwrap())
        .collect_vec();
    pub(crate) static ref EU_CAPITALS: Vec<DesignedLevel> = number_levels(
        include_str!("levels/global_location/european_capitals.tsv")
            .lines()
            .map(DesignedLevel::from_tsv_line)
            .map(|x| x.unwrap()),
        "European Capitals"
    );
    pub(crate) static ref EU_COUNTRIES: Vec<DesignedLevel> = number_levels(
        include_str!("levels/global_location/european_countries.tsv")
            .lines()
            .map(DesignedLevel::from_tsv_line)
            .map(|x| x.unwrap()),
        "European Countries"
    );
    pub(crate) static ref US_STATES: Vec<DesignedLevel> = number_levels(
        include_str!("levels/global_location/us_states.tsv")
            .lines()
            .map(DesignedLevel::from_tsv_line)
            .map(|x| x.unwrap()),
        "US States"
    );
    pub(crate) static ref GREEK_GODS: Vec<DesignedLevel> = number_levels(
        include_str!("levels/history_and_mythology/greek_gods.tsv")
            .lines()
            .map(DesignedLevel::from_tsv_line)
            .map(|x| x.unwrap()),
        "Greek Gods"
    );
    pub(crate) static ref US_PRESIDENTS: Vec<DesignedLevel> = number_levels(
        include_str!("levels/history_and_mythology/us_presidents.tsv")
            .lines()
            .map(DesignedLevel::from_tsv_line)
            .map(|x| x.unwrap()),
        "US Presidents"
    );
    pub(crate) static ref SCIENTISTS: Vec<DesignedLevel> = number_levels(
        include_str!("levels/science/scientists.tsv")
            .lines()
            .map(DesignedLevel::from_tsv_line)
            .map(|x| x.unwrap()),
        "Scientists"
    );
    pub(crate) static ref INSECTS: Vec<DesignedLevel> = number_levels(
        include_str!("levels/science/insects.tsv")
            .lines()
            .map(DesignedLevel::from_tsv_line)
            .map(|x| x.unwrap()),
        "Insects"
    );
    pub(crate) static ref FRUIT: Vec<DesignedLevel> = number_levels(
        include_str!("levels/science/fruit.tsv")
            .lines()
            .map(DesignedLevel::from_tsv_line)
            .map(|x| x.unwrap()),
        "Fruit"
    );
    pub(crate) static ref GEMSTONES: Vec<DesignedLevel> = number_levels(
        include_str!("levels/science/gemstones.tsv")
            .lines()
            .map(DesignedLevel::from_tsv_line)
            .map(|x| x.unwrap()),
        "Gemstones"
    );
    pub(crate) static ref VEGETABLES: Vec<DesignedLevel> = number_levels(
        include_str!("levels/science/vegetables.tsv")
            .lines()
            .map(DesignedLevel::from_tsv_line)
            .map(|x| x.unwrap()),
        "Vegetables"
    );
    pub(crate) static ref ELEMENTS: Vec<DesignedLevel> = number_levels(
        include_str!("levels/science/elements.tsv")
            .lines()
            .map(DesignedLevel::from_tsv_line)
            .map(|x| x.unwrap()),
        "Elements"
    );
    pub(crate) static ref QUEENS: Vec<DesignedLevel> = number_levels(
        include_str!("levels/history_and_mythology/famous_queens.tsv")
            .lines()
            .map(DesignedLevel::from_tsv_line)
            .map(|x| x.unwrap()),
        "Famous Queens"
    );
    pub(crate) static ref ROMAN_GODS: Vec<DesignedLevel> = number_levels(
        include_str!("levels/history_and_mythology/roman_gods.tsv")
            .lines()
            .map(DesignedLevel::from_tsv_line)
            .map(|x| x.unwrap()),
        "Roman Gods"
    );
    pub(crate) static ref EGYPTIAN_GODS: Vec<DesignedLevel> = number_levels(
        include_str!("levels/history_and_mythology/egyptian_gods.tsv")
            .lines()
            .map(DesignedLevel::from_tsv_line)
            .map(|x| x.unwrap()),
        "Egyptian Gods"
    );
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

    use std::str::FromStr;

    use ws_core::finder::{helpers::FinderSingleWord, node::GridResult, orientation};

    use super::*;
    pub fn get_all_levels() -> Vec<DesignedLevel> {
        [
            TUTORIAL.iter(),
            EU_CAPITALS.iter(),
            EU_COUNTRIES.iter(),
            US_STATES.iter(),
            GREEK_GODS.iter(),
            US_PRESIDENTS.iter(),
            SCIENTISTS.iter(),
            INSECTS.iter(),
            FRUIT.iter(),
            GEMSTONES.iter(),
            VEGETABLES.iter(),
            ELEMENTS.iter(),
            QUEENS.iter(),
            GREEK_GODS.iter(),
            EGYPTIAN_GODS.iter(),
            ROMAN_GODS.iter(),
        ]
        .iter()
        .cloned()
        .flat_map(|x| x)
        .cloned()
        .collect_vec()
    }

    #[test]
    pub fn test_all_levels_valid() {
        let levels = get_all_levels();

        assert!(levels.len() > 5);

        let mut taboo_errors: Vec<String> = Default::default();

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
                taboo_errors.push(err);
            }
        }

        for error in taboo_errors.iter() {
            println!("{error}")
        }

        assert!(taboo_errors.is_empty())
    }

    #[test]
    pub fn test_daily_challenge_levels_valid() {
        assert!(DAILY_CHALLENGE.len() > 10);

        let mut taboo_errors: Vec<String> = Default::default();

        for level in DAILY_CHALLENGE.clone().into_iter() {
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
                taboo_errors.push(err);
            }
        }

        for error in taboo_errors.iter() {
            println!("{error}")
        }

        assert!(taboo_errors.is_empty())
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

    lazy_static! {
        pub(crate) static ref DAILY_CHALLENGE: Vec<DesignedLevel> =
            include_str!("../../ws_game/daily.tsv")
                .lines()
                .map(DesignedLevel::from_tsv_line)
                .map(|x| x.unwrap())
                .collect_vec();
    }
}
