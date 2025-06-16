use crate::state::icrc3_get_blocks as icrc3_get_blocks_impl;

use ic_cdk::query;
pub use icrc3_example_api::queries::icrc3_get_blocks::{
    Args as GetBlocksArg, Response as GetBlocksResponse,
};
pub use icrc_ledger_types::icrc3::blocks::GetBlocksResult;

#[query]
fn icrc3_get_blocks(args: GetBlocksArg) -> GetBlocksResult {
    icrc3_get_blocks_impl(args)
}
