use crate::state::icrc3_add_transaction;
use crate::state::read_state;
use crate::utils::trace;

use ic_cdk_macros::update;
pub use icrc3_example_api::updates::add_random_transaction::{
    Args as RandomTransactionArgs, Response as RandomTransactionResponse,
};

#[update]
async fn add_random_transaction(_: RandomTransactionArgs) -> RandomTransactionResponse {
    trace(&format!("add_random_transaction"));
    let transaction = read_state(|state| state.data.create_fake_transaction());

    trace(&format!(
        "add_random_transaction transaction: {:?}",
        transaction
    ));

    match icrc3_add_transaction(transaction.clone())
        .await
        .map_err(|e| format!("Error adding transaction: {}", e))
    {
        Ok(_) => {
            trace(&format!("transaction added."));
        }
        Err(e) => {
            trace(&format!("error adding transaction: {}", e));
        }
    }

    // trace(&format!("add_random_transaction result: {:?}", result));

    ()
}
