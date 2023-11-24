use itertools::Itertools;
use ws_core::{
    finder::{helpers::FinderWord, node::GridResult},
    prelude::*,
};

pub fn optimize_orientation(grid_result: &mut GridResult) {
    let flips = [FlipAxes::None, FlipAxes::Horizontal];
    let rotations = [
        QuarterTurns::Zero,
        QuarterTurns::One,
        QuarterTurns::Two,
        QuarterTurns::Three,
    ];

    let transforms = flips.into_iter().cartesian_product(rotations);

    let (axes, quarter_turns) = transforms
        .max_by_key(|(axes, quarter_turns)| {
            let mut new_grid = grid_result.grid.clone();
            new_grid.rotate(*quarter_turns);
            new_grid.flip(*axes);

            calculate_max_score(&new_grid, &grid_result.words)
        })
        .unwrap();

    grid_result.grid.rotate(quarter_turns);
    grid_result.grid.flip(axes);
}

fn calculate_max_score(grid: &Grid, words: &[FinderWord]) -> i32 {
    //println!("{}", grid);
    //println!();
    words
        .iter()
        .map(|word| calculate_score(word, grid))
        .max()
        .unwrap_or_default()
}

fn calculate_score(word: &FinderWord, grid: &Grid) -> i32 {
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

    use itertools::Itertools;
    use test_case::test_case;
    use ws_core::finder::node::GridResult;

    use super::optimize_orientation;

    #[test_case(
        // spellchecker:disable-next-line
        "VENMOUAULTRSHPEN	7	Earth	Mars	Neptune	Pluto	Saturn	Uranus	Venus"
    )]
    #[test_case(
        // spellchecker:disable-next-line
        "ZEUAMSTIEREH_DAN	8	Ares    	Athena  	Demeter 	Hades   	Hera    	Hermes  	Hestia  	Zeus    "
    )]

    pub fn test_optimize(input: &str) {
        let mut grid_result = GridResult::from_str(input).unwrap();
        let before =grid_result.grid.iter().join("");
        optimize_orientation(&mut grid_result);
        let after =grid_result.grid.iter().join("");

        assert_eq!(before, after)
    }
}
