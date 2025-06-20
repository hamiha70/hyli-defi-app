use std::{sync::Arc, time::Duration};

use anyhow::Result;
use axum::{
    extract::{Json, State},
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
use contract2::Contract2Action;

use hyle_modules::{
    bus::{BusClientReceiver, SharedMessageBus},
    module_bus_client, module_handle_messages,
    modules::{prover::AutoProverEvent, BuildApiContextInner, Module},
};
use sdk::{Blob, BlobTransaction, ContractName};
use serde::Serialize;
use tokio::sync::Mutex;
use tower_http::cors::{Any, CorsLayer};

pub struct AppModule {
    bus: AppModuleBusClient,
}

pub struct AppModuleCtx {
    pub api: Arc<BuildApiContextInner>,
    pub node_client: Arc<NodeApiHttpClient>,
    pub contract1_cn: ContractName,
    pub contract2_cn: ContractName,
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
            contract2_cn: ctx.contract2_cn.clone(),
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
            .route("/api/test-amm", post(test_amm))
            .route("/api/config", get(get_config))
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
    pub contract2_cn: ContractName,
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

#[derive(serde::Deserialize)]
struct MintTokensRequest {
    wallet_blobs: [Blob; 2],
    token: String,
    amount: u128,
}

#[derive(serde::Deserialize)]
struct TestAmmRequest {
    wallet_blobs: [Blob; 2],
}

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
    
    send_amm_action(ctx, auth, request.wallet_blobs, action_contract1).await
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
    
    send_amm_action(ctx, auth, request.wallet_blobs, action_contract1).await
}

async fn get_config(State(ctx): State<RouterCtx>) -> impl IntoResponse {
    Json(ConfigResponse {
        contract_name: ctx.contract1_cn.0,
    })
}

async fn send_amm_action(
    ctx: RouterCtx, 
    auth: AuthHeaders, 
    wallet_blobs: [Blob; 2],
    amm_action: Contract1Action
) -> Result<impl IntoResponse, AppError> {
    let identity = auth.user.clone();

    // For now, keeping contract2 simple (could be identity verification later)
    let action_contract2 = Contract2Action::Increment;

    let mut blobs = wallet_blobs.to_vec();

    blobs.extend(vec![
        amm_action.as_blob(ctx.contract1_cn.clone()),
        action_contract2.as_blob(ctx.contract2_cn.clone()),
    ]);

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

    tokio::time::timeout(Duration::from_secs(5), async {
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
