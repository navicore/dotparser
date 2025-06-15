use crate::events::{Direction, EdgeType, GraphEvent, LayoutType, NodeType, Position, Properties};
use crate::plantuml::types::ArrowType;
use pest::Parser;
use pest_derive::Parser;
use std::collections::HashMap;

#[derive(Parser)]
#[grammar = "plantuml/grammar.pest"]
pub struct PlantUMLParser;

/// Parse a `PlantUML` sequence diagram and return events
pub fn parse(input: &str) -> Result<Vec<GraphEvent>, String> {
    let mut events = Vec::new();
    let mut participant_order = 0;
    let mut sequence_number = 0;
    let mut participants = HashMap::new(); // alias -> id mapping
    let mut known_ids = std::collections::HashSet::new(); // track all known IDs

    // Start batch
    events.push(GraphEvent::BatchStart);

    // Set layout for sequence diagrams
    events.push(GraphEvent::SetLayout {
        layout_type: LayoutType::Sequential {
            direction: Direction::LeftToRight,
        },
        properties: Properties::default(),
    });

    let pairs =
        PlantUMLParser::parse(Rule::plantuml, input).map_err(|e| format!("Parse error: {e}"))?;

    for pair in pairs {
        if pair.as_rule() == Rule::plantuml {
            for inner_pair in pair.into_inner() {
                if inner_pair.as_rule() == Rule::diagram_content {
                    process_diagram_content(
                        inner_pair,
                        &mut events,
                        &mut participant_order,
                        &mut sequence_number,
                        &mut participants,
                        &mut known_ids,
                    )?;
                }
            }
        }
    }

    // End batch
    events.push(GraphEvent::BatchEnd);

    Ok(events)
}

fn process_diagram_content(
    pairs: pest::iterators::Pair<Rule>,
    events: &mut Vec<GraphEvent>,
    participant_order: &mut u32,
    sequence_number: &mut u32,
    participants: &mut HashMap<String, String>, // alias -> id mapping
    known_ids: &mut std::collections::HashSet<String>,
) -> Result<(), String> {
    for pair in pairs.into_inner() {
        match pair.as_rule() {
            Rule::participant_declaration => {
                process_participant(pair, events, participant_order, participants, known_ids);
            }
            Rule::message => {
                process_message(
                    pair,
                    events,
                    participant_order,
                    sequence_number,
                    participants,
                    known_ids,
                )?;
            }
            Rule::activation => {
                process_activation(pair, events);
            }
            Rule::deactivation => {
                process_deactivation(pair, events);
            }
            _ => {
                // TODO: Handle notes, control blocks, and other rules
            }
        }
    }
    Ok(())
}

fn process_participant(
    pair: pest::iterators::Pair<Rule>,
    events: &mut Vec<GraphEvent>,
    participant_order: &mut u32,
    participants: &mut HashMap<String, String>,
    known_ids: &mut std::collections::HashSet<String>,
) {
    let mut participant_type = "participant";
    let mut id = String::new();
    let mut alias = None;

    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::participant_type => {
                participant_type = inner_pair.as_str();
            }
            Rule::identifier => {
                id = extract_identifier(inner_pair);
            }
            Rule::alias => {
                // Extract the identifier from the alias
                for alias_inner in inner_pair.into_inner() {
                    if alias_inner.as_rule() == Rule::identifier {
                        alias = Some(extract_identifier(alias_inner));
                    }
                }
            }
            _ => {}
        }
    }

    let display_name = alias.clone().unwrap_or_else(|| id.clone());

    // Store mapping for message resolution
    if let Some(alias_name) = &alias {
        participants.insert(alias_name.clone(), id.clone());
    }

    let node_type = match participant_type {
        "actor" => NodeType::Actor {
            actor_type: "human".to_string(),
        },
        "database" => NodeType::DataStore,
        "entity" => NodeType::External,
        "boundary" | "control" => NodeType::Process,
        _ => NodeType::Actor {
            actor_type: participant_type.to_string(),
        },
    };

    let properties = Properties {
        position: Some(Position::Sequential {
            order: *participant_order,
        }),
        ..Default::default()
    };

    events.push(GraphEvent::AddNode {
        id: id.clone(),
        label: Some(display_name),
        node_type,
        properties,
    });

    known_ids.insert(id);
    *participant_order += 1;
}

