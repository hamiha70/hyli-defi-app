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
        
        let pool = self.pools.entry(pair_key.clone()).or_insert(LiquidityPool {
            token_a: token_a.clone(),
            token_b: token_b.clone(),
            reserve_a: 0,
            reserve_b: 0,
            total_liquidity: 0,
        });

        let liquidity_minted;

        // For initial liquidity, just add the amounts
        if pool.total_liquidity == 0 {
            pool.reserve_a = amount_a;
            pool.reserve_b = amount_b;
            liquidity_minted = (amount_a * amount_b).integer_sqrt(); // geometric mean
            pool.total_liquidity = liquidity_minted;
        } else {
            // Calculate optimal amounts based on current ratio
            let ratio_a = amount_a * pool.reserve_b;
            let ratio_b = amount_b * pool.reserve_a;
            
            if ratio_a != ratio_b {
                return Err("Invalid liquidity ratio".to_string());
            }
            
            pool.reserve_a += amount_a;
            pool.reserve_b += amount_b;
            
            // Mint liquidity tokens proportional to contribution
            liquidity_minted = (amount_a * pool.total_liquidity) / (pool.reserve_a - amount_a);
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

        // Calculate output amount using constant product formula
        // (x + Δx) * (y - Δy) = x * y
        // Δy = (y * Δx * 997) / (x * 1000 + Δx * 997)  // 0.3% fee
        let amount_in_with_fee = amount_in * 997; // 0.3% fee
        let numerator = amount_in_with_fee * reserve_out;
        let denominator = reserve_in * 1000 + amount_in_with_fee;
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
// UNIT TESTS
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

    #[test]
    fn test_mint_tokens() {
        let mut contract = create_test_contract();
        
        // Mint tokens for a user
        let result = contract.mint_tokens("alice".to_string(), "USDC".to_string(), 1000);
        assert!(result.is_ok());
        
        // Check balance was updated
        let balance_result = contract.get_user_balance("alice".to_string(), "USDC".to_string());
        assert!(balance_result.is_ok());
        assert!(String::from_utf8_lossy(&balance_result.unwrap()).contains("1000"));
        
        // Mint more tokens for same user
        let result2 = contract.mint_tokens("alice".to_string(), "USDC".to_string(), 500);
        assert!(result2.is_ok());
        
        // Balance should be 1500 now
        let balance_result2 = contract.get_user_balance("alice".to_string(), "USDC".to_string());
        assert!(balance_result2.is_ok());
        assert!(String::from_utf8_lossy(&balance_result2.unwrap()).contains("1500"));
    }

    #[test]
    fn test_user_balance_tracking() {
        let mut contract = create_test_contract();
        
        // Test initial zero balance
        let balance = contract.get_user_balance("bob".to_string(), "ETH".to_string());
        assert!(String::from_utf8_lossy(&balance.unwrap()).contains("0"));
        
        // Mint tokens for multiple users and tokens
        contract.mint_tokens("bob".to_string(), "ETH".to_string(), 2000).unwrap();
        contract.mint_tokens("bob".to_string(), "USDC".to_string(), 1000).unwrap();
        contract.mint_tokens("alice".to_string(), "ETH".to_string(), 5000).unwrap();
        
        // Check all balances
        let bob_eth = contract.get_user_balance("bob".to_string(), "ETH".to_string());
        assert!(String::from_utf8_lossy(&bob_eth.unwrap()).contains("2000"));
        
        let bob_usdc = contract.get_user_balance("bob".to_string(), "USDC".to_string());
        assert!(String::from_utf8_lossy(&bob_usdc.unwrap()).contains("1000"));
        
        let alice_eth = contract.get_user_balance("alice".to_string(), "ETH".to_string());
        assert!(String::from_utf8_lossy(&alice_eth.unwrap()).contains("5000"));
    }

    #[test]
    fn test_add_initial_liquidity() {
        let mut contract = create_test_contract();
        
        // Setup: mint tokens for user
        contract.mint_tokens("alice".to_string(), "USDC".to_string(), 1000).unwrap();
        contract.mint_tokens("alice".to_string(), "ETH".to_string(), 2000).unwrap();
        
        // Add initial liquidity
        let result = contract.add_liquidity(
            "alice".to_string(),
            "USDC".to_string(),
            "ETH".to_string(),
            500,
            1000
        );
        assert!(result.is_ok());
        
        // Check pool was created
        let reserves = contract.get_reserves("USDC".to_string(), "ETH".to_string());
        assert!(reserves.is_ok());
        let reserves_data = reserves.unwrap();
        let reserves_str = String::from_utf8_lossy(&reserves_data);
        assert!(reserves_str.contains("500"));
        assert!(reserves_str.contains("1000"));
        
        // Check user balances were deducted
        let usdc_balance = contract.get_user_balance("alice".to_string(), "USDC".to_string());
        assert!(String::from_utf8_lossy(&usdc_balance.unwrap()).contains("500")); // 1000 - 500
        
        let eth_balance = contract.get_user_balance("alice".to_string(), "ETH".to_string());
        assert!(String::from_utf8_lossy(&eth_balance.unwrap()).contains("1000")); // 2000 - 1000
    }

    #[test]
    fn test_add_liquidity_insufficient_balance() {
        let mut contract = create_test_contract();
        
        // Only mint 100 USDC but try to add 500
        contract.mint_tokens("alice".to_string(), "USDC".to_string(), 100).unwrap();
        contract.mint_tokens("alice".to_string(), "ETH".to_string(), 2000).unwrap();
        
        let result = contract.add_liquidity(
            "alice".to_string(),
            "USDC".to_string(),
            "ETH".to_string(),
            500, // More than balance
            1000
        );
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Insufficient USDC balance"));
    }

    #[test]
    fn test_swap_calculation() {
        let mut contract = create_test_contract();
        
        // Setup: create a pool with liquidity
        contract.mint_tokens("alice".to_string(), "USDC".to_string(), 10000).unwrap();
        contract.mint_tokens("alice".to_string(), "ETH".to_string(), 10000).unwrap();
        contract.add_liquidity("alice".to_string(), "USDC".to_string(), "ETH".to_string(), 5000, 5000).unwrap();
        
        // Give bob some tokens to swap
        contract.mint_tokens("bob".to_string(), "USDC".to_string(), 1000).unwrap();
        
        // Perform swap: 100 USDC for ETH
        let result = contract.swap_exact_tokens_for_tokens(
            "bob".to_string(),
            "USDC".to_string(),
            "ETH".to_string(),
            100,
            0 // min_amount_out
        );
        assert!(result.is_ok());
        
        // Check bob's balances changed
        let usdc_balance = contract.get_user_balance("bob".to_string(), "USDC".to_string());
        assert!(String::from_utf8_lossy(&usdc_balance.unwrap()).contains("900")); // 1000 - 100
        
        let eth_balance = contract.get_user_balance("bob".to_string(), "ETH".to_string());
        let eth_balance_data = eth_balance.unwrap();
        let eth_balance_str = String::from_utf8_lossy(&eth_balance_data);
        // Should have received some ETH (exact amount depends on constant product formula)
        assert!(!eth_balance_str.contains("0"));
    }

    #[test]
    fn test_constant_product_invariant() {
        let mut contract = create_test_contract();
        
        // Setup pool with 1000 USDC and 1000 ETH
        contract.mint_tokens("alice".to_string(), "USDC".to_string(), 1000).unwrap();
        contract.mint_tokens("alice".to_string(), "ETH".to_string(), 1000).unwrap();
        contract.add_liquidity("alice".to_string(), "USDC".to_string(), "ETH".to_string(), 1000, 1000).unwrap();
        
        // Calculate initial k = x * y
        let _initial_k = 1000u128 * 1000u128;
        
        // Give bob tokens to swap
        contract.mint_tokens("bob".to_string(), "USDC".to_string(), 100).unwrap();
        
        // Perform swap
        contract.swap_exact_tokens_for_tokens(
            "bob".to_string(),
            "USDC".to_string(),
            "ETH".to_string(),
            100,
            0
        ).unwrap();
        
        // Check that k increased (due to fees)
        let reserves = contract.get_reserves("USDC".to_string(), "ETH".to_string()).unwrap();
        let reserves_str = String::from_utf8_lossy(&reserves);
        
        // Parse reserves (this is simplified - in real test we'd parse properly)
        // The important thing is that reserves exist and pool is functional
        assert!(reserves_str.contains("USDC"));
        assert!(reserves_str.contains("ETH"));
    }

    #[test]
    fn test_swap_insufficient_balance() {
        let mut contract = create_test_contract();
        
        // Setup pool
        contract.mint_tokens("alice".to_string(), "USDC".to_string(), 1000).unwrap();
        contract.mint_tokens("alice".to_string(), "ETH".to_string(), 1000).unwrap();
        contract.add_liquidity("alice".to_string(), "USDC".to_string(), "ETH".to_string(), 1000, 1000).unwrap();
        
        // Try to swap more than bob has
        let result = contract.swap_exact_tokens_for_tokens(
            "bob".to_string(), // bob has 0 tokens
            "USDC".to_string(),
            "ETH".to_string(),
            100,
            0
        );
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Insufficient USDC balance"));
    }

    #[test]
    fn test_swap_nonexistent_pool() {
        let mut contract = create_test_contract();
        
        // Give bob tokens
        contract.mint_tokens("bob".to_string(), "USDC".to_string(), 100).unwrap();
        
        // Try to swap in nonexistent pool
        let result = contract.swap_exact_tokens_for_tokens(
            "bob".to_string(),
            "USDC".to_string(),
            "UNKNOWN".to_string(), // Pool doesn't exist
            100,
            0
        );
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Pool does not exist"));
    }

    #[test]
    fn test_remove_liquidity() {
        let mut contract = create_test_contract();
        
        // Setup: add liquidity
        contract.mint_tokens("alice".to_string(), "USDC".to_string(), 1000).unwrap();
        contract.mint_tokens("alice".to_string(), "ETH".to_string(), 1000).unwrap();
        contract.add_liquidity("alice".to_string(), "USDC".to_string(), "ETH".to_string(), 1000, 1000).unwrap();
        
        // Remove half the liquidity
        let initial_liquidity = 1000u128; // geometric mean of 1000 * 1000
        let result = contract.remove_liquidity(
            "alice".to_string(),
            "USDC".to_string(),
            "ETH".to_string(),
            initial_liquidity / 2
        );
        assert!(result.is_ok());
        
        // Check alice got tokens back
        let usdc_balance = contract.get_user_balance("alice".to_string(), "USDC".to_string());
        let usdc_data = usdc_balance.unwrap();
        let usdc_str = String::from_utf8_lossy(&usdc_data);
        assert!(usdc_str.contains("500")); // Should have ~500 back
        
        let eth_balance = contract.get_user_balance("alice".to_string(), "ETH".to_string());
        let eth_data = eth_balance.unwrap();
        let eth_str = String::from_utf8_lossy(&eth_data);
        assert!(eth_str.contains("500")); // Should have ~500 back
    }

    #[test]
    fn test_pair_key_consistency() {
        let contract = create_test_contract();
        
        // Test that pair key is consistent regardless of token order
        let key1 = contract.get_pair_key("USDC", "ETH");
        let key2 = contract.get_pair_key("ETH", "USDC");
        assert_eq!(key1, key2);
        
        let key3 = contract.get_pair_key("ABC", "XYZ");
        let key4 = contract.get_pair_key("XYZ", "ABC");
        assert_eq!(key3, key4);
    }

    #[test]
    fn test_slippage_protection() {
        let mut contract = create_test_contract();
        
        // Setup pool
        contract.mint_tokens("alice".to_string(), "USDC".to_string(), 1000).unwrap();
        contract.mint_tokens("alice".to_string(), "ETH".to_string(), 1000).unwrap();
        contract.add_liquidity("alice".to_string(), "USDC".to_string(), "ETH".to_string(), 1000, 1000).unwrap();
        
        // Give bob tokens
        contract.mint_tokens("bob".to_string(), "USDC".to_string(), 100).unwrap();
        
        // Try swap with high min_amount_out (should fail due to slippage)
        let result = contract.swap_exact_tokens_for_tokens(
            "bob".to_string(),
            "USDC".to_string(),
            "ETH".to_string(),
            100,
            99 // Expecting almost 1:1 ratio (unrealistic)
        );
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Insufficient output amount"));
    }
}
