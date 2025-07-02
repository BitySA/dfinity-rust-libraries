use crate::state::icrc3_prepare_transaction;
use crate::utils::trace;

use ic_cdk_macros::update;
pub use icrc3_example_api::updates::prepare_transaction::{
    Args as PrepareTransactionArgs, Response as PrepareTransactionResponse,
};

#[update]
fn prepare_transaction(transaction: PrepareTransactionArgs) -> PrepareTransactionResponse {
    trace(&format!("prepare_transaction: starting"));

    // Prepare the transaction
    let prepared_tx = match icrc3_prepare_transaction(transaction) {
        Ok(prepared) => {
            trace(&format!(
                "prepare_transaction: transaction prepared successfully"
            ));
            prepared
        }
        Err(e) => {
            trace(&format!(
                "prepare_transaction: error preparing transaction: {}",
                e
            ));
            return Err(format!("Error preparing transaction: {}", e));
        }
    };

    trace(&format!(
        "prepare_transaction: returning hash: {:?}",
        prepared_tx.transaction_hash
    ));
    Ok((prepared_tx.transaction_hash, prepared_tx.timestamp))
}
