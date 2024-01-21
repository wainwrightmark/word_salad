use iai_callgrind::{library_benchmark, library_benchmark_group, main};
use ws_core::{
    finder::{
        counter::FakeCounter,
        helpers::{FinderSingleWord, LetterCounts},
        node::try_make_grid_with_blank_filling,
    },
    Character,
};

fn set_up(input: &str) -> (LetterCounts, Vec<FinderSingleWord>) {
    let words = ws_core::finder::helpers::make_finder_group_vec_from_file(input);
    let words: Vec<FinderSingleWord> = words.into_iter().flat_map(|x| x.words).collect();

    let mut letters = LetterCounts::default();
    for word in words.iter() {
        letters = letters
            .try_union(&word.counts)
            .expect("Should be able to combine letters");
    }
    let letter_count = letters.into_iter().count();

    if letter_count > 16 {
        panic!("Too many letters");
    }

    let mut blanks_to_add = 16usize.saturating_sub(letter_count);
    while blanks_to_add > 0 {
        match letters.try_insert(Character::Blank) {
            Some(n) => letters = n,
            None => {
                panic!("Prime bag wont accept more blanks")
            }
        }
        blanks_to_add -= 1;
    }

    (letters, words)
}
// spellchecker:disable
#[library_benchmark]
#[bench::europe(set_up("Croatia\nRomania\nIreland\nLatvia\nPoland\nFrance\nMalta"))]
#[bench::states_1(set_up("Utah\nOhio\nMaine\nIdaho\nIndiana\nMontana\nArizona"))]
#[bench::states_2(set_up("IOWA\nOHIO\nIDAHO\nUTAH\nHAWAII\nINDIANA\nMONTANA"))]
#[bench::pokemon(set_up(
    "Abra\nDratini\nArbok\nNidoran\nNidorina\nNidorino\nDragonite\nNidoking\nDragonair"
))]
#[bench::colors(set_up("Teal\nSage\nGreen\nCyan\nOlive\nGray\nClaret\nMagenta\nSilver"))]
// spellchecker:enable
fn solve_grid(input: (LetterCounts, Vec<FinderSingleWord>)) {
    let exclude_words = vec![];
    let mut solution = None;
    try_make_grid_with_blank_filling(
        input.0,
        &input.1,
        &exclude_words,
        Character::E,
        &mut FakeCounter,
        &mut solution,
    )
}

library_benchmark_group!(name= solve_group; benchmarks=solve_grid);
main!(library_benchmark_groups = solve_group);
