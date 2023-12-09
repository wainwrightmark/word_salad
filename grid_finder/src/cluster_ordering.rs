use std::cmp::Reverse;

use crate::combinations::WordSet;

pub fn order_cluster(mut cluster: Vec<WordSet>) -> Vec<WordSet> {
    let mut new_ordering = Vec::with_capacity(cluster.len());

    let Some((first_index, _)) = cluster
        .iter()
        .enumerate()
        .max_by_key(|x| OrderingScore::calculate(x.1, &cluster))
    else {
        return new_ordering;
    };

    let first_set = cluster.remove(first_index);

    new_ordering.push(first_set);

    loop {
        let prev = new_ordering.last().unwrap();

        let Some((index, _)) = cluster.iter().enumerate().min_by_key(|(_, x)| {
            (
                x.intersect(prev).count(),
                Reverse(OrderingScore::calculate(x, &cluster)),
            )
        }) else {
            return new_ordering;
        };

        let next_set = cluster.remove(index);

        new_ordering.push(next_set);
    }
}

const KEY_SIZE: usize = 16;

#[derive(Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct OrderingScore([Reverse<u8>; KEY_SIZE]);

impl OrderingScore {
    pub fn calculate(point: &WordSet, all: &[WordSet]) -> Self {
        let mut numbers = [Reverse::<u8>(0); KEY_SIZE];

        for x in all {
            if point == x {
                continue;
            };
            let c = point.intersect(x).count() as usize;

            numbers[c] = Reverse(numbers[c].0 + 1);
        }

        return Self(numbers);
    }
}
