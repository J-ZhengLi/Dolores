//! Capture-bound accessibility targets, shared with the native input broker.
use crate::desktop_input::Input;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Element {
    pub id: String,
    pub name: String,
    pub role: String,
    pub runtime_id: Vec<i32>,
    /// Providers without runtime IDs use explicit property/hit-test checks.
    #[serde(default = "runtime_identity")]
    pub identity: String,
    /// Physical desktop bounds, used only by the broker for revalidation.
    pub bounds: [i32; 4],
    /// Centre in the associated screenshot's pixel space.
    pub x: u32,
    pub y: u32,
}
fn runtime_identity() -> String {
    "runtime".into()
}

/// Resolve only a reference returned in this observation; never search by name.
pub fn resolve(value: &Value, observation: &Value) -> Result<(Input, Option<Element>), String> {
    let mut action = value.clone();
    let object = action
        .as_object_mut()
        .ok_or("Desktop arguments must be an object.")?;
    object.remove("capture");
    if object
        .remove("consequential")
        .is_some_and(|v| !v.is_boolean())
    {
        return Err("consequential must be a boolean.".into());
    }
    let element = if let Some(id) = object.remove("target") {
        if !matches!(
            object.get("operation").and_then(Value::as_str),
            Some("click" | "doubleClick" | "scroll")
        ) || object.contains_key("x")
            || object.contains_key("y")
        {
            return Err("Use target OR x,y for click/doubleClick/scroll. Observe again if the control changed; no input dispatched.".into());
        }
        let element: Element = observation["accessibility"]["elements"]
            .as_array().and_then(|elements| elements.iter().find(|e| e["id"] == id && id.is_string()))
            .cloned().and_then(|e| serde_json::from_value(e).ok())
            .ok_or("Unknown control target. Observe again and use a target ID from that capture; no input dispatched.")?;
        if !matches!(element.identity.as_str(), "runtime" | "properties")
            || (element.runtime_id.is_empty() && element.identity != "properties")
            || element.runtime_id.len() > 32
            || element.bounds[0] >= element.bounds[2]
            || element.bounds[1] >= element.bounds[3]
        {
            return Err("Invalid control target. Observe again; no input dispatched.".into());
        }
        object.insert("x".into(), element.x.into());
        object.insert("y".into(), element.y.into());
        Some(element)
    } else {
        None
    };
    let input: Input = serde_json::from_value(action)
        .map_err(|_| "Use only fields required by this desktop operation; no input dispatched.")?;
    input.validate(
        observation["width"].as_u64().unwrap_or(0) as u32,
        observation["height"].as_u64().unwrap_or(0) as u32,
    )?;
    Ok((input, element))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn public_form_reference_and_coordinate_fallback_share_input_validation() {
        let observation = json!({"width":800,"height":600,"accessibility":{"elements":[{
            "id":"e1","name":"Save note","role":"button","runtimeId":[42,9],
            "bounds":[100,200,180,240],"x":140,"y":220
        }]}});
        let (input, element) = resolve(
            &json!({"operation":"click","target":"e1","capture":"bound-by-host"}),
            &observation,
        )
        .unwrap();
        assert_eq!(input, Input::Click { x: 140, y: 220 });
        assert_eq!(element.unwrap().name, "Save note");
        assert!(resolve(
            &json!({"operation":"scroll","target":"e1","delta":-120}),
            &observation
        )
        .is_ok());
        for value in [
            json!({"operation":"click","target":"Save note"}),
            json!({"operation":"click","target":"e1","x":1}),
            json!({"operation":"type","target":"e1","text":"no"}),
        ] {
            assert!(resolve(&value, &observation).is_err());
        }
        let unavailable = json!({"width":800,"height":600,"accessibility":{"status":"unavailable","elements":[]}});
        let mut legacy = observation.clone();
        legacy["accessibility"]["elements"][0]["runtimeId"] = json!([]);
        assert!(resolve(&json!({"operation":"click","target":"e1"}), &legacy).is_err());
        legacy["accessibility"]["elements"][0]["identity"] = json!("properties");
        assert_eq!(
            resolve(&json!({"operation":"click","target":"e1"}), &legacy)
                .unwrap()
                .1
                .unwrap()
                .identity,
            "properties"
        );
        assert!(resolve(&json!({"operation":"click","x":20,"y":30}), &unavailable).is_ok());
        assert!(
            resolve(&json!({"operation":"click","target":"e1"}), &unavailable)
                .unwrap_err()
                .contains("Observe")
        );
    }
}
