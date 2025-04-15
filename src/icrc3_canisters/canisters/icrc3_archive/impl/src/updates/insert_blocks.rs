use crate::guards::caller_is_authorized;
use crate::state::mutate_state;
pub use bity_ic_icrc3_archive_api::insert_blocks::{
    Args as AppendTransactionsArgs, Response as AppendTransactionsResponse,
};
use ic_cdk::update;

#[update(guard = "caller_is_authorized")]
async fn insert_blocks(new_blocks: AppendTransactionsArgs) -> AppendTransactionsResponse {
    let max_memory_size_bytes =
        mutate_state(|s| s.data.archive.archive_config.get_max_memory_size_bytes());

    if max_memory_size_bytes < new_blocks.len() as u128 {
        ic_cdk::api::trap(
            format!(
                "New blocks size is too big, limit is: {}",
                max_memory_size_bytes
            )
            .as_str(),
        );
    }

    // Insert Blocks trap in case of no space left. Rolling back the transaction.
    let result = mutate_state(|s| s.data.archive.insert_blocks(new_blocks));

    match result {
        Ok(_) => AppendTransactionsResponse::Success,
        Err(e) => AppendTransactionsResponse::Error(e),
    }
}
