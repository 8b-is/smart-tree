// JSON Decoder - Convert quantum format to JSON
// Shows how all formats are now just decoders of the quantum stream

use super::{QuantumDecoder, QuantumEntry, TraversalCode};
use std::io::Write;
use anyhow::Result;
use serde_json::{json, Value};

pub struct JsonDecoder {
    root: Value,
    stack: Vec<Value>,
    current_children: Vec<Value>,
}

impl JsonDecoder {
    pub fn new() -> Self {
        Self {
            root: json!(null),
            stack: Vec::new(),
            current_children: Vec::new(),
        }
    }
}

impl QuantumDecoder for JsonDecoder {
    fn init(&mut self, writer: &mut dyn Write) -> Result<()> {
        writeln!(writer, "{{")?;
        writeln!(writer, "  \"format\": \"quantum-decoded\",")?;
        writeln!(writer, "  \"version\": \"1.0\",")?;
        writeln!(writer, "  \"tree\": [")?;
        Ok(())
    }
    
    fn decode_entry(&mut self, entry: &QuantumEntry, writer: &mut dyn Write) -> Result<()> {
        let mut node = json!({
            "name": entry.name,
            "type": if entry.is_dir { "directory" } else { "file" },
        });
        
        if let Some(size) = entry.size {
            node["size"] = json!(size);
        }
        
        if let Some(perms) = entry.perms_delta {
            node["permissions_delta"] = json!(format!("0x{:04x}", perms));
        }
        
        match entry.traversal {
            TraversalCode::Deeper => {
                // Starting a new directory level
                node["children"] = json!([]);
                self.stack.push(node);
            }
            TraversalCode::Back => {
                // Exiting directory level
                if let Some(mut parent) = self.stack.pop() {
                    parent["children"] = json!(self.current_children.clone());
                    self.current_children.clear();
                    self.current_children.push(parent);
                }
            }
            TraversalCode::Same => {
                // Same level
                self.current_children.push(node);
            }
            TraversalCode::Summary => {
                // Summary node
                self.current_children.push(node);
            }
        }
        
        Ok(())
    }
    
    fn finish(&mut self, writer: &mut dyn Write) -> Result<()> {
        // Finalize any remaining stack
        while let Some(mut parent) = self.stack.pop() {
            parent["children"] = json!(self.current_children.clone());
            self.current_children.clear();
            self.current_children.push(parent);
        }
        
        // Write the tree
        let tree_json = serde_json::to_string_pretty(&self.current_children)?;
        write!(writer, "{}", tree_json)?;
        
        writeln!(writer, "\n  ]")?;
        writeln!(writer, "}}")?;
        Ok(())
    }
}