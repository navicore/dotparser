use dotparser::{GraphEvent, dot, plantuml};

fn main() {
    println!("=== DOT Graph Events ===");
    demo_dot_events();

    println!("\n=== PlantUML Sequence Events ===");
    demo_plantuml_events();
}

fn demo_dot_events() {
    let dot_content = r#"
        digraph G {
            rankdir=LR;
            
            // Nodes with attributes
            Start [type="initial", label="Start"];
            Process [type="process", label="Data Processing"];
            Decision [type="decision", label="Valid?"];
            End [type="final", label="End"];
            
            // Edges
            Start -> Process;
            Process -> Decision;
            Decision -> End [label="Yes"];
            Decision -> Process [label="No"];
        }
    "#;

    let events = dot::parse(dot_content);

    for event in &events {
        match event {
            GraphEvent::SetLayout { layout_type, .. } => {
                println!("Layout: {layout_type:?}");
            }
            GraphEvent::AddNode {
                id,
                label,
                node_type,
                properties,
            } => {
                println!("Node: {} ({})", id, label.as_deref().unwrap_or("no label"));
                println!("  Type: {node_type:?}");
                if let Some(pos) = &properties.position {
                    println!("  Position: {pos:?}");
                }
            }
            GraphEvent::AddEdge {
                from, to, label, ..
            } => {
                println!("Edge: {from} -> {to}");
                if let Some(lbl) = label {
                    println!("  Label: {lbl}");
                }
            }
            GraphEvent::BatchStart => println!("--- Batch Start ---"),
            GraphEvent::BatchEnd => println!("--- Batch End ---"),
            _ => {}
        }
    }
}

fn demo_plantuml_events() {
    let plantuml_content = r#"
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

    match plantuml::parse(plantuml_content) {
        Ok(events) => {
            for event in &events {
                match event {
                    GraphEvent::SetLayout { layout_type, .. } => {
                        println!("Layout: {layout_type:?}");
                    }
                    GraphEvent::AddNode {
                        id,
                        label,
                        node_type,
                        properties,
                    } => {
                        println!(
                            "Participant: {} ({})",
                            id,
                            label.as_deref().unwrap_or("no label")
                        );
                        println!("  Type: {node_type:?}");
                        if let Some(pos) = &properties.position {
                            println!("  Position: {pos:?}");
                        }
                    }
                    GraphEvent::AddEdge {
                        id,
                        from,
                        to,
                        edge_type,
                        label,
                        ..
                    } => {
                        println!("Message {id}: {from} -> {to}");
                        println!("  Type: {edge_type:?}");
                        if let Some(lbl) = label {
                            println!("  Text: {lbl}");
                        }
                    }
                    GraphEvent::UpdateNode { id, properties, .. } => {
                        println!("Update {}: {:?}", id, properties.custom);
                    }
                    GraphEvent::BatchStart => println!("--- Batch Start ---"),
                    GraphEvent::BatchEnd => println!("--- Batch End ---"),
                    _ => {}
                }
            }
        }
        Err(e) => eprintln!("Parse error: {e}"),
    }
}
