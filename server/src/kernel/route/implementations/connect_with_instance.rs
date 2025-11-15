use crate::kernel::route::{Area, Atom, ConnectivityLimits, Entity};
use crate::kernel::route::{ConnectivityGraph, Contiguous};
use bumpalo::Bump;
use tracing::trace;
use std::collections::{BinaryHeap, HashMap, HashSet};

/// Simplified priority node for pathfinding: lower distance = higher priority
#[derive(Debug, Clone)]
struct PathNode<'a> {
    area_id: Atom<'a>,
    distance: u64,
    x: f64,
    y: f64,
}

impl<'a> PartialEq for PathNode<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.distance == other.distance
    }
}

impl<'a> Eq for PathNode<'a> {}

impl<'a> PartialOrd for PathNode<'a> {
    #[allow(clippy::non_canonical_partial_ord_impl)]
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.distance.partial_cmp(&other.distance)?.reverse())
    }
}

impl<'a> Ord for PathNode<'a> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other)
            .unwrap_or(std::cmp::Ordering::Greater)
    }
}

/// Utils: Manhattan distance
fn manhattan_distance(x1: f64, y1: f64, x2: f64, y2: f64) -> f64 {
    (x1 - x2).abs() + (y1 - y2).abs()
}

fn reconstruct_path<'a>(
    came_from: &HashMap<Atom<'a>, (Atom<'a>, Atom<'a>)>,
    current: Atom<'a>,
    alloc: &'a Bump,
) -> bumpalo::collections::Vec<'a, (Atom<'a>, Atom<'a>)> {
    let mut total_path = bumpalo::collections::Vec::new_in(alloc);
    let mut total_connectivity = bumpalo::collections::Vec::new_in(alloc);
    total_path.push(current);
    let mut current = current;
    while let Some((prev, conn)) = came_from.get(&current) {
        total_path.push(*prev);
        total_connectivity.push(*conn);
        current = *prev;
    }
    total_connectivity.push(Atom::new_in(alloc)); // Starting point has no connectivity
    total_path.reverse();
    total_connectivity.reverse();
    bumpalo::collections::Vec::from_iter_in(total_path.into_iter().zip(total_connectivity), alloc)
}

pub type ConnectivityPath<'a> = (Atom<'a>, Atom<'a>);
type ConnectivityRoute<'a> = bumpalo::collections::Vec<'a, ConnectivityPath<'a>>;

