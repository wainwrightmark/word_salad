use std::cmp::Reverse;

use const_sized_bit_set::BitSet;
use itertools::Itertools;

pub fn order_cluster<const W: usize>(cluster: &mut Vec<BitSet<W>>) {
    let Some((first_index, _)) = cluster
        .iter()
        .enumerate()
        .max_by_key(|x| OrderingScore::calculate(x.1, cluster))
    else {
        return;
    };

    cluster.swap(0, first_index);
    let mut number_ordered = 1usize;

    'reorder: loop {
        let Some(prev) = cluster.get(number_ordered.saturating_sub(1usize)) else {
            break 'reorder;
        };

        let Some((index, _)) =
            cluster
                .iter()
                .enumerate()
                .skip(number_ordered)
                .min_by_key(|(_, x)| {
                    (
                        x.intersect(prev).count(),
                        Reverse(OrderingScore::calculate(x, cluster)),
                    )
                })
        else {
            break 'reorder;
        };

        cluster.swap(number_ordered, index);
        number_ordered += 1;
    }

    //rotate the list to minimize adjacency
    let Some(rotate_index) = (0..cluster.len()).map(|index|{
        let s1 = cluster[index];
        let s2 = cluster[(index + 1) % cluster.len()];
        s1.intersect(&s2).count()
    }).position_max() else{return;};

    cluster.rotate_left(rotate_index + 1)

}

const KEY_SIZE: usize = 16;

#[derive(Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct OrderingScore([Reverse<u8>; KEY_SIZE]);

impl OrderingScore {
    pub fn calculate<const W: usize>(point: &BitSet<W>, all: &[BitSet<W>]) -> Self {
        let mut numbers = [Reverse::<u8>(0); KEY_SIZE];

        for x in all {
            if point == x {
                continue;
            };
            let c = point.intersect(x).count() as usize;

            numbers[c] = Reverse(numbers[c].0 + 1);
        }

        Self(numbers)
    }
}
