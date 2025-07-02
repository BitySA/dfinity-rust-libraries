use crate::state::icrc3_commit_prepared_transaction;
use crate::utils::trace;

use ic_cdk_macros::update;
pub use icrc3_example_api::updates::commit_prepared_transaction::{
    Args as CommitPreparedTransactionArgs, Response as CommitPreparedTransactionResponse,
};

#[update]
fn commit_prepared_transaction(
    args: CommitPreparedTransactionArgs,
) -> CommitPreparedTransactionResponse {
    let (transaction, timestamp) = args;
    trace(&format!(
        "commit_prepared_transaction: starting with timestamp: {}",
        timestamp
    ));

    // Commit the prepared transaction
    let commit_result = icrc3_commit_prepared_transaction(transaction, timestamp);

    match commit_result {
        Ok(tx_index) => {
            trace(&format!(
                "commit_prepared_transaction: transaction committed successfully with index: {}",
                tx_index
            ));
            Ok(tx_index)
        }
        Err(e) => {
            trace(&format!(
                "commit_prepared_transaction: error committing transaction: {}",
                e
            ));
            Err(format!("Error committing transaction: {}", e))
        }
    }
}
