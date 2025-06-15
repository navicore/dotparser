use dotparser::{EdgeType, GraphEvent, plantuml};

fn main() {
    let sequence_diagram = r#"
@startuml
actor User
participant "Web Server" as Web
database "Database" as DB

User -> Web: HTTP Request
activate Web
Web -> DB: Query
activate DB
DB --> Web: Results
deactivate DB
Web --> User: HTTP Response
deactivate Web
@enduml
"#;

    match plantuml::parse(sequence_diagram) {
        Ok(events) => {
            println!("Parsed PlantUML sequence diagram as events:");

            // Count participants and messages
            let mut participants = Vec::new();
            let mut messages = Vec::new();

            for event in &events {
                match event {
                    GraphEvent::AddNode {
                        id,
                        label,
                        node_type,
                        ..
                    } => {
                        participants.push((id.clone(), label.clone(), node_type.clone()));
                    }
                    GraphEvent::AddEdge {
                        edge_type: EdgeType::Message { .. },
                        from,
                        to,
                        label,
                        ..
                    } => {
                        messages.push((from.clone(), to.clone(), label.clone()));
                    }
                    _ => {}
                }
            }

            println!("Participants: {}", participants.len());
            for (id, label, node_type) in &participants {
                println!(
                    "  - {} ({:?}): {}",
                    id,
                    node_type,
                    label.as_deref().unwrap_or("no label")
                );
            }

            println!("\nMessages: {}", messages.len());
            for (i, (from, to, label)) in messages.iter().enumerate() {
                println!(
                    "  {}. {} -> {}: {}",
                    i + 1,
                    from,
                    to,
                    label.as_deref().unwrap_or("no label")
                );
            }
        }
        Err(e) => {
            eprintln!("Error parsing PlantUML: {e}");
        }
    }
}
