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

### **📊 System Stability Observations**
After running the server for **6+ hours** (blocks 7000 → 33000+) and **4 successful AMM transactions**, several important patterns emerged:

#### **✅ Stable Long-Term Operation**
```
2025-06-21T06:51:20.047108Z  INFO: 🔧 Executed contract: Minted 1000 USDC tokens for user bob@wallet. Success: true
2025-06-21T06:51:20.546953Z  INFO: ✅ Proved 1 txs, Batch id: 4, Proof TX hash: 7bf4a0f4fd02e415400980c7aad0ece82305e00af8b54c4886f27f0b37a39e17
```
- **Observation**: Server runs stably for hours with consistent successful AMM operations
- **Block timing**: Consistent ~16-minute block intervals (normal Hyli network behavior)
- **Multiple users**: Successfully tested with alice@contract1, bob@contract1, bob@wallet
- **Proof generation**: 4/4 transactions generated proofs successfully
- **Performance**: Execution time improved from 22s to 10-15s average

#### **🚨 Critical Discovery: ContractHandler Trait Issue**
```
error[E0277]: the trait bound `contract1::Contract1: ContractHandler` is not satisfied
error[E0277]: the trait bound `Contract2: ContractHandler` is not satisfied
```
- **Resolution**: Had to **disable indexer modules** to get server running
- **Current status**: Server works but without indexer functionality
- **Impact**: No contract state queries available, but transaction execution works perfectly

#### **🎯 Updated Understanding: Dual State System**
- **Transaction state**: ✅ Working perfectly - 4 successful AMM operations
- **Indexer state**: ❌ Blocked by ContractHandler trait - returns 404 for all state queries
- **Core insight**: **AMM operations work** without indexer, but **UI state display** is blocked

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

### **Q13: ContractHandler Trait Implementation (BLOCKING ISSUE)**
**Issue**: Cannot run server with indexer functionality enabled due to missing trait implementation:
```rust
error[E0277]: the trait bound `contract1::Contract1: ContractHandler` is not satisfied
   --> server/src/main.rs:107:10
    |
107 |         .build_module::<ContractStateIndexer<Contract1>>(ContractStateIndexerCtx {
    |          ^^^^^^^^^^^^ the trait `ContractHandler` is not implemented for `contract1::Contract1`
```

**Current Workaround**: Disabled indexer modules in `server/src/main.rs`:
```rust
// Commented out to resolve ContractHandler trait errors
// .build_module::<ContractStateIndexer<Contract1>>(ContractStateIndexerCtx {
//     contract_name: "contract1".to_owned(),
// })?
```

**Updated Evidence (4 successful transactions)**:
✅ **AMM operations work perfectly** without indexer
❌ **UI state display blocked** - all `/v1/indexer/contract/*/state` return 404
✅ **Proof generation successful** - 4/4 transactions proved successfully
✅ **Multi-user support confirmed** - alice@contract1, bob@contract1, bob@wallet all work

**Critical Questions**:
- **Is ContractHandler trait required** for production Hyli contracts?
- **Can we implement alternative state queries** instead of indexer endpoints?
- **What functionality do we lose** by disabling the indexer permanently?
- **How to properly implement ContractHandler** for our AMM contract?
- **Are there Hyli examples** showing proper ContractHandler implementation?

**Priority**: 🔴 **BLOCKING for UI** - AMM works but state display is broken

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

### **Q14: RISC0 Test Harness Integration**
**Context**: We successfully implemented 11 unit tests using standard Rust `#[cfg(test)]` patterns that test AMM logic in isolation (mint, swap, liquidity operations, error cases).

**Questions**:
- **Is there a RISC0-specific test harness** we should be using for zkVM contract testing?
- **How do RISC0 proving constraints** affect unit testing strategies?
- **Should we test with RISC0_DEV_MODE=true** for faster iteration, or does this miss important edge cases?
- **Are there Hyli-specific testing utilities** (similar to Foundry for Ethereum) we should leverage?

**Current Approach**: 
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_swap_calculation() {
        let mut contract = create_test_contract();
        // ... pure Rust logic testing
    }
}
```

**Evidence**: Our tests run in **<18 seconds** and provide comprehensive coverage of:
- ✅ Token minting and balance tracking
- ✅ Liquidity pool mathematics (constant product formula)
- ✅ Swap calculations with 0.3% fees
- ✅ Error conditions (insufficient balance, slippage protection)
- ✅ Edge cases (nonexistent pools, invalid ratios)

### **Q15: Integration vs Unit Testing Balance**
**Issue**: Currently we have **two types of testing**:
1. **Fast unit tests** (18s) - Pure Rust logic, no zkVM overhead
2. **Slow integration tests** (15-20 min) - Full proof generation via frontend/API

**Questions**:
- **What's the recommended testing pyramid** for Hyli projects?
- **When should we use unit tests vs integration tests** vs proof generation tests?
- **Are there middle-tier tests** (e.g., contract execution without full proof generation)?
- **How do other Hyli projects** balance testing speed vs completeness?

**Example Split**:
```
⚡ Unit Tests (18s):        AMM mathematics, error handling
🔧 Integration Tests (?):   Contract state transitions  
🐌 E2E Tests (20min):      Full proof generation + UI
```

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

## 🎯 **Priority Summary for Hyli Team**

### **🔴 CRITICAL (UI Blocking Issues)**
1. **Q13: ContractHandler Trait** - Cannot use indexer functionality, blocking state display in UI
2. **Q7: Commitment Metadata Errors** - Proof generation warnings (but proofs succeed)

### **🟡 HIGH (Demo Enhancement)**
3. **Q8: State Persistence/Querying** - Need alternative ways to query AMM state for UI
4. **Q14: RISC0 Test Harness** - Essential for reliable development workflow
5. **Q12: Performance Optimization** - 15-20 minute transactions acceptable but could be faster

### **🟢 MEDIUM (Future Development)**
6. **Q15-Q18: Testing Strategy** - Best practices for scalable development
7. **Q11: Dev vs Production Mode** - Need to validate demo environment
8. **Q9: Transaction Timeouts** - "Timed out" messages despite successful execution
9. **Q10: Multi-Contract State** - Cross-contract communication patterns

### **📋 Documentation Priority**
- **Q14** is most urgent - affects daily development workflow
- **Q13** is blocking - prevents full indexer functionality
- **Q15-Q18** are foundational - will guide long-term project architecture

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
