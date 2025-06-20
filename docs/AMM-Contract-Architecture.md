# AMM Contract with Integrated Token Management

## 📋 **Overview**

This document describes the Automated Market Maker (AMM) contract implementation for the Hyli DeFi app, featuring integrated token management designed for ZKHack Berlin 2025.

**Key Innovation**: Unlike traditional EVM-based AMMs that require separate ERC-20 contracts for each token, our implementation includes built-in token management within the AMM contract itself, optimized for rapid hackathon development while maintaining production-grade AMM mathematics.

---

## 🏗️ **Architecture Design**

### **Hybrid Architecture Choice**

We chose a **hybrid approach** that balances development speed with functionality:

```
Traditional EVM Approach:
├── Token A Contract (ERC-20)
├── Token B Contract (ERC-20)  
└── AMM Contract (DEX logic)

Our Hyli Approach:
├── AMM Contract (DEX logic + token management)
└── Identity Contract (ZKPassport integration)
```

### **Benefits of This Design**

✅ **Hackathon-Optimized**: Single contract for all token operations  
✅ **Complete Functionality**: Full AMM with balance tracking  
✅ **ZK-Native**: Designed for Hyli's proof composition  
✅ **Future-Proof**: Easy to refactor to separate contracts later  

---

## 🔧 **Contract Components**

### **Core Data Structures**

#### **AmmContract State**
```rust
pub struct AmmContract {
    pools: HashMap<String, LiquidityPool>,           // Token pair pools
    user_balances: HashMap<String, u128>,            // User token balances
}
```

#### **LiquidityPool**
```rust
pub struct LiquidityPool {
    pub token_a: String,        // First token symbol
    pub token_b: String,        // Second token symbol  
    pub reserve_a: u128,        // Token A reserves
    pub reserve_b: u128,        // Token B reserves
    pub total_liquidity: u128,  // Total LP tokens minted
}
```

#### **User Balance Key Format**
- **Token Balance**: `"user_token"` → `"alice_USDC"` = 1000
- **Liquidity Position**: `"user_liquidity_tokenA_tokenB"` → `"alice_liquidity_USDC_ETH"` = 50

---

## 🚀 **Available Functions**

### **1. Token Management**

#### **MintTokens**
```rust
AmmAction::MintTokens { 
    user: String, 
    token: String, 
    amount: u128 
}
```
**Purpose**: Create test tokens for development  
**Use Case**: Fund user accounts for testing AMM functionality  
**Note**: In production, this would be handled by separate token contracts

#### **GetUserBalance**  
```rust
AmmAction::GetUserBalance { 
    user: String, 
    token: String 
}
```
**Purpose**: Query user's token balance  
**Returns**: Current balance for specified user and token

### **2. Liquidity Management**

#### **AddLiquidity**
```rust
AmmAction::AddLiquidity {
    user: String,
    token_a: String,
    token_b: String, 
    amount_a: u128,
    amount_b: u128
}
```

**Process**:
1. Validates user has sufficient token balances
2. Calculates liquidity ratio (for existing pools)
3. Updates pool reserves
4. Mints LP tokens proportional to contribution
5. Deducts tokens from user balances

**Math**: 
- **Initial Liquidity**: `LP_tokens = sqrt(amount_a * amount_b)`
- **Subsequent**: `LP_tokens = (amount_a * total_liquidity) / reserve_a`

#### **RemoveLiquidity**
```rust
AmmAction::RemoveLiquidity {
    user: String,
    token_a: String,
    token_b: String,
    liquidity_amount: u128
}
```

**Process**:
1. Validates user has sufficient LP tokens
2. Calculates proportional token amounts to return
3. Burns LP tokens
4. Returns tokens to user balance

**Math**: `amount_out = (liquidity_amount * reserve) / total_liquidity`

### **3. Trading**

#### **SwapExactTokensForTokens**
```rust
AmmAction::SwapExactTokensForTokens {
    user: String,
    token_in: String,
    token_out: String,
    amount_in: u128,
    min_amount_out: u128
}
```

**Process**:
1. Validates user has sufficient input token balance
2. Calculates output using constant product formula
3. Applies 0.3% trading fee
4. Checks slippage protection (`min_amount_out`)
5. Updates pool reserves and user balances

**Math (Constant Product Formula)**:
```
(x + Δx) * (y - Δy) = x * y
Δy = (y * Δx * 997) / (x * 1000 + Δx * 997)  // 0.3% fee
```

### **4. Information Queries**

#### **GetReserves**
```rust
AmmAction::GetReserves { 
    token_a: String, 
    token_b: String 
}
```
**Purpose**: Get current pool reserves and total liquidity  
**Returns**: Reserve amounts and LP token supply

