# Noir-Hyli Integration Implementation Plan

*Based on insights from microbecode/zkhack reference implementation*

## 🎯 **Key Insights from microbecode/zkhack**

### **What Works in Reference Implementation:**
1. **UltraHonk Backend**: Modern proving system for Noir on Hyli
2. **app-scaffold Pattern**: Proper integration with Hyli node infrastructure  
3. **Module Registration**: Noir contracts registered as Hyli modules
4. **Real Proof Pipeline**: Actual proof generation and verification

### **What's Missing in Our Current Implementation:**
1. **No Hyli Module Registration**: Noir contract is standalone
2. **Mock Proof Generation**: No real UltraHonk integration
3. **No State Indexing**: Noir contract state not queryable
4. **Missing Verifier Module**: No on-chain proof verification

## 🚀 **Implementation Phases**

### **Phase 1: Infrastructure Setup (Following microbecode Pattern)**

#### **1.1 Add UltraHonk Support**
```toml
# noir-contracts/zkpassport_identity/Nargo.toml
[package]
name = "zkpassport_identity"
type = "bin"
authors = [""]

[dependencies]
# Add UltraHonk backend support
noir_ultrahonk = { tag = "v1.0.0-beta.3", git = "https://github.com/noir-lang/noir" }
```

#### **1.2 Update Build System**
```rust
// contracts/build.rs
use std::env;

fn main() {
    // Existing contract1 build
    println!("cargo:rerun-if-changed=contract1/src");
    
    // Add Noir contract compilation
    println!("cargo:rerun-if-changed=../noir-contracts/zkpassport_identity/src");
    
    // Compile Noir contract to UltraHonk
    let noir_output = std::process::Command::new("nargo")
        .args(["compile", "--backend", "ultrahonk"])
        .current_dir("../noir-contracts/zkpassport_identity")
        .output()
        .expect("Failed to compile Noir contract");
        
    if !noir_output.status.success() {
        panic!("Noir compilation failed: {}", String::from_utf8_lossy(&noir_output.stderr));
    }
}
```

#### **1.3 Add Noir Verifier Module**
```rust
// server/src/noir_verifier.rs
use std::sync::Arc;
use hyle_modules::modules::{Module, SharedMessageBus};
use sdk::{Blob, ContractName};

pub struct NoirVerifier {
    contract_name: ContractName,
    verifier_circuit: Arc<UltraHonkVerifier>,
}

impl Module for NoirVerifier {
    type Context = Arc<NoirVerifierCtx>;
    
    async fn build(bus: SharedMessageBus, ctx: Self::Context) -> Result<Self> {
        // Load compiled Noir verifier circuit
        let verifier_circuit = load_ultrahonk_verifier("zkpassport_identity").await?;
        
        Ok(NoirVerifier {
            contract_name: ctx.contract_name.clone(),
            verifier_circuit: Arc::new(verifier_circuit),
        })
    }
    
    async fn run(&mut self) -> Result<()> {
        // Handle proof verification requests
        loop {
            match self.recv().await? {
                NoirVerifierEvent::VerifyProof(proof_blob, tx_hash) => {
                    let is_valid = self.verify_proof(&proof_blob).await?;
                    self.send_verification_result(tx_hash, is_valid).await?;
                }
            }
        }
    }
}

impl NoirVerifier {
    async fn verify_proof(&self, proof_blob: &Blob) -> Result<bool> {
        // Extract proof data from blob
        let proof_data = extract_noir_proof(proof_blob)?;
        
        // Verify using UltraHonk verifier
        self.verifier_circuit.verify(proof_data).await
    }
}
```

### **Phase 2: Proof Pipeline Integration**

#### **2.1 Real Proof Generation**
```rust
// server/src/noir_prover.rs
pub struct NoirProver {
    circuit: Arc<CompiledNoirCircuit>,
}

impl NoirProver {
    pub async fn generate_password_proof(
        &self, 
        username: &str, 
        password: &str
    ) -> Result<NoirProof> {
        // Generate witness from private inputs
        let witness = generate_witness(&[
            ("username", hash_to_field(username)),
            ("password", hash_to_field(password)),
        ])?;
        
        // Generate UltraHonk proof
        let proof = self.circuit.prove(witness).await?;
        
        Ok(NoirProof {
            proof_data: proof.to_bytes(),
            public_inputs: extract_public_inputs(&proof),
            verification_key: self.circuit.verification_key(),
        })
    }
}
```

