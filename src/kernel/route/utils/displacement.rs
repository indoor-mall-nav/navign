use crate::kernel::route::utils::blocks::{BoundedBlock, BoundedBlockArray, ContiguousBlockArray};
use std::collections::{BinaryHeap, HashMap};

pub trait DisplacementRoute<'a, T: Sized + Clone + Copy>: ContiguousBlockArray<T> {
    type Node: Sized + Clone + Copy + Ord + PartialOrd + Eq + PartialEq;
    fn find_displacement(&self, departure: (f64, f64), arrival: (f64, f64)) -> Option<Vec<T>>;
}

struct Utils;

impl Utils {
    fn manhattan(a: (f64, f64), b: (f64, f64)) -> f64 {
        (a.0 - b.0).abs() + (a.1 - b.1).abs()
    }

    fn reconstruct_path(came_from: &HashMap<usize, usize>, current: usize) -> Vec<usize> {
        let mut total_path = vec![current];
        let mut current = current;
        while let Some(&prev) = came_from.get(&current) {
            total_path.push(prev);
            current = prev;
        }
        total_path.reverse();
        total_path
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PathNode {
    index: usize,
    f_score: f64,
    g_score: f64,
}

impl Eq for PathNode {}

impl Ord for PathNode {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other
            .f_score
            .partial_cmp(&self.f_score)
            .unwrap_or(std::cmp::Ordering::Equal)
    }
}

impl PartialOrd for PathNode {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<'a> DisplacementRoute<'a, BoundedBlock> for BoundedBlockArray<'a> {
    type Node = PathNode;
    fn find_displacement(
        &self,
        departure: (f64, f64),
        arrival: (f64, f64),
    ) -> Option<Vec<BoundedBlock>> {
        let (departure_x, departure_y) = departure;
        let (arrival_x, arrival_y) = arrival;

        let departure_block = self.fit(departure_x, departure_y)?;
        let arrival_block = self.fit(arrival_x, arrival_y)?;

        let departure_index = self.get_index(departure_block)?;
        let arrival_index = self.get_index(arrival_block)?;

        if departure_index == arrival_index {
            return Some(vec![departure_block]);
        }

        let h_score = Utils::manhattan(departure_block.center(), arrival_block.center());

        let mut open_set = BinaryHeap::new();
        let mut came_from: HashMap<usize, usize> = HashMap::new();
        let mut g_score: HashMap<usize, f64> = HashMap::new();

        g_score.insert(departure_index, 0.0);
        open_set.push(PathNode {
            index: departure_index,
            f_score: h_score,
            g_score: 0.0,
        });

        while let Some(curr_node) = open_set.pop() {
            let current_index = curr_node.index;

            if current_index == arrival_index {
                let path_indices = Utils::reconstruct_path(&came_from, current_index);
                let path_blocks: Vec<BoundedBlock> =
                    path_indices.iter().map(|&idx| self[idx]).collect();
                return Some(path_blocks);
            }

            let current_block = self[current_index];
            let (cx, cy) = self.deaccess(current_block)?;

            for neighbor in self.contiguous_access_matrix(cx, cy) {
                if !neighbor.is_bounded {
                    continue;
                }

                let neighbor_index = self.get_index(neighbor)?;

                let current_center = current_block.center();
                let neighbor_center = neighbor.center();

                let travel_distance = Utils::manhattan(current_center, neighbor_center);
                let tentative_g_score =
                    g_score.get(&current_index).unwrap_or(&f64::INFINITY) + travel_distance;
                if tentative_g_score < *g_score.get(&neighbor_index).unwrap_or(&f64::INFINITY) {
                    came_from.insert(neighbor_index, current_index);
                    g_score.insert(neighbor_index, tentative_g_score);
                    let f_score = tentative_g_score
                        + Utils::manhattan(neighbor.center(), arrival_block.center());
                    open_set.push(PathNode {
                        index: neighbor_index,
                        f_score,
                        g_score: tentative_g_score,
                    });
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kernel::route::utils::blocks::Polygon;

    #[test]
    fn find_easy() {
        let polygon = &[
            (0.0, 0.0),
            (0.0, 2.0),
            (1.0, 2.0),
            (1.0, 1.0),
            (2.0, 1.0),
            (2.0, 3.0),
            (3.0, 3.0),
            (3.0, 0.0),
            (0.0, 0.0),
        ];
        let poly = Polygon::from(polygon.as_slice());

        let arr = poly.as_bounded_block_array();

        let res = arr.find_displacement((0.0, 1.5), (2.5, 2.5));

        assert!(res.is_some());
        let results = res.unwrap();
        assert_eq!(results.len(), 6);
        assert_eq!(results[0].center().0, 0.5);
        assert_eq!(results[0].center().1, 1.5);
        assert_eq!(results[1].center().0, 0.5);
        assert_eq!(results[1].center().1, 0.5);
        assert_eq!(results[2].center().0, 1.5);
        assert_eq!(results[2].center().1, 0.5);
        assert_eq!(results[3].center().0, 2.5);
        assert_eq!(results[3].center().1, 0.5);
        assert_eq!(results[4].center().0, 2.5);
        assert_eq!(results[4].center().1, 1.5);
        assert_eq!(results[5].center().0, 2.5);
        assert_eq!(results[5].center().1, 2.5);
    }
    #[test]
    fn test2() {
        let polygon = [
            (0.0, 60.0),
            (0.0, 75.0),
            (5.0, 75.0),
            (5.0, 70.0),
            (45.0, 70.0),
            (45.0, 72.0),
            (48.0, 72.0),
            (48.0, 66.0),
            (5.0, 66.0),
            (5.0, 60.0),
        ];
        let poly = Polygon::from(polygon.as_slice());
        let array = poly.as_bounded_block_array();
        let res = array.find_displacement((40.0, 66.0), (2.0, 64.0));
        assert!(res.is_some());
    }
}
