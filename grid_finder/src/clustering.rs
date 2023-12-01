use std::{collections::{BTreeMap, HashSet}, cmp::Reverse};

use itertools::Itertools;
use ws_core::{
    finder::{helpers::FinderWord, node::GridResult},
    CharsArray,
};
type WordsSet = geometrid::tile_set::TileSet128<128, 1, 128>;


#[derive(Debug, Clone, Copy, Eq, PartialEq, PartialOrd, Ord)]
pub struct ClusterScore{
    /// The maximum overlap between two elements.
    /// Lower is better
    pub max_overlap: std::cmp::Reverse<u32>,
    /// The total overlap between all pairs of elements.
    /// Lower is better
    pub total_overlap: std::cmp::Reverse<u32>,
    /// The total sum of the item count of all elements
    /// Higher is better
    pub total_element_items: u32
}

impl ClusterScore
{

    pub fn increment(&self, original_cluster: &[WordsSet], new_item: &WordsSet)-> Self{
        let mut result = self.clone();
        result.total_element_items += new_item.count();

        for item in original_cluster{
            let overlap = item.intersect(new_item).count();
            result.max_overlap = result.max_overlap.max(Reverse(overlap));
            result.total_overlap = Reverse(result.total_overlap.0 + overlap);
        }

        result
    }


    pub fn calculate<'a>(cluster: &'a [WordsSet])-> Self{

        let total_element_items = cluster.iter().map(|x|x.count()).sum();
        let mut max_overlap = 0;
        let mut total_overlap = 0;

        for (a,b) in cluster.iter().tuple_combinations(){
            let overlap = a.intersect(&b).count();
            total_overlap += overlap;
            max_overlap = max_overlap.max(overlap);
        }

        ClusterScore { max_overlap: Reverse(max_overlap), total_overlap : Reverse(total_overlap), total_element_items }
    }
}

impl<'a> From<&'a  [WordsSet]> for ClusterScore{
    fn from(cluster: &'a [WordsSet]) -> Self {
        let total_element_items = cluster.iter().map(|x|x.count()).sum();
        let mut max_overlap = 0;
        let mut total_overlap = 0;

        for (a,b) in cluster.iter().tuple_combinations(){
            let overlap = a.intersect(b).count();
            total_overlap += overlap;
            max_overlap = max_overlap.max(overlap);
        }

        ClusterScore { max_overlap: Reverse(max_overlap), total_overlap : Reverse(total_overlap), total_element_items }
    }
}


#[derive(Debug, Clone)]
pub struct Cluster {
    pub grids: Vec<GridResult>,
    pub score: ClusterScore
    //pub score: f32,
}

impl Cluster {
    pub fn new(points: &[WordsSet], map: &BTreeMap<WordsSet, GridResult>) -> Self {
        //let distance: u32 = calculate_set_distance(points);

        let score = ClusterScore::calculate(points);

        let grids = points
            .iter()
            .flat_map(|p| map.get(p))
            .cloned()
            .collect_vec();

        Self { grids, score }
    }


}

impl std::fmt::Display for Cluster {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let grid_words = self.grids.iter().join("\n");


        let average_overlap = self.score.total_overlap.0 as f32 / (2.0 * (binomial_coefficient(self.grids.len(), 2) as f32));
        let distinct_items = self.grids.iter().flat_map(|x|x.words.iter()).sorted().dedup().count();

        write!(
            f,
            "Grids: {l:2}\tMax overlap: {max_overlap:2}\tMean overlap: {average_overlap:2.2}\tMean items: {mean_items:2.2}\tDistinct items: {distinct_items:3}\n{grid_words}",
            l = self.grids.len(),
            max_overlap = self.score.max_overlap.0,
            mean_items = self.score.total_element_items as f32 / self.grids.len() as f32
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
        .max_by_key(|(a, b)|ClusterScore::calculate(&[*a,*b]))
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
            let distance = ClusterScore::calculate(chosen_points.as_slice());
            let Some((removed_index, best_point, new_distance)) = (0..chosen_points.len())
                .map(|index| {
                    let mut new_chosen = chosen_points.clone();
                    let _removed = new_chosen.remove(index);
                    let best_point = find_best_point_to_add(&new_chosen, &all_points);
                    new_chosen.push(best_point);
                    let distance = ClusterScore::calculate(&new_chosen);

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

// fn calculate_distance(set1: WordsSet, set2: WordsSet) -> u32 {
//     let unique_elements = std::ops::BitXor::bitxor(set1.into_inner(), set2.into_inner());
//     unique_elements.count_ones()
// }

// fn calculate_set_distance(points: &[WordsSet]) -> u32 {
//     points
//         .iter()
//         .tuple_combinations()
//         .map(|(a, b)| calculate_distance(*a, *b))
//         .sum()
// }

fn find_best_point_to_add(chosen_points: &[WordsSet], all_points: &[WordsSet]) -> WordsSet {
    let original_score = ClusterScore::calculate(chosen_points);

    *all_points
        .iter()
        .filter(|p| !chosen_points.contains(p))
        .max_by_key(|point| {
            original_score.increment(chosen_points, &point)
        })
        .unwrap()
}

#[cfg(test)]
pub mod test {
    use std::str::FromStr;

    use itertools::Itertools;
    use ws_core::{
        finder::{helpers::FinderWord, node::GridResult},
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
                    .map(|x| FinderWord::from_str(x).unwrap())
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
