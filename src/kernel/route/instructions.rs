use std::fmt::Display;
use serde::{Deserialize, Serialize};
use crate::schema::connection::ConnectionType;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum InstructionType {
    MoveTo(f64, f64),
    Turn(f64),
    Transport(ConnectionType),
}

impl Display for InstructionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InstructionType::MoveTo(x, y) => write!(f, "Move to ({}, {})", x, y),
            InstructionType::Turn(angle) => match angle {
                90.0 => write!(f, "Turn right"),
                -90.0 => write!(f, "Turn left"),
                180.0 | -180.0 => write!(f, "Go back"),
                30.0 | 45.0 | 60.0 => write!(f, "Turn slightly right"),
                -30.0 | -45.0 | -60.0 => write!(f, "Turn slightly left"),
                _ => write!(f, "Turn {:.2} degrees", angle),
            },
            InstructionType::Transport(conn_type) => write!(f, "Take the {}", conn_type),
        }
    }
}

impl From<(f64, f64)> for InstructionType {
    fn from(value: (f64, f64)) -> Self {
        InstructionType::MoveTo(value.0, value.1)
    }
}

impl From<f64> for InstructionType {
    fn from(value: f64) -> Self {
        InstructionType::Turn(value)
    }
}

impl From<ConnectionType> for InstructionType {
    fn from(value: ConnectionType) -> Self {
        InstructionType::Transport(value)
    }
}