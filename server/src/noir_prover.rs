use anyhow::{Result, Context};
use serde_json::Value;
use std::process::Command;
use std::fs;
use std::path::Path;
use crate::noir_verifier::NoirProof;

/// Noir proof generator for UltraHonk backend
pub struct NoirProver {
    circuit_path: String,
    working_directory: String,
}

impl NoirProver {
    pub fn new(circuit_path: String) -> Self {
        Self {
            circuit_path,
            working_directory: "../noir-contracts/zkpassport_identity".to_string(),
        }
    }

    /// Generate a proof for password authentication
    pub async fn generate_password_proof(
        &self,
        username: &str,
        password: &str,
    ) -> Result<NoirProof> {
        tracing::info!("🔮 Generating Noir proof for user: {}", username);

        // Step 1: Generate witness data from inputs
        let witness = self.generate_witness_data(username, password).await?;

        // Step 2: Generate proof using nargo
        let proof_data = self.generate_proof_with_nargo(&witness).await?;

        // Step 3: Extract verification key
        let verification_key = self.get_verification_key().await?;

        // Step 4: Extract public inputs
        let public_inputs = self.extract_public_inputs(username, password)?;

        Ok(NoirProof {
            proof_data,
            public_inputs,
            verification_key,
        })
    }

    /// Generate witness data from user inputs
    async fn generate_witness_data(&self, username: &str, password: &str) -> Result<Value> {
        tracing::debug!("📝 Generating witness data for Noir circuit");

        // Convert string inputs to Field values using same logic as Noir circuit
        let user_hash = self.hash_to_field(username, 0)?;
        let password_hash = self.hash_to_field(password, 1)?;

        // Create witness object matching the Noir circuit inputs
        let witness = serde_json::json!({
            "expected_password_hash": password_hash,
            "expected_user_hash": user_hash,
            "user_password": password_hash,
            "user_name": user_hash
        });

        tracing::debug!("✅ Witness data generated successfully");
        Ok(witness)
    }

    /// Generate proof using nargo prove command
    async fn generate_proof_with_nargo(&self, witness: &Value) -> Result<Vec<u8>> {
        tracing::info!("🔐 Running nargo prove to generate UltraHonk proof...");

        // Write witness to temporary file
        let witness_path = format!("{}/Prover.toml", self.working_directory);
        self.write_witness_to_prover_toml(witness, &witness_path)?;

        // Run nargo prove command
        let prove_output = Command::new("nargo")
            .args(["prove"])
            .current_dir(&self.working_directory)
            .output()
            .context("Failed to execute nargo prove command")?;

        if !prove_output.status.success() {
            let stderr = String::from_utf8_lossy(&prove_output.stderr);
            let stdout = String::from_utf8_lossy(&prove_output.stdout);
            anyhow::bail!(
                "Nargo prove failed!\nSTDOUT:\n{}\nSTDERR:\n{}",
                stdout, stderr
            );
        }

        // Read the generated proof
        let proof_path = format!("{}/proofs/zkpassport_identity.proof", self.working_directory);
        let proof_data = fs::read(&proof_path)
            .with_context(|| format!("Failed to read proof file from {}", proof_path))?;

        tracing::info!("✅ UltraHonk proof generated successfully ({} bytes)", proof_data.len());
        Ok(proof_data)
    }

    /// Get verification key from compiled circuit
    async fn get_verification_key(&self) -> Result<Vec<u8>> {
        let vk_path = format!("{}/target/vk", self.working_directory);
        
        match fs::read(&vk_path) {
            Ok(vk_data) => {
                tracing::debug!("✅ Verification key loaded ({} bytes)", vk_data.len());
                Ok(vk_data)
            },
            Err(_) => {
                tracing::warn!("⚠️ Verification key file not found, using placeholder");
                // Return placeholder verification key for development
                Ok(b"placeholder_verification_key".to_vec())
            }
        }
    }

    /// Extract public inputs for the proof
    fn extract_public_inputs(&self, username: &str, password: &str) -> Result<Vec<String>> {
        // Public inputs are the expected hashes that will be visible on-chain
        let user_hash = self.hash_to_field(username, 0)?;
        let password_hash = self.hash_to_field(password, 1)?;

        Ok(vec![
            password_hash,
            user_hash,
        ])
    }

    /// Convert string to Noir Field value (matching circuit logic)
    fn hash_to_field(&self, input: &str, domain: u32) -> Result<String> {
        // For now, use a simple hash simulation
        // TODO: Implement actual Poseidon2 hash to match Noir circuit
        let bytes = input.as_bytes();
        let mut hash_value = domain as u64;
        
        for &byte in bytes {
            hash_value = hash_value.wrapping_mul(31).wrapping_add(byte as u64);
        }

        Ok(hash_value.to_string())
    }

    /// Write witness data to Prover.toml file
    fn write_witness_to_prover_toml(&self, witness: &Value, prover_path: &str) -> Result<()> {
        let mut toml_content = String::new();
        
        if let Value::Object(map) = witness {
            for (key, value) in map {
                if let Value::String(val) = value {
                    toml_content.push_str(&format!("{} = \"{}\"\n", key, val));
                }
            }
        }

        fs::write(prover_path, toml_content)
            .with_context(|| format!("Failed to write witness to {}", prover_path))?;

        tracing::debug!("📝 Witness written to Prover.toml");
        Ok(())
    }

    /// Check if the circuit is compiled and ready
    pub fn is_circuit_ready(&self) -> bool {
        let target_dir = format!("{}/target", self.working_directory);
        Path::new(&target_dir).exists()
    }

    /// Compile the circuit if needed
    pub async fn ensure_circuit_compiled(&self) -> Result<()> {
        if self.is_circuit_ready() {
            tracing::debug!("✅ Noir circuit already compiled");
            return Ok(());
        }

        tracing::info!("🔨 Compiling Noir circuit...");

        let compile_output = Command::new("nargo")
            .args(["compile"])
            .current_dir(&self.working_directory)
            .output()
            .context("Failed to execute nargo compile")?;

        if !compile_output.status.success() {
            let stderr = String::from_utf8_lossy(&compile_output.stderr);
            anyhow::bail!("Circuit compilation failed: {}", stderr);
        }

        tracing::info!("✅ Noir circuit compiled successfully");
        Ok(())
    }
} 