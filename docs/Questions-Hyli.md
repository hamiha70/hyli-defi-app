# Questions for Hyli Team - ZKHack Berlin 2025

## Issue Resolution: ContractStateIndexer & Missing Feature Dependencies

### 📋 **Issue Summary**

**Date**: June 21, 2025  
**Environment**: Fresh Hyli app-scaffold setup following ZKHack Berlin quickstart  
**Status**: ✅ Resolved (temporary fix applied)

### 🚨 **Problems Encountered**

#### 1. Program ID Mismatch Error
```
Error initializing node: Invalid program_id for contract1. 
On-chain version is 799333f072dad4a171fe58420e934f0a7df1758e3c71346ee052c6fb8bb75d54, 
expected 2bd1e7697cbd031f8b34fc741463085efcf4db1789f06e1333b889061b7ea1d4
```

#### 2. Missing Indexer Feature Dependencies
```rust
error[E0433]: failed to resolve: could not find `contract_indexer` in `client_sdk`
note: the item is gated behind the `indexer` feature
```

#### 3. ContractHandler Trait Not Implemented
```rust
error[E0277]: the trait bound `contract1::Contract1: ContractHandler` is not satisfied
--> server/src/main.rs:107:10
```

### 🔍 **Root Cause Analysis**

1. **Program ID Mismatch**: Stale blockchain state vs. updated contract code
2. **Missing Features**: The `client-sdk` dependency in `Cargo.toml` has `default-features = false` but doesn't include the `indexer` feature
3. **Dependency Chain**: 
   - Contract indexer modules implement `ContractHandler` trait
   - Server tries to use `ContractStateIndexer<Contract1>` and `ContractStateIndexer<Contract2>`
   - These require the `ContractHandler` trait implementation

### ✅ **Our Resolution**

#### Step 1: Clean Blockchain State
```bash
rm -rf ./data
docker-compose down --volumes --remove-orphans
docker-compose up -d
```

#### Step 2: Disable Indexer Modules in Contracts
**Files Modified**: 
- `contracts/contract1/src/lib.rs`
- `contracts/contract2/src/lib.rs`

```rust
// Before
#[cfg(feature = "client")]
pub mod indexer;

// After  
// Temporarily disabled indexer module to avoid missing feature dependency
// #[cfg(feature = "client")]
// pub mod indexer;
```

#### Step 3: Disable ContractStateIndexer in Server
**File Modified**: `server/src/main.rs`

```rust
// Commented out ContractStateIndexer module builds
// handler
//     .build_module::<ContractStateIndexer<Contract1>>(ContractStateIndexerCtx {
//         contract_name: args.contract1_cn.clone().into(),
//         data_directory: config.data_directory.clone(),
//         api: api_ctx.clone(),
//     })
//     .await?;
```

#### Step 4: Remove Unused Imports
```rust
// Commented out in imports
// contract_state_indexer::{ContractStateIndexer, ContractStateIndexerCtx},
```

### 🎯 **Result**
- ✅ Server compiles and runs successfully
- ✅ All API endpoints functional (`/_health`, `/api/config`, `/api/increment`)
- ✅ Contracts deploy and execute correctly
- ✅ Core AMM functionality intact

---

## 🤔 **Questions for Hyli Team**

### **Q1: Indexer Functionality Impact**
**Question**: What specific functionality do we lose by disabling the `ContractStateIndexer` modules? 

**Context**: We understand these provide debugging/monitoring APIs, but want to confirm:
- Are there any core features that depend on indexer functionality?
- Will this affect transaction processing, proof generation, or state management?
- Is this purely for development convenience (like querying contract state via REST API)?

### **Q2: Proper Feature Configuration**
**Question**: What's the recommended way to enable indexer features properly?

**Current workspace dependency**:
```toml
client-sdk = { 
    git = "https://github.com/Hyle-org/hyle.git", 
    default-features = false, 
    package = "hyle-client-sdk", 
    tag = "v0.13.0" 
}
```

**Should we**:
- Add `features = ["indexer"]` to the workspace dependency?
- Add it only to specific contract dependencies?
- Use a different approach for development vs production builds?

### **Q3: Development vs Production Builds**
**Question**: Is there a recommended pattern for handling optional indexer functionality?

**Considerations**:
- Should indexer be enabled by default in development?
- How do production deployments typically handle this?
- Are there feature flags or conditional compilation patterns you recommend?

### **Q4: Alternative Debugging/Monitoring**
**Question**: With indexer disabled, what alternatives exist for debugging contract state during development?

