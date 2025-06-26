use crate::state::icrc3_add_transaction;
use crate::state::read_state;
use crate::utils::trace;

use ic_cdk_macros::update;
pub use icrc3_example_api::updates::add_created_transaction::{
    Args as AddCreatedTransactionArgs, Response as AddCreatedTransactionResponse,
};

#[update]
fn add_created_transaction(
    transaction: AddCreatedTransactionArgs,
) -> AddCreatedTransactionResponse {
    trace(&format!("add_created_transaction"));

    match icrc3_add_transaction(transaction.clone())
        .map_err(|e| format!("Error adding transaction: {}", e))
    {
        Ok(_) => {
            trace(&format!("transaction added."));
            Ok(())
        }
        Err(e) => {
            trace(&format!("error adding transaction: {}", e));
            Err(e)
        }
    }
}
