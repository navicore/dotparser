#![allow(clippy::cast_possible_truncation)] // Stack depth won't exceed u32::MAX

use crate::events::{Direction, EdgeType, GraphEvent, LayoutType, NodeType, Position, Properties};
use std::collections::HashMap;

// Type alias for node attributes to reduce complexity
type NodeAttributes = HashMap<
    String,
    (
        Option<String>,
        Option<u32>,
        Option<String>,
        HashMap<String, String>,
    ),
>;

/// Parse a DOT file and return events
pub fn parse_dot_to_events(content: &str) -> Vec<GraphEvent> {
    let mut events = Vec::new();
    let mut node_attributes = HashMap::new();

    // Start batch for efficiency
    events.push(GraphEvent::BatchStart);

    // Check if this is a nested subgraph format
    let has_edges = content.contains("->");
    let is_digraph = content.contains("digraph");

    if !has_edges && content.contains("subgraph") {
        parse_nested_subgraphs_to_events(content, &mut events);
    } else {
        parse_regular_dot(content, &mut events, &mut node_attributes, is_digraph);
    }

    // End batch
    events.push(GraphEvent::BatchEnd);

    events
}

fn parse_regular_dot(
    content: &str,
    events: &mut Vec<GraphEvent>,
    node_attributes: &mut NodeAttributes,
    is_digraph: bool,
) {
    // Detect layout direction
    if let Some(rankdir) = extract_rankdir(content) {
        let direction = match rankdir.as_str() {
            "BT" => Direction::BottomToTop,
            "LR" => Direction::LeftToRight,
            "RL" => Direction::RightToLeft,
            _ => Direction::TopToBottom, // Default: TB
        };
        events.push(GraphEvent::SetLayout {
            layout_type: LayoutType::Hierarchical { direction },
            properties: Properties::default(),
        });
    }

    let lines: Vec<&str> = content.lines().collect();

    // Parse nodes
    parse_nodes(&lines, events, node_attributes);

    // Parse edges
    parse_edges(&lines, events, node_attributes, is_digraph);
}

fn parse_nodes(lines: &[&str], events: &mut Vec<GraphEvent>, node_attributes: &mut NodeAttributes) {
    for line in lines {
        let trimmed = line.trim();

        // Skip comments and empty lines
        if trimmed.starts_with("//") || trimmed.is_empty() {
            continue;
        }

        // Parse node definitions with attributes
        if trimmed.contains('[') && trimmed.contains(']') && !trimmed.contains("->") {
            if let Some(node_end) = trimmed.find('[') {
                let node_id = trimmed[..node_end].trim().trim_matches('"');

                // Extract attributes
                let attrs_str = &trimmed[node_end + 1..trimmed.rfind(']').unwrap_or(trimmed.len())];
                let mut node_type = None;
                let mut level = None;
                let mut label = None;
                let mut properties = Properties::default();
                let mut custom_props = HashMap::new();

                // Parse attributes
                for attr in attrs_str.split(',') {
                    let parts: Vec<&str> = attr.split('=').collect();
                    if parts.len() == 2 {
                        let key = parts[0].trim();
                        let value = parts[1].trim().trim_matches('"');

                        match key {
                            "type" => node_type = Some(value.to_string()),
                            "level" => level = value.parse::<u32>().ok(),
                            "label" => label = Some(value.to_string()),
                            _ => {
                                custom_props.insert(key.to_string(), value.to_string());
                            }
                        }
                    }
                }

                // Store attributes for later use
                node_attributes.insert(
                    node_id.to_string(),
                    (
                        node_type.clone(),
                        level,
                        label.clone(),
                        custom_props.clone(),
                    ),
                );

                // Set position if level is specified
                if let Some(lvl) = level {
                    properties.position = Some(Position::Layer { level: lvl });
                }

                properties.custom = custom_props;

                // Emit node event
                events.push(GraphEvent::AddNode {
                    id: node_id.to_string(),
                    label: label.or_else(|| Some(node_id.to_string())),
                    node_type: node_type.map_or(NodeType::Node, NodeType::Custom),
                    properties,
                });
            }
        }
    }
}

