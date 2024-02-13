use crate::{finder::*, CharsArray, DesignedLevel};
use itertools::Itertools;
use std::collections::{BTreeMap, HashSet};

use const_sized_bit_set::BitSet;

use self::node::GridResult;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ClusterScoreBuilder<const W: usize> {
    /// Higher is better
    pub min_underlap: u32,

    pub all_items: BitSet<W>,
    //pub trigrams: hashbrown::HashSet<Trigram>,
    /// The sum of the counts of symmetric differences between all pairs of elements.
    /// Higher is better
    pub total_underlap: u32,
    // /// The total overlap between all pairs of elements.
    // /// Lower is better
    // pub total_overlap: std::cmp::Reverse<u32>,
    /// The total sum of the item count of all elements
    /// Higher is better
    pub total_element_items: u32,
}

impl<const W: usize> Into<ClusterScore> for ClusterScoreBuilder<W> {
    fn into(self) -> ClusterScore {
        let ClusterScoreBuilder {
            min_underlap,
            all_items,
            total_underlap,
            total_element_items,
        } = self;

        ClusterScore {
            min_underlap,
            all_items_count: all_items.count(),
            total_underlap,
            total_element_items,
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, PartialOrd, Ord)]
pub struct ClusterScore {
    /// Higher is better
    pub min_underlap: u32,

    pub all_items_count: u32,
    //pub trigrams: hashbrown::HashSet<Trigram>,
    /// The sum of the counts of symmetric differences between all pairs of elements.
    /// Higher is better
    pub total_underlap: u32,
    // /// The total overlap between all pairs of elements.
    // /// Lower is better
    // pub total_overlap: std::cmp::Reverse<u32>,
    /// The total sum of the item count of all elements
    /// Higher is better
    pub total_element_items: u32,
}

impl<const W: usize> ClusterScoreBuilder<W> {
    pub fn increment(&self, original_cluster: &[BitSet<W>], new_item: &BitSet<W>) -> Self {
        let mut result = self.clone();
        let new_item_count = new_item.count();
        result.total_element_items += new_item_count;

        for item in original_cluster {
            let left_underlap = item.intersect(&new_item.negate()).count();
            let right_underlap = new_item.intersect(&item.negate()).count();
            let lower_underlap = left_underlap.min(right_underlap);

            //let underlap = item.symmetric_difference(new_item).count();

            result.min_underlap = result.min_underlap.min(lower_underlap);
            result.total_underlap += lower_underlap;
        }

        result.all_items = result.all_items.union(new_item);

        // for t in Trigram::iter_from_bitset(new_item) {
        //     result.trigrams.insert(t);
        // }

        result
    }

