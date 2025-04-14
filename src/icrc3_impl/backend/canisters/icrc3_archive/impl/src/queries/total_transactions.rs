use crate::state::read_state;
use ic_cdk::query;
pub use icrc3_archive_api::queries::total_transactions::{
    Args as GetTotalTransactionsArg, Response as GetTotalTransactionsResponse,
};

#[query]
async fn total_transactions(_: GetTotalTransactionsArg) -> GetTotalTransactionsResponse {
    read_state(|s| s.data.archive.get_len() as usize)
}
