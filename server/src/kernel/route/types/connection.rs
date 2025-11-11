use crate::kernel::route::types::{Area, Atom, CloneIn, Dummy, FromIn, IntoIn};
use crate::schema::ConnectionType;
use bumpalo::{Bump, boxed::Box, collections::Vec};
use std::fmt::{Debug, Formatter};

#[derive(Debug)]
pub struct Connection<'a> {
    pub name: Atom<'a>,
    pub description: Option<Atom<'a>>,
    pub database_id: Atom<'a>,
    pub r#type: Box<'a, ConnectionType>,
    pub connected_areas: Vec<'a, (Box<'a, Area<'a>>, f64, f64)>, // (Area, x, y)
    pub available_hours: Option<Vec<'a, (i32, i32)>>, // (start_hour, end_hour) in 24-hour format
    phantom: std::marker::PhantomData<&'a ()>,
}

impl<'a> std::fmt::Display for Connection<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Connection {} connecting areas: [", self.name)?;
        for (area, x, y) in self.connected_areas.iter() {
            write!(f, "{} at ({}, {}), ", area.name, x, y)?;
        }
        write!(f, "] {}.", self.r#type)
    }
}

impl<'a> Connection<'a> {
    pub fn connected_area_from(
        &self,
        area: &Area<'a>,
        alloc: &'a Bump,
    ) -> Vec<'a, (Box<'a, Area<'a>>, f64, f64)> {
        let mut result = Vec::new_in(alloc);
        for (connected_area, x, y) in self.connected_areas.iter() {
            if connected_area.database_id != area.database_id {
                result.push((Box::new_in(connected_area.clone_in(alloc), alloc), *x, *y));
            }
        }
        result
    }
}

impl<'a, 'b: 'a> CloneIn<'b> for Connection<'a> {
    type Cloned = Connection<'b>;
    fn clone_in(&self, allocator: &'b Bump) -> Connection<'b> {
        Connection {
            name: self.name.clone_in(allocator),
            description: self.description.as_ref().map(|d| d.clone_in(allocator)),
            database_id: self.database_id.clone_in(allocator),
            r#type: Box::new_in(*self.r#type, allocator),
            connected_areas: Vec::from_iter_in(
                self.connected_areas.iter().map(|(area, x, y)| {
                    (
                        Box::new_in(area.as_ref().clone_in(allocator), allocator),
                        *x,
                        *y,
                    )
                }),
                allocator,
            ),
            available_hours: self
                .available_hours
                .as_ref()
                .map(|hours| Vec::from_iter_in(hours.iter().cloned(), allocator)),
            phantom: std::marker::PhantomData,
        }
    }
}

impl<'a> Dummy<'a> for Connection<'a> {
    fn dummy(allocator: &'a Bump) -> Self {
        Self {
            name: Atom::from("Dummy Connection"),
            description: Some(Atom::from("This is a dummy connection for testing.")),
            database_id: Atom::from_in(uuid::Uuid::new_v4().to_string(), allocator),
            r#type: Box::new_in(ConnectionType::Escalator, allocator),
            connected_areas: Vec::new_in(allocator),
            available_hours: None,
            phantom: std::marker::PhantomData,
        }
    }
}

impl<'a> FromIn<'a, crate::schema::Connection> for Connection<'a> {
    fn from_in(value: crate::schema::Connection, allocator: &'a Bump) -> Self {
        Self {
            name: Atom::from_in(value.name, allocator),
            description: value.description.map(|d| Atom::from_in(d, allocator)),
            database_id: Atom::from_in(value.id.to_string(), allocator),
            r#type: Box::new_in(value.r#type, allocator),
            connected_areas: Vec::new_in(allocator), // Needs to be populated separately
            available_hours: Some(Vec::from_iter_in(
                value.available_period.iter().cloned(),
                allocator,
            )),
            phantom: std::marker::PhantomData,
        }
    }
}

impl<'a> IntoIn<'a, crate::schema::Connection> for Connection<'a> {
    fn into_in(self, _allocator: &'a Bump) -> crate::schema::Connection {
        // Warn: it's better to reread from database to avoid data loss
        crate::schema::Connection {
            id: uuid::Uuid::parse_str(self.database_id.as_str())
                .unwrap_or_else(|_| uuid::Uuid::new_v4()),
            entity_id: uuid::Uuid::new_v4(), // Needs to be set properly
            name: self.name.to_string(),
            description: self.description.map(|d| d.to_string()),
            r#type: (*self.r#type),
            available_period: self
                .available_hours
                .map(|hours| hours.iter().cloned().collect())
                .unwrap_or_default(),
            ..Default::default()
        }
    }
}
