# Hyli Development & Debugging Guide

*Lessons learned from building an AMM with integrated token management on Hyli*

---

## 🎯 **Purpose**

This guide documents real-world debugging experiences, solutions, and best practices discovered while implementing a complex AMM contract on Hyli during ZKHack Berlin 2025. These learnings can help accelerate future development and avoid common pitfalls.

---

## 🚨 **Common Issues & Solutions**

### **1. Rust Borrow Checker Errors in Contract Logic**

#### **Problem**
```rust
error[E0502]: cannot borrow `self.user_balances` as mutable because it is also borrowed as immutable
```

#### **Root Cause**
Holding immutable references while trying to mutate the same HashMap:
```rust
// ❌ Wrong - holds reference while trying to mutate
let user_balance = self.user_balances.get(&key).unwrap_or(&0);
self.user_balances.insert(key, user_balance - amount); // Error!
```

#### **Solution**
Copy values instead of holding references:
```rust
// ✅ Correct - copies value, no reference held
let user_balance = *self.user_balances.get(&key).unwrap_or(&0);
self.user_balances.insert(key, user_balance - amount); // Works!
```

#### **Lesson**
When working with `HashMap` operations that read and write, always **dereference** (`*`) to copy values and avoid borrow conflicts.

---

### **2. Format String Mismatches**

#### **Problem**
```rust
error: 7 positional arguments in format string, but there are 5 arguments
```

#### **Root Cause**
Too many `{}` placeholders in format strings:
```rust
// ❌ Wrong - 7 placeholders, 5 arguments
format!("Reserves: {} {} = {}, {} {} = {}, Total Liquidity: {}", 
    token_a, reserve_a, token_b, reserve_b, total_liquidity)
```

#### **Solution**
Match placeholder count to argument count:
```rust
// ✅ Correct - 5 placeholders, 5 arguments  
format!("Reserves: {} = {}, {} = {}, Total Liquidity: {}", 
    token_a, reserve_a, token_b, reserve_b, total_liquidity)
```

#### **Lesson**
Always count format placeholders carefully, especially when refactoring complex output strings.

---

### **3. Missing Feature Dependencies**

#### **Problem**
```rust
error[E0433]: failed to resolve: could not find `contract_indexer` in `client_sdk`
note: the item is gated behind the `indexer` feature
```

#### **Root Cause**
The `client-sdk` dependency has `default-features = false` but doesn't include required features:
```toml
client-sdk = { 
    git = "https://github.com/Hyle-org/hyle.git", 
    default-features = false, 
    package = "hyle-client-sdk", 
    tag = "v0.13.0" 
}
```

#### **Solutions**

**Option A: Temporary Fix (Hackathon)**
Comment out indexer modules to avoid the dependency:
```rust
// #[cfg(feature = "client")]
// pub mod indexer;
```

**Option B: Proper Fix (Production)**
Enable the indexer feature:
```toml
client-sdk = { 
    features = ["indexer"],
    # ... other config
}
```

#### **Lesson**
For hackathons, temporary fixes are acceptable. For production, properly configure feature flags.

---

### **4. Server API Data Format Issues**

#### **Problem A: Wrong BlobData Format**
```
Failed to deserialize: expected a sequence, got string "dGVzdA=="
```

#### **Solution**
Use byte arrays instead of base64 strings:
```json
// ❌ Wrong
{"data": "dGVzdA=="}

// ✅ Correct  
{"data": [116, 101, 115, 116]}
```

#### **Problem B: Invalid Transaction Identity**
```
Transaction identity alice is not correctly formed. It should be in the form <id>@<contract_id_name>
```

#### **Solution**
Use proper identity format:
```
// ❌ Wrong
x-user: alice

// ✅ Correct
x-user: alice@contract1
```

#### **Problem C: Unknown Contract References**
```
Failed to handle blob transaction: Blob Transaction contains blobs for unknown contracts: test, test2
```

