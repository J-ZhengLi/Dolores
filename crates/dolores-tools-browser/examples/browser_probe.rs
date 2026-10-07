//! Bounded browser probe against an explicitly supplied synthetic URL.
use dolores_core::{ToolCall, ToolPlugin};
use dolores_tools_browser::{Browser, BrowserRuntime};
use tokio_util::sync::CancellationToken;
#[tokio::main]
async fn main() {
    let folder = tempfile::tempdir().unwrap();
    let browser = Browser::new(BrowserRuntime::discover(folder.path().to_path_buf()).unwrap());
    let url = std::env::args().nth(1).expect("Synthetic URL required");
    let r = browser
        .prepare(&ToolCall {
            id: "probe".into(),
            name: "browser".into(),
            arguments: serde_json::json!({"operation":"open","url":url}).to_string(),
        })
        .unwrap();
    match browser.invoke(&r, CancellationToken::new()).await {
        Ok(result) => {
            let value: serde_json::Value = serde_json::from_str(&result).unwrap();
            println!(
                "{}",
                serde_json::json!({"outcome":value["outcome"],"controls":value["controls"].as_array().map(Vec::len)})
            );
        }
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(1);
        }
    }
}
