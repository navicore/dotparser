use petgraph::graph::{DiGraph, NodeIndex};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NodeType {
    Organization,
    LineOfBusiness,
    Site,
    Team,
    User,
    Default,
}

impl NodeType {
    #[must_use]
    pub fn parse(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "organization" | "org" => Self::Organization,
            "lob" | "lineofbusiness" | "line_of_business" => Self::LineOfBusiness,
            "site" => Self::Site,
            "team" => Self::Team,
            "user" => Self::User,
            _ => Self::Default,
        }
    }
}

#[derive(Debug)]
pub struct NodeInfo {
    pub name: String,
    pub node_type: NodeType,
    pub level: u32,
}

pub struct GraphData {
    pub graph: DiGraph<NodeInfo, ()>,
    #[allow(dead_code)]
    pub node_map: HashMap<String, NodeIndex>,
}