pub trait ConnectWithInstance<'a>: Sized {
    fn get_areas(&self) -> &[bumpalo::boxed::Box<'a, Area<'a>>];

    fn find_path(
        &self,
        departure: Atom<'a>,
        departure_point: (f64, f64),
        arrival: Atom<'a>,
        limits: ConnectivityLimits,
        alloc: &'a Bump,
    ) -> Option<ConnectivityRoute<'a>> {
        trace!("Finding path from {} to {}", departure, arrival);
        let departure_area = self
            .get_areas()
            .iter()
            .find(|area| area.database_id == departure)?;
        let arrival_area = self
            .get_areas()
            .iter()
            .find(|area| area.database_id == arrival)?;

        if departure_area.database_id == arrival_area.database_id {
            trace!("Departure and arrival are in the same area.");
            return Some(bumpalo::collections::Vec::from_iter_in(
                vec![(departure_area.database_id, Atom::new_in(alloc))],
                alloc,
            ));
        }

        if let Some(quick_path) = departure_area.is_contiguous(arrival_area, alloc, limits) {
            trace!("Areas are contiguous.");
            return Some(quick_path);
        }

        trace!("Using Dijkstra's algorithm for pathfinding.");

        let area_map = self
            .get_areas()
            .iter()
            .map(|area| (area.database_id, area.as_ref()))
            .collect::<HashMap<_, _>>();

        trace!("Area map constructed with {} areas.", area_map.len());

        let mut heap = BinaryHeap::new();
        let mut visited = HashSet::new();
        let mut parent_map: HashMap<Atom<'a>, (Atom<'a>, Atom<'a>)> = HashMap::new();
        let mut distance_map = HashMap::new();

        heap.push(PathNode {
            area_id: departure_area.database_id,
            distance: 0,
            x: departure_point.0,
            y: departure_point.1,
        });
        distance_map.insert(departure_area.database_id, 0);

        while let Some(PathNode {
            area_id: current_area,
            distance: current_distance,
            x,
            y,
        }) = heap.pop()
        {
            if visited.contains(&current_area) {
                continue;
            }
            visited.insert(current_area);

            if current_area == arrival_area.database_id {
                return Some(reconstruct_path(&parent_map, current_area, alloc));
            }

            let current_area = match area_map.get(&current_area) {
                Some(area) => area,
                None => continue,
            };

            for (neighbor, connectivity, _, conn_x, conn_y) in
                current_area.connectivity_graph(alloc, limits).iter()
            {
                trace!("Exploring neighbor area: {}", neighbor.database_id);
                let neighbor_id = neighbor.database_id;
                if visited.contains(&neighbor_id) {
                    continue;
                }

                let neighbor_loc = (*conn_x, *conn_y);
                let edge_distance = manhattan_distance(x, y, neighbor_loc.0, neighbor_loc.1);
                let tentative_distance = current_distance + edge_distance as u64;

                let should_update = match distance_map.get(&neighbor_id) {
                    Some(&current_dist) => tentative_distance < current_dist,
                    None => true,
                };

                if should_update {
                    parent_map.insert(neighbor_id, (current_area.database_id, *connectivity));
                    distance_map.insert(neighbor_id, tentative_distance);
                    heap.push(PathNode {
                        area_id: neighbor_id,
                        distance: tentative_distance,
                        x: neighbor_loc.0,
                        y: neighbor_loc.1,
                    });
                }
            }
        }

        trace!("No path found.");

        None
    }
}

