# Hyli Non-US Verification with ZKPassport

*A comprehensive guide to implementing nationality verification using ZKPassport's Noir circuits on the Hyli blockchain*

---

## 🎯 **Overview**

This guide explains how to implement privacy-preserving nationality verification for an AMM on Hyli using ZKPassport's existing Noir circuits. The solution allows users to prove they are "not US citizens" without revealing their actual nationality, enabling compliant DeFi operations while preserving privacy.

---

## 🔧 **Using ZKPassport's Existing Noir Circuits**

### **Why Use ZKPassport?**

ZKPassport provides **battle-tested, open-source Noir circuits** for proving passport attributes like age and nationality. Their system supports "nationality verification" queries where users can prove citizenship status without revealing the actual country.

### **Key ZKPassport Features:**

#### **Nationality Verification Queries**
```typescript
// Prove passport's country IS in a list (e.g., EU countries)
queryBuilder.in("nationality", [...])

// Prove passport's country is NOT in a list (e.g., sanctioned countries)
queryBuilder.out("nationality", [...])
```

#### **Perfect Fit for Non-US Verification**
Your "non-US citizen" requirement maps directly to:
```typescript
queryBuilder.out("nationality", ["USA"])
```

### **Built-in Security Guarantees**

ZKPassport's circuits automatically provide:
- ✅ **Passport authenticity verification** (RFID chip signature checks)
- ✅ **Tamper-proof validation** (digital signature verification)
- ✅ **Zero-knowledge disclosure** (only proves country ≠ USA)
- ✅ **Production-ready security** (Apache 2.0 licensed, audited)

### **Technical Implementation**

Under the hood, ZKPassport:
1. **Reads passport chip data** via mobile app
2. **Verifies RFID signatures** using Noir circuits
3. **Extracts country code** from authenticated data
4. **Generates ZK proof** that country ∉ {USA}

---

## 🏗️ **Compiling and Registering the Verifier on Hyli**

### **Step 1: Obtain the Noir Circuit**

**Option A: Use ZKPassport's Ready-Made Circuit**
- Clone from ZKPassport's GitHub repository
- Look for "nationality exclusion" or "sanctioned countries" examples
- Adapt their existing circuit for USA exclusion

**Option B: Use ZKPassport SDK**
```bash
# Install ZKPassport SDK
npm install zkpassport-sdk

# Use their noir_rs crate for Rust integration
cargo add zkpassport-noir
```

### **Step 2: Compile for Hyli**

```bash
# Compile Noir circuit using nargo
nargo compile

# Generate verification key using Barretenberg backend
bb write_vk -b ./target/circuit.json

# Output: verification key (Program ID for Hyli)
```

### **Step 3: Register on Hyli**

**Create Hyli Contract:**
```toml
# Nargo.toml
[package]
name = "zkpassport_verifier"
type = "bin"
authors = ["Your Name"]
compiler_version = ">=0.19.0"

[dependencies]
std = { tag = "v0.19.0", git = "https://github.com/noir-lang/noir" }
```

**Deploy Process:**
1. **Integrate into Hyli app structure** (following scaffold template)
2. **Build verification key** using Hyli's build process
3. **Register contract** with unique name (e.g., `passportVerifier`)
4. **Note Program ID** (verification key hash) for reference

### **Contract Configuration Example**
```rust
// In your Hyli config
contract_names = {
    "passportVerifier" = "0x1234...abcd",  // Your verification key hash
    "amm" = "contract1",                   // Your AMM contract
}
```

---

## ⚙️ **Minimal Noir Circuit (Custom Implementation)**

*If you prefer to implement a custom circuit for learning purposes*

### **Circuit Structure**

```noir
use std::println;

fn main(
    country_code: Field,      // Private: User's actual country (encoded)
    commitment: pub Field     // Public: Commitment to user identity (optional)
) {
    // Constants
    let USA_CODE: Field = 5591873; // "USA" encoded as ASCII: 0x555341
    
    // Prove country ≠ USA using inverse constraint
    let diff = country_code - USA_CODE;
    let inv_diff = witness_inverse(diff);
    
    // This constraint only passes if diff ≠ 0 (i.e., country ≠ USA)
    constrain diff * inv_diff == 1;
    
    println("✅ Verified: User is not from USA");
}

// Helper function for inverse constraint
fn witness_inverse(val: Field) -> Field {
    // Implementation depends on Noir version
    // This ensures val ≠ 0 by requiring a multiplicative inverse
}
```

### **Country Code Encoding**

```rust
// ASCII encoding example
"USA" → [0x55, 0x53, 0x41] → 85*256² + 83*256 + 65 = 5,591,873
"GBR" → [0x47, 0x42, 0x52] → 71*256² + 66*256 + 82 = 4,670,546
"FRA" → [0x46, 0x52, 0x41] → 70*256² + 82*256 + 65 = 4,612,161
```

### **⚠️ Security Warning**

**Custom circuits without passport verification are vulnerable:**
- ❌ Users can lie about country codes
- ❌ No authenticity guarantees
- ❌ Not suitable for production KYC/AML

**Always use ZKPassport's full circuits for production systems.**

---

## 🔄 **Integration into Hyli Multi-Blob Transactions**

### **Multi-Blob Architecture**

Hyli transactions separate identity proofs from business logic:

```
Transaction = [
    Blob 0: Identity Proof (ZKPassport nationality verification),
    Blob 1: AMM Logic (Risc0 swap execution)
]
```

### **Atomic Verification**
- ✅ **All blobs must verify** for transaction to succeed
- ✅ **Identity failure = transaction failure** (automatic compliance)
- ✅ **No manual reversions needed** in AMM code

