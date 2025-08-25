use bity_ic_canister_client::generate_candid_c2c_call;
use bity_ic_icrc3::types::*;

generate_candid_c2c_call!(icrc3_get_blocks);
generate_candid_c2c_call!(icrc3_get_properties);
generate_candid_c2c_call!(icrc3_supported_block_types);
generate_candid_c2c_call!(icrc3_get_archives);
generate_candid_c2c_call!(icrc3_get_tip_certificate);
