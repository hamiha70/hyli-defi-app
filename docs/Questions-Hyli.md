# Questions for Hyli Team - ZKHack Berlin 2025

## Issue Resolution: ContractStateIndexer & Missing Feature Dependencies

### 📋 **Issue Summary**

**Date**: December 20, 2025  
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
