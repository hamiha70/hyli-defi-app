use borsh::{io::Error, BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use sdk::RunResult;

#[cfg(feature = "client")]
pub mod client;
#[cfg(feature = "client")]
pub mod indexer;

impl sdk::ZkContract for AmmContract {
    /// Entry point of the contract's logic
    fn execute(&mut self, calldata: &sdk::Calldata) -> RunResult {
        // Parse contract inputs
        let (action, ctx) = sdk::utils::parse_raw_calldata::<AmmAction>(calldata)?;

        // Execute the given action
        let res = match action {
            AmmAction::MintTokens { user, token, amount } => {
                self.mint_tokens(user, token, amount)?
            },
            AmmAction::AddLiquidity { user, token_a, token_b, amount_a, amount_b } => {
                self.add_liquidity(user, token_a, token_b, amount_a, amount_b)?
            },
            AmmAction::RemoveLiquidity { user, token_a, token_b, liquidity_amount } => {
                self.remove_liquidity(user, token_a, token_b, liquidity_amount)?
            },
            AmmAction::SwapExactTokensForTokens { user, token_in, token_out, amount_in, min_amount_out } => {
                self.swap_exact_tokens_for_tokens(user, token_in, token_out, amount_in, min_amount_out)?
            },
            AmmAction::GetReserves { token_a, token_b } => {
                self.get_reserves(token_a, token_b)?
            },
            AmmAction::GetUserBalance { user, token } => {
                self.get_user_balance(user, token)?
            },
        };

        Ok((res, ctx, vec![]))
    }

    /// Serialize the full AMM state on-chain
    fn commit(&self) -> sdk::StateCommitment {
        sdk::StateCommitment(self.as_bytes().expect("Failed to encode AMM state"))
    }
}

impl AmmContract {
    /// Mint tokens for testing purposes (would be separate contract in production)
    pub fn mint_tokens(&mut self, user: String, token: String, amount: u128) -> Result<Vec<u8>, String> {
        let balance_key = format!("{}_{}", user, token);
        let current_balance = *self.user_balances.get(&balance_key).unwrap_or(&0);
        self.user_balances.insert(balance_key, current_balance + amount);
        
        Ok(format!("Minted {} {} tokens for user {}", amount, token, user).into_bytes())
    }

    /// Get user token balance
    pub fn get_user_balance(&self, user: String, token: String) -> Result<Vec<u8>, String> {
        let balance_key = format!("{}_{}", user, token);
        let balance = *self.user_balances.get(&balance_key).unwrap_or(&0);
        
        Ok(format!("User {} has {} {} tokens", user, balance, token).into_bytes())
    }

    /// Add liquidity to a token pair pool
    pub fn add_liquidity(
        &mut self, 
        user: String,
        token_a: String, 
        token_b: String, 
        amount_a: u128, 
        amount_b: u128
    ) -> Result<Vec<u8>, String> {
        // Check user has sufficient balance - copy values to avoid borrow issues
        let balance_a_key = format!("{}_{}", user, token_a);
        let balance_b_key = format!("{}_{}", user, token_b);
        
        let user_balance_a = *self.user_balances.get(&balance_a_key).unwrap_or(&0);
        let user_balance_b = *self.user_balances.get(&balance_b_key).unwrap_or(&0);
        
        if user_balance_a < amount_a {
            return Err(format!("Insufficient {} balance", token_a));
        }
        if user_balance_b < amount_b {
            return Err(format!("Insufficient {} balance", token_b));
        }

        let pair_key = self.get_pair_key(&token_a, &token_b);
        
        // Ensure consistent token ordering (alphabetically)
        let mut tokens = [token_a.as_str(), token_b.as_str()];
        tokens.sort();
        let (sorted_token_a, sorted_token_b) = (tokens[0], tokens[1]);
        
        let pool = self.pools.entry(pair_key.clone()).or_insert(LiquidityPool {
            token_a: sorted_token_a.to_string(),
            token_b: sorted_token_b.to_string(),
            reserve_a: 0,
            reserve_b: 0,
            total_liquidity: 0,
        });

        // Map user amounts to sorted pool amounts
        let (pool_amount_a, pool_amount_b) = if token_a == sorted_token_a {
            (amount_a, amount_b) // token_a maps to pool.token_a, token_b maps to pool.token_b
        } else {
            (amount_b, amount_a) // token_a maps to pool.token_b, token_b maps to pool.token_a
        };

        let liquidity_minted;

        // For initial liquidity, just add the amounts
        if pool.total_liquidity == 0 {
            pool.reserve_a = pool_amount_a;
            pool.reserve_b = pool_amount_b;
            liquidity_minted = (pool_amount_a * pool_amount_b).integer_sqrt(); // geometric mean
            pool.total_liquidity = liquidity_minted;
        } else {
            // Calculate optimal amounts based on current ratio
            let ratio_a = pool_amount_a * pool.reserve_b;
            let ratio_b = pool_amount_b * pool.reserve_a;
            
            if ratio_a != ratio_b {
                return Err("Invalid liquidity ratio".to_string());
            }
            
            pool.reserve_a += pool_amount_a;
            pool.reserve_b += pool_amount_b;
            
            // Mint liquidity tokens proportional to contribution
            liquidity_minted = (pool_amount_a * pool.total_liquidity) / (pool.reserve_a - pool_amount_a);
            pool.total_liquidity += liquidity_minted;
        }

        // Deduct from user balances
        self.user_balances.insert(balance_a_key, user_balance_a - amount_a);
        self.user_balances.insert(balance_b_key, user_balance_b - amount_b);

        // Track user's liquidity position
        let liquidity_key = format!("{}_liquidity_{}", user, pair_key);
        let current_liquidity = *self.user_balances.get(&liquidity_key).unwrap_or(&0);
        self.user_balances.insert(liquidity_key, current_liquidity + liquidity_minted);

        Ok(format!("Added liquidity: {} {}, {} {} to {}/{} pool. Minted {} liquidity tokens.", 
            amount_a, token_a, amount_b, token_b, token_a, token_b, liquidity_minted).into_bytes())
    }

    /// Remove liquidity from a token pair pool
    pub fn remove_liquidity(
        &mut self, 
        user: String,
        token_a: String, 
        token_b: String, 
        liquidity_amount: u128
    ) -> Result<Vec<u8>, String> {
        let pair_key = self.get_pair_key(&token_a, &token_b);
        
        // Check user has sufficient liquidity tokens - copy value to avoid borrow issues
        let liquidity_key = format!("{}_liquidity_{}", user, pair_key);
        let user_liquidity = *self.user_balances.get(&liquidity_key).unwrap_or(&0);
        
        if user_liquidity < liquidity_amount {
            return Err("Insufficient liquidity tokens".to_string());
        }

        let pool = self.pools.get_mut(&pair_key)
            .ok_or("Pool does not exist")?;

        if liquidity_amount > pool.total_liquidity {
            return Err("Insufficient pool liquidity".to_string());
        }

        // Calculate amount to return based on liquidity share
        let amount_a = (liquidity_amount * pool.reserve_a) / pool.total_liquidity;
        let amount_b = (liquidity_amount * pool.reserve_b) / pool.total_liquidity;

        pool.reserve_a -= amount_a;
        pool.reserve_b -= amount_b;
        pool.total_liquidity -= liquidity_amount;

        // Update user balances - copy current values to avoid borrow issues
        let balance_a_key = format!("{}_{}", user, token_a);
        let balance_b_key = format!("{}_{}", user, token_b);
        
        let current_balance_a = *self.user_balances.get(&balance_a_key).unwrap_or(&0);
        let current_balance_b = *self.user_balances.get(&balance_b_key).unwrap_or(&0);
        
        self.user_balances.insert(balance_a_key, current_balance_a + amount_a);
        self.user_balances.insert(balance_b_key, current_balance_b + amount_b);
        self.user_balances.insert(liquidity_key, user_liquidity - liquidity_amount);

        Ok(format!("Removed liquidity: {} {}, {} {} from {}/{} pool", 
            amount_a, token_a, amount_b, token_b, token_a, token_b).into_bytes())
    }

    /// Swap exact amount of tokens for tokens (constant product formula)
    pub fn swap_exact_tokens_for_tokens(
        &mut self, 
        user: String,
        token_in: String, 
        token_out: String, 
        amount_in: u128, 
        min_amount_out: u128
    ) -> Result<Vec<u8>, String> {
        // Check user has sufficient balance - copy value to avoid borrow issues
        let balance_in_key = format!("{}_{}", user, token_in);
        let user_balance_in = *self.user_balances.get(&balance_in_key).unwrap_or(&0);
        
        if user_balance_in < amount_in {
            return Err(format!("Insufficient {} balance", token_in));
        }

        let pair_key = self.get_pair_key(&token_in, &token_out);
        
        let pool = self.pools.get_mut(&pair_key)
            .ok_or("Pool does not exist")?;

        if pool.reserve_a == 0 || pool.reserve_b == 0 {
            return Err("Insufficient liquidity".to_string());
        }

        // Determine which token is which in the pool
        let (reserve_in, reserve_out) = if pool.token_a == token_in {
            (pool.reserve_a, pool.reserve_b)
        } else {
            (pool.reserve_b, pool.reserve_a)
        };

        // Calculate output amount using constant product formula (no fees)
        // (x + Δx) * (y - Δy) = x * y
        // Δy = (y * Δx) / (x + Δx)  // No fees for testing
        let numerator = amount_in * reserve_out;
        let denominator = reserve_in + amount_in;
        let amount_out = numerator / denominator;

        if amount_out < min_amount_out {
            return Err("Insufficient output amount".to_string());
        }

        // Update pool reserves
        if pool.token_a == token_in {
            pool.reserve_a += amount_in;
            pool.reserve_b -= amount_out;
        } else {
            pool.reserve_b += amount_in;
            pool.reserve_a -= amount_out;
        }

        // Update user balances - copy current value to avoid borrow issues
        let balance_out_key = format!("{}_{}", user, token_out);
        let current_balance_out = *self.user_balances.get(&balance_out_key).unwrap_or(&0);
        
        self.user_balances.insert(balance_in_key, user_balance_in - amount_in);
        self.user_balances.insert(balance_out_key, current_balance_out + amount_out);

        Ok(format!("Swapped {} {} for {} {}", 
            amount_in, token_in, amount_out, token_out).into_bytes())
    }

    /// Get current reserves for a token pair
    pub fn get_reserves(&self, token_a: String, token_b: String) -> Result<Vec<u8>, String> {
        let pair_key = self.get_pair_key(&token_a, &token_b);
        
        let pool = self.pools.get(&pair_key)
            .ok_or("Pool does not exist")?;

        Ok(format!("Reserves: {} = {}, {} = {}, Total Liquidity: {}", 
            pool.token_a, pool.reserve_a, 
            pool.token_b, pool.reserve_b,
            pool.total_liquidity).into_bytes())
    }

    /// Generate a consistent pair key for any token order
    fn get_pair_key(&self, token_a: &str, token_b: &str) -> String {
        let mut tokens = [token_a, token_b];
        tokens.sort();
        format!("{}_{}", tokens[0], tokens[1])
    }
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Debug, Clone, Default)]
pub struct AmmContract {
    pools: HashMap<String, LiquidityPool>,
    user_balances: HashMap<String, u128>, // "user_token" -> balance
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Debug, Clone)]
pub struct LiquidityPool {
    pub token_a: String,
    pub token_b: String,
    pub reserve_a: u128,
    pub reserve_b: u128,
    pub total_liquidity: u128,
}

