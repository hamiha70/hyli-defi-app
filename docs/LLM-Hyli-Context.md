# Hyli Blockchain Ecosystem - Technical Context

## 🌐 Network Status & Availability

### Current Network State
- **Status:** Public Testnet (Mainnet not live yet)
- **Launch:** June 2025 (invite-only initially)
- **Phases:**
  - **Phase 1:** "Play and learn" with curated demo apps
  - **Phase 2:** Open developer deployment (coming weeks)
- **Mainnet:** Planned after testnet validation (timeline TBD)

---

## 🎮 Live Applications on Hyli Testnet

### Core Infrastructure Apps

#### **Hyli Wallet & Leaderboard**
- **Purpose:** Unified wallet/identity across all testnet apps
- **Features:** 
  - Username/password login (no MetaMask required)
  - Cross-app achievement tracking
  - One account works with every Hyli app
- **Innovation:** Demonstrates identity abstraction without bridges

#### **On-Chain Order Book DEX**
- **Type:** Central Limit Order Book (CLOB)
- **Purpose:** Traditional exchange model fully on-chain
- **Benefits:** High throughput, familiar trading interface
- **Demo:** Showcases DeFi capabilities on Hyli

### Gaming & Entertainment Apps

#### **ORANJ Faucet Game**
- **Type:** Fruit Ninja-style token faucet
- **Mechanics:** Slice oranges to earn $ORANJ tokens
- **ZK Feature:** Every slice verified by ZK proof
- **Tech Stack:** Multi-contract proof composition (faucet + token)

#### **Orange Trail Game**
- **Type:** Risk-based exploration game
- **Mechanics:** Each step is a bet on safe return
- **Demo:** Illustrates randomness and risk via proofs

#### **eZKasino**
- **Type:** Provable on-chain blackjack
- **Interface:** Windows XP-style gaming UI
- **ZK Feature:** All game logic (cards, outcomes) proven fair
- **Innovation:** Zero-knowledge casino with verifiable randomness

#### **ZK Chat**
- **Type:** Global on-chain messaging
- **Features:** Messages posted on-chain with ZK safeguards
- **Demo:** Social applications leveraging Hyli architecture

#### **Hyligotchi** *(Coming Soon)*
- **Type:** Tamagotchi-like virtual pet game

---

## 🏦 DeFi Capabilities & Token Standards

### Existing DeFi Infrastructure

#### **AMM History: Hyleoof**
- **Background:** Earlier zkAMM demo built by Hyli team
- **Features:**
  - Trustless, non-custodial AMM
  - Constant-product formula (x*y=k)
  - Atomic transaction composition
- **Tokens:** Hyllar and Hyllar2 test tokens
- **Innovation:** Single transaction with multiple blobs:
  - Identity authentication
  - Token approvals
  - AMM swap logic
  - Token transfers
- **Benefits:** One signature, one fee instead of multiple transactions

### Token Implementation

#### **Native Token Standards**
- **Architecture:** Not EVM-based, no ERC-20 contracts
- **Implementation:** Rust/Risc0 contracts with Merkle tree state
- **Examples:**
  - `$ORANJ`: Main testnet currency
  - `Hyllar` tokens: AMM demo tokens
- **Storage:** State commitment on-chain, full ledger off-chain

#### **Available Libraries**
- **hyle-smt-token:** Sparse Merkle tree token implementation
- **hyle-amm:** AMM logic patterns
- **Standard Functions:** mint, transfer, approve (similar to ERC-20)

---

## 🏗️ Technical Architecture

### Core Design Philosophy

#### **Proof-Powered L1**
- **Execution:** Off-chain computation with on-chain verification
- **State:** Minimal on-chain storage (commitments only)
- **Verification:** Native support for multiple proof systems
- **Benefits:** High throughput, low storage costs

### Transaction Flow

#### **Two-Step Process**
1. **Sequencing:** Immediate transaction ordering (instant feedback)
2. **Proving:** Asynchronous proof generation and verification
3. **Settlement:** Final state update after proof validation

#### **Proof Composition**
- **Multiple Systems:** Mix Risc0, Noir, SP1 in one transaction
- **Atomicity:** All proofs must pass or transaction fails
- **No Recursion:** Native composition without complex aggregation

### Consensus & Performance

