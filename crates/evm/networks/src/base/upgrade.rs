//! Base network upgrade definitions.
//!
//! Mirrors the [`BaseUpgrade`] enum from `base-common-chains` without pulling in
//! cross-repo dependencies. Only the variants and their chronological ordering
//! are reproduced here: precompile dispatch keys off the variant and the
//! `<=` ordering implied by their declaration order. Timestamp-based
//! resolution from a `ChainConfig` is intentionally omitted because it would
//! require importing the upstream chain-config crates.

use clap::ValueEnum;
use serde::{Deserialize, Serialize};

/// Base network upgrades, in chronological order.
///
/// The order of the variants is meaningful: [`BaseUpgrade::is_enabled_in`]
/// treats earlier variants as activated whenever a later variant is the
/// currently-selected hardfork.
///
/// The default is [`BaseUpgrade::LATEST`] (currently [`BaseUpgrade::Beryl`]).
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize, ValueEnum,
)]
#[serde(rename_all = "kebab-case")]
pub enum BaseUpgrade {
    /// Bedrock.
    Bedrock,
    /// Regolith.
    Regolith,
    /// Canyon.
    Canyon,
    /// Ecotone.
    Ecotone,
    /// Fjord: introduces RIP-7212 P256VERIFY at 0x0100.
    Fjord,
    /// Granite: restricts BN254 pairing input size.
    Granite,
    /// Holocene: precompile set is identical to Granite.
    Holocene,
    /// Isthmus: adds Prague BLS12-381 precompiles with Base-specific input caps.
    Isthmus,
    /// Jovian: tightens the BN254 pairing and BLS12-381 input caps.
    Jovian,
    /// Azul: first Base-specific network upgrade; adopts Osaka MODEXP and
    /// P256VERIFY pricing.
    Azul,
    /// Beryl: second Base-specific network upgrade; adds app-layer
    /// precompiles (TokenFactory, PolicyRegistry, ActivationRegistry, B20).
    #[default]
    Beryl,
}

impl BaseUpgrade {
    /// Latest Base upgrade. Used when no explicit hardfork is requested.
    pub const LATEST: Self = Self::Beryl;

    /// Returns `true` if the upgrade `self` is active when the currently
    /// selected hardfork is `current`.
    ///
    /// Earlier variants in the declaration order are considered active
    /// whenever a later variant is selected.
    pub const fn is_enabled_in(self, current: Self) -> bool {
        (self as u8) <= (current as u8)
    }

    /// Returns `true` if Beryl (app-layer precompiles) is active for this hardfork.
    pub const fn is_at_least_beryl(self) -> bool {
        Self::Beryl.is_enabled_in(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_matches_latest() {
        assert_eq!(BaseUpgrade::default(), BaseUpgrade::LATEST);
        assert_eq!(BaseUpgrade::LATEST, BaseUpgrade::Beryl);
    }

    #[test]
    fn is_enabled_in_orders_chronologically() {
        assert!(BaseUpgrade::Fjord.is_enabled_in(BaseUpgrade::Fjord));
        assert!(BaseUpgrade::Fjord.is_enabled_in(BaseUpgrade::Jovian));
        assert!(BaseUpgrade::Fjord.is_enabled_in(BaseUpgrade::Azul));

        assert!(!BaseUpgrade::Jovian.is_enabled_in(BaseUpgrade::Fjord));
        assert!(!BaseUpgrade::Azul.is_enabled_in(BaseUpgrade::Granite));

        assert!(BaseUpgrade::Bedrock.is_enabled_in(BaseUpgrade::Bedrock));
        assert!(!BaseUpgrade::Regolith.is_enabled_in(BaseUpgrade::Bedrock));
    }
}