#### **Solution**
Reference actual deployed contracts in wallet_blobs:
```json
// ❌ Wrong - references non-existent contracts
"wallet_blobs": [
    {"contract_name": "test", "data": [...]},
    {"contract_name": "test2", "data": [...]}
]

// ✅ Correct - references actual wallet contracts
"wallet_blobs": [
    {"contract_name": "hydentity", "data": [...]},
    {"contract_name": "wallet", "data": [...]}
]
```

#### **Lesson**
Always check server logs for detailed error messages. HTTP status codes alone don't provide enough debugging information.

---

### **5. Proof Generation Timeouts**

#### **Problem**
```
500 Internal Server Error: deadline has elapsed
```

#### **Root Cause**
Proof generation takes longer than the 5-second timeout:
```rust
tokio::time::timeout(Duration::from_secs(5), async {
    // Proof generation happens here
})
```

#### **Solution**
Increase timeout for development:
```rust
tokio::time::timeout(Duration::from_secs(30), async {
    // Give more time for proof generation
})
```

#### **Lesson**
**"deadline has elapsed" is actually GOOD NEWS** - it means your transaction was accepted and is being processed. The proof just takes time to generate.

---

## 🔧 **Development Best Practices**

### **1. Incremental Testing Strategy**

```bash
# 1. Test contract compilation first
cargo build -p contracts --features build --features all

# 2. Test server compilation  
cargo check -p server

# 3. Test server startup
RISC0_DEV_MODE=true cargo run -p server

# 4. Test basic endpoints
curl http://localhost:4002/_health

# 5. Test with minimal data
curl -X POST http://localhost:4002/api/test-amm \
  -H "Content-Type: application/json" \
  -H "x-user: alice@contract1" \
  -d '{"wallet_blobs": [...]}'
```

### **2. Error Debugging Workflow**

1. **Check compilation errors first** - Fix Rust errors before testing
2. **Read complete error messages** - Don't just look at HTTP status codes
3. **Check server logs** - Most valuable debugging information is in server output
4. **Test data formats incrementally** - Start with simple data, add complexity
5. **Verify contract names** - Ensure wallet_blobs reference deployed contracts

### **3. Development Environment Setup**

```bash
# Always run in development mode to speed up proving
export RISC0_DEV_MODE=true

# Keep services running in separate terminals:
# Terminal 1: Hyli node
docker-compose up -d

# Terminal 2: Server  
RISC0_DEV_MODE=true cargo run -p server

# Terminal 3: Frontend
cd front && bun run dev

# Terminal 4: Testing/curl commands
```

### **4. Contract Development Patterns**

#### **Self-Contained Contracts (Hackathon)**
```rust
// Include all functionality in one contract for rapid development
pub struct AmmContract {
    pools: HashMap<String, LiquidityPool>,
    user_balances: HashMap<String, u128>, // Built-in token management
}
```

#### **Modular Contracts (Production)**
```rust
// Separate concerns into different contracts
pub struct TokenContract { balances: HashMap<String, u128> }
pub struct AmmContract { pools: HashMap<String, LiquidityPool> }
```

---

## 📊 **Performance & Debugging Insights**

### **Proof Generation Times**
- **Simple operations**: 1-5 seconds
- **Complex AMM operations**: 5-30 seconds  
- **Multiple contract interactions**: 10-60 seconds

### **Common Timeout Values**
```rust
// Development
Duration::from_secs(30)   // Allow time for debugging

// Testing  
Duration::from_secs(10)   // Balance speed vs reliability

// Production
Duration::from_secs(60)   // Handle worst-case scenarios
```

### **Server Log Interpretation**

```bash
# ✅ Good signs
INFO server::init: ✅ contract1 contract is up to date
INFO hyle_modules::modules::prover: ✅ Catching up finished

# ⚠️ Pay attention to
ERROR hyle_modules::node_state: Failed to handle blob transaction
INFO hyle_modules::modules::rest: [POST] /api/test-amm - 500 Internal Server Error

# 🔍 Debugging info
INFO hyle_modules::modules::da_listener: 📝 Loaded contract state for contract1
```

---

