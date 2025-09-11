use crate::schema::connection::ConnectionType;
use std::fmt::Display;

#[derive(Debug, Clone, PartialEq)]
pub enum InstructionType {
    MoveToPoint {
        x: i32,
        y: i32,
    },
    MoveToMerchant {
        merchant_id: String,
    },
    Wait {
        duration: u32,
    },
    /// ObjectId of the Connection
    Transport {
        r#type: ConnectionType,
        connection: String,
    },
    EnterArea {
        area_id: String,
    },
    ExitArea {
        area_id: String,
    },
    Waypoint {
        x: i32,
        y: i32,
    },
}

impl Display for InstructionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InstructionType::MoveToPoint { x, y } => write!(f, "Move to point ({}, {})", x, y),
            InstructionType::MoveToMerchant { merchant_id } => {
                write!(f, "Move to merchant {}", merchant_id)
            }
            InstructionType::Wait { duration } => write!(f, "Wait for {} seconds", duration),
            InstructionType::Transport { r#type, connection } => write!(
                f,
                "Transport via {:?} (connection id: {})",
                r#type, connection
            ),
            InstructionType::EnterArea { area_id } => write!(f, "Enter area {}", area_id),
            InstructionType::ExitArea { area_id } => write!(f, "Exit area {}", area_id),
            InstructionType::Waypoint { x, y } => write!(f, "Waypoint at ({}, {})", x, y),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Instruction {
    pub instruction_type: InstructionType,
    pub description: Option<String>,
}

impl Instruction {
    pub fn new(instruction_type: InstructionType, description: Option<String>) -> Self {
        Self {
            instruction_type,
            description,
        }
    }
}