    pub fn calculate(cluster: &[BitSet<W>]) -> Self {
        let total_element_items = cluster.iter().map(|x| x.count()).sum();
        let mut min_lower_underlap = u32::MAX;
        let mut total_underlap = 0;

        for (a, b) in cluster.iter().tuple_combinations() {
            let left_underlap = a.intersect(&b.negate()).count();
            let right_underlap = b.intersect(&a.negate()).count();
            let lower_underlap = left_underlap.min(right_underlap);

            min_lower_underlap = min_lower_underlap.min(lower_underlap);
            total_underlap += lower_underlap;
        }

        let mut all_items = BitSet::<W>::EMPTY;

        for set in cluster {
            all_items = all_items.union(set);
        }

        // let mut trigrams: hashbrown::HashSet<Trigram> = Default::default();
        // for c in cluster {
        //     for t in Trigram::iter_from_bitset(c) {
        //         trigrams.insert(t);
        //     }
        // }

        Self {
            min_underlap: min_lower_underlap,
            all_items,
            total_underlap,
            total_element_items,
        }
    }
}

impl<'a, const W: usize> From<&'a [BitSet<W>]> for ClusterScoreBuilder<W> {
    fn from(cluster: &'a [BitSet<W>]) -> Self {
        Self::calculate(cluster)
    }
}
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AdjacencyScore {
    pub max_adjacent: u32,
    pub mean_adjacency: f32,
}

impl AdjacencyScore {
    pub fn calculate<const W: usize>(slice: &[BitSet<W>]) -> Self {
        let mut max = 0u32;
        let mut total = 0f32;
        let mut count = 0f32;
        for (a, b) in slice.iter().tuple_windows() {
            count += 1.0;
            let overlap = a.intersect(b).count();
            max = max.max(overlap);
            total += overlap as f32;
        }

        Self {
            max_adjacent: max,
            mean_adjacency: total / count,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Cluster {
    pub grids: Vec<GridResult>,
    pub score: ClusterScore,
    pub adjacency_score: AdjacencyScore,
}

impl Cluster {
    pub fn new<const W: usize>(
        mut points: Vec<BitSet<W>>,
        map: &BTreeMap<BitSet<W>, GridResult>,
    ) -> Self {
        cluster_ordering::order_cluster(&mut points);
        let score_builder: ClusterScoreBuilder<W> =
            ClusterScoreBuilder::calculate(points.as_slice());
        let adjacency_score = AdjacencyScore::calculate(points.as_slice());

        let grids = points
            .iter()
            .flat_map(|p| map.get(p))
            .cloned()
            .collect_vec();

        Self {
            grids,
            score: score_builder.into(),
            adjacency_score,
        }
    }

    pub fn from_levels(levels: &[DesignedLevel]) -> Self {
        let all_words: Vec<CharsArray> = levels
            .iter()
            .flat_map(|x| x.words.iter().map(|w| w.characters.clone()))
            .sorted()
            .dedup()
            .collect();

        let sets: Vec<(BitSet<4>, GridResult)> = levels
            .iter()
            .map(|level| {
                let set: HashSet<CharsArray> =
                    HashSet::from_iter(level.words.iter().map(|x| x.characters.clone()));

                let set: BitSet<4> = BitSet::<4>::from_fn(|word_id| {
                    let Some(word) = all_words.get(word_id) else {
                        return false;
                    };
                    set.contains(word)
                });
                let grid_result: GridResult = level.into();
                (set, grid_result)
            })
            .collect();

        let points = sets.iter().map(|x| x.0).collect_vec();
        let map: BTreeMap<BitSet<4>, GridResult> = sets.into_iter().collect();

        Self::new(points, &map)
    }

    pub fn header(&self) -> String {
        let mean_underlap =
            self.score.total_underlap as f32 / (binomial_coefficient(self.grids.len(), 2) as f32);
        let distinct_items = self
            .grids
            .iter()
            .flat_map(|x| x.words.iter())
            .sorted()
            .dedup()
            .count();

        let mean_util = (self
            .grids
            .iter()
            .map(|g| g.words.iter().map(|w| w.array.len()).sum::<usize>())
            .sum::<usize>() as f32)
            / (16 * self.grids.len()) as f32;

        format!(
            "Grids: {l:2}\tMin underlap: {min_underlap:2}\tMean underlap: {mean_underlap:2.2}\tMean items: {mean_items:2.2}\tDistinct items: {distinct_items:3}\tMax adjacent {max_adj:2}\tMean adjacent {mean_adj:2.2}\tMean utilization {mean_util:1.2}",
            l = self.grids.len(),
            min_underlap = self.score.min_underlap ,
            mean_items = self.score.total_element_items as f32 / self.grids.len() as f32,
            max_adj = self.adjacency_score.max_adjacent,
            mean_adj = self.adjacency_score.mean_adjacency,
        )
    }
}

impl std::fmt::Display for Cluster {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let grid_words = self.grids.iter().join("\n");
        let header = self.header();

        write!(f, "{header}\n{grid_words}",)
    }
}

fn binomial_coefficient(n: usize, k: usize) -> usize {
    let mut res = 1;
    for i in 0..k {
        res = (res * (n - i)) / (i + 1);
    }
    res
}
