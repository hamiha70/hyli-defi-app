# ZK-Gated AMM Implementation Plan for Hyli

## 📊 Project Overview

We aim to build a zero-knowledge identity-gated Automated Market Maker (AMM) on the Hyli blockchain as part of ZK Hack Berlin 2025. The system will allow only non-US citizens to execute swaps by integrating ZKPassport for private identity verification. The Hyli blockchain's support for multiple zk proof systems and native composition will be leveraged to atomically validate identity and execute trades.

### Key Features
- **Identity-Gated Trading**: Only verified non-US citizens can trade
- **ZKPassport Integration**: Private identity verification without revealing personal data
- **Native Proof Composition**: Atomic transactions combining identity + swap proofs
- **Modern AMM**: Constant product formula (x*y=k) with slippage protection
- **Multi-Token Support**: ERC20-style tokens using Merkle tree state

### Risk Assessment
- **Complexity Level:** Medium-High
- **Risk Level:** Medium (manageable with proper phases)
- **Success Probability:** 75-80% (high feasibility with current tools)

---

## 🎯 Phase-by-Phase Implementation Strategy

### Phase 1: Foundation & Token Infrastructure ⭐ *[LOW RISK]*
**Duration:** 2-3 days  
**Testable Outcome:** Basic token operations working

#### Step 1.1: Token Contract Implementation
- Replace `contract1` with TokenA (fungible token contract)
- Replace `contract2` with TokenB (fungible token contract)
- Implement core functions:
  - `mint(to: Address, amount: u128)`
  - `transfer(from: Address, to: Address, amount: u128)`
  - `balance_of(address: Address) -> u128`
  - `approve(spender: Address, amount: u128)`
