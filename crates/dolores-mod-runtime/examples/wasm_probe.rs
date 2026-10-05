fn main() {
    let start = std::time::Instant::now();
    let module = dolores_mod_runtime::RecoveryMod::compile(dolores_mod_runtime::REPAIRED).unwrap();
    println!("ready_us={}", start.elapsed().as_micros());
    for _ in 0..1000 {
        module
            .invoke(1, &tokio_util::sync::CancellationToken::new())
            .unwrap();
    }
    std::thread::sleep(std::time::Duration::from_secs(2));
}
