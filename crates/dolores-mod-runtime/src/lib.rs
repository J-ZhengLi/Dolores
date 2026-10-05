//! No WASI, imports, memory, tables or brokered effects. ABI 1 is stateless.
use sha2::{Digest, Sha256};
use tokio_util::sync::CancellationToken;
use wasmi::{Config, Engine, Linker, Module, Store, StoreLimits, StoreLimitsBuilder};

pub const SOURCE_LIMIT: usize = 8192;
pub const FUEL: u64 = 10_000;

pub fn digest(source: &str) -> String {
    format!("{:x}", Sha256::digest(source.as_bytes()))
}

/// Only public failure categories enter the VM; private text/keys never do.
/// 0 unknown, 1 output, 2 context, 3 tool budget, 4 denied, 5 interrupted.
/// Result: 0 inspect, 1 explicit continuation, 2 context review,
/// 3 task allowance review, 4 permission review, 5 checkpoint review.
pub struct RecoveryMod {
    engine: Engine,
    module: Module,
    pub digest: String,
}

impl RecoveryMod {
    pub fn compile(source: &str) -> Result<Self, String> {
        if source.is_empty() || source.len() > SOURCE_LIMIT {
            return Err("Mod source must be 1–8192 bytes. Baseline retained.".into());
        }
        let bytes = wat::parse_str(source)
            .map_err(|_| "Invalid Wasm text. Fix the source and test again; baseline retained.")?;
        if bytes.len() > SOURCE_LIMIT {
            return Err("Compiled mod exceeds 8192 bytes. Baseline retained.".into());
        }
        let mut config = Config::default();
        config.consume_fuel(true);
        let engine = Engine::new(&config);
        let module = Module::new(&engine, &bytes[..])
            .map_err(|_| "Unsupported Wasm module. Baseline retained.")?;
        if module.imports().next().is_some() {
            return Err(
                "Mods cannot import files, network, processes, credentials or host functions."
                    .into(),
            );
        }
        let result = Self {
            engine,
            module,
            digest: digest(source),
        };
        // Instantiation checks hidden memories/tables/start and exact hook signature.
        result.invoke(0, &CancellationToken::new())?;
        Ok(result)
    }

    pub fn invoke(&self, category: i32, cancel: &CancellationToken) -> Result<i32, String> {
        if cancel.is_cancelled() {
            return Err("Mod stopped. Baseline and task remain available.".into());
        }
        if !(0..=5).contains(&category) {
            return Err("Unsupported recovery category.".into());
        }
        let limits = StoreLimitsBuilder::new()
            .instances(1)
            .memories(0)
            .tables(0)
            .memory_size(0)
            .table_elements(0)
            .build();
        let mut store: Store<StoreLimits> = Store::new(&self.engine, limits);
        store.limiter(|limits| limits);
        store.set_fuel(FUEL).map_err(|_| "Mod fuel unavailable.")?;
        let linker: Linker<StoreLimits> = Linker::new(&self.engine);
        let instance = linker
            .instantiate(&mut store, &self.module)
            .and_then(|pre| pre.start(&mut store))
            .map_err(|_| "Mod load failed or exceeded fuel/memory limits. Restore the baseline.")?;
        let hook = instance
            .get_typed_func::<i32, i32>(&store, "recovery_hint")
            .map_err(|_| "Mod requires recovery_hint(i32) -> i32, ABI 1.")?;
        let result = hook.call(&mut store, category)
            .map_err(|_| "Mod trapped or exhausted its 10000-operation fuel allowance. Baseline retained; inspect or restore it.")?;
        if cancel.is_cancelled() {
            return Err("Mod stopped. No result was activated.".into());
        }
        if !(0..=5).contains(&result) {
            return Err("Mod returned an unsupported recovery action. Baseline retained.".into());
        }
        Ok(result)
    }
}

pub const BASELINE: &str =
    "(module (func (export \"recovery_hint\") (param i32) (result i32) i32.const 0))";
pub const REPAIRED: &str =
    "(module (func (export \"recovery_hint\") (param i32) (result i32) local.get 0))";

/// Host-owned independent cases; candidates cannot supply or weaken criteria.
pub fn evaluate(module: &RecoveryMod, cancel: &CancellationToken) -> Result<Vec<bool>, String> {
    [1, 4, 0, 5, 2, 3]
        .into_iter()
        .map(|category| module.invoke(category, cancel).map(|hint| hint == category))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn fixed_suite_and_capability_refusal() {
        let token = CancellationToken::new();
        assert!(evaluate(&RecoveryMod::compile(REPAIRED).unwrap(), &token)
            .unwrap()
            .iter()
            .all(|v| *v));
        assert!(!evaluate(&RecoveryMod::compile(BASELINE).unwrap(), &token)
            .unwrap()
            .iter()
            .all(|v| *v));
        for source in [
            "(module (import \"wasi_snapshot_preview1\" \"path_open\" (func)) (func (export \"recovery_hint\") (param i32) (result i32) i32.const 0))",
            "(module (memory 1) (func (export \"recovery_hint\") (param i32) (result i32) i32.const 0))",
            "(module (table 1 funcref) (func (export \"recovery_hint\") (param i32) (result i32) i32.const 0))",
            "(module (func (export \"recovery_hint\") (param i32) (result i32) i32.const 99))",
        ] { assert!(RecoveryMod::compile(source).is_err()); }
    }
    #[test]
    fn runaway_and_stop_are_bounded() {
        let runaway = "(module (func (export \"recovery_hint\") (param i32) (result i32) (loop $x br $x) i32.const 0))";
        let start = std::time::Instant::now();
        assert!(RecoveryMod::compile(runaway).is_err());
        assert!(start.elapsed().as_secs() < 2);
        let module = RecoveryMod::compile(REPAIRED).unwrap();
        let token = CancellationToken::new();
        token.cancel();
        assert!(module.invoke(1, &token).is_err());
    }
}
