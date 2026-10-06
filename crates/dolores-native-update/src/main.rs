#[cfg(windows)]
mod launcher;
fn main() {
    #[cfg(windows)]
    let result = launcher::run();
    #[cfg(not(windows))]
    let result: Result<(), String> =
        Err("Reviewed native installation is qualified on Windows only.".into());
    if let Err(error) = result {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
