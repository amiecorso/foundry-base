# foundry-base

A fork of [foundry-rs/foundry](https://github.com/foundry-rs/foundry) that adds **Base custom precompile support** to `forge` and `anvil`, so Solidity tests can exercise the real Rust precompiles used by [base/base](https://github.com/base/base) without leaving the foundry workflow.

Two layers of Base precompiles are supported:

1. **Protocol-level** — RIP-7212 P256VERIFY, BLS12-381 (G1 MSM, G2 MSM, pairing), BN254 pairing, and MODEXP with Base's per-hardfork input-size limits.
2. **App-layer** (active from `Beryl`) — TokenFactory, PolicyRegistry, ActivationRegistry, B20Token, B20Stablecoin. Vendored verbatim from base/base@`feat/b20stablecoin` plus a small adapter that wires their storage abstraction onto foundry's revm.

The result: `forge test --fork-url https://rpc.vibes.base.org/` runs your Solidity tests with the real Base precompile logic executing locally in revm. No mocks, no remote-delegation hacks.

---

## Quick start

```bash
# Build forge with Base support
cd /path/to/foundry-base
cargo build --release -p forge --bin forge
# Resulting binary: target/release/forge

# (Optional) install it system-wide
cargo install --path crates/forge --force
```

Enable Base mode in your project's `foundry.toml`:

```toml
[profile.default]
base = true
# base_hardfork = "beryl"   # Optional; Beryl is the default
```

Or auto-detected when forking a known Base chain:

| Chain | ID | Auto-enables `base` |
|---|---|---|
| Base mainnet | 8453 | ✅ |
| Base Sepolia | 84532 | ✅ |
| Base Zeronet | 763360 | ✅ |
| Base Devnet | 1337 | ✅ |
| Vibenet | 84,538,453 | ✅ |

That's it. `forge test --fork-url https://rpc.vibes.base.org/` now sees the real precompiles.

---

## What's different from upstream foundry

Everything lives on the `base-precompiles` branch. Commit map:

| Commit | What |
|---|---|
| `b272ce336` | Upstream foundry-rs/foundry baseline |
| `b853dabfa` | Add `BaseUpgrade` enum + `base/` module skeleton |
| `9512d0f7e` | Wire protocol-level precompiles (P256/BLS12-381/BN254/MODEXP) into `NetworkConfigs` |
| `205a97143` | Test fixup |
| `9d1cea143` | Phase-1 reconnaissance verdict (see `PHASE1_RECON.md`) |
| `e6ebcac67` | Vendor `base-precompile-storage`, `base-precompile-macros`, `base-common-precompiles` |
| `28db4ed45` | Wire app-layer precompiles into `NetworkConfigs::inject_precompiles()` |
| `7c1b51387` | Vibenet `TokenFactory.createToken` integration test |
| `646c6bf72` | Align default hardfork with `LATEST = Beryl`, add vibenet chain ID |

Files added:
```
crates/evm/networks/src/base/                       # protocol-level wiring + BaseUpgrade enum
crates/evm/networks/src/base/app_precompiles.rs     # app-layer registration
crates/evm/networks/vendor/precompile-storage/      # vendored crate (with adapter)
crates/evm/networks/vendor/precompile-macros/       # vendored crate
crates/evm/networks/vendor/base-common-precompiles/ # vendored crate (trimmed)
PHASE1_RECON.md                                     # design doc for the adapter
README-BASE.md                                      # this file
```

Files modified: `crates/evm/networks/src/lib.rs`, `crates/evm/networks/Cargo.toml`, `Cargo.toml`, `crates/forge/tests/cli/precompiles.rs`.

---

## How it works

### Protocol-level precompiles

Thin Rust wrappers around `revm::precompile::{secp256r1, bls12_381, bn254, modexp}`. Each wrapper enforces Base's hardfork-specific input-size limit, then delegates to the underlying revm crypto. Per-hardfork dispatch lives in `crates/evm/networks/src/base/precompiles.rs::register_for_hardfork`.

### App-layer precompiles

The harder ones. The strategy: **vendor Rayyan's three crates verbatim**, then write a single adapter file so their storage abstraction talks to foundry's revm instead of reth's.

```
base/crates/common/precompile-storage     →  foundry-base/crates/evm/networks/vendor/precompile-storage
base/crates/common/precompile-macros      →  foundry-base/crates/evm/networks/vendor/precompile-macros
base/crates/common/precompiles            →  foundry-base/crates/evm/networks/vendor/base-common-precompiles
```

The only file that diverges from upstream is the adapter (`crates/evm/networks/vendor/precompile-storage/src/evm.rs`'s `EvmPrecompileStorageProvider`), which implements `PrecompileStorageProvider` against `alloy_evm::EvmInternals` — foundry's storage API.

Registration happens through foundry's existing `inject_precompiles()` mechanism:

```rust
// crates/evm/networks/src/lib.rs
pub fn inject_precompiles(self, precompiles: &mut PrecompilesMap) {
    if self.base {
        let hardfork = self.base_hardfork.unwrap_or_default();
        base::precompiles::register_for_hardfork(precompiles, hardfork);
        if hardfork.is_at_least_beryl() {
            base::app_precompiles::install_all(precompiles);
        }
    }
}
```

`inject_precompiles()` is called by foundry's EVM factory (`crates/evm/core/src/evm.rs`) and anvil's executor (`crates/anvil/src/eth/backend/executor.rs`) every time a fresh EVM is constructed. So precompiles end up in both `forge test` and `anvil` execution paths automatically.

### The one compromise

The vendored storage trait has a `checkpoint`/`commit`/`revert` trio (introduced in `alloy-evm` 0.27). Foundry pins 0.26.3, which doesn't expose those methods. Audit confirmed exactly one Base precompile uses them — `TokenFactory::create_token`, as a defensive double-guard around state mutations that revm's frame-level rollback already covers. The adapter null-ops the trio with an in-source explanation and an upgrade path.

If a future Rayyan precompile uses nested checkpoint-then-fail semantics, the trigger is to bump foundry's `alloy-evm` dependency to 0.27.x. Everything else is 1:1.

See [`PHASE1_RECON.md`](./PHASE1_RECON.md) for the full method-by-method trait surface mapping.

---

## Maintaining the fork

Two re-sync axes: upstream foundry, and upstream Base precompiles. They're independent.

### Re-syncing Base precompiles (when Rayyan ships changes)

This is the common case. Estimated 15-30 minutes per cycle.

```bash
# 1. Fetch latest from base/base
cd /Users/amiecorso/base
git fetch origin
git log origin/feat/b20stablecoin -- crates/common/precompiles/ \
                                     crates/common/precompile-storage/ \
                                     crates/common/precompile-macros/

# 2. For each changed file, copy into the vendor dir
cd /Users/amiecorso/foundry-base
git checkout -b sync/base-precompiles-YYYY-MM-DD base-precompiles

# Example: a new precompile file appeared
git -C /Users/amiecorso/base show origin/feat/b20stablecoin:crates/common/precompiles/src/new_thing.rs \
  > crates/evm/networks/vendor/base-common-precompiles/src/new_thing.rs
# Update mod.rs to export it
# Add a registration call in crates/evm/networks/src/base/app_precompiles.rs

# 3. Build and let cargo tell you what broke
cargo check -p foundry-evm-networks
# Fix any import drift or trait surface drift
# Most syncs: nothing breaks. The adapter trait surface has been stable.

# 4. Run the smoke tests
cargo test --release -p forge --test cli -- precompiles::

# 5. Commit and merge
git add -A
git commit -m "sync: pull base precompile changes from <upstream-sha>"
git checkout base-precompiles && git merge --ff-only sync/base-precompiles-YYYY-MM-DD
```

**When you'd need more than 30 minutes:**
- Rayyan refactors the `PrecompileStorageProvider` trait surface. Then the adapter (`crates/evm/networks/vendor/precompile-storage/src/evm.rs`) needs corresponding updates. AI agent can handle the mechanical parts.
- Rayyan adds a precompile that needs `checkpoint`/`commit`/`revert` for real. Trigger to bump foundry's `alloy-evm` to 0.27 and implement the trio against the real journal API.
- Rayyan introduces a new crate that the precompiles depend on. Vendor it under `crates/evm/networks/vendor/` and add to the workspace.

### Re-syncing on upstream foundry

Less frequent. Foundry releases every few weeks.

```bash
cd /Users/amiecorso/foundry-base
git fetch upstream
git checkout base-precompiles
git rebase upstream/master
# Resolve any conflicts (typically only in crates/evm/networks/src/lib.rs)
# Re-run tests
cargo test --release -p forge --test cli -- precompiles::
```

If foundry refactors `NetworkConfigs`, `inject_precompiles`, or the precompile API itself, the wiring layer (`crates/evm/networks/src/lib.rs` + `base/app_precompiles.rs`) needs corresponding changes. The vendored Rust precompiles themselves should be unaffected since they only depend on revm + alloy.

---

## Caveats

1. **Vibenet starts empty.** No activation registry entries set by default. Tests must `vm.prank(ACTIVATION_ADMIN)` and activate the features they need. The integration test in `crates/forge/tests/cli/precompiles.rs::base_token_factory_integration` shows the pattern.

2. **Network dependency.** Fork tests require outbound HTTPS to vibenet (or wherever you're forking). Mark `#[ignore]` for air-gapped CI.

3. **base-std `Mock*.sol` etching takes precedence.** If you `vm.etch(StdPrecompiles.TOKEN_FACTORY_ADDRESS, type(MockTokenFactory).runtimeCode)` in your `setUp`, foundry uses the mock bytecode, not the real precompile. Disable the etch when you want real execution.

4. **Solidity interface drift.** Some base-std interfaces have drifted from the Rust ABI. Tracked separately in a base-std PR.

5. **You permanently maintain a foundry fork.** No realistic upstream path — foundry-rs is unlikely to accept Base-specific app precompiles. Living on the fork is fine.

---

## Reference: where things live

| Concern | Location |
|---|---|
| Base precompile wiring | `crates/evm/networks/src/base/` |
| Vendored Rust crates | `crates/evm/networks/vendor/` |
| Storage adapter (only file that diverges) | `crates/evm/networks/vendor/precompile-storage/src/evm.rs` |
| `inject_precompiles()` plumbing | `crates/evm/networks/src/lib.rs` |
| Forge tests | `crates/forge/tests/cli/precompiles.rs` |
| Phase-1 design doc | `PHASE1_RECON.md` |
| Upstream foundry README | `README.md` |

## Reference: upstream sources

- foundry: https://github.com/foundry-rs/foundry — branched at `b272ce336`
- Base precompiles: https://github.com/base/base, branch `feat/b20stablecoin`, vendored at the commit reachable from that branch when `e6ebcac67` was authored
- base-std Solidity interfaces: https://github.com/base/base-std (private/internal)
- vibenet RPC: https://rpc.vibes.base.org/ (chain ID 84,538,453)
