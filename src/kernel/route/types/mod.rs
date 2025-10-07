use serde::{Deserialize, Serialize};
use std::borrow::Cow;

mod area;
mod connection;
mod entity;
mod merchant;

pub use area::Area;
pub use connection::Connection;
pub use entity::Entity;
pub use merchant::Merchant;
use crate::schema::connection::ConnectionType;

pub trait CloneIn<'a>: Sized {
    type Cloned;
    fn clone_in(&self, allocator: &'a bumpalo::Bump) -> Self::Cloned;
}

pub trait Dummy<'a>: Sized {
    fn dummy(allocator: &'a bumpalo::Bump) -> Self;
}

#[allow(unused)]
pub trait TakeIn<'a>: Dummy<'a> {
    fn take_in(&mut self, allocator: &'a bumpalo::Bump) -> Self {
        std::mem::replace(self, Self::dummy(allocator))
    }
}

pub trait FromIn<'a, T> {
    fn from_in(value: T, allocator: &'a bumpalo::Bump) -> Self;
}

#[allow(unused)]
pub trait IntoIn<'a, T> {
    fn into_in(self, allocator: &'a bumpalo::Bump) -> T;
}

#[derive(Eq, PartialEq, Ord, PartialOrd, Clone, Copy, Hash, Serialize, Deserialize)]
/// Ported from <https://github.com/oxc-project/oxc/blob/main/crates/oxc_span/src/atom.rs>
pub struct Atom<'a>(&'a str);

impl<'a> Atom<'a> {
    pub fn from(s: &'a str) -> Self {
        Self(s)
    }

    pub fn as_str(&self) -> &str {
        self.0
    }
}

impl<'a> Atom<'a> {
    pub fn new_in(allocator: &'a bumpalo::Bump) -> Self {
        let s = allocator.alloc_str("");
        Self(s)
    }
}

impl<'a, 'b> CloneIn<'b> for Atom<'a> {
    type Cloned = Atom<'b>;
    fn clone_in(&self, allocator: &'b bumpalo::Bump) -> Atom<'b> {
        let s = allocator.alloc_str(self.0);
        Atom(s)
    }
}

impl<'a> std::fmt::Debug for Atom<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "\"{}\"", self.0)
    }
}

impl<'a> std::ops::Deref for Atom<'a> {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        self.0
    }
}

impl<'a> std::fmt::Display for Atom<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl<'a> From<&'a str> for Atom<'a> {
    fn from(s: &'a str) -> Self {
        Self(s)
    }
}

impl<'a> FromIn<'a, String> for Atom<'a> {
    fn from_in(value: String, allocator: &'a bumpalo::Bump) -> Self {
        let s = allocator.alloc_str(&value);
        Self(s)
    }
}

impl<'a> FromIn<'a, Cow<'a, str>> for Atom<'a> {
    fn from_in(value: Cow<'a, str>, allocator: &'a bumpalo::Bump) -> Self {
        let s = allocator.alloc_str(&value);
        Self(s)
    }
}

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

pub type ConnectivityNode<'a> = (bumpalo::boxed::Box<'a, Area<'a>>, Atom<'a>, ConnectionType, f64, f64);
