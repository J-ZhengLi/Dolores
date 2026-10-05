//! Test-only broker: delegate to the qualified native helper, then lose its receipt.
//! Never included in the desktop bundle; only synthetic fixture data is used.
use std::{
    io::{self, BufRead, Read, Write},
    process::{Command, Stdio},
};
fn native(directory: &std::path::Path) -> Command {
    let mut command = Command::new(directory.join("native-desktop-helper.exe"));
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x08000000);
    }
    command
}
fn main() {
    let directory = std::env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .to_owned();
    let mode = std::fs::read_to_string(directory.join("fault-mode.txt")).unwrap_or_default();
    let input = std::env::args().nth(1).as_deref() == Some("--input");
    if !input {
        let mut bytes = Vec::new();
        io::stdin().take(16385).read_to_end(&mut bytes).unwrap();
        if mode == "locked" {
            println!("{{\"ok\":false,\"error\":\"Desktop locked in deterministic fixture. Unlock and capture again.\"}}");
            return;
        }
        let mut child = native(&directory)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();
        child.stdin.take().unwrap().write_all(&bytes).unwrap();
        let output = child.wait_with_output().unwrap();
        io::stdout().write_all(&output.stdout).unwrap();
        return;
    }
    let mut reader = io::stdin().lock();
    let mut request = String::new();
    reader.read_line(&mut request).unwrap();
    let mut child = native(&directory)
        .arg("--input")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    let mut inner = child.stdin.take().unwrap();
    inner.write_all(request.as_bytes()).unwrap();
    let mut output = io::BufReader::new(child.stdout.take().unwrap());
    let mut ready = String::new();
    output.read_line(&mut ready).unwrap();
    print!("{ready}");
    io::stdout().flush().unwrap();
    if serde_json::from_str::<serde_json::Value>(&ready).unwrap()["ready"] != true {
        child.wait().unwrap();
        return;
    }
    let mut ack = String::new();
    reader.read_line(&mut ack).unwrap();
    inner.write_all(ack.as_bytes()).unwrap();
    drop(inner);
    let mut receipt = Vec::new();
    output.read_to_end(&mut receipt).unwrap();
    child.wait().unwrap();
    let value: serde_json::Value = serde_json::from_slice(&receipt).unwrap();
    if value["ok"] == true && ["crash", "stall"].contains(&mode.as_str()) {
        std::fs::write(
            directory.join("effect-applied.json"),
            serde_json::json!({"pid":std::process::id(),"nativeReturned":true}).to_string(),
        )
        .unwrap();
        if mode == "crash" {
            std::process::exit(2);
        }
        std::thread::sleep(std::time::Duration::from_secs(30));
        return;
    }
    io::stdout().write_all(&receipt).unwrap();
}