fn parse_edges(
    lines: &[&str],
    events: &mut Vec<GraphEvent>,
    node_attributes: &mut NodeAttributes,
    is_digraph: bool,
) {
    for line in lines {
        let trimmed = line.trim();

        if trimmed.contains("->") || trimmed.contains("--") {
            let arrow = if is_digraph { "->" } else { "--" };
            if let Some(arrow_pos) = trimmed.find(arrow) {
                let from = trimmed[..arrow_pos]
                    .trim()
                    .trim_matches('"')
                    .trim_end_matches(';');

                let to_part = &trimmed[arrow_pos + arrow.len()..];
                let to = to_part
                    .split('[')
                    .next()
                    .unwrap_or(to_part)
                    .trim()
                    .trim_matches('"')
                    .trim_end_matches(';');

                // Ensure nodes exist
                if !node_attributes.contains_key(from) {
                    events.push(GraphEvent::AddNode {
                        id: from.to_string(),
                        label: Some(from.to_string()),
                        node_type: NodeType::Node,
                        properties: Properties::default(),
                    });
                    node_attributes.insert(from.to_string(), (None, None, None, HashMap::new()));
                }

                if !node_attributes.contains_key(to) {
                    events.push(GraphEvent::AddNode {
                        id: to.to_string(),
                        label: Some(to.to_string()),
                        node_type: NodeType::Node,
                        properties: Properties::default(),
                    });
                    node_attributes.insert(to.to_string(), (None, None, None, HashMap::new()));
                }

                // Create edge
                let edge_type = if is_digraph {
                    EdgeType::Directed
                } else {
                    EdgeType::Undirected
                };

                events.push(GraphEvent::AddEdge {
                    id: format!("{from}{arrow}{to}"),
                    from: from.to_string(),
                    to: to.to_string(),
                    edge_type,
                    label: None,
                    properties: Properties::default(),
                });
            }
        }
    }
}

fn extract_rankdir(content: &str) -> Option<String> {
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("rankdir") {
            if let Some(eq_pos) = trimmed.find('=') {
                let value = trimmed[eq_pos + 1..]
                    .trim()
                    .trim_end_matches(';')
                    .trim_matches('"');
                return Some(value.to_string());
            }
        }
    }
    None
}

fn parse_nested_subgraphs_to_events(content: &str, events: &mut Vec<GraphEvent>) {
    let mut stack: Vec<(String, Option<String>)> = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim();

        // Parse subgraph start
        if trimmed.starts_with("subgraph") {
            if let Some(cluster_start) = trimmed.find("cluster_") {
                let cluster_name = trimmed[cluster_start..]
                    .split_whitespace()
                    .next()
                    .unwrap_or("");

                // Find label in subsequent lines
                stack.push((cluster_name.to_string(), None));
            }
        }
        // Parse labels
        else if (trimmed.starts_with("label=") || trimmed.starts_with("Label="))
            && !stack.is_empty()
        {
            let label = extract_label_value(trimmed);

            // Determine node type based on label content
            let node_type = if label.to_lowercase().contains("tenant")
                || label.to_lowercase().contains("organization")
            {
                NodeType::Custom("organization".to_string())
            } else if label.to_lowercase().contains("contact center") {
                NodeType::Custom("line_of_business".to_string())
            } else if label.to_lowercase().contains("site") {
                NodeType::Custom("site".to_string())
            } else {
                NodeType::Node
            };

            let level = stack.len() as u32 - 1;
            let properties = Properties {
                position: Some(Position::Layer { level }),
                ..Default::default()
            };

            // Create node for this cluster
            let node_id = label.clone();
            events.push(GraphEvent::AddNode {
                id: node_id.clone(),
                label: Some(label),
                node_type,
                properties,
            });

            // Connect to parent if exists
            if stack.len() > 1 {
                if let Some((_, Some(parent_id))) = stack.iter().rev().nth(1) {
                    events.push(GraphEvent::AddEdge {
                        id: format!("{parent_id}->{node_id}"),
                        from: parent_id.clone(),
                        to: node_id.clone(),
                        edge_type: EdgeType::Directed,
                        label: None,
                        properties: Properties::default(),
                    });
                }
            }

            // Update stack with node ID
            if let Some((cluster, _)) = stack.last_mut() {
                *stack.last_mut().unwrap() = (cluster.clone(), Some(node_id));
            }
        }
        // Parse standalone nodes
        else if trimmed.contains('[') && trimmed.contains("label=") && !trimmed.contains("->") {
            if let Some(node_end) = trimmed.find('[') {
                let node_id = trimmed[..node_end].trim().trim_matches('"');
                let label = extract_node_label(trimmed).unwrap_or_else(|| node_id.to_string());

                let level = stack.len() as u32;
                let node_type = if label.to_lowercase().contains("supervisor") {
                    NodeType::Custom("team".to_string())
                } else {
                    NodeType::Custom("user".to_string())
                };

                let properties = Properties {
                    position: Some(Position::Layer { level }),
                    ..Default::default()
                };

                events.push(GraphEvent::AddNode {
                    id: label.clone(),
                    label: Some(label.clone()),
                    node_type,
                    properties,
                });

                // Connect to parent if exists
                if let Some((_, Some(parent_id))) = stack.last() {
                    events.push(GraphEvent::AddEdge {
                        id: format!("{parent_id}->{label}"),
                        from: parent_id.clone(),
                        to: label,
                        edge_type: EdgeType::Directed,
                        label: None,
                        properties: Properties::default(),
                    });
                }
            }
        }
        // Handle closing braces
        else if trimmed == "}" && !stack.is_empty() {
            stack.pop();
        }
    }
}

