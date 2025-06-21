use borsh::{io::Error, BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use sdk::RunResult;

#[cfg(feature = "client")]
pub mod client;
#[cfg(feature = "client")]
pub mod indexer;

impl sdk::ZkContract for IdentityContract {
    /// Entry point of the contract's logic
    fn execute(&mut self, calldata: &sdk::Calldata) -> RunResult {
        // Parse contract inputs
        let (action, ctx) = sdk::utils::parse_raw_calldata::<IdentityAction>(calldata)?;

        // Execute the given action
        let res = match action {
            IdentityAction::VerifyIdentity { user, country_code, proof_data } => {
                self.verify_identity(user, country_code, proof_data)?
            },
            IdentityAction::GetVerificationStatus { user } => {
                self.get_verification_status(user)?
            },
            IdentityAction::IsUserAllowed { user } => {
                self.is_user_allowed(user)?
            },
        };

        Ok((res, ctx, vec![]))
    }

    /// Serialize the full identity state on-chain
    fn commit(&self) -> sdk::StateCommitment {
        sdk::StateCommitment(self.as_bytes().expect("Failed to encode Identity state"))
    }
}

impl IdentityContract {
    /// Verify user identity and check they are NOT from US
    pub fn verify_identity(&mut self, user: String, country_code: String, proof_data: Vec<u8>) -> Result<Vec<u8>, String> {
        // Basic proof validation (in real implementation, this would verify ZKPassport SNARK proof)
        if proof_data.len() < 32 {
            return Err("Invalid proof data - too short".to_string());
        }
        
        // Check if country code indicates US citizenship/residency
        let is_us_related = country_code == "USA" || country_code == "US" || country_code == "840"; // ISO country codes
        
        let verification_result = IdentityVerification {
            user: user.clone(),
            country_code: country_code.clone(),
            is_allowed: !is_us_related, // Allow if NOT US-related
            verified_at: self.get_current_timestamp(),
            proof_hash: self.hash_proof(&proof_data),
        };
        
        // Store verification result
        self.verifications.insert(user.clone(), verification_result.clone());
        
        // Update allowed users list
        if verification_result.is_allowed {
            self.allowed_users.insert(user.clone());
        } else {
            self.allowed_users.remove(&user);
        }
        
        let status = if verification_result.is_allowed { "ALLOWED" } else { "BLOCKED" };
        Ok(format!("Identity verified for user {}: {} (Country: {}, Status: {})", 
            user, verification_result.proof_hash, country_code, status).into_bytes())
    }

    /// Get verification status for a user
    pub fn get_verification_status(&self, user: String) -> Result<Vec<u8>, String> {
        match self.verifications.get(&user) {
            Some(verification) => {
                let status = if verification.is_allowed { "ALLOWED" } else { "BLOCKED" };
                Ok(format!("User {}: {} - Country: {}, Verified: {}, Status: {}", 
                    user, verification.proof_hash, verification.country_code, 
                    verification.verified_at, status).into_bytes())
            },
            None => Ok(format!("User {} has not been verified", user).into_bytes())
        }
    }
    
    /// Check if user is allowed (not US citizen/resident)
    pub fn is_user_allowed(&self, user: String) -> Result<Vec<u8>, String> {
        let is_allowed = self.allowed_users.contains(&user);
        Ok(format!("User {} is {}", user, if is_allowed { "ALLOWED" } else { "NOT ALLOWED" }).into_bytes())
    }
    
    /// Simple timestamp simulation (in real implementation would use block timestamp)
    fn get_current_timestamp(&self) -> u64 {
        // In a real implementation, this would come from block metadata
        1000000 + (self.verifications.len() as u64) // Simple incrementing timestamp
    }
    
    /// Hash proof data for storage (simplified)
    fn hash_proof(&self, proof_data: &[u8]) -> String {
        // Simple hash simulation - in real implementation would use proper cryptographic hash
        format!("proof_{:08x}", proof_data.iter().map(|&b| b as u32).sum::<u32>())
    }
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Debug, Clone, Default)]
pub struct IdentityContract {
    /// Map of user -> their identity verification
    verifications: HashMap<String, IdentityVerification>,
    /// Set of users who are allowed (not US citizens/residents)
    allowed_users: std::collections::HashSet<String>,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Debug, Clone)]
pub struct IdentityVerification {
    pub user: String,
    pub country_code: String,
    pub is_allowed: bool,
    pub verified_at: u64,
    pub proof_hash: String,
}

/// Enum representing possible calls to the identity contract
#[derive(Serialize, Deserialize, BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
pub enum IdentityAction {
    /// Verify user identity with ZKPassport proof
    VerifyIdentity {
        user: String,
        country_code: String,
        proof_data: Vec<u8>,
    },
    /// Get verification status for a user
    GetVerificationStatus {
        user: String,
    },
    /// Check if user is allowed (not US citizen/resident)
    IsUserAllowed {
        user: String,
    },
}

impl IdentityAction {
    pub fn as_blob(&self, contract_name: sdk::ContractName) -> sdk::Blob {
        sdk::Blob {
            contract_name,
            data: sdk::BlobData(borsh::to_vec(self).expect("Failed to encode IdentityAction")),
        }
    }
}

impl IdentityContract {
    pub fn as_bytes(&self) -> Result<Vec<u8>, Error> {
        borsh::to_vec(self)
    }
}

impl From<sdk::StateCommitment> for IdentityContract {
    fn from(state: sdk::StateCommitment) -> Self {
        borsh::from_slice(&state.0)
            .map_err(|_| "Could not decode identity state".to_string())
            .unwrap()
    }
}

// Type aliases for backward compatibility
pub type Contract2 = IdentityContract;
pub type Contract2Action = IdentityAction;
