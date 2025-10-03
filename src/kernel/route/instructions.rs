use std::fmt::Display;
use serde::{Deserialize, Serialize};
use crate::kernel::route::utils::blocks::BoundedBlock;
use crate::kernel::route::utils::connectivity::ConnectivityPath;
use crate::schema::connection::ConnectionType;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum InstructionType {
    GoStraight(u64), // in meters
    Turn(i64),
    Transport(String, ConnectionType),
}

impl Display for InstructionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InstructionType::GoStraight(val) => write!(f, "Go {} meters", val),
            InstructionType::Turn(angle) => match angle {
                90 => write!(f, "Turn right"),
                -90 => write!(f, "Turn left"),
                180 | -180 => write!(f, "Go back"),
                30 | 45 | 60 => write!(f, "Turn slightly right"),
                -30 | -45 | -60 => write!(f, "Turn slightly left"),
                _ => write!(f, "Turn {:.2} degrees", angle),
            },
            InstructionType::Transport(conn_id, conn_type) => write!(f, "Take the {conn_type} (ID: {conn_id})"),
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
        let mut instructions = Vec::new();
        if self.is_empty() {
            return instructions;
        }

        // Start from the first block
        let first_block = &self[0];
        let (mut current_x, mut current_y) = first_block.center();
        let diff_meters = ((first_block.x2 - first_block.x1).powi(2) + (first_block.y2 - first_block.y1).powi(2)).sqrt();
        instructions.push(InstructionType::GoStraight(diff_meters as u64));

        // Initial direction is unknown, we will set it based on the first movement
        let mut current_direction: Option<f64> = None;

        for window in self.windows(2) {
            let (_, block_b) = (&window[0], &window[1]);
            let (next_x, next_y) = block_b.center();

            // Calculate the direction to the next block
            let direction_to_next = (next_y - current_y).atan2(next_x - current_x).to_degrees();

            if let Some(dir) = current_direction {
                // Calculate the angle difference
                let angle_diff = direction_to_next - dir;
                let angle_diff = if angle_diff > 180.0 {
                    angle_diff - 360.0
                } else if angle_diff < -180.0 {
                    angle_diff + 360.0
                } else {
                    angle_diff
                };

                // Determine turn instruction based on angle difference
                if angle_diff.abs() > 10.0 {
                    instructions.push(InstructionType::Turn(angle_diff as i64));
                }
            }

            // If the former time was go straight, we need to merge them
            if let Some(InstructionType::GoStraight(last_distance)) = instructions.last_mut() {
                let additional_distance = ((next_x - current_x).powi(2) + (next_y - current_y).powi(2)).sqrt() as u64;
                *last_distance += additional_distance;
            } else {
                let distance = ((next_x - current_x).powi(2) + (next_y - current_y).powi(2)).sqrt();
                instructions.push(InstructionType::GoStraight(distance as u64));
            }

            // Update current position and direction
            current_x = next_x;
            current_y = next_y;
            current_direction = Some(direction_to_next);
        }

        instructions
    }
}

#[cfg(test)]
mod tests {
    use crate::kernel::route::instructions::AsInstructions;
    use crate::kernel::route::utils::blocks::BoundedBlock;

    #[test]
    fn test_instruction_display() {
        use super::*;

        let move_instr = InstructionType::GoStraight(10);
        assert_eq!(format!("{}", move_instr), "Go 10 meters");

        let turn_instr = InstructionType::Turn(90);
        assert_eq!(format!("{}", turn_instr), "Turn right");

        let transport_instr = InstructionType::Transport("conn123".to_string(), ConnectionType::Elevator);
        assert_eq!(format!("{}", transport_instr), "Take the elevator (ID: conn123)");
    }

    #[test]
    fn test_block_to_instruction() {
        let blocks = vec![
            BoundedBlock { x1: 0.0, y1: 0.0, x2: 1.0, y2: 1.0, is_bounded: true },
            BoundedBlock { x1: 1.0, y1: 0.0, x2: 2.0, y2: 1.0, is_bounded: true },
            BoundedBlock { x1: 2.0, y1: 0.0, x2: 3.0, y2: 1.0, is_bounded: true },
        ];
        assert_eq!(blocks.as_instructions().len(), 1);
    }

    #[test]
    fn test_block_to_instruction_2() {
        let blocks = vec![
            BoundedBlock { x1: 0.0, y1: 0.0, x2: 1.0, y2: 1.0, is_bounded: true },
            BoundedBlock { x1: 1.0, y1: 0.0, x2: 2.0, y2: 1.0, is_bounded: true },
            BoundedBlock { x1: 2.0, y1: 0.0, x2: 3.0, y2: 1.0, is_bounded: true },
            BoundedBlock { x1: 2.0, y1: 1.0, x2: 3.0, y2: 2.0, is_bounded: true },
        ];
        println!("{:?}", blocks.as_instructions());
        assert_eq!(blocks.as_instructions().len(), 3);
    }
}