- Use Merkle tree for balance state (following Hyli's hyli-smt-token pattern)
- Store only state commitment on-chain

**Risk Level:** LOW - Well-documented patterns exist  
**Tests:**
- Mint tokens to test accounts
- Transfer tokens between accounts
- Verify balance updates
- Test approval mechanisms

#### Step 1.2: Token Frontend Integration
- Update frontend to display token balances for both TokenA and TokenB
- Add simple mint/transfer UI components
- Show real-time balance updates
- Basic transaction status tracking

**Tests:**
- User can mint tokens through UI
- Transfer functionality works end-to-end
- Balance updates reflect immediately

---

### Phase 2: AMM Core Logic ⭐⭐ *[MEDIUM RISK]*
**Duration:** 3-4 days  
**Testable Outcome:** Basic AMM swaps working (without identity gating)

#### Step 2.1: AMM Contract Development
- Create new contract `amm_contract` (Rust/Risc0)
- Implement constant product formula: `x * y = k`
- Core AMM functions:
  - `add_liquidity(token_a_amount: u128, token_b_amount: u128)`
  - `remove_liquidity(liquidity_amount: u128)`
  - `swap(token_in: TokenType, amount_in: u128, min_amount_out: u128)`
  - `get_price(token_in: TokenType, amount_in: u128) -> u128`
- Handle slippage protection and price impact calculations
- Pool state management with reserves tracking

**Risk Level:** MEDIUM - AMM math is standard but state management complex  
**Tests:**
- Pool initialization with correct ratios
- Price calculations match expected formulas
- Slippage protection works correctly
- Large trade impact verification

#### Step 2.2: Multi-Contract Transaction Composition
- Integrate AMM with token contracts using Hyli's proof composition
- Single atomic transaction flow:
  1. Token approval blob
  2. AMM swap calculation blob
  3. Token transfer execution blobs
- Use Hyli's delegated calls pattern for token movements
- Ensure atomicity: all operations succeed or all fail

**Tests:**
- End-to-end swap in single transaction
- Failed swaps don't modify state
- Proper token balance updates
- Gas efficiency verification

#### Step 2.3: AMM Frontend Interface
- Build modern trading interface (Uniswap-style)
- Features:
  - Pool liquidity display
  - Real-time exchange rate calculation
  - Swap interface with slippage controls
  - Price impact warnings
  - Transaction history
- Beautiful, responsive design with modern UX

**Tests:**
- User can perform swaps through UI
- Price updates in real-time
- Slippage settings work correctly
- Mobile responsiveness

---

### Phase 3: Identity Integration ⭐⭐⭐ *[HIGH RISK - TECHNICAL]*
**Duration:** 4-5 days  
**Testable Outcome:** ZKPassport proof integration working

#### Step 3.1: ZKPassport SDK Integration
- Install and configure ZKPassport TypeScript SDK
- Create identity verification component in frontend
- Implement passport scanning/verification flow
- Generate "non-US citizen" SNARK proofs locally in browser
- Handle proof generation errors and user feedback

**Risk Level:** HIGH - External dependency, documentation may be limited  
**Mitigation:** Start early, prepare mock implementation fallback  
**Tests:**
- Generate valid ZKPassport proofs in browser
- Handle various passport types
- Error handling for invalid documents
- Proof serialization/deserialization

#### Step 3.2: Noir Identity Verifier Contract
- Create Noir-based contract for ZKPassport verification
- Register verifying key on Hyli network
- Implement proof verification logic:
  - Accept SNARK proof and public inputs
  - Verify against registered verifying key
  - Output boolean result (is_allowed)
- Handle edge cases and invalid proofs

**Risk Level:** HIGH - Noir/SNARK integration complexity  
**Mitigation:** Use Hyli's existing Noir examples as templates  
**Tests:**
- Submit valid ZKPassport proofs and verify on-chain
- Reject invalid or malformed proofs
- Public input validation
- Gas efficiency of verification

#### Step 3.3: Identity-Gated AMM Logic
- Modify AMM contract to require identity verification
- Add identity assertion in swap function:
  ```rust
  assert_eq!(identity_result.is_allowed, true);
  ```
- Integrate with identity contract through Hyli's proof composition
- Ensure swap fails atomically if identity verification fails

**Tests:**
- Swaps work only with valid identity proofs
- Invalid identity proofs reject entire transaction
- Identity-free transactions are blocked
- Error messages are clear and helpful

---

### Phase 4: Integration & UI Polish ⭐ *[LOW RISK]*
**Duration:** 2-3 days  
**Testable Outcome:** Complete user journey working smoothly

#### Step 4.1: End-to-End User Flow
- Complete user journey implementation:
  1. Wallet connection
  2. Identity verification (one-time or session-based)
  3. Token balance display
  4. Trading interface
  5. Transaction confirmation and tracking
- Comprehensive error handling and user feedback
- Transaction status polling and confirmation
- Session management for identity proofs

#### Step 4.2: UI/UX Enhancement
- Modern, beautiful trading interface with:
  - Clean, professional design
  - Identity status indicators
  - Real-time price charts (optional)
  - Transaction history panel
  - Loading states and animations
  - Responsive mobile design
- User onboarding flow
- Help documentation and tooltips

**Tests:**
- Smooth user experience from start to finish
- All error cases handled gracefully
- Mobile compatibility
- Accessibility compliance

---

### Phase 5: Optional Enhancements ⭐⭐ *[MEDIUM RISK]*
**Duration:** 2-3 days  
**Testable Outcome:** Performance optimizations and advanced features

#### Step 5.1: Boundless Integration *(Optional)*
- Integrate with RISC Zero's Boundless proving network
- Offload AMM proof generation to distributed provers
- Implement async proof handling
- Benefits: Faster proof generation, higher throughput

**Risk Level:** MEDIUM - New service integration  
**Tests:**
- Faster proof generation compared to local proving
- Higher transaction throughput
- Reliable proof delivery

#### Step 5.2: Advanced Features *(Optional)*
- Dynamic pool creation for new token pairs
- Liquidity provider rewards and fees
- Advanced price charts and analytics
- Portfolio tracking
- Multi-pair arbitrage detection

---

## 🚨 Risk Analysis & Mitigation Strategies

### High Risk Areas

#### 1. ZKPassport Integration
- **Risk:** External dependency, potentially limited documentation
- **Impact:** Could block core functionality
- **Mitigation:**
  - Start ZKPassport integration early in Phase 1
  - Prepare mock identity verification as fallback
  - Contact ZKPassport team for support if needed
  - Test with multiple passport types

#### 2. Noir Contract Development
- **Risk:** Different proving system from Risc0, complex SNARK verification
- **Impact:** Identity verification may not work
- **Mitigation:**
  - Use Hyli's existing Noir examples as templates
  - Start with simple proof verification first
  - Test verifying key registration early
  - Have backup plan for simpler identity check

#### 3. Proof Composition Complexity
- **Risk:** Multiple contracts in one transaction may fail
- **Impact:** Atomic transactions may not work as expected
- **Mitigation:**
  - Test each contract individually first
  - Use Hyli's proven composition patterns
  - Implement comprehensive error handling
  - Test failure scenarios extensively

### Medium Risk Areas

#### 1. AMM Mathematics and State Management
- **Risk:** Complex calculations and state transitions
- **Impact:** Incorrect pricing or failed swaps
- **Mitigation:**
  - Use battle-tested AMM formulas
  - Extensive unit testing for edge cases
  - Test with various pool sizes and ratios
  - Implement comprehensive slippage protection

#### 2. Frontend State Management
- **Risk:** Complex UI state with multiple contracts
- **Impact:** Poor user experience, inconsistent data
- **Mitigation:**
  - Build incrementally with clear state management
  - Test each UI component thoroughly
  - Implement proper loading and error states
  - Use established React patterns

### Low Risk Areas

#### 1. Token Contracts
- **Risk:** Well-established patterns exist
- **Mitigation:** Follow Hyli's hyli-smt-token examples

#### 2. Basic UI Components
- **Risk:** Standard React development
- **Mitigation:** Use proven UI libraries and patterns

---

## 📋 Implementation Options

### Option A: Full Implementation *(Recommended)*
- **Scope:** Complete all phases as described
- **Timeline:** 12-15 days
- **Success Rate:** 75%
- **Impact:** High - demonstrates full capability
- **Pros:** Complete solution, maximum impact for hackathon
- **Cons:** Higher risk, longer timeline

### Option B: Core AMM + Mock Identity
- **Scope:** Implement AMM without real ZKPassport integration
- **Timeline:** 8-10 days
- **Success Rate:** 90%
- **Impact:** Medium - working AMM with simulated identity
- **Pros:** Lower risk, faster delivery
- **Cons:** Not truly innovative, missing key differentiator

### Option C: Progressive Implementation *(Recommended)*
- **Scope:** Start with Option B, then upgrade to real identity verification
- **Timeline:** 10-12 days
- **Success Rate:** 85%
- **Impact:** High - working solution with upgrade path
- **Pros:** Reduces risk while maintaining progress, allows pivoting
- **Cons:** Slightly longer than Option B

---

## 🧪 Testing Strategy

### Unit Testing
- **Token Contracts:**
  - Mint, transfer, balance operations
  - Edge cases: zero amounts, insufficient balance
  - State consistency after operations

- **AMM Contract:**
  - Price calculations for various scenarios
  - Liquidity operations
  - Slippage protection
  - Mathematical invariants (x*y=k)

- **Identity Contract:**
  - Proof verification with valid/invalid proofs
  - Public input validation
  - Edge cases and malformed data

### Integration Testing
- **Multi-Contract Transactions:**
  - Atomic swap operations
  - Failure rollback scenarios
  - Cross-contract state consistency

- **End-to-End User Flows:**
  - Complete trading journey
  - Error handling at each step
  - Session management

### Demo Scenarios

#### 1. Happy Path Scenario
```
User connects wallet → 
Verifies identity with ZKPassport → 
Views token balances → 
Adds liquidity to pool → 
Performs successful swap → 
Views updated balances
```

#### 2. Rejection Scenario
```
User connects wallet → 
Attempts swap without identity verification → 
Gets clear rejection message → 
Completes identity verification → 
Successfully performs swap
```

#### 3. Edge Case Testing
```
Large swaps with high price impact → 
Low liquidity scenarios → 
Slippage limit testing → 
Network failure recovery → 
Invalid proof handling
```

---

## 💡 Recommended Starting Approach

**I recommend Option C: Progressive Implementation** for the following reasons:

### Why Progressive Implementation?

1. **Risk Mitigation:** Start with lower-risk components first
2. **Early Value:** Working AMM within first week provides demo value
3. **Flexibility:** Can pivot if ZKPassport integration proves problematic
4. **Testable Milestones:** Each phase delivers functional improvements
5. **Team Confidence:** Early wins build momentum for complex parts

### Sprint Breakdown

#### Sprint 1 (Days 1-4): Foundation
- **Goal:** Working token contracts and basic AMM
- **Deliverable:** Users can mint tokens and perform basic swaps
- **Risk:** Low
- **Tests:** Token operations and simple swaps work

#### Sprint 2 (Days 5-8): AMM Enhancement
- **Goal:** Complete AMM functionality with beautiful UI
- **Deliverable:** Production-ready AMM interface
- **Risk:** Medium
- **Tests:** Complex swaps, liquidity operations, UI responsiveness

#### Sprint 3 (Days 9-12): Identity Integration
- **Goal:** Real ZKPassport integration and identity gating
- **Deliverable:** Identity-gated AMM with ZK proofs
- **Risk:** High
- **Tests:** Identity verification and gated trading

This approach ensures we have a working AMM by day 4, a beautiful trading platform by day 8, and the full ZK-gated solution by day 12, with buffer time for polish and debugging.

---

## 🔗 Technical References

### Core Technologies
- **Hyli Documentation:** https://docs.hyli.org
- **Hyli Scaffold:** https://github.com/hyli-org/app-scaffold
- **ZKPassport SDK:** https://github.com/0xpass/zkpassport
- **Risc0 zkVM:** https://github.com/risc0/risc0
- **Boundless:** https://github.com/risc0/boundless

### Example Contracts
- **Faucet Contract:** https://github.com/hyli-org/faucet
- **Wallet Contract:** https://github.com/hyli-org/wallet
- **Hyli Explorer:** https://explorer.hyli.org

### Key Documentation
- **Hyli vs Traditional Blockchains:** Understanding the proof-composition model
- **Multi-Proof Transactions:** How to combine Noir and Risc0 proofs
- **Identity Management:** Session keys and credential verification
- **AMM Mathematics:** Constant product formula implementation

---

## 📅 Next Steps

1. **Immediate:** Begin Phase 1, Step 1.1 - Token contract implementation
2. **Day 1-2:** Complete token contracts and basic testing
3. **Day 3-4:** Start AMM contract development
4. **Weekly:** Review progress and adjust timeline if needed

**Ready to start coding!** 🚀

---

*This implementation plan will be updated as development progresses and new insights are gained.*