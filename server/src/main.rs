use anyhow::{Context, Result};
use app::{AppModule, AppModuleCtx};
use axum::Router;
use clap::Parser;
use client_sdk::{
    helpers::risc0::Risc0Prover,
    rest_client::{IndexerApiHttpClient, NodeApiHttpClient},
};
use conf::Conf;
use contract1::Contract1;
// Contract2 removed - will be replaced with Noir identity verification
use hyle_modules::{
    bus::{metrics::BusMetrics, SharedMessageBus},
    modules::{
        // Temporarily disabled indexer functionality
        contract_state_indexer::{ContractStateIndexer, ContractStateIndexerCtx},
        da_listener::{DAListener, DAListenerConf},
        prover::{AutoProver, AutoProverCtx},
        rest::{RestApi, RestApiRunContext},
        BuildApiContextInner, ModulesHandler,
    },
    utils::logger::setup_tracing,
};
use prometheus::Registry;
use sdk::{api::NodeInfo, info, ZkContract};
use std::sync::{Arc, Mutex};
use tracing::error;

mod app;
mod conf;
mod init;
mod noir_verifier; // New Noir verification module
mod noir_prover;   // New Noir proof generation module

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[arg(long, default_value = "config.toml")]
    pub config_file: Vec<String>,

    #[arg(long, default_value = "contract1")]
    pub contract1_cn: String,

    // Contract2 removed - will use Noir identity verification
    // #[arg(long, default_value = "contract2")]
    // pub contract2_cn: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let config = Conf::new(args.config_file).context("reading config file")?;

    setup_tracing(
        &config.log_format,
        format!("{}(nopkey)", config.id.clone(),),
    )
    .context("setting up tracing")?;

    let config = Arc::new(config);

    info!("Starting app with config: {:?}", &config);

    let node_client =
        Arc::new(NodeApiHttpClient::new(config.node_url.clone()).context("build node client")?);
    let indexer_client = Arc::new(
        IndexerApiHttpClient::new(config.indexer_url.clone()).context("build indexer client")?,
    );

    let contracts = vec![
        init::ContractInit {
            name: args.contract1_cn.clone().into(),
            program_id: contract1::client::tx_executor_handler::metadata::PROGRAM_ID,
            initial_state: Contract1::default().commit(),
        },
        // Contract2 initialization removed - will be replaced with Noir contract
    ];

    match init::init_node(node_client.clone(), indexer_client.clone(), contracts).await {
        Ok(_) => {}
        Err(e) => {
            error!("Error initializing node: {:?}", e);
            return Ok(());
        }
    }
    let bus = SharedMessageBus::new(BusMetrics::global(config.id.clone()));

    std::fs::create_dir_all(&config.data_directory).context("creating data directory")?;

    let mut handler = ModulesHandler::new(&bus).await;

    let api_ctx = Arc::new(BuildApiContextInner {
        router: Mutex::new(Some(Router::new())),
        openapi: Default::default(),
    });

    let app_ctx = Arc::new(AppModuleCtx {
        api: api_ctx.clone(),
        node_client,
        contract1_cn: args.contract1_cn.clone().into(),
        // Contract2 removed - Noir identity will be handled separately
        contract2_cn: "zkpassport_identity".into(), // Placeholder for Noir contract
    });

    handler.build_module::<AppModule>(app_ctx.clone()).await?;

    handler
        .build_module::<ContractStateIndexer<Contract1>>(ContractStateIndexerCtx {
            contract_name: args.contract1_cn.clone().into(),
            data_directory: config.data_directory.clone(),
            api: api_ctx.clone(),
        })
        .await?;

    // Contract2 indexer removed - Noir contracts handled differently
    // handler
    //     .build_module::<ContractStateIndexer<Contract2>>(ContractStateIndexerCtx {
    //         contract_name: args.contract2_cn.clone().into(),
    //         data_directory: config.data_directory.clone(),
    //         api: api_ctx.clone(),
    //     })
    //     .await?;

    handler
        .build_module::<AutoProver<Contract1>>(Arc::new(AutoProverCtx {
            data_directory: config.data_directory.clone(),
            prover: Arc::new(Risc0Prover::new(contracts::CONTRACT1_ELF)),
            contract_name: args.contract1_cn.clone().into(),
            node: app_ctx.node_client.clone(),
            default_state: Default::default(),
            buffer_blocks: config.buffer_blocks,
            max_txs_per_proof: config.max_txs_per_proof,
        }))
        .await?;

    // Contract2 prover removed - Noir proofs handled separately
    // handler
    //     .build_module::<AutoProver<Contract2>>(Arc::new(AutoProverCtx {
    //         data_directory: config.data_directory.clone(),
    //         prover: Arc::new(Risc0Prover::new(contracts::CONTRACT2_ELF)),
    //         contract_name: args.contract2_cn.clone().into(),
    //         node: app_ctx.node_client.clone(),
    //         default_state: Default::default(),
    //         buffer_blocks: config.buffer_blocks,
    //         max_txs_per_proof: config.max_txs_per_proof,
    //     }))
    //     .await?;

    // This module connects to the da_address and receives all the blocks²
    handler
        .build_module::<DAListener>(DAListenerConf {
            start_block: None,
            data_directory: config.data_directory.clone(),
            da_read_from: config.da_read_from.clone(),
        })
        .await?;

    // Should come last so the other modules have nested their own routes.
    #[allow(clippy::expect_used, reason = "Fail on misconfiguration")]
    let router = api_ctx
        .router
        .lock()
        .expect("Context router should be available.")
        .take()
        .expect("Context router should be available.");
    #[allow(clippy::expect_used, reason = "Fail on misconfiguration")]
    let openapi = api_ctx
        .openapi
        .lock()
        .expect("OpenAPI should be available")
        .clone();

    handler
        .build_module::<RestApi>(RestApiRunContext {
            port: config.rest_server_port,
            max_body_size: config.rest_server_max_body_size,
            registry: Registry::new(),
            router,
            openapi,
            info: NodeInfo {
                id: config.id.clone(),
                da_address: config.da_read_from.clone(),
                pubkey: None,
            },
        })
        .await?;

    handler.start_modules().await?;

    // Run until shut down or an error occurs
    handler.exit_process().await?;

    Ok(())
}
