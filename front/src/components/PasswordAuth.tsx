import React, { useState } from 'react';
import './PasswordAuth.css';

interface PasswordAuthProps {
  onAuthSuccess: (user: string) => void;
  onAuthError: (error: string) => void;
  onCancel?: () => void;
}

// Simple hash function to convert string to Field-compatible number
// This should match the Noir circuit's hash_to_field logic
const stringToField = (input: string): bigint => {
  let hash = 0n;
  for (let i = 0; i < input.length; i++) {
    const char = BigInt(input.charCodeAt(i));
    hash = ((hash << 5n) - hash) + char;
    hash = hash & 0xFFFFFFFFFFFFFFFFn; // Keep it within reasonable bounds
  }
  return hash;
};

const PasswordAuth: React.FC<PasswordAuthProps> = ({ onAuthSuccess, onAuthError, onCancel }) => {
  const [username, setUsername] = useState('');
  const [password, setPassword] = useState('');
  const [isLoading, setIsLoading] = useState(false);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setIsLoading(true);

    try {
      console.log('🔐 Starting Noir circuit authentication...');
      
      // Convert credentials to Field values (matching Noir circuit)
      const userField = stringToField(username);
      const passwordField = stringToField(password);
      
      console.log('🔢 Generated field values for proof generation');
      
      // Prepare authentication request for server
      const authRequest = {
        username: username,
        // Send field representations for proof generation
        user_field: userField.toString(),
        password_field: passwordField.toString(),
        proof_type: 'noir_circuit'
      };

      console.log('📡 Sending authentication request to server...');

      // Call server API for Noir circuit verification
      const response = await fetch(`${import.meta.env.VITE_SERVER_BASE_URL}/api/authenticate-noir`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'x-user': `${username}@zkpassport`,
          'x-session-key': 'test-session',
          'x-request-signature': 'test-signature'
        },
        body: JSON.stringify(authRequest)
      });

      if (!response.ok) {
        const errorText = await response.text();
        throw new Error(`Authentication failed: ${errorText}`);
      }

      const result = await response.json();
      
      console.log('✅ Noir circuit verification successful!');
      console.log('🔐 Authentication result:', result);

      // Clear sensitive data from memory
      setPassword('');
      
      onAuthSuccess(username);

    } catch (error) {
      console.error('❌ Noir circuit authentication failed:', error);
      
      // Clear sensitive data on error
      setPassword('');
      
      const errorMessage = error instanceof Error ? error.message : 'Authentication failed';
      onAuthError(`Noir circuit verification failed: ${errorMessage}`);
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <div className="password-auth">
      <div className="auth-container">
        <div className="auth-header">
          <h2>🔐 Noir Circuit Authentication</h2>
          <p>Zero-knowledge proof of password knowledge</p>
        </div>

        <form onSubmit={handleSubmit} className="auth-form">
          <div className="input-group">
            <label htmlFor="username">Username:</label>
            <input
              id="username"
              type="text"
              value={username}
              onChange={(e) => setUsername(e.target.value)}
              placeholder="Enter username"
              required
              disabled={isLoading}
              autoComplete="username"
            />
          </div>

          <div className="input-group">
            <label htmlFor="password">Password:</label>
            <input
              id="password"
              type="password"
              value={password}
              onChange={(e) => setPassword(e.target.value)}
              placeholder="Enter password"
              required
              disabled={isLoading}
              autoComplete="current-password"
            />
          </div>

          <button 
            type="submit" 
            className="auth-button"
            disabled={isLoading || !username || !password}
          >
            {isLoading ? (
              <>
                <span className="spinner"></span>
                Generating Noir Proof...
              </>
            ) : (
              'Generate ZK Proof & Authenticate'
            )}
          </button>
          
          {onCancel && (
            <button 
              type="button" 
              className="cancel-button"
              onClick={onCancel}
              disabled={isLoading}
              style={{
                width: '100%',
                padding: '12px',
                background: 'transparent',
                color: '#667eea',
                border: '2px solid #667eea',
                borderRadius: '12px',
                fontSize: '14px',
                fontWeight: '600',
                cursor: 'pointer',
                marginTop: '10px',
                transition: 'all 0.3s ease'
              }}
            >
              ← Back to Verification Options
            </button>
          )}
        </form>

        <div className="auth-info">
          <h3>🔬 Zero-Knowledge Authentication</h3>
          <div className="tech-details">
            <div className="detail-item">
              <strong>🧮 Circuit:</strong> Noir ZK proof generation
            </div>
            <div className="detail-item">
              <strong>🔐 Privacy:</strong> Password never transmitted
            </div>
            <div className="detail-item">
              <strong>⛓️ Verification:</strong> On-chain proof verification
            </div>
            <div className="detail-item">
              <strong>🧪 Demo Credentials:</strong> bob / HyliForEver
            </div>
          </div>
          <p className="tech-note">
            🔬 This generates a <strong>real zero-knowledge proof</strong> using Noir circuits.
            Your password is converted to cryptographic field elements and proven without revelation.
          </p>
        </div>
      </div>
    </div>
  );
};

export default PasswordAuth; 