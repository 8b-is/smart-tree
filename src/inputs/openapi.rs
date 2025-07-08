//! OpenAPI/Swagger specification input adapter
//! 
//! Transforms API specs into navigable context trees

use super::*;
use async_trait::async_trait;
use anyhow::Result;
use serde_json;

pub struct OpenApiAdapter;

#[async_trait]
impl InputAdapter for OpenApiAdapter {
    fn name(&self) -> &'static str {
        "OpenAPI"
    }
    
    fn supported_formats(&self) -> Vec<&'static str> {
        vec!["openapi", "swagger", "oas", "api"]
    }
    
    async fn can_handle(&self, input: &InputSource) -> bool {
        match input {
            InputSource::Path(path) => {
                if let Some(ext) = path.extension() {
                    let ext = ext.to_string_lossy().to_lowercase();
                    return ext == "yaml" || ext == "yml" || ext == "json";
                }
                false
            }
            InputSource::Url(url) => {
                url.contains("swagger") || 
                url.contains("openapi") ||
                url.ends_with("/api-docs")
            }
            InputSource::Raw { format_hint, .. } => {
                format_hint.as_ref().map(|h| {
                    h == "openapi" || h == "swagger"
                }).unwrap_or(false)
            }
            _ => false,
        }
    }
    
    async fn parse(&self, input: InputSource) -> Result<ContextNode> {
        let spec = match input {
            InputSource::Path(path) => {
                let content = std::fs::read_to_string(&path)?;
                if path.extension().map(|e| e == "yaml" || e == "yml").unwrap_or(false) {
                    serde_yaml::from_str(&content)?
                } else {
                    serde_json::from_str(&content)?
                }
            }
            InputSource::Url(url) => {
                let response = reqwest::get(url).await?;
                response.json::<serde_json::Value>().await?
            }
            InputSource::Raw { data, .. } => {
                serde_json::from_slice(&data)?
            }
            _ => anyhow::bail!("Invalid input for OpenAPI adapter"),
        };
        
        self.parse_openapi_spec(&spec)
    }
}

impl OpenApiAdapter {
    fn parse_openapi_spec(&self, spec: &serde_json::Value) -> Result<ContextNode> {
        let title = spec.get("info")
            .and_then(|i| i.get("title"))
            .and_then(|t| t.as_str())
            .unwrap_or("API");
            
        let version = spec.get("info")
            .and_then(|i| i.get("version"))
            .and_then(|v| v.as_str())
            .unwrap_or("1.0.0");
        
        let mut root = ContextNode {
            id: "api_root".to_string(),
            name: format!("{} v{}", title, version),
            node_type: NodeType::ApiSchema,
            quantum_state: None,
            children: vec![],
            metadata: spec.get("info").cloned().unwrap_or_default(),
            entanglements: vec![],
        };
        
        // Parse paths
        if let Some(paths) = spec.get("paths").and_then(|p| p.as_object()) {
            let mut path_nodes = Vec::new();
            
            for (path, methods) in paths {
                let mut path_node = ContextNode {
                    id: path.clone(),
                    name: path.clone(),
                    node_type: NodeType::ApiEndpoint,
                    quantum_state: None,
                    children: vec![],
                    metadata: serde_json::json!({}),
                    entanglements: vec![],
                };
                
                // Parse methods
                if let Some(methods_obj) = methods.as_object() {
                    for (method, details) in methods_obj {
                        if ["get", "post", "put", "delete", "patch", "head", "options"].contains(&method.as_str()) {
                            let operation_id = details.get("operationId")
                                .and_then(|o| o.as_str())
                                .unwrap_or(method);
                                
                            let mut method_node = ContextNode {
                                id: format!("{}_{}", path, method),
                                name: format!("{} {}", method.to_uppercase(), operation_id),
                                node_type: NodeType::ApiEndpoint,
                                quantum_state: Some(self.calculate_endpoint_quantum_state(method, details)),
                                children: vec![],
                                metadata: details.clone(),
                                entanglements: self.find_endpoint_entanglements(path, method, details),
                            };
                            
                            // Add parameter nodes
                            if let Some(params) = details.get("parameters").and_then(|p| p.as_array()) {
                                for param in params {
                                    let param_name = param.get("name")
                                        .and_then(|n| n.as_str())
                                        .unwrap_or("param");
                                    
                                    method_node.children.push(ContextNode {
                                        id: format!("{}_{}_{}", path, method, param_name),
                                        name: format!("param: {}", param_name),
                                        node_type: NodeType::ApiSchema,
                                        quantum_state: None,
                                        children: vec![],
                                        metadata: param.clone(),
                                        entanglements: vec![],
                                    });
                                }
                            }
                            
                            path_node.children.push(method_node);
                        }
                    }
                }
                
                path_nodes.push(path_node);
            }
            
            // Group by path segments
            root.children.push(ContextNode {
                id: "endpoints".to_string(),
                name: "Endpoints".to_string(),
                node_type: NodeType::Directory,
                quantum_state: None,
                children: self.organize_by_path_segments(path_nodes),
                metadata: serde_json::json!({}),
                entanglements: vec![],
            });
        }
        
        // Parse schemas/components
        if let Some(components) = spec.get("components").or_else(|| spec.get("definitions")) {
            root.children.push(self.parse_schemas(components)?);
        }
        
        Ok(root)
    }
    
