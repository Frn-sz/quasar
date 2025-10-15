use crate::metrics::{TRANSACTIONS_FAILED_TOTAL, TRANSACTIONS_PROCESSED_TOTAL};

use {
    crate::{
        config::GrpcConfig,
        models::{
            CreateAccountInstruction, DepositInstruction, Transaction, TransactionStatus,
            TransferInstruction,
        },
        transaction_processor::{
            TransactionProcessor,
            interface::{TransactionProcessorInterface, TransactionResult},
        },
    },
    std::{
        convert::TryFrom,
        str::FromStr,
        sync::{Arc, RwLock},
    },
    tonic::{Request, Response, Status, transport::Server},
    tracing::{error, info},
    uuid::Uuid,
};

pub mod server {
    tonic::include_proto!("server");
}

use server::{
    CreateAccountRequest, CreateAccountResponse, DepositRequest, GenericResponse,
    GetBalanceRequest, GetBalanceResponse, TransferRequest,
    grpc_service_server::{GrpcService, GrpcServiceServer},
};

pub struct QuasarGrpcServer {
    processor: Arc<RwLock<TransactionProcessor>>,
}

impl TryFrom<TransferRequest> for Transaction {
    type Error = Status;
    fn try_from(req: TransferRequest) -> Result<Self, Self::Error> {
        Ok(Transaction {
            id: Uuid::parse_str(&req.transaction_id)
                .map_err(|_| Status::invalid_argument("Invalid transaction ID"))?,
            instruction: crate::models::Instruction::Transfer(TransferInstruction {
                source_account_id: Uuid::parse_str(&req.source_account_id)
                    .map_err(|_| Status::invalid_argument("Invalid source account ID"))?,
                destination_account_id: Uuid::parse_str(&req.destination_account_id)
                    .map_err(|_| Status::invalid_argument("Invalid destination account ID"))?,
                amount: req.amount,
            }),
            status: TransactionStatus::Pending,
            timestamp: chrono::Utc::now(),
        })
    }
}

impl TryFrom<CreateAccountRequest> for Transaction {
    type Error = Status;
    fn try_from(req: CreateAccountRequest) -> Result<Self, Self::Error> {
        Ok(Transaction {
            id: Uuid::parse_str(&req.transaction_id)
                .map_err(|_| Status::invalid_argument("Invalid transaction ID"))?,
            instruction: crate::models::Instruction::CreateAccount(CreateAccountInstruction {
                keys: vec![],
            }),
            status: TransactionStatus::Pending,
            timestamp: chrono::Utc::now(),
        })
    }
}

impl TryFrom<DepositRequest> for Transaction {
    type Error = Status;
    fn try_from(req: DepositRequest) -> Result<Self, Self::Error> {
        Ok(Transaction {
            id: Uuid::parse_str(&req.transaction_id)
                .map_err(|_| Status::invalid_argument("Invalid transaction ID"))?,
            instruction: crate::models::Instruction::Deposit(DepositInstruction {
                destination_account_id: Uuid::parse_str(&req.destination_account_id)
                    .map_err(|_| Status::invalid_argument("Invalid destination account ID"))?,
                amount: req.amount,
            }),
            status: TransactionStatus::Pending,
            timestamp: chrono::Utc::now(),
        })
    }
}

impl TryFrom<GetBalanceRequest> for Transaction {
    type Error = Status;
    fn try_from(req: GetBalanceRequest) -> Result<Self, Self::Error> {
        Ok(Transaction {
            id: Uuid::parse_str(&req.transaction_id)
                .map_err(|_| Status::invalid_argument("Invalid transaction ID"))?,
            instruction: crate::models::Instruction::GetBalance(
                crate::models::GetBalanceInstruction {
                    account_id: Uuid::parse_str(&req.account_id)
                        .map_err(|_| Status::invalid_argument("Invalid account ID"))?,
                },
            ),
            status: TransactionStatus::Pending,
            timestamp: chrono::Utc::now(),
        })
    }
}

#[tonic::async_trait]
impl GrpcService for QuasarGrpcServer {
    async fn create_account(
        &self,
        request: Request<CreateAccountRequest>,
    ) -> Result<Response<CreateAccountResponse>, Status> {
        let domain_transaction = request.into_inner().try_into()?;
        let mut processor = self.processor.write().unwrap();

        match processor.process_transaction(domain_transaction) {
            Ok(TransactionResult::AccountCreated(id)) => {
                TRANSACTIONS_PROCESSED_TOTAL.inc();

                info!("Successfully processed create_account request");

                Ok(Response::new(CreateAccountResponse {
                    success: true,
                    created_account_id: id.to_string(),
                    error_message: String::new(),
                }))
            }
            Err(e) => {
                TRANSACTIONS_FAILED_TOTAL.inc();

                error!("Failed to process create_account request: {}", e);

                Ok(Response::new(CreateAccountResponse {
                    success: false,
                    error_message: e.to_string(),
                    ..Default::default()
                }))
            }
            _ => Err(Status::internal("Unexpected processor result")),
        }
    }

    async fn process_transfer(
        &self,
        request: Request<TransferRequest>,
    ) -> Result<Response<GenericResponse>, Status> {
        let domain_transaction = request.into_inner().try_into()?;
        let mut processor = self.processor.write().unwrap();

        match processor.process_transaction(domain_transaction) {
            Ok(TransactionResult::Success) => {
                info!("Successfully processed transfer request");
                Ok(Response::new(GenericResponse {
                    success: true,
                    ..Default::default()
                }))
            }
            Err(e) => Ok(Response::new(GenericResponse {
                success: false,
                error_message: e.to_string(),
            })),
            _ => Err(Status::internal("Unexpected processor result")),
        }
    }

    async fn process_deposit(
        &self,
        request: Request<DepositRequest>,
    ) -> Result<Response<GenericResponse>, Status> {
        let domain_transaction = request.into_inner().try_into()?;
        let mut processor = self.processor.write().unwrap();

        match processor.process_transaction(domain_transaction) {
            Ok(TransactionResult::Success) => {
                info!("Successfully processed deposit request");
                Ok(Response::new(GenericResponse {
                    success: true,
                    ..Default::default()
                }))
            }
            Err(e) => Ok(Response::new(GenericResponse {
                success: false,
                error_message: e.to_string(),
            })),
            _ => Err(Status::internal("Unexpected processor result")),
        }
    }

    async fn get_balance(
        &self,
        request: Request<GetBalanceRequest>,
    ) -> Result<Response<GetBalanceResponse>, Status> {
        let domain_transaction = request.into_inner().try_into()?;
        let mut processor = self.processor.write().unwrap();

        match processor.process_transaction(domain_transaction) {
            Ok(TransactionResult::Balance(amount)) => {
                info!("Successfully processed get_balance request");
                Ok(Response::new(GetBalanceResponse {
                    balance: amount,
                    success: true,
                    ..Default::default()
                }))
            }
            Err(e) => Ok(Response::new(GetBalanceResponse {
                success: false,
                error_message: e.to_string(),
                balance: 0,
            })),
            _ => Err(Status::internal("Unexpected processor result")),
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
            error!("Invalid gRPC address: {}: {}", address, e);
            return;
        }
    };

    let service = QuasarGrpcServer { processor };

    let shutdown = async {
        shutdown_receiver.recv().await.ok();
        info!("gRPC server is shutting down...");
    };

    info!("Initializing gRPC server at {}", address);

    if let Err(e) = Server::builder()
        .add_service(GrpcServiceServer::new(service))
        .serve_with_shutdown(socket_addr, shutdown)
        .await
    {
        error!("Error in gRPC server: {}", e);
    }
}
