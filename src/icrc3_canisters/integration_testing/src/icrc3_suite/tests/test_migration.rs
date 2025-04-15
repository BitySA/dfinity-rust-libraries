use crate::client::icrc3::*;
use crate::icrc3_suite::setup::{default_test_setup, setup_icrc3::upgrade_icrc3_canister};

use bity_ic_types::BuildVersion;
use candid::Nat;
use icrc3_example_api::post_upgrade::UpgradeArgs;
use icrc_ledger_types::icrc3::blocks::GetBlocksRequest;

#[test]
fn test_migration() {
    let mut test_env = default_test_setup();

    let result = add_random_transaction(
        &mut test_env.pic,
        test_env.controller,
        test_env.icrc3_id,
        &(),
    );

    let get_blocks_args = vec![GetBlocksRequest {
        start: Nat::from(0u64),
        length: Nat::from(10u64),
    }];

    let get_blocks_result = icrc3_get_blocks(
        &mut test_env.pic,
        test_env.controller,
        test_env.icrc3_id,
        &get_blocks_args,
    );

    let mut archived_blocks_before = Vec::new();
    get_blocks_result
        .archived_blocks
        .iter()
        .for_each(|archived_block| {
            let get_blocks_result_2 = icrc3_get_blocks(
                &mut test_env.pic,
                test_env.controller,
                archived_block.callback.canister_id,
                &archived_block.args,
            );
            archived_blocks_before.extend(get_blocks_result_2.blocks);
        });

    upgrade_icrc3_canister(
        &mut test_env.pic,
        test_env.icrc3_id,
        icrc3_example_api::Args::Upgrade(UpgradeArgs {
            version: BuildVersion::min(),
            commit_hash: "commit_hash 2".to_string(),
        }),
        test_env.controller,
    );

    let get_blocks_result_3 = icrc3_get_blocks(
        &mut test_env.pic,
        test_env.controller,
        test_env.icrc3_id,
        &get_blocks_args,
    );

    let mut archived_blocks_after = Vec::new();
    get_blocks_result_3
        .archived_blocks
        .iter()
        .for_each(|archived_block| {
            let get_blocks_result_4 = icrc3_get_blocks(
                &mut test_env.pic,
                test_env.controller,
                archived_block.callback.canister_id,
                &archived_block.args,
            );
            archived_blocks_after.extend(get_blocks_result_4.blocks);
        });

    assert_eq!(archived_blocks_after, archived_blocks_before);
}
