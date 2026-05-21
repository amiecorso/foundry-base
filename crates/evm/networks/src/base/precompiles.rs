//! Base custom precompiles.
//!
//! Each Base hardfork layers additional precompiles on top of the standard
//! Ethereum set. This module mirrors the dispatch in
//! `base-common-precompiles::BasePrecompiles` by wrapping the underlying
//! revm precompile functions with the input-size limits and gas rules Base
//! enforces.
//!
//! [`register_for_hardfork`] is the only entry point: it mutates a
//! [`PrecompilesMap`] in place so the EVM factory used by `forge` and `anvil`
//! sees the Base-overridden behavior.

use alloy_evm::precompiles::{DynPrecompile, PrecompileInput, PrecompilesMap};
use alloy_primitives::{Address, address};
use revm::precompile::{
    PrecompileError, PrecompileId, bls12_381, bn254, modexp, secp256r1,
};

use super::upgrade::BaseUpgrade;

/// BN254 pairing precompile address (0x08).
pub const BN254_PAIR_ADDRESS: Address = address!("0x0000000000000000000000000000000000000008");

/// BLS12-381 G1 MSM precompile address (0x0c).
pub const BLS12_381_G1_MSM_ADDRESS: Address =
    address!("0x000000000000000000000000000000000000000c");

/// BLS12-381 G2 MSM precompile address (0x0e).
pub const BLS12_381_G2_MSM_ADDRESS: Address =
    address!("0x000000000000000000000000000000000000000e");

/// BLS12-381 pairing precompile address (0x0f).
pub const BLS12_381_PAIRING_ADDRESS: Address =
    address!("0x000000000000000000000000000000000000000f");

/// MODEXP precompile address (0x05).
pub const MODEXP_ADDRESS: Address = address!("0x0000000000000000000000000000000000000005");

/// RIP-7212 P256VERIFY precompile address (0x0100).
pub const P256VERIFY_ADDRESS: Address = address!("0x0000000000000000000000000000000000000100");

/// Trace label for the Base BN254 pairing precompile.
pub const BN254_PAIR_LABEL: &str = "BASE_BN254_PAIRING";
/// Trace label for the Base BLS12-381 G1 MSM precompile.
pub const BLS12_381_G1_MSM_LABEL: &str = "BASE_BLS12_381_G1MSM";
/// Trace label for the Base BLS12-381 G2 MSM precompile.
pub const BLS12_381_G2_MSM_LABEL: &str = "BASE_BLS12_381_G2MSM";
/// Trace label for the Base BLS12-381 pairing precompile.
pub const BLS12_381_PAIRING_LABEL: &str = "BASE_BLS12_381_PAIRING";
/// Trace label for the Base MODEXP precompile.
pub const MODEXP_LABEL: &str = "BASE_MODEXP";
/// Trace label for the Base P256VERIFY precompile.
pub const P256VERIFY_LABEL: &str = "BASE_P256VERIFY";

/// Max input size for BN254 pairing under the Granite hardfork.
pub const GRANITE_BN254_PAIR_MAX: usize = 112_687;
/// Max input size for BN254 pairing under the Jovian hardfork.
pub const JOVIAN_BN254_PAIR_MAX: usize = 81_984;

/// Max input size for BLS12-381 G1 MSM under the Isthmus hardfork.
pub const ISTHMUS_BLS12_381_G1_MSM_MAX: usize = 513_760;
/// Max input size for BLS12-381 G2 MSM under the Isthmus hardfork.
pub const ISTHMUS_BLS12_381_G2_MSM_MAX: usize = 488_448;
/// Max input size for BLS12-381 pairing under the Isthmus hardfork.
pub const ISTHMUS_BLS12_381_PAIRING_MAX: usize = 235_008;

/// Max input size for BLS12-381 G1 MSM under the Jovian hardfork.
pub const JOVIAN_BLS12_381_G1_MSM_MAX: usize = 288_960;
/// Max input size for BLS12-381 G2 MSM under the Jovian hardfork.
pub const JOVIAN_BLS12_381_G2_MSM_MAX: usize = 278_784;
/// Max input size for BLS12-381 pairing under the Jovian hardfork.
pub const JOVIAN_BLS12_381_PAIRING_MAX: usize = 156_672;