impl<'a> ConnectWithInstance<'a> for Entity<'a> {
    fn get_areas(&self) -> &[bumpalo::boxed::Box<'a, Area<'a>>] {
        &self.areas
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kernel::route::types::{Area, CloneIn, Connection, ConnectivityLimits, Dummy};
    use crate::schema::ConnectionType;
    use bumpalo::boxed::Box;

    #[test]
    fn contiguous_areas_no_agent() {
        let alloc = Bump::new();
        let mut area1 = Area::dummy(&alloc);
        area1.name = Atom::from("Area 1");
        area1.database_id = Atom::from("area1");
        let mut area2 = Area::dummy(&alloc);
        area2.name = Atom::from("Area 2");
        area2.database_id = Atom::from("area2");
        let mut area3 = Area::dummy(&alloc);
        area3.name = Atom::from("Area 3");
        area3.database_id = Atom::from("area3");
        let mut area4 = Area::dummy(&alloc);
        area4.name = Atom::from("Area 4");
        area4.database_id = Atom::from("area4");
        let mut conn1 = Connection::dummy(&alloc);
        conn1.database_id = Atom::from("conn1");
        conn1
            .connected_areas
            .push((Box::new_in(area1.clone_in(&alloc), &alloc), 0.0, 0.0));
        conn1
            .connected_areas
            .push((Box::new_in(area2.clone_in(&alloc), &alloc), 10.0, 0.0));
        let mut conn2 = Connection::dummy(&alloc);
        conn2.database_id = Atom::from("conn2");
        conn2
            .connected_areas
            .push((Box::new_in(area2.clone_in(&alloc), &alloc), 10.0, 0.0));
        conn2
            .connected_areas
            .push((Box::new_in(area3.clone_in(&alloc), &alloc), 10.0, 10.0));
        conn2
            .connected_areas
            .push((Box::new_in(area4.clone_in(&alloc), &alloc), 0.0, 10.0));
        let mut conn3 = Connection::dummy(&alloc);
        conn3.database_id = Atom::from("conn3");
        conn3
            .connected_areas
            .push((Box::new_in(area1.clone_in(&alloc), &alloc), 0.0, 0.0));
        conn3
            .connected_areas
            .push((Box::new_in(area4.clone_in(&alloc), &alloc), 0.0, 10.0));
        area1
            .connections
            .push(Box::new_in(conn1.clone_in(&alloc), &alloc));
        area1
            .connections
            .push(Box::new_in(conn3.clone_in(&alloc), &alloc));
        area2
            .connections
            .push(Box::new_in(conn1.clone_in(&alloc), &alloc));
        area2
            .connections
            .push(Box::new_in(conn2.clone_in(&alloc), &alloc));
        let mut entity = Entity::dummy(&alloc);
        entity
            .areas
            .push(Box::new_in(area1.clone_in(&alloc), &alloc));
        entity
            .areas
            .push(Box::new_in(area2.clone_in(&alloc), &alloc));
        entity
            .areas
            .push(Box::new_in(area3.clone_in(&alloc), &alloc));
        entity
            .areas
            .push(Box::new_in(area4.clone_in(&alloc), &alloc));
        let limits = ConnectivityLimits::default();
        let path = entity.find_path(
            Atom::from("area1"),
            (0.0, 0.0),
            Atom::from("area2"),
            limits,
            &alloc,
        );
        assert!(path.is_some());
        let path = path.unwrap();
        assert_eq!(path.len(), 2);
        assert_eq!(path[0].0, Atom::from("area1"));
        assert_eq!(path[1].0, Atom::from("area2"));
        assert_eq!(path[1].1, Atom::from("conn1"));
    }

    #[test]
    fn contiguous_areas_terminus_agent() {
        let alloc = Bump::new();
        let mut area1 = Area::dummy(&alloc);
        area1.name = Atom::from("Area 1");
        area1.database_id = Atom::from("area1");
        let mut area2 = Area::dummy(&alloc);
        area2.name = Atom::from("Area 2");
        area2.database_id = Atom::from("area2");
        let mut area3 = Area::dummy(&alloc);
        area3.name = Atom::from("Area 3");
        area3.database_id = Atom::from("area3");
        let mut area4 = Area::dummy(&alloc);
        area4.name = Atom::from("Area 4");
        area4.database_id = Atom::from("area4");
        let mut conn1 = Connection::dummy(&alloc);
        conn1.database_id = Atom::from("conn1");
        conn1
            .connected_areas
            .push((Box::new_in(area1.clone_in(&alloc), &alloc), 0.0, 0.0));
        conn1
            .connected_areas
            .push((Box::new_in(area2.clone_in(&alloc), &alloc), 10.0, 0.0));
        let mut conn2 = Connection::dummy(&alloc);
        conn2.database_id = Atom::from("conn2");
        conn2
            .connected_areas
            .push((Box::new_in(area2.clone_in(&alloc), &alloc), 10.0, 0.0));
        conn2
            .connected_areas
            .push((Box::new_in(area3.clone_in(&alloc), &alloc), 10.0, 10.0));
        conn2
            .connected_areas
            .push((Box::new_in(area4.clone_in(&alloc), &alloc), 0.0, 10.0));
        area1
            .connections
            .push(Box::new_in(conn1.clone_in(&alloc), &alloc));
        area2
            .connections
            .push(Box::new_in(conn1.clone_in(&alloc), &alloc));
        area2
            .connections
            .push(Box::new_in(conn2.clone_in(&alloc), &alloc));
        area3
            .connections
            .push(Box::new_in(conn2.clone_in(&alloc), &alloc));
        area4
            .connections
            .push(Box::new_in(conn2.clone_in(&alloc), &alloc));
        let mut entity = Entity::dummy(&alloc);
        entity
            .areas
            .push(Box::new_in(area1.clone_in(&alloc), &alloc));
        entity
            .areas
            .push(Box::new_in(area2.clone_in(&alloc), &alloc));
        entity
            .areas
            .push(Box::new_in(area3.clone_in(&alloc), &alloc));
        entity
            .areas
            .push(Box::new_in(area4.clone_in(&alloc), &alloc));
        let limits = ConnectivityLimits::default();
        let path = entity.find_path(
            Atom::from("area3"),
            (0.0, 0.0),
            Atom::from("area1"),
            limits,
            &alloc,
        );
        assert!(path.is_some());
        let path = path.unwrap();
        assert_eq!(path.len(), 3);
        assert_eq!(path[0].0, Atom::from("area3"));
        assert_eq!(path[1].0, Atom::from("area2"));
        assert_eq!(path[2].0, Atom::from("area1"));
        assert_eq!(path[1].1, Atom::from("conn2"));
        assert_eq!(path[2].1, Atom::from("conn1"));
    }

    #[test]
    fn dijkstra() {
        let alloc = Bump::new();
        let mut area1 = Area::dummy(&alloc);
        area1.name = Atom::from("Area 1");
        area1.database_id = Atom::from("area1");
        let mut area2 = Area::dummy(&alloc);
        area2.name = Atom::from("Area 2");
        area2.database_id = Atom::from("area2");
        let mut area3 = Area::dummy(&alloc);
        area3.name = Atom::from("Area 3");
        area3.database_id = Atom::from("area3");
        let mut area4 = Area::dummy(&alloc);
        area4.name = Atom::from("Area 4");
        area4.database_id = Atom::from("area4");
        let mut conn1 = Connection::dummy(&alloc);
        conn1.database_id = Atom::from("conn1");
        conn1
            .connected_areas
            .push((Box::new_in(area1.clone_in(&alloc), &alloc), 0.0, 0.0));
        conn1
            .connected_areas
            .push((Box::new_in(area2.clone_in(&alloc), &alloc), 10.0, 0.0));
        let mut conn2 = Connection::dummy(&alloc);
        conn2.database_id = Atom::from("conn2");
        conn2
            .connected_areas
            .push((Box::new_in(area2.clone_in(&alloc), &alloc), 10.0, 0.0));
        conn2
            .connected_areas
            .push((Box::new_in(area3.clone_in(&alloc), &alloc), 10.0, 10.0));
        conn2
            .connected_areas
            .push((Box::new_in(area4.clone_in(&alloc), &alloc), 0.0, 10.0));
        let mut conn3 = Connection::dummy(&alloc);
        conn3.database_id = Atom::from("conn3");
        conn3
            .connected_areas
            .push((Box::new_in(area1.clone_in(&alloc), &alloc), 0.0, 0.0));
        conn3
            .connected_areas
            .push((Box::new_in(area4.clone_in(&alloc), &alloc), 0.0, 10.0));
        area1
            .connections
            .push(Box::new_in(conn1.clone_in(&alloc), &alloc));
        area1
            .connections
            .push(Box::new_in(conn3.clone_in(&alloc), &alloc));
        area2
            .connections
            .push(Box::new_in(conn1.clone_in(&alloc), &alloc));
        area2
            .connections
            .push(Box::new_in(conn2.clone_in(&alloc), &alloc));
        area3
            .connections
            .push(Box::new_in(conn2.clone_in(&alloc), &alloc));
        area4
            .connections
            .push(Box::new_in(conn2.clone_in(&alloc), &alloc));
        area4
            .connections
            .push(Box::new_in(conn3.clone_in(&alloc), &alloc));
        let mut entity = Entity::dummy(&alloc);
        entity
            .areas
            .push(Box::new_in(area1.clone_in(&alloc), &alloc));
        entity
            .areas
            .push(Box::new_in(area2.clone_in(&alloc), &alloc));
        entity
            .areas
            .push(Box::new_in(area3.clone_in(&alloc), &alloc));
        entity
            .areas
            .push(Box::new_in(area4.clone_in(&alloc), &alloc));
        let limits = ConnectivityLimits::default();
        let path = entity.find_path(
            Atom::from("area3"),
            (0.0, 0.0),
            Atom::from("area1"),
            limits,
            &alloc,
        );
        assert!(path.is_some());
        let path = path.unwrap();
        assert_eq!(path.len(), 3);
        assert_eq!(path[0].0, Atom::from("area3"));
        assert_eq!(path[1].0, Atom::from("area2"));
        assert_eq!(path[2].0, Atom::from("area1"));
        assert_eq!(path[1].1, Atom::from("conn2"));
        assert_eq!(path[2].1, Atom::from("conn1"));
    }

    #[test]
    fn school() {
        let alloc = Bump::default();
        let ent = Entity::dummy(&alloc);
        let limits = ConnectivityLimits::default();
        let mut f2 = Area::dummy(&alloc);
        f2.name = Atom::from("2nd Floor");
        f2.database_id = Atom::from("f2");
        let mut f3 = Area::dummy(&alloc);
        f3.name = Atom::from("3rd Floor");
        f3.database_id = Atom::from("f3");
        let mut f4 = Area::dummy(&alloc);
        f4.name = Atom::from("4th Floor");
        f4.database_id = Atom::from("f4");
        let mut st1 = Connection::dummy(&alloc);
        st1.r#type = Box::new_in(ConnectionType::Stairs, &alloc);
        st1.database_id = Atom::from("st1");
        st1.connected_areas
            .push((Box::new_in(f2.clone_in(&alloc), &alloc), 0.0, 0.0));
        st1.connected_areas
            .push((Box::new_in(f3.clone_in(&alloc), &alloc), 0.0, 0.0));
        let mut st2 = Connection::dummy(&alloc);
        st2.r#type = Box::new_in(ConnectionType::Stairs, &alloc);
        st2.database_id = Atom::from("st2");
        st2.connected_areas
            .push((Box::new_in(f2.clone_in(&alloc), &alloc), 0.0, 0.0));
        st2.connected_areas
            .push((Box::new_in(f3.clone_in(&alloc), &alloc), 0.0, 0.0));
        let mut st3 = Connection::dummy(&alloc);
        st3.r#type = Box::new_in(ConnectionType::Stairs, &alloc);
        st3.database_id = Atom::from("st3");
        st3.connected_areas
            .push((Box::new_in(f3.clone_in(&alloc), &alloc), 0.0, 0.0));
        st3.connected_areas
            .push((Box::new_in(f4.clone_in(&alloc), &alloc), 0.0, 0.0));
        f2.connections
            .push(Box::new_in(st1.clone_in(&alloc), &alloc));
        f2.connections
            .push(Box::new_in(st2.clone_in(&alloc), &alloc));
        f3.connections
            .push(Box::new_in(st1.clone_in(&alloc), &alloc));
        f3.connections
            .push(Box::new_in(st2.clone_in(&alloc), &alloc));
        f3.connections
            .push(Box::new_in(st3.clone_in(&alloc), &alloc));
        f4.connections
            .push(Box::new_in(st3.clone_in(&alloc), &alloc));
        let mut ent = ent;
        ent.name = Atom::from("School");
        ent.areas.push(Box::new_in(f2.clone_in(&alloc), &alloc));
        ent.areas.push(Box::new_in(f3.clone_in(&alloc), &alloc));
        ent.areas.push(Box::new_in(f4.clone_in(&alloc), &alloc));
        trace!("{}", ent);
        let path = ent.find_path(
            Atom::from("f2"),
            (0.0, 0.0),
            Atom::from("f4"),
            limits,
            &alloc,
        );
        assert!(path.is_some());
        let path = path.unwrap();
        assert_eq!(path.len(), 3);
        assert_eq!(path[0].0, Atom::from("f2"));
        assert_eq!(path[1].0, Atom::from("f3"));
        assert_eq!(path[2].0, Atom::from("f4"));
    }
}
