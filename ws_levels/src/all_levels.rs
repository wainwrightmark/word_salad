use itertools::Itertools;
use lazy_static::lazy_static;
use ws_core::DesignedLevel;

lazy_static! {
    pub(crate) static ref TUTORIAL: Vec<DesignedLevel> = include_str!("levels/tutorial.tsv")
        .lines()
        .map(DesignedLevel::from_tsv_line)
        .map(|x| x.unwrap())
        .collect_vec();
    pub(crate) static ref DAILY_CHALLENGE: Vec<DesignedLevel> =
        include_str!("levels/daily_challenge.tsv")
            .lines()
            .map(DesignedLevel::from_tsv_line)
            .map(|x| x.unwrap())
            .collect_vec();
    pub(crate) static ref EU_CAPITALS: Vec<DesignedLevel> = number_levels(
        include_str!("levels/global_location/eu_capitals.tsv")
            .lines()
            .map(DesignedLevel::from_tsv_line)
            .map(|x| x.unwrap()),
        "EU Capitals"
    );
    pub(crate) static ref EU_COUNTRIES: Vec<DesignedLevel> = number_levels(
        include_str!("levels/global_location/eu_countries.tsv")
            .lines()
            .map(DesignedLevel::from_tsv_line)
            .map(|x| x.unwrap()),
        "EU Countries"
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
}

pub fn number_levels(
    levels: impl Iterator<Item = DesignedLevel>,
    prefix: &'static str,
) -> Vec<DesignedLevel> {
    let mut r = vec![];
    let mut index = 1;
    for mut l in levels {
        l.name = format!("{prefix} {index}");
        index += 1;
        r.push(l)
    }

    r
}

#[cfg(test)]
pub mod tests {

    use super::*;
    pub fn get_all_levels() -> Vec<DesignedLevel> {
        [
            TUTORIAL.iter(),
            DAILY_CHALLENGE.iter(),
            EU_CAPITALS.iter(),
            EU_COUNTRIES.iter(),
            US_STATES.iter(),
            GREEK_GODS.iter(),
            US_PRESIDENTS.iter(),
            SCIENTISTS.iter(),
            INSECTS.iter(),
        ]
        .iter()
        .cloned()
        .flat_map(|x| x)
        .cloned()
        .collect_vec()
    }

    #[test]
    pub fn test_all_levels_valid() {
        for level in get_all_levels() {
            let name = &level.name;
            assert!(level.words.len() > 0, "Level {name} should have words");
            for word in level.words.into_iter() {
                let solution = word.find_solution(&level.grid);
                if solution.is_none() {
                    panic!(
                        "Level '{name}' has no solution for '{word}'",
                        word = word.text
                    )
                }
            }
        }
    }
}