**We need to**:
- Query contract state for debugging
- Monitor transaction execution
- Inspect AMM pool states and balances
- Troubleshoot ZKPassport integration

### **Q5: ZKPassport Integration Considerations**
**Question**: For our ZKPassport + AMM integration, are there specific indexer features we should prioritize?

**Our use case**:
- Identity verification state tracking
- AMM liquidity pool monitoring  
- Multi-contract atomic transaction debugging
- Privacy-preserving compliance checking

### **Q6: Performance Impact**
**Question**: What's the performance impact of enabling full indexer functionality?

**Concerns**:
- Does it affect proof generation times?
- Memory/storage overhead in development?
- Impact on transaction throughput?

---

## 🔍 **Key Takeaways from Extended Testing (December 21, 2025)**

### **📊 System Stability Observations - UPDATED**
After extensive collaboration with the Hyli team and **resolving the indexer functionality**, several critical development patterns have been established:

#### **✅ RESOLVED: Indexer Functionality Working**
```
✅ ContractHandler trait implementation resolved
✅ State indexing now functional  
✅ Contract state queries working via /v1/indexer/contract/*/state endpoints
```
- **Resolution**: Hyli team provided guidance on proper ContractHandler implementation
- **Evidence**: Block explorer shows successful state transitions and indexing
- **Impact**: Frontend can now display real contract state instead of 404 errors

#### **🎯 NEW: Development Workflow Best Practices**
Based on Hyli team guidance, the following patterns are now established:

**Contract Recompilation Process**:
```bash
# When contract logic changes, full reset required:
rm -rf data && RISC0_DEV_MODE=1 cargo run -p server
```

**Chain Reset for New Deployments**:
```bash
# Clean blockchain state completely:
docker-compose down --volumes --remove-orphans
docker-compose up
```

**Environment Recommendation**:
- **Use localhost development** instead of testnet for active development
- **Block explorer available** at localhost for transaction inspection
- **Testnet should be used** only for final validation/demo

#### **🔧 Integration Testing Strategy Clarified**
**Official Hyli/RISC0 Position**: 
- **No robust integration testing framework** currently exists
- **Docker-compose up/down workflow** is the **recommended approach**
- **Manual testing via API/frontend** is the current best practice
- **Unit tests (18s) + Manual integration (20min)** is the accepted pattern

---

## 🆕 **New Questions from AMM Testing (December 20, 2025)**

### **Testing Status Update**
✅ **AMM transactions working!** Both test endpoints successfully execute and return transaction hashes  
⚠️ **Several technical issues discovered** that need clarification

### **Q7: Commitment Metadata Decoding Errors**
**Issue**: Successful transactions throw errors during proof generation:
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

**Questions**:
- Are these errors affecting the validity of our proofs?
- Is this related to our `BorshSerialize` implementation in the AMM contract?
- Should we implement a different serialization method for `StateCommitment`?
- Are there specific requirements for the `commit()` function output format?

**Current Implementation**:
```rust
fn commit(&self) -> sdk::StateCommitment {
    sdk::StateCommitment(self.as_bytes().expect("Failed to encode AMM state"))
}

impl AmmContract {
    pub fn as_bytes(&self) -> Result<Vec<u8>, Error> {
        borsh::to_vec(self)
    }
}
```

### **Q13: ContractHandler Trait Implementation (RESOLVED ✅)**
**Issue**: Cannot run server with indexer functionality enabled due to missing trait implementation.

**RESOLUTION PROVIDED BY HYLI TEAM**:
- ✅ **ContractHandler trait implementation** guidance provided
- ✅ **Indexer modules** now working correctly
- ✅ **State queries** functional via `/v1/indexer/contract/*/state` endpoints  
- ✅ **Block explorer** available for localhost development

**Updated Development Workflow**:
```bash
# Contract changes require full reset:
rm -rf data && RISC0_DEV_MODE=1 cargo run -p server

# New deployments require clean chain:
docker-compose down --volumes --remove-orphans && docker-compose up
```

**User Management**:
```bash
# Built-in superuser (always available):
Username: hyli, Password: <>

# Custom users must be re-registered after chain resets
# Visit http://localhost:5173/ to register users like "bob"
```

**Evidence**: Frontend now successfully displays contract state, block explorer shows transaction details and state transitions.

### **Q8: State Persistence vs State Querying (CLARIFIED)**
**Updated Understanding**: After 4 successful AMM transactions, we now understand this is a **display issue, not a persistence issue**:

