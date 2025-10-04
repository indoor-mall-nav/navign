use crate::kernel::route::instructions::{AsInstructions, InstructionType};
use crate::kernel::route::types::Atom;
use crate::kernel::route::types::entity::Entity;
use crate::kernel::route::utils::connectivity::{ConnectWithInstance, ConnectivityLimits};
use crate::kernel::route::utils::displacement::DisplacementRoute;
use bumpalo::Bump;
use log::info;
use serde::{Deserialize, Serialize};
use std::error::Error;

pub mod blocks;
pub mod connectivity;
pub mod displacement;

#[derive(Debug, Clone, PartialEq)]
pub enum NavigationError {
    NoRoute,
    InvalidDeparture,
    InvalidArrival,
    AccessDenied,
    Other(String),
}

impl Error for NavigationError {}

impl std::fmt::Display for NavigationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s: String = self.clone().into();
        write!(f, "{}", s)
    }
}

impl From<NavigationError> for String {
    fn from(err: NavigationError) -> Self {
        match err {
            NavigationError::NoRoute => "No route found".to_string(),
            NavigationError::InvalidDeparture => "Invalid departure point".to_string(),
            NavigationError::InvalidArrival => "Invalid arrival point".to_string(),
            NavigationError::AccessDenied => "Access denied".to_string(),
            NavigationError::Other(msg) => msg,
        }
    }
}

impl Serialize for NavigationError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let s: String = self.clone().into();
        serializer.serialize_str(&s)
    }
}

impl<'de> Deserialize<'de> for NavigationError {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            "No route found" => Ok(NavigationError::NoRoute),
            "Invalid departure point" => Ok(NavigationError::InvalidDeparture),
            "Invalid arrival point" => Ok(NavigationError::InvalidArrival),
            "Access denied" => Ok(NavigationError::AccessDenied),
            other => Ok(NavigationError::Other(other.to_string())),
        }
    }
}

type NavigationResult = Result<Vec<InstructionType>, NavigationError>;
type Location<'a> = (f64, f64, Atom<'a>);

pub trait Navigate<'a> {
    fn navigate(
        &self,
        departure: Location<'a>,
        arrival: Location<'a>,
        limits: ConnectivityLimits,
        allocator: &'a Bump,
    ) -> NavigationResult;
}

impl<'a> Navigate<'a> for Entity<'a> {
    fn navigate(
        &self,
        departure: Location<'a>,
        arrival: Location<'a>,
        limits: ConnectivityLimits,
        allocator: &'a Bump,
    ) -> NavigationResult {
        if departure.2 == arrival.2 {
            info!("Finding path inside area {}", departure.2);
            info!("From ({}, {}) to ({}, {})", departure.0, departure.1, arrival.0, arrival.1);
            let area = match self
                .get_areas()
                .iter()
                .find(|area| area.database_id == departure.2)
            {
                Some(area) => area,
                None => return Err(NavigationError::InvalidDeparture),
            };
            let route = match area
                .polygon
                .as_bounded_block_array()
                .find_displacement((departure.0, departure.1), (arrival.0, arrival.1))
            {
                Some(route) => route,
                None => return Err(NavigationError::NoRoute),
            };
            let instructions = route.as_instructions();
            Ok(instructions)
        } else {
            info!(
                "Finding path from area {} to area {}",
                departure.2, arrival.2
            );
            let mut instructions = bumpalo::collections::Vec::new_in(allocator);
            let connectivity = match self.find_path(
                departure.2,
                (departure.0, departure.1),
                arrival.2,
                limits,
                allocator,
            ) {
                Some(conn) => conn,
                None => return Err(NavigationError::NoRoute),
            };
            info!("Connectivity Path: {connectivity:?}");
            let conns = connectivity.into_iter().collect::<Vec<_>>();
            let (mut target_x, mut target_y) = (departure.0, departure.1);
            for window in conns.windows(2) {
                let [(area_src_id, _), (area_dest_id, conn_dest_id)] = window else {
                    return Err(NavigationError::Other("Unexpected window size".to_string()));
                };
                println!(
                    "From area {area_src_id} via connection {conn_dest_id} to area {area_dest_id}"
                );
                let area_src = match self
                    .get_areas()
                    .iter()
                    .find(|area| area.database_id == *area_src_id)
                {
                    Some(area) => area,
                    None => return Err(NavigationError::InvalidDeparture),
                };
                let conn = match area_src
                    .connections
                    .iter()
                    .find(|conn| conn.database_id == *conn_dest_id)
                {
                    Some(conn) => conn,
                    None => {
                        return Err(NavigationError::Other(
                            "Connection not found in source area".to_string(),
                        ));
                    }
                };
                let (_, x, y) = match conn
                    .connected_areas
                    .iter()
                    .find(|(a, _, _)| a.database_id == *area_src_id)
                {
                    Some((a, x, y)) => (a, *x, *y),
                    None => {
                        return Err(NavigationError::Other(
                            "Connected area not found in connection".to_string(),
                        ));
                    }
                };
                println!(
                    "  Walking inside area {area_src_id} from ({target_x}, {target_y}) to ({x}, {y})"
                );
                let inner_route = area_src
                    .polygon
                    .as_bounded_block_array()
                    .find_displacement((target_x, target_y), (x, y));
                println!("  Inner route: {inner_route:?}");
                (_, target_x, target_y) = match conn
                    .connected_areas
                    .iter()
                    .find(|(a, _, _)| a.database_id == *area_dest_id)
                {
                    Some((a, x, y)) => (a, *x, *y),
                    None => {
                        return Err(NavigationError::Other(
                            "Connected area not found in connection".to_string(),
                        ));
                    }
                };
                if let Some(inner_route) = inner_route {
                    println!(
                        "  Inner route instructions: {:?}",
                        inner_route.as_instructions()
                    );
                    instructions.extend(inner_route.as_instructions());
                } else {
                    return Err(NavigationError::NoRoute);
                }
                instructions.push(InstructionType::Transport(
                    conn.database_id.to_string(),
                    *conn.r#type,
                ));
            }
            let area_dest = match self
                .get_areas()
                .iter()
                .find(|area| area.database_id == arrival.2)
            {
                Some(area) => area,
                None => return Err(NavigationError::InvalidArrival),
            };
            println!(
                "Final walk inside area {}, from ({target_x}, {target_y}) to ({}, {})",
                arrival.2, arrival.0, arrival.1
            );
            let final_route = area_dest
                .polygon
                .as_bounded_block_array()
                .find_displacement((target_x, target_y), (arrival.0, arrival.1));
            println!("Final route: {final_route:?}");
            if let Some(final_route) = final_route {
                instructions.extend(final_route.as_instructions());
            } else {
                return Err(NavigationError::NoRoute);
            }
            Ok(instructions.into_iter().collect())
        }
    }
}
