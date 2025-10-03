use crate::kernel::route::types::area::Area;
use crate::kernel::route::types::{Atom, CloneIn};
use crate::schema::connection::ConnectionType;
use bumpalo::boxed::Box;
use bumpalo::collections::Vec;
use bumpalo::Bump;
use std::collections::{BinaryHeap, HashMap, HashSet};
use crate::kernel::route::types::entity::Entity;

#[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq)]
pub struct ConnectivityLimits {
    pub elevator: bool,
    pub stairs: bool,
    pub escalator: bool,
}

impl Default for ConnectivityLimits {
    fn default() -> Self {
        Self {
            elevator: true,
            stairs: true,
            escalator: true,
        }
    }
}

pub type ConnectivityNode<'a> = (Box<'a, Area<'a>>, Atom<'a>, ConnectionType, f64, f64);

pub trait ConnectivityGraph<'a>: Sized {
    fn connectivity_graph(
        &self,
        alloc: &'a Bump,
        limits: ConnectivityLimits,
    ) -> Vec<'a, ConnectivityNode<'a>>;
}

impl<'a> ConnectivityGraph<'a> for Area<'a> {
    fn connectivity_graph(
        &self,
        alloc: &'a Bump,
        limits: ConnectivityLimits,
    ) -> Vec<'a, ConnectivityNode<'a>> {
        Vec::from_iter_in(
            self.connections
                .iter()
                .flat_map(|conn| {
                    Vec::from_iter_in(
                        conn.connected_area_from(self, alloc)
                            .into_iter()
                            .map(|(area, x, y)| (area, conn.database_id, *conn.r#type.as_ref(), x, y)),
                        alloc,
                    )
                })
                .filter(|(_, _, conn_type, _, _)| match conn_type {
                    ConnectionType::Elevator => limits.elevator,
                    ConnectionType::Escalator => limits.escalator,
                    ConnectionType::Stairs => limits.stairs,
                    _ => true,
                }),
            alloc,
        )
    }
}

/// Agent instance is like, suppose, you are going to a cinema and the cinema was regarded as an area,
/// but you can only access with the cinema by, like, entering the fourth floor of a building.
/// So you need to define an agent area that is the fourth floor of the building, and
/// the connectivity graph will be calculated from that area, not the whole building.
pub trait AgentInstance<'a>: Sized + ConnectivityGraph<'a> {
    fn agent_instance(&self, alloc: &'a Bump, limits: ConnectivityLimits) -> Option<(Box<'a, Self>, Atom<'a>)>;
}

impl<'a> AgentInstance<'a> for Area<'a> {
    fn agent_instance(&self, alloc: &'a Bump, limits: ConnectivityLimits) -> Option<(Box<'a, Self>, Atom<'a>)> {
        let graph = self.connectivity_graph(alloc, limits);
        // Only one node, which points to only one area (not self).
        if graph.is_empty() {
            return None;
        }

        if graph.len() == 1 {
            let (area, conn_id, _, _, _) = &graph[0];
            let area = area.as_ref();
            if area.database_id != self.database_id {
                return Some((Box::new_in(area.clone_in(alloc), alloc), *conn_id));
            }
        }

        None
    }
}

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

pub trait Contiguous<'a> {
    fn is_contiguous(
        &self,
        other: &Self,
        alloc: &'a Bump,
        limits: ConnectivityLimits,
    ) -> Option<Vec<'a, (Atom<'a>, Atom<'a>)>>;
}

impl<'a> Contiguous<'a> for Area<'a> {
    fn is_contiguous(
        &self,
        other: &Self,
        alloc: &'a Bump,
        limits: ConnectivityLimits,
    ) -> Option<Vec<'a, (Atom<'a>, Atom<'a>)>> {
        let start_id = self.database_id;
        let terminus_id = other.database_id;

        let start_neighbors = self.connectivity_graph(alloc, limits);
        for (neighbor, node, _, _, _) in start_neighbors.iter() {
            if neighbor.database_id == terminus_id {
                return Some(Vec::from_iter_in(vec![(start_id, Atom::new_in(&alloc)), (terminus_id, *node)], alloc));
            }
        }

        let start_agent = self.agent_instance(alloc, limits);
        let terminus_agent = other.agent_instance(alloc, limits);

        if let Some((start_agent, connectivity)) = start_agent.as_ref()
            && start_agent.database_id == terminus_id
        {
            return Some(Vec::from_iter_in(vec![(start_id, Atom::new_in(&alloc)), (terminus_id, *connectivity)], alloc));
        }

        if let Some((terminus_agent, connectivity)) = terminus_agent.as_ref()
            && terminus_agent.database_id == start_id
        {
            return Some(Vec::from_iter_in(vec![(start_id, Atom::new_in(&alloc)), (terminus_id, *connectivity)], alloc));
        }

        if let (Some((start_agent, start_connectivity)), Some((terminus_agent, terminus_connectivity))) =
            (start_agent.as_ref(), terminus_agent.as_ref())
            && start_agent.database_id == terminus_agent.database_id
        {
            let intermediate_id = start_agent.database_id.clone();
            return Some(Vec::from_iter_in(
                vec![(start_id, Atom::new_in(alloc)), (intermediate_id, *start_connectivity), (terminus_id, *terminus_connectivity)],
                alloc,
            ));
        }

        None
    }
}

