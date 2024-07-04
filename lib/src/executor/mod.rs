use crate::client::AzeClient;
use miden_client::{
    client::transactions::transaction_request::TransactionRequest,
    store::TransactionFilter,
    errors::ClientError,
};
use miden_objects::transaction::TransactionId;
use miden_tx::{ TransactionExecutorError, TransactionCompilerError };
use std::time::Duration;

pub async fn execute_tx_and_sync(client: &mut AzeClient, tx_request: TransactionRequest) {
    
    sync_state_with_retry(client).await;
    let transaction_id = submit_transaction_with_retry(client, tx_request.clone()).await;

    // wait until tx is committed
    loop {
        sync_state_with_retry(client).await;

        // Check if executed transaction got committed by the node
        let uncommited_transactions = client
            .get_transactions(TransactionFilter::Uncomitted)
            .unwrap();
        let is_tx_committed = uncommited_transactions
            .iter()
            .find(|uncommited_tx| uncommited_tx.id == transaction_id)
            .is_none();

        if is_tx_committed {
            break;
        }

        std::thread::sleep(Duration::new(3, 0));
    }
}

async fn sync_state_with_retry(client: &mut AzeClient) {
    for _try_number in 0..20 {
        match client.sync_state().await {
            Err(ClientError::NodeRpcClientError(_)) => {
                std::thread::sleep(Duration::from_secs(2));
            },
            Err(other_error) => {
                panic!("Unexpected error: {other_error}");
            },
            _ => return,
        }
    }
    panic!("Failed to sync state");
}

async fn submit_transaction_with_retry(client: &mut AzeClient, tx_request: TransactionRequest) -> TransactionId {
    for _try_number in 0..20 {
        let transaction_execution_result = match client.new_transaction(tx_request.clone()) {
            Ok(result) => result,
            Err(e) => {
                match e {
                    ClientError::TransactionExecutorError(TransactionExecutorError::CompileTransactionFailed(TransactionCompilerError::NoteIncompatibleWithAccountInterface(_))) => {
                        // Suppress this specific error
                        break;
                    }
                    _ => {
                        // Handle other errors
                        println!("Error creating transaction: {:?}", e);
                        break;
                    }
                };
            }
        };
        let transaction_id = transaction_execution_result.executed_transaction().id();

        match client.submit_transaction(transaction_execution_result).await {
            Err(ClientError::NodeRpcClientError(_)) => {
                std::thread::sleep(Duration::from_secs(2));
            },
            Err(other_error) => {
                panic!("Unexpected error: {other_error}");
            },
            _ => return transaction_id,
        }
    }
    panic!("Failed to submit transaction");
}