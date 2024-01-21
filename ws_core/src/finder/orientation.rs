use std::collections::HashSet;

use crate::{
    character,
    finder::{helpers::FinderSingleWord, node::GridResult},
    prelude::*,
};
use itertools::Itertools;
use lazy_static::lazy_static;

lazy_static! {
    static ref TABOO_WORDS: HashSet<CharsArray> = {
        let text = include_str!("taboo.txt");

        let words: HashSet<CharsArray> = text
            .lines()
            .map(|l| character::normalize_characters_array(l).ok())
            .flatten()
            .collect();
        words
    };
    static ref TABOO_PREFIXES: HashSet<CharsArray> = {
        let mut set: HashSet<CharsArray> = Default::default();

        for word in TABOO_WORDS.iter() {
            for len in 1..=word.len() {
                let mut clone = word.clone();
                clone.truncate(len);
                set.insert(clone);
            }
        }
        set
    };
}

pub fn find_taboo_word(grid: &Grid) -> Option<CharsArray> {
    fn find_taboo_inner(
        grid: &Grid,
        prefix: &mut CharsArray,
        last_tile: Tile,
        mut allow_wrap: bool,
    ) -> Option<CharsArray> {
        if TABOO_WORDS.contains(prefix) {
            return Some(prefix.clone());
        }

        let next_tiles = [
            if allow_wrap {
                last_tile.try_next()
            } else {
                last_tile.const_add(&Vector::EAST)
            },
            last_tile.const_add(&Vector::SOUTH),
            last_tile.const_add(&Vector::SOUTH_EAST),
        ];

        for next_tile in next_tiles.into_iter().flatten() {
            let c = grid[next_tile];
            prefix.push(c);
            if let Some(answer) = find_taboo_inner(grid, prefix, next_tile, allow_wrap) {
                return Some(answer);
            }
            prefix.pop();
            allow_wrap = false; //basically only allow wrap
        }

        None
    }

    for tile in Tile::iter_by_row() {
        let mut prefix = CharsArray::new();
        prefix.push(grid[tile]);
        if TABOO_PREFIXES.contains(&prefix) {
            if let Some(answer) = find_taboo_inner(grid, &mut prefix, tile, tile.x() <= 1) {
                return Some(answer);
            }
        }
    }

    None
}

/// Returns Ok(true) if the orientation was changed
pub fn try_optimize_orientation(grid_result: &mut GridResult) -> Result<bool, String> {
    let flips = [FlipAxes::Horizontal, FlipAxes::None];
    let rotations = [
        QuarterTurns::One,
        QuarterTurns::Two,
        QuarterTurns::Three,
        QuarterTurns::Zero, //do zero and one last because max returns the last maximal element
    ];

    let transforms = flips.into_iter().cartesian_product(rotations);

    if let Some(new_grid) = transforms
        .map(|(axes, quarter_turns)| {
            let mut new_grid = grid_result.grid.clone();
            new_grid.rotate(quarter_turns);
            new_grid.flip(axes);
            new_grid
        })
        .filter(|grid| find_taboo_word(grid).is_none())
        .max_by_key(|new_grid| calculate_max_score(&new_grid, &grid_result.words))
    {
        if grid_result.grid != new_grid {
            //log::info!("Changed \n{}\n to \n{new_grid}", grid_result.grid);
            grid_result.grid = new_grid;
            Ok(true)
        } else {
            Ok(false)
        }
    } else {
        let mut taboo_words: HashSet<String> = Default::default();

        for (axes, quarter_turns) in flips.into_iter().cartesian_product(rotations) {
            let mut new_grid = grid_result.grid.clone();
            new_grid.rotate(quarter_turns);
            new_grid.flip(axes);
            if let Some(word) = find_taboo_word(&new_grid) {
                taboo_words.insert(word.iter().join(""));
            }
        }

        Err(format!(
            "Could not find a good orientation for \n{} (taboo words: {})",
            grid_result.grid,
            taboo_words.into_iter().join(", ")
        ))
    }
}

pub fn calculate_best_word(grid_result: &GridResult) -> (FinderSingleWord, i32) {
    grid_result
        .words
        .iter()
        .map(|word| (word, calculate_score(word, &grid_result.grid)))
        .max_by_key(|x| x.1)
        .map(|x| (x.0.clone(), x.1))
        .unwrap()
}

