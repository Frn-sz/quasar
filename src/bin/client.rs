use {
    clap::Parser,
    quasar::{
        config::QuasarClientConfig,
        grpc_server::server::{
            CreateAccountRequest, DepositRequest, GetBalanceRequest, TransferRequest,
            grpc_service_client::GrpcServiceClient,
        },
    },
    rand::{Rng, SeedableRng, seq::IndexedRandom},
    std::{sync::Arc, time::Duration},
    tokio::sync::RwLock,
    tonic::transport::Channel,
    tracing::{error, info, warn},
    uuid::Uuid,
};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value = "config.toml")]
    config: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let config = QuasarClientConfig::from_file(&args.config)
        .map_err(|e| format!("Failed to load client configuration file: {}", e))?;

    let _logging_guard = quasar::logging::init_logging(config.debug);

    let account_ids = Arc::new(RwLock::new(Vec::<Uuid>::new()));

    let mut join_handles = Vec::new();
    for i in 0..config.tasks {
        let client = GrpcServiceClient::connect(format!(
            "http://{}:{}",
            config.grpc.address, config.grpc.port
        ))
        .await?;
        let handle = tokio::spawn(run_worker(
            i.try_into().unwrap(),
            client,
            account_ids.clone(),
            config.clone(),
        ));
        join_handles.push(handle);
    }

    info!("Starting load generator with {} tasks...", config.tasks);
    for handle in join_handles {
        if let Err(e) = handle.await {
            error!("One of the worker tasks failed: {}", e);
        };
    }
    Ok(())
}

async fn run_worker(
    worker_id: u32,
    mut client: GrpcServiceClient<Channel>,
    account_ids: Arc<RwLock<Vec<Uuid>>>,
    config: QuasarClientConfig,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut rng = rand::rngs::StdRng::from_os_rng();

    loop {
        let operation_chance = rng.random_range(0..100);

        if operation_chance < config.create_chance {
            let create_req = CreateAccountRequest {
                transaction_id: Uuid::new_v4().to_string(),
            };

            let Ok(creation_response) = client.create_account(create_req.clone()).await else {
                continue;
            };

            let creation = creation_response.into_inner();

            if creation.success {
                let new_id = Uuid::parse_str(&creation.created_account_id)?;
                {
                    account_ids.write().await.push(new_id);
                }
                info!("[Worker {}] Created account: {}", worker_id, new_id);
            }
        } else if operation_chance < config.create_chance + config.deposit_chance {
            let Some(id_to_deposit) = ({ account_ids.read().await.choose(&mut rng).cloned() })
            else {
                continue;
            };

            let amount = rng.random_range(100..500);

            let deposit_req = DepositRequest {
                transaction_id: Uuid::new_v4().to_string(),
                destination_account_id: id_to_deposit.to_string(),
                amount: rng.random_range(100..500),
            };

            if client.process_deposit(deposit_req).await.is_ok() {
                info!(
                    "[Worker {}] Deposited {} into account {}",
                    worker_id, amount, id_to_deposit
                );
            }
        } else {
            let (source_id, dest_id) = {
                let ids_lock = account_ids.read().await;
                if ids_lock.len() < 2 {
                    // Need at least 2 accounts to transfer between
                    continue;
                }
                let sample: Vec<&Uuid> = ids_lock.choose_multiple(&mut rng, 2).collect();
                (*sample[0], *sample[1])
            };

            let get_balance_req = GetBalanceRequest {
                transaction_id: Uuid::new_v4().to_string(),
                account_id: source_id.to_string(),
            };

            let Ok(balance_response) = client.get_balance(get_balance_req).await else {
                continue;
            };

            let balance = balance_response.into_inner().balance;

            if balance == 0 {
                continue;
            }

            let amount_to_transfer = rng.random_range(1..=balance);

            let transfer_req = TransferRequest {
                transaction_id: Uuid::new_v4().to_string(),
                source_account_id: source_id.to_string(),
                destination_account_id: dest_id.to_string(),
                amount: amount_to_transfer,
            };

            match client.process_transfer(transfer_req).await {
                Ok(_) => {
                    info!(
                        "[Worker {}] Transferred {} from {} to {}",
                        worker_id, amount_to_transfer, source_id, dest_id
                    );
                }
                Err(e) => {
                    warn!(
                        "[Worker {}] Transfer of {} from {} to {} failed: {}",
                        worker_id, amount_to_transfer, source_id, dest_id, e
                    );
                }
            }
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
}
