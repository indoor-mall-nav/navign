use bumpalo::Bump;
use crate::kernel::route::implementations::connectivity_graph::ConnectivityGraph;
use crate::kernel::route::types::{Area, Atom, CloneIn, ConnectivityLimits};

/// Agent instance is like, suppose, you are going to a cinema and the cinema was regarded as an area,
/// but you can only access with the cinema by, like, entering the fourth floor of a building.
/// So you need to define an agent area that is the fourth floor of the building, and
/// the connectivity graph will be calculated from that area, not the whole building.
pub trait AgentInstance<'a>: Sized + ConnectivityGraph<'a> {
    fn agent_instance(
        &self,
        alloc: &'a Bump,
        limits: ConnectivityLimits,
    ) -> Option<(bumpalo::boxed::Box<'a, Self>, Atom<'a>)>;
}

impl<'a> AgentInstance<'a> for Area<'a> {
    fn agent_instance(
        &self,
        alloc: &'a Bump,
        limits: ConnectivityLimits,
    ) -> Option<(bumpalo::boxed::Box<'a, Self>, Atom<'a>)> {
        let graph = self.connectivity_graph(alloc, limits);
        // Only one node, which points to only one area (not self).
        if graph.is_empty() {
            return None;
        }

        if graph.len() == 1 {
            let (area, conn_id, _, _, _) = &graph[0];
            let area = area.as_ref();
            if area.database_id != self.database_id {
                return Some((bumpalo::boxed::Box::new_in(area.clone_in(alloc), alloc), *conn_id));
            }
        }

        None
    }
}
