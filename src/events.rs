#![allow(clippy::derive_partial_eq_without_eq)] // Can't derive Eq due to f32 fields

use std::collections::HashMap;

/// Rich graph events that can represent any type of diagram
#[derive(Debug, Clone, PartialEq)]
pub enum GraphEvent {
    /// Add a node to the graph
    AddNode {
        id: String,
        label: Option<String>,
        node_type: NodeType,
        properties: Properties,
    },

    /// Update an existing node
    UpdateNode {
        id: String,
        label: Option<String>,
        properties: Properties,
    },

    /// Remove a node
    RemoveNode { id: String },

    /// Add a connection between nodes
    AddEdge {
        id: String,
        from: String,
        to: String,
        edge_type: EdgeType,
        label: Option<String>,
        properties: Properties,
    },

    /// Update an existing edge
    UpdateEdge {
        id: String,
        label: Option<String>,
        properties: Properties,
    },

    /// Remove an edge
    RemoveEdge { id: String },

    /// Group nodes together
    AddGroup {
        id: String,
        label: Option<String>,
        members: Vec<String>,
        group_type: GroupType,
        properties: Properties,
    },

    /// Update group membership
    UpdateGroup { id: String, members: Vec<String> },

    /// Remove a group
    RemoveGroup { id: String },

    /// Set a layout hint
    SetLayout {
        layout_type: LayoutType,
        properties: Properties,
    },

    /// Clear the entire graph
    Clear,

    /// Batch operation start (for performance)
    BatchStart,

    /// Batch operation end
    BatchEnd,
}

/// Types of nodes - generic enough for any diagram
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NodeType {
    /// Standard node (default)
    Node,
    /// Actor/participant in sequence diagrams
    Actor { actor_type: String },
    /// State in state machines
    State { state_type: StateType },
    /// Process/activity
    Process,
    /// Data store
    DataStore,
    /// External entity
    External,
    /// Custom type with metadata
    Custom(String),
}

/// State types for state machines
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StateType {
    Initial,
    Final,
    Normal,
    Composite,
    History,
}

/// Types of edges/connections
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EdgeType {
    /// Directed edge (default)
    Directed,
    /// Undirected edge
    Undirected,
    /// Bidirectional
    Bidirectional,
    /// Message in sequence diagram
    Message {
        message_type: MessageType,
        sequence: Option<u32>,
    },
    /// State transition
    Transition {
        trigger: Option<String>,
        guard: Option<String>,
        action: Option<String>,
    },
    /// Association
    Association { association_type: String },
    /// Custom edge type
    Custom(String),
}

/// Message types for sequence diagrams
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MessageType {
    Synchronous,
    Asynchronous,
    Return,
    Create,
    Destroy,
}

/// Group types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GroupType {
    /// Logical grouping
    Cluster,
    /// Sequential grouping (alt/loop/opt in sequence diagrams)
    Sequential { sequence_type: String },
    /// Parallel execution
    Parallel,
    /// Hierarchical containment
    Container,
    /// Custom grouping
    Custom(String),
}

/// Layout hints for visualization
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LayoutType {
    /// Hierarchical layout (trees, org charts)
    Hierarchical { direction: Direction },
    /// Force-directed layout
    Force,
    /// Circular layout
    Circular,
    /// Grid layout
    Grid { columns: Option<u32> },
    /// Sequential layout (for sequence diagrams)
    Sequential { direction: Direction },
    /// Layered layout (for workflows)
    Layered { direction: Direction },
    /// Custom layout
    Custom(String),
}

/// Direction for layouts
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Direction {
    TopToBottom,
    BottomToTop,
    LeftToRight,
    RightToLeft,
}

/// Generic properties that can be attached to any element
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Properties {
    /// Visual properties
    pub style: Option<Style>,
    /// Positional hints
    pub position: Option<Position>,
    /// Custom key-value pairs
    pub custom: HashMap<String, String>,
}

/// Visual style properties
#[derive(Debug, Clone, PartialEq)]
pub struct Style {
    pub color: Option<String>,
    pub background_color: Option<String>,
    pub border_style: Option<String>,
    pub border_color: Option<String>,
    pub border_width: Option<f32>,
    pub shape: Option<String>,
    pub size: Option<f32>,
    pub font_size: Option<f32>,
    pub font_family: Option<String>,
    pub opacity: Option<f32>,
}

/// Position hints
#[derive(Debug, Clone, PartialEq)]
pub enum Position {
    /// Absolute coordinates
    Absolute { x: f32, y: f32, z: Option<f32> },
    /// Relative to another element
    Relative {
        anchor: String,
        offset_x: f32,
        offset_y: f32,
        offset_z: Option<f32>,
    },
    /// Grid position
    Grid { row: u32, column: u32 },
    /// Sequential order
    Sequential { order: u32 },
    /// Layer/level in hierarchy
    Layer { level: u32 },
}

/// Result of processing an event
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EventResult {
    Success,
    /// Node already exists
    NodeExists(String),
    /// Node not found
    NodeNotFound(String),
    /// Edge already exists
    EdgeExists(String),
    /// Edge not found
    EdgeNotFound(String),
    /// Invalid operation
    Invalid(String),
}

impl GraphEvent {
    /// Create a simple node event
    pub fn simple_node(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self::AddNode {
            id: id.into(),
            label: Some(label.into()),
            node_type: NodeType::Node,
            properties: Properties::default(),
        }
    }

    /// Create a simple edge event
    pub fn simple_edge(from: impl Into<String>, to: impl Into<String>) -> Self {
        let from = from.into();
        let to = to.into();
        Self::AddEdge {
            id: format!("{from}->{to}"),
            from,
            to,
            edge_type: EdgeType::Directed,
            label: None,
            properties: Properties::default(),
        }
    }
}
