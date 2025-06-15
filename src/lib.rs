pub mod dot;
pub mod events;
pub mod plantuml;
mod types;

// Main event-based API
pub use events::{
    Direction, EdgeType, EventResult, GraphEvent, GroupType, LayoutType, MessageType, NodeType,
    Position, Properties, StateType, Style,
};

// Legacy types - deprecated
#[deprecated(note = "Use the event-based API instead")]
pub use types::{GraphData, NodeInfo};
