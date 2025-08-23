use std::fmt::{Display, Formatter};
use std::str::FromStr;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct XYNode {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RThetaNode {
    pub r: f64,
    pub theta: f64, // in degrees
}

impl Display for XYNode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "({:.2},{:.2})", self.x, self.y)
    }
}

impl FromStr for XYNode {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim_matches(|c| c == '(' || c == ')');
        let parts: Vec<&str> = s.split(',').collect();
        if parts.len() != 2 {
            return Err("Invalid format".to_string());
        }
        let x = parts[0].parse::<f64>().map_err(|_| "Invalid x value".to_string())?;
        let y = parts[1].parse::<f64>().map_err(|_| "Invalid y value".to_string())?;
        Ok(XYNode { x, y })
    }
}

impl Display for RThetaNode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "({:.2}:{:.2})", self.r, self.theta)
    }
}

impl FromStr for RThetaNode {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim_matches(|c| c == '(' || c == ')');
        let parts: Vec<&str> = s.split(':').collect();
        if parts.len() != 2 {
            return Err("Invalid format".to_string());
        }
        let r = parts[0].parse::<f64>().map_err(|_| "Invalid r value".to_string())?;
        let theta = parts[1].parse::<f64>().map_err(|_| "Invalid theta value".to_string())?;
        Ok(RThetaNode { r, theta })
    }
}

impl From<XYNode> for RThetaNode {
    fn from(node: XYNode) -> Self {
        let r = (node.x.powi(2) + node.y.powi(2)).sqrt();
        let theta = node.y.atan2(node.x).to_degrees();
        RThetaNode { r, theta }
    }
}

impl From<RThetaNode> for XYNode {
    fn from(node: RThetaNode) -> Self {
        let x = node.r * node.theta.to_radians().cos();
        let y = node.r * node.theta.to_radians().sin();
        XYNode { x, y }
    }
}

impl From<(f64, f64)> for XYNode {
    fn from(coords: (f64, f64)) -> Self {
        XYNode {
            x: coords.0,
            y: coords.1,
        }
    }
}

impl From<(f64, f64)> for RThetaNode {
    fn from(coords: (f64, f64)) -> Self {
        let r = (coords.0.powi(2) + coords.1.powi(2)).sqrt();
        let theta = coords.1.atan2(coords.0);
        RThetaNode { r, theta }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Node {
    XY(XYNode),
    RTheta(RThetaNode),
}

#[allow(unused)]
impl Node {
    pub fn into_xy(self) -> XYNode {
        match self {
            Node::XY(node) => node.clone(),
            Node::RTheta(node) => node.clone().into(),
        }
    }

    pub fn into_rtheta(self) -> RThetaNode {
        match self {
            Node::XY(node) => node.clone().into(),
            Node::RTheta(node) => node.clone(),
        }
    }
    
    pub fn as_xy(&self) -> &XYNode {
        match self {
            Node::XY(node) => node,
            Node::RTheta(_) => panic!("Node is not in XY format"),
        }
    }
    
    pub fn as_rtheta(&self) -> &RThetaNode {
        match self {
            Node::XY(_) => panic!("Node is not in RTheta format"),
            Node::RTheta(node) => node,
        }
    }
    
    pub fn as_xy_mut(&mut self) -> &mut XYNode {
        match self {
            Node::XY(node) => node,
            Node::RTheta(_) => panic!("Node is not in XY format"),
        }
    }
    
    pub fn as_rtheta_mut(&mut self) -> &mut RThetaNode {
        match self {
            Node::XY(_) => panic!("Node is not in RTheta format"),
            Node::RTheta(node) => node,
        }
    }
}

impl From<XYNode> for Node {
    fn from(node: XYNode) -> Self {
        Node::XY(node)
    }
}

impl From<RThetaNode> for Node {
    fn from(node: RThetaNode) -> Self {
        Node::RTheta(node)
    }
}

impl Display for Node {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Node::XY(node) => write!(f, "{}", node),
            Node::RTheta(node) => write!(f, "{}", node),
        }
    }
}

impl FromStr for Node {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.contains(',') {
            let xy_node: XYNode = s.parse()?;
            Ok(Node::XY(xy_node))
        } else if s.contains(':') {
            let rtheta_node: RThetaNode = s.parse()?;
            Ok(Node::RTheta(rtheta_node))
        } else {
            Err("Invalid node format".to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xy_to_rtheta_conversion() {
        let xy_node = XYNode { x: 1.0, y: 1.0 };
        let rtheta_node: RThetaNode = xy_node.clone().into();
        assert!((rtheta_node.r - (2.0f64).sqrt()).abs() < 1e-6);
        assert!((rtheta_node.theta - 45.0).abs() < 1e-6);

        let converted_back: XYNode = rtheta_node.into();
        assert!((converted_back.x - xy_node.x).abs() < 1e-6);
        assert!((converted_back.y - xy_node.y).abs() < 1e-6);
    }

    #[test]
    fn test_rtheta_to_xy_conversion() {
        let rtheta_node = RThetaNode { r: 2.0, theta: 90.0 };
        let xy_node: XYNode = rtheta_node.clone().into();
        assert!((xy_node.x - 0.0).abs() < 1e-6);
        assert!((xy_node.y - 2.0).abs() < 1e-6);

        let converted_back: RThetaNode = xy_node.into();
        assert!((converted_back.r - rtheta_node.r).abs() < 1e-6);
        assert!((converted_back.theta - rtheta_node.theta).abs() < 1e-6);
    }

    #[test]
    fn test_node_display() {
        let xy_node = Node::XY(XYNode { x: 3.0, y: 4.0 });
        assert_eq!(format!("{}", xy_node), "(3.00,4.00)");

        let rtheta_node = Node::RTheta(RThetaNode { r: 5.0, theta: 53.13 });
        assert_eq!(format!("{}", rtheta_node), "(5.00:53.13)");
    }

    #[test]
    fn test_xynode_fromstr() {
        let s = "(3.0,4.0)";
        let node: XYNode = s.parse().unwrap();
        assert_eq!(node, XYNode { x: 3.0, y: 4.0 });
    }

    #[test]
    fn test_rthetanode_fromstr() {
        let s = "(5.0:53.13)";
        let node: RThetaNode = s.parse().unwrap();
        assert_eq!(node, RThetaNode { r: 5.0, theta: 53.13 });
    }

    #[test]
    fn test_node_fromstr() {
        let s_xy = "(3.0,4.0)";
        let node_xy: Node = s_xy.parse().unwrap();
        assert_eq!(node_xy, Node::XY(XYNode { x: 3.0, y: 4.0 }));

        let s_rtheta = "(5.0:53.13)";
        let node_rtheta: Node = s_rtheta.parse().unwrap();
        assert_eq!(node_rtheta, Node::RTheta(RThetaNode { r: 5.0, theta: 53.13 }));
    }
}