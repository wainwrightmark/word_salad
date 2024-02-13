use std::collections::{BTreeMap, HashSet};

use indicatif::{ProgressBar, ProgressStyle};
use itertools::Itertools;
use ws_core::{
    finder::{helpers::FinderSingleWord, node::GridResult},
    CharsArray,
};

use crate::cluster_ordering;
use const_sized_bit_set::BitSet;

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
}

impl std::fmt::Display for Cluster {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let grid_words = self.grids.iter().join("\n");

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

        write!(
            f,
            "Grids: {l:2}\tMin underlap: {min_underlap:2}\tMean underlap: {mean_underlap:2.2}\tMean items: {mean_items:2.2}\tDistinct items: {distinct_items:3}\tMax adjacent {max_adj:2}\tMean adjacent {mean_adj:2.2}\tMean utilization {mean_util:1.2}\n{grid_words}",
            l = self.grids.len(),
            min_underlap = self.score.min_underlap ,
            mean_items = self.score.total_element_items as f32 / self.grids.len() as f32,
            max_adj = self.adjacency_score.max_adjacent,
            mean_adj = self.adjacency_score.mean_adjacency,
        )
    }
}

pub fn cluster_words<const W: usize>(
    groups: Vec<GridResult>,
    all_words: &Vec<FinderSingleWord>,
    max_clusters: usize,
    category: Option<String>,
) -> Vec<Cluster> {
    let clusters = max_clusters.min(groups.len());
    let pb = category.map(|category| {
        ProgressBar::new(clusters as u64)
            .with_style(
                ProgressStyle::with_template("{prefix} {msg:35} {pos:3}/{len:3} {elapsed} {bar}")
                    .unwrap(),
            )
            .with_prefix(format!(
                "{category:35}: {:7} grids {:3} words",
                groups.len(),
                all_words.len()
            ))
            .with_message("Clustering: Processing Grids")
    });

    pb.iter().for_each(|x| x.tick());

    let mut results: Vec<Cluster> = Default::default();

    let map: BTreeMap<BitSet<W>, GridResult> = groups
        .into_iter()
        .map(|grid_result| {
            let set: HashSet<CharsArray> =
                HashSet::from_iter(grid_result.words.iter().map(|x| x.array.clone()));

            let set: BitSet<W> = BitSet::<W>::from_fn(|word_id| {
                let Some(word) = all_words.get(word_id) else {
                    return false;
                };
                set.contains(&word.array)
            });
            (set, grid_result)
        })
        .collect();

    pb.iter()
        .for_each(|x| x.set_message("Clustering: Finding First Points"));

    let all_points = map
        .keys()
        .cloned()
        .sorted_by_key(|x| std::cmp::Reverse(x.count()))
        .collect_vec();

    let Some((point1, point2)) = find_best_pair(&all_points) else {
        println!("Must be at least two groups to find clusters");
        return results;
    };

    pb.iter()
        .for_each(|x| x.set_message("Clustering: Finding larger clusters"));

    let mut chosen_points: Vec<BitSet<W>> = vec![point1, point2];
    results.push(Cluster::new(chosen_points.clone(), &map));

    while chosen_points.len() < clusters {
        pb.iter().for_each(|x| x.inc(1));
        let point_to_add = find_best_point_to_add(chosen_points.as_slice(), all_points.as_slice());
        chosen_points.push(point_to_add);
        //let mut swaps = 0;
        'inner: loop {
            let distance: ClusterScore =
                ClusterScoreBuilder::calculate(chosen_points.as_slice()).into();
            let Some((removed_index, best_point, new_distance)) = (0..chosen_points.len())
                .map(|index| {
                    let mut new_chosen = chosen_points.clone();
                    let _removed = new_chosen.remove(index);
                    let best_point = find_best_point_to_add(&new_chosen, &all_points);
                    new_chosen.push(best_point);
                    let score_builder = ClusterScoreBuilder::calculate(&new_chosen);
                    let score: ClusterScore = score_builder.into();
                    (index, best_point, score)
                })
                .max_by_key(|(_, _, score)| *score)
            else {
                break 'inner;
            };

            if new_distance > distance {
                let _ = chosen_points.remove(removed_index);
                chosen_points.push(best_point);
                //swaps += 1;
            } else {
                break 'inner;
            }
        }
        results.push(Cluster::new(chosen_points.clone(), &map));
    }

    if let Some(final_adjacency) = results.last().map(|x| x.adjacency_score) {
        pb.iter().for_each(|x| {
            x.set_message(format!(
                "Finished: adjacency max {:2} mean {:2.3}",
                final_adjacency.max_adjacent, final_adjacency.mean_adjacency
            ))
        });
    } else {
        pb.iter().for_each(|x| x.set_message("Could not cluster"));
    }

    pb.iter().for_each(|x| x.finish());
    results
}