    fn calculate_endpoint_quantum_state(&self, method: &str, details: &serde_json::Value) -> QuantumState {
        // Calculate quantum properties based on endpoint characteristics
        let complexity = details.get("parameters")
            .and_then(|p| p.as_array())
            .map(|a| a.len())
            .unwrap_or(0) as f64;
            
        let has_security = details.get("security").is_some();
        
        QuantumState {
            amplitude: match method {
                "get" => 0.9,     // READ operations are stable
                "post" => 0.7,    // CREATE operations
                "put" => 0.6,     // UPDATE operations
                "delete" => 0.5,  // DELETE operations are disruptive
                _ => 0.8,
            },
            frequency: 1.0 + complexity, // More params = higher frequency
            phase: if has_security { std::f64::consts::PI / 2.0 } else { 0.0 },
            collapse_probability: if method == "delete" { 0.9 } else { 0.3 },
        }
    }
    
    fn find_endpoint_entanglements(&self, path: &str, _method: &str, details: &serde_json::Value) -> Vec<Entanglement> {
        let mut entanglements = Vec::new();
        
        // Find schema references
        if let Some(schema_ref) = details.get("requestBody")
            .and_then(|r| r.get("content"))
            .and_then(|c| c.get("application/json"))
            .and_then(|j| j.get("schema"))
            .and_then(|s| s.get("$ref"))
            .and_then(|r| r.as_str()) 
        {
            if let Some(schema_name) = schema_ref.split('/').last() {
                entanglements.push(Entanglement {
                    target_id: format!("schema_{}", schema_name),
                    strength: 0.9,
                    relationship: "uses_schema".to_string(),
                });
            }
        }
        
        // Find related endpoints (same resource)
        let resource = path.split('/').nth(1).unwrap_or("");
        if !resource.is_empty() {
            entanglements.push(Entanglement {
                target_id: format!("resource_{}", resource),
                strength: 0.7,
                relationship: "same_resource".to_string(),
            });
        }
        
        entanglements
    }
    
    fn organize_by_path_segments(&self, paths: Vec<ContextNode>) -> Vec<ContextNode> {
        // For simplicity, return as-is
        // In real implementation, would group by common path prefixes
        paths
    }
    
    fn parse_schemas(&self, components: &serde_json::Value) -> Result<ContextNode> {
        let mut schemas_node = ContextNode {
            id: "schemas".to_string(),
            name: "Schemas".to_string(),
            node_type: NodeType::Directory,
            quantum_state: None,
            children: vec![],
            metadata: serde_json::json!({}),
            entanglements: vec![],
        };
        
        if let Some(schemas) = components.get("schemas").and_then(|s| s.as_object()) {
            for (name, schema) in schemas {
                schemas_node.children.push(ContextNode {
                    id: format!("schema_{}", name),
                    name: name.clone(),
                    node_type: NodeType::ApiSchema,
                    quantum_state: None,
                    children: self.parse_schema_properties(schema),
                    metadata: schema.clone(),
                    entanglements: vec![],
                });
            }
        }
        
        Ok(schemas_node)
    }
    
    fn parse_schema_properties(&self, schema: &serde_json::Value) -> Vec<ContextNode> {
        let mut props = Vec::new();
        
        if let Some(properties) = schema.get("properties").and_then(|p| p.as_object()) {
            for (prop_name, prop_schema) in properties {
                props.push(ContextNode {
                    id: format!("prop_{}", prop_name),
                    name: prop_name.clone(),
                    node_type: NodeType::ApiSchema,
                    quantum_state: None,
                    children: vec![],
                    metadata: prop_schema.clone(),
                    entanglements: vec![],
                });
            }
        }
        
        props
    }
}