/// EIP-7823 input field size cap enforced by MODEXP from Azul onwards.
pub const AZUL_MODEXP_MAX: usize = 1024;

/// P256VERIFY at standard pre-Osaka gas (RIP-7212). Enabled from Fjord.
pub fn p256_verify() -> DynPrecompile {
    DynPrecompile::new(PrecompileId::P256Verify, |input: PrecompileInput<'_>| {
        secp256r1::p256_verify(input.data, input.gas)
    })
}

/// P256VERIFY at Osaka gas (doubled base fee). Enabled from Azul.
pub fn p256_verify_osaka() -> DynPrecompile {
    DynPrecompile::new(PrecompileId::P256Verify, |input: PrecompileInput<'_>| {
        secp256r1::p256_verify_osaka(input.data, input.gas)
    })
}

/// BN254 pairing wrapped with the Granite input-size limit.
pub fn bn254_pair_granite() -> DynPrecompile {
    DynPrecompile::new(PrecompileId::Bn254Pairing, |input: PrecompileInput<'_>| {
        if input.data.len() > GRANITE_BN254_PAIR_MAX {
            return Err(PrecompileError::Bn254PairLength);
        }
        bn254::run_pair(
            input.data,
            bn254::pair::ISTANBUL_PAIR_PER_POINT,
            bn254::pair::ISTANBUL_PAIR_BASE,
            input.gas,
        )
    })
}

/// BN254 pairing wrapped with the tighter Jovian input-size limit.
pub fn bn254_pair_jovian() -> DynPrecompile {
    DynPrecompile::new(PrecompileId::Bn254Pairing, |input: PrecompileInput<'_>| {
        if input.data.len() > JOVIAN_BN254_PAIR_MAX {
            return Err(PrecompileError::Bn254PairLength);
        }
        bn254::run_pair(
            input.data,
            bn254::pair::ISTANBUL_PAIR_PER_POINT,
            bn254::pair::ISTANBUL_PAIR_BASE,
            input.gas,
        )
    })
}

/// BLS12-381 G1 MSM wrapped with the Isthmus input-size limit.
pub fn bls12_381_g1_msm_isthmus() -> DynPrecompile {
    DynPrecompile::new(PrecompileId::Bls12G1Msm, |input: PrecompileInput<'_>| {
        if input.data.len() > ISTHMUS_BLS12_381_G1_MSM_MAX {
            return Err(PrecompileError::Other(
                "G1MSM input length too long for Base input size limitation after the Isthmus hardfork".into(),
            ));
        }
        bls12_381::g1_msm::g1_msm(input.data, input.gas)
    })
}

/// BLS12-381 G2 MSM wrapped with the Isthmus input-size limit.
pub fn bls12_381_g2_msm_isthmus() -> DynPrecompile {
    DynPrecompile::new(PrecompileId::Bls12G2Msm, |input: PrecompileInput<'_>| {
        if input.data.len() > ISTHMUS_BLS12_381_G2_MSM_MAX {
            return Err(PrecompileError::Other(
                "G2MSM input length too long for Base input size limitation after the Isthmus hardfork".into(),
            ));
        }
        bls12_381::g2_msm::g2_msm(input.data, input.gas)
    })
}

/// BLS12-381 pairing wrapped with the Isthmus input-size limit.
pub fn bls12_381_pairing_isthmus() -> DynPrecompile {
    DynPrecompile::new(PrecompileId::Bls12Pairing, |input: PrecompileInput<'_>| {
        if input.data.len() > ISTHMUS_BLS12_381_PAIRING_MAX {
            return Err(PrecompileError::Other(
                "Pairing input length too long for Base input size limitation after the Isthmus hardfork".into(),
            ));
        }
        bls12_381::pairing::pairing(input.data, input.gas)
    })
}

/// BLS12-381 G1 MSM wrapped with the tighter Jovian input-size limit.
pub fn bls12_381_g1_msm_jovian() -> DynPrecompile {
    DynPrecompile::new(PrecompileId::Bls12G1Msm, |input: PrecompileInput<'_>| {
        if input.data.len() > JOVIAN_BLS12_381_G1_MSM_MAX {
            return Err(PrecompileError::Other(
                "G1MSM input length too long for Base input size limitation after the Jovian hardfork".into(),
            ));
        }
        bls12_381::g1_msm::g1_msm(input.data, input.gas)
    })
}