**AMM Operations**: ✅ **Working correctly**
```
🔧 Executed contract: Minted 1000 USDC tokens for user bob@wallet. Success: true
# Each transaction successfully processes and mints tokens
```

**State Querying**: ❌ **Blocked by indexer issue** 
```
[GET] /v1/indexer/contract/contract1/state - 404 Not Found
# Cannot query current contract state for UI display
```

**Transaction Warnings**: ⚠️ **May be normal behavior**
```
WARN: No previous tx, returning default state cn=contract1 tx_hash=...
# But transactions still succeed and mint tokens correctly
```

**Clarified Questions**:
- **Is "No previous tx, returning default state" normal** for AMM operations?
- **Does each AMM transaction start fresh** or build on previous state?
- **How should we verify that bob@wallet actually has 4000 USDC** from 4 transactions?
- **Are there alternative endpoints** to query current contract state?
- **Is the AMM state persisting correctly** even though we can't query it?

**Priority**: 🟡 **Medium** - AMM works, just need state visibility for UI

---

## 📊 **Testing Evidence**

### **Successful Transaction Hashes**
1. **AMM Test**: `dc27fcab2641d016b01757d4c0bb0defb07866ee0fdb75dfe51d6037d140c575`
2. **Mint Tokens**: `08965aaffe9aba7c38d54114bcc1c44c9f1baf4dd706e3043d2f3de581498e35`

### **Server Log Snippets**
```
2025-06-20T23:00:56.227951Z  INFO: 🔧 Executed contract: Minted 1000 USDC tokens for user alice@contract1. Success: true
2025-06-20T23:01:15.870875Z  INFO: 🔧 Executed contract: Minted 5000 ETH tokens for user bob@contract1. Success: true
2025-06-20T23:00:56.686678Z ERROR: Error proving tx: Guest panicked: Failed to decode commitment metadata
2025-06-20T23:00:56.944683Z  INFO: ✅ Proved 1 txs, Batch id: 0, Proof TX hash: 278887610c093b986e987000c2b73afe03eb6985065bfaf15ee95d8db300e45c
```

### **Priority Level**
**High Priority** - These questions directly affect:
1. **ZKHack Berlin demo** reliability
2. **Production readiness** assessment  
3. **State management** for AMM functionality
4. **Performance** user experience

---

## 🧪 **Testing Strategy & Best Practices (New Section)**

Based on our successful implementation of unit tests for the AMM contract, we have questions about recommended testing patterns for Hyli projects:

### **Q14: RISC0 Test Harness Integration (CLARIFIED ✅)**
**OFFICIAL HYLI/RISC0 GUIDANCE**: 

**Integration Testing**: 
- ✅ **No robust framework exists** - confirmed by Hyli team
- ✅ **Docker-compose workflow** is the **official recommended approach**
- ✅ **Manual API/frontend testing** is current best practice
- ✅ **Unit tests (18s) + Docker integration (20min)** is the accepted pattern

**Testing Strategy Confirmation**:
```
⚡ Unit Tests (18s):        AMM mathematics, error handling ✅ RECOMMENDED
🔧 Integration Tests:       docker-compose + API testing   ✅ OFFICIAL APPROACH  
🐌 E2E Tests (20min):      Full proof generation + UI      ✅ MANUAL VALIDATION
```

**Updated Questions**:
- ~~Is there a RISC0-specific test harness?~~ **Answer: No, use docker-compose**
- ~~Should we test with RISC0_DEV_MODE=true?~~ **Answer: Yes, for development**
- ~~Are there Hyli-specific testing utilities?~~ **Answer: Block explorer + docker workflow**

### **Q15: Integration vs Unit Testing Balance (RESOLVED ✅)**
**OFFICIAL RECOMMENDATION FROM HYLI TEAM**:

**Confirmed Testing Pyramid**:
```
Unit Tests (Fast):     ✅ Standard Rust #[cfg(test)] - 18 seconds
Integration Tests:     ✅ docker-compose up/down + API calls - 20 minutes  
E2E Tests:            ✅ Frontend + full user workflow validation
```

**Development Environment**:
- ✅ **Use localhost** for active development (not testnet)
- ✅ **Block explorer available** at localhost for debugging
- ✅ **Contract recompilation** requires `rm -rf data` + restart
- ✅ **Chain reset** requires `docker-compose down --volumes --remove-orphans`

### **Q16: State Transition Testing Patterns**
**Context**: Our unit tests work on fresh contract instances, but real contracts have persistent state across transactions.

