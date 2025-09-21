//! # Instructions Module
//!
//! The `instructions` module defines the data structures and enums used for representing route instructions in a navigation system. These instructions guide users through various actions such as entering merchant areas, going straight, turning, and arriving at destinations.
use bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use std::fmt::Display;

/// Enum representing the possible turn directions.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Copy)]
pub enum TurnTo {
    Left,
    Right,
    Back,
}

/// Enum representing the different types of route instructions.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RouteInstruction {
    /// Indicates entering a merchant area identified by its ObjectId.
    EnterMerchantArea(ObjectId),
    /// Indicates to go straight.
    GoStraight,
    /// Indicates a turn in a specified direction.
    Turn(TurnTo),
    /// Indicates entering a connection identified by its ObjectId.
    EnterConnection(ObjectId),
    /// Indicates exiting a connection identified by its ObjectId.
    ExitConnection(ObjectId),
    /// Indicates arrival at the destination.
    ArriveDestination,
}

impl Display for RouteInstruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RouteInstruction::EnterMerchantArea(id) => {
                write!(f, "Enter merchant area: {}", id)
            }
            RouteInstruction::GoStraight => write!(f, "Go straight"),
            RouteInstruction::Turn(direction) => match direction {
                TurnTo::Left => write!(f, "Turn left"),
                TurnTo::Right => write!(f, "Turn right"),
                TurnTo::Back => write!(f, "Turn back"),
            },
            RouteInstruction::EnterConnection(id) => {
                write!(f, "Enter connection: {}", id)
            }
            RouteInstruction::ExitConnection(id) => {
                write!(f, "Exit connection: {}", id)
            }
            RouteInstruction::ArriveDestination => write!(f, "Arrive at destination"),
        }
    }
}
