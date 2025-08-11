use crate::state::read_state;
use crate::utils::trace;

use ic_cdk_macros::query;
pub use icrc3_example_api::updates::create_transactions::{
    Args as CreateTransactionsArgs, Response as CreateTransactionsResponse,
};

#[query]
fn create_transactions(_: CreateTransactionsArgs) -> CreateTransactionsResponse {
    trace("create_transactions");
    let transaction = read_state(|state| state.data.create_fake_transaction());

    transaction
}