pub fn find_single_row_word(grid_result: &GridResult) -> Option<FinderSingleWord> {
    for word in grid_result.words.iter() {
        if word.array.len() == 4 {
            for s in find_solutions(&word.array, &grid_result.grid) {
                if s.iter().map(|t| t.x()).all_equal() || s.iter().map(|t| t.y()).all_equal() {
                    return Some(word.clone());
                }
            }
        }
    }
    return None;
}

fn calculate_max_score(grid: &Grid, words: &[FinderSingleWord]) -> i32 {
    //println!("{}", grid);
    //println!();
    words
        .iter()
        .map(|word| calculate_score(word, grid))
        .max()
        .unwrap_or_default()
}

fn calculate_score(word: &FinderSingleWord, grid: &Grid) -> i32 {
    find_solutions(&word.array, grid)
        .into_iter()
        .map(|x| {
            let score = score_solution(&x);
            //println!("{}: {score}", word.text);
            score
        })
        .max()
        .unwrap_or_default()
}

fn score_solution(solution: &ArrayVec<Tile, 16>) -> i32 {
    //look at the first five tiles. Score is accumulated based on letters going left to right, preferably in the same row
    const FIRST_ROW: Tile = Tile::new_const::<0, 1>();
    let mut total = match solution.get(0) {
        Some(&Tile::NORTH_WEST) => 10, //bonus for being top left
        Some(&FIRST_ROW) => 8,         // bonus for being one below top left
        _ => 0,
    };

    let mut streak = true;

    let mut windows = solution.iter().tuple_windows();

    while let Some((a, b)) = windows.next() {
        match b.x().cmp(&a.x()) {
            std::cmp::Ordering::Less => {
                return total - 3;
            }
            std::cmp::Ordering::Equal => streak = false,
            std::cmp::Ordering::Greater => {
                if a.y() == b.y() {
                    total += if streak { 4 } else { 2 };
                } else {
                    total += 1;
                    streak = false;
                }
            }
        }
    }

    total
}

#[cfg(test)]
pub mod tests {
    use std::str::FromStr;

    use crate::prelude;
    use itertools::Itertools;
    use test_case::test_case;

    use super::*;

    /* spellchecker:disable */

    #[test_case("VENMOUAULTRSHPEN	7	Earth	Mars	Neptune	Pluto	Saturn	Uranus	Venus")]
    #[test_case(
        "\
    ZEUA\
    MSTI\
    EREH\
    _DAN	8	Ares	Athena	Demeter	Hades	Hera	Hermes	Hestia	Zeus"
    )]

    pub fn test_optimize(input: &str) {
        let mut grid_result = GridResult::from_str(input).unwrap();
        let before = grid_result.grid.iter().join("");
        let result = try_optimize_orientation(&mut grid_result);
        let after = grid_result.grid.iter().join("");

        assert_eq!(before, after);
        assert_eq!(Ok(false), result);
    }

    #[test_case("ABCDEFGHIJKLMNOP", "")]
    #[test_case("WANKZZZZZZZZZZZZ", "WANK")]
    #[test_case("ZZZZWANKZZZZZZZZ", "WANK")]
    #[test_case(
        "\
    ZZZW\
    ZZZA\
    ZZZN\
    ZZZK",
        "WANK"
    )]
    #[test_case(
        "\
    ZWAN\
    KZZZ\
    ZZZZ\
    ZZZZ",
        "WANK"
    )]
    #[test_case(
        "\
    WZZZ\
    ZAZZ\
    ZZNZ\
    ZZZK",
        "WANK"
    )]
    #[test_case(
        "\
    ZEUA\
    MSTI\
    EREH\
    _DAN",
        ""
    )]
    pub fn test_taboo(input: &str, expected: &str) {
        let grid = prelude::try_make_grid(input).unwrap();

        let expected = if expected == "" {
            None
        } else {
            Some(crate::character::normalize_characters_array(expected).unwrap())
        };

        let actual = find_taboo_word(&grid);

        assert_eq!(actual, expected);
    }
}
