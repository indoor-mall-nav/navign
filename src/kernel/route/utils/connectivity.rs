use crate::kernel::route::types::area::Area;
use crate::schema::connection::ConnectionType;
use bumpalo::boxed::Box;
use bumpalo::collections::Vec;
use bumpalo::Bump;

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
    ) -> Vec<'a, (Box<'a, Area<'a>>, ConnectionType, f64, f64)>;
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
