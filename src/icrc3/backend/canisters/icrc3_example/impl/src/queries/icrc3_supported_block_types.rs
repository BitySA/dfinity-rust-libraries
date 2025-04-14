use crate::state::icrc3_supported_block_types as icrc3_supported_block_types_impl;
use crate::state::FakeTransaction;

use ic_cdk::query;
pub use icrc3_example_api::icrc3_supported_block_types::{
    Args as GetSupportedBlockTypesArg, Response as GetSupportedBlockTypesResponse,
};

#[query]
async fn icrc3_supported_block_types(
    _: GetSupportedBlockTypesArg,
) -> GetSupportedBlockTypesResponse {
    icrc3_supported_block_types_impl::<FakeTransaction>()
}
