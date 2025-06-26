use crate::{generate_pocket_query_call, generate_pocket_update_call};
use icrc3_example_api::add_created_transaction;
use icrc3_example_api::add_random_transaction;
use icrc3_example_api::add_same_transactions;
use icrc3_example_api::create_transactions;
use icrc3_example_api::icrc3_get_archives;
use icrc3_example_api::icrc3_get_blocks;
use icrc3_example_api::icrc3_get_properties;
use icrc3_example_api::icrc3_get_tip_certificate;
use icrc3_example_api::icrc3_supported_block_types;
// // Queries
generate_pocket_query_call!(icrc3_get_properties);
generate_pocket_query_call!(icrc3_get_blocks);
generate_pocket_query_call!(icrc3_get_tip_certificate);
generate_pocket_query_call!(icrc3_supported_block_types);
generate_pocket_query_call!(icrc3_get_archives);
generate_pocket_query_call!(create_transactions);
// Updates
// generate_pocket_update_call!(add_authorized_principals);
// generate_pocket_update_call!(add_batch_transactions);
generate_pocket_update_call!(add_random_transaction);
generate_pocket_update_call!(add_same_transactions);
// generate_pocket_update_call!(remove_authorized_principals);
generate_pocket_update_call!(add_created_transaction);
