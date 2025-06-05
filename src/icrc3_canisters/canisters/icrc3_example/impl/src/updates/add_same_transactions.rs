use crate::state::icrc3_add_transaction;
use crate::state::read_state;
use crate::utils::trace;

use ic_cdk_macros::update;
pub use icrc3_example_api::updates::add_random_transaction::{
    Args as RandomTransactionArgs, Response as RandomTransactionResponse,
};

#[update]
fn add_same_transactions(_: RandomTransactionArgs) -> RandomTransactionResponse {
    trace(&format!("add_same_transactions"));
    let transaction = read_state(|state| state.data.create_fake_transaction());

    match icrc3_add_transaction(transaction.clone())
        .map_err(|e| format!("Error adding transaction: {}", e))
    {
        Ok(_) => {}
        Err(e) => {
            ic_cdk::trap(&format!("error adding transaction: {}", e));
        }
    }

    match icrc3_add_transaction(transaction.clone()) {
        Ok(_) => {
            ic_cdk::trap(&format!("Transaction already added"));
        }
        Err(e) => match e {
            bity_ic_icrc3::types::Icrc3Error::DuplicateTransaction { duplicate_of } => {
                if duplicate_of == 1 {
                    trace(&format!("transaction already added: {}", duplicate_of));
                } else {
                    ic_cdk::trap(&format!(
                        "Transaction already added, but duplicate_of is not 1"
                    ));
                }
            }
            _ => {
                ic_cdk::trap(&format!(
                    "Error adding transaction should be DuplicateTransaction, but is {}",
                    e
                ));
            }
        },
    }

    // trace(&format!("add_random_transaction result: {:?}", result));

    ()
}
