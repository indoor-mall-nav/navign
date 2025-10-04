//! TODO: Use universal Arena Bump to manage shared references, not Oxc allocator.

use crate::kernel::route::types::Atom;
use crate::kernel::route::types::area::Area;
use crate::kernel::route::types::connection::Connection;
use crate::kernel::route::types::merchant::Merchant;
use crate::kernel::route::types::{CloneIn, Dummy, FromIn, IntoIn, TakeIn};
use crate::schema::entity::EntityType;
use bson::oid::ObjectId;
use bumpalo::{Bump, boxed::Box, collections::Vec};
use futures::TryStreamExt;
use log::info;
use std::cell::RefCell;
use std::fmt::{Debug, Display, Formatter};
use std::ops::Deref;
use std::rc::Rc;

#[derive(Debug)]
pub struct Entity<'a> {
    pub r#type: EntityType,
    pub name: Atom<'a>,
    /// TODO: use HashMap instead.
    pub areas: Vec<'a, Box<'a, Area<'a>>>,
    pub(crate) database_id: Atom<'a>,
    phantom: std::marker::PhantomData<&'a ()>,
}

impl<'a> Display for Entity<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Entity {} ({})", self.name, self.r#type)?;
        for area in self.areas.iter() {
            writeln!(f, "- {}", area)?;
        }
        Ok(())
    }
}

impl<'a, 'b: 'a> CloneIn<'b> for Entity<'a> {
    type Cloned = Entity<'b>;
    fn clone_in(&self, allocator: &'b Bump) -> Entity<'b> {
        Entity {
            r#type: self.r#type.clone(),
            name: self.name.clone_in(allocator),
            areas: Vec::from_iter_in(
                self.areas
                    .iter()
                    .map(|area| Box::new_in(area.as_ref().clone_in(allocator), allocator)),
                allocator,
            ),
            database_id: self.database_id.clone_in(allocator),
            phantom: std::marker::PhantomData,
        }
    }
}

impl<'a> Dummy<'a> for Entity<'a> {
    fn dummy(allocator: &'a Bump) -> Self {
        Self {
            r#type: EntityType::Mall,
            name: Atom::from(""),
            areas: Vec::new_in(allocator),
            database_id: Atom::from_in(ObjectId::new().to_hex(), allocator),
            phantom: std::marker::PhantomData,
        }
    }
}

impl<'a> TakeIn<'a> for Entity<'a> {}

impl<'a> FromIn<'a, crate::schema::entity::Entity> for Entity<'a> {
    fn from_in(value: crate::schema::entity::Entity, allocator: &'a Bump) -> Self {
        Self {
            r#type: value.r#type,
            name: Atom::from_in(value.name, allocator),
            areas: Vec::new_in(allocator), // Areas should be loaded separately
            database_id: Atom::from_in(value.id.to_hex(), allocator),
            phantom: std::marker::PhantomData,
        }
    }
}

impl<'a> IntoIn<'a, crate::schema::entity::Entity> for Entity<'a> {
    fn into_in(self, _allocator: &'a Bump) -> crate::schema::entity::Entity {
        // Warn: it's better to reread from database to avoid data loss
        crate::schema::entity::Entity {
            id: ObjectId::parse_str(self.database_id.as_str()).unwrap_or_else(|_| ObjectId::new()),
            r#type: self.r#type,
            name: self.name.to_string(),
            ..Default::default()
        }
    }
}

impl<'a> Entity<'a> {
    pub fn convert_area_in(
        alloc: &'a Bump,
        entity: crate::schema::Entity,
        area_list: std::vec::Vec<crate::schema::Area>,
        connection_list: std::vec::Vec<crate::schema::Connection>,
        merchant_list: std::vec::Vec<crate::schema::Merchant>,
    ) -> Option<Entity<'a>> {
        if area_list.is_empty() || merchant_list.is_empty() || connection_list.is_empty() {
            return None;
        }
        let mut result = Entity::from_in(entity, alloc);
        info!("Converted entity to internal representation");
        let allocated_areas = Rc::new(RefCell::new(Vec::from_iter_in(
            area_list
                .into_iter()
                .map(|area| Box::new_in(Area::from_in(area, alloc), alloc)),
            alloc,
        )));
        info!("Converted areas to internal representation");
        let allocated_connections = Vec::from_iter_in(
            connection_list.into_iter().map(|conn| {
                info!("Processing connection id: {}", conn.id);
                let mut areas_map = Vec::new_in(alloc);
                for (connected_area_id, x, y) in conn.get_connected_areas().iter() {
                    info!("Processing connected area id: {}", connected_area_id);
                    if let Some(connected_area) = Rc::clone(&allocated_areas)
                        .borrow()
                        .iter()
                        .find(|a| a.database_id.as_str() == connected_area_id.to_hex().as_str())
                    {
                        let ptr = connected_area.deref() as *const Area;
                        // SAFETY: The pointer is valid as long as allocated_areas is alive
                        let area_ref = unsafe { &*ptr };
                        areas_map.push((Box::new_in(area_ref.clone_in(alloc), alloc), *x, *y));
                        info!("Connected area found and added: {}", connected_area_id);
                    }
                }
                let mut target = Box::new_in(Connection::from_in(conn, alloc), alloc);
                target.connected_areas = areas_map;
                target
            }),
            alloc,
        );
        println!("{:#?}", allocated_connections);
        for area in allocated_areas.borrow_mut().iter_mut() {
            info!("Processing area id: {}", area.database_id);
            let area_id = ObjectId::parse_str(area.database_id.as_str()).ok()?;
            let connections = Vec::from_iter_in(
                allocated_connections.iter().filter_map(|conn| {
                    info!("Checking connection id: {}", conn.database_id);
                    conn.connected_areas
                        .iter()
                        .any(|(a, _, _)| {
                            let a_id = ObjectId::parse_str(a.database_id.as_str()).ok();
                            info!("Comparing area ids: {} and {:?}", area_id, a_id);
                            a_id == Some(area_id)
                        })
                        .then(|| {
                            info!(
                                "Connection {} belongs to area {}",
                                conn.database_id, area.database_id
                            );
                            let ptr = conn.deref() as *const Connection;
                            // SAFETY: The pointer is valid as long as allocated_connections is alive
                            let conn_ref = unsafe { &*ptr };
                            Box::new_in(conn_ref.clone_in(alloc), alloc)
                        })
                }),
                alloc,
            );
            info!(
                "Area {} has {} connections: {:?}",
                area.database_id,
                connections.len(),
                connections
            );
            area.connections = connections;
            // Merchants are directly filtered from the original list
            let merchants = Vec::from_iter_in(
                merchant_list.iter().filter_map(|m| {
                    info!("Checking merchant id: {} with area id: {}", m.id, m.area);
                    if m.area == area_id {
                        info!("Merchant {} belongs to area {}", m.id, area.database_id);
                        Some(Box::new_in(Merchant::from_in(m.clone(), alloc), alloc))
                    } else {
                        None
                    }
                }),
                alloc,
            );
            info!("Area {} has merchants: {:?}", area.database_id, merchants);
            area.merchants = merchants;
            info!(
                "Area {} has {} connections and {} merchants",
                area.database_id,
                area.connections.len(),
                area.merchants.len()
            );
        }
        info!("Populated areas with connections and merchants");
        result.areas = Rc::try_unwrap(allocated_areas)
            .map_err(|_| ())
            .ok()?
            .into_inner();
        Some(result)
    }
}
