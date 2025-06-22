#[allow(unused)]
#[cfg(all(not(clippy), feature = "nonreproducible"))]
mod methods {
    include!(concat!(env!("OUT_DIR"), "/methods.rs"));
}

// Include Noir contract constants when building
#[allow(unused)]
#[cfg(all(not(clippy), feature = "build"))]
mod noir_constants {
    include!(concat!(env!("OUT_DIR"), "/noir_constants.rs"));
}

#[cfg(all(not(clippy), feature = "nonreproducible", feature = "all"))]
mod metadata {
    pub const CONTRACT1_ELF: &[u8] = crate::methods::CONTRACT1_ELF;
    pub const CONTRACT1_ID: [u8; 32] = sdk::to_u8_array(&crate::methods::CONTRACT1_ID);

    // CONTRACT2 removed - replaced with Noir identity verification
    
    // Noir identity contract constants (UltraHonk backend)
    #[cfg(feature = "build")]
    pub use crate::noir_constants::*;
}

#[cfg(any(clippy, not(feature = "nonreproducible")))]
mod metadata {
    pub const CONTRACT1_ELF: &[u8] =
        contract1::client::tx_executor_handler::metadata::CONTRACT1_ELF;
    pub const CONTRACT1_ID: [u8; 32] = contract1::client::tx_executor_handler::metadata::PROGRAM_ID;

    // CONTRACT2 removed - replaced with Noir identity verification
    
    // Placeholder Noir constants for non-build scenarios
    pub const ZKPASSPORT_IDENTITY_CONTRACT_PATH: &str = "../noir-contracts/zkpassport_identity/target/zkpassport_identity.json";
    pub const ZKPASSPORT_IDENTITY_VERIFICATION_KEY_PATH: &str = "../noir-contracts/zkpassport_identity/target/vk";
    pub const ZKPASSPORT_IDENTITY_CONTRACT_NAME: &str = "zkpassport_identity";
}

pub use metadata::*;
