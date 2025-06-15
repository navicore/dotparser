# PlantUML Parser Module

This module provides parsing support for PlantUML sequence diagrams.

## Usage

```rust
use dotparser::plantuml;

let sequence_diagram = r#"
    @startuml
    actor User
    participant Server
    User -> Server: Request
    Server --> User: Response
    @enduml
"#;

let events = plantuml::parse(sequence_diagram)?;
```

## Features

- Parses PlantUML sequence diagrams
- Supports various participant types (actor, database, entity, etc.)
- Handles participant aliases
- Multiple arrow types for different message styles
- Activation/deactivation support
- Auto-creates undeclared participants

## Event Types

The parser emits:
- `GraphEvent::SetLayout` - Sequential layout for proper visualization
- `GraphEvent::AddNode` - For each participant
- `GraphEvent::AddEdge` - For each message with sequence numbers
- `GraphEvent::UpdateNode` - For activation/deactivation
- `GraphEvent::BatchStart/BatchEnd` - For efficient processing

## Grammar

The parser uses Pest with a PEG grammar defined in `grammar.pest`.