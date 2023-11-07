use criterion::{criterion_group, criterion_main, Criterion};
use ws_core::{finder::{node::try_make_grid_with_blank_filling, helpers::LetterCounts, counter::FakeCounter}, Character};

pub fn criterion_benchmark(c: &mut Criterion) {
    let words = ws_core::finder::helpers::make_words_vec_from_file(
        "CROATIA, ROMANIA, IRELAND, LATVIA, POLAND, FRANCE, MALTA",
    );

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

    c.bench_function("EU Countries", |b| {
        b.iter(|| {
            try_make_grid_with_blank_filling(letters, &words, Character::E, &mut FakeCounter)
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
