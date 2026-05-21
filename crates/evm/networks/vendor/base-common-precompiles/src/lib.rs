#![cfg_attr(not(feature = "std"), no_std)]
#![allow(rustdoc::all)]
#![allow(clippy::all)]
#![allow(unused_qualifications)]
#![allow(unused_imports)]
#![allow(dead_code)]
#![allow(clippy::doc_markdown)]
#![allow(rust_2018_idioms)]

extern crate alloc;

mod macros;

mod activation;
pub use activation::{ActivationRegistry, ActivationRegistryStorage, IActivationRegistry};

mod common;
pub use common::{
    Burnable, CAPABILITY_CAP_MUTABLE, CAPABILITY_PAUSABLE, Configurable, Mintable, Pausable,
    Permittable, Policy, PolicyRegistry, Redeemable, Token, TokenAccounting, Transferable,
};

mod b20;
pub use b20::{B20Token, B20TokenPrecompile, B20TokenStorage, IB20};

mod b20_stablecoin;
pub use b20_stablecoin::{
    B20StablecoinPrecompile, B20StablecoinStorage, B20StablecoinToken, IB20Stablecoin,
    StablecoinAccounting,
};

mod factory;
pub use factory::{ITokenFactory, TokenFactory, TokenFactoryStorage, TokenVariant};

mod policy;
pub use policy::{IPolicyRegistry, PolicyHandle, PolicyRegistryPrecompile, PolicyRegistryStorage};
