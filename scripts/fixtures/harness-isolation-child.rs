// Public, credential-free EH00 probe. Not a production sandbox launcher.
use std::{fs, net::TcpStream, path::Path, time::Duration};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let work = Path::new(&args[1]);
    let allowed_read = fs::read(work.join("input.txt")).is_ok();
    let allowed_write = fs::write(work.join("written.txt"), "probe").is_ok();
    let outside_read = fs::read(&args[2]);
    let outside_write = fs::write(&args[2], "changed");
    let network = TcpStream::connect_timeout(&args[3].parse().unwrap(), Duration::from_secs(2));
    let error = |r: &Result<(), std::io::Error>| r.as_ref().err().and_then(|e| e.raw_os_error()).unwrap_or(0);
    let read_error = outside_read.as_ref().err().and_then(|e| e.raw_os_error()).unwrap_or(0);
    let net_error = network.as_ref().err().and_then(|e| e.raw_os_error()).unwrap_or(0);
    let receipt = format!(
        "{{\"allowedRead\":{allowed_read},\"allowedWrite\":{allowed_write},\"outsideReadError\":{read_error},\"outsideWriteError\":{},\"networkError\":{net_error}}}",
        error(&outside_write)
    );
    if fs::write(work.join("receipt.json"), receipt).is_err() {
        std::process::exit(2);
    }
}
