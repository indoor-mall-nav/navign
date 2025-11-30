#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SetVelocity {
    pub vx: f32,
    pub vy: f32,
    pub omega: f32,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SetWeels {
    pub fl: f32,
    pub fr: f32,
    pub bl: f32,
    pub br: f32,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum MotionCommand {
    SetVelocity(SetVelocity),
    SetWheels(SetWeels),
    Stop,
    EmergencyStop,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Odometry {
    pub x: f32,
    pub y: f32,
    pub theta: f32,

    pub vx: f32,
    pub vy: f32,
    pub omega: f32,
}
