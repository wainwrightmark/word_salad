use std::cmp::Reverse;

use crate::combinations::WordSet;

pub fn order_cluster(cluster: &mut Vec<WordSet>) {
    let Some((first_index, _)) = cluster
        .iter()
        .enumerate()
        .max_by_key(|x| OrderingScore::calculate(x.1, &cluster))
    else {
        return;
    };

    cluster.swap(0, first_index);
    let mut number_ordered = 1usize;

    loop {
        let Some(prev) = cluster.get(number_ordered.saturating_sub(1usize)) else {
            return;
        };

        let Some((index, _)) =
            cluster
                .iter()
                .enumerate()
                .skip(number_ordered)
                .min_by_key(|(_, x)| {
                    (
                        x.intersect(prev).count(),
                        Reverse(OrderingScore::calculate(x, &cluster)),
                    )
                })
        else {
            return;
        };

        cluster.swap(number_ordered, index);
        number_ordered += 1;
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
