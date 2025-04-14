use crate::state::icrc3_get_blocks as icrc3_get_blocks_impl;
use crate::state::FakeTransaction;

use ic_cdk::query;
pub use icrc3_example_api::queries::icrc3_get_blocks::{
    Args as GetBlocksArg, Response as GetBlocksResponse,
};
pub use icrc_ledger_types::icrc3::blocks::GetBlocksResult;

#[query]
async fn icrc3_get_blocks(args: GetBlocksArg) -> GetBlocksResult {
    icrc3_get_blocks_impl::<FakeTransaction>(args).await
}
