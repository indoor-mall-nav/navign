use crate::kernel::instruction::Instruction;
use crate::kernel::route::types::area::Area;
use bumpalo::collections::Vec;

pub enum InterareaRouteError {
    NoRouteFound,
    InvalidArea,
    AccessDenied,
    Other(String),
}

impl From<String> for InterareaRouteError {
    fn from(msg: String) -> Self {
        InterareaRouteError::Other(msg)
    }
}

impl From<InterareaRouteError> for anyhow::Error {
    fn from(err: InterareaRouteError) -> Self {
        match err {
            InterareaRouteError::NoRouteFound => anyhow::anyhow!("No route found"),
            InterareaRouteError::InvalidArea => anyhow::anyhow!("Invalid area"),
            InterareaRouteError::AccessDenied => anyhow::anyhow!("Access denied"),
            InterareaRouteError::Other(msg) => anyhow::anyhow!(msg),
        }
    }
}

#[derive(Debug, Clone)]
pub struct InterareaRouteConfig {
    pub allow_elevators: bool,
    /// Assumption I: If happen to a basement with parking lot, prefer elevator.
    pub basement_elevator: bool,
    /// Assumption II: If the route involves more than this number of escalators, prefer elevator.
    /// For example, you may want to take escalators if there are only 1 or 2 escalators,
    /// but if there are 3 or more escalators, you may want to take elevator instead.
    pub escalator_prefer_threshold: usize,
}

pub trait InterareaRoute<'a>: Sized {
    fn interarea_route(&self, target: Area<'a>) -> Result<Vec<'a, Instruction>, InterareaRouteError>;
}