fn extract_label_value(line: &str) -> String {
    let label_start = line.find('=').unwrap_or(0) + 1;
    let mut label = line[label_start..]
        .trim()
        .trim_matches('"')
        .trim_matches(';')
        .to_string();

    // Extract meaningful name from label (after the colon if present)
    if let Some(colon_pos) = label.find(':') {
        label = label[colon_pos + 1..].trim().to_string();
    }

    label
}

fn extract_node_label(line: &str) -> Option<String> {
    line.find("label=").and_then(|label_start| {
        let label_part = &line[label_start + 6..];
        label_part.find('"').and_then(|first_quote| {
            label_part[first_quote + 1..].find('"').map(|second_quote| {
                label_part[first_quote + 1..first_quote + 1 + second_quote]
                    .replace("\\n", " ")
                    .trim()
                    .to_string()
            })
        })
    })
}

// ============================================================================
// Legacy API - Deprecated
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_graph_to_events() {
        let dot = r"
            digraph {
                A -> B;
                B -> C;
            }
        ";

        let events = parse_dot_to_events(dot);

        // Should have: BatchStart, 3 nodes, 2 edges, BatchEnd
        assert!(events.len() >= 6);
        assert!(matches!(events.first(), Some(GraphEvent::BatchStart)));
        assert!(matches!(events.last(), Some(GraphEvent::BatchEnd)));

        // Count node and edge events
        let node_count = events
            .iter()
            .filter(|e| matches!(e, GraphEvent::AddNode { .. }))
            .count();
        let edge_count = events
            .iter()
            .filter(|e| matches!(e, GraphEvent::AddEdge { .. }))
            .count();

        assert_eq!(node_count, 3);
        assert_eq!(edge_count, 2);
    }

    #[test]
    fn test_parse_node_with_attributes_to_events() {
        let dot = r#"
            digraph {
                "Node1" [type="team", level="2", label="Team Alpha"];
                "Node2" [type="user", level="1"];
                "Node1" -> "Node2";
            }
        "#;

        let events = parse_dot_to_events(dot);

        // Find the Node1 event
        let node1_event = events
            .iter()
            .find(|e| matches!(e, GraphEvent::AddNode { id, .. } if id == "Node1"));

        assert!(node1_event.is_some());

        if let Some(GraphEvent::AddNode {
            label,
            node_type,
            properties,
            ..
        }) = node1_event
        {
            assert_eq!(label.as_deref(), Some("Team Alpha"));
            assert!(matches!(node_type, NodeType::Custom(t) if t == "team"));
            assert!(matches!(
                properties.position,
                Some(Position::Layer { level: 2 })
            ));
        }
    }

    #[test]
    fn test_layout_detection() {
        let dot = r"
            digraph {
                rankdir=LR;
                A -> B;
            }
        ";

        let events = parse_dot_to_events(dot);

        // Should have a SetLayout event
        let layout_event = events
            .iter()
            .find(|e| matches!(e, GraphEvent::SetLayout { .. }));
        assert!(layout_event.is_some());

        if let Some(GraphEvent::SetLayout { layout_type, .. }) = layout_event {
            assert!(matches!(
                layout_type,
                LayoutType::Hierarchical {
                    direction: Direction::LeftToRight
                }
            ));
        }
    }
}
