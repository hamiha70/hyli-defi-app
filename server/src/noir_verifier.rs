use std::sync::Arc;
use anyhow::{Result, Context};
use sdk::{Blob, ContractName, BlobTransaction};
use client_sdk::rest_client::{NodeApiHttpClient, NodeApiClient};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

/// Noir proof verification module for UltraHonk backend integration
pub struct NoirVerifier {
    contract_name: ContractName,
    node_client: Arc<NodeApiHttpClient>,
    verification_stats: Arc<Mutex<VerificationStats>>,
}

pub struct NoirVerifierCtx {
    pub contract_name: ContractName,
    pub node_client: Arc<NodeApiHttpClient>,
}

#[derive(Debug, Clone)]
pub struct NoirProof {
    pub proof_data: Vec<u8>,
    pub public_inputs: Vec<String>,
    pub verification_key: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationStats {
    pub total_proofs_verified: u64,
    pub successful_verifications: u64,
    pub failed_verifications: u64,
    pub average_verification_time_ms: f64,
}

impl Default for VerificationStats {
    fn default() -> Self {
        Self {
            total_proofs_verified: 0,
            successful_verifications: 0,
            failed_verifications: 0,
            average_verification_time_ms: 0.0,
        }
    }
}

impl NoirVerifier {
    pub fn new(ctx: NoirVerifierCtx) -> Self {
        Self {
            contract_name: ctx.contract_name,
            node_client: ctx.node_client,
            verification_stats: Arc::new(Mutex::new(VerificationStats::default())),
        }
    }

    /// Submit a Noir proof to the Hyli blockchain for verification
    pub async fn submit_proof_to_chain(
        &self,
        proof: NoirProof,
        user_identity: String,
    ) -> Result<String> {
        tracing::info!("🔐 Submitting Noir proof to Hyli chain for user: {}", user_identity);

        // Create blob transaction with Noir proof
        let proof_blob = self.create_proof_blob(proof)?;
        let blob_tx = BlobTransaction::new(user_identity.clone(), vec![proof_blob]);

        // Submit transaction to Hyli node
        let tx_hash = self.node_client
            .send_tx_blob(blob_tx)
            .await
            .context("Failed to submit Noir proof transaction to Hyli")?;

        tracing::info!("✅ Noir proof submitted to chain with tx_hash: {}", tx_hash);
        Ok(tx_hash.to_string())
    }

    /// Verify Noir proof locally (before chain submission)
    pub async fn verify_proof_locally(&self, proof: &NoirProof) -> Result<bool> {
        let start_time = std::time::Instant::now();
        
        tracing::info!("🧮 Starting local Noir proof verification...");

        // TODO: Implement actual UltraHonk verification
        // For now, basic validation of proof structure
        let is_valid = self.validate_proof_structure(proof)?;

        let verification_time = start_time.elapsed().as_millis() as f64;
        
        // Update statistics
        let mut stats = self.verification_stats.lock().await;
        stats.total_proofs_verified += 1;
        if is_valid {
            stats.successful_verifications += 1;
        } else {
            stats.failed_verifications += 1;
        }
        
        // Update average verification time
        let total_time = stats.average_verification_time_ms * (stats.total_proofs_verified - 1) as f64 + verification_time;
        stats.average_verification_time_ms = total_time / stats.total_proofs_verified as f64;

        tracing::info!(
            "🔍 Local verification complete: {} (took {}ms)", 
            if is_valid { "VALID" } else { "INVALID" }, 
            verification_time
        );

        Ok(is_valid)
    }

    /// Get verification statistics
    pub async fn get_verification_stats(&self) -> VerificationStats {
        self.verification_stats.lock().await.clone()
    }

    /// Create proof blob for chain submission
    fn create_proof_blob(&self, proof: NoirProof) -> Result<Blob> {
        // Serialize proof data for blockchain storage
        let proof_payload = ProofPayload {
            proof_data: proof.proof_data,
            public_inputs: proof.public_inputs,
            verification_key: proof.verification_key,
            timestamp: chrono::Utc::now().timestamp(),
            proof_type: "ultrahonk".to_string(),
        };

        let serialized_proof = serde_json::to_vec(&proof_payload)
            .context("Failed to serialize Noir proof for blockchain submission")?;

        Ok(Blob {
            contract_name: self.contract_name.clone(),
            data: sdk::BlobData(serialized_proof),
        })
    }

    /// Validate proof structure before verification
    fn validate_proof_structure(&self, proof: &NoirProof) -> Result<bool> {
        // Basic structural validation
        if proof.proof_data.is_empty() {
            tracing::warn!("❌ Invalid proof: empty proof data");
            return Ok(false);
        }

        if proof.verification_key.is_empty() {
            tracing::warn!("❌ Invalid proof: empty verification key");
            return Ok(false);
        }

        if proof.public_inputs.is_empty() {
            tracing::warn!("❌ Invalid proof: no public inputs");
            return Ok(false);
        }

        // Validate proof data size (reasonable bounds)
        if proof.proof_data.len() < 32 || proof.proof_data.len() > 1024 * 1024 {
            tracing::warn!("❌ Invalid proof: proof data size out of bounds ({})", proof.proof_data.len());
            return Ok(false);
        }

        tracing::debug!("✅ Proof structure validation passed");
        Ok(true)
    }
}

/// Proof payload for blockchain storage
#[derive(Serialize, Deserialize)]
struct ProofPayload {
    proof_data: Vec<u8>,
    public_inputs: Vec<String>,
    verification_key: Vec<u8>,
    timestamp: i64,
    proof_type: String,
}

// TODO: Implement actual UltraHonk verification when Hyli provides the integration
// This module provides the foundation for real proof verification 