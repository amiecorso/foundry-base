# Phase 1 Reconnaissance: Porting Base app precompiles to foundry

**Date:** 2026-05-21
**Branch:** `base-precompiles`
**Verdict:** **TRACTABLE** (with one well-understood adaptation in the storage adapter)

---

## 1. What does the adapter need to provide?

The storage adapter trait surface is defined in
`base/crates/common/precompile-storage/src/provider.rs`:

```text
trait PrecompileStorageProvider {
    fn chain_id(&self) -> u64;
    fn timestamp(&self) -> U256;
    fn beneficiary(&self) -> Address;
    fn block_number(&self) -> u64;

    fn set_code(&mut self, address, code: Bytecode) -> Result<()>;
    fn with_account_info(&mut self, address, f: &mut dyn FnMut(&AccountInfo)) -> Result<()>;

    fn sload(&mut self, address, key)        -> Result<U256>;
    fn tload(&mut self, address, key)        -> Result<U256>;
    fn sstore(&mut self, address, key, value)-> Result<()>;
    fn tstore(&mut self, address, key, value)-> Result<()>;

    fn emit_event(&mut self, address, event: LogData) -> Result<()>;

    fn deduct_gas(&mut self, gas: u64) -> Result<()>;
    fn refund_gas(&mut self, gas: i64);
    fn gas_limit(&self) -> u64;
    fn gas_used(&self) -> u64;
    fn state_gas_used(&self) -> u64;
    fn gas_refunded(&self) -> i64;
    fn reservoir(&self) -> u64;
    fn is_static(&self) -> bool;
    fn caller(&self) -> Address;

    fn checkpoint(&mut self) -> JournalCheckpoint;
    fn checkpoint_commit(&mut self, checkpoint: JournalCheckpoint);
    fn checkpoint_revert(&mut self, checkpoint: JournalCheckpoint);

    fn keccak256(&mut self, data: &[u8]) -> Result<B256>;
}
```

The production implementation `EvmPrecompileStorageProvider`
(`base/crates/common/precompile-storage/src/evm.rs`) wraps
`alloy_evm::EvmInternals` and delegates every method.

### Mapping each method to foundry/revm

| Trait method            | Production impl (Rayyan)                                | Available in foundry's alloy-evm 0.26.3? |
| ----------------------- | ------------------------------------------------------- | ---------------------------------------- |
| `chain_id`              | `internals.chain_id()`                                  | ✅                                       |
| `timestamp`             | `internals.block_env().timestamp()`                     | ✅                                       |
| `beneficiary`           | `internals.block_env().beneficiary()`                   | ✅                                       |
| `block_number`          | `internals.block_env().number()`                        | ✅                                       |
| `set_code`              | `internals.set_code(address, code)`                     | ✅                                       |
| `with_account_info`     | `internals.load_account(addr)` → `data.info`            | ✅                                       |
| `sload`                 | `internals.sload(addr, key)`                            | ✅                                       |
| `tload`                 | `internals.tload(addr, key)`                            | ✅                                       |
| `sstore`                | `internals.sstore(addr, key, value)`                    | ✅                                       |
| `tstore`                | `internals.tstore(addr, key, value)`                    | ✅                                       |
| `emit_event`            | `internals.log(Log { ... })`                            | ✅                                       |
| `caller`                | `PrecompileInput::caller`                               | ✅                                       |
| `is_static`             | `PrecompileInput::is_static`                            | ✅                                       |
| Gas accounting (all)    | local `revm::interpreter::gas::Gas` instance            | ✅                                       |
| `checkpoint`            | `internals.checkpoint()`                                | ❌ added in alloy-evm 0.27.0             |
| `checkpoint_commit`     | `internals.checkpoint_commit()`                         | ❌ added in alloy-evm 0.27.0             |
| `checkpoint_revert`     | `internals.checkpoint_revert(cp)`                       | ❌ added in alloy-evm 0.27.0             |
| `keccak256`             | trait default; calls `alloy_primitives::keccak256`      | ✅                                       |

### The only real delta: checkpoint methods

`alloy-evm 0.26.3` (foundry) exposes the journal indirectly but does **not**
have `checkpoint`/`commit`/`revert` on the `EvmInternals` wrapper.
`alloy-evm 0.27.x` (Base) added them.

**Audit of checkpoint usage in the whole app-precompile codebase:**