---

## 🧮 **AMM Mathematics**

### **Constant Product Formula**
Our AMM implements the proven **x * y = k** model:

- **x**: Reserve of token A
- **y**: Reserve of token B  
- **k**: Constant product (must remain constant)

### **Price Calculation**
```
Price of A in terms of B = reserve_b / reserve_a
Price of B in terms of A = reserve_a / reserve_b
```

### **Slippage Protection**
Users can specify `min_amount_out` to protect against:
- Front-running attacks
- High slippage due to large trades
- Price movements during transaction processing

### **Fee Structure**
- **Trading Fee**: 0.3% (standard Uniswap model)
- **Fee Distribution**: Added to pool reserves (benefits all LPs)
- **No Protocol Fee**: Simplified for hackathon

---

## 🔐 **Security Features**

### **Balance Validation**
- All operations check sufficient user balances before execution
- Prevents overdraft and ensures atomic transactions

### **Pool Integrity**
- Liquidity ratio validation prevents pool manipulation
- Minimum output protection for swaps

### **Consistent Pair Keys**
```rust
fn get_pair_key(&self, token_a: &str, token_b: &str) -> String {
    let mut tokens = [token_a, token_b];
    tokens.sort();  // Ensures "ETH_USDC" = "USDC_ETH"
    format!("{}_{}", tokens[0], tokens[1])
}
```

---

## 🧪 **Testing Workflow**

### **1. Setup Test Environment**
```bash
# Start Hyli node
docker-compose up -d

# Start server  
RISC0_DEV_MODE=true cargo run -p server

# Start frontend
cd front && bun run dev
```

### **2. Basic AMM Testing Sequence**

#### **Step 1: Mint Test Tokens**
```json
{
  "user": "alice",
  "token": "USDC", 
  "amount": 10000
}
```

#### **Step 2: Create Liquidity Pool**
```json
{
  "user": "alice",
  "token_a": "USDC",
  "token_b": "ETH",
  "amount_a": 1000,
  "amount_b": 1
}
```

#### **Step 3: Test Token Swap**
```json
{
  "user": "alice", 
  "token_in": "USDC",
  "token_out": "ETH",
  "amount_in": 100,
  "min_amount_out": 0
}
```

#### **Step 4: Verify Balances**
- Check user balances after each operation
- Verify pool reserves match expected values
- Confirm LP token accounting

---

## 🔮 **Future Considerations**

### **Production Refactoring**
When moving to production, consider splitting into:

```
Token Contracts:
├── USDC Token Contract
├── ETH Token Contract  
└── Custom Token Contracts

AMM Infrastructure:
├── Core AMM Contract
├── Router Contract (multi-hop swaps)
└── Fee Management Contract
```

### **Advanced Features to Add**
- **Multi-hop routing**: Swap A→C via A→B→C
- **Time-weighted average prices**: Price oracles
- **Concentrated liquidity**: Uniswap V3 style ranges
- **Flash loans**: Borrow tokens within single transaction

### **ZKPassport Integration Points**
- **Identity-gated pools**: Require verification for certain tokens
- **Compliance checking**: Geographic restrictions on trading
- **Privacy-preserving KYC**: Prove eligibility without revealing identity

---

## 📊 **Performance Characteristics**

### **Gas Efficiency** (Hyli Advantages)
- **Batched operations**: Multiple actions in single proof
- **State-efficient**: HashMap storage optimized for zkVM
- **Proof composition**: Combine identity + trading proofs

### **Scalability**
- **Parallel proving**: Multiple pools can be proven simultaneously
- **Atomic multi-contract**: Identity verification + trading in one transaction

---

## 🛠️ **Development Notes**

### **Type Aliases for Compatibility**
```rust
pub type Contract1 = AmmContract;
pub type Contract1Action = AmmAction;
```
Maintains backward compatibility with existing server/frontend code.

### **Error Handling**
All functions return `Result<Vec<u8>, String>` with descriptive error messages:
- "Insufficient USDC balance"
- "Pool does not exist" 
- "Invalid liquidity ratio"

### **Helper Utilities**
- **Integer square root**: For geometric mean LP token calculation
- **Consistent pair keys**: Prevents duplicate pools for same pair

---

## 🎯 **Integration with ZKPassport**

This AMM contract is designed to work seamlessly with ZKPassport identity verification:

1. **User proves identity** via ZKPassport (Contract2)
2. **Identity verification result** passed to AMM contract
3. **Trading permissions** granted based on compliance status
4. **Atomic transactions** ensure both proofs succeed or fail together

---

*Built for ZKHack Berlin 2025 - Demonstrating privacy-preserving DeFi with zero-knowledge proofs* 