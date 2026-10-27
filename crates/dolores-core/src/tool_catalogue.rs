use crate::ToolSpec;
use std::collections::HashSet;

pub const DISCOVER_TOOLS: &str = "discover_tools";

pub(crate) struct ToolCatalogue {
    registered: Vec<ToolSpec>,
    active: HashSet<String>,
    deferred: bool,
}
impl ToolCatalogue {
    pub(crate) fn new(specs: &[ToolSpec]) -> Self {
        let deferred = specs.len() > 8;
        let active = specs
            .iter()
            .filter(|s| {
                !deferred
                    || matches!(
                        s.name.as_str(),
                        "read_text_file"
                            | "list_folder"
                            | "search_text"
                            | "edit_text_file"
                            | "create_text_file"
                            | "run_command"
                            | "command_session"
                            | "request_user_input"
                    )
                    || s.name.starts_with("mcp_tool_")
            })
            .map(|s| s.name.clone())
            .collect();
        Self {
            registered: specs.to_vec(),
            active,
            deferred,
        }
    }
    pub(crate) fn specs(&self) -> Vec<ToolSpec> {
        let mut specs: Vec<_> = self
            .registered
            .iter()
            .filter(|s| self.active.contains(&s.name))
            .cloned()
            .collect();
        if self.deferred {
            let available = self
                .registered
                .iter()
                .map(|s| {
                    format!(
                        "{}: {}",
                        s.name,
                        s.description.chars().take(80).collect::<String>()
                    )
                })
                .collect::<Vec<_>>()
                .join("; ");
            specs.push(ToolSpec{name:DISCOVER_TOOLS.into(),description:format!("Load schemas with names or unload unused schemas with unload (1–8 exact names per list). Metadata only: no execution or permission grant. Schemas update next request; unloaded tools can be reloaded. Registered directory: {available}"),parameters:serde_json::json!({"type":"object","properties":{"names":{"type":"array","items":{"type":"string"},"maxItems":8},"unload":{"type":"array","items":{"type":"string"},"maxItems":8}},"minProperties":1,"additionalProperties":false})});
        }
        specs
    }
    pub(crate) fn activate(&mut self, name: &str) {
        if self.registered.iter().any(|s| s.name == name) {
            self.active.insert(name.into());
        }
    }
    pub(crate) fn discover(&mut self, arguments: &str) -> Result<String, String> {
        #[derive(serde::Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Selection {
            #[serde(default)]
            names: Vec<String>,
            #[serde(default)]
            unload: Vec<String>,
        }
        let selection: Selection = serde_json::from_str(arguments)
            .map_err(|_| "Use names: an array of 1–8 exact registered tool names.")?;
        if (selection.names.is_empty() && selection.unload.is_empty())
            || selection.names.len() > 8
            || selection.unload.len() > 8
            || selection.names.iter().any(|n| selection.unload.contains(n))
            || selection
                .names
                .iter()
                .chain(&selection.unload)
                .any(|n| !self.registered.iter().any(|s| &s.name == n))
        {
            return Err("Choose 1–8 names from the advertised tool catalogue; unavailable tools cannot be loaded.".into());
        }
        for n in &selection.names {
            self.activate(n);
        }
        for n in &selection.unload {
            self.active.remove(n);
        }
        Ok(serde_json::json!({"activated":selection.names,"unloaded":selection.unload,"note":"Schemas update next request. Execution still requires the original task access and normal tool review."}).to_string())
    }
}

/// The same initial catalogue is used by context preview and actual sending.
pub fn initial_tool_specs(specs: &[ToolSpec]) -> Vec<ToolSpec> {
    ToolCatalogue::new(specs).specs()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn deferred_catalogue_reduces_schemas_and_loads_only_registered_tools() {
        let all = (0..20)
            .map(|i| ToolSpec {
                name: if i == 0 {
                    "run_command".into()
                } else {
                    format!("optional_{i}")
                },
                description: "Optional fixture".into(),
                parameters: serde_json::json!({"description":"large schema".repeat(100)}),
            })
            .collect::<Vec<_>>();
        let mut catalogue = ToolCatalogue::new(&all);
        assert_eq!(catalogue.specs().len(), 2);
        assert!(
            serde_json::to_vec(&catalogue.specs()).unwrap().len()
                < serde_json::to_vec(&all).unwrap().len() / 3
        );
        assert!(catalogue
            .discover(r#"{"names":["optional_3","missing"]}"#)
            .is_err());
        assert_eq!(
            catalogue.specs().len(),
            2,
            "Invalid batch must not partially activate"
        );
        catalogue.discover(r#"{"names":["optional_3"]}"#).unwrap();
        assert_eq!(
            catalogue
                .specs()
                .iter()
                .filter(|s| s.name == "optional_3")
                .count(),
            1
        );
        assert_eq!(catalogue.specs().len(), 3);
        catalogue.discover(r#"{"unload":["optional_3"]}"#).unwrap();
        assert_eq!(catalogue.specs().len(), 2);
        catalogue.discover(r#"{"names":["optional_3"]}"#).unwrap();
        assert_eq!(catalogue.specs().len(), 3);
    }
}