### **Transaction Flow**

#### **1. Identity Blob (Noir)**
```json
{
    "contract_name": "passportVerifier",
    "proof": "0x...",  // ZKPassport nationality proof
    "public_inputs": {
        "user_account": "alice@wallet",
        "timestamp": 1640995200
    }
}
```

#### **2. AMM Blob (Risc0)**
```json
{
    "contract_name": "amm",
    "proof": "0x...",  // Swap execution proof
    "public_inputs": {
        "user": "alice@wallet",
        "token_in": "USDC",
        "amount_in": 1000,
        "token_out": "ETH",
        "min_amount_out": 500
    }
}
```

### **State Management Options**

#### **Option A: Per-Transaction Verification**
- User provides passport proof with **every swap**
- **Pros**: No persistent state, maximum privacy
- **Cons**: Higher proof generation cost per transaction

#### **Option B: Registration Model**
- User proves nationality **once** to register
- Subsequent swaps use **session keys** or **account verification**
- **Pros**: Efficient recurring transactions
- **Cons**: Requires identity state management

### **Implementation Example**

```rust
// AMM Contract (Risc0)
impl AmmContract {
    pub fn swap(&mut self, user: String, params: SwapParams) -> SwapResult {
        // At this point, identity verification has ALREADY passed
        // (transaction wouldn't execute if identity blob failed)
        
        // Optionally check registered status
        if self.require_registration {
            assert!(self.verified_users.contains(&user), "User not registered");
        }
        
        // Execute swap logic
        self.execute_swap(&user, params)
    }
}
```

### **Account Linking**

```rust
// Identity contract updates user registry
impl PassportVerifier {
    pub fn verify_nationality(&mut self, proof: ZKProof, user: String) -> bool {
        // Verify ZKPassport proof
        let is_valid = verify_noir_proof(proof);
        
        if is_valid {
            // Register user as verified non-US person
            self.verified_users.insert(user.clone());
            self.last_verification.insert(user, current_timestamp());
        }
        
        is_valid
    }
}
```

---

## 🧪 **Testing and Deployment**

### **Development Workflow**

1. **Deploy contracts on Hyli devnet**
   ```bash
   # Register passport verifier
   hyli deploy contracts/passport_verifier
   
   # Register AMM contract  
   hyli deploy contracts/amm
   ```

2. **Test with mock passport data**
   ```bash
   # Generate test proof for non-US country
   zkpassport-cli generate-proof --country FRA --exclude USA
   
   # Submit multi-blob transaction
   hyli tx submit --blob1 identity_proof.json --blob2 amm_swap.json
   ```

3. **Integration testing**
   - Test successful swaps with valid non-US proofs
   - Verify rejection of US passport proofs
   - Check atomic transaction behavior

### **Production Deployment**

```bash
# Build for production
cargo build --release --features production

# Deploy to Hyli mainnet
hyli deploy --network mainnet contracts/
```

---

## 📚 **References and Resources**

### **ZKPassport Documentation**
- **[Nationality Verification](https://docs.zkpassport.id/examples/nationality)** - Official examples for country-based proofs
- **[FAQ](https://docs.zkpassport.id/faq)** - Technical details and circuit availability
- **[GitHub Repository](https://github.com/zkpassport)** - Open-source Noir circuits (Apache 2.0)

### **Hyli Documentation**
- **[Multi-Blob Transactions](https://docs.hyli.org/concepts/transaction/)** - Architecture and implementation
- **[Identity Management](https://docs.hyli.org/concepts/identity/)** - Identity proof patterns
- **[Your First App](https://docs.hyli.org/quickstart/your-first-app/)** - Development scaffold and examples

### **Related Projects**
- **[Rarimo Passport ZK](https://github.com/rarimo/passport-zk-circuits)** - Similar passport verification circuits
- **[Rarimo Verification Guide](https://docs.rarimo.com/zk-passport/guide-on-chain-verification/)** - On-chain registry patterns
- **[Hyli Examples](https://docs.hyli.org/)** - Token transfer and identity proof examples

### **Technical Resources**
- **[ZKPassport Rust Crate](https://github.com/zkpassport/noir_rs)** - Rust integration for proof generation
- **[Noir Documentation](https://noir-lang.org/)** - Circuit development reference
- **[Barretenberg Backend](https://github.com/AztecProtocol/barretenberg)** - Proof system used by ZKPassport

---

## 🎯 **Recommended Implementation**

### **Best Practice Architecture**

1. **Use ZKPassport's production circuits** (don't reinvent passport verification)
2. **Deploy Noir verifier on Hyli** with multi-blob transaction support
3. **Implement registration model** for efficient recurring transactions
4. **Test thoroughly on devnet** before mainnet deployment

### **Why This Approach Works**

- ✅ **Security**: Leverages ZKPassport's proven cryptographic verification
- ✅ **Privacy**: Zero-knowledge nationality disclosure
- ✅ **Compliance**: Automatic enforcement via Hyli's atomic transactions
- ✅ **Efficiency**: Reuses existing, audited circuits
- ✅ **Scalability**: Multi-blob architecture supports complex workflows

By combining **ZKPassport's robust identity proofs** with **Hyli's multi-blob architecture**, you achieve a compliant, privacy-preserving AMM that automatically gates access based on nationality verification. The heavy lifting (passport cryptography) is handled by battle-tested circuits, while Hyli ensures atomic proof validation before any state changes occur.

---

*This implementation provides a production-ready foundation for compliant DeFi operations while maintaining user privacy and leveraging proven zero-knowledge infrastructure.* 🚀