/// Enum representing possible calls to the AMM contract
#[derive(Serialize, Deserialize, BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
pub enum AmmAction {
    MintTokens {
        user: String,
        token: String,
        amount: u128,
    },
    AddLiquidity {
        user: String,
        token_a: String,
        token_b: String,
        amount_a: u128,
        amount_b: u128,
    },
    RemoveLiquidity {
        user: String,
        token_a: String,
        token_b: String,
        liquidity_amount: u128,
    },
    SwapExactTokensForTokens {
        user: String,
        token_in: String,
        token_out: String,
        amount_in: u128,
        min_amount_out: u128,
    },
    GetReserves {
        token_a: String,
        token_b: String,
    },
    GetUserBalance {
        user: String,
        token: String,
    },
}

impl AmmAction {
    pub fn as_blob(&self, contract_name: sdk::ContractName) -> sdk::Blob {
        sdk::Blob {
            contract_name,
            data: sdk::BlobData(borsh::to_vec(self).expect("Failed to encode AmmAction")),
        }
    }
}

impl AmmContract {
    pub fn as_bytes(&self) -> Result<Vec<u8>, Error> {
        borsh::to_vec(self)
    }
}

impl From<sdk::StateCommitment> for AmmContract {
    fn from(state: sdk::StateCommitment) -> Self {
        borsh::from_slice(&state.0)
            .map_err(|_| "Could not decode AMM state".to_string())
            .unwrap()
    }
}

