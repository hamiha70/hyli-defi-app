import { useState, useEffect } from 'react';
import { WalletProvider, HyliWallet, useWallet } from "hyli-wallet";
import './App.css';
import './WalletStyles.css';

interface ContractState {
  state: unknown;
  error?: string;
}

function ScaffoldApp() {
  const { logout, wallet, createIdentityBlobs } = useWallet();
  const [contract1State, setContract1State] = useState<ContractState | null>(null);
  const [contract2State, setContract2State] = useState<ContractState | null>(null);
  const [loading, setLoading] = useState(false);
  const [initialResult, setInitialResult] = useState<string | null>(null);
  const [confirmationResult, setConfirmationResult] = useState<string | null>(null);

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
    // Refresh states every minute
    const interval = setInterval(fetchStates, 60000);
    return () => clearInterval(interval);
  }, []);

  const pollTransactionStatus = async (txHash: string): Promise<void> => {
    const maxAttempts = 1200; // 20 minutes timeout
    let attempts = 0;
    
    // Update confirmation result to show progress
    setConfirmationResult(`Transaction submitted! Executing contract and generating proof... (Hash: ${txHash})`);
    
    while (attempts < maxAttempts) {
      try {
        const response = await fetch(`${import.meta.env.VITE_NODE_BASE_URL}/v1/indexer/transaction/hash/${txHash}`);
        if (!response.ok) {
          throw new Error(`HTTP error ${response.status}`);
        }
        
        const data = await response.json();
        if (data.transaction_status === "Success") {
          setConfirmationResult(`✅ AMM Transaction confirmed successful! Tokens minted. Hash: ${txHash}`);
          return;
        }
        
        // Update progress every 30 seconds
        if (attempts % 30 === 0 && attempts > 0) {
          const minutes = Math.floor(attempts / 60);
          setConfirmationResult(`⏳ Still processing... (${minutes}m elapsed) - Contract execution and proof generation in progress. Hash: ${txHash}`);
        }
        
        // Wait 1 second before next attempt
        await new Promise(resolve => setTimeout(resolve, 1000));
        attempts++;
      } catch (error) {
        console.error('Error polling transaction:', error);
        // Continue polling even if there's an error
      }
    }
    
    setConfirmationResult(`⚠️ Transaction ${txHash} polling timed out after ${Math.floor(maxAttempts/60)} minutes. Transaction may still be processing - check server logs.`);
  };

  const sendBlobTx = async () => {
    setInitialResult('');
    if (!wallet?.address) {
      setInitialResult('Wallet not connected');
      setConfirmationResult(null);
      return;
    }

    setLoading(true);
    setConfirmationResult(null);
    try {
      // Create identity blobs
      const [blob0, blob1] = createIdentityBlobs();
      
      const headers = new Headers();
      headers.append('content-type', 'application/json');
      headers.append('x-user', wallet.address);
      headers.append('x-session-key', 'test-session');
      headers.append('x-request-signature', 'test-signature');

      const response = await fetch(`${import.meta.env.VITE_SERVER_BASE_URL}/api/test-amm`, {
        method: 'POST',
        headers: headers,
        body: JSON.stringify({
          wallet_blobs: [blob0, blob1]
        })
      });
      
      if (!response.ok) {
        const errorText = await response.text();
        throw new Error(errorText || `HTTP error ${response.status}`);
      }

      const data = await response.json();
      setInitialResult(`AMM Transaction submitted! Minting tokens... Hash: ${JSON.stringify(data)}`);
      
      // Start polling for transaction status
      await pollTransactionStatus(data);
    } catch (error) {
      console.error('Error sending transaction:', error);
      setInitialResult(`Error: ${error instanceof Error ? error.message : String(error)}`);
      setConfirmationResult(null);
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="App">
      <button 
        className="logout-button"
        onClick={logout}
        style={{ position: 'absolute', top: '24px', right: '24px' }}
      >
        Logout
      </button>
      <div className="app-header">
        <h1 className="app-title">Hyli AMM Interface</h1>
        <p className="app-subtitle">Privacy-preserving Automated Market Maker with ZK proofs</p>
      </div>
      <div className="wallet-info">
        <div className="wallet-address">
          <span className="wallet-label">Connected Wallet:</span>
          <span className="wallet-value">{wallet?.address || 'Not connected'}</span>
        </div>
      </div>
      <button 
        className="blob-button" 
        onClick={sendBlobTx}
        disabled={loading}
      >
        {loading ? 'MINTING TOKENS...' : 'MINT AMM TOKENS (Test)'}
      </button>
      {initialResult && <div className="result">{initialResult}</div>}
      {confirmationResult && <div className="result">{confirmationResult}</div>}
      <div className="contract-states">
        <div className="contract-state">
          <h2>AMM Contract State (Contract 1)</h2>
          {contract1State?.error ? (
            <div className="error">{contract1State.error}</div>
          ) : (
            <pre>{contract1State?.state ? JSON.stringify(contract1State.state, null, 2) : 'Loading...'}</pre>
          )}
        </div>
        <div className="contract-state">
          <h2>Identity Contract State (Contract 2)</h2>
          {contract2State?.error ? (
            <div className="error">{contract2State.error}</div>
          ) : (
            <pre>{contract2State?.state ? JSON.stringify(contract2State.state, null, 2) : 'Loading...'}</pre>
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
                    <span className="gradient-text">Hyli</span> App Scaffold
                </h1>
                <p className="hero-subtitle">
                    A starting point for your next blockchain application
                </p>
                <HyliWallet
                    providers={["password", "google", "github"]}
                />
            </div>
            <div className="floating-shapes">
                <div className="shape shape-1"></div>
                <div className="shape shape-2"></div>
                <div className="shape shape-3"></div>
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
                duration: 24 * 60 * 60 * 1000, // Session key duration in ms (default: 72h)
                whitelist: ["contract1", "contract2"], // Required: contracts allowed for session key
            }}
        >
            <AppContent />
        </WalletProvider>
    )
}

export default App;
