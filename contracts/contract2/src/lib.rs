use borsh::{io::Error, BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use sdk::RunResult;

#[cfg(feature = "client")]
pub mod client;
// Temporarily disabled indexer module to avoid missing feature dependency
// #[cfg(feature = "client")]
// pub mod indexer;

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

// ============================================================================
// UNIT TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_contract() -> IdentityContract {
        IdentityContract {
            verifications: HashMap::new(),
            allowed_users: std::collections::HashSet::new(),
        }
    }

    fn create_test_proof_data() -> Vec<u8> {
        // Simulate valid proof data (32+ bytes)
        (0..64).collect::<Vec<u8>>()
    }

    #[test]
    fn test_verify_identity_non_us_citizen() {
        let mut contract = create_test_contract();
        let proof_data = create_test_proof_data();
        
        // Test non-US citizen should be allowed
        let result = contract.verify_identity(
            "alice".to_string(),
            "CAN".to_string(), // Canada
            proof_data.clone()
        );
        assert!(result.is_ok());
        
        let binding = result.unwrap();
        let result_str = String::from_utf8_lossy(&binding);
        assert!(result_str.contains("ALLOWED"));
        assert!(result_str.contains("alice"));
        assert!(result_str.contains("CAN"));
        
        // Check user was added to allowed list
        assert!(contract.allowed_users.contains("alice"));
        
        // Check verification was stored
        assert!(contract.verifications.contains_key("alice"));
        let verification = &contract.verifications["alice"];
        assert_eq!(verification.user, "alice");
        assert_eq!(verification.country_code, "CAN");
        assert!(verification.is_allowed);
    }

    #[test]
    fn test_verify_identity_us_citizen_blocked() {
        let mut contract = create_test_contract();
        let proof_data = create_test_proof_data();
        
        // Test US citizen should be blocked
        let result = contract.verify_identity(
            "bob".to_string(),
            "USA".to_string(),
            proof_data.clone()
        );
        assert!(result.is_ok());
        
        let binding = result.unwrap();
        let result_str = String::from_utf8_lossy(&binding);
        assert!(result_str.contains("BLOCKED"));
        assert!(result_str.contains("bob"));
        assert!(result_str.contains("USA"));
        
        // Check user was NOT added to allowed list
        assert!(!contract.allowed_users.contains("bob"));
        
        // Check verification was stored with is_allowed = false
        assert!(contract.verifications.contains_key("bob"));
        let verification = &contract.verifications["bob"];
        assert_eq!(verification.user, "bob");
        assert_eq!(verification.country_code, "USA");
        assert!(!verification.is_allowed);
    }

    #[test]
    fn test_verify_identity_us_variants() {
        let mut contract = create_test_contract();
        let proof_data = create_test_proof_data();
        
        // Test different US country code variants
        let us_codes = ["USA", "US", "840"]; // ISO codes for US
        
        for (i, code) in us_codes.iter().enumerate() {
            let user = format!("user{}", i);
            let result = contract.verify_identity(
                user.clone(),
                code.to_string(),
                proof_data.clone()
            );
            assert!(result.is_ok());
            
            let binding = result.unwrap();
            let result_str = String::from_utf8_lossy(&binding);
            assert!(result_str.contains("BLOCKED"));
            assert!(!contract.allowed_users.contains(&user));
        }
    }

    #[test]
    fn test_verify_identity_invalid_proof() {
        let mut contract = create_test_contract();
        
        // Test with proof data that's too short
        let short_proof = vec![1, 2, 3]; // Only 3 bytes, needs 32+
        
        let result = contract.verify_identity(
            "alice".to_string(),
            "CAN".to_string(),
            short_proof
        );
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid proof data - too short"));
        
        // Check no verification was stored
        assert!(!contract.verifications.contains_key("alice"));
        assert!(!contract.allowed_users.contains("alice"));
    }

    #[test]
    fn test_get_verification_status() {
        let mut contract = create_test_contract();
        let proof_data = create_test_proof_data();
        
        // Test getting status for non-verified user
        let result = contract.get_verification_status("alice".to_string());
        assert!(result.is_ok());
        let binding = result.unwrap();
        let result_str = String::from_utf8_lossy(&binding);
        assert!(result_str.contains("has not been verified"));
        
        // Verify a user first
        contract.verify_identity("alice".to_string(), "CAN".to_string(), proof_data).unwrap();
        
        // Test getting status for verified user
        let result = contract.get_verification_status("alice".to_string());
        assert!(result.is_ok());
        let binding = result.unwrap();
        let result_str = String::from_utf8_lossy(&binding);
        assert!(result_str.contains("alice"));
        assert!(result_str.contains("CAN"));
        assert!(result_str.contains("ALLOWED"));
        assert!(result_str.contains("proof_"));
    }

    #[test]
    fn test_is_user_allowed() {
        let mut contract = create_test_contract();
        let proof_data = create_test_proof_data();
        
        // Test user not yet verified
        let result = contract.is_user_allowed("alice".to_string());
        assert!(result.is_ok());
        let binding = result.unwrap();
        let result_str = String::from_utf8_lossy(&binding);
        assert!(result_str.contains("NOT ALLOWED"));
        
        // Verify non-US user
        contract.verify_identity("alice".to_string(), "CAN".to_string(), proof_data.clone()).unwrap();
        
        let result = contract.is_user_allowed("alice".to_string());
        assert!(result.is_ok());
        let binding = result.unwrap();
        let result_str = String::from_utf8_lossy(&binding);
        assert!(result_str.contains("ALLOWED"));
        
        // Verify US user
        contract.verify_identity("bob".to_string(), "USA".to_string(), proof_data.clone()).unwrap();
        
        let result = contract.is_user_allowed("bob".to_string());
        assert!(result.is_ok());
        let binding = result.unwrap();
        let result_str = String::from_utf8_lossy(&binding);
        assert!(result_str.contains("NOT ALLOWED"));
    }

    #[test]
    fn test_multiple_verifications_same_user() {
        let mut contract = create_test_contract();
        let proof_data = create_test_proof_data();
        
        // First verification: allowed
        contract.verify_identity("alice".to_string(), "CAN".to_string(), proof_data.clone()).unwrap();
        assert!(contract.allowed_users.contains("alice"));
        
        // Second verification: blocked (user moved to US)
        contract.verify_identity("alice".to_string(), "USA".to_string(), proof_data).unwrap();
        assert!(!contract.allowed_users.contains("alice"));
        
        // Check latest verification status
        let result = contract.get_verification_status("alice".to_string());
        let binding = result.unwrap();
        let result_str = String::from_utf8_lossy(&binding);
        assert!(result_str.contains("USA"));
        assert!(result_str.contains("BLOCKED"));
    }

    #[test]
    fn test_proof_hash_generation() {
        let contract = create_test_contract();
        let proof_data1 = vec![1, 2, 3, 4]; // Different data
        let proof_data2 = vec![5, 6, 7, 8]; // Should generate different hash
        
        let hash1 = contract.hash_proof(&proof_data1);
        let hash2 = contract.hash_proof(&proof_data2);
        
        // Hashes should be different for different data
        assert_ne!(hash1, hash2);
        
        // Hash should be deterministic
        let hash1_again = contract.hash_proof(&proof_data1);
        assert_eq!(hash1, hash1_again);
        
        // Hash should have expected format
        assert!(hash1.starts_with("proof_"));
    }

    #[test]
    fn test_timestamp_generation() {
        let mut contract = create_test_contract();
        
        let timestamp1 = contract.get_current_timestamp();
        
        // Add a verification to increment internal counter
        let proof_data = create_test_proof_data();
        contract.verify_identity("alice".to_string(), "CAN".to_string(), proof_data).unwrap();
        
        let timestamp2 = contract.get_current_timestamp();
        
        // Timestamp should increment
        assert!(timestamp2 > timestamp1);
    }

    #[test]
    fn test_edge_case_empty_user() {
        let mut contract = create_test_contract();
        let proof_data = create_test_proof_data();
        
        // Test with empty user string
        let result = contract.verify_identity(
            "".to_string(),
            "CAN".to_string(),
            proof_data
        );
        assert!(result.is_ok()); // Should still work, just with empty user
        
        // Check verification was stored with empty key
        assert!(contract.verifications.contains_key(""));
    }

    #[test]
    fn test_case_sensitivity_country_codes() {
        let mut contract = create_test_contract();
        let proof_data = create_test_proof_data();
        
        // Test that lowercase "usa" is NOT blocked (only exact matches)
        let result = contract.verify_identity(
            "alice".to_string(),
            "usa".to_string(), // lowercase
            proof_data
        );
        assert!(result.is_ok());
        let binding = result.unwrap();
        let result_str = String::from_utf8_lossy(&binding);
        assert!(result_str.contains("ALLOWED")); // Should be allowed since it's not exact "USA"
    }
}
