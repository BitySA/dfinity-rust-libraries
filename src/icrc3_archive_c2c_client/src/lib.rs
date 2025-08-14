use bity_ic_canister_client::canister_client_macros::candid;
use bity_ic_canister_client::generate_candid_c2c_call;
use bity_ic_icrc3_archive_api::*;

// Queries
generate_candid_c2c_call!(icrc3_get_blocks);
generate_candid_c2c_call!(get_version);
generate_candid_c2c_call!(remaining_capacity);
generate_candid_c2c_call!(total_transactions);

// Updates
generate_candid_c2c_call!(insert_blocks);
