fn main() {
    let start = std::time::Instant::now();
    let mut engine = rhai::Engine::new();
    engine
        .set_max_operations(10_000)
        .set_max_string_size(8192)
        .disable_symbol("eval");
    let ast = engine.compile("1").unwrap();
    println!("ready_us={}", start.elapsed().as_micros());
    for _ in 0..1000 {
        engine.eval_ast::<i64>(&ast).unwrap();
    }
    std::thread::sleep(std::time::Duration::from_secs(2));
}
