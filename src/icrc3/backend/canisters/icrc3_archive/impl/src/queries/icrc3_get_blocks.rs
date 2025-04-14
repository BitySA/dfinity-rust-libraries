// use crate::guards::caller_is_main_canister;
use crate::state::read_state;
use crate::utils::trace;

use candid::Nat;
use ic_cdk::query;
use icrc3_archive_api::lifecycle::BlockType;
use icrc3_archive_api::types::block_interface::Block;
use icrc3_archive_api::types::defaultblock::DefaultBlock;
pub use icrc3_archive_api::{
    icrc3_get_blocks::{Args as GetBlocksArg, Response as GetBlockseResponse},
    types::encoded_blocks::EncodedBlock,
};
use icrc_ledger_types::icrc3::blocks::BlockWithId;

// #[query(guard = "caller_is_main_canister")]
#[query]
fn icrc3_get_blocks(req: GetBlocksArg) -> GetBlockseResponse {
    let log_length = read_state(|s| s.data.archive.get_len());
    let block_type = read_state(|s| s.data.block_type.clone());
    let mut blocks = vec![];

    let mut response = for arg in req {
        let start = arg.start.clone().0.try_into().unwrap();
        let length = arg.length.clone().0.try_into().unwrap();

        let response = read_state(|s| s.data.archive.get_blocks_range(start, length));

        for (idx, block) in response.into_iter().enumerate() {
            let block_id = idx + arg.start.clone();
            match block_type {
                BlockType::Default => {
                    let encoded_block = EncodedBlock::from_vec(block.into_vec());
                    // decode block
                    match DefaultBlock::decode(encoded_block) {
                        Ok(block) => {
                            blocks.push(BlockWithId {
                                id: Nat::from(block_id),
                                block: block.transaction,
                            });
                        }
                        Err(e) => {
                            trace(&format!("Error decoding block: {}", e));
                        }
                    }
                }
                _ => {
                    // TODO: handle other block types
                    trace(&format!("TODO: handle other block types"));
                }
            }
        }
    };

    GetBlockseResponse {
        log_length: Nat::from(log_length),
        blocks: blocks,
        archived_blocks: vec![],
    }
}
