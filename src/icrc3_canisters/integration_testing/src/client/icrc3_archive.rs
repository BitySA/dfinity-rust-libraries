use crate::generate_pocket_query_call;
// use icrc3_archive_api::get_archive_size;
// use icrc3_archive_api::get_transaction;
use bity_ic_icrc3_archive_api::get_version;
// use icrc3_archive_api::icrc3_get_blocks;
use bity_ic_icrc3_archive_api::remaining_capacity;
use bity_ic_icrc3_archive_api::total_transactions;

// Queries
// generate_pocket_query_call!(get_archive_size);
// generate_pocket_query_call!(get_transaction);
generate_pocket_query_call!(get_version);
// generate_pocket_query_call!(icrc3_get_blocks);
generate_pocket_query_call!(remaining_capacity);
generate_pocket_query_call!(total_transactions);

// Updates
