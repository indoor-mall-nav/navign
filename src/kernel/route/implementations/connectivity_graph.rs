use bumpalo::Bump;
use crate::kernel::route::types::{Area, ConnectivityLimits, ConnectivityNode};
use crate::schema::connection::ConnectionType;

pub trait ConnectivityGraph<'a>: Sized {
    fn connectivity_graph(
        &self,
        alloc: &'a Bump,
        limits: ConnectivityLimits,
    ) -> bumpalo::collections::Vec<'a, ConnectivityNode<'a>>;
}

impl<'a> ConnectivityGraph<'a> for Area<'a> {
    fn connectivity_graph(
        &self,
        alloc: &'a Bump,
        limits: ConnectivityLimits,
    ) -> bumpalo::collections::Vec<'a, ConnectivityNode<'a>> {
        bumpalo::collections::Vec::from_iter_in(
            self.connections
                .iter()
                .flat_map(|conn| {
                    bumpalo::collections::Vec::from_iter_in(
                        conn.connected_area_from(self, alloc)
                            .into_iter()
                            .map(|(area, x, y)| {
                                (area, conn.database_id, *conn.r#type.as_ref(), x, y)
                            }),
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
