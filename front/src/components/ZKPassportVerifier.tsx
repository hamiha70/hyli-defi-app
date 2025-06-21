import React, { useState, useRef, useEffect } from 'react';
import { ZKPassport } from '@zkpassport/sdk';
import qrcode from 'qrcode';

interface VerificationResult {
  uniqueIdentifier: string;
  verified: boolean;
  ageProof: any;
  proofData: string;
}

interface ZKPassportVerifierProps {
  onVerificationComplete: (result: VerificationResult) => void;
  onError: (error: string) => void;
}

type VerificationState = 
  | 'idle' 
  | 'generating-qr' 
  | 'waiting-scan' 
  | 'request-received' 
  | 'generating-proof' 
  | 'verifying' 
  | 'completed' 
  | 'error';

export const ZKPassportVerifier: React.FC<ZKPassportVerifierProps> = ({
  onVerificationComplete,
  onError
}) => {
  const [state, setState] = useState<VerificationState>('idle');
  const [errorMessage, setErrorMessage] = useState<string>('');
  const [currentStep, setCurrentStep] = useState<string>('');
  const [qrUrl, setQrUrl] = useState<string>('');
  const [proofCount, setProofCount] = useState<number>(0);
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const zkPassportRef = useRef<ZKPassport | null>(null);

  // Initialize ZKPassport SDK
  useEffect(() => {
    try {
      // Auto-detect domain in browser environment
      zkPassportRef.current = new ZKPassport();
    } catch (error) {
      console.error('Failed to initialize ZKPassport SDK:', error);
      setErrorMessage('Failed to initialize ZKPassport SDK');
      setState('error');
    }
  }, []);

  // Generate QR code when we have URL and canvas is ready
  useEffect(() => {
    if (state === 'waiting-scan' && qrUrl && canvasRef.current) {
      console.log('🔄 useEffect: Generating QR code for URL:', qrUrl);
      qrcode.toCanvas(canvasRef.current, qrUrl, {
        width: 300,
        margin: 2,
        color: {
          dark: '#000000',
          light: '#FFFFFF',
        },
      }).then(() => {
        console.log('✅ QR code generated successfully via useEffect');
      }).catch((qrError) => {
        console.error('❌ QR code generation failed via useEffect:', qrError);
        setState('error');
        setErrorMessage('Failed to generate QR code');
      });
    }
  }, [state, qrUrl, canvasRef.current]);

  const startVerification = async () => {
    if (!zkPassportRef.current) {
      onError('ZKPassport SDK not initialized');
      return;
    }

    try {
      setState('generating-qr');
      setCurrentStep('Creating verification request...');
      setProofCount(0); // Reset proof counter
      console.log('🔄 Starting ZKPassport verification...');

      // Add timeout to prevent hanging
      const timeoutPromise = new Promise((_, reject) => {
        setTimeout(() => reject(new Error('ZKPassport request timed out after 30 seconds')), 30000);
      });

      // Create verification request following the SDK pattern
      console.log('🔄 Creating request with ZKPassport SDK...');
      const requestPromise = zkPassportRef.current.request({
        name: "Hyli DeFi AMM",
        logo: "data:image/svg+xml;base64,PHN2ZyB3aWR0aD0iNDAiIGhlaWdodD0iNDAiIHZpZXdCb3g9IjAgMCA0MCA0MCIgZmlsbD0ibm9uZSIgeG1sbnM9Imh0dHA6Ly93d3cudzMub3JnLzIwMDAvc3ZnIj4KPGNpcmNsZSBjeD0iMjAiIGN5PSIyMCIgcj0iMjAiIGZpbGw9IiM2NjdlZWEiLz4KPHN2ZyB4PSI4IiB5PSI4IiB3aWR0aD0iMjQiIGhlaWdodD0iMjQiIGZpbGw9IndoaXRlIj4KPHRleHQgZm9udC1mYW1pbHk9IkFyaWFsIiBmb250LXNpemU9IjEyIiBmaWxsPSJ3aGl0ZSIgeD0iNiIgeT0iMTYiPkgvRjwvdGV4dD4KPHN2Zz4K", // Simple SVG logo
        purpose: "Prove you are younger than 25 for DeFi trading compliance",
        scope: "hyli-amm-age-verification",
        devMode: true, // Enable dev mode for testing with mock proofs
      });
      
      const queryBuilder = await Promise.race([requestPromise, timeoutPromise]) as any;
      console.log('✅ Query builder created:', queryBuilder);

              setCurrentStep('Building age verification query...');
              console.log('🔄 Building age query with .gte("age", 25) - checking if 25 or older...');

        // Build the query - prove age >= 25, then we expect result to be FALSE (not 25+)
        // This maintains privacy without disclosing actual age
        const queryResult = queryBuilder.gte("age", 25);
              console.log('✅ Query result after .gte():', queryResult);
        
        console.log('🔄 Calling .done() to finalize query...');
      const {
        url,
        requestId,
        onRequestReceived,
        onGeneratingProof,
        onProofGenerated,
        onResult,
        onReject,
        onError: onVerificationError,
      } = queryResult.done();
      console.log('✅ Query finalized. URL:', url, 'RequestID:', requestId);

      // IMPORTANT: Update UI state after getting the URL
      console.log('🔄 Updating UI state to show QR code...');
      setQrUrl(url); // Store URL for QR generation
      setState('waiting-scan');
      setCurrentStep('Scan the QR code with ZKPassport mobile app');

      // Set up event handlers
      onRequestReceived(() => {
        console.log('✅ Request received by user');
        setState('request-received');
        setCurrentStep('Request received! User is reviewing...');
      });

      onGeneratingProof(() => {
        console.log('🔄 Generating proof... Started at:', new Date().toLocaleTimeString());
        setState('generating-proof');
        setCurrentStep('Generating cryptographic proof (this may take up to 10 seconds)...');
        
        // Add timeout for proof generation (2 minutes max)
        setTimeout(() => {
          if (state === 'generating-proof') {
            console.log('⏰ Proof generation timeout - taking too long after 2 minutes');
            setState('error');
            setErrorMessage('Proof generation is taking too long. Please try again.');
            onError('Proof generation timeout');
          }
        }, 120000); // 2 minutes timeout
      });

      onProofGenerated(({ vkeyHash, version, name }: any) => {
        const newCount = proofCount + 1;
        setProofCount(newCount);
        console.log(`🔐 Individual proof generated (${newCount}):`, { name, vkeyHash, version });
        setCurrentStep(`Generated ${name} proof... (${newCount} proofs completed)`);
        
        // Add debug info for age-related proofs
        if (name && name.toLowerCase().includes('age')) {
          console.log('🎯 AGE PROOF DETECTED:', name);
        }
        if (name && name.toLowerCase().includes('compare')) {
          console.log('🎯 COMPARE PROOF DETECTED:', name);
        }
      });

      onResult(({ uniqueIdentifier, verified, result }: any) => {
        console.log('🎯🎯🎯 FINAL VERIFICATION RESULT RECEIVED! 🎯🎯🎯');
        console.log('🔍 Full result object:', { uniqueIdentifier, verified, result });
        console.log('🔍 Result structure:', JSON.stringify(result, null, 2));
        
        if (verified && result.age?.gte?.result !== undefined) {
          const isOver25 = result.age.gte.result;
          console.log('🎯 Age >= 25 check result:', isOver25);
          
          if (!isOver25) {
            // Result is FALSE, meaning they're NOT 25 or older (i.e., younger than 25)
            setState('completed');
            setCurrentStep('✅ Verification successful!');
            
            // Package the result for the parent component
            const verificationResult: VerificationResult = {
              uniqueIdentifier: uniqueIdentifier || 'unknown',
              verified: true,
              ageProof: { 
                result: true, 
                threshold: 25, 
                actualAge: null, // Privacy-preserving - don't reveal actual age
                queryType: 'age-less-than-25' 
              },
              proofData: JSON.stringify({
                result,
                uniqueIdentifier: uniqueIdentifier || 'unknown',
                verified,
                ageThreshold: 25,
                isYoungerThan25: true,
                timestamp: Date.now()
              })
            };
            
            onVerificationComplete(verificationResult);
          } else {
            console.error('❌ Age verification failed - user is 25 or older');
            setState('error');
            setErrorMessage(`Age verification failed. You must be younger than 25 years old.`);
            onError(`Verification failed: User is 25 years old or older`);
          }
        } else {
          console.error('❌ Verification failed - no age proof returned:', result);
          setState('error');
          setErrorMessage('Age verification failed. Age could not be verified from your document.');
          onError('Verification failed: Age proof incomplete or invalid');
        }
      });

      onReject(() => {
        console.log('❌ User rejected the verification request');
        setState('error');
        setErrorMessage('Verification was rejected by user');
        onError('User rejected the verification request');
      });

      onVerificationError((error: any) => {
        console.error('❌ Verification error:', error);
        setState('error');
        const errorMsg = String(error);
        setErrorMessage(`Verification error: ${errorMsg}`);
        onError(`Verification error: ${errorMsg}`);
      });

      // QR code will be generated by useEffect when canvas is ready

    } catch (error) {
      console.error('Failed to start verification:', error);
      setState('error');
      setErrorMessage(`Failed to start verification: ${error}`);
      onError(`Failed to start verification: ${error}`);
    }
  };



  const resetVerification = () => {
    setState('idle');
    setErrorMessage('');
    setCurrentStep('');
    setQrUrl('');
    setProofCount(0);
    if (canvasRef.current) {
      const ctx = canvasRef.current.getContext('2d');
      if (ctx) {
        ctx.clearRect(0, 0, canvasRef.current.width, canvasRef.current.height);
      }
    }
  };

  const getStepDescription = () => {
    switch (state) {
      case 'idle':
        return 'Ready to verify your age for DeFi compliance';
      case 'generating-qr':
        return 'Generating secure verification request...';
      case 'waiting-scan':
        return 'Waiting for you to scan the QR code with ZKPassport app';
      case 'request-received':
        return 'Request received! Check your phone to continue';
      case 'generating-proof':
        return 'Generating zero-knowledge proof of your age...';
      case 'verifying':
        return 'Verifying your proof...';
      case 'completed':
        return '✅ Verification completed successfully!';
      case 'error':
        return '❌ Verification failed';
      default:
        return currentStep;
    }
  };

  return (
    <div className="zkpassport-verifier">
      <div className="verification-container">
        <h2>🛂 Age Verification</h2>
        <p className="verification-description">
          To comply with regulations, please verify that you are younger than 25 using ZKPassport.
          This process is privacy-preserving - we only verify your age status, not your exact age.
        </p>

        <div className="verification-status">
          <div className={`status-indicator ${state}`}>
            <div className="status-dot"></div>
            <span>{getStepDescription()}</span>
          </div>
          {currentStep && state !== 'idle' && (
            <div className="current-step">{currentStep}</div>
          )}
        </div>

        {state === 'idle' && (
          <div className="start-verification">
            <button 
              onClick={startVerification}
              className="start-button"
            >
              🚀 Start Age Verification
            </button>
            <button 
              onClick={() => {
                console.log('🧪 Using test mode - simulating successful verification');
                const mockResult = {
                  uniqueIdentifier: 'test-user-' + Date.now(),
                  verified: true,
                  ageProof: { result: true, threshold: 25, actualAge: 22, queryType: 'age-less-than' },
                  proofData: JSON.stringify({ test: true, age: 22, threshold: 25, timestamp: Date.now() })
                };
                onVerificationComplete(mockResult);
              }}
              className="start-button"
              style={{ background: 'linear-gradient(135deg, #10b981, #059669)', marginTop: '1rem' }}
            >
              ⚡ Skip Verification (Test Mode)
            </button>
            <div className="requirements">
              <h4>ZKPassport Requirements:</h4>
              <ul>
                <li>📱 ZKPassport mobile app installed</li>
                <li>🛂 Valid passport or national ID</li>
                <li>🎂 Must be younger than 25 years old</li>
              </ul>
            </div>
          </div>
        )}

        {(state === 'waiting-scan' || state === 'request-received' || state === 'generating-proof') && (
          <div className="qr-section">
            <canvas 
              ref={canvasRef} 
              className="qr-canvas"
              style={{ border: '2px solid #e0e0e0', borderRadius: '8px' }}
            />
            <div className="qr-instructions">
              <p><strong>Instructions:</strong></p>
              <ol>
                <li>Open ZKPassport app on your mobile device</li>
                <li>Tap "Scan QR Code" or use the camera</li>
                <li>Scan the QR code above</li>
                <li>Follow the prompts to generate your proof</li>
              </ol>
              {state === 'generating-proof' && (
                <div className="proof-progress">
                  <div className="loading-spinner"></div>
                  <p>⏳ Generating proof... This may take up to 10 seconds</p>
                  <p>📊 Proofs completed: {proofCount}</p>
                  {proofCount >= 3 && (
                    <div style={{ marginTop: '1rem', padding: '1rem', background: 'rgba(255,255,255,0.1)', borderRadius: '8px' }}>
                      <p><strong>🔍 Debug Info:</strong> {proofCount} proofs generated. If this is taking too long, check the console for any errors.</p>
                    </div>
                  )}
                </div>
              )}
            </div>
          </div>
        )}

        {state === 'completed' && (
          <div className="verification-success">
            <div className="success-icon">✅</div>
            <h3>Age Verification Successful!</h3>
            <p>You have successfully proven that you are younger than 25.</p>
            <p>You may now proceed with trading on the AMM.</p>
          </div>
        )}

        {state === 'error' && (
          <div className="verification-error">
            <div className="error-icon">❌</div>
            <h3>Age Verification Failed</h3>
            <p className="error-message">{errorMessage}</p>
            <button 
              onClick={resetVerification}
              className="retry-button"
            >
              🔄 Try Again
            </button>
          </div>
        )}

        {state !== 'idle' && state !== 'completed' && state !== 'error' && (
          <div className="verification-actions">
            <button 
              onClick={resetVerification}
              className="cancel-button"
            >
              Cancel Verification
            </button>
          </div>
        )}
      </div>
    </div>
  );
}; 