use crate::state::read_state;
pub use bity_ic_icrc3_archive_api::queries::total_transactions::{
    Args as GetTotalTransactionsArg, Response as GetTotalTransactionsResponse,
};
use ic_cdk::query;

#[query]
async fn total_transactions(_: GetTotalTransactionsArg) -> GetTotalTransactionsResponse {
    read_state(|s| s.data.archive.get_len() as usize)
}
