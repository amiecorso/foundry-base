//! Base app-layer precompiles (TokenFactory, B20Token, B20Stablecoin,
//! PolicyRegistry, ActivationRegistry).
//!
//! These are the user-facing native contract surfaces introduced by the Base
//! Beryl hardfork. The dispatch logic lives in `base-common-precompiles` (a
//! vendored snapshot of `github.com/base/base@feat/b20stablecoin`); this
//! module is the foundry-side wiring that installs them into a
//! `PrecompilesMap`.

use alloy_evm::precompiles::PrecompilesMap;
use alloy_primitives::{Address, address};
use base_common_precompiles::{
    ActivationRegistry, B20TokenPrecompile, PolicyRegistryPrecompile, TokenFactory,
};

/// Canonical activation-registry admin address used on vibenet and base
/// sepolia/mainnet. Set as the contract that controls feature activation.
pub const ACTIVATION_ADMIN: Address = address!("0xcb00000000000000000000000000000000000000");

/// Install all app-layer precompiles (Beryl-and-later) into `precompiles`.
///
/// Mirrors `BasePrecompiles::install()` from upstream `base-common-precompiles`
/// for the Beryl spec, minus the protocol-precompile additions which foundry
/// installs separately via `crate::base::precompiles::register_for_hardfork`.
pub fn install_all(precompiles: &mut PrecompilesMap) {
    TokenFactory::install(precompiles);
    B20TokenPrecompile::install(precompiles);
    PolicyRegistryPrecompile::install(precompiles);
    ActivationRegistry::install(precompiles, Some(ACTIVATION_ADMIN));
}
