use bumpalo::Bump;
use crate::kernel::route::implementations::{AgentInstance, ConnectivityGraph};
use crate::kernel::route::types::{Area, Atom, ConnectivityLimits};

pub trait Contiguous<'a> {
    fn is_contiguous(
        &self,
        other: &Self,
        alloc: &'a Bump,
        limits: ConnectivityLimits,
    ) -> Option<bumpalo::collections::Vec<'a, (Atom<'a>, Atom<'a>)>>;
}

impl<'a> Contiguous<'a> for Area<'a> {
    fn is_contiguous(
        &self,
        other: &Self,
        alloc: &'a Bump,
        limits: ConnectivityLimits,
    ) -> Option<bumpalo::collections::Vec<'a, (Atom<'a>, Atom<'a>)>> {
        let start_id = self.database_id;
        let terminus_id = other.database_id;

        let start_neighbors = self.connectivity_graph(alloc, limits);
        for (neighbor, node, _, _, _) in start_neighbors.iter() {
            if neighbor.database_id == terminus_id {
                return Some(bumpalo::collections::Vec::from_iter_in(
                    vec![(start_id, Atom::new_in(alloc)), (terminus_id, *node)],
                    alloc,
                ));
            }
        }

        let start_agent = self.agent_instance(alloc, limits);
        let terminus_agent = other.agent_instance(alloc, limits);

        if let Some((start_agent, connectivity)) = start_agent.as_ref()
            && start_agent.database_id == terminus_id
        {
            return Some(bumpalo::collections::Vec::from_iter_in(
                vec![
                    (start_id, Atom::new_in(alloc)),
                    (terminus_id, *connectivity),
                ],
                alloc,
            ));
        }

        if let Some((terminus_agent, connectivity)) = terminus_agent.as_ref()
            && terminus_agent.database_id == start_id
        {
            return Some(bumpalo::collections::Vec::from_iter_in(
                vec![
                    (start_id, Atom::new_in(alloc)),
                    (terminus_id, *connectivity),
                ],
                alloc,
            ));
        }

        if let (
            Some((start_agent, start_connectivity)),
            Some((terminus_agent, terminus_connectivity)),
        ) = (start_agent.as_ref(), terminus_agent.as_ref())
            && start_agent.database_id == terminus_agent.database_id
        {
            let intermediate_id = start_agent.database_id;
            return Some(bumpalo::collections::Vec::from_iter_in(
                vec![
                    (start_id, Atom::new_in(alloc)),
                    (intermediate_id, *start_connectivity),
                    (terminus_id, *terminus_connectivity),
                ],
                alloc,
            ));
        }

        None
    }
}