// Helper trait for integer square root
trait IntegerSqrt {
    fn integer_sqrt(self) -> Self;
}

impl IntegerSqrt for u128 {
    fn integer_sqrt(self) -> Self {
        if self == 0 {
            return 0;
        }
        let mut x = self;
        let mut y = (x + 1) / 2;
        while y < x {
            x = y;
            y = (x + self / x) / 2;
        }
        x
    }
}

// Type alias for backward compatibility
pub type Contract1 = AmmContract;
pub type Contract1Action = AmmAction;

// ============================================================================
// COMPREHENSIVE UNIT TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_contract() -> AmmContract {
        AmmContract {
            pools: HashMap::new(),
            user_balances: HashMap::new(),
        }
    }

    fn get_user_balance_value(contract: &AmmContract, user: &str, token: &str) -> u128 {
        let balance_bytes = contract.get_user_balance(user.to_string(), token.to_string()).unwrap();
        let balance_str = String::from_utf8_lossy(&balance_bytes);
        // Extract number from "User alice has 1000 USDC tokens" format (index 3)
        balance_str.split_whitespace().nth(3).unwrap_or("0").parse().unwrap_or(0)
    }

    fn get_pool_reserves(contract: &AmmContract, token_a: &str, token_b: &str) -> (u128, u128, u128) {
        let reserves_bytes = contract.get_reserves(token_a.to_string(), token_b.to_string()).unwrap();
        let reserves_str = String::from_utf8_lossy(&reserves_bytes);
        // Parse reserves from format: "Reserves: USDC = X, ETH = Y, Total Liquidity: Z"
        let parts: Vec<&str> = reserves_str.split(", ").collect();
        let reserve_a = parts[0].split(" = ").nth(1).unwrap_or("0").parse().unwrap_or(0);
        let reserve_b = parts[1].split(" = ").nth(1).unwrap_or("0").parse().unwrap_or(0);
        let liquidity = parts[2].split(": ").nth(1).unwrap_or("0").parse().unwrap_or(0);
        (reserve_a, reserve_b, liquidity)
    }

    // ========================================================================
    // MINTING TESTS
    // ========================================================================

    #[test]
    fn test_minting_increases_balance() {
        let mut contract = create_test_contract();
        
        // Test initial zero balance
        assert_eq!(get_user_balance_value(&contract, "bob", "USDC"), 0);
        
        // Mint tokens increases balance
        contract.mint_tokens("bob".to_string(), "USDC".to_string(), 1000).unwrap();
        assert_eq!(get_user_balance_value(&contract, "bob", "USDC"), 1000);
        
        // Additional minting adds to existing balance
        contract.mint_tokens("bob".to_string(), "USDC".to_string(), 500).unwrap();
        assert_eq!(get_user_balance_value(&contract, "bob", "USDC"), 1500);
    }

    #[test]
    fn test_minting_multiple_users_and_tokens() {
        let mut contract = create_test_contract();
        
        // Mint different amounts for different users and tokens
        contract.mint_tokens("alice".to_string(), "USDC".to_string(), 2000).unwrap();
        contract.mint_tokens("alice".to_string(), "ETH".to_string(), 1000).unwrap();
        contract.mint_tokens("bob".to_string(), "USDC".to_string(), 500).unwrap();
        
        // Verify independent balances
        assert_eq!(get_user_balance_value(&contract, "alice", "USDC"), 2000);
        assert_eq!(get_user_balance_value(&contract, "alice", "ETH"), 1000);
        assert_eq!(get_user_balance_value(&contract, "bob", "USDC"), 500);
        assert_eq!(get_user_balance_value(&contract, "bob", "ETH"), 0);
    }

    // ========================================================================
    // POOL INITIALIZATION TESTS
    // ========================================================================

    #[test]
    fn test_pool_initialization_with_different_prices() {
        let mut contract = create_test_contract();
        
        // Setup user funds (increased amounts to handle multiple pools)
        contract.mint_tokens("alice".to_string(), "USDC".to_string(), 20000).unwrap();
        contract.mint_tokens("alice".to_string(), "ETH".to_string(), 10000).unwrap();
        contract.mint_tokens("alice".to_string(), "BTC".to_string(), 1000).unwrap();
        contract.mint_tokens("alice".to_string(), "GOLD".to_string(), 1000).unwrap();
        contract.mint_tokens("alice".to_string(), "SILVER".to_string(), 10000).unwrap();
        
        // Test 1:1 price pool
        contract.add_liquidity("alice".to_string(), "USDC".to_string(), "ETH".to_string(), 1000, 1000).unwrap();
        let (reserve_a, reserve_b, _) = get_pool_reserves(&contract, "USDC", "ETH");
        assert_eq!(reserve_a, 1000);
        assert_eq!(reserve_b, 1000);
        
        // Test 2:1 price pool (different tokens)
        contract.add_liquidity("alice".to_string(), "USDC".to_string(), "BTC".to_string(), 2000, 100).unwrap();
        let (reserve_a, reserve_b, _) = get_pool_reserves(&contract, "USDC", "BTC");
        // BTC comes first alphabetically, so reserve_a=100(BTC), reserve_b=2000(USDC)
        assert_eq!(reserve_a, 100); // BTC
        assert_eq!(reserve_b, 2000); // USDC
        
        // Test 10:1 price pool
        contract.add_liquidity("alice".to_string(), "GOLD".to_string(), "SILVER".to_string(), 100, 1000).unwrap();
        let (reserve_a, reserve_b, _) = get_pool_reserves(&contract, "GOLD", "SILVER");
        assert_eq!(reserve_a, 100);  // GOLD
        assert_eq!(reserve_b, 1000); // SILVER
    }

    #[test]
    fn test_pool_funding_on_initialization() {
        let mut contract = create_test_contract();
        
        // Setup: alice has 2000 USDC and 2000 ETH
        contract.mint_tokens("alice".to_string(), "USDC".to_string(), 2000).unwrap();
        contract.mint_tokens("alice".to_string(), "ETH".to_string(), 2000).unwrap();
        
        // Initialize pool with 1000 USDC and 1000 ETH
        contract.add_liquidity("alice".to_string(), "USDC".to_string(), "ETH".to_string(), 1000, 1000).unwrap();
        
        // Check pool has the funds
        let (reserve_a, reserve_b, liquidity) = get_pool_reserves(&contract, "USDC", "ETH");
        assert_eq!(reserve_a, 1000); // ETH (alphabetically first)
        assert_eq!(reserve_b, 1000); // USDC
        assert_eq!(liquidity, 1000);  // sqrt(1000 * 1000) = 1000
        
        // Check alice's balances were deducted
        assert_eq!(get_user_balance_value(&contract, "alice", "USDC"), 1000); // 2000 - 1000
        assert_eq!(get_user_balance_value(&contract, "alice", "ETH"), 1000);  // 2000 - 1000
    }

    // ========================================================================
    // POOL INVARIANT TESTS
    // ========================================================================

    #[test]
    fn test_constant_product_invariant_with_no_fees() {
        let mut contract = create_test_contract();
        
        // Setup equal liquidity pool
        contract.mint_tokens("alice".to_string(), "USDC".to_string(), 1000).unwrap();
        contract.mint_tokens("alice".to_string(), "ETH".to_string(), 1000).unwrap();
        contract.add_liquidity("alice".to_string(), "USDC".to_string(), "ETH".to_string(), 1000, 1000).unwrap();
        
        let (initial_reserve_a, initial_reserve_b, _) = get_pool_reserves(&contract, "USDC", "ETH");
        let initial_k = initial_reserve_a * initial_reserve_b;
        
        // Give bob tokens to swap
        contract.mint_tokens("bob".to_string(), "ETH".to_string(), 100).unwrap();
        
        // Perform swap: 100 ETH for USDC
        contract.swap_exact_tokens_for_tokens("bob".to_string(), "ETH".to_string(), "USDC".to_string(), 100, 0).unwrap();
        
        let (final_reserve_a, final_reserve_b, _) = get_pool_reserves(&contract, "USDC", "ETH");
        let final_k = final_reserve_a * final_reserve_b;
        
        // With integer arithmetic, k should increase slightly (benefits liquidity providers)
        // Allow up to 0.2% increase in k due to rounding
        let k_increase_percentage = ((final_k as f64 - initial_k as f64) / initial_k as f64) * 100.0;
        assert!(k_increase_percentage >= 0.0, "K should not decrease: {} -> {}", initial_k, final_k);
        assert!(k_increase_percentage <= 0.2, "K increase should be minimal: {}% ({}->{})", k_increase_percentage, initial_k, final_k);
    }

    #[test]
    fn test_liquidity_provision_preserves_ratios() {
        let mut contract = create_test_contract();
        
        // Setup initial pool with 2:1 ratio (USDC:ETH)
        contract.mint_tokens("alice".to_string(), "USDC".to_string(), 4000).unwrap();
        contract.mint_tokens("alice".to_string(), "ETH".to_string(), 4000).unwrap();
        contract.add_liquidity("alice".to_string(), "USDC".to_string(), "ETH".to_string(), 2000, 1000).unwrap();
        
        let (initial_reserve_a, initial_reserve_b, initial_liquidity) = get_pool_reserves(&contract, "USDC", "ETH");
        let initial_ratio = initial_reserve_b as f64 / initial_reserve_a as f64; // USDC/ETH ratio
        
        // Bob adds liquidity maintaining the same ratio (1000 USDC : 500 ETH maintains 2:1)
        contract.mint_tokens("bob".to_string(), "USDC".to_string(), 1000).unwrap();
        contract.mint_tokens("bob".to_string(), "ETH".to_string(), 1000).unwrap();
        contract.add_liquidity("bob".to_string(), "USDC".to_string(), "ETH".to_string(), 1000, 500).unwrap();
        
        let (final_reserve_a, final_reserve_b, final_liquidity) = get_pool_reserves(&contract, "USDC", "ETH");
        let final_ratio = final_reserve_b as f64 / final_reserve_a as f64;
        
        // Ratio should be preserved within 0.1%
        let ratio_change_percentage = ((final_ratio - initial_ratio).abs() / initial_ratio) * 100.0;
        assert!(ratio_change_percentage < 0.1, "Ratio should be preserved: {} vs {} ({}% change)", initial_ratio, final_ratio, ratio_change_percentage);
        
        // Total reserves should increase proportionally
        assert_eq!(final_reserve_a, initial_reserve_a + 500); // ETH
        assert_eq!(final_reserve_b, initial_reserve_b + 1000); // USDC
        assert!(final_liquidity > initial_liquidity, "Liquidity should increase");
    }

    // ========================================================================
    // PRICE CHANGE TESTS
    // ========================================================================

    #[test]
    fn test_swap_changes_price_correctly() {
        let mut contract = create_test_contract();
        
        // Setup 1:1 pool (1000 USDC : 1000 ETH)
        contract.mint_tokens("alice".to_string(), "USDC".to_string(), 1000).unwrap();
        contract.mint_tokens("alice".to_string(), "ETH".to_string(), 1000).unwrap();
        contract.add_liquidity("alice".to_string(), "USDC".to_string(), "ETH".to_string(), 1000, 1000).unwrap();
        
        let (initial_eth, initial_usdc, _) = get_pool_reserves(&contract, "USDC", "ETH");
        let initial_price_eth_per_usdc = initial_eth as f64 / initial_usdc as f64; // ETH per USDC
        
        // Bob swaps USDC for ETH
        contract.mint_tokens("bob".to_string(), "USDC".to_string(), 100).unwrap();
        contract.swap_exact_tokens_for_tokens("bob".to_string(), "USDC".to_string(), "ETH".to_string(), 100, 0).unwrap();
        
        let (final_eth, final_usdc, _) = get_pool_reserves(&contract, "USDC", "ETH");
        let final_price_eth_per_usdc = final_eth as f64 / final_usdc as f64;
        
        // After swapping USDC for ETH:
        // - More USDC in pool, less ETH in pool
        // - Price of ETH (in USDC terms) should increase
        // - Price of USDC (in ETH terms) should decrease
        assert!(final_usdc > initial_usdc, "USDC reserves should increase");
        assert!(final_eth < initial_eth, "ETH reserves should decrease");
        assert!(final_price_eth_per_usdc < initial_price_eth_per_usdc, "ETH per USDC should decrease (ETH price in USDC increased)");
    }

    #[test]
    fn test_swap_direction_affects_price_correctly() {
        let mut contract = create_test_contract();
        
        // Setup asymmetric pool (500 USDC : 1000 ETH) - ETH is cheaper
        contract.mint_tokens("alice".to_string(), "USDC".to_string(), 500).unwrap();
        contract.mint_tokens("alice".to_string(), "ETH".to_string(), 1000).unwrap();
        contract.add_liquidity("alice".to_string(), "USDC".to_string(), "ETH".to_string(), 500, 1000).unwrap();
        
        let (initial_eth, initial_usdc, _) = get_pool_reserves(&contract, "USDC", "ETH");
        
        // Test 1: Swap ETH for USDC (selling ETH)
        contract.mint_tokens("bob".to_string(), "ETH".to_string(), 100).unwrap();
        contract.swap_exact_tokens_for_tokens("bob".to_string(), "ETH".to_string(), "USDC".to_string(), 100, 0).unwrap();
        
        let (mid_eth, mid_usdc, _) = get_pool_reserves(&contract, "USDC", "ETH");
        
        // After selling ETH: more ETH in pool, less USDC, so ETH price should drop
        assert!(mid_eth > initial_eth, "ETH reserves should increase after selling ETH");
        assert!(mid_usdc < initial_usdc, "USDC reserves should decrease after buying USDC");
        
        // Test 2: Swap back USDC for ETH (buying ETH)
        let usdc_received = initial_usdc - mid_usdc;
        contract.swap_exact_tokens_for_tokens("bob".to_string(), "USDC".to_string(), "ETH".to_string(), usdc_received, 0).unwrap();
        
        let (final_eth, final_usdc, _) = get_pool_reserves(&contract, "USDC", "ETH");
        
        // After buying ETH back: less ETH in pool, more USDC, so ETH price should increase
        assert!(final_eth < mid_eth, "ETH reserves should decrease after buying ETH");
        assert!(final_usdc > mid_usdc, "USDC reserves should increase after selling USDC");
    }

    // ========================================================================
    // NO-FEE REVERSIBILITY TESTS
    // ========================================================================

    #[test]
    fn test_swapping_back_and_forth_preserves_balances() {
        let mut contract = create_test_contract();
        
        // Setup equal pool
        contract.mint_tokens("alice".to_string(), "USDC".to_string(), 1000).unwrap();
        contract.mint_tokens("alice".to_string(), "ETH".to_string(), 1000).unwrap();
        contract.add_liquidity("alice".to_string(), "USDC".to_string(), "ETH".to_string(), 1000, 1000).unwrap();
        
        // Give bob initial tokens
        contract.mint_tokens("bob".to_string(), "USDC".to_string(), 100).unwrap();
        let initial_usdc = get_user_balance_value(&contract, "bob", "USDC");
        let initial_eth = get_user_balance_value(&contract, "bob", "ETH");
        
        // Swap USDC for ETH
        contract.swap_exact_tokens_for_tokens("bob".to_string(), "USDC".to_string(), "ETH".to_string(), 100, 0).unwrap();
        let eth_received = get_user_balance_value(&contract, "bob", "ETH");
        
        // Swap all ETH back for USDC
        contract.swap_exact_tokens_for_tokens("bob".to_string(), "ETH".to_string(), "USDC".to_string(), eth_received, 0).unwrap();
        
        let final_usdc = get_user_balance_value(&contract, "bob", "USDC");
        let final_eth = get_user_balance_value(&contract, "bob", "ETH");
        
        // With integer arithmetic, allow small losses due to rounding (up to 2% of original amount)
        let usdc_loss_percentage = ((initial_usdc as f64 - final_usdc as f64) / initial_usdc as f64) * 100.0;
        assert!(usdc_loss_percentage >= 0.0, "USDC balance should not increase");
        assert!(usdc_loss_percentage <= 2.0, "USDC loss should be minimal: {}% ({} -> {})", usdc_loss_percentage, initial_usdc, final_usdc);
        assert_eq!(initial_eth, final_eth, "ETH balance should be preserved");
    }

    #[test]
    fn test_multiple_round_trip_swaps_preserve_pool_state() {
        let mut contract = create_test_contract();
        
        // Setup pool
        contract.mint_tokens("alice".to_string(), "USDC".to_string(), 1000).unwrap();
        contract.mint_tokens("alice".to_string(), "ETH".to_string(), 1000).unwrap();
        contract.add_liquidity("alice".to_string(), "USDC".to_string(), "ETH".to_string(), 1000, 1000).unwrap();
        
        let (initial_eth, initial_usdc, initial_liquidity) = get_pool_reserves(&contract, "USDC", "ETH");
        
        // Perform multiple round-trip swaps
        for i in 1..=5 {
            contract.mint_tokens("bob".to_string(), "USDC".to_string(), 50).unwrap();
            
            // Swap USDC -> ETH
            contract.swap_exact_tokens_for_tokens("bob".to_string(), "USDC".to_string(), "ETH".to_string(), 50, 0).unwrap();
            let eth_received = get_user_balance_value(&contract, "bob", "ETH");
            
            // Swap ETH -> USDC
            contract.swap_exact_tokens_for_tokens("bob".to_string(), "ETH".to_string(), "USDC".to_string(), eth_received, 0).unwrap();
            
            println!("Completed round-trip swap {}", i);
        }
        
        let (final_eth, final_usdc, final_liquidity) = get_pool_reserves(&contract, "USDC", "ETH");
        
        // Allow small pool growth due to accumulated rounding (up to 1% increase)
        let eth_growth_percentage = ((final_eth as f64 - initial_eth as f64) / initial_eth as f64) * 100.0;
        let usdc_growth_percentage = ((final_usdc as f64 - initial_usdc as f64) / initial_usdc as f64) * 100.0;
        
        assert!(eth_growth_percentage >= 0.0 && eth_growth_percentage <= 1.0, 
                "ETH reserves should grow minimally: {}% ({} -> {})", eth_growth_percentage, initial_eth, final_eth);
        assert!(usdc_growth_percentage >= 0.0 && usdc_growth_percentage <= 1.0, 
                "USDC reserves should grow minimally: {}% ({} -> {})", usdc_growth_percentage, initial_usdc, final_usdc);
        assert_eq!(initial_liquidity, final_liquidity, "Total liquidity should be preserved");
    }

    // ========================================================================
    // EDGE CASES AND ERROR CONDITIONS
    // ========================================================================

    #[test]
    fn test_insufficient_balance_errors() {
        let mut contract = create_test_contract();
        
        // Test minting doesn't affect insufficient balance checks
        contract.mint_tokens("bob".to_string(), "USDC".to_string(), 50).unwrap();
        
        // Setup pool
        contract.mint_tokens("alice".to_string(), "USDC".to_string(), 1000).unwrap();
        contract.mint_tokens("alice".to_string(), "ETH".to_string(), 1000).unwrap();
        contract.add_liquidity("alice".to_string(), "USDC".to_string(), "ETH".to_string(), 1000, 1000).unwrap();
        
        // Try to swap more than balance
        let result = contract.swap_exact_tokens_for_tokens("bob".to_string(), "USDC".to_string(), "ETH".to_string(), 100, 0);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Insufficient USDC balance"));
        
        // Try to add liquidity with insufficient balance
        let result = contract.add_liquidity("bob".to_string(), "USDC".to_string(), "ETH".to_string(), 100, 100);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Insufficient"));
    }

    #[test]
    fn test_nonexistent_pool_error() {
        let mut contract = create_test_contract();
        
        contract.mint_tokens("bob".to_string(), "USDC".to_string(), 100).unwrap();
        
        let result = contract.swap_exact_tokens_for_tokens("bob".to_string(), "USDC".to_string(), "UNKNOWN".to_string(), 50, 0);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Pool does not exist"));
    }

    #[test]
    fn test_slippage_protection() {
        let mut contract = create_test_contract();
        
        // Setup uneven pool (2:1 ratio)
        contract.mint_tokens("alice".to_string(), "USDC".to_string(), 1000).unwrap();
        contract.mint_tokens("alice".to_string(), "ETH".to_string(), 500).unwrap();
        contract.add_liquidity("alice".to_string(), "USDC".to_string(), "ETH".to_string(), 1000, 500).unwrap();
        
        contract.mint_tokens("bob".to_string(), "USDC".to_string(), 100).unwrap();
        
        // Calculate expected output: (100 * 500) / (1000 + 100) = ~45.45, so expect ~45 ETH
        // Try to demand 50 ETH (more than possible) - should fail
        let result = contract.swap_exact_tokens_for_tokens("bob".to_string(), "USDC".to_string(), "ETH".to_string(), 100, 50);
        assert!(result.is_err(), "Should fail due to slippage protection");
        assert!(result.unwrap_err().contains("Insufficient output amount"));
    }

    #[test]
    fn test_pair_key_consistency() {
        let contract = create_test_contract();
        
        // Test that pair key is consistent regardless of token order
        assert_eq!(contract.get_pair_key("USDC", "ETH"), contract.get_pair_key("ETH", "USDC"));
        assert_eq!(contract.get_pair_key("ABC", "XYZ"), contract.get_pair_key("XYZ", "ABC"));
        assert_eq!(contract.get_pair_key("TOKEN1", "TOKEN2"), "TOKEN1_TOKEN2");
        assert_eq!(contract.get_pair_key("TOKEN2", "TOKEN1"), "TOKEN1_TOKEN2");
    }

    // ========================================================================
    // COMPLEX SCENARIOS
    // ========================================================================

    #[test]
    fn test_multiple_pools_independent_operation() {
        let mut contract = create_test_contract();
        
        // Setup multiple pools with different ratios
        contract.mint_tokens("alice".to_string(), "USDC".to_string(), 5000).unwrap();
        contract.mint_tokens("alice".to_string(), "ETH".to_string(), 2000).unwrap();
        contract.mint_tokens("alice".to_string(), "BTC".to_string(), 100).unwrap();
        
        // Pool 1: USDC/ETH (2:1 ratio)
        contract.add_liquidity("alice".to_string(), "USDC".to_string(), "ETH".to_string(), 2000, 1000).unwrap();
        
        // Pool 2: USDC/BTC (30:1 ratio)  
        contract.add_liquidity("alice".to_string(), "USDC".to_string(), "BTC".to_string(), 3000, 100).unwrap();
        
        let (usdc_eth_reserve_a, usdc_eth_reserve_b, _) = get_pool_reserves(&contract, "USDC", "ETH");
        let (btc_usdc_reserve_a, btc_usdc_reserve_b, _) = get_pool_reserves(&contract, "BTC", "USDC");
        
        // Verify pools are independent and correctly set up
        assert_eq!(usdc_eth_reserve_a, 1000); // ETH
        assert_eq!(usdc_eth_reserve_b, 2000); // USDC
        assert_eq!(btc_usdc_reserve_a, 100);  // BTC  
        assert_eq!(btc_usdc_reserve_b, 3000); // USDC
        
        // Trade in one pool shouldn't affect the other
        contract.mint_tokens("bob".to_string(), "ETH".to_string(), 100).unwrap();
        contract.swap_exact_tokens_for_tokens("bob".to_string(), "ETH".to_string(), "USDC".to_string(), 100, 0).unwrap();
        
        // BTC/USDC pool should be unchanged
        let (btc_usdc_reserve_a_after, btc_usdc_reserve_b_after, _) = get_pool_reserves(&contract, "BTC", "USDC");
        assert_eq!(btc_usdc_reserve_a, btc_usdc_reserve_a_after);
        assert_eq!(btc_usdc_reserve_b, btc_usdc_reserve_b_after);
    }

    #[test]
    fn test_large_liquidity_operations() {
        let mut contract = create_test_contract();
        
        // Test with large numbers to check for overflow issues
        let large_amount = 1_000_000_000u128; // 1 billion
        
        contract.mint_tokens("whale".to_string(), "USDC".to_string(), large_amount).unwrap();
        contract.mint_tokens("whale".to_string(), "ETH".to_string(), large_amount).unwrap();
        
        // Add large liquidity
        contract.add_liquidity("whale".to_string(), "USDC".to_string(), "ETH".to_string(), large_amount / 2, large_amount / 2).unwrap();
        
        let (reserve_a, reserve_b, liquidity) = get_pool_reserves(&contract, "USDC", "ETH");
        assert_eq!(reserve_a, large_amount / 2);
        assert_eq!(reserve_b, large_amount / 2);
        assert_eq!(liquidity, large_amount / 2); // sqrt(x*x) = x
        
        // Verify whale's remaining balance
        assert_eq!(get_user_balance_value(&contract, "whale", "USDC"), large_amount / 2);
        assert_eq!(get_user_balance_value(&contract, "whale", "ETH"), large_amount / 2);
    }
}
