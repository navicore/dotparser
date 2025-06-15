# DOT Parser Module

This module provides parsing support for Graphviz DOT format files.

## Usage

```rust
use dotparser::dot;

let dot_content = r#"
    digraph G {
        A -> B;
        B -> C;
    }
"#;

let events = dot::parse(dot_content);
```

## Features

- Parses both directed (`digraph`) and undirected (`graph`) graphs
- Supports node and edge attributes
- Handles nested subgraphs
- Extracts layout hints (e.g., `rankdir`)
- Emits rich graph events for visualization

## Event Types

The parser emits:
- `GraphEvent::SetLayout` - Layout hints from the DOT file
- `GraphEvent::AddNode` - For each node with attributes
- `GraphEvent::AddEdge` - For each edge
- `GraphEvent::BatchStart/BatchEnd` - For efficient processing