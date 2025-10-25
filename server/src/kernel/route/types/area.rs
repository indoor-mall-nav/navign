use crate::kernel::route::implementations::Polygon;
use crate::kernel::route::types::{Atom, Connection, Merchant};
use crate::kernel::route::types::{CloneIn, Dummy, FromIn, IntoIn, TakeIn};
use crate::schema::Floor;
use bumpalo::{Bump, boxed::Box, collections::Vec};
use std::fmt::{Debug, Display, Formatter};

#[derive(Debug)]
pub struct Area<'a> {
    pub name: Atom<'a>,
    pub description: Option<Atom<'a>>,
    pub floor: Option<Floor>, // Floor number or name
    pub connections: Vec<'a, Box<'a, Connection<'a>>>,
    pub merchants: Vec<'a, Box<'a, Merchant<'a>>>,
    pub database_id: Atom<'a>,
    pub polygon: Polygon<'a>,
    phantom: std::marker::PhantomData<&'a ()>,
}

impl Display for Area<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Area {}", self.name)?;
        if let Some(floor) = &self.floor {
            write!(f, " ({floor})")?;
        }
        writeln!(f)?;
        writeln!(f, "  Connects:")?;
        for connectivity in &self.connections {
            writeln!(f, "  - {connectivity}")?;
        }
        writeln!(f, "  Merchants")?;
        for merchant in &self.merchants {
            writeln!(f, "  - {merchant}")?;
        }
        Ok(())
    }
}

impl<'a, 'b: 'a> CloneIn<'b> for Area<'a> {
    type Cloned = Area<'b>;
    fn clone_in(&self, allocator: &'b Bump) -> Area<'b> {
        Area {
            name: self.name.clone_in(allocator),
            description: self.description.as_ref().map(|d| d.clone_in(allocator)),
            floor: self.floor,
            connections: Vec::from_iter_in(
                self.connections
                    .iter()
                    .map(|c| Box::new_in(c.as_ref().clone_in(allocator), allocator)),
                allocator,
            ),
            merchants: Vec::from_iter_in(
                self.merchants
                    .iter()
                    .map(|m| Box::new_in(m.as_ref().clone_in(allocator), allocator)),
                allocator,
            ),
            database_id: self.database_id.clone_in(allocator),
            polygon: self.polygon.clone_in(allocator),
            phantom: std::marker::PhantomData,
        }
    }
}

impl<'a> Dummy<'a> for Area<'a> {
    fn dummy(allocator: &'a Bump) -> Self {
        Self {
            name: Atom::from("Dummy Area"),
            description: Some(Atom::from("This is a dummy area for testing.")),
            floor: None,
            connections: Vec::new_in(allocator),
            merchants: Vec::new_in(allocator),
            database_id: Atom::from_in(bson::oid::ObjectId::new().to_hex(), allocator),
            polygon: Polygon::default(),
            phantom: std::marker::PhantomData,
        }
    }
}

impl<'a> TakeIn<'a> for Area<'a> {}

impl<'a> FromIn<'a, crate::schema::Area> for Area<'a> {
    fn from_in(area: crate::schema::Area, allocator: &'a Bump) -> Self {
        Self {
            name: Atom::from_in(area.name.clone(), allocator),
            description: area
                .description
                .as_ref()
                .map(|d| Atom::from_in(d.clone(), allocator)),
            floor: area.floor,
            connections: Vec::new_in(allocator), // Connections need to be set up separately
            merchants: Vec::new_in(allocator),   // Merchants need to be set up separately
            database_id: Atom::from_in(area.id.to_hex(), allocator),
            polygon: Polygon::from(area.polygon),
            phantom: std::marker::PhantomData,
        }
    }
}

impl<'a> IntoIn<'a, crate::schema::Area> for Area<'a> {
    fn into_in(self, _allocator: &'a Bump) -> crate::schema::Area {
        // Warn: it's better to reread from database to avoid data loss
        crate::schema::Area {
            id: bson::oid::ObjectId::parse_str(self.database_id.as_str())
                .unwrap_or_else(|_| bson::oid::ObjectId::new()),
            name: self.name.to_string(),
            description: self.description.as_ref().map(|d| d.to_string()),
            floor: self.floor,
            ..Default::default()
        }
    }
}
