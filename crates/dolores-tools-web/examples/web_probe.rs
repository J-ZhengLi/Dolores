//! Bounded public web probe; prints summary only, never keys or private transcripts.
use dolores_core::{CredentialStore, ToolCall};
use serde_json::json;
use std::sync::Arc;
struct NoKeys;
impl CredentialStore for NoKeys {
    fn descriptor(&self) -> dolores_core::PluginDescriptor {
        dolores_core::PluginDescriptor {
            id: "none",
            kind: "credentials",
            api_version: 1,
        }
    }
    fn read(&self, _: &str) -> Result<Option<String>, String> {
        Ok(None)
    }
    fn write(&self, _: &str, _: &str) -> Result<(), String> {
        Err("read-only probe".into())
    }
    fn delete(&self, _: &str) -> Result<(), String> {
        Ok(())
    }
}
#[tokio::main]
async fn main() {
    let tools = dolores_tools_web::plugins(Default::default(), Arc::new(NoKeys)).unwrap();
    for (index, args) in [
        (0, json!({"query":"rust programming language"})),
        (
            1,
            json!({"url":"https://doc.rust-lang.org/book/ch04-01-what-is-ownership.html"}),
        ),
    ] {
        let call = ToolCall {
            id: format!("probe{index}"),
            name: tools[index].spec().name,
            arguments: args.to_string(),
        };
        let receipt = tools[index]
            .invoke(
                &tools[index].prepare(&call).unwrap(),
                tokio_util::sync::CancellationToken::new(),
            )
            .await
            .expect("live public probe failed");
        let receipt: serde_json::Value = serde_json::from_str(&receipt).unwrap();
        println!(
            "{}",
            json!({"tool":call.name,"sources":receipt["results"].as_array().map(Vec::len),"url":receipt["url"],"textBytes":receipt["text"].as_str().map(str::len),"partial":receipt["truncated"],"untrusted":receipt["untrusted"]})
        );
    }
}