fn binomial_coefficient(n: usize, k: usize) -> usize {
    let mut res = 1;
    for i in 0..k {
        res = (res * (n - i)) / (i + 1);
    }
    res
}

fn find_best_point_to_add<const W: usize>(
    chosen_points: &[BitSet<W>],
    all_points: &[BitSet<W>],
) -> BitSet<W> {
    let original_score = ClusterScoreBuilder::calculate(chosen_points);

    *all_points
        .iter()
        .filter(|p| !chosen_points.contains(p))
        .max_by_key(|point| {
            let builder = original_score.increment(chosen_points, point);
            let score: ClusterScore = builder.into();
            score
        })
        .unwrap()
}

fn find_best_pair<const W: usize>(all_points: &[BitSet<W>]) -> Option<(BitSet<W>, BitSet<W>)> {
    let mut min_intersection = all_points.first()?.count();
    let mut best = None;

    let ranges = segment_into_ranges(all_points);

    let combined_sizes = ranges
        .into_iter()
        .combinations_with_replacement(2)
        .map(|v| v.into_iter().next_tuple::<(_, _)>().unwrap())
        .sorted_by_key(|((a, _), (b, _))| std::cmp::Reverse(a + b))
        .collect_vec();

    for ((size_1, elements_1), (size_2, elements_2)) in combined_sizes {
        if size_1 == size_2 {
            for (set1, set2) in elements_1.iter().tuple_combinations() {
                let intersection = set1.intersect(set2).count();
                if intersection < min_intersection {
                    best = Some((set1.clone(), set2.clone()));
                    if intersection == 0 {
                        return best;
                    }
                    min_intersection = intersection;
                }
            }
        } else {
            for (set1, set2) in elements_1.iter().cartesian_product(elements_2.iter()) {
                let intersection = set1.intersect(set2).count();
                if intersection < min_intersection {
                    best = Some((set1.clone(), set2.clone()));
                    if intersection == 0 {
                        return best;
                    }
                    min_intersection = intersection;
                }
            }
        }
    }

    best
}

fn segment_into_ranges<const W: usize>(all_points: &[BitSet<W>]) -> Vec<(u32, &[BitSet<W>])> {
    let mut result = vec![];
    let Some(first_count) = all_points.first().map(|x| x.count()) else {
        return result;
    };
    let mut current_count = first_count;
    let mut remaining = all_points;

    while let Some((index, set)) = remaining
        .iter()
        .find_position(|x| x.count() != current_count)
    {
        let (current, remainder) = remaining.split_at(index);
        result.push((current_count, current));
        current_count = set.count();
        remaining = remainder;
    }
    result.push((current_count, remaining));

    result
}

#[cfg(test)]
pub mod test {
    use std::str::FromStr;

    use itertools::Itertools;
    use ws_core::{
        finder::{helpers::FinderSingleWord, node::GridResult},
        TileMap,
    };

    use super::cluster_words;

    #[test]
    pub fn test_clustering() {
        let months = include_str!("../grids/planets.txt");
        let word_vectors = get_words_vectors(months, 5);
        let all_words = word_vectors
            .iter()
            .flat_map(|x| x.iter())
            .cloned()
            .sorted()
            .dedup()
            .collect_vec();

        let grs = word_vectors
            .into_iter()
            .map(|words| GridResult {
                words,
                grid: TileMap::from_fn(|_| ws_core::Character::Blank),
                letters: Default::default(),
            })
            .collect_vec();

        let clusters = cluster_words::<1>(grs, &all_words, 10, None);

        assert_eq!(clusters.len(), 9);
        let text = clusters.into_iter().join("\n");
        insta::assert_snapshot!(text);
    }

    fn get_words_vectors(file: &str, min_words: usize) -> Vec<Vec<FinderSingleWord>> {
        file.lines()
            .map(|line| {
                let words = line
                    .split('\t')
                    .skip(2)
                    .map(|x| FinderSingleWord::from_str(x).unwrap())
                    .collect_vec();
                words
            })
            .filter(|x| x.len() >= min_words)
            .collect_vec()
    }
}
