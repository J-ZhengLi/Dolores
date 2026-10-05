use dolores_mod_runtime::{RecoveryMod, REPAIRED};
use std::time::Instant;
use tokio_util::sync::CancellationToken;
fn main() {
    let start = Instant::now();
    let module = RecoveryMod::compile(REPAIRED).unwrap();
    let compile = start.elapsed().as_micros();
    let start = Instant::now();
    for _ in 0..1000 {
        assert_eq!(module.invoke(1, &CancellationToken::new()).unwrap(), 1);
    }
    println!(
        "wasmi compile_us={compile} calls_1000_us={}",
        start.elapsed().as_micros()
    );
    let start = Instant::now();
    let mut engine = rhai::Engine::new();
    engine
        .set_max_operations(10_000)
        .set_max_string_size(8192)
        .set_max_array_size(64)
        .set_max_map_size(16)
        .set_max_variables(32)
        .set_max_expr_depths(32, 16);
    engine.disable_symbol("eval");
    let ast = engine.compile("category").unwrap();
    let compile = start.elapsed().as_micros();
    let start = Instant::now();
    for _ in 0..1000 {
        let mut scope = rhai::Scope::new();
        scope.push("category", 1_i64);
        assert_eq!(
            engine.eval_ast_with_scope::<i64>(&mut scope, &ast).unwrap(),
            1
        );
    }
    println!(
        "rhai compile_us={compile} calls_1000_us={}",
        start.elapsed().as_micros()
    );
    assert!(engine.eval::<i64>("loop {}").is_err());
    assert!(engine.eval::<String>("read_file(\"sentinel\")").is_err());
    println!("rhai loop bounded; unregistered file access refused; no OS sandbox");
}