## 🎯 **Hyli-Specific Learnings**

### **Transaction Identity Format**
```
Pattern: <user_id>@<contract_name>
Example: alice@contract1
```

### **Proof Composition Architecture**
```rust
// Multiple contracts can be called in one transaction
blobs.extend(vec![
    amm_action.as_blob(contract1_name),      // AMM logic
    identity_action.as_blob(contract2_name), // Identity verification  
]);
```

### **Contract Deployment State**
- Contracts are automatically deployed when server starts
- Contract updates require restarting the server
- State persists in `./data` directory

### **Development vs Production Proving**
```bash
# Development (fast, for testing)
RISC0_DEV_MODE=true cargo run -p server

# Production (secure, slow)
cargo run -p server
```

---

## 🚀 **Advanced Debugging Techniques**

### **1. Contract State Inspection**

```bash
# Check what contracts are loaded
grep "📝 Loaded contract state" server_logs.txt

# Check proof generation status  
grep "Catching up" server_logs.txt

# Monitor transaction processing
grep "Failed to handle blob transaction" server_logs.txt
```

### **2. Manual Transaction Testing**

```bash
# Test with minimal wallet blobs
curl -X POST http://localhost:4002/api/test-amm \
  -H "x-user: alice@contract1" \
  -d '{"wallet_blobs": [
    {"contract_name": "hydentity", "data": [1,2,3,4]},
    {"contract_name": "wallet", "data": [1,2,3,4]}
  ]}'
```

### **3. Frontend Integration Testing**

```javascript
// Test AMM functionality from browser console
const response = await fetch('/api/test-amm', {
  method: 'POST',
  headers: {
    'Content-Type': 'application/json',
    'x-user': 'alice@contract1'
  },
  body: JSON.stringify({
    wallet_blobs: [
      {contract_name: "hydentity", data: [1,2,3,4]},
      {contract_name: "wallet", data: [1,2,3,4]}
    ]
  })
});
```

---

## 📚 **Resources for Further Debugging**