fn process_message(
    pair: pest::iterators::Pair<Rule>,
    events: &mut Vec<GraphEvent>,
    participant_order: &mut u32,
    sequence_number: &mut u32,
    participants: &HashMap<String, String>,
    known_ids: &mut std::collections::HashSet<String>,
) -> Result<(), String> {
    let mut from = String::new();
    let mut to = String::new();
    let mut arrow_str = String::new();
    let mut text = String::new();

    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::identifier => {
                if from.is_empty() {
                    from = extract_identifier(inner_pair);
                } else {
                    to = extract_identifier(inner_pair);
                }
            }
            Rule::arrow => {
                arrow_str = inner_pair.as_str().to_string();
            }
            Rule::message_label => {
                // Extract message text
                for label_inner in inner_pair.into_inner() {
                    if label_inner.as_rule() == Rule::message_text {
                        text = label_inner.as_str().trim().to_string();
                    }
                }
            }
            _ => {}
        }
    }

    // Parse arrow type
    let arrow_type = ArrowType::parse_arrow(&arrow_str)
        .ok_or_else(|| format!("Unknown arrow type: {arrow_str}"))?;

    // Handle reversed arrows
    let (actual_from, actual_to) = if arrow_type.is_reversed() {
        (to, from)
    } else {
        (from, to)
    };

    // Resolve aliases to IDs
    let from_id = participants
        .get(&actual_from)
        .cloned()
        .unwrap_or_else(|| actual_from.clone());
    let to_id = participants
        .get(&actual_to)
        .cloned()
        .unwrap_or_else(|| actual_to.clone());

    // Auto-create participants if not declared
    if !known_ids.contains(&from_id) {
        let properties = Properties {
            position: Some(Position::Sequential {
                order: *participant_order,
            }),
            ..Default::default()
        };

        events.push(GraphEvent::AddNode {
            id: from_id.clone(),
            label: Some(actual_from),
            node_type: NodeType::Actor {
                actor_type: "participant".to_string(),
            },
            properties,
        });

        known_ids.insert(from_id.clone());
        *participant_order += 1;
    }

    if !known_ids.contains(&to_id) {
        let properties = Properties {
            position: Some(Position::Sequential {
                order: *participant_order,
            }),
            ..Default::default()
        };

        events.push(GraphEvent::AddNode {
            id: to_id.clone(),
            label: Some(actual_to),
            node_type: NodeType::Actor {
                actor_type: "participant".to_string(),
            },
            properties,
        });

        known_ids.insert(to_id.clone());
        *participant_order += 1;
    }

    // Create message edge
    let message_type = arrow_type.to_message_type();
    let edge_type = EdgeType::Message {
        message_type,
        sequence: Some(*sequence_number),
    };

    events.push(GraphEvent::AddEdge {
        id: format!("msg-{sequence_number}"),
        from: from_id,
        to: to_id,
        edge_type,
        label: if text.is_empty() { None } else { Some(text) },
        properties: Properties::default(),
    });

    *sequence_number += 1;

    Ok(())
}

fn process_activation(pair: pest::iterators::Pair<Rule>, events: &mut Vec<GraphEvent>) {
    for inner_pair in pair.into_inner() {
        if inner_pair.as_rule() == Rule::identifier {
            let id = extract_identifier(inner_pair);

            // Update node to show activation
            let mut properties = Properties::default();
            properties
                .custom
                .insert("activated".to_string(), "true".to_string());

            events.push(GraphEvent::UpdateNode {
                id,
                label: None,
                properties,
            });
        }
    }
}

