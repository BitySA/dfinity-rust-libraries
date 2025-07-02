use crate::state::{icrc3_commit_prepared_transaction, icrc3_prepare_transaction};
use crate::utils::trace;

use ic_cdk_macros::update;
pub use icrc3_example_api::updates::add_transactions_with_async::{
    Args as AddTransactionsWithAsyncArgs, Response as AddTransactionsWithAsyncResponse,
};

#[update]
async fn add_transactions_with_async(
    transaction: AddTransactionsWithAsyncArgs,
) -> AddTransactionsWithAsyncResponse {
    trace(&format!("add_transactions_with_async: starting"));

    // Step 1: Prepare the transaction
    let prepared_tx = match icrc3_prepare_transaction(transaction.clone()) {
        Ok(prepared) => {
            trace(&format!(
                "add_transactions_with_async: transaction prepared successfully"
            ));
            prepared
        }
        Err(e) => {
            trace(&format!(
                "add_transactions_with_async: error preparing transaction: {}",
                e
            ));
            return Err(format!("Error preparing transaction: {}", e));
        }
    };

    // Step 2: Simulate async operation (e.g., external API call, database operation, etc.)
    trace(&format!(
        "add_transactions_with_async: performing async operation"
    ));

    // Simulate some async work
    perform_async_operation().await.unwrap();

    // Step 3: Commit the prepared transaction
    let commit_result = icrc3_commit_prepared_transaction(transaction, prepared_tx.timestamp);

    match commit_result {
        Ok(tx_index) => {
            trace(&format!(
                "add_transactions_with_async: transaction committed successfully with index: {}",
                tx_index
            ));
            Ok(tx_index)
        }
        Err(e) => {
            trace(&format!(
                "add_transactions_with_async: error committing transaction: {}",
                e
            ));
            Err(format!("Error committing transaction: {}", e))
        }
    }
}

async fn perform_async_operation() -> Result<(), String> {
    let _ = ic_cdk::call::Call::unbounded_wait(
        ic_cdk::api::canister_self(), // Call self (could be any canister)
        "create_transactions",
    )
    .await
    .unwrap();

    return Ok(());
}
