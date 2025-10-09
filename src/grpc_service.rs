use std::{
    net::SocketAddrV4,
    str::FromStr,
    sync::{Arc, RwLock},
};
use tonic::{Request, Response, Status, transport::Server};
use tracing::{error, info};

use crate::{
    config::GrpcConfig,
    models::{CreateAccountInstruction, Instruction, Transaction, TransferInstruction},
    transaction_processor::{TransactionProcessor, interface::TransactionProcessorInterface},
};
use uuid::Uuid;

pub mod quasar {
    tonic::include_proto!("quasar");
}

use quasar::{
    TransactionRequest, TransactionResponse,
    quasar_service_server::{QuasarService, QuasarServiceServer},
    transaction_request,
};

pub struct GrpcService {
    processor: Arc<RwLock<TransactionProcessor>>,
}

#[tonic::async_trait]
impl QuasarService for GrpcService {
    async fn process_transaction(
        &self,
        request: Request<TransactionRequest>,
    ) -> Result<Response<TransactionResponse>, Status> {
        let req = request.into_inner();

        let transaction_id = Uuid::parse_str(&req.id)
            .map_err(|_| Status::invalid_argument("Invalid transaction ID format"))?;

        let instruction = req
            .instruction
            .ok_or_else(|| Status::invalid_argument("Instruction is required"))?;

        let domain_transaction = Transaction {
            id: transaction_id,
            instruction: match instruction {
                transaction_request::Instruction::Transfer(t) => {
                    let from = Uuid::parse_str(&t.source_account_id)
                        .map_err(|_| Status::invalid_argument("Invalid source account ID"))?;
                    let to = Uuid::parse_str(&t.destination_account_id)
                        .map_err(|_| Status::invalid_argument("Invalid destination account ID"))?;

                    Instruction::Transfer(TransferInstruction {
                        source_account_id: from,
                        destination_account_id: to,
                        amount: t.amount,
                    })
                }
                transaction_request::Instruction::CreateAccount(ca) => {
                    // Keys must be added later; for now, we create an empty account.
                    Instruction::CreateAccount(CreateAccountInstruction { keys: vec![] })
                }
            },
            status: crate::models::TransactionStatus::Pending,
            timestamp: chrono::Utc::now(),
        };

        let mut processor_lock = self.processor.write().unwrap();

        match processor_lock.process_transaction(domain_transaction) {
            Ok(_) => Ok(Response::new(TransactionResponse {
                success: true,
                error_message: String::new(),
            })),
            Err(e) => Ok(Response::new(TransactionResponse {
                success: false,
                error_message: e.to_string(),
            })),
        }
    }
}

pub async fn start_grpc_service(
    config: GrpcConfig,
    processor: Arc<RwLock<TransactionProcessor>>,
    mut shutdown_receiver: tokio::sync::broadcast::Receiver<()>,
) {
    let address = format!("{}:{}", config.address, config.port);

    let socket_addr = match std::net::SocketAddr::from_str(&address) {
        Ok(addr) => addr,
        Err(e) => {
            error!("Invalid gRPC address: {}: {}", config.address, e);
            return;
        }
    };

    let shutdown = async {
        shutdown_receiver.recv().await.ok();
        info!("gRPC server is shutting down...");
    };

    info!("Initializing gRPC server at {}", address);

    if let Err(e) = Server::builder()
        .add_service(QuasarServiceServer::new(GrpcService { processor }))
        .serve_with_shutdown(socket_addr, shutdown)
        .await
    {
        error!("gRPC server error: {}", e);
    }
}