fn reconstruct_path<'a>(
    came_from: &HashMap<Atom<'a>, (Atom<'a>, Atom<'a>)>,
    current: Atom<'a>,
    alloc: &'a Bump,
) -> Vec<'a, (Atom<'a>, Atom<'a>)> {
    let mut total_path = Vec::new_in(alloc);
    let mut total_connectivity = Vec::new_in(alloc);
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
    Vec::from_iter_in(
        total_path.into_iter().zip(total_connectivity),
        alloc,
    )
}

pub type ConnectivityPath<'a> = (Atom<'a>, Atom<'a>);
type ConnectivityRoute<'a> = Vec<'a, ConnectivityPath<'a>>;

pub trait ConnectWithInstance<'a>: Sized {
    fn get_areas(&self) -> &[Box<'a, Area<'a>>];

    fn find_path(
        &self,
        departure: Atom<'a>,
        departure_point: (f64, f64),
        arrival: Atom<'a>,
        limits: ConnectivityLimits,
        alloc: &'a Bump,
    ) -> Option<ConnectivityRoute<'a>> {
        println!("Finding path from {} to {}", departure, arrival);
        let departure_area = self
            .get_areas()
            .iter()
            .find(|area| area.database_id == departure)?;
        let arrival_area = self
            .get_areas()
            .iter()
            .find(|area| area.database_id == arrival)?;

        if departure_area.database_id == arrival_area.database_id {
            println!("Departure and arrival are in the same area.");
            return Some(Vec::from_iter_in(vec![(departure_area.database_id, Atom::new_in(alloc))], alloc));
        }

        if let Some(quick_path) = departure_area.is_contiguous(arrival_area, alloc, limits) {
            println!("Areas are contiguous.");
            return Some(quick_path);
        }

        let area_map = self
            .get_areas()
            .iter()
            .map(|area| (area.database_id, area.as_ref()))
            .collect::<HashMap<_, _>>();

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
                let neighbor_id = neighbor.database_id;
                if visited.contains(&neighbor_id) {
                    continue;
                }

                let neighbor_loc = (*conn_x, *conn_y);
                let edge_distance = manhattan_distance(x, y, neighbor_loc.0, neighbor_loc.1);
                let tentative_distance = current_distance + edge_distance as u64;

                if !distance_map.contains_key(&neighbor_id)
                    || tentative_distance < *distance_map.get(&neighbor_id).unwrap()
                {
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

        None
    }
}

impl<'a> ConnectWithInstance<'a> for Entity<'a> {
    fn get_areas(&self) -> &[Box<'a, Area<'a>>] {
        &self.areas
    }
}

