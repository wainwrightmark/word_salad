use std::collections::{BTreeMap, HashSet};

use itertools::Itertools;
use ws_core::{
    finder::{helpers::FinderWord, node::GridResult},
    CharsArray,
};
type WordsSet = geometrid::tile_set::TileSet128<128, 1, 128>;

#[derive(Debug, Clone)]
pub struct Cluster {
    pub grids: Vec<GridResult>,
    pub score: f32,
}

impl Cluster {
    pub fn new(points: &[WordsSet], map: &BTreeMap<WordsSet, GridResult>) -> Self {
        let distance: u32 = calculate_set_distance(points);

        let score = distance as f32 / (2.0 * (binomial_coefficient(points.len(), 2) as f32));

        let grids = points
            .iter()
            .flat_map(|p| map.get(p))
            .cloned()
            .collect_vec();

        Self { grids, score }
    }

    pub fn max_overlap(&self) -> usize {
        self.grids
            .iter()
            .tuple_combinations()
            .map(|(a, b)| a.words.iter().filter(|w| b.words.contains(w)).count())
            .max()
            .unwrap_or_default()
    }
}

impl std::fmt::Display for Cluster {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let grid_words = self.grids.iter().join("\n");
        let overlap = self.max_overlap();
        write!(
            f,
            "items: {l:2} dist: {average_distance:2.4} max overlap: {overlap:2} \n{grid_words}",
            l = self.grids.len(),
            average_distance = self.score,
        )
    }
}

pub fn cluster_words(
    groups: Vec<GridResult>,
    all_words: &Vec<FinderWord>,
    max_clusters: usize,
) -> Vec<Cluster> {
    let mut results: Vec<Cluster> = Default::default();
    if all_words.len() > 128 {
        println!("Clustering does not work with more than 128 words");
        return results;
    }
    let clusters = max_clusters.min(groups.len());
    let map: BTreeMap<WordsSet, GridResult> = groups
        .into_iter()
        .map(|grid_result| {
            let set: HashSet<CharsArray> =
                HashSet::from_iter(grid_result.words.iter().map(|x| x.array.clone()));

            let set: WordsSet = WordsSet::from_fn(|word_id| {
                let Some(word) = all_words.get(word_id.inner() as usize) else {
                    return false;
                };
                set.contains(&word.array)
            });
            (set, grid_result)
        })
        .collect();

    let all_points = map.keys().cloned().collect_vec();

    let Some((point1, point2)) = all_points
        .iter()
        .cloned()
        .tuple_combinations()
        .max_by_key(|(a, b)| calculate_distance(*a, *b))
    else {
        println!("Must be at least two groups to find clusters");
        return results;
    };

    let mut chosen_points: Vec<WordsSet> = vec![point1, point2];
    results.push(Cluster::new(&chosen_points, &map));

    while chosen_points.len() < clusters {
        let point_to_add = find_best_point_to_add(chosen_points.as_slice(), all_points.as_slice());
        chosen_points.push(point_to_add);
        //let mut swaps = 0;
        'inner: loop {
            let distance = calculate_set_distance(&chosen_points);
            let Some((removed_index, best_point, new_distance)) = (0..chosen_points.len())
                .map(|index| {
                    let mut new_chosen = chosen_points.clone();
                    let _removed = new_chosen.remove(index);
                    let best_point = find_best_point_to_add(&new_chosen, &all_points);
                    new_chosen.push(best_point);
                    let distance = calculate_set_distance(&new_chosen);

                    (index, best_point, distance)
                })
                .max_by_key(|(_, _, distance)| *distance)
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
        //println!("{swaps} swaps");
        //report_set(&chosen_points, &map);
        results.push(Cluster::new(&chosen_points, &map));
    }

    results
}

fn binomial_coefficient(n: usize, k: usize) -> usize {
    let mut res = 1;
    for i in 0..k {
        res = (res * (n - i)) / (i + 1);
    }
    res
}

fn calculate_distance(set1: WordsSet, set2: WordsSet) -> u32 {
    let unique_elements = std::ops::BitXor::bitxor(set1.into_inner(), set2.into_inner());
    unique_elements.count_ones()
}

fn calculate_set_distance(points: &[WordsSet]) -> u32 {
    points
        .iter()
        .tuple_combinations()
        .map(|(a, b)| calculate_distance(*a, *b))
        .sum()
}

fn find_best_point_to_add(chosen_points: &[WordsSet], all_points: &[WordsSet]) -> WordsSet {
    *all_points
        .iter()
        .filter(|p| !chosen_points.contains(p))
        .max_by_key(|point| {
            let sum: u32 = chosen_points
                .iter()
                .map(|cp| calculate_distance(**point, *cp))
                .sum();
            sum
        })
        .unwrap()
}

#[cfg(test)]
pub mod test {
    use itertools::Itertools;
    use ws_core::{
        finder::{helpers::FinderWord, node::GridResult},
        TileMap,
    };

    use super::cluster_words;

    #[test]
    pub fn test_clustering() {
        let months = include_str!("../grids/months.txt");
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

        let clusters = cluster_words(grs, &all_words, 10);

        assert_eq!(clusters.len(), 9);

        for cluster in clusters {
            println!("{cluster}");
        }
    }

    fn get_words_vectors(file: &str, min_words: usize) -> Vec<Vec<FinderWord>> {
        file.lines()
            .map(|line| {
                let words = line
                    .split('\t')
                    .skip(2)
                    .map(|x| FinderWord::try_new(x).unwrap())
                    .collect_vec();
                words
            })
            .filter(|x| x.len() >= min_words)
            .collect_vec()
    }
}

// fn calculate_total_cost(centroid1 : WordsSet, centroids: &[WordsSet], all_points: &[WordsSet]) -> u32 {
//     let total: u32 = all_points
//         .iter()
//         .map(|point| {
//             centroids
//                 .iter().chain([centroid1])
//                 .map(|centroid| calculate_distance(point, centroid))
//                 .min()
//                 .unwrap_or_default()
//         })
//         .sum();
//     total
// }

// fn find_new_centroid(centroids: &[WordsSet], all_points: &[WordsSet]) -> WordsSet {
//     all_points
//         .iter()
//         .filter(|point| !centroids.contains(point))
//         .min_by_key(|new_centroid| {

//             calculate_total_cost(new_centroid, centroids, all_points);
//         })
// }
