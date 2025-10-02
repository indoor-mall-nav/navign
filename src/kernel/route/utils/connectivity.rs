use std::collections::HashMap;
use crate::kernel::route::types::area::Area;
use crate::schema::connection::ConnectionType;
use bumpalo::boxed::Box;
use bumpalo::collections::Vec;
use bumpalo::Bump;
use crate::kernel::route::types::{Atom, CloneIn};

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

pub trait ConnectivityGraph<'a>: Sized {
    fn connectivity_graph(
        &self,
        alloc: &'a Bump,
        limits: ConnectivityLimits,
    ) -> Vec<'a, (Box<'a, Self>, ConnectionType, f64, f64)>;
}

impl<'a> ConnectivityGraph<'a> for Area<'a> {
    fn connectivity_graph(
        &self,
        alloc: &'a Bump,
        limits: ConnectivityLimits,
    ) -> Vec<'a, (Box<'a, Area<'a>>, ConnectionType, f64, f64)> {
        Vec::from_iter_in(
            self.connections.iter().flat_map(|conn| {
                Vec::from_iter_in(
                    conn.connected_area_from(self, alloc)
                        .into_iter()
                        .map(|(area, x, y)| (area, *conn.r#type.as_ref(), x, y)),
                    alloc,
                )
            }).filter(|(_, conn_type, _, _)| {
                match conn_type {
                    ConnectionType::Elevator => limits.elevator,
                    ConnectionType::Escalator => limits.escalator,
                    ConnectionType::Stairs => limits.stairs,
                    _ => true
                }
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
    fn agent_instance(&self, alloc: &'a Bump, limits: ConnectivityLimits) -> Option<Box<'a, Self>>;
}

impl<'a> AgentInstance<'a> for Area<'a> {
    fn agent_instance(&self, alloc: &'a Bump, limits: ConnectivityLimits) -> Option<Box<'a, Self>> {
        let graph = self.connectivity_graph(alloc, limits);
        // All nodes point to only one area (not self); can have multiple elements in `graph`, but its area is the same.
        if graph.is_empty() {
            return None;
        }
        // Get areas accessible
        let areas_set = Vec::from_iter_in(
            graph.iter().map(|(area, _, _, _)| area.database_id.to_string()).collect::<std::collections::HashSet<_>>(),
            alloc,
        );
        if areas_set.len() != 1 || areas_set[0] == self.database_id.to_string() {
            return None;
        }
        Some(Box::new_in(
            graph.first().unwrap().0.as_ref().clone_in(alloc),
            alloc,
        ))
    }
}

pub trait ConnectWithInstance<'a>: Sized + ConnectivityGraph<'a> + AgentInstance<'a> {
    fn get_identifier(&self) -> Atom<'a>;

    fn reconstruct_path(
        &self,
        parent_map: HashMap<Atom<'a>, Atom<'a>>,
        start_id: Atom<'a>,
        end_id: Atom<'a>,
    ) -> std::vec::Vec<Atom<'a>> {
        let mut path = std::vec::Vec::new();
        let mut current_id = end_id;

        path.push(current_id);

        while current_id != start_id {
            if let Some(parent_id) = parent_map.get(&current_id) {
                path.push(*parent_id);
                current_id = *parent_id;
            } else {
                // No path found
                return vec![];
            }
        }

        path.reverse();
        path
    }

    fn find_path(&'a self, terminus: &'a Self, alloc: &'a Bump) -> Option<std::vec::Vec<Atom<'a>>> {
        let start_id = self.get_identifier();
        let terminus_id = terminus.get_identifier();

        if start_id == terminus_id {
            return Some(vec![start_id]);
        }

        let mut queue = std::collections::VecDeque::new();
        let mut visited = std::collections::HashSet::new();
        let mut parent_map = HashMap::new();

        queue.push_back(start_id);
        visited.insert(start_id);

        while let Some(current_id) = queue.pop_front() {
            let current_area = if current_id == self.get_identifier() {
                self
            } else if current_id == terminus.get_identifier() {
                terminus
            } else {
                continue; // In a full implementation, you'd look up the area by ID
            };

            for (neighbor_area, _, _, _) in current_area.connectivity_graph(alloc, ConnectivityLimits::default()) {
                let neighbor_id = neighbor_area.get_identifier();
                if !visited.contains(&neighbor_id) {
                    visited.insert(neighbor_id);
                    parent_map.insert(neighbor_id, current_id);
                    queue.push_back(neighbor_id);

                    if neighbor_id == terminus_id {
                        return Some(self.reconstruct_path(parent_map, start_id, terminus_id));
                    }
                }
            }
        }

        None // No path found
    }
}

impl<'a> ConnectWithInstance<'a> for Area<'a> {
    fn get_identifier(&self) -> Atom<'a> {
        self.database_id.clone()
    }
}
