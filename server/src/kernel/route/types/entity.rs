use crate::kernel::route::types::{Area, Atom, CloneIn, Dummy, FromIn, IntoIn, TakeIn};
use crate::schema::EntityType;
use bson::oid::ObjectId;
use bumpalo::{Bump, boxed::Box, collections::Vec};
use std::fmt::{Debug, Display, Formatter};

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

impl<'a> FromIn<'a, crate::schema::Entity> for Entity<'a> {
    fn from_in(value: crate::schema::Entity, allocator: &'a Bump) -> Self {
        Self {
            r#type: value.r#type,
            name: Atom::from_in(value.name, allocator),
            areas: Vec::new_in(allocator), // Areas should be loaded separately
            database_id: Atom::from_in(value.id.to_hex(), allocator),
            phantom: std::marker::PhantomData,
        }
    }
}

impl<'a> IntoIn<'a, crate::schema::Entity> for Entity<'a> {
    fn into_in(self, _allocator: &'a Bump) -> crate::schema::Entity {
        // Warn: it's better to reread from database to avoid data loss
        crate::schema::Entity {
            id: ObjectId::parse_str(self.database_id.as_str()).unwrap_or_else(|_| ObjectId::new()),
            r#type: self.r#type,
            name: self.name.to_string(),
            ..Default::default()
        }
    }
}
