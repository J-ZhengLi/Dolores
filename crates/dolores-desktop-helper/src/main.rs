use std::io::{self, Read};

#[cfg(windows)]
mod capture;

fn main() {
    let result = (|| -> Result<serde_json::Value, String> {
        let mut bytes = Vec::new();
        io::stdin()
            .take(16385)
            .read_to_end(&mut bytes)
            .map_err(|_| "Observation request unavailable.")?;
        if bytes.len() > 16384 {
            return Err("Observation request exceeds its limit.".into());
        }
        let request = serde_json::from_slice(&bytes).map_err(|_| "Invalid observation request.")?;
        #[cfg(windows)]
        {
            capture::run(request)
        }
        #[cfg(not(windows))]
        {
            let _: serde_json::Value = request;
            Err("Desktop observation is only qualified on Windows.".into())
        }
    })();
    let value = match result {
        Ok(value) => serde_json::json!({"ok":true,"result":value}),
        Err(error) => serde_json::json!({"ok":false,"error":error}),
    };
    println!("{value}");
}