```
$ git grep -E "(\.checkpoint\(\)|checkpoint\.commit\(\)|CheckpointGuard|checkpoint_commit|checkpoint_revert)" \
    origin/feat/b20stablecoin -- 'crates/common/precompiles/'

crates/common/precompiles/src/factory/storage.rs: let checkpoint = self.storage.checkpoint();
crates/common/precompiles/src/factory/storage.rs: checkpoint.commit();
```

**Exactly one call site, in `TokenFactory::create_token`**, and its structure
is "open a checkpoint, do bytecode install + init calls, commit at the end
unconditionally; any `?` early-return drops the guard which reverts the
checkpoint."

Because the dispatcher converts `BasePrecompileError` into
`PrecompileOutput::new_reverted(gas, bytes)`, **revm's frame-level rollback
already handles atomicity for the whole precompile call** — the inner
checkpoint is a defensive double-guard. Null-op'ing `checkpoint`,
`checkpoint_commit`, `checkpoint_revert` on the adapter gives identical
observable semantics for the only call site that uses them.

(If we ever vendor a precompile that genuinely needs nested checkpoints, the
right move is to bump foundry's `alloy-evm` to `0.27.x` — it still uses
`revm = 34.0.0`, so the bump is contained.)

---

## 2. What does `#[contract]` generate?

`base/crates/common/precompile-macros/Cargo.toml` declares `proc-macro = true`
and depends only on `syn`, `quote`, `proc-macro2`, `alloy-primitives`. No reth,
no revm, no alloy-evm.

The generated code references types via fully-qualified
`::base_precompile_storage::` paths (e.g. `::base_precompile_storage::Mapping`,
`::base_precompile_storage::StorageCtx`). The storage crate has
`extern crate self as base_precompile_storage;` so this works inside its own
crate tests.

**Implication:** if we vendor the storage crate into the foundry workspace, we
**must keep its package name `base-precompile-storage`** so generated code in
consumer crates still resolves. Same for `base-precompile-macros`.

---

## 3. What's the `PrecompileProvider<CTX>` CTX bound?

```rust
impl<CTX, S> PrecompileProvider<CTX> for BasePrecompiles<S>
where
    S: BasePrecompileSpec,
    CTX: ContextTr<Cfg: Cfg<Spec = S>>,
{ ... }
```

This is the `BasePrecompiles` _whole-set_ provider, used in reth's evm
factory. **We do not need it.** Foundry already has its own evm factory and
exposes a `PrecompilesMap` to mutate. Each individual app precompile installs
via:

```rust
TokenFactory::install(&mut precompiles);        // PrecompilesMap
B20TokenPrecompile::install(&mut precompiles);
PolicyRegistryPrecompile::install(&mut precompiles);
ActivationRegistry::install(&mut precompiles, activation_admin_address);
```

…where each `install` is just:

```rust
precompiles.extend_precompiles(once((ADDRESS, base_precompile!("Name", |ctx, calldata| {
    Storage::new(ctx).dispatch(ctx, &calldata)
}))));
```

`base_precompile!` reduces to `alloy_evm::precompiles::DynPrecompile::new_stateful(...)`,
which is exactly the API foundry's existing Celo and protocol precompiles use.

So the dispatch wiring is a one-line call from foundry's
`NetworkConfigs::inject_precompiles`. We can skip `BasePrecompiles`,
`BasePrecompileSpec`, `provider.rs`, and `spec.rs` from the precompiles crate
entirely.

---

## 4. What does the precompile do that's hard from foundry?

| Concern                              | Mechanism                                                                                 | Hard? |
| ------------------------------------ | ----------------------------------------------------------------------------------------- | ----- |
| Install bytecode at derived address  | `internals.set_code(addr, Bytecode)` — straight revm journal write                        | No    |
| Emit logs                            | `internals.log(Log { ... })` — straight revm journal log                                  | No    |
| Cross-precompile calls               | Direct `OtherStorage::new(ctx)` calls within the same Rust process — no recursive `CALL`  | No    |
| Activation gate                      | First step in every dispatch is `ActivationRegistryStorage::ensure_activated(...)` — sload | No    |
| Static-call detection                | `PrecompileInput::is_static` flag                                                         | No    |

`TokenFactory::create_token` flow:

1. `ActivationRegistry::ensure_activated(TOKEN_FACTORY)` — sload + revert if not active.
2. Validate calldata, derive `(token_address, _) = variant.compute_address(...)`.
3. Check `with_account_info(token).is_empty_code_hash()` — revert if already deployed.
4. Open checkpoint (null-op in our adapter, see §1).
5. `set_code(token, Bytecode::new_legacy([0xef]))` — stub.
6. `B20TokenStorage::from_address(token, storage).initialize(name, symbol, cap, caps)`.
7. Emit `TokenCreated` event.
8. Run `initCalls` (each is a sub-dispatch on the same `B20Token` storage handle).
9. Commit checkpoint (null-op).
10. Return token address.

Every step is a direct revm primitive. No reth-specific code path.

---

## 5. Verdict and porting plan

**TRACTABLE.** The three crates are essentially revm-native already; the only
real-world delta against foundry is the missing `checkpoint` API on `alloy-evm
0.26.3`, which has a defensible null-op workaround verified by code audit.

### Porting plan (executed in Phase 2)

1. **Vendor as workspace members**, keep original package names:
   - `crates/evm/networks/vendor/base-precompile-storage/`
   - `crates/evm/networks/vendor/base-precompile-macros/` (proc-macro crate)
   - `crates/evm/networks/vendor/base-common-precompiles/`
2. **Adapt only `precompile-storage/src/evm.rs`** for the alloy-evm 0.26.3 API:
   - Null-op `checkpoint`, `checkpoint_commit`, `checkpoint_revert` (return a
     dummy `JournalCheckpoint`).
   - Drop `internals_mut` calls that aren't in 0.26.3 if any (likely none —
     EvmInternals owns the methods, not a wrapper).
3. **Drop from `common-precompiles`**: `provider.rs` (`BasePrecompiles`), `spec.rs`
   (`BasePrecompileSpec`), `bn254_pair.rs`/`bls12_381.rs` (already in foundry's
   `crates/evm/networks/src/base/precompiles.rs`), tests, benches.
4. **Stub or vendor `base-common-chains`'s `BaseUpgrade`** — only needed if we
   keep `provider.rs`/`spec.rs`. Since we drop them, no stub needed.
5. **Pin `iso_currency`** — used in one file (`b20_stablecoin/storage.rs`).
   Currency validation; only matters if test exercises the stablecoin path,
   which our minimal createToken-of-default-B20 does not. Can be feature-gated
   off or kept.
6. **Wire into `NetworkConfigs::inject_precompiles`**:
   ```rust
   if self.base {
       base::precompiles::register_for_hardfork(precompiles, hardfork);
       // NEW:
       TokenFactory::install(precompiles);
       B20TokenPrecompile::install(precompiles);
       PolicyRegistryPrecompile::install(precompiles);
       ActivationRegistry::install(precompiles, Some(ACTIVATION_ADMIN));
       // (We may want to gate on hardfork >= Beryl, mirroring upstream.)
   }
   ```
7. **Fix `StdPrecompiles.sol`** in base-std (separate repo, separate commit) —
   PolicyRegistry address typo `0xb000...0001` → `0xb030...0000`.
8. **Add vibenet integration test** in `crates/forge/tests/cli/precompiles.rs`.

### Estimated effort

- Vendor + adapt evm.rs + delete unused: ~1 hour mechanical.
- Get the workspace to compile: ~1–2 hours debugging name/path/feature mismatches.
- Wire into foundry + first compile pass: ~30 min.
- Vibenet integration test + debug: ~1 hour.

**Total: ~4 hours.** Stretch to 6 if alloy-evm 0.26.3 has any unexpected API drift.

---

## Risks and mitigations

| Risk                                                                | Mitigation                                                                                                    |
| ------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------- |
| `revm-context-interface` `GasParams::default()` not in foundry's 14.x | Already verified present at line 92 of `revm-context-interface-14.0.0/src/cfg/gas_params.rs`.                  |
| Vendored proc-macro doesn't see `::base_precompile_storage::` path  | Vendor storage crate with its **original package name** preserved. `extern crate self as ...;` still works.   |
| TokenFactory not activated on vibenet                                | Test calls `ActivationRegistry.activate(TOKEN_FACTORY)` from the activation admin via `vm.prank` if needed.    |
| Atomicity bug from null-op checkpoint                                | Only one call site uses checkpoint; revm frame revert covers it. If we hit a corner case, bump alloy-evm to 0.27. |
| Vendored code drift over time                                       | Out of scope for this task; tracked separately.                                                               |

---

**Proceeding to Phase 2.**
