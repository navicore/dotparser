# dotparser

A Rust library for parsing Graphviz DOT files into graph structures.

## Overview

`dotparser` converts DOT format files into a graph representation using `petgraph`. It handles both standard edge-based graphs and nested subgraph structures.

## Usage

```rust
use dotparser::dot;

let dot_content = r#"
    digraph {
        A -> B;
        B -> C;
    }
"#;

let graph_data = dot::parse(dot_content);
println!("Nodes: {}", graph_data.graph.node_count());
println!("Edges: {}", graph_data.graph.edge_count());
```

## Features

- Parses standard DOT edge notation (`A -> B`)
- Supports nested subgraphs
- Handles node attributes (type, level)
- Preserves node labels and relationships

## Data Structures

The parser outputs a `GraphData` structure containing:
- A `petgraph::DiGraph` with node information
- A HashMap for node name lookups

Nodes can have types like Organization, Team, User, etc., useful for hierarchical visualizations.

## Future

This crate currently supports DOT format only. Other diagram formats may be added as separate modules.

## License

MIT