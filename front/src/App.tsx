import { useState, useEffect } from 'react';
import { WalletProvider, HyliWallet, useWallet } from "hyli-wallet";
import { ZKPassportVerifier } from './components/ZKPassportVerifier';
import './App.css';
import './WalletStyles.css';
import './ZKPassportStyles.css';

interface ContractState {
  state: unknown;
  error?: string;
}

// interface TokenBalance {
//   [key: string]: number;
// }

interface VerificationResult {
  uniqueIdentifier: string;
  verified: boolean;
  ageProof: any;
  proofData: string;
}

// Note: We now use real-time pool data instead of static exchange rates

// Fruit pairs that can form liquidity pools
const FRUIT_PAIRS = [
  ['MELON', 'ORANJ'],
  ['ORANJ', 'VITAMINE'], 
  ['MELON', 'VITAMINE'],
  ['VITAMINE', 'OXYGENE']
];

// Minting amounts based on exchange rates (adjusted for trading)
const MINT_AMOUNTS = {
  'MELON': 100,     // Most valuable fruit
  'ORANJ': 500,     // 1 Melon = 5 Oranj, so mint 5x more
  'VITAMINE': 10000, // 1 Oranj = 200 Vitamine, so mint 20x more than Oranj
  'OXYGENE': 50000   // 1 Vitamine = 5 Oxygene, so mint 5x more than Vitamine
};

