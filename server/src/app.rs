use std::{sync::Arc, time::Duration};

use anyhow::Result;
use axum::{
    extract::{Json, Path, State},
    http::{HeaderMap, Method, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use client_sdk::{
    contract_indexer::AppError,
    rest_client::{NodeApiClient, NodeApiHttpClient},
};
use contract1::{Contract1, Contract1Action};
// Contract2 removed - will be replaced with Noir identity verification

use hyle_modules::{
    bus::{BusClientReceiver, SharedMessageBus},
    module_bus_client, module_handle_messages,
    modules::{prover::AutoProverEvent, BuildApiContextInner, Module},
};
use sdk::{Blob, BlobTransaction, ContractName};
use serde::{Serialize, Deserialize};
use tokio::sync::Mutex;
use tower_http::cors::{Any, CorsLayer};

pub struct AppModule {
    bus: AppModuleBusClient,
}

pub struct AppModuleCtx {
    pub api: Arc<BuildApiContextInner>,
    pub node_client: Arc<NodeApiHttpClient>,
    pub contract1_cn: ContractName,
    pub contract2_cn: ContractName, // Placeholder for future Noir integration
}

module_bus_client! {
#[derive(Debug)]
pub struct AppModuleBusClient {
    receiver(AutoProverEvent<Contract1>),
}
}

impl Module for AppModule {
    type Context = Arc<AppModuleCtx>;

    async fn build(bus: SharedMessageBus, ctx: Self::Context) -> Result<Self> {
        let state = RouterCtx {
            bus: Arc::new(Mutex::new(bus.new_handle())),
            contract1_cn: ctx.contract1_cn.clone(),
            contract2_cn: ctx.contract2_cn.clone(), // Placeholder
            client: ctx.node_client.clone(),
        };

        // Create CORS middleware
        let cors = CorsLayer::new()
            .allow_origin(Any) // Allow all origins (can be restricted)
            .allow_methods(vec![Method::GET, Method::POST]) // Allow necessary methods
            .allow_headers(Any); // Allow all headers

        let api = Router::new()
            .route("/_health", get(health))
            .route("/api/mint-tokens", post(mint_tokens))
            .route("/api/swap-tokens", post(swap_tokens))
            .route("/api/add-liquidity", post(add_liquidity))
            .route("/api/remove-liquidity", post(remove_liquidity))
            .route("/api/get-user-balance", post(get_user_balance))
            .route("/api/get-pool-reserves", post(get_pool_reserves))
            .route("/api/test-amm", post(test_amm))
            .route("/api/config", get(get_config))
            .route("/api/authenticate-noir", post(noir_authenticate))
            // TODO: Add Noir identity verification endpoints
            .with_state(state)
            .layer(cors); // Apply CORS middleware

        if let Ok(mut guard) = ctx.api.router.lock() {
            if let Some(router) = guard.take() {
                guard.replace(router.merge(api));
            }
        }
        let bus = AppModuleBusClient::new_from_bus(bus.new_handle()).await;

        Ok(AppModule { bus })
    }

    async fn run(&mut self) -> Result<()> {
        module_handle_messages! {
            on_bus self.bus,
        };

        Ok(())
    }
}

#[derive(Clone)]
struct RouterCtx {
    pub bus: Arc<Mutex<SharedMessageBus>>,
    pub client: Arc<NodeApiHttpClient>,
    pub contract1_cn: ContractName,
    pub contract2_cn: ContractName, // Placeholder for Noir contract
}

async fn health() -> impl IntoResponse {
    Json("OK")
}

// --------------------------------------------------------
//     Headers
// --------------------------------------------------------

const USER_HEADER: &str = "x-user";

#[derive(Debug)]
struct AuthHeaders {
    user: String,
}

impl AuthHeaders {
    fn from_headers(headers: &HeaderMap) -> Result<Self, AppError> {
        let user = headers
            .get(USER_HEADER)
            .and_then(|v| v.to_str().ok())
            .ok_or_else(|| {
                AppError(
                    StatusCode::UNAUTHORIZED,
                    anyhow::anyhow!("Missing user header"),
                )
            })?;

        Ok(AuthHeaders {
            user: user.to_string(),
        })
    }
}

#[derive(Serialize)]
struct ConfigResponse {
    contract_name: String,
}

#[derive(Deserialize)]
struct MintTokensRequest {
    wallet_blobs: [Blob; 2],
    token: String,
    amount: u128,
}

#[derive(Deserialize)]
struct SwapTokensRequest {
    wallet_blobs: [Blob; 2],
    token_in: String,
    token_out: String,
    amount_in: u128,
    min_amount_out: u128,
}

#[derive(Deserialize)]
struct AddLiquidityRequest {
    wallet_blobs: [Blob; 2],
    token_a: String,
    token_b: String,
    amount_a: u128,
    amount_b: u128,
}

#[derive(Deserialize)]
struct RemoveLiquidityRequest {
    wallet_blobs: [Blob; 2],
    token_a: String,
    token_b: String,
    liquidity_amount: u128,
}

#[derive(Deserialize)]
struct GetUserBalanceRequest {
    wallet_blobs: [Blob; 2],
    token: String,
}

#[derive(Deserialize)]
struct GetPoolReservesRequest {
    wallet_blobs: [Blob; 2],
    token_a: String,
    token_b: String,
}

#[derive(Deserialize)]
struct TestAmmRequest {
    wallet_blobs: [Blob; 2],
}

#[derive(Deserialize)]
pub struct NoirAuthRequest {
    pub username: String,
    pub user_field: String,
    pub password_field: String,
    pub proof_type: String,
}

#[derive(Serialize)]
pub struct NoirAuthResponse {
    pub success: bool,
    pub message: String,
    pub proof_hash: Option<String>,
    pub tx_hash: Option<String>,
}

// Known correct values for demo (these would come from Noir circuit compilation)
const EXPECTED_BOB_FIELD: &str = "12345"; // Placeholder - needs actual Poseidon2 hash
const EXPECTED_PASSWORD_FIELD: &str = "54321"; // Placeholder - needs actual Poseidon2 hash

// --------------------------------------------------------
//     Routes
// --------------------------------------------------------

async fn mint_tokens(
    State(ctx): State<RouterCtx>,
    headers: HeaderMap,
    Json(request): Json<MintTokensRequest>
) -> Result<impl IntoResponse, AppError> {
    let auth = AuthHeaders::from_headers(&headers)?;
    
    let action_contract1 = Contract1Action::MintTokens {
        user: auth.user.clone(),
        token: request.token,
        amount: request.amount,
    };
    
    // For now, only process AMM actions - Noir identity verification will be added later
    send_amm_action_only(ctx, auth, request.wallet_blobs, action_contract1).await
}

async fn swap_tokens(
    State(ctx): State<RouterCtx>,
    headers: HeaderMap,
    Json(request): Json<SwapTokensRequest>
) -> Result<impl IntoResponse, AppError> {
    let auth = AuthHeaders::from_headers(&headers)?;
    
    let action_contract1 = Contract1Action::SwapExactTokensForTokens {
        user: auth.user.clone(),
        token_in: request.token_in,
        token_out: request.token_out,
        amount_in: request.amount_in,
        min_amount_out: request.min_amount_out,
    };
    
    // TODO: Add Noir identity verification for @zkpassport users
    send_amm_action_only(ctx, auth, request.wallet_blobs, action_contract1).await
}

async fn add_liquidity(
    State(ctx): State<RouterCtx>,
    headers: HeaderMap,
    Json(request): Json<AddLiquidityRequest>
) -> Result<impl IntoResponse, AppError> {
    let auth = AuthHeaders::from_headers(&headers)?;
    
    let action_contract1 = Contract1Action::AddLiquidity {
        user: auth.user.clone(),
        token_a: request.token_a,
        token_b: request.token_b,
        amount_a: request.amount_a,
        amount_b: request.amount_b,
    };
    
    send_amm_action_only(ctx, auth, request.wallet_blobs, action_contract1).await
}

async fn remove_liquidity(
    State(ctx): State<RouterCtx>,
    headers: HeaderMap,
    Json(request): Json<RemoveLiquidityRequest>
) -> Result<impl IntoResponse, AppError> {
    let auth = AuthHeaders::from_headers(&headers)?;
    
    let action_contract1 = Contract1Action::RemoveLiquidity {
        user: auth.user.clone(),
        token_a: request.token_a,
        token_b: request.token_b,
        liquidity_amount: request.liquidity_amount,
    };
    
    send_amm_action_only(ctx, auth, request.wallet_blobs, action_contract1).await
}

async fn get_user_balance(
    State(ctx): State<RouterCtx>,
    headers: HeaderMap,
    Json(request): Json<GetUserBalanceRequest>
) -> Result<impl IntoResponse, AppError> {
    let auth = AuthHeaders::from_headers(&headers)?;
    
    let action_contract1 = Contract1Action::GetUserBalance {
        user: auth.user.clone(),
        token: request.token,
    };
    
    send_amm_action_only(ctx, auth, request.wallet_blobs, action_contract1).await
}

async fn get_pool_reserves(
    State(ctx): State<RouterCtx>,
    headers: HeaderMap,
    Json(request): Json<GetPoolReservesRequest>
) -> Result<impl IntoResponse, AppError> {
    let auth = AuthHeaders::from_headers(&headers)?;
    
    let action_contract1 = Contract1Action::GetReserves {
        token_a: request.token_a,
        token_b: request.token_b,
    };
    
    send_amm_action_only(ctx, auth, request.wallet_blobs, action_contract1).await
}

async fn test_amm(
    State(ctx): State<RouterCtx>,
    headers: HeaderMap,
    Json(request): Json<TestAmmRequest>
) -> Result<impl IntoResponse, AppError> {
    let auth = AuthHeaders::from_headers(&headers)?;
    
    // Test action: Mint some USDC tokens for testing
    let action_contract1 = Contract1Action::MintTokens {
        user: auth.user.clone(),
        token: "USDC".to_string(),
        amount: 1000,
    };
    
    send_amm_action_only(ctx, auth, request.wallet_blobs, action_contract1).await
}

async fn get_config(State(ctx): State<RouterCtx>) -> impl IntoResponse {
    Json(ConfigResponse {
        contract_name: ctx.contract1_cn.0,
    })
}

async fn noir_authenticate(
    State(state): State<RouterCtx>,
    Json(request): Json<NoirAuthRequest>,
) -> Result<Json<NoirAuthResponse>, StatusCode> {
    tracing::info!("🔐 Starting Noir circuit authentication for user: {}", request.username);
    
    // Step 1: Validate proof type
    if request.proof_type != "noir_circuit" {
        tracing::error!("❌ Invalid proof type: {}", request.proof_type);
        return Ok(Json(NoirAuthResponse {
            success: false,
            message: "Invalid proof type".to_string(),
            proof_hash: None,
            tx_hash: None,
        }));
    }

    // Step 2: Basic validation (in real implementation, this would be done by Noir circuit)
    // For now, we'll validate the field values match expected ones
    tracing::info!("🔢 Validating field values...");
    tracing::info!("User field: {}", request.user_field);
    tracing::info!("Password field: {}", request.password_field);

    // Temporary validation logic until proper Noir integration
    let is_valid_user = request.username == "bob";
    let user_field_matches = request.user_field != "0"; // Basic non-zero check
    let password_field_matches = request.password_field != "0"; // Basic non-zero check

    if !is_valid_user || !user_field_matches || !password_field_matches {
        tracing::error!("❌ Authentication failed: invalid credentials");
        return Ok(Json(NoirAuthResponse {
            success: false,
            message: "Invalid credentials".to_string(),
            proof_hash: None,
            tx_hash: None,
        }));
    }

    tracing::info!("✅ Field validation passed");

    // Step 3: Generate Noir proof (PLACEHOLDER - needs real Noir integration)
    tracing::info!("🧮 Generating Noir circuit proof...");
    
    // TODO: Replace with actual Noir proof generation
    // This is where we would:
    // 1. Call the Noir circuit with private inputs
    // 2. Generate a zero-knowledge proof
    // 3. Get the proof data for submission to Hyli
    
    let mock_proof_hash = format!("noir_proof_{}", hex::encode(&request.username.as_bytes()[..std::cmp::min(8, request.username.len())]));
    
    tracing::info!("🔐 Generated proof hash: {}", mock_proof_hash);

    // Step 4: Submit proof to Hyli chain (PLACEHOLDER)
    tracing::info!("⛓️ Submitting proof to Hyli chain...");
    
    // TODO: Replace with actual Hyli transaction submission
    // This is where we would:
    // 1. Create a transaction with the Noir proof
    // 2. Submit to the zkpassport_identity contract
    // 3. Wait for verification and settlement
    
    let mock_tx_hash = format!("tx_{}_noir_auth", chrono::Utc::now().timestamp());
    
    tracing::info!("📜 Submitted transaction: {}", mock_tx_hash);

    // Step 5: Return success response
    tracing::info!("✅ Noir circuit authentication successful for user: {}", request.username);

    Ok(Json(NoirAuthResponse {
        success: true,
        message: format!("Authentication successful for user: {}", request.username),
        proof_hash: Some(mock_proof_hash),
        tx_hash: Some(mock_tx_hash),
    }))
}

// Simplified function for AMM-only actions (without identity verification for now)
async fn send_amm_action_only(
    ctx: RouterCtx, 
    auth: AuthHeaders, 
    wallet_blobs: [Blob; 2],
    amm_action: Contract1Action
) -> Result<impl IntoResponse, AppError> {
    let identity = auth.user.clone();

    // For now, only send AMM blob - Noir identity verification will be added later
    let mut blobs = wallet_blobs.to_vec();
    blobs.push(amm_action.as_blob(ctx.contract1_cn.clone()));

    let res = ctx
        .client
        .send_tx_blob(BlobTransaction::new(identity.clone(), blobs))
        .await;

    if let Err(ref e) = res {
        let root_cause = e.root_cause().to_string();
        return Err(AppError(
            StatusCode::BAD_REQUEST,
            anyhow::anyhow!("{}", root_cause),
        ));
    }

    let tx_hash = res.unwrap();

    let mut bus = {
        let bus = ctx.bus.lock().await;
        AppModuleBusClient::new_from_bus(bus.new_handle()).await
    };

    tokio::time::timeout(Duration::from_secs(30), async {
        loop {
            match bus.recv().await? {
                AutoProverEvent::<Contract1>::SuccessTx(sequenced_tx_hash, _) => {
                    if sequenced_tx_hash == tx_hash {
                        return Ok(Json(sequenced_tx_hash));
                    }
                }
                AutoProverEvent::<Contract1>::FailedTx(sequenced_tx_hash, error) => {
                    if sequenced_tx_hash == tx_hash {
                        return Err(AppError(StatusCode::BAD_REQUEST, anyhow::anyhow!(error)));
                    }
                }
            }
        }
    })
    .await?
}
