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

// Exchange rate constants based on requirements
const EXCHANGE_RATES = {
  'MELON_ORANJ': { base: 'MELON', quote: 'ORANJ', rate: 5 }, // 1 Melon = 5 Oranj
  'ORANJ_VITAMINE': { base: 'ORANJ', quote: 'VITAMINE', rate: 200 }, // 1 Oranj = 200 Vitamine  
  'MELON_VITAMINE': { base: 'MELON', quote: 'VITAMINE', rate: 100 }, // 1 Melon = 100 Vitamine
  'VITAMINE_OXYGENE': { base: 'VITAMINE', quote: 'OXYGENE', rate: 5 }, // 1 Vitamine = 5 Oxygene
};

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
  
  // AMM State
  const [currentUser, setCurrentUser] = useState('bob@wallet');
  // const [tokenBalances, setTokenBalances] = useState<TokenBalance>({});
  
  // Swap State
  const [swapTokenIn, setSwapTokenIn] = useState('ORANJ');
  const [swapTokenOut, setSwapTokenOut] = useState('VITAMINE');
  const [swapAmountIn, setSwapAmountIn] = useState('');
  const [swapAmountOut, setSwapAmountOut] = useState('');
  
  // Liquidity State
  const [liquidityTokenA, setLiquidityTokenA] = useState('ORANJ');
  const [liquidityTokenB, setLiquidityTokenB] = useState('VITAMINE');
  const [liquidityAmountA, setLiquidityAmountA] = useState('');
  const [liquidityAmountB, setLiquidityAmountB] = useState('');

  const availableTokens = ['ORANJ', 'VITAMINE', 'OXYGENE', 'MELON'];
  const availableUsers = ['bob@wallet', 'alice@contract1', 'charlie@contract1', 'dave@wallet'];

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

  const handleSwap = async () => {
    if (!swapAmountIn || !swapTokenIn || !swapTokenOut) {
      setResult('Please fill in all swap fields');
      return;
    }

    await sendTransaction('/api/swap-tokens', {
      token_in: swapTokenIn,
      token_out: swapTokenOut,
      amount_in: parseInt(swapAmountIn),
      min_amount_out: parseInt(swapAmountOut) || 0
    });
  };

  const handleAddLiquidity = async () => {
    if (!liquidityAmountA || !liquidityAmountB || !liquidityTokenA || !liquidityTokenB) {
      setResult('Please fill in all liquidity fields');
      return;
    }

    // Check if this is a valid fruit pair
    const isValidPair = FRUIT_PAIRS.some(pair => 
      (pair.includes(liquidityTokenA) && pair.includes(liquidityTokenB))
    );

    if (!isValidPair) {
      setResult('❌ Invalid fruit pair! You can only add liquidity between: MELON/ORANJ, ORANJ/VITAMINE, MELON/VITAMINE, or VITAMINE/OXYGENE');
      return;
    }

    await sendTransaction('/api/add-liquidity', {
      token_a: liquidityTokenA,
      token_b: liquidityTokenB,
      amount_a: parseInt(liquidityAmountA),
      amount_b: parseInt(liquidityAmountB)
    });
  };

  // Calculate estimated output for swap with proper exchange rates
  const calculateSwapOutput = () => {
    if (!swapAmountIn) return '';
    const amount = parseFloat(swapAmountIn);
    const fee = 0.003; // 0.3% fee
    const amountAfterFee = amount * (1 - fee);
    
    // Use predefined exchange rates
    const pairKey = `${swapTokenIn}_${swapTokenOut}`;
    const reversePairKey = `${swapTokenOut}_${swapTokenIn}`;
    
    let rate = 1;
    if (EXCHANGE_RATES[pairKey as keyof typeof EXCHANGE_RATES]) {
      rate = EXCHANGE_RATES[pairKey as keyof typeof EXCHANGE_RATES].rate;
    } else if (EXCHANGE_RATES[reversePairKey as keyof typeof EXCHANGE_RATES]) {
      rate = 1 / EXCHANGE_RATES[reversePairKey as keyof typeof EXCHANGE_RATES].rate;
    } else {
      // Calculate indirect rate through common pairs
      if (swapTokenIn === 'MELON' && swapTokenOut === 'OXYGENE') {
        // MELON -> VITAMINE -> OXYGENE: 100 * 5 = 500
        rate = 500;
      } else if (swapTokenIn === 'OXYGENE' && swapTokenOut === 'MELON') {
        // OXYGENE -> VITAMINE -> MELON: 1/5 * 1/100 = 1/500
        rate = 1/500;
      } else if (swapTokenIn === 'ORANJ' && swapTokenOut === 'OXYGENE') {
        // ORANJ -> VITAMINE -> OXYGENE: 200 * 5 = 1000
        rate = 1000;
      } else if (swapTokenIn === 'OXYGENE' && swapTokenOut === 'ORANJ') {
        // OXYGENE -> VITAMINE -> ORANJ: 1/5 * 1/200 = 1/1000
        rate = 1/1000;
      }
    }
    
    return (amountAfterFee * rate).toFixed(4);
  };

  // Auto-calculate liquidity amounts based on pool ratios
  const calculateLiquidityAmountB = () => {
    if (!liquidityAmountA) return '';
    const amount = parseFloat(liquidityAmountA);
    
    const pairKey = `${liquidityTokenA}_${liquidityTokenB}`;
    const reversePairKey = `${liquidityTokenB}_${liquidityTokenA}`;
    
    let rate = 1;
    if (EXCHANGE_RATES[pairKey as keyof typeof EXCHANGE_RATES]) {
      rate = EXCHANGE_RATES[pairKey as keyof typeof EXCHANGE_RATES].rate;
    } else if (EXCHANGE_RATES[reversePairKey as keyof typeof EXCHANGE_RATES]) {
      rate = 1 / EXCHANGE_RATES[reversePairKey as keyof typeof EXCHANGE_RATES].rate;
    }
    
    return (amount * rate).toFixed(0);
  };

  useEffect(() => {
    const estimated = calculateSwapOutput();
    setSwapAmountOut(estimated);
  }, [swapAmountIn, swapTokenIn, swapTokenOut]);

  useEffect(() => {
    const calculatedB = calculateLiquidityAmountB();
    setLiquidityAmountB(calculatedB);
  }, [liquidityAmountA, liquidityTokenA, liquidityTokenB]);

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
    if (!contract1State?.state) return null;
    
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

  const renderSwapInterface = () => (
    <div className="swap-interface colorful-swap">
      <h3>🌈 Magical Fruit Swap</h3>
      <div className="fee-display rainbow-text">Fee: 0.3% ✨ | Real Exchange Rates 📊</div>
      <div className="swap-container">
        <div className="token-input-group colorful-input">
          <label>From 🚀</label>
          <div className="token-input">
            <select 
              value={swapTokenIn} 
              onChange={(e) => setSwapTokenIn(e.target.value)}
              disabled={loading}
              style={{ borderColor: getTokenColor(swapTokenIn) }}
            >
              {availableTokens.map(token => (
                <option key={token} value={token}>{getTokenEmoji(token)} {token}</option>
              ))}
            </select>
            <input
              type="number"
              placeholder="0.0"
              value={swapAmountIn}
              onChange={(e) => setSwapAmountIn(e.target.value)}
              disabled={loading}
              style={{ borderColor: getTokenColor(swapTokenIn) }}
            />
          </div>
        </div>
        
        <div className="swap-arrow rainbow-arrow">⬇✨🔥</div>
        
        <div className="token-input-group colorful-input">
          <label>To (after 0.3% fee) 🎯</label>
          <div className="token-input">
            <select 
              value={swapTokenOut} 
              onChange={(e) => setSwapTokenOut(e.target.value)}
              disabled={loading}
              style={{ borderColor: getTokenColor(swapTokenOut) }}
            >
              {availableTokens.filter(t => t !== swapTokenIn).map(token => (
                <option key={token} value={token}>{getTokenEmoji(token)} {token}</option>
              ))}
            </select>
            <input
              type="number"
              placeholder="0.0"
              value={swapAmountOut}
              readOnly
              className="estimated-output"
              style={{ borderColor: getTokenColor(swapTokenOut), backgroundColor: `${getTokenColor(swapTokenOut)}11` }}
            />
          </div>
        </div>
        
        <button 
          className="action-button swap-button rainbow-swap-btn"
          onClick={handleSwap}
          disabled={loading || !swapAmountIn}
        >
          {loading ? 'Swapping Fruits... 🌪️' : 'Swap Magic Fruits ✨🔮'}
        </button>
      </div>
    </div>
  );

  const renderLiquidityInterface = () => (
    <div className="liquidity-interface colorful-liquidity">
      <h3>🌊 Magical Liquidity Pools</h3>
      <div className="valid-pairs-info">
        <span className="pairs-label">🍎 Valid Fruit Pairs:</span>
        <div className="pairs-list">
          {FRUIT_PAIRS.map((pair, index) => (
            <span key={index} className="pair-badge" style={{ 
              background: `linear-gradient(45deg, ${getTokenColor(pair[0])}, ${getTokenColor(pair[1])})` 
            }}>
              {getTokenEmoji(pair[0])}{pair[0]}/{getTokenEmoji(pair[1])}{pair[1]}
            </span>
          ))}
        </div>
      </div>
      <div className="liquidity-container">
        <div className="token-pair-inputs">
          <div className="token-input-group colorful-input">
            <label>Fruit A 🥝</label>
            <div className="token-input">
              <select 
                value={liquidityTokenA} 
                onChange={(e) => setLiquidityTokenA(e.target.value)}
                disabled={loading}
                style={{ borderColor: getTokenColor(liquidityTokenA) }}
              >
                {availableTokens.map(token => (
                  <option key={token} value={token}>{getTokenEmoji(token)} {token}</option>
                ))}
              </select>
              <input
                type="number"
                placeholder="0.0"
                value={liquidityAmountA}
                onChange={(e) => setLiquidityAmountA(e.target.value)}
                disabled={loading}
                style={{ borderColor: getTokenColor(liquidityTokenA) }}
              />
            </div>
          </div>
          
          <div className="plus-sign rainbow-plus">+ 🌟</div>
          
          <div className="token-input-group colorful-input">
            <label>Fruit B 🥭</label>
            <div className="token-input">
              <select 
                value={liquidityTokenB} 
                onChange={(e) => setLiquidityTokenB(e.target.value)}
                disabled={loading}
                style={{ borderColor: getTokenColor(liquidityTokenB) }}
              >
                {availableTokens.filter(t => t !== liquidityTokenA).map(token => (
                  <option key={token} value={token}>{getTokenEmoji(token)} {token}</option>
                ))}
              </select>
              <input
                type="number"
                placeholder="Auto-calculated"
                value={liquidityAmountB}
                readOnly
                className="auto-calculated"
                style={{ borderColor: getTokenColor(liquidityTokenB), backgroundColor: `${getTokenColor(liquidityTokenB)}11` }}
              />
            </div>
          </div>
        </div>
        
        <button 
          className="action-button liquidity-button rainbow-liquidity-btn"
          onClick={handleAddLiquidity}
          disabled={loading || !liquidityAmountA || !liquidityAmountB}
        >
          {loading ? 'Adding Liquidity... 🌊' : 'Add Magical Liquidity 💫🌈'}
        </button>
      </div>
    </div>
  );

  const renderPoolInfo = () => {
    if (!contract1State?.state) return null;
    
    const state = contract1State.state as any;
    const pools = state.pools || {};
    
    return (
      <div className="pool-info">
        <h3>🏊 Active Fruit Pools</h3>
        {Object.keys(pools).length === 0 ? (
          <p className="no-pools">No fruit pools yet. Initialize pools to start trading! 🌱</p>
        ) : (
          <div className="pools-grid">
            {Object.entries(pools).map(([poolKey, pool]: [string, any]) => (
              <div key={poolKey} className="pool-card colorful-pool" style={{
                background: `linear-gradient(135deg, ${getTokenColor(pool.token_a)}22, ${getTokenColor(pool.token_b)}22)`
              }}>
                <h4>
                  {getTokenEmoji(pool.token_a)}{pool.token_a}/{getTokenEmoji(pool.token_b)}{pool.token_b}
                </h4>
                <div className="pool-reserves">
                  <div className="reserve-item">
                    <span style={{ color: getTokenColor(pool.token_a) }}>{pool.token_a}:</span>
                    <span>{pool.reserve_a?.toLocaleString() || 0}</span>
                  </div>
                  <div className="reserve-item">
                    <span style={{ color: getTokenColor(pool.token_b) }}>{pool.token_b}:</span>
                    <span>{pool.reserve_b?.toLocaleString() || 0}</span>
                  </div>
                  <div className="reserve-item total">
                    <span>Total Liquidity:</span>
                    <span>{pool.total_liquidity?.toLocaleString() || 0}</span>
                  </div>
                  <div className="pool-fee">
                    <span>Pool Fee: 0.3% 💎</span>
                  </div>
                </div>
              </div>
            ))}
          </div>
        )}
      </div>
    );
  };

  // Show ZKPassport verification if user is not verified
  if (showVerification) {
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
        
        <ZKPassportVerifier
          onVerificationComplete={handleVerificationComplete}
          onError={handleVerificationError}
        />
        
        {result && <div className="transaction-result">{result}</div>}
      </div>
    );
  }

  // Show compliance gate if user hasn't verified yet
  if (!isVerified) {
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

        <div className="compliance-gate">
          <div className="compliance-container">
            <h2>🛂 Compliance Verification Required</h2>
            <div className="compliance-info">
              <p>
                To trade on this decentralized AMM, you must verify that you are not a US citizen. 
                This verification is done privately using ZKPassport - we never see your passport data.
              </p>
              <div className="compliance-features">
                <div className="feature">
                  <span className="feature-icon">🔐</span>
                  <span>Zero-knowledge proof</span>
                </div>
                <div className="feature">
                  <span className="feature-icon">🛂</span>
                  <span>Passport verification</span>
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
                  The proof is generated locally on your device and only confirms your nationality status.
                </p>
              </div>
            </div>
          </div>
        </div>
        
        {result && <div className="transaction-result">{result}</div>}
      </div>
    );
  }

  return (
    <div className="amm-app">
      {/* Magical background elements */}
      <div className="hyli-dust">
        <div className="dust-particle"></div>
        <div className="dust-particle"></div>
        <div className="dust-particle"></div>
        <div className="dust-particle"></div>
        <div className="dust-particle"></div>
      </div>
      
      <div className="app-header">
        <h1 className="app-title">
          <span className="fruit-gradient">Fruit Swap</span> 🍇
        </h1>
        <p className="app-subtitle">
          Powered by <span className="tech-highlight">ZKPassport</span> & <span className="tech-highlight">Boundless</span>
        </p>
        <p className="app-tagline">Privacy-preserving fruit trading with magical zero-knowledge proofs ✨</p>
        <div className="header-actions">
          <div className="verification-status">
            ✅ Verified: {verificationResult?.uniqueIdentifier.substring(0, 8) || 'Demo'}...
          </div>
          <button 
            className="logout-button"
            onClick={logout}
          >
            Logout 👋
          </button>
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

      {/* Main Dashboard - All sections visible */}
      <div className="main-dashboard">
        <div className="dashboard-row">
          <div className="dashboard-card">
            {renderBalanceCard()}
          </div>
          <div className="dashboard-card">
            {renderSwapInterface()}
          </div>
        </div>
        
        <div className="dashboard-row">
          <div className="dashboard-card">
            {renderLiquidityInterface()}
          </div>
          <div className="dashboard-card">
            {renderPoolInfo()}
          </div>
        </div>
      </div>

      {result && <div className="transaction-result">{result}</div>}

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
