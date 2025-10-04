use crate::kernel::route::utils::blocks::BoundedBlock;
use crate::schema::connection::ConnectionType;
use serde::{Deserialize, Serialize};
use std::fmt::Display;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum InstructionType {
    Move(f64, f64),
    Transport(String, ConnectionType),
}

impl From<(f64, f64)> for InstructionType {
    fn from((x, y): (f64, f64)) -> Self {
        InstructionType::Move(x, y)
    }
}

impl From<BoundedBlock> for InstructionType {
    fn from(block: BoundedBlock) -> Self {
        InstructionType::from(block.center())
    }
}

impl From<&BoundedBlock> for InstructionType {
    fn from(block: &BoundedBlock) -> Self {
        InstructionType::from(block.center())
    }
}

impl Display for InstructionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InstructionType::Move(from, to) => write!(f, "Move to ({}, {})", from, to),
            InstructionType::Transport(conn_id, conn_type) => {
                write!(f, "Take the {conn_type} (ID: {conn_id})")
            }
        }
    }
}

pub trait AsInstructions {
    fn as_instructions(&self) -> Vec<InstructionType>;
}

impl AsInstructions for Vec<BoundedBlock> {
    /// To convert the blocks into instructions, we need to do following things:
    /// 1. Find out block center.
    /// 2. For every block, find out if it requires you to walk straight, turn left, or turn right.
    fn as_instructions(&self) -> Vec<InstructionType> {
        self.to_vec().iter().map(Into::into).collect()
    }
}

#[cfg(test)]
mod tests {
    use crate::kernel::route::instructions::AsInstructions;
    use crate::kernel::route::utils::blocks::BoundedBlock;

    #[test]
    fn test_instruction_display() {
        use super::*;

        let move_instr = InstructionType::Move(1.0, 2.0);
        assert_eq!(format!("{}", move_instr), "Move to (1, 2)");

        let transport_instr =
            InstructionType::Transport("conn123".to_string(), ConnectionType::Elevator);
        assert_eq!(
            format!("{}", transport_instr),
            "Take the elevator (ID: conn123)"
        );
    }

    #[test]
    fn test_block_to_instruction() {
        let blocks = vec![
            BoundedBlock {
                x1: 0.0,
                y1: 0.0,
                x2: 1.0,
                y2: 1.0,
                is_bounded: true,
            },
            BoundedBlock {
                x1: 1.0,
                y1: 0.0,
                x2: 2.0,
                y2: 1.0,
                is_bounded: true,
            },
            BoundedBlock {
                x1: 2.0,
                y1: 0.0,
                x2: 3.0,
                y2: 1.0,
                is_bounded: true,
            },
        ];
        assert_eq!(blocks.as_instructions().len(), 3);
    }

    #[test]
    fn test_block_to_instruction_2() {
        let blocks = vec![
            BoundedBlock {
                x1: 0.0,
                y1: 0.0,
                x2: 1.0,
                y2: 1.0,
                is_bounded: true,
            },
            BoundedBlock {
                x1: 1.0,
                y1: 0.0,
                x2: 2.0,
                y2: 1.0,
                is_bounded: true,
            },
            BoundedBlock {
                x1: 2.0,
                y1: 0.0,
                x2: 3.0,
                y2: 1.0,
                is_bounded: true,
            },
            BoundedBlock {
                x1: 2.0,
                y1: 1.0,
                x2: 3.0,
                y2: 2.0,
                is_bounded: true,
            },
        ];
        println!("{:?}", blocks.as_instructions());
        assert_eq!(blocks.as_instructions().len(), 4);
    }
}
