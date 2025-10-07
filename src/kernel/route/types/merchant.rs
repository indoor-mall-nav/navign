use crate::kernel::route::types::{Atom, CloneIn, Dummy, FromIn, IntoIn, TakeIn};
use crate::schema::merchant::MerchantType;
use bumpalo::{Bump, boxed::Box};

#[derive(Debug)]
pub struct Merchant<'a> {
    pub name: Atom<'a>,
    pub coordinates: (f64, f64),
    pub r#type: Box<'a, MerchantType>,
    pub database_id: Atom<'a>,
    phantom: std::marker::PhantomData<&'a ()>,
}

impl<'a> std::fmt::Display for Merchant<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Merchant {} at ({}, {}) ({})",
            self.name, self.coordinates.0, self.coordinates.1, self.r#type,
        )
    }
}

impl<'a, 'b: 'a> CloneIn<'b> for Merchant<'a> {
    type Cloned = Merchant<'b>;
    fn clone_in(&self, allocator: &'b Bump) -> Merchant<'b> {
        Merchant {
            name: self.name.clone_in(allocator),
            coordinates: self.coordinates,
            r#type: Box::new_in(self.r#type.as_ref().clone(), allocator),
            database_id: self.database_id.clone_in(allocator),
            phantom: std::marker::PhantomData,
        }
    }
}

impl<'a> Dummy<'a> for Merchant<'a> {
    fn dummy(allocator: &'a Bump) -> Self {
        Self {
            name: Atom::from("Dummy Merchant"),
            coordinates: (0.0, 0.0),
            r#type: Box::new_in(MerchantType::Other, allocator),
            database_id: Atom::from_in(bson::oid::ObjectId::new().to_hex(), allocator),
            phantom: std::marker::PhantomData,
        }
    }
}

impl<'a> TakeIn<'a> for Merchant<'a> {}

impl<'a> FromIn<'a, crate::schema::merchant::Merchant> for Merchant<'a> {
    fn from_in(merchant: crate::schema::merchant::Merchant, allocator: &'a Bump) -> Self {
        Self {
            name: Atom::from_in(merchant.name.clone(), allocator),
            coordinates: merchant.location,
            r#type: Box::new_in(merchant.r#type.clone(), allocator),
            database_id: Atom::from_in(merchant.id.to_hex(), allocator),
            phantom: std::marker::PhantomData,
        }
    }
}

impl<'a> IntoIn<'a, crate::schema::merchant::Merchant> for Merchant<'a> {
    fn into_in(self, _allocator: &'a Bump) -> crate::schema::merchant::Merchant {
        // Warn: it's better to reread from database to avoid data loss
        crate::schema::merchant::Merchant {
            id: bson::oid::ObjectId::parse_str(self.database_id.as_str())
                .unwrap_or_else(|_| bson::oid::ObjectId::new()),
            name: self.name.to_string(),
            location: self.coordinates,
            r#type: (*self.r#type).clone(),
            description: None,
            area: bson::oid::ObjectId::new(),
            tags: vec![],
            ..Default::default()
        }
    }
}