/// BLS12-381 G2 MSM wrapped with the tighter Jovian input-size limit.
pub fn bls12_381_g2_msm_jovian() -> DynPrecompile {
    DynPrecompile::new(PrecompileId::Bls12G2Msm, |input: PrecompileInput<'_>| {
        if input.data.len() > JOVIAN_BLS12_381_G2_MSM_MAX {
            return Err(PrecompileError::Other(
                "G2MSM input length too long for Base input size limitation after the Jovian hardfork".into(),
            ));
        }
        bls12_381::g2_msm::g2_msm(input.data, input.gas)
    })
}

/// BLS12-381 pairing wrapped with the tighter Jovian input-size limit.
pub fn bls12_381_pairing_jovian() -> DynPrecompile {
    DynPrecompile::new(PrecompileId::Bls12Pairing, |input: PrecompileInput<'_>| {
        if input.data.len() > JOVIAN_BLS12_381_PAIRING_MAX {
            return Err(PrecompileError::Other(
                "Pairing input length too long for Base input size limitation after the Jovian hardfork".into(),
            ));
        }
        bls12_381::pairing::pairing(input.data, input.gas)
    })
}

/// MODEXP with the Osaka gas schedule and EIP-7823 input cap (Azul rules).
pub fn modexp_osaka() -> DynPrecompile {
    DynPrecompile::new(PrecompileId::ModExp, |input: PrecompileInput<'_>| {
        modexp::osaka_run(input.data, input.gas)
    })
}

/// Apply Base-specific precompiles for `hardfork` on top of an existing
/// [`PrecompilesMap`].
///
/// Pre-Fjord hardforks (Bedrock, Regolith, Canyon, Ecotone) register no
/// Base-specific precompiles; the underlying Ethereum precompile set is left
/// untouched. Each later hardfork registers the additional precompiles it
/// introduces or replaces the prior version of, mirroring the dispatch in
/// `base-common-precompiles::BasePrecompiles::new_with_spec`.
pub fn register_for_hardfork(precompiles: &mut PrecompilesMap, hardfork: BaseUpgrade) {
    if BaseUpgrade::Fjord.is_enabled_in(hardfork) {
        if BaseUpgrade::Azul.is_enabled_in(hardfork) {
            precompiles
                .apply_precompile(&P256VERIFY_ADDRESS, |_| Some(p256_verify_osaka()));
        } else {
            precompiles.apply_precompile(&P256VERIFY_ADDRESS, |_| Some(p256_verify()));
        }
    }

    if BaseUpgrade::Granite.is_enabled_in(hardfork) {
        if BaseUpgrade::Jovian.is_enabled_in(hardfork) {
            precompiles.apply_precompile(&BN254_PAIR_ADDRESS, |_| Some(bn254_pair_jovian()));
        } else {
            precompiles.apply_precompile(&BN254_PAIR_ADDRESS, |_| Some(bn254_pair_granite()));
        }
    }

    if BaseUpgrade::Isthmus.is_enabled_in(hardfork) {
        if BaseUpgrade::Jovian.is_enabled_in(hardfork) {
            precompiles
                .apply_precompile(&BLS12_381_G1_MSM_ADDRESS, |_| Some(bls12_381_g1_msm_jovian()));
            precompiles
                .apply_precompile(&BLS12_381_G2_MSM_ADDRESS, |_| Some(bls12_381_g2_msm_jovian()));
            precompiles
                .apply_precompile(&BLS12_381_PAIRING_ADDRESS, |_| Some(bls12_381_pairing_jovian()));
        } else {
            precompiles
                .apply_precompile(&BLS12_381_G1_MSM_ADDRESS, |_| Some(bls12_381_g1_msm_isthmus()));
            precompiles
                .apply_precompile(&BLS12_381_G2_MSM_ADDRESS, |_| Some(bls12_381_g2_msm_isthmus()));
            precompiles.apply_precompile(&BLS12_381_PAIRING_ADDRESS, |_| {
                Some(bls12_381_pairing_isthmus())
            });
        }
    }

    if BaseUpgrade::Azul.is_enabled_in(hardfork) {
        precompiles.apply_precompile(&MODEXP_ADDRESS, |_| Some(modexp_osaka()));
    }
}
