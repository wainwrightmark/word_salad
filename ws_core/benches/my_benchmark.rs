use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
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

//

pub fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("Solve Grid");
    group.sample_size(10);
    let exclude_words = vec![];
    // spellchecker:disable
    let euro_countries = (
        "European Countries 1",
        "Croatia\nRomania\nIreland\nLatvia\nPoland\nFrance\nMalta",
    );
    let states1 = (
        "Us States 1",
        "Utah\nOhio\nMaine\nIdaho\nIndiana\nMontana\nArizona",
    );
    let states2 = (
        "Us States 2",
        "IOWA\nOHIO\nIDAHO\nUTAH\nHAWAII\nINDIANA\nMONTANA",
    );
    let pokemon = (
        "Pokemon",
        "Abra\nDratini\nArbok\nNidoran\nNidorina\nNidorino\nDragonite\nNidoking\nDragonair",
    );
    let colors = (
        "Colors",
        "Teal\nSage\nGreen\nCyan\nOlive\nGray\nClaret\nMagenta\nSilver",
    );
    // spellchecker:enable
    for (name, data) in [euro_countries, states1, states2, pokemon, colors] {
        let input = set_up(data);

        group.bench_with_input(BenchmarkId::new("Solve: ", name), &input, |b, i| {
            b.iter(|| {
                let mut solution = None;
                try_make_grid_with_blank_filling(
                    i.0,
                    &i.1,
                    &exclude_words,
                    Character::E,
                    &mut FakeCounter,
                    &mut solution,
                )
            })
        });
    }
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