#### **Autobahn BFT Consensus**
- **Type:** High-performance Proof-of-Stake
- **Features:** Fast finality, network resilience
- **Performance:** Optimized for high TPS with minimal latency
- **Publication:** State-of-the-art protocol (2024)

---

## 💻 Developer Experience

### Supported Languages & Proving Systems

#### **Rust (Risc0 zkVM)**
- **Use Case:** Complex stateful contracts
- **Benefits:** Full programming language with libraries
- **Examples:** Token contracts, AMM logic, games
- **Compilation:** RISC-V bytecode for zk-STARK VM

#### **Noir (zkSNARKs)**
- **Use Case:** Arithmetic circuits, identity verification
- **Benefits:** Succinct proofs for specific computations
- **Examples:** Password verification, identity checks
- **Integration:** Native SNARK verification on Hyli

#### **Succinct SP1**
- **Use Case:** General computation with SNARK proofs
- **Benefits:** LLVM-based proving system
- **Integration:** Multi-scheme transactions with Risc0

#### **Future Support**
- Cairo/Starknet VM
- Custom ZK circuits
- Trusted hardware (TEEs)

### Smart Contract Model

#### **App Structure**
- **Identity:** Unique name + verifier type + program ID
- **State:** Off-chain storage with on-chain commitment
- **Verification:** Program hash or SNARK verifying key
- **Updates:** Proof-verified state transitions

#### **Cross-Contract Interaction**
- **Composition:** Multiple contracts in one transaction
- **Calls:** Trustless interaction via proof verification
- **Examples:** AMM + token transfers, faucet + minting

---

## 🛠️ Development Tooling

### Local Development

#### **Docker Environment**
- **Setup:** Simple `docker-compose up` for local devnet
- **Components:** Hyli node + wallet server
- **Benefits:** No complex setup, immediate testing

#### **SDKs & APIs**
- **Rust SDK:** Official client library for transaction construction
- **TypeScript:** Bindings for frontend integration
- **APIs:** JSON-RPC for chain interaction
- **Explorer:** Web-based transaction and state viewing

### Scaffolding & Examples

#### **App Scaffold**
- **Repository:** https://github.com/hyli-org/app-scaffold
- **Components:**
  - Frontend (React/Vue with wallet integration)
  - Contracts (Risc0 and SP1 examples)
  - Backend (autoprover service)
- **Development Mode:** `RISC0_DEV_MODE=true` for fast iteration

#### **Reference Implementations**
- **Faucet:** Token minting with game logic
- **Wallet:** Identity management with Noir
- **Explorer:** Block and transaction browser

### Identity & Session Management

#### **Account Abstraction**
- **Model:** Session keys instead of transaction signing
- **Flow:** One-time authentication → session capabilities
- **Benefits:** Web2-like UX with Web3 security
- **Integration:** Built-in wallet contract for identity

---

## 🔗 Key Resources

### Official Documentation
- **Main Docs:** https://docs.hyli.org
- **Developer Hub:** Guides, quickstarts, and references
- **Blog:** https://blog.hyli.org (technical deep dives)
- **Explorer:** https://explorer.hyli.org

### GitHub Repositories
- **Core Node:** https://github.com/hyli-org/hyli
- **App Scaffold:** https://github.com/hyli-org/app-scaffold
- **Examples:**
  - Faucet: https://github.com/hyli-org/faucet
  - Wallet: https://github.com/hyli-org/wallet
  - Explorer: https://github.com/hyli-org/explorer

### Community & Support
- **Telegram:** Builder Group for technical questions
- **Documentation:** ZK Hack Berlin quickstart guide
- **Templates:** Ready-to-use project structures

---

## 🎯 Key Advantages for DeFi Development

### **Atomic Composition**
- Single transaction for complex operations
- No multi-step approval workflows
- Reduced gas costs and improved UX

### **Language Flexibility**
- Choose optimal proving system per use case
- Noir for identity, Rust for stateful logic
- No vendor lock-in to single VM

### **High Performance**
- Off-chain execution with on-chain verification
- Parallel proof generation and verification
- Optimized consensus for fast finality

### **Developer Experience**
- Familiar languages (Rust, TypeScript)
- Rich tooling and scaffolding
- Local development environment
- Clear documentation and examples

This ecosystem provides a solid foundation for building innovative DeFi applications with zero-knowledge proofs, combining the benefits of multiple proving systems in a single, high-performance blockchain.