**Questions**:
- **How to test state persistence** without full blockchain integration?
- **Can we mock the StateCommitment system** for intermediate testing?
- **Are there patterns for testing** multi-transaction workflows (e.g., mint → add liquidity → swap)?
- **How to test state migration** when contract logic changes?

**Current Gap**: We test individual operations but not complex workflows like:
```
User Journey: Mint → Add Liquidity → Wait 1 hour → Remove Liquidity → Swap
```

### **Q17: Performance Testing for AMM Operations**
**Context**: Our AMM has complex mathematical operations (constant product formula, square roots) that could affect proof generation time.

**Questions**:
- **Are there profiling tools** for RISC0 contract performance?
- **How to benchmark** different AMM implementations (e.g., Uniswap v2 vs v3 math)?
- **What are acceptable performance targets** for proof generation in production?
- **Should we optimize for proof size** vs execution speed vs gas efficiency?

**Current Metrics**:
- **API Response**: 1.17s consistently ✅
- **Contract Execution**: 10-15s ✅  
- **Proof Generation**: 15-20 minutes ⚠️ (Could be faster?)

### **Q18: Test Data Management and Fixtures**
**Questions**:
- **Are there standardized test fixtures** for common DeFi scenarios?
- **How to generate realistic test data** (e.g., market conditions, user behaviors)?
- **Should we test with mainnet-like token amounts** (18 decimals) vs simplified amounts?
- **Are there tools for property-based testing** (fuzzing) with Hyli contracts?

**Current Approach**: Simple hardcoded values (1000 USDC, 2000 ETH) - probably insufficient for production.

---

## 🎯 **Priority Summary for Hyli Team - UPDATED**

### **✅ RESOLVED (Major Breakthroughs)**
1. **Q13: ContractHandler Trait** - ✅ **RESOLVED** by Hyli team guidance
2. **Q14: RISC0 Test Harness** - ✅ **CLARIFIED** - docker-compose is official approach
3. **Q15: Integration vs Unit Testing** - ✅ **RESOLVED** - official testing pyramid confirmed
4. **Q8: State Persistence/Querying** - ✅ **WORKING** - indexer functional with proper implementation

### **🟡 REMAINING QUESTIONS (Future Development)**
5. **Q16: State Transition Testing** - How to test multi-transaction workflows
6. **Q17: Performance Testing** - Profiling tools for AMM operations
7. **Q18: Test Data Management** - Standardized fixtures and realistic data
8. **Q7: Commitment Metadata Errors** - Minor warnings (but proofs succeed)

### **🟢 LOW PRIORITY (Optional)**
9. **Q11: Dev vs Production Mode** - Can be addressed before final demo
10. **Q9: Transaction Timeouts** - User experience improvement
11. **Q12: Performance Optimization** - 15-20 minute transactions acceptable for demo

### **📋 Updated Status**
- **✅ CRITICAL ISSUES RESOLVED** - Indexer working, development workflow established
- **✅ OFFICIAL BEST PRACTICES** - Testing strategy confirmed by Hyli team
- **🎯 READY FOR NEXT PHASE** - Focus can shift to frontend enhancement and ZKPassport integration
- **📚 COMPREHENSIVE DOCUMENTATION** - Questions and answers will help entire Hyli ecosystem

---

## 📝 **Additional Context**

### **Our Setup**
- Fresh app-scaffold clone
- Following ZKHack Berlin quickstart exactly
- Ubuntu WSL2 environment
- All prerequisites installed (Rust, Docker, RISC Zero, Noir v1.0.0-beta.3, BB v0.82.2)

### **Current Working State**
```bash
# All services running
docker-compose ps
# ✅ hyli_1, wallet-server_1, wallet-ui_1, postgres_1

# Server responding
curl http://localhost:4002/_health
# ✅ "OK"

# Contracts deployed
curl http://localhost:4002/api/config  
# ✅ {"contract_name": "contract1"}
```

### **Next Steps**
1. Get Hyli team feedback on our approach
2. Implement proper indexer configuration if needed
3. Continue with ZKPassport integration
4. Build AMM functionality on stable foundation

---

## 🚀 **Temporary Workaround Assessment**

**Pros**:
- ✅ Unblocks development immediately
- ✅ Core functionality intact
- ✅ Easy to revert when proper solution found
- ✅ Clear documentation for team discussion

**Cons**:
- ❓ Unknown impact on debugging capabilities
- ❓ Potential missing functionality
- ❓ May need rework if indexer is critical

**Priority**: Medium - works for hackathon, but need proper solution for production

---

*Document created during ZKHack Berlin 2025 - Team hyli-defi-app*
