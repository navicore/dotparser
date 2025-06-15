// Legacy types kept for backward compatibility
// New code should use the events API directly

use petgraph::graph::{DiGraph, NodeIndex};
use std::collections::HashMap;

/// Legacy node info - will be removed in future versions
#[derive(Debug, Clone)]
pub struct NodeInfo {
    pub name: String,
    pub node_type: Option<String>,
    pub level: u32,
}

/// Legacy graph data - will be removed in future versions
#[derive(Debug, Clone)]
pub struct GraphData {
    pub graph: DiGraph<NodeInfo, ()>,
    pub node_map: HashMap<String, NodeIndex>,
}

// Note: SequenceData, Participant, etc. have been removed
// Use GraphEvent streams instead
