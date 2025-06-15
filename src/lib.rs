mod parser;
pub mod types;

// Re-export types that all parsers will use
pub use types::{GraphData, NodeInfo, NodeType};

// Parser modules - each format gets its own module
pub mod dot {
    pub use crate::parser::parse_dot_file as parse;
}

// Future: pub mod plantuml { ... }
// Future: pub mod mermaid { ... }