fn process_deactivation(pair: pest::iterators::Pair<Rule>, events: &mut Vec<GraphEvent>) {
    for inner_pair in pair.into_inner() {
        if inner_pair.as_rule() == Rule::identifier {
            let id = extract_identifier(inner_pair);

            // Update node to show deactivation
            let mut properties = Properties::default();
            properties
                .custom
                .insert("activated".to_string(), "false".to_string());

            events.push(GraphEvent::UpdateNode {
                id,
                label: None,
                properties,
            });
        }
    }
}

fn extract_identifier(pair: pest::iterators::Pair<Rule>) -> String {
    match pair.as_rule() {
        Rule::identifier => {
            // Check if it's a quoted string or simple identifier
            for inner in pair.into_inner() {
                match inner.as_rule() {
                    Rule::quoted_string => {
                        return extract_quoted_string(inner).unwrap_or_default();
                    }
                    Rule::simple_identifier => {
                        return inner.as_str().to_string();
                    }
                    _ => {}
                }
            }
            String::new()
        }
        Rule::quoted_string => extract_quoted_string(pair).unwrap_or_default(),
        _ => pair.as_str().to_string(),
    }
}

fn extract_quoted_string(pair: pest::iterators::Pair<Rule>) -> Option<String> {
    for inner in pair.into_inner() {
        if inner.as_rule() == Rule::inner_string {
            return Some(inner.as_str().to_string());
        }
    }
    None
}

// ArrowType implementation is in plantuml/types.rs

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_sequence_to_events() {
        let input = r"@startuml
participant A
participant B
A -> B: Hello
B --> A: Hi
@enduml";

        let events = parse(input).unwrap();

        // Should have: BatchStart, SetLayout, 2 participants, 2 messages, BatchEnd
        assert!(events.len() >= 7);
        assert!(matches!(events.first(), Some(GraphEvent::BatchStart)));
        assert!(matches!(events.last(), Some(GraphEvent::BatchEnd)));

        // Check for layout event
        let has_layout = events
            .iter()
            .any(|e| matches!(e, GraphEvent::SetLayout { .. }));
        assert!(has_layout);

        // Count nodes and messages
        let node_count = events
            .iter()
            .filter(|e| matches!(e, GraphEvent::AddNode { .. }))
            .count();
        let message_count = events
            .iter()
            .filter(|e| {
                matches!(
                    e,
                    GraphEvent::AddEdge {
                        edge_type: EdgeType::Message { .. },
                        ..
                    }
                )
            })
            .count();

        assert_eq!(node_count, 2);
        assert_eq!(message_count, 2);
    }

    #[test]
    fn test_participant_types_to_events() {
        let input = r"@startuml
actor User
database DB
entity Service
User -> Service: Request
Service -> DB: Query
@enduml";

        let events = parse(input).unwrap();

        // Find the actor event
        let actor_event = events
            .iter()
            .find(|e| matches!(e, GraphEvent::AddNode { id, .. } if id == "User"));

        assert!(actor_event.is_some());

        if let Some(GraphEvent::AddNode { node_type, .. }) = actor_event {
            assert!(matches!(node_type, NodeType::Actor { .. }));
        }

        // Find the database event
        let db_event = events
            .iter()
            .find(|e| matches!(e, GraphEvent::AddNode { id, .. } if id == "DB"));

        if let Some(GraphEvent::AddNode { node_type, .. }) = db_event {
            assert!(matches!(node_type, NodeType::DataStore));
        }
    }

    #[test]
    fn test_participant_alias_to_events() {
        let input = r#"@startuml
participant A as "Alice"
participant B as "Bob"
A -> B: Hello Bob
@enduml"#;

        let events = parse(input).unwrap();

        // Find Alice node
        let alice_event = events
            .iter()
            .find(|e| matches!(e, GraphEvent::AddNode { id, .. } if id == "A"));

        if let Some(GraphEvent::AddNode { label, .. }) = alice_event {
            assert_eq!(label.as_deref(), Some("Alice"));
        }
    }
}
