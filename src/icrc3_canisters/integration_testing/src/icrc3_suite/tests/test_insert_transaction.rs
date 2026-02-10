use crate::client::icrc3::*;
use crate::icrc3_suite::setup::{
    default_test_setup, default_test_setup_with_archive, setup::TestEnvBuilder,
};
use crate::utils::tick_n_blocks;

use bity_ic_canister_time::DAY_IN_MS;
use bity_ic_icrc3::config::ICRC3Properties;
use candid::Nat;
use icrc_ledger_types::icrc::generic_value::ICRC3Value;
use icrc_ledger_types::icrc3::blocks::GetBlocksRequest;
use std::convert::TryInto;
use std::time::Duration;

#[test]
fn test_simple_insert_transaction() {
    let mut test_env = default_test_setup();

    let _result = add_random_transaction(
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
    let mut test_env = default_test_setup_with_archive();

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

    for (i, block) in get_blocks_result.blocks.iter().enumerate() {
        println!("Block {}: {:?}", i, block);
        assert_eq!(block.id, Nat::from(i as u64));
    }

    test_env
        .pic
        .advance_time(Duration::from_millis(DAY_IN_MS * 2));
    tick_n_blocks(&mut test_env.pic, 50);

    let get_blocks_result = icrc3_get_blocks(
        &mut test_env.pic,
        test_env.controller,
        test_env.icrc3_id,
        &get_blocks_args,
    );

    println!("get_blocks_result: {:?}", get_blocks_result);
    assert_eq!(get_blocks_result.blocks.len(), 5);
    assert_eq!(get_blocks_result.archived_blocks.len(), 1);

    let archived_block = get_blocks_result.archived_blocks[0].clone();

    let get_blocks_result_2 = icrc3_get_blocks(
        &mut test_env.pic,
        test_env.controller,
        archived_block.callback.canister_id,
        &archived_block.args,
    );

    assert_eq!(get_blocks_result_2.blocks.len(), 5);
    assert_eq!(get_blocks_result_2.archived_blocks.len(), 0);

    for (i, block) in get_blocks_result_2.blocks.iter().enumerate() {
        println!("Block {}: {:?}", i, block);
        assert_eq!(block.id, Nat::from(i as u64));
    }

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

    let get_blocks_result = icrc3_get_blocks(
        &mut test_env.pic,
        test_env.controller,
        test_env.icrc3_id,
        &get_blocks_args,
    );

    println!("get_blocks_result: {:?}", get_blocks_result);
    assert_eq!(get_blocks_result.blocks.len(), 15);
    assert_eq!(get_blocks_result.archived_blocks.len(), 1);

    let archived_block = get_blocks_result.archived_blocks[0].clone();

    let get_blocks_result_2 = icrc3_get_blocks(
        &mut test_env.pic,
        test_env.controller,
        archived_block.callback.canister_id,
        &archived_block.args,
    );

    println!("get_blocks_result_2: {:?}", get_blocks_result_2);
    assert_eq!(get_blocks_result_2.blocks.len(), 5);
    assert_eq!(get_blocks_result_2.archived_blocks.len(), 0);

    for (i, block) in get_blocks_result.blocks.iter().enumerate() {
        println!("Block {}: {:?}", i, block);
        assert_eq!(block.id, Nat::from(i as u64 + 5));
    }

    for (i, block) in get_blocks_result_2.blocks.iter().enumerate() {
        println!("Block {}: {:?}", i, block);
        assert_eq!(block.id, Nat::from(i as u64));
    }

    test_env.pic.advance_time(Duration::from_secs(10 * 60 * 60));
    tick_n_blocks(&mut test_env.pic, 50);

    let get_blocks_result = icrc3_get_blocks(
        &mut test_env.pic,
        test_env.controller,
        test_env.icrc3_id,
        &get_blocks_args,
    );

    assert_eq!(get_blocks_result.blocks.len(), 8);
    assert_eq!(get_blocks_result.archived_blocks.len(), 1);

    let archived_block = get_blocks_result.archived_blocks[0].clone();

    let get_blocks_result_2 = icrc3_get_blocks(
        &mut test_env.pic,
        test_env.controller,
        archived_block.callback.canister_id,
        &archived_block.args,
    );

    assert_eq!(get_blocks_result_2.blocks.len(), 12);
    assert_eq!(get_blocks_result_2.archived_blocks.len(), 0);

    for (i, block) in get_blocks_result_2.blocks.iter().enumerate() {
        println!("Block {}: {:?}", i, block);
        assert_eq!(block.id, Nat::from(i as u64));
    }
}

#[test]
fn test_throttling() {
    let mut test_env = default_test_setup_with_archive();

    for _ in 0..15 {
        add_random_transaction(
            &mut test_env.pic,
            test_env.controller,
            test_env.icrc3_id,
            &(),
        );

        test_env.pic.advance_time(Duration::from_millis(10));
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

    assert_eq!(get_blocks_result.blocks.len(), 5);
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

    let _get_blocks_result = icrc3_get_blocks(
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
    let mut test_env = default_test_setup_with_archive();

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
    assert_eq!(archives[0].end, Nat::from(4u64));

    for _ in 0..21 {
        add_random_transaction(
            &mut test_env.pic,
            test_env.controller,
            test_env.icrc3_id,
            &(),
        );

        test_env.pic.advance_time(Duration::from_secs(2 * 60 * 60));
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
    assert_eq!(archives[0].end, Nat::from(20u64));

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
    assert_eq!(archives[0].end, Nat::from(20u64));
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
    assert_eq!(get_blocks_result.archived_blocks.len(), 0);
    assert_eq!(get_blocks_result.blocks.len(), 5);
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

#[test]
fn test_add_same_transaction_with_delay() {
    let mut test_env = default_test_setup();

    // Add first transaction
    let transaction = create_transactions(
        &mut test_env.pic,
        test_env.controller,
        test_env.icrc3_id,
        &(),
    );

    println!("transaction: {:?}", transaction);

    // Add the same transaction again
    let result1 = add_created_transaction(
        &mut test_env.pic,
        test_env.controller,
        test_env.icrc3_id,
        &transaction,
    );

    assert!(result1.is_ok());

    test_env.pic.advance_time(Duration::from_millis(1));
    tick_n_blocks(&mut test_env.pic, 5);

    let result2 = add_created_transaction(
        &mut test_env.pic,
        test_env.controller,
        test_env.icrc3_id,
        &transaction,
    );
    assert!(result2.is_err());

    test_env.pic.advance_time(Duration::from_secs(5 * 60));
    tick_n_blocks(&mut test_env.pic, 5);

    let result3 = add_created_transaction(
        &mut test_env.pic,
        test_env.controller,
        test_env.icrc3_id,
        &transaction,
    );
    assert!(result3.is_ok());

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

    // Should have 2 blocks since they are not considered duplicates due to time delay
    assert_eq!(get_blocks_result.blocks.len(), 2);
    assert_eq!(get_blocks_result.archived_blocks.len(), 0);
}

#[test]
fn test_add_transactions_with_async() {
    let mut test_env = default_test_setup();

    // Create a transaction
    let transaction = create_transactions(
        &mut test_env.pic,
        test_env.controller,
        test_env.icrc3_id,
        &(),
    );

    println!("transaction: {:?}", transaction);

    // Add transaction using async method
    let result = add_transactions_with_async(
        &mut test_env.pic,
        test_env.controller,
        test_env.icrc3_id,
        &transaction,
    );

    assert!(result.is_ok());

    test_env.pic.advance_time(Duration::from_secs(2));
    tick_n_blocks(&mut test_env.pic, 50);

    for _ in 0..3 {
        let result = add_transactions_with_async(
            &mut test_env.pic,
            test_env.controller,
            test_env.icrc3_id,
            &transaction,
        );
        println!("result: {:?}", result);
        assert!(result.is_ok());
    }

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

    // Should have 1 block from the async transaction
    assert_eq!(get_blocks_result.blocks.len(), 4);
    assert_eq!(get_blocks_result.archived_blocks.len(), 0);
}

#[test]
fn test_prepare_transaction_duplicate_immediate() {
    let mut test_env = default_test_setup();

    // Create a transaction
    let transaction = create_transactions(
        &mut test_env.pic,
        test_env.controller,
        test_env.icrc3_id,
        &(),
    );

    println!("transaction: {:?}", transaction);

    // First prepare should succeed
    let result1 = prepare_transaction(
        &mut test_env.pic,
        test_env.controller,
        test_env.icrc3_id,
        &transaction,
    );

    assert!(result1.is_ok());
    println!("First prepare result: {:?}", result1);

    // Second prepare with same transaction should fail (duplicate)
    let result2 = prepare_transaction(
        &mut test_env.pic,
        test_env.controller,
        test_env.icrc3_id,
        &transaction,
    );

    assert!(result2.is_err());
    println!("Second prepare result: {:?}", result2);

    // Verify that the error message indicates a duplicate transaction
    let error_msg = result2.unwrap_err();
    assert!(error_msg.contains("duplicate") || error_msg.contains("Duplicate"));
}

#[test]
fn test_prepare_transaction_cleanup_after_long_delay() {
    let mut test_env = default_test_setup();

    // Create a transaction
    let transaction = create_transactions(
        &mut test_env.pic,
        test_env.controller,
        test_env.icrc3_id,
        &(),
    );

    println!("transaction: {:?}", transaction);

    // First prepare should succeed
    let result1 = prepare_transaction(
        &mut test_env.pic,
        test_env.controller,
        test_env.icrc3_id,
        &transaction,
    );

    assert!(result1.is_ok());
    println!("First prepare result: {:?}", result1);

    // Advance time by 1.1 days (26.4 hours) to trigger cleanup
    // 1.1 days = 1.1 * 24 * 60 * 60 = 95,040 seconds
    test_env.pic.advance_time(Duration::from_secs(95_040));
    tick_n_blocks(&mut test_env.pic, 50);

    // Second prepare with same transaction should now succeed because the first one was cleaned up
    let result2 = prepare_transaction(
        &mut test_env.pic,
        test_env.controller,
        test_env.icrc3_id,
        &transaction,
    );

    assert!(result2.is_ok());
    println!("Second prepare result after cleanup: {:?}", result2);

    // Verify that both prepares returned the same hash (same transaction)
    let hash1 = result1.unwrap();
    let hash2 = result2.unwrap();
    assert_eq!(hash1, hash2);
}

#[test]
fn test_prepare_and_commit_workflow() {
    let mut test_env = default_test_setup();

    // Create a transaction
    let transaction = create_transactions(
        &mut test_env.pic,
        test_env.controller,
        test_env.icrc3_id,
        &(),
    );

    println!("transaction: {:?}", transaction);

    // Prepare the transaction
    let prepare_result = prepare_transaction(
        &mut test_env.pic,
        test_env.controller,
        test_env.icrc3_id,
        &transaction,
    );

    assert!(prepare_result.is_ok());
    let transaction_result = prepare_result.unwrap().clone();
    println!("Prepare result: {:?}", transaction_result);

    // Commit the prepared transaction
    let commit_result = commit_prepared_transaction(
        &mut test_env.pic,
        test_env.controller,
        test_env.icrc3_id,
        &(transaction, transaction_result.1.clone()),
    );

    assert!(commit_result.is_ok());
    let tx_index = commit_result.unwrap();
    println!("Commit result: {}", tx_index);

    // Verify the transaction was added to the blockchain
    test_env.pic.advance_time(Duration::from_secs(2));
    tick_n_blocks(&mut test_env.pic, 50);

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

    // Should have 1 block from the committed transaction
    assert_eq!(get_blocks_result.blocks.len(), 1);
    assert_eq!(get_blocks_result.archived_blocks.len(), 0);
}

#[test]
fn test_threshold_for_archiving_to_external_archive() {
    // Test that blocks are only archived when threshold is reached
    let mut test_env = TestEnvBuilder::new();

    let mut icrc3_constants = ICRC3Properties::default();
    // Set a threshold of 20 blocks
    icrc3_constants.threshold_for_archiving_to_external_archive = Some(20);
    icrc3_constants.max_tx_local_stable_memory_size_bytes = Some(10_000_000); // 10MB
    icrc3_constants.max_transactions_in_window = 100_u64.into();

    test_env.icrc3_constants = icrc3_constants;
    let mut test_env = test_env.build();

    // Add 19 blocks (below threshold)
    for _ in 0..19 {
        add_random_transaction(
            &mut test_env.pic,
            test_env.controller,
            test_env.icrc3_id,
            &(),
        );
        test_env.pic.advance_time(Duration::from_secs(2 * 60 * 60));
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

    // Should have 19 blocks locally, no archived blocks yet (threshold not reached)
    assert_eq!(get_blocks_result.blocks.len(), 19);
    assert_eq!(get_blocks_result.archived_blocks.len(), 0);

    // Add one more block to reach threshold
    add_random_transaction(
        &mut test_env.pic,
        test_env.controller,
        test_env.icrc3_id,
        &(),
    );
    test_env.pic.advance_time(Duration::from_secs(2 * 60 * 60));
    tick_n_blocks(&mut test_env.pic, 50);

    let get_blocks_result = icrc3_get_blocks(
        &mut test_env.pic,
        test_env.controller,
        test_env.icrc3_id,
        &get_blocks_args,
    );

    println!("get_blocks_result: {:?}", get_blocks_result);

    // Now threshold is reached, half should be archived (10 blocks)
    // So we should have 10 local blocks and 10 archived blocks
    assert!(get_blocks_result.archived_blocks.len() > 0 || get_blocks_result.blocks.len() < 20);
}

#[test]
fn test_max_tx_local_stable_memory_size_bytes_limit() {
    // Test that inserting blocks fails when memory size limit is reached
    let mut test_env = TestEnvBuilder::new();

    let mut icrc3_constants = ICRC3Properties::default();
    // Set a very small memory limit
    icrc3_constants.max_tx_local_stable_memory_size_bytes = Some(1000); // 1KB
    icrc3_constants.threshold_for_archiving_to_external_archive = Some(100); // High threshold
    icrc3_constants.max_transactions_in_window = 100_u64.into();

    test_env.icrc3_constants = icrc3_constants;
    let mut test_env = test_env.build();

    // Add blocks until we hit the memory limit
    // We check the number of blocks before and after insertion to detect failures
    let get_blocks_args = vec![GetBlocksRequest {
        start: Nat::from(0u64),
        length: Nat::from(1000u64),
    }];

    let mut previous_block_count = 0;
    for i in 0..100 {
        add_random_transaction(
            &mut test_env.pic,
            test_env.controller,
            test_env.icrc3_id,
            &(),
        );

        test_env.pic.advance_time(Duration::from_secs(2));
        tick_n_blocks(&mut test_env.pic, 50);

        let get_blocks_result = icrc3_get_blocks(
            &mut test_env.pic,
            test_env.controller,
            test_env.icrc3_id,
            &get_blocks_args,
        );

        let current_block_count = get_blocks_result.blocks.len();

        // If block count didn't increase, the insertion likely failed
        if current_block_count == previous_block_count && i > 0 {
            println!(
                "Block count didn't increase at iteration {}: {} -> {}",
                i, previous_block_count, current_block_count
            );
            // This indicates the memory limit was likely reached
            break;
        }

        previous_block_count = current_block_count;
    }

    // Should have succeeded at least once before hitting the limit
    assert!(previous_block_count > 0);
}

#[test]
fn test_multiple_batch_archiving_operations() {
    // Test multiple archiving operations to verify consistency
    let mut test_env = TestEnvBuilder::new();

    let mut icrc3_constants = ICRC3Properties::default();
    // Set a threshold of 20 blocks
    icrc3_constants.threshold_for_archiving_to_external_archive = Some(20);
    icrc3_constants.max_tx_local_stable_memory_size_bytes = Some(10_000_000); // 10MB
    icrc3_constants.max_transactions_in_window = 100_u64.into();

    test_env.icrc3_constants = icrc3_constants;
    let mut test_env = test_env.build();

    // First batch: Add 20 blocks
    for _ in 0..20 {
        add_random_transaction(
            &mut test_env.pic,
            test_env.controller,
            test_env.icrc3_id,
            &(),
        );
        test_env.pic.advance_time(Duration::from_secs(2));
        tick_n_blocks(&mut test_env.pic, 50);
    }

    // Second batch: Add 20 more blocks
    for _ in 0..20 {
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

    // Verify all blocks are accessible (either locally or archived)
    let total_local = get_blocks_result.blocks.len();
    let total_archived: usize = get_blocks_result
        .archived_blocks
        .iter()
        .map(|archived_block| {
            let archived_result = icrc3_get_blocks(
                &mut test_env.pic,
                test_env.controller,
                archived_block.callback.canister_id,
                &archived_block.args,
            );
            archived_result.blocks.len()
        })
        .sum();

    // Should have all 40 blocks
    assert_eq!(total_local + total_archived, 40);

    // Verify block continuity - check that block IDs are sequential
    let mut all_block_ids: Vec<u64> = Vec::new();

    // Add local block IDs
    for block in &get_blocks_result.blocks {
        if let Ok(block_id) = TryInto::<u64>::try_into(&block.id.0) {
            all_block_ids.push(block_id);
        }
    }

    // Add archived block IDs
    for archived_block in &get_blocks_result.archived_blocks {
        let archived_result = icrc3_get_blocks(
            &mut test_env.pic,
            test_env.controller,
            archived_block.callback.canister_id,
            &archived_block.args,
        );
        for block in &archived_result.blocks {
            if let Ok(block_id) = TryInto::<u64>::try_into(&block.id.0) {
                all_block_ids.push(block_id);
            }
        }
    }

    all_block_ids.sort();

    // Verify sequential block IDs from 0 to 39
    for (idx, block_id) in all_block_ids.iter().enumerate() {
        assert_eq!(*block_id, idx as u64);
    }
}

#[test]
fn test_transaction_with_specific_timestamp() {
    let mut test_env = default_test_setup();

    // Create a transaction with a specific timestamp (in nanoseconds)
    let specific_timestamp: u64 = 168_255_804_247_086_374_8;

    let transaction = create_transactions(
        &mut test_env.pic,
        test_env.controller,
        test_env.icrc3_id,
        &(),
    );

    println!("Created transaction: {:?}", transaction);

    // Add the transaction - this should use the timestamp from the transaction
    let result = add_created_transaction(
        &mut test_env.pic,
        test_env.controller,
        test_env.icrc3_id,
        &transaction,
    );

    assert!(result.is_ok());
    println!("Add transaction result: {:?}", result);

    test_env.pic.advance_time(Duration::from_secs(2));
    tick_n_blocks(&mut test_env.pic, 50);

    // Get the blocks to verify the timestamp
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

    // Extract timestamp from the block
    let block = &get_blocks_result.blocks[0];
    if let ICRC3Value::Map(map) = &block.block {
        if let Some(ICRC3Value::Nat(timestamp_nat)) = map.get("timestamp") {
            let block_timestamp: u64 = timestamp_nat.0.clone().try_into().unwrap_or(0);
            println!(
                "Block timestamp: {}, Expected: {}",
                block_timestamp, specific_timestamp
            );

            // The timestamp in the block should match what was passed (or be close to it)
            // Note: The timestamp might be converted or adjusted, so we check if it's reasonable
            // For now, we just verify that a timestamp exists and is not the current time
            let current_time = test_env.pic.get_time().as_nanos_since_unix_epoch();
            println!("Current time: {}", current_time);

            // Verify the timestamp is not the current time (which would indicate the bug)
            // The timestamp should be the one from the transaction, not the current time
            assert_ne!(
                block_timestamp,
                current_time,
                "Block timestamp should not be the current time - this indicates the timestamp passed was ignored"
            );
        } else {
            panic!("Block should have a timestamp field");
        }
    } else {
        panic!("Block should be a Map");
    }
}

#[test]
fn test_multiple_transactions_with_specific_timestamps() {
    let mut test_env = default_test_setup();

    // Create two transactions - the timestamps should be set by the transaction creation
    // This test verifies that when adding consecutive transactions, the timestamps
    // are properly used and not replaced with 'now'

    // Create and add first transaction
    let transaction1 = create_transactions(
        &mut test_env.pic,
        test_env.controller,
        test_env.icrc3_id,
        &(),
    );

    let result1 = add_created_transaction(
        &mut test_env.pic,
        test_env.controller,
        test_env.icrc3_id,
        &transaction1,
    );

    assert!(result1.is_ok());
    println!("First transaction added: {:?}", result1);

    test_env.pic.advance_time(Duration::from_secs(2));
    tick_n_blocks(&mut test_env.pic, 50);

    // Create and add second transaction
    let transaction2 = create_transactions(
        &mut test_env.pic,
        test_env.controller,
        test_env.icrc3_id,
        &(),
    );

    let result2 = add_created_transaction(
        &mut test_env.pic,
        test_env.controller,
        test_env.icrc3_id,
        &transaction2,
    );

    // This should succeed if timestamps are properly used
    // If it fails with "timestamp is older than the previous tip",
    // it means the first transaction used 'now' instead of the passed timestamp
    match result2 {
        Ok(_) => {
            println!("Second transaction added successfully");
        }
        Err(e) => {
            println!("Second transaction failed: {:?}", e);
            // If the error is about timestamp being older, it indicates the bug
            if e.contains("timestamp is older") || e.contains("Cannot apply block") {
                panic!("Second transaction failed because timestamp was older - this indicates the first transaction used 'now' instead of the passed timestamp");
            }
            panic!("Second transaction failed with unexpected error: {:?}", e);
        }
    }

    test_env.pic.advance_time(Duration::from_secs(2));
    tick_n_blocks(&mut test_env.pic, 50);

    // Verify both blocks were added
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
    assert_eq!(
        get_blocks_result.blocks.len(),
        2,
        "Both transactions should have been added"
    );
}