#[cfg(test)]
mod tests {
    use crate::kernel::route::types::area::Area;
    use crate::kernel::route::types::{Dummy, FromIn};
    use crate::kernel::route::types::connection::Connection;
    use super::*;

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
        conn1.connected_areas.push(
            (Box::new_in(area1.clone_in(&alloc), &alloc), 0.0, 0.0)
        );
        conn1.connected_areas.push(
            (Box::new_in(area2.clone_in(&alloc), &alloc), 10.0, 0.0)
        );
        let mut conn2 = Connection::dummy(&alloc);
        conn2.database_id = Atom::from("conn2");
        conn2.connected_areas.push(
            (Box::new_in(area2.clone_in(&alloc), &alloc), 10.0, 0.0)
        );
        conn2.connected_areas.push(
            (Box::new_in(area3.clone_in(&alloc), &alloc), 10.0, 10.0)
        );
        conn2.connected_areas.push(
            (Box::new_in(area4.clone_in(&alloc), &alloc), 0.0, 10.0)
        );
        let mut conn3 = Connection::dummy(&alloc);
        conn3.database_id = Atom::from("conn3");
        conn3.connected_areas.push(
            (Box::new_in(area1.clone_in(&alloc), &alloc), 0.0, 0.0)
        );
        conn3.connected_areas.push(
            (Box::new_in(area4.clone_in(&alloc), &alloc), 0.0, 10.0)
        );
        area1.connections.push(Box::new_in(conn1.clone_in(&alloc), &alloc));
        area1.connections.push(Box::new_in(conn3.clone_in(&alloc), &alloc));
        area2.connections.push(Box::new_in(conn1.clone_in(&alloc), &alloc));
        area2.connections.push(Box::new_in(conn2.clone_in(&alloc), &alloc));
        let mut entity = Entity::dummy(&alloc);
        entity.areas.push(Box::new_in(area1.clone_in(&alloc), &alloc));
        entity.areas.push(Box::new_in(area2.clone_in(&alloc), &alloc));
        entity.areas.push(Box::new_in(area3.clone_in(&alloc), &alloc));
        entity.areas.push(Box::new_in(area4.clone_in(&alloc), &alloc));
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
        conn1.connected_areas.push(
            (Box::new_in(area1.clone_in(&alloc), &alloc), 0.0, 0.0)
        );
        conn1.connected_areas.push(
            (Box::new_in(area2.clone_in(&alloc), &alloc), 10.0, 0.0)
        );
        let mut conn2 = Connection::dummy(&alloc);
        conn2.database_id = Atom::from("conn2");
        conn2.connected_areas.push(
            (Box::new_in(area2.clone_in(&alloc), &alloc), 10.0, 0.0)
        );
        conn2.connected_areas.push(
            (Box::new_in(area3.clone_in(&alloc), &alloc), 10.0, 10.0)
        );
        conn2.connected_areas.push(
            (Box::new_in(area4.clone_in(&alloc), &alloc), 0.0, 10.0)
        );
        area1.connections.push(Box::new_in(conn1.clone_in(&alloc), &alloc));
        area2.connections.push(Box::new_in(conn1.clone_in(&alloc), &alloc));
        area2.connections.push(Box::new_in(conn2.clone_in(&alloc), &alloc));
        area3.connections.push(Box::new_in(conn2.clone_in(&alloc), &alloc));
        area4.connections.push(Box::new_in(conn2.clone_in(&alloc), &alloc));
        let mut entity = Entity::dummy(&alloc);
        entity.areas.push(Box::new_in(area1.clone_in(&alloc), &alloc));
        entity.areas.push(Box::new_in(area2.clone_in(&alloc), &alloc));
        entity.areas.push(Box::new_in(area3.clone_in(&alloc), &alloc));
        entity.areas.push(Box::new_in(area4.clone_in(&alloc), &alloc));
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
        conn1.connected_areas.push(
            (Box::new_in(area1.clone_in(&alloc), &alloc), 0.0, 0.0)
        );
        conn1.connected_areas.push(
            (Box::new_in(area2.clone_in(&alloc), &alloc), 10.0, 0.0)
        );
        let mut conn2 = Connection::dummy(&alloc);
        conn2.database_id = Atom::from("conn2");
        conn2.connected_areas.push(
            (Box::new_in(area2.clone_in(&alloc), &alloc), 10.0, 0.0)
        );
        conn2.connected_areas.push(
            (Box::new_in(area3.clone_in(&alloc), &alloc), 10.0, 10.0)
        );
        conn2.connected_areas.push(
            (Box::new_in(area4.clone_in(&alloc), &alloc), 0.0, 10.0)
        );
        let mut conn3 = Connection::dummy(&alloc);
        conn3.database_id = Atom::from("conn3");
        conn3.connected_areas.push(
            (Box::new_in(area1.clone_in(&alloc), &alloc), 0.0, 0.0)
        );
        conn3.connected_areas.push(
            (Box::new_in(area4.clone_in(&alloc), &alloc), 0.0, 10.0)
        );
        area1.connections.push(Box::new_in(conn1.clone_in(&alloc), &alloc));
        area1.connections.push(Box::new_in(conn3.clone_in(&alloc), &alloc));
        area2.connections.push(Box::new_in(conn1.clone_in(&alloc), &alloc));
        area2.connections.push(Box::new_in(conn2.clone_in(&alloc), &alloc));
        area3.connections.push(Box::new_in(conn2.clone_in(&alloc), &alloc));
        area4.connections.push(Box::new_in(conn2.clone_in(&alloc), &alloc));
        area4.connections.push(Box::new_in(conn3.clone_in(&alloc), &alloc));
        let mut entity = Entity::dummy(&alloc);
        entity.areas.push(Box::new_in(area1.clone_in(&alloc), &alloc));
        entity.areas.push(Box::new_in(area2.clone_in(&alloc), &alloc));
        entity.areas.push(Box::new_in(area3.clone_in(&alloc), &alloc));
        entity.areas.push(Box::new_in(area4.clone_in(&alloc), &alloc));
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
}