function ScaffoldApp() {
  const { logout, wallet, createIdentityBlobs } = useWallet();
  const [contract1State, setContract1State] = useState<ContractState | null>(null);
  const [contract2State, setContract2State] = useState<ContractState | null>(null);
  const [loading, setLoading] = useState(false);
  const [result, setResult] = useState<string | null>(null);
  
  // ZKPassport Verification State
  const [isVerified, setIsVerified] = useState(false);
  const [verificationResult, setVerificationResult] = useState<VerificationResult | null>(null);
  const [showVerification, setShowVerification] = useState(false);
  
  // AMM State - Updated for per-pool architecture
  const [currentUser, setCurrentUser] = useState('bob@wallet');
  
  // New: Per-pool state management
  const [poolStates, setPoolStates] = useState<{[key: string]: {
    swapAmountIn: string;
    swapAmountOut: string;
    swapDirection: 'AtoB' | 'BtoA';
    liquidityAmount: string;
    liquidityTokenAmount: string;
    liquidityMode: 'add' | 'remove';
  }}>({});
  
  const availableTokens = ['ORANJ', 'VITAMINE', 'OXYGENE', 'MELON'];
  const availableUsers = ['bob@wallet', 'alice@contract1', 'charlie@contract1', 'dave@wallet'];

  // Utility function to get consistent pair key
  const getPairKey = (tokenA: string, tokenB: string): string => {
    const tokens = [tokenA, tokenB].sort();
    return `${tokens[0]}_${tokens[1]}`;
  };

  // Get current price ratio from pool reserves
  const getCurrentPrice = (pool: any, tokenA: string): number => {
    if (!pool || pool.reserve_a === 0 || pool.reserve_b === 0) return 0;
    
    // Determine which token is which in the pool and return price of tokenA in terms of the other token
    if (pool.token_a === tokenA) {
      return pool.reserve_b / pool.reserve_a; // Price of tokenA in terms of tokenB
    } else {
      return pool.reserve_a / pool.reserve_b; // Price of tokenA in terms of tokenB
    }
  };

  // Calculate swap output using constant product formula from actual pool reserves
  const calculateSwapOutputFromPool = (
    pool: any, 
    tokenIn: string, 
    amountIn: number
  ): number => {
    if (!pool || pool.reserve_a === 0 || pool.reserve_b === 0 || amountIn <= 0) return 0;
    
    // Determine which reserve is which
    const [reserve_in, reserve_out] = pool.token_a === tokenIn 
      ? [pool.reserve_a, pool.reserve_b]
      : [pool.reserve_b, pool.reserve_a];
    
    // Constant product formula with 0.3% fee
    const amountInWithFee = amountIn * 997; // 0.3% fee
    const numerator = amountInWithFee * reserve_out;
    const denominator = reserve_in * 1000 + amountInWithFee;
    
    return numerator / denominator;
  };

  // Calculate price impact percentage
  const calculatePriceImpact = (
    pool: any, 
    tokenIn: string, 
    amountIn: number
  ): number => {
    if (!pool || amountIn <= 0) return 0;
    
    const currentPrice = getCurrentPrice(pool, tokenIn);
    const amountOut = calculateSwapOutputFromPool(pool, tokenIn, amountIn);
    const effectivePrice = amountOut / amountIn;
    
    return Math.abs((effectivePrice - currentPrice) / currentPrice) * 100;
  };

  // Initialize pool state if it doesn't exist
  const getPoolState = (pairKey: string) => {
    if (!poolStates[pairKey]) {
      setPoolStates(prev => ({
        ...prev,
        [pairKey]: {
          swapAmountIn: '',
          swapAmountOut: '',
          swapDirection: 'AtoB',
          liquidityAmount: '',
          liquidityTokenAmount: '',
          liquidityMode: 'add'
        }
      }));
      return {
        swapAmountIn: '',
        swapAmountOut: '',
        swapDirection: 'AtoB' as const,
        liquidityAmount: '',
        liquidityTokenAmount: '',
        liquidityMode: 'add' as const
      };
    }
    return poolStates[pairKey];
  };

  // Update pool state
  const updatePoolState = (pairKey: string, updates: Partial<typeof poolStates[string]>) => {
    setPoolStates(prev => ({
      ...prev,
      [pairKey]: { ...prev[pairKey], ...updates }
    }));
  };

  // ZKPassport verification handlers
  const handleVerificationComplete = (result: VerificationResult) => {
    console.log('✅ ZKPassport verification completed:', result);
    setVerificationResult(result);
    setIsVerified(true);
    setShowVerification(false);
    setResult(`✅ Age verified! Unique ID: ${result.uniqueIdentifier.substring(0, 8)}...`);
  };

  const handleVerificationError = (error: string) => {
    console.error('❌ ZKPassport verification error:', error);
    setResult(`❌ Verification failed: ${error}`);
    setShowVerification(false);
  };

  const startVerification = () => {
    setShowVerification(true);
    setResult('');
  };

  const skipVerification = () => {
    // For development/demo purposes only
    setIsVerified(true);
    setResult('⚠️ Verification skipped (demo mode)');
  };

  const fetchContractState = async (contractName: string) => {
    try {
      const response = await fetch(`${import.meta.env.VITE_SERVER_BASE_URL}/v1/indexer/contract/${contractName}/state`);
      
      if (!response.ok) {
        const errorText = await response.text();
        throw new Error(`HTTP error ${response.status}: ${errorText || response.statusText}`);
      }
      
      const text = await response.text();
      if (!text) {
        throw new Error('Empty response');
      }
      
      const data = JSON.parse(text);
      return { state: data };
    } catch (error) {
      console.error(`Error fetching ${contractName} state:`, error);
      return { state: null, error: error instanceof Error ? error.message : String(error) };
    }
  };

  useEffect(() => {
    const fetchStates = async () => {
      const [state1, state2] = await Promise.all([
        fetchContractState('contract1'),
        fetchContractState('contract2')
      ]);
      setContract1State(state1);
      setContract2State(state2);
    };

    fetchStates();
    const interval = setInterval(fetchStates, 30000);
    return () => clearInterval(interval);
  }, []);

  const pollTransactionStatus = async (txHash: string): Promise<void> => {
    const maxAttempts = 1200;
    let attempts = 0;
    
    setResult(`⏳ Transaction submitted! Processing... (Hash: ${txHash})`);
    
    while (attempts < maxAttempts) {
      try {
        const response = await fetch(`${import.meta.env.VITE_NODE_BASE_URL}/v1/indexer/transaction/hash/${txHash}`);
        if (!response.ok) {
          throw new Error(`HTTP error ${response.status}`);
        }
        
        const data = await response.json();
        if (data.transaction_status === "Success") {
          setResult(`✅ Transaction successful! Hash: ${txHash}`);
          // Refresh contract states
          const [state1, state2] = await Promise.all([
            fetchContractState('contract1'),
            fetchContractState('contract2')
          ]);
          setContract1State(state1);
          setContract2State(state2);
          return;
        }
        
        if (attempts % 30 === 0 && attempts > 0) {
          const minutes = Math.floor(attempts / 60);
          setResult(`⏳ Still processing... (${minutes}m elapsed) Hash: ${txHash}`);
        }
        
        await new Promise(resolve => setTimeout(resolve, 1000));
        attempts++;
      } catch (error) {
        console.error('Error polling transaction:', error);
      }
    }
    
    setResult(`⚠️ Transaction ${txHash} polling timed out. Check server logs.`);
  };

  const sendTransaction = async (endpoint: string, payload: any) => {
    if (!wallet?.address) {
      setResult('Wallet not connected');
      return;
    }

    setLoading(true);
    setResult('');
    
    try {
      const [blob0, blob1] = createIdentityBlobs();
      
      const headers = new Headers();
      headers.append('content-type', 'application/json');
      headers.append('x-user', currentUser);
      headers.append('x-session-key', 'test-session');
      headers.append('x-request-signature', 'test-signature');

      const requestBody = {
        wallet_blobs: [blob0, blob1],
        ...payload
      };

      const response = await fetch(`${import.meta.env.VITE_SERVER_BASE_URL}${endpoint}`, {
        method: 'POST',
        headers: headers,
        body: JSON.stringify(requestBody)
      });
      
      if (!response.ok) {
        const errorText = await response.text();
        throw new Error(errorText || `HTTP error ${response.status}`);
      }

      const txHash = await response.json();
      await pollTransactionStatus(txHash);
    } catch (error) {
      console.error('Error sending transaction:', error);
      setResult(`Error: ${error instanceof Error ? error.message : String(error)}`);
    } finally {
      setLoading(false);
    }
  };

  const handleMintTokens = async (token: string, amount?: number) => {
    const mintAmount = amount || MINT_AMOUNTS[token as keyof typeof MINT_AMOUNTS] || 1000;
    await sendTransaction('/api/mint-tokens', {
      token,
      amount: mintAmount
    });
  };

  const handleMintAllTokens = async () => {
    for (const token of availableTokens) {
      await handleMintTokens(token);
      // Small delay between mints to avoid overwhelming the system
      await new Promise(resolve => setTimeout(resolve, 500));
    }
  };

  // Initialize all four fruit pools with proper ratios
  const handleInitializePools = async () => {
    const poolInitializations = [
      { tokenA: 'MELON', tokenB: 'ORANJ', amountA: 20, amountB: 100 },      // 1:5 ratio
      { tokenA: 'ORANJ', tokenB: 'VITAMINE', amountA: 5, amountB: 1000 },   // 1:200 ratio
      { tokenA: 'MELON', tokenB: 'VITAMINE', amountA: 10, amountB: 1000 },  // 1:100 ratio
      { tokenA: 'VITAMINE', tokenB: 'OXYGENE', amountA: 100, amountB: 500 } // 1:5 ratio
    ];

    for (const pool of poolInitializations) {
      await sendTransaction('/api/add-liquidity', {
        token_a: pool.tokenA,
        token_b: pool.tokenB,
        amount_a: pool.amountA,
        amount_b: pool.amountB
      });
      // Small delay between pool creations
      await new Promise(resolve => setTimeout(resolve, 1000));
    }
  };

  // Auto-setup function for demo purposes
  const handleAutoSetup = async () => {
    setResult('🚀 Auto-setting up your DeFi paradise...');
    
    // Step 1: Mint all tokens
    await handleMintAllTokens();
    
    // Small delay before pools
    await new Promise(resolve => setTimeout(resolve, 2000));
    
    // Step 2: Initialize all pools
    await handleInitializePools();
    
    setResult('✅ Auto-setup complete! Your DeFi AMM is ready for trading! 🎉');
  };

  // Updated transaction handlers for per-pool architecture
  const handlePoolSwap = async (tokenA: string, tokenB: string) => {
    const pairKey = getPairKey(tokenA, tokenB);
    const poolState = getPoolState(pairKey);
    
    if (!poolState.swapAmountIn) {
      setResult('Please enter swap amount');
      return;
    }

    const tokenIn = poolState.swapDirection === 'AtoB' ? tokenA : tokenB;
    const tokenOut = poolState.swapDirection === 'AtoB' ? tokenB : tokenA;

    await sendTransaction('/api/swap-tokens', {
      token_in: tokenIn,
      token_out: tokenOut,
      amount_in: parseInt(poolState.swapAmountIn),
      min_amount_out: parseInt(poolState.swapAmountOut) || 0
    });
  };

  const handlePoolAddLiquidity = async (tokenA: string, tokenB: string) => {
    const pairKey = getPairKey(tokenA, tokenB);
    const poolState = getPoolState(pairKey);
    
    if (!poolState.liquidityAmount) {
      setResult('Please enter liquidity amount');
      return;
    }

    const state = contract1State?.state as any;
    const pools = state.pools || {};
    const pool = pools[pairKey];
    
    if (!pool || (pool.reserve_a === 0 && pool.reserve_b === 0)) {
      // New pool - need both amounts
      if (!poolState.liquidityTokenAmount) {
        setResult('Please enter both token amounts for new pool');
        return;
      }
      
      await sendTransaction('/api/add-liquidity', {
        token_a: tokenA,
        token_b: tokenB,
        amount_a: parseInt(poolState.liquidityAmount),
        amount_b: parseInt(poolState.liquidityTokenAmount)
      });
    } else {
      // Existing pool - calculate second amount from ratio
      const ratio = pool.token_a === tokenA 
        ? pool.reserve_b / pool.reserve_a 
        : pool.reserve_a / pool.reserve_b;
      
      const amount_a = parseInt(poolState.liquidityAmount);
      const amount_b = Math.floor(amount_a * ratio);
      
      await sendTransaction('/api/add-liquidity', {
        token_a: tokenA,
        token_b: tokenB,
        amount_a: amount_a,
        amount_b: amount_b
      });
    }
  };

  const handlePoolRemoveLiquidity = async (tokenA: string, tokenB: string) => {
    const pairKey = getPairKey(tokenA, tokenB);
    const poolState = getPoolState(pairKey);
    
    if (!poolState.liquidityAmount) {
      setResult('Please enter liquidity amount to remove');
      return;
    }

    await sendTransaction('/api/remove-liquidity', {
      token_a: tokenA,
      token_b: tokenB,
      liquidity_amount: parseInt(poolState.liquidityAmount)
    });
  };



  const getTokenEmoji = (token: string) => {
    switch (token) {
      case 'ORANJ': return '🍊';
      case 'VITAMINE': return '🍋';
      case 'OXYGENE': return '🫐';
      case 'MELON': return '🍈';
      default: return '🍇';
    }
  };

  const getTokenColor = (token: string) => {
    switch (token) {
      case 'ORANJ': return '#FF6B35';    // Orange
      case 'VITAMINE': return '#FFD23F'; // Yellow  
      case 'OXYGENE': return '#4A90E2';  // Blue
      case 'MELON': return '#7ED321';    // Green
      default: return '#9013FE';         // Purple
    }
  };

  const renderBalanceCard = () => {
    // Show loading state if contract state is being fetched
    if (contract1State === null) {
      return (
        <div className="balance-card">
          <h3>🍇 Your Fruit Treasury</h3>
          <div style={{ 
            textAlign: 'center', 
            padding: '40px', 
            background: 'rgba(255,255,255,0.1)', 
            borderRadius: '12px',
            color: '#666'
          }}>
            <div style={{ fontSize: '24px', marginBottom: '10px' }}>⏳</div>
            <div>Loading contract state...</div>
            <div style={{ fontSize: '12px', marginTop: '10px' }}>
              Make sure the server is running with <code>RISC0_DEV_MODE=1 cargo run -p server</code>
            </div>
          </div>
        </div>
      );
    }

    // Show error state if contract state failed to load
    if (contract1State?.error) {
      return (
        <div className="balance-card">
          <h3>🍇 Your Fruit Treasury</h3>
          <div style={{ 
            textAlign: 'center', 
            padding: '40px', 
            background: 'rgba(255,0,0,0.1)', 
            borderRadius: '12px',
            color: '#d32f2f'
          }}>
            <div style={{ fontSize: '24px', marginBottom: '10px' }}>❌</div>
            <div>Failed to load contract state</div>
            <div style={{ fontSize: '12px', marginTop: '10px', color: '#666' }}>
              Error: {contract1State.error}
            </div>
            <div style={{ fontSize: '12px', marginTop: '10px', color: '#666' }}>
              Try starting the server: <code>RISC0_DEV_MODE=1 cargo run -p server</code>
            </div>
          </div>
        </div>
      );
    }

    // Show demo mode fallback if contract state is empty but we're in demo mode
    if (!contract1State?.state && isVerified) {
      return (
        <div className="balance-card">
          <h3>🍇 Your Fruit Treasury (Demo Mode)</h3>
          <div className="mint-all-section">
            <button 
              className="mint-all-btn rainbow-btn"
              onClick={handleMintAllTokens}
              disabled={true}
              style={{ opacity: 0.6 }}
            >
              🌱 Demo Mode - Start Server to Enable
            </button>
            <button 
              className="initialize-pools-btn"
              onClick={handleInitializePools}
              disabled={true}
              style={{ opacity: 0.6 }}
            >
              🌊 Demo Mode - Start Server to Enable
            </button>
            <button 
              className="auto-setup-btn"
              onClick={handleAutoSetup}
              disabled={true}
              style={{
                background: 'linear-gradient(135deg, #667eea 0%, #764ba2 100%)',
                color: 'white',
                border: 'none',
                borderRadius: '12px',
                padding: '12px 20px',
                fontWeight: 'bold',
                cursor: 'not-allowed',
                fontSize: '14px',
                opacity: 0.6
              }}
            >
              🚀 Demo Mode - Start Server to Enable
            </button>
          </div>
          <div className="balance-grid">
            {availableTokens.map(token => (
              <div key={token} className="balance-item" style={{ borderColor: getTokenColor(token) }}>
                <span className="token-emoji">{getTokenEmoji(token)}</span>
                <span className="token-name" style={{ color: getTokenColor(token) }}>{token}</span>
                <span className="token-balance">0</span>
                <button 
                  className="mint-btn"
                  style={{ 
                    background: `linear-gradient(135deg, ${getTokenColor(token)}, ${getTokenColor(token)}AA)`,
                    opacity: 0.6
                  }}
                  disabled={true}
                >
                  Demo Mode
                </button>
              </div>
            ))}
          </div>
          <div style={{
            marginTop: '20px',
            padding: '15px',
            background: '#fff3cd',
            border: '1px solid #ffeaa7',
            borderRadius: '8px',
            color: '#856404',
            fontSize: '14px',
            textAlign: 'center'
          }}>
            <strong>🔧 Demo Mode Active</strong><br/>
            To enable full functionality, start the server:<br/>
            <code style={{ background: '#f8f9fa', padding: '2px 6px', borderRadius: '4px' }}>
              RISC0_DEV_MODE=1 cargo run -p server
            </code>
          </div>
        </div>
      );
    }
    
    // Normal operation with loaded contract state
    const state = contract1State.state as any;
    const userBalances = state.user_balances || {};
    
    return (
      <div className="balance-card">
        <h3>🍇 Your Fruit Treasury</h3>
        <div className="mint-all-section">
          <button 
            className="mint-all-btn rainbow-btn"
            onClick={handleMintAllTokens}
            disabled={loading}
          >
            {loading ? 'Growing All Fruits... 🌱' : 'Harvest All Fruits 🌈'}
          </button>
          <button 
            className="initialize-pools-btn"
            onClick={handleInitializePools}
            disabled={loading}
          >
            {loading ? 'Creating Pools... 🌊' : 'Initialize Fruit Pools 🏊‍♂️'}
          </button>
          <button 
            className="auto-setup-btn"
            onClick={handleAutoSetup}
            disabled={loading}
            style={{
              background: 'linear-gradient(135deg, #667eea 0%, #764ba2 100%)',
              color: 'white',
              border: 'none',
              borderRadius: '12px',
              padding: '12px 20px',
              fontWeight: 'bold',
              cursor: 'pointer',
              fontSize: '14px',
              boxShadow: '0 4px 15px rgba(102, 126, 234, 0.4)',
              transition: 'all 0.3s ease'
            }}
          >
            {loading ? '🔄 Setting up...' : '🚀 Auto-Setup Demo'}
          </button>
        </div>
        <div className="balance-grid">
          {availableTokens.map(token => {
            const balanceKey = `${currentUser.split('@')[0]}_${token}`;
            const balance = userBalances[balanceKey] || 0;
            const mintAmount = MINT_AMOUNTS[token as keyof typeof MINT_AMOUNTS];
            return (
              <div key={token} className="balance-item" style={{ borderColor: getTokenColor(token) }}>
                <span className="token-emoji">{getTokenEmoji(token)}</span>
                <span className="token-name" style={{ color: getTokenColor(token) }}>{token}</span>
                <span className="token-balance">{balance.toLocaleString()}</span>
                <button 
                  className="mint-btn"
                  style={{ background: `linear-gradient(135deg, ${getTokenColor(token)}, ${getTokenColor(token)}AA)` }}
                  onClick={() => handleMintTokens(token)}
                  disabled={loading}
                >
                  +{mintAmount.toLocaleString()}
                </button>
              </div>
            );
          })}
        </div>
      </div>
    );
  };

  // New PoolInterface component for per-pool architecture
  const renderPoolInterface = (tokenA: string, tokenB: string) => {
    const pairKey = getPairKey(tokenA, tokenB);
    const poolState = getPoolState(pairKey);
    const state = contract1State?.state as any;
    const pools = state?.pools || {};
    const pool = pools[pairKey];
    const userBalances = state?.user_balances || {};
    const userKey = currentUser.split('@')[0];
    
    const isNewPool = !pool || (pool.reserve_a === 0 && pool.reserve_b === 0);
    const currentPrice = getCurrentPrice(pool, tokenA);
    
    // Calculate swap amounts and price impact
    const swapAmountIn = parseFloat(poolState.swapAmountIn) || 0;
    const tokenIn = poolState.swapDirection === 'AtoB' ? tokenA : tokenB;
    const tokenOut = poolState.swapDirection === 'AtoB' ? tokenB : tokenA;
    const swapAmountOut = calculateSwapOutputFromPool(pool, tokenIn, swapAmountIn);
    const priceImpact = calculatePriceImpact(pool, tokenIn, swapAmountIn);
    
    // Calculate current swap output (will be updated via input handlers)
    
    // Calculate liquidity amounts with proper constraints
    const liquidityAmountA = parseFloat(poolState.liquidityAmount) || 0;
    let liquidityAmountB = 0;
    if (isNewPool) {
      liquidityAmountB = parseFloat(poolState.liquidityTokenAmount) || 0;
    } else if (pool && liquidityAmountA > 0) {
      const ratio = pool.token_a === tokenA 
        ? pool.reserve_b / pool.reserve_a 
        : pool.reserve_a / pool.reserve_b;
      liquidityAmountB = liquidityAmountA * ratio;
    }
    
    // Check user balances
    const userBalanceA = userBalances[`${userKey}_${tokenA}`] || 0;
    const userBalanceB = userBalances[`${userKey}_${tokenB}`] || 0;
    const userBalanceTokenIn = userBalances[`${userKey}_${tokenIn}`] || 0;
    
    return (
      <div className="pool-interface" style={{
        background: `linear-gradient(135deg, ${getTokenColor(tokenA)}11, ${getTokenColor(tokenB)}11)`,
        border: `2px solid ${getTokenColor(tokenA)}33`,
        borderRadius: '16px',
        padding: '20px',
        margin: '10px'
      }}>
        {/* Pool Header */}
        <div className="pool-header" style={{ marginBottom: '20px' }}>
          <h3 style={{ 
            background: `linear-gradient(45deg, ${getTokenColor(tokenA)}, ${getTokenColor(tokenB)})`,
            WebkitBackgroundClip: 'text',
            WebkitTextFillColor: 'transparent',
            fontSize: '20px',
            fontWeight: 'bold',
            textAlign: 'center',
            margin: '0 0 10px 0'
          }}>
            {getTokenEmoji(tokenA)}{tokenA} / {getTokenEmoji(tokenB)}{tokenB}
          </h3>
          
          {/* Pool Stats */}
          <div className="pool-stats" style={{ 
            display: 'grid', 
            gridTemplateColumns: '1fr 1fr', 
            gap: '10px',
            fontSize: '12px',
            background: 'rgba(255,255,255,0.1)',
            borderRadius: '8px',
            padding: '10px'
          }}>
            <div>
              <strong>Reserves:</strong><br/>
              {pool ? `${pool.reserve_a?.toLocaleString() || 0} ${tokenA}` : 'No pool'}
            </div>
            <div>
              <strong>Price:</strong><br/>
              {pool && currentPrice > 0 ? `1 ${tokenA} = ${currentPrice.toFixed(4)} ${tokenB}` : 'Not set'}
            </div>
            <div>
              <strong>Liquidity:</strong><br/>
              {pool?.total_liquidity?.toLocaleString() || 0}
            </div>
            <div>
              <strong>Your Balance:</strong><br/>
              {userBalanceA} {tokenA} | {userBalanceB} {tokenB}
            </div>
          </div>
        </div>

        {/* Swap Section */}
        <div className="swap-section" style={{ marginBottom: '20px' }}>
          <h4 style={{ color: getTokenColor(tokenA), marginBottom: '15px' }}>
            🔄 Swap {tokenA}/{tokenB}
          </h4>
          
          {/* Swap Direction Toggle */}
          <div style={{ display: 'flex', gap: '10px', marginBottom: '15px' }}>
            <button
              onClick={() => {
                const newDirection = 'AtoB';
                const inputAmount = parseFloat(poolState.swapAmountIn) || 0;
                const newTokenIn = newDirection === 'AtoB' ? tokenA : tokenB;
                const outputAmount = calculateSwapOutputFromPool(pool, newTokenIn, inputAmount);
                updatePoolState(pairKey, { 
                  swapDirection: newDirection,
                  swapAmountOut: outputAmount.toFixed(4)
                });
              }}
              style={{
                background: poolState.swapDirection === 'AtoB' ? getTokenColor(tokenA) : 'transparent',
                color: poolState.swapDirection === 'AtoB' ? 'white' : getTokenColor(tokenA),
                border: `1px solid ${getTokenColor(tokenA)}`,
                borderRadius: '8px',
                padding: '8px 16px',
                fontSize: '12px',
                cursor: 'pointer'
              }}
            >
              {tokenA} → {tokenB}
            </button>
            <button
              onClick={() => {
                const newDirection = 'BtoA';
                const inputAmount = parseFloat(poolState.swapAmountIn) || 0;
                const newTokenIn = newDirection === 'BtoA' ? tokenB : tokenA;
                const outputAmount = calculateSwapOutputFromPool(pool, newTokenIn, inputAmount);
                updatePoolState(pairKey, { 
                  swapDirection: newDirection,
                  swapAmountOut: outputAmount.toFixed(4)
                });
              }}
              style={{
                background: poolState.swapDirection === 'BtoA' ? getTokenColor(tokenB) : 'transparent',
                color: poolState.swapDirection === 'BtoA' ? 'white' : getTokenColor(tokenB),
                border: `1px solid ${getTokenColor(tokenB)}`,
                borderRadius: '8px',
                padding: '8px 16px',
                fontSize: '12px',
                cursor: 'pointer'
              }}
            >
              {tokenB} → {tokenA}
            </button>
          </div>
          
          {/* Swap Input */}
          <div style={{ display: 'flex', gap: '10px', marginBottom: '10px' }}>
            <input
              type="number"
              placeholder={`${tokenIn} amount`}
              value={poolState.swapAmountIn}
              onChange={(e) => {
                const inputAmount = parseFloat(e.target.value) || 0;
                const outputAmount = calculateSwapOutputFromPool(pool, tokenIn, inputAmount);
                updatePoolState(pairKey, { 
                  swapAmountIn: e.target.value,
                  swapAmountOut: outputAmount.toFixed(4)
                });
              }}
              style={{
                flex: 1,
                padding: '12px',
                borderRadius: '8px',
                border: `2px solid ${getTokenColor(tokenIn)}`,
                fontSize: '14px'
              }}
            />
            <input
              type="number"
              placeholder={`${tokenOut} output`}
              value={poolState.swapAmountOut}
              readOnly
              style={{
                flex: 1,
                padding: '12px',
                borderRadius: '8px',
                border: `2px solid ${getTokenColor(tokenOut)}`,
                backgroundColor: `${getTokenColor(tokenOut)}11`,
                fontSize: '14px'
              }}
            />
          </div>
          
          {/* Price Impact Warning */}
          {priceImpact > 5 && (
            <div style={{
              background: '#fff3cd',
              border: '1px solid #ffeaa7',
              borderRadius: '8px',
              padding: '8px',
              fontSize: '12px',
              color: '#856404',
              marginBottom: '10px'
            }}>
              ⚠️ High price impact: {priceImpact.toFixed(2)}%
            </div>
          )}
          
          {/* Insufficient Balance Warning */}
          {swapAmountIn > userBalanceTokenIn && (
            <div style={{
              background: '#f8d7da',
              border: '1px solid #f5c6cb',
              borderRadius: '8px',
              padding: '8px',
              fontSize: '12px',
              color: '#721c24',
              marginBottom: '10px'
            }}>
              ❌ Insufficient {tokenIn} balance: need {swapAmountIn}, have {userBalanceTokenIn}
            </div>
          )}
          
          <button
            onClick={() => handlePoolSwap(tokenA, tokenB)}
            disabled={loading || !poolState.swapAmountIn || swapAmountIn > userBalanceTokenIn || !pool}
            style={{
              width: '100%',
              padding: '12px',
              background: `linear-gradient(45deg, ${getTokenColor(tokenA)}, ${getTokenColor(tokenB)})`,
              color: 'white',
              border: 'none',
              borderRadius: '8px',
              fontSize: '14px',
              fontWeight: 'bold',
              cursor: 'pointer',
              opacity: (loading || !poolState.swapAmountIn || swapAmountIn > userBalanceTokenIn || !pool) ? 0.5 : 1
            }}
          >
            {loading ? 'Swapping...' : `Swap ${tokenIn} for ${tokenOut}`}
          </button>
        </div>

        {/* Liquidity Section */}
        <div className="liquidity-section">
          <h4 style={{ color: getTokenColor(tokenB), marginBottom: '15px' }}>
            💧 {poolState.liquidityMode === 'add' ? 'Add' : 'Remove'} Liquidity
          </h4>
          
          {/* Liquidity Mode Toggle */}
          <div style={{ display: 'flex', gap: '10px', marginBottom: '15px' }}>
            <button
              onClick={() => updatePoolState(pairKey, { liquidityMode: 'add' })}
              style={{
                background: poolState.liquidityMode === 'add' ? '#28a745' : 'transparent',
                color: poolState.liquidityMode === 'add' ? 'white' : '#28a745',
                border: '1px solid #28a745',
                borderRadius: '8px',
                padding: '8px 16px',
                fontSize: '12px',
                cursor: 'pointer'
              }}
            >
              Add Liquidity
            </button>
            <button
              onClick={() => updatePoolState(pairKey, { liquidityMode: 'remove' })}
              disabled={isNewPool}
              style={{
                background: poolState.liquidityMode === 'remove' ? '#dc3545' : 'transparent',
                color: poolState.liquidityMode === 'remove' ? 'white' : '#dc3545',
                border: '1px solid #dc3545',
                borderRadius: '8px',
                padding: '8px 16px',
                fontSize: '12px',
                cursor: 'pointer',
                opacity: isNewPool ? 0.5 : 1
              }}
            >
              Remove Liquidity
            </button>
          </div>
          
          {poolState.liquidityMode === 'add' ? (
            <>
              {/* Add Liquidity Inputs */}
              <div style={{ display: 'flex', gap: '10px', marginBottom: '10px' }}>
                <input
                  type="number"
                  placeholder={`${tokenA} amount`}
                  value={poolState.liquidityAmount}
                  onChange={(e) => updatePoolState(pairKey, { liquidityAmount: e.target.value })}
                  style={{
                    flex: 1,
                    padding: '12px',
                    borderRadius: '8px',
                    border: `2px solid ${getTokenColor(tokenA)}`,
                    fontSize: '14px'
                  }}
                />
                <input
                  type="number"
                  placeholder={`${tokenB} amount`}
                  value={isNewPool ? poolState.liquidityTokenAmount : liquidityAmountB.toFixed(2)}
                  onChange={(e) => isNewPool && updatePoolState(pairKey, { liquidityTokenAmount: e.target.value })}
                  readOnly={!isNewPool}
                  style={{
                    flex: 1,
                    padding: '12px',
                    borderRadius: '8px',
                    border: `2px solid ${getTokenColor(tokenB)}`,
                    backgroundColor: isNewPool ? 'white' : `${getTokenColor(tokenB)}11`,
                    fontSize: '14px'
                  }}
                />
              </div>
              
              {isNewPool ? (
                <div style={{
                  background: '#d4edda',
                  border: '1px solid #c3e6cb',
                  borderRadius: '8px',
                  padding: '8px',
                  fontSize: '12px',
                  color: '#155724',
                  marginBottom: '10px'
                }}>
                  🆕 Creating new pool - you can set both amounts to establish the initial price
                </div>
              ) : (
                <div style={{
                  background: '#d1ecf1',
                  border: '1px solid #bee5eb',
                  borderRadius: '8px',
                  padding: '8px',
                  fontSize: '12px',
                  color: '#0c5460',
                  marginBottom: '10px'
                }}>
                  📊 Adding to existing pool - {tokenB} amount is auto-calculated from current ratio
                </div>
              )}
              
              <button
                onClick={() => handlePoolAddLiquidity(tokenA, tokenB)}
                disabled={
                  loading || 
                  !poolState.liquidityAmount || 
                  (isNewPool && !poolState.liquidityTokenAmount) ||
                  liquidityAmountA > userBalanceA ||
                  liquidityAmountB > userBalanceB
                }
                style={{
                  width: '100%',
                  padding: '12px',
                  background: '#28a745',
                  color: 'white',
                  border: 'none',
                  borderRadius: '8px',
                  fontSize: '14px',
                  fontWeight: 'bold',
                  cursor: 'pointer',
                  opacity: (
                    loading || 
                    !poolState.liquidityAmount || 
                    (isNewPool && !poolState.liquidityTokenAmount) ||
                    liquidityAmountA > userBalanceA ||
                    liquidityAmountB > userBalanceB
                  ) ? 0.5 : 1
                }}
              >
                {loading ? 'Adding Liquidity...' : 'Add Liquidity'}
              </button>
            </>
          ) : (
            <>
              {/* Remove Liquidity */}
              <input
                type="number"
                placeholder="Liquidity tokens to remove"
                value={poolState.liquidityAmount}
                onChange={(e) => updatePoolState(pairKey, { liquidityAmount: e.target.value })}
                style={{
                  width: '100%',
                  padding: '12px',
                  borderRadius: '8px',
                  border: '2px solid #dc3545',
                  fontSize: '14px',
                  marginBottom: '10px'
                }}
              />
              
              <button
                onClick={() => handlePoolRemoveLiquidity(tokenA, tokenB)}
                disabled={loading || !poolState.liquidityAmount}
                style={{
                  width: '100%',
                  padding: '12px',
                  background: '#dc3545',
                  color: 'white',
                  border: 'none',
                  borderRadius: '8px',
                  fontSize: '14px',
                  fontWeight: 'bold',
                  cursor: 'pointer',
                  opacity: (loading || !poolState.liquidityAmount) ? 0.5 : 1
                }}
              >
                {loading ? 'Removing Liquidity...' : 'Remove Liquidity'}
              </button>
            </>
          )}
        </div>
      </div>
    );
  };

  // New function to render all pool interfaces
  const renderPoolInterfaces = () => {
    // Show demo mode pools if contract state is not available but we're verified
    if ((!contract1State?.state || contract1State?.error) && isVerified) {
      return (
        <div className="pool-interfaces">
          <h2 style={{ 
            textAlign: 'center', 
            marginBottom: '20px',
            background: 'linear-gradient(45deg, #FF6B35, #FFD23F, #4A90E2, #7ED321)',
            WebkitBackgroundClip: 'text',
            WebkitTextFillColor: 'transparent',
            fontSize: '24px',
            fontWeight: 'bold'
          }}>
            🏊‍♂️ Fruit Trading Pools (Demo Mode)
          </h2>
          <div className="pools-grid" style={{
            display: 'grid',
            gridTemplateColumns: 'repeat(auto-fit, minmax(400px, 1fr))',
            gap: '20px'
          }}>
            {FRUIT_PAIRS.map(([tokenA, tokenB]) => (
              <div key={`${tokenA}_${tokenB}`} className="pool-interface" style={{
                background: `linear-gradient(135deg, ${getTokenColor(tokenA)}11, ${getTokenColor(tokenB)}11)`,
                border: `2px solid ${getTokenColor(tokenA)}33`,
                borderRadius: '16px',
                padding: '20px',
                margin: '10px'
              }}>
                {/* Pool Header */}
                <div className="pool-header" style={{ marginBottom: '20px' }}>
                  <h3 style={{ 
                    background: `linear-gradient(45deg, ${getTokenColor(tokenA)}, ${getTokenColor(tokenB)})`,
                    WebkitBackgroundClip: 'text',
                    WebkitTextFillColor: 'transparent',
                    fontSize: '20px',
                    fontWeight: 'bold',
                    textAlign: 'center',
                    margin: '0 0 10px 0'
                  }}>
                    {getTokenEmoji(tokenA)}{tokenA} / {getTokenEmoji(tokenB)}{tokenB}
                  </h3>
                  
                  {/* Demo Pool Stats */}
                  <div className="pool-stats" style={{ 
                    display: 'grid', 
                    gridTemplateColumns: '1fr 1fr', 
                    gap: '10px',
                    fontSize: '12px',
                    background: 'rgba(255,255,255,0.1)',
                    borderRadius: '8px',
                    padding: '10px'
                  }}>
                    <div>
                      <strong>Reserves:</strong><br/>
                      No pool (Demo)
                    </div>
                    <div>
                      <strong>Price:</strong><br/>
                      Not available
                    </div>
                    <div>
                      <strong>Liquidity:</strong><br/>
                      0
                    </div>
                    <div>
                      <strong>Your Balance:</strong><br/>
                      0 {tokenA} | 0 {tokenB}
                    </div>
                  </div>
                </div>

                {/* Demo Swap Section */}
                <div className="swap-section" style={{ marginBottom: '20px' }}>
                  <h4 style={{ color: getTokenColor(tokenA), marginBottom: '15px' }}>
                    🔄 Swap {tokenA}/{tokenB} (Demo)
                  </h4>
                  
                  <div style={{ display: 'flex', gap: '10px', marginBottom: '10px' }}>
                    <input
                      type="number"
                      placeholder="Demo mode"
                      disabled={true}
                      style={{
                        flex: 1,
                        padding: '12px',
                        borderRadius: '8px',
                        border: `2px solid ${getTokenColor(tokenA)}`,
                        fontSize: '14px',
                        opacity: 0.6
                      }}
                    />
                    <input
                      type="number"
                      placeholder="Demo mode"
                      disabled={true}
                      style={{
                        flex: 1,
                        padding: '12px',
                        borderRadius: '8px',
                        border: `2px solid ${getTokenColor(tokenB)}`,
                        backgroundColor: `${getTokenColor(tokenB)}11`,
                        fontSize: '14px',
                        opacity: 0.6
                      }}
                    />
                  </div>
                  
                  <button
                    disabled={true}
                    style={{
                      width: '100%',
                      padding: '12px',
                      background: `linear-gradient(45deg, ${getTokenColor(tokenA)}, ${getTokenColor(tokenB)})`,
                      color: 'white',
                      border: 'none',
                      borderRadius: '8px',
                      fontSize: '14px',
                      fontWeight: 'bold',
                      cursor: 'not-allowed',
                      opacity: 0.6
                    }}
                  >
                    Demo Mode - Start Server
                  </button>
                </div>

                {/* Demo Liquidity Section */}
                <div className="liquidity-section">
                  <h4 style={{ color: getTokenColor(tokenB), marginBottom: '15px' }}>
                    💧 Add Liquidity (Demo)
                  </h4>
                  
                  <div style={{ display: 'flex', gap: '10px', marginBottom: '10px' }}>
                    <input
                      type="number"
                      placeholder="Demo mode"
                      disabled={true}
                      style={{
                        flex: 1,
                        padding: '12px',
                        borderRadius: '8px',
                        border: `2px solid ${getTokenColor(tokenA)}`,
                        fontSize: '14px',
                        opacity: 0.6
                      }}
                    />
                    <input
                      type="number"
                      placeholder="Demo mode"
                      disabled={true}
                      style={{
                        flex: 1,
                        padding: '12px',
                        borderRadius: '8px',
                        border: `2px solid ${getTokenColor(tokenB)}`,
                        backgroundColor: `${getTokenColor(tokenB)}11`,
                        fontSize: '14px',
                        opacity: 0.6
                      }}
                    />
                  </div>
                  
                  <button
                    disabled={true}
                    style={{
                      width: '100%',
                      padding: '12px',
                      background: '#28a745',
                      color: 'white',
                      border: 'none',
                      borderRadius: '8px',
                      fontSize: '14px',
                      fontWeight: 'bold',
                      cursor: 'not-allowed',
                      opacity: 0.6
                    }}
                  >
                    Demo Mode - Start Server
                  </button>
                </div>
              </div>
            ))}
          </div>
          <div style={{
            textAlign: 'center',
            marginTop: '20px',
            padding: '20px',
            background: '#e7f3ff',
            border: '1px solid #b3d9ff',
            borderRadius: '12px',
            color: '#0066cc'
          }}>
            <div style={{ fontSize: '24px', marginBottom: '10px' }}>🚀</div>
            <strong>Ready to Trade!</strong><br/>
            Start the server to enable full AMM functionality:<br/>
            <code style={{ 
              background: '#f8f9fa', 
              padding: '4px 8px', 
              borderRadius: '4px',
              marginTop: '10px',
              display: 'inline-block'
            }}>
              RISC0_DEV_MODE=1 cargo run -p server
            </code>
          </div>
        </div>
      );
    }

    // Normal operation with loaded contract state
    return (
      <div className="pool-interfaces">
        <h2 style={{ 
          textAlign: 'center', 
          marginBottom: '20px',
          background: 'linear-gradient(45deg, #FF6B35, #FFD23F, #4A90E2, #7ED321)',
          WebkitBackgroundClip: 'text',
          WebkitTextFillColor: 'transparent',
          fontSize: '24px',
          fontWeight: 'bold'
        }}>
          🏊‍♂️ Fruit Trading Pools
        </h2>
        <div className="pools-grid" style={{
          display: 'grid',
          gridTemplateColumns: 'repeat(auto-fit, minmax(400px, 1fr))',
          gap: '20px'
        }}>
          {FRUIT_PAIRS.map(([tokenA, tokenB]) => (
            <div key={`${tokenA}_${tokenB}`}>
              {renderPoolInterface(tokenA, tokenB)}
            </div>
          ))}
        </div>
      </div>
    );
  };

  return (
    <div className="amm-app">
      <div className="app-header">
        <h1 className="app-title">
          <span className="fruit-gradient">Fruit Swap</span> 🍇
        </h1>
        <p className="app-subtitle">
          Powered by <span className="tech-highlight">ZKPassport</span> & <span className="tech-highlight">Boundless</span>
        </p>
        <button 
          className="logout-button"
          onClick={logout}
        >
          Logout 👋
        </button>
      </div>
      
      {/* Always render ZKPassportVerifier to avoid hooks ordering issues */}
      <div style={{ display: showVerification ? 'block' : 'none' }}>
        <ZKPassportVerifier
          onVerificationComplete={handleVerificationComplete}
          onError={handleVerificationError}
        />
      </div>
      
      {/* Compliance gate - only show when not verified and not showing verification */}
      {!isVerified && !showVerification && (
        <div className="compliance-gate">
          <div className="compliance-container">
            <h2>🛂 Compliance Verification Required</h2>
            <div className="compliance-info">
              <p>
                To trade on this decentralized AMM, you must verify that you are younger than 25. 
                This verification is done privately using ZKPassport - we never see your passport data.
              </p>
              <div className="compliance-features">
                <div className="feature">
                  <span className="feature-icon">🔐</span>
                  <span>Zero-knowledge proof</span>
                </div>
                <div className="feature">
                  <span className="feature-icon">🛂</span>
                  <span>Age verification</span>
                </div>
                <div className="feature">
                  <span className="feature-icon">🚫</span>
                  <span>No personal data shared</span>
                </div>
                <div className="feature">
                  <span className="feature-icon">⚡</span>
                  <span>One-time verification</span>
                </div>
              </div>
              
              <div className="compliance-actions">
                <button 
                  className="verify-button"
                  onClick={startVerification}
                >
                  🚀 Start ZKPassport Verification
                </button>
                <button 
                  className="skip-button"
                  onClick={skipVerification}
                >
                  ⚠️ Skip (Demo Mode)
                </button>
              </div>
              
              <div className="compliance-disclaimer">
                <p>
                  <strong>Note:</strong> This verification ensures compliance with financial regulations. 
                  The proof is generated locally on your device and only confirms your age status.
                </p>
              </div>
            </div>
          </div>
        </div>
      )}

      {/* Main AMM interface - only show when verified */}
      {isVerified && (
        <>
          {/* Magical background elements */}
          <div className="hyli-dust">
            <div className="dust-particle"></div>
            <div className="dust-particle"></div>
            <div className="dust-particle"></div>
            <div className="dust-particle"></div>
            <div className="dust-particle"></div>
          </div>
          
          <div className="app-tagline">Privacy-preserving fruit trading with magical zero-knowledge proofs ✨</div>
          <div className="header-actions">
            <div className="verification-status">
              ✅ Verified: {verificationResult?.uniqueIdentifier.substring(0, 8) || 'Demo'}...
            </div>
          </div>

          <div className="user-selector">
            <label>🧙‍♂️ Current User:</label>
            <select 
              value={currentUser} 
              onChange={(e) => setCurrentUser(e.target.value)}
              disabled={loading}
            >
              {availableUsers.map(user => (
                <option key={user} value={user}>{user}</option>
              ))}
            </select>
          </div>

          <div className="wallet-info">
            <span className="wallet-label">🔗 Connected Wallet:</span>
            <span className="wallet-value">{wallet?.address || 'Not connected'}</span>
          </div>

          {/* Main Dashboard - New per-pool architecture */}
          <div className="main-dashboard">
            <div className="dashboard-row">
              <div className="dashboard-card full-width">
                {renderBalanceCard()}
              </div>
            </div>
            
            <div className="dashboard-row">
              <div className="dashboard-card full-width">
                {renderPoolInterfaces()}
              </div>
            </div>
          </div>

          <div className="contract-states">
            <div className="contract-state">
              <h3>🔒 AMM Contract State</h3>
              {contract1State?.error ? (
                <div className="error">{contract1State.error}</div>
              ) : (
                <pre className="state-display">{contract1State?.state ? JSON.stringify(contract1State.state, null, 2) : 'Loading...'}</pre>
              )}
            </div>
            <div className="contract-state">
              <h3>👤 Identity Contract State</h3>
              {contract2State?.error ? (
                <div className="error">{contract2State.error}</div>
              ) : (
                <pre className="state-display">{contract2State?.state ? JSON.stringify(contract2State.state, null, 2) : 'Loading...'}</pre>
              )}
            </div>
          </div>
        </>
      )}
      
      {result && <div className="transaction-result">{result}</div>}
    </div>
  );
}

function LandingPage() {
  return (
    <div className="wallet-page-wrapper">
      <div className="landing-content-simple">
        <h1 className="hero-title">
          <span className="gradient-text">Fruit Swap</span> 🍇
        </h1>
        <p className="hero-subtitle">
          Powered by <span className="tech-highlight">ZKPassport</span> & <span className="tech-highlight">Boundless</span>
        </p>
        <p className="hero-description">
          Privacy-preserving fruit trading with magical zero-knowledge proofs ✨
        </p>
        <HyliWallet
          providers={["password", "google", "github"]}
        />
      </div>
      <div className="floating-shapes">
        <div className="shape shape-1">🍊</div>
        <div className="shape shape-2">🍋</div>
        <div className="shape shape-3">🫐</div>
        <div className="shape shape-4">🍈</div>
      </div>
    </div>
  );
}

function AppContent() {
  const { wallet } = useWallet();
  
  if (!wallet) {
    return <LandingPage />;
  }
  
  return <ScaffoldApp />;
}

function App() {
  return (
    <WalletProvider
      config={{
        nodeBaseUrl: import.meta.env.VITE_NODE_BASE_URL,
        walletServerBaseUrl: import.meta.env.VITE_WALLET_SERVER_BASE_URL,
        applicationWsUrl: import.meta.env.VITE_WALLET_WS_URL,
      }}
      sessionKeyConfig={{
        duration: 24 * 60 * 60 * 1000,
        whitelist: ["contract1", "contract2"],
      }}
    >
      <AppContent />
    </WalletProvider>
  )
}

export default App;
