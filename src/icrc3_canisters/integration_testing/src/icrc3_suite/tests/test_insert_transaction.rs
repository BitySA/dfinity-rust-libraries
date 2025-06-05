use crate::client::icrc3::*;
use crate::icrc3_suite::setup::default_test_setup;
use crate::utils::tick_n_blocks;

use candid::Nat;
use icrc_ledger_types::icrc3::blocks::GetBlocksRequest;
use std::time::Duration;

#[test]
fn test_simple_insert_transaction() {
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

            println!("get_blocks_result_2: {:?}", get_blocks_result_2);
            get_blocks_result_2.blocks.iter().for_each(|block| {
                println!("block: {:?}", block);
            });
        });
}

#[test]
fn test_multiple_transactions() {
    let mut test_env = default_test_setup();

    for _ in 0..10 {
        add_random_transaction(
            &mut test_env.pic,
            test_env.controller,
            test_env.icrc3_id,
            &(),
        );

        test_env.pic.advance_time(Duration::from_secs(2));
        tick_n_blocks(&mut test_env.pic, 50);
    }

    let get_blocks_args = vec![GetBlocksRequest {
        start: Nat::from(0u64),
        length: Nat::from(100u64),
    }];

    let get_blocks_result = icrc3_get_blocks(
        &mut test_env.pic,
        test_env.controller,
        test_env.icrc3_id,
        &get_blocks_args,
    );

    assert_eq!(get_blocks_result.blocks.len(), 10);
}

#[test]
fn test_throttling() {
    let mut test_env = default_test_setup();

    for _ in 0..15 {
        add_random_transaction(
            &mut test_env.pic,
            test_env.controller,
            test_env.icrc3_id,
            &(),
        );

        test_env.pic.advance_time(Duration::from_millis(100));
        tick_n_blocks(&mut test_env.pic, 50);
    }

    let get_blocks_args = vec![GetBlocksRequest {
        start: Nat::from(0u64),
        length: Nat::from(100u64),
    }];

    let get_blocks_result = icrc3_get_blocks(
        &mut test_env.pic,
        test_env.controller,
        test_env.icrc3_id,
        &get_blocks_args,
    );

    assert_eq!(get_blocks_result.blocks.len(), 13);
    assert_eq!(get_blocks_result.archived_blocks.len(), 0);
}

