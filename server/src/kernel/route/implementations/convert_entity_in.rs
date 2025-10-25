use crate::kernel::route::{Area, CloneIn, Connection, Entity, FromIn, Merchant};
use bson::oid::ObjectId;
use bumpalo::Bump;
use log::trace;
use std::cell::RefCell;
use std::ops::Deref;
use std::rc::Rc;

pub trait ConvertEntityIn<'a, T> {
    fn convert_entity_in(
        allocator: &'a Bump,
        entity: crate::schema::Entity,
        area_list: Vec<crate::schema::Area>,
        connection_list: Vec<crate::schema::Connection>,
        merchant_list: Vec<crate::schema::Merchant>,
    ) -> Option<T>;
}

impl<'a> ConvertEntityIn<'a, Entity<'a>> for Entity<'a> {
    fn convert_entity_in(
        alloc: &'a Bump,
        entity: crate::schema::Entity,
        area_list: Vec<crate::schema::Area>,
        connection_list: Vec<crate::schema::Connection>,
        merchant_list: Vec<crate::schema::Merchant>,
    ) -> Option<Entity<'a>> {
        if area_list.is_empty() || merchant_list.is_empty() || connection_list.is_empty() {
            return None;
        }
        let mut result = Entity::from_in(entity, alloc);
        trace!("Converted entity to internal representation");
        let allocated_areas = Rc::new(RefCell::new(bumpalo::collections::Vec::from_iter_in(
            area_list
                .into_iter()
                .map(|area| bumpalo::boxed::Box::new_in(Area::from_in(area, alloc), alloc)),
            alloc,
        )));
        trace!("Converted areas to internal representation");
        let allocated_connections = bumpalo::collections::Vec::from_iter_in(
            connection_list.into_iter().map(|conn| {
                trace!("Processing connection id: {}", conn.id);
                let mut areas_map = bumpalo::collections::Vec::new_in(alloc);
                for (connected_area_id, x, y, open) in conn.get_connected_areas().iter() {
                    trace!("Processing connected area id: {}", connected_area_id);
                    if let Some(connected_area) =
                        Rc::clone(&allocated_areas).borrow().iter().find(|a| {
                            a.database_id.as_str() == connected_area_id.to_hex().as_str() && *open
                        })
                    {
                        let ptr = connected_area.deref() as *const Area;
                        // SAFETY: The pointer is valid as long as allocated_areas is alive
                        let area_ref = unsafe { &*ptr };
                        areas_map.push((
                            bumpalo::boxed::Box::new_in(area_ref.clone_in(alloc), alloc),
                            *x,
                            *y,
                        ));
                        trace!("Connected area found and added: {}", connected_area_id);
                    }
                }
                let mut target =
                    bumpalo::boxed::Box::new_in(Connection::from_in(conn, alloc), alloc);
                target.connected_areas = areas_map;
                target
            }),
            alloc,
        );
        for area in allocated_areas.borrow_mut().iter_mut() {
            trace!("Processing area id: {}", area.database_id);
            let area_id = ObjectId::parse_str(area.database_id.as_str()).ok()?;
            let connections = bumpalo::collections::Vec::from_iter_in(
                allocated_connections.iter().filter_map(|conn| {
                    trace!("Checking connection id: {}", conn.database_id);
                    conn.connected_areas
                        .iter()
                        .any(|(a, _, _)| {
                            let a_id = ObjectId::parse_str(a.database_id.as_str()).ok();
                            trace!("Comparing area ids: {} and {:?}", area_id, a_id);
                            a_id == Some(area_id)
                        })
                        .then(|| {
                            trace!(
                                "Connection {} belongs to area {}",
                                conn.database_id, area.database_id
                            );
                            let ptr = conn.deref() as *const Connection;
                            // SAFETY: The pointer is valid as long as allocated_connections is alive
                            let conn_ref = unsafe { &*ptr };
                            bumpalo::boxed::Box::new_in(conn_ref.clone_in(alloc), alloc)
                        })
                }),
                alloc,
            );
            trace!(
                "Area {} has {} connections: {:?}",
                area.database_id,
                connections.len(),
                connections
            );
            area.connections = connections;
            // Merchants are directly filtered from the original list
            let merchants = bumpalo::collections::Vec::from_iter_in(
                merchant_list.iter().filter_map(|m| {
                    trace!("Checking merchant id: {} with area id: {}", m.id, m.area);
                    if m.area == area_id {
                        trace!("Merchant {} belongs to area {}", m.id, area.database_id);
                        Some(bumpalo::boxed::Box::new_in(
                            Merchant::from_in(m.clone(), alloc),
                            alloc,
                        ))
                    } else {
                        None
                    }
                }),
                alloc,
            );
            trace!("Area {} has merchants: {:?}", area.database_id, merchants);
            area.merchants = merchants;
            trace!(
                "Area {} has {} connections and {} merchants",
                area.database_id,
                area.connections.len(),
                area.merchants.len()
            );
        }
        trace!("Populated areas with connections and merchants");
        result.areas = Rc::try_unwrap(allocated_areas)
            .map_err(|_| ())
            .ok()?
            .into_inner();
        Some(result)
    }
}
