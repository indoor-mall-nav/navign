#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BoundedBlock {
    pub x1: f64,
    pub y1: f64,
    pub x2: f64,
    pub y2: f64,
    pub is_bounded: bool,
}

impl BoundedBlock {
    pub fn is_point_inside(&self, x: f64, y: f64) -> bool {
        x >= self.x1 && x <= self.x2 && y >= self.y1 && y <= self.y2
    }

    pub fn center(&self) -> (f64, f64) {
        ((self.x1 + self.x2) / 2.0, (self.y1 + self.y2) / 2.0)
    }
}