#[test]
fn test_concurrent_transactions() {
    let mut test_env = default_test_setup();

    add_same_transactions(
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

    println!("get_blocks_result: {:?}", get_blocks_result);

    assert_eq!(get_blocks_result.blocks.len(), 1);
    assert_eq!(get_blocks_result.archived_blocks.len(), 0);
}

#[test]
fn test_certificate() {
    let mut test_env = default_test_setup();

    for _ in 0..10 {
        add_random_transaction(
            &mut test_env.pic,
            test_env.controller,
            test_env.icrc3_id,
            &(),
        );

        test_env.pic.advance_time(Duration::from_secs(2));
        tick_n_blocks(&mut test_env.pic, 50);
    }

    let get_blocks_args = vec![GetBlocksRequest {
        start: Nat::from(0u64),
        length: Nat::from(50u64),
    }];

    let get_blocks_result = icrc3_get_blocks(
        &mut test_env.pic,
        test_env.controller,
        test_env.icrc3_id,
        &get_blocks_args,
    );

    let certificate = icrc3_get_tip_certificate(
        &mut test_env.pic,
        test_env.controller,
        test_env.icrc3_id,
        &(),
    );

    println!("certificate: {:?}", certificate);
}

#[test]
fn test_get_archives() {
    let mut test_env = default_test_setup();

    for _ in 0..10 {
        add_random_transaction(
            &mut test_env.pic,
            test_env.controller,
            test_env.icrc3_id,
            &(),
        );

        test_env.pic.advance_time(Duration::from_secs(2 * 60));
        tick_n_blocks(&mut test_env.pic, 50);
    }

    test_env.pic.advance_time(Duration::from_secs(10 * 60));
    tick_n_blocks(&mut test_env.pic, 50);

    let archives = icrc3_get_archives(
        &mut test_env.pic,
        test_env.controller,
        test_env.icrc3_id,
        &(),
    );

    println!("archives: {:?}", archives);

    assert_eq!(archives.len(), 1);
    assert_eq!(archives[0].start, Nat::from(0u64));
    assert_eq!(archives[0].end, Nat::from(9u64));

    for _ in 0..21 {
        add_random_transaction(
            &mut test_env.pic,
            test_env.controller,
            test_env.icrc3_id,
            &(),
        );

        test_env.pic.advance_time(Duration::from_secs(2 * 60));
        tick_n_blocks(&mut test_env.pic, 50);
    }

    let archives = icrc3_get_archives(
        &mut test_env.pic,
        test_env.controller,
        test_env.icrc3_id,
        &(),
    );

    println!("archives: {:?}", archives);
    assert_eq!(archives.len(), 1);
    assert_eq!(archives[0].start, Nat::from(0u64));
    assert_eq!(archives[0].end, Nat::from(28u64));

    test_env.pic.advance_time(Duration::from_secs(100 * 60));
    tick_n_blocks(&mut test_env.pic, 50);

    let archives = icrc3_get_archives(
        &mut test_env.pic,
        test_env.controller,
        test_env.icrc3_id,
        &(),
    );

    println!("archives: {:?}", archives);
    assert_eq!(archives.len(), 1);
    assert_eq!(archives[0].start, Nat::from(0u64));
    assert_eq!(archives[0].end, Nat::from(30u64));
}

#[test]
fn test_get_blocks_after_multiple_operations() {
    let mut test_env = default_test_setup();

    for _ in 0..5 {
        add_random_transaction(
            &mut test_env.pic,
            test_env.controller,
            test_env.icrc3_id,
            &(),
        );

        test_env.pic.advance_time(Duration::from_secs(2 * 120));
        tick_n_blocks(&mut test_env.pic, 5);
    }

    let get_blocks_args = vec![GetBlocksRequest {
        start: Nat::from(0u64),
        length: Nat::from(10u64),
    }];

    test_env.pic.advance_time(Duration::from_secs(2 * 120));
    tick_n_blocks(&mut test_env.pic, 5);

    let get_blocks_result = icrc3_get_blocks(
        &mut test_env.pic,
        test_env.controller,
        test_env.icrc3_id,
        &get_blocks_args,
    );

    println!("get_blocks_result: {:?}", get_blocks_result);

    assert_eq!(get_blocks_result.log_length, Nat::from(5u64));
    assert_eq!(get_blocks_result.archived_blocks.len(), 1);
    assert_eq!(get_blocks_result.archived_blocks[0].args.len(), 1);
    assert_eq!(
        get_blocks_result.archived_blocks[0].args[0],
        GetBlocksRequest {
            start: Nat::from(0u64),
            length: Nat::from(5u64),
        }
    );

    tick_n_blocks(&mut test_env.pic, 5);

    let get_blocks_result = icrc3_get_blocks(
        &mut test_env.pic,
        test_env.controller,
        get_blocks_result.archived_blocks[0].callback.canister_id,
        &get_blocks_result.archived_blocks[0].args,
    );

    println!("get_blocks_result: {:?}", get_blocks_result);

    assert_eq!(get_blocks_result.log_length, Nat::from(5u64));
    assert_eq!(get_blocks_result.blocks.len(), 5);
    assert_eq!(get_blocks_result.archived_blocks.len(), 0);
}

#[test]
fn test_add_same_transactions() {
    let mut test_env = default_test_setup();

    add_same_transactions(
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

    println!("get_blocks_result: {:?}", get_blocks_result);

    assert_eq!(get_blocks_result.blocks.len(), 1);
    assert_eq!(get_blocks_result.archived_blocks.len(), 0);
}