### **Hyli Documentation**
- [Proof Composition](https://docs.hyli.org/concepts/proof-composition/)
- [Transaction Format](https://docs.hyli.org/concepts/transactions/)
- [Contract Development](https://docs.hyli.org/quickstart/)

### **Error Code References**
- **422 Unprocessable Entity**: Data format issues
- **400 Bad Request**: Invalid transaction format  
- **500 Internal Server Error**: Usually proof timeouts or contract errors
- **404 Not Found**: Endpoint doesn't exist (old server code)

### **Debugging Commands**
```bash
# Check contract compilation
cargo check -p contracts

# Check server compilation  
cargo check -p server

# Test with verbose output
curl -v http://localhost:4002/api/test-amm

# Monitor server logs in real-time
RISC0_DEV_MODE=true cargo run -p server | tee server.log
```

---

## 🎉 **Successful AMM Testing Results**

### **Test Results Summary (December 20, 2025)**

After resolving compilation and API format issues, we achieved **successful AMM transactions** with interesting findings:

---

## 🔧 **OFFICIAL Development Workflow (Updated December 21, 2025)**

Based on guidance from the Hyli team, the following development workflow is now established as **best practice**:

### **🔄 Contract Recompilation Process**

**When to Use**: Any time you modify contract logic in `contracts/contract1/src/lib.rs` or `contracts/contract2/src/lib.rs`

**Required Command**:
```bash
# Full reset required for contract changes
rm -rf data && RISC0_DEV_MODE=1 cargo run -p server
```

**What This Does**:
- **`rm -rf data`**: Removes all cached state, proofs, and indexer data
- **`RISC0_DEV_MODE=1`**: Enables faster proving for development (not production-ready proofs)
- **`cargo run -p server`**: Recompiles contracts and starts server with fresh state

**⚠️ Important**: Regular `cargo run -p server` **will not** pick up contract changes - you must delete the `data` directory.

### **🏗️ Chain Reset for New Deployments**

**When to Use**: When you have major state corruption, want a completely fresh start, or are testing deployment scenarios

**Required Commands**:
```bash
# Complete blockchain reset
docker-compose down --volumes --remove-orphans
docker-compose up
```

**What This Does**:
- **`--volumes`**: Removes all Docker volumes (blockchain data, wallet state)
- **`--remove-orphans`**: Cleans up any leftover containers
- **Restarts**: Fresh Hyli blockchain node, wallet server, all services

**⚠️ Warning**: This wipes **all transaction history** and **all user accounts**. Use sparingly.

### **🔍 Development Environment Recommendations**

**Use Localhost (Not Testnet)**:
- ✅ **Faster iteration**: No network delays
- ✅ **Block explorer available**: View transactions at localhost block explorer
- ✅ **Full control**: Reset chain when needed
- ✅ **Privacy**: No public transaction history during development

**Block Explorer Usage**:
- **Purpose**: Debug transaction details, state transitions, proof status
- **Access**: Available on localhost when running Hyli node
- **Benefits**: Visual inspection of transaction flow, blob data, proof generation

### **👤 User Management in Development**

**Important**: User accounts are **not persistent** across chain resets. You must recreate users after each full reset.

#### **Built-in Superuser Account**
**Always Available**:
```
Username: hyli
Password: hylisecure
```

**Benefits**:
- ✅ **Pre-funded**: Has tokens available for testing
- ✅ **Always exists**: Survives chain resets
- ✅ **Reliable**: Good for automated testing scripts
- ✅ **Quick testing**: No need to register or fund

#### **Custom Test Users (e.g., "bob")**
**After Chain Reset Required**:
```bash
# Must re-register users after:
# docker-compose down --volumes --remove-orphans
```

**Process**:
1. **Visit frontend**: http://localhost:5173/
2. **Create new user**: Register "bob" with password
3. **Update .env**: Set USER=bob, PASSWORD=... if needed
4. **Test transactions**: User starts with 0 tokens, needs minting

**⚠️ Important**: Custom users like "bob" **do not persist** across full chain resets. You'll need to re-register them each time.

### **📋 Daily Development Routine**

**1. Start Development Session**:
```bash
# Start all services
docker-compose up -d
RISC0_DEV_MODE=1 cargo run -p server
cd front && bun run dev
```

**2. User Setup** (if needed):
```bash
# Option A: Use built-in superuser (recommended for quick testing)
# Username: hyli, Password: hylisecure (always available)

# Option B: Create custom user (after chain resets)
# Visit http://localhost:5173/ and register new user
# Update .env with USER=bob, PASSWORD=...
```

**3. Contract Development Cycle**:
```bash
# Edit contract logic in contracts/contract1/src/lib.rs
# Test with unit tests first:
cargo test -p contract1

# Then integrate:
rm -rf data && RISC0_DEV_MODE=1 cargo run -p server
# Test via frontend at http://localhost:5173
```

**4. Full Chain Reset** (when needed):
```bash
# Complete reset (removes all users, transactions, state)
docker-compose down --volumes --remove-orphans
docker-compose up

# ⚠️ Must re-register custom users after this step
# Built-in "hyli" user will still be available
```

**5. End Development Session**:
```bash
# Optional: Preserve state for next session
# Just stop the server with Ctrl+C

# Optional: Clean reset for next session  
docker-compose down --volumes --remove-orphans
```

### **🧪 Integration Testing Strategy (Official)**

**Confirmed by Hyli Team**: No robust integration testing framework exists. The **recommended approach** is:

**1. Unit Tests (18 seconds)**:
```bash
cargo test -p contract1
```

**2. Manual Integration Testing (20 minutes)**:
```bash
# Reset environment
rm -rf data && RISC0_DEV_MODE=1 cargo run -p server

# Test via API
curl -X POST http://localhost:4002/api/test-amm \
  -H "x-user: alice@contract1" \
  -d '{"wallet_blobs": [...]}'

# Test via frontend
# Visit http://localhost:5173 and perform user actions
```

**3. Block Explorer Validation**:
- View transaction details in block explorer
- Verify state transitions are correct
- Check proof generation status

**This is the official Hyli/RISC0 recommended testing approach** - not a limitation, but the current best practice.

---

## ✅ **Successful Transactions**

#### **✅ Successful Transactions**
```bash
# Test 1: Basic AMM test (1000 USDC)
curl -X POST http://localhost:4002/api/test-amm \
  -H "x-user: alice@contract1" \
  -d '{"wallet_blobs": [{"contract_name": "hydentity", "data": [1,2,3,4]}, {"contract_name": "wallet", "data": [5,6,7,8]}]}'
# Result: HTTP 200 OK, Hash: dc27fcab2641d016b01757d4c0bb0defb07866ee0fdb75dfe51d6037d140c575

# Test 2: Mint tokens endpoint (5000 ETH)  
curl -X POST http://localhost:4002/api/mint-tokens \
  -H "x-user: bob@contract1" \
  -d '{"wallet_blobs": [...], "token": "ETH", "amount": 5000}'
# Result: HTTP 200 OK, Hash: 08965aaffe9aba7c38d54114bcc1c44c9f1baf4dd706e3043d2f3de581498e35
```

#### **✅ Contract Execution Success**
```
INFO: 🔧 Executed contract: Minted 1000 USDC tokens for user alice@contract1. Success: true
INFO: 🔧 Executed contract: Minted 5000 ETH tokens for user bob@contract1. Success: true
INFO: 🔧 Executed contract: Minted 1000 USDC tokens for user bob@wallet. Success: true (×2 more)
```

### **🎨 Frontend User Experience Improvements**

After resolving the core backend issues, we improved the frontend for better user experience:

#### **✅ Enhanced Transaction Handling**
- **Extended timeout**: 30 seconds → 20 minutes to match actual proof generation time
- **Progress indicators**: Shows "Still processing..." with elapsed time every 30 seconds
- **Better messaging**: Clear AMM-focused language instead of generic "blob tx"
- **Multi-user support**: Tested with alice@contract1, bob@contract1, bob@wallet

#### **✅ UI Improvements**
```typescript
// Improved button text
{loading ? 'MINTING TOKENS...' : 'MINT AMM TOKENS (Test)'}

// Better progress updates
`⏳ Still processing... (${minutes}m elapsed) - Contract execution and proof generation in progress`

// Success confirmation
`✅ AMM Transaction confirmed successful! Tokens minted. Hash: ${txHash}`
```

#### **✅ Expected User Flow**
1. **Login**: User authenticates with password/Google/GitHub
2. **Connected**: Shows "Connected Wallet: bob@wallet"
3. **Transaction**: Click "MINT AMM TOKENS (Test)"
4. **Progress**: See real-time updates every 30 seconds
5. **Success**: Get confirmation with transaction hash
6. **Total time**: 15-20 minutes end-to-end

### **🚨 Issues Discovered During Testing**

#### **1. Commitment Metadata Decoding Errors**
```
ERROR: Guest panicked: Failed to decode commitment metadata: Custom { 
  kind: InvalidData, 
  error: "Unexpected length of input" 
}
ERROR: Guest panicked: Failed to decode commitment metadata: Custom { 
  kind: InvalidData, 
  error: "Not all bytes read" 
}
```

**Impact**: Proof generation throws errors but still completes successfully
**Status**: ⚠️ **Needs Hyli team clarification**

#### **2. State Persistence Issues**
```
WARN: No previous tx, returning default state cn=contract1 tx_hash=...
WARN: No previous tx, returning default state cn=contract2 tx_hash=...
```

**Impact**: Each transaction starts with default state instead of persisting previous state
**Status**: ❓ **Unknown if this is expected behavior**

#### **3. Transaction Timeout Warnings**
```
INFO: ⏰ Blob tx timed out: dc27fcab2641d016b01757d4c0bb0defb07866ee0fdb75dfe51d6037d140c575
```

**Impact**: Some transactions timing out despite successful execution
**Status**: ⚠️ **May affect user experience**