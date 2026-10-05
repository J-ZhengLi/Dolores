//! Deterministic owned-process cancellation fixture; never included in the app bundle.
use std::io::Read;
fn main() {
    let path = std::env::current_exe().unwrap().with_file_name("stall.pid");
    std::fs::write(path, std::process::id().to_string()).unwrap();
    let mut request = Vec::new();
    std::io::stdin()
        .take(16385)
        .read_to_end(&mut request)
        .unwrap();
    std::thread::sleep(std::time::Duration::from_secs(60));
}