#### **2.2 Replace Mock Authentication**
```rust
// server/src/app.rs - Replace mock implementation
async fn noir_authenticate(
    State(state): State<RouterCtx>,
    Json(request): Json<NoirAuthRequest>,
) -> Result<Json<NoirAuthResponse>, StatusCode> {
    tracing::info!("🔐 Starting real Noir authentication for user: {}", request.username);
    
    // Step 1: Generate actual Noir proof
    let noir_prover = state.get_noir_prover().await?;
    let proof = noir_prover.generate_password_proof(
        &request.username, 
        &request.password_field
    ).await.map_err(|e| {
        tracing::error!("❌ Proof generation failed: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    
    // Step 2: Submit proof to Hyli chain
    let proof_blob = Blob::new(
        state.noir_contract_name.clone(),
        proof.to_blob_data(),
    );
    
    let tx_hash = state.node_client
        .send_tx_blob(BlobTransaction::new(
            format!("{}@zkpassport", request.username),
            vec![proof_blob]
        ))
        .await
        .map_err(|e| {
            tracing::error!("❌ Transaction submission failed: {}", e);
            StatusCode::BAD_REQUEST
        })?;
    
    // Step 3: Wait for verification
    let verification_result = wait_for_noir_verification(
        &state.bus, 
        tx_hash, 
        Duration::from_secs(30)
    ).await?;
    
    if verification_result.is_valid {
        tracing::info!("✅ Noir proof verified on-chain");
        Ok(Json(NoirAuthResponse {
            success: true,
            message: format!("Authentication successful for user: {}", request.username),
            proof_hash: Some(hex::encode(proof.proof_data)),
            tx_hash: Some(tx_hash.to_string()),
        }))
    } else {
        tracing::error!("❌ Noir proof verification failed");
        Err(StatusCode::UNAUTHORIZED)
    }
}
```

### **Phase 3: State Management & Indexing**

#### **3.1 Noir State Indexer**
```rust
// Following microbecode pattern for state indexing
use hyle_modules::modules::contract_state_indexer::{ContractStateIndexer, ContractStateIndexerCtx};

// In server/src/main.rs
handler
    .build_module::<ContractStateIndexer<NoirIdentityContract>>(ContractStateIndexerCtx {
        contract_name: "zkpassport_identity".into(),
        data_directory: config.data_directory.clone(),
        api: api_ctx.clone(),
    })
    .await?;
```

#### **3.2 Mixed Contract Queries**
```rust
// server/src/api/mixed_queries.rs
pub async fn get_user_identity_status(
    State(ctx): State<RouterCtx>,
    Path(username): Path<String>
) -> Result<Json<IdentityStatus>, AppError> {
    // Query Noir contract state
    let identity_verified = ctx.noir_indexer
        .get_user_verification_status(&username)
        .await?;
    
    // Query AMM contract state  
    let amm_balances = ctx.amm_indexer
        .get_user_balances(&username)
        .await?;
    
    Ok(Json(IdentityStatus {
        username,
        is_verified: identity_verified,
        balances: amm_balances,
        verification_method: "noir_circuit".to_string(),
    }))
}
```

## 🔧 **Required Dependencies**

### **Cargo.toml Updates**
```toml
[dependencies]
# Add UltraHonk support
noir-ultrahonk = "0.1.0"
hyle-noir-verifier = "0.1.0"

# Add async proof generation
tokio = { version = "1.0", features = ["full"] }
async-trait = "0.1"
```

### **Noir Contract Updates**
```noir
// noir-contracts/zkpassport_identity/src/main.nr
#![backend = "ultrahonk"]

use std::hash::poseidon2::Poseidon2::hash;

fn main(
    // Public inputs (on-chain verification)
    expected_password_hash: pub Field,
    expected_user_hash: pub Field,
    
    // Private inputs (client-side proving)
    user_password: Field,
    user_name: Field
) -> pub Field {
    // Verify credentials
    assert(user_name == expected_user_hash);
    assert(user_password == expected_password_hash);
    
    // Return verification success
    1
}

// Add UltraHonk-specific compilation targets
#[ultrahonk::main]
pub fn verify_password_ultrahonk(
    proof: &[u8],
    verification_key: &[u8],
    public_inputs: &[Field]
) -> bool {
    // UltraHonk verification logic
    ultrahonk::verify(proof, verification_key, public_inputs)
}
```

## 🧪 **Testing Strategy**

### **Integration Tests**
```bash
# Test real proof generation
cargo test noir_proof_generation

# Test Hyli integration  
cargo test hyli_noir_verification

# Test mixed contract interaction
cargo test mixed_amm_identity_flow
```

### **End-to-End Verification**
```bash
# Test complete authentication flow
./test-scripts/test-real-noir-auth.sh

# Verify state consistency
./test-scripts/test-mixed-contract-state.sh
```

## 📊 **Success Metrics**

- ✅ **Real Proofs**: UltraHonk proof generation working
- ✅ **On-Chain Verification**: Noir proofs verified by Hyli
- ✅ **State Indexing**: Noir contract state queryable
- ✅ **Mixed Operations**: AMM gated by Noir identity verification
- ✅ **Performance**: Sub-5-second proof generation and verification

---

**This implementation plan transforms your current mock Noir integration into a production-ready system following the proven patterns from microbecode/zkhack.** 🚀 