use std::fmt::{Display, Formatter};
use serde::{Deserialize, Serialize};
use crate::schema::polygon::node::Node;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LineTo {
    terminus: Node,
    /// TikZ style options for the line `in`
    r#in: Option<f64>,
    /// TikZ style options for the line `out`
    out: Option<f64>,
}

impl Display for LineTo {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if let (Some(r#in), Some(out)) = (self.r#in, self.out) {
            write!(f, "to[in={:.2}, out={:.2}] {}", r#in, out, self.terminus)
        } else if let Some(r#in) = self.r#in {
            write!(f, "to[in={:.2}] {}", r#in, self.terminus)
        } else if let Some(out) = self.out {
            write!(f, "to[out={:.2}] {}", out, self.terminus)
        } else {
            write!(f, "-- {}", self.terminus)
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LineArc {
    radius: f64,
    start_angle: f64,
    end_angle: f64,
}

impl Display for LineArc {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "arc ({:.2}:{:.2}:{:.2})",
            self.radius, self.start_angle, self.end_angle
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Line {
    To(LineTo),
    Arc(LineArc),
}

impl Display for Line {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Line::To(line_to) => write!(f, "{}", line_to),
            Line::Arc(line_arc) => write!(f, "{}", line_arc),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Path {
    start: Node,
    lines: Vec<Line>,
}

impl Display for Path {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "\\draw ")?;
        write!(f, "{}", self.start)?;
        for line in &self.lines {
            write!(f, " {}", line)?;
        }
        write!(f, ";")?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::schema::polygon::node::XYNode;
    use super::*;
    #[test]
    fn test_line_to_display() {
        let line_to = LineTo {
            terminus: Node::XY(XYNode { x: 3.0, y: 4.0 }),
            r#in: Some(45.0),
            out: Some(30.0),
        };
        assert_eq!(line_to.to_string(), "to[in=45.00, out=30.00] (3.00,4.00)");

        let line_to_no_in = LineTo {
            terminus: Node::XY(XYNode { x: 1.0, y: 2.0 }),
            r#in: None,
            out: Some(60.0),
        };
        assert_eq!(line_to_no_in.to_string(), "to[out=60.00] (1.00,2.00)");

        let line_to_no_out = LineTo {
            terminus: Node::XY(XYNode { x: 5.0, y: 6.0 }),
            r#in: Some(90.0),
            out: None,
        };
        assert_eq!(line_to_no_out.to_string(), "to[in=90.00] (5.00,6.00)");

        let line_to_no_in_out = LineTo {
            terminus: Node::XY(XYNode { x: 7.0, y: 8.0 }),
            r#in: None,
            out: None,
        };
        assert_eq!(line_to_no_in_out.to_string(), "-- (7.00,8.00)");
    }

    #[test]
    fn test_line_arc_display() {
        let line_arc = LineArc {
            radius: 5.0,
            start_angle: 0.0,
            end_angle: 90.0,
        };
        assert_eq!(line_arc.to_string(), "arc (5.00:0.00:90.00)");
    }

    #[test]
    fn test_line_display() {
        let line_to = Line::To(LineTo {
            terminus: Node::XY(XYNode { x: 3.0, y: 4.0 }),
            r#in: Some(45.0),
            out: Some(30.0),
        });
        assert_eq!(line_to.to_string(), "to[in=45.00, out=30.00] (3.00,4.00)");
        let line_arc = Line::Arc(LineArc {
            radius: 5.0,
            start_angle: 0.0,
            end_angle: 90.0,
        });
        assert_eq!(line_arc.to_string(), "arc (5.00:0.00:90.00)");
    }

    #[test]
    fn test_path_display() {
        let path = Path {
            start: Node::XY(XYNode { x: 0.0, y: 0.0 }),
            lines: vec![
                Line::To(LineTo {
                    terminus: Node::XY(XYNode { x: 3.0, y: 4.0 }),
                    r#in: Some(45.0),
                    out: Some(30.0),
                }),
                Line::Arc(LineArc {
                    radius: 5.0,
                    start_angle: 0.0,
                    end_angle: 90.0,
                }),
            ],
        };
        assert_eq!(path.to_string(), "\\draw (0.00,0.00) to[in=45.00, out=30.00] (3.00,4.00) arc (5.00:0.00:90.00);");
    }
}