use crate::state::read_state;
pub use bity_ic_icrc3_archive_api::get_version::{
    Args as GetVersionArg, Response as GetVersionResponse,
};
use ic_cdk::query;

#[query]
async fn get_version(_: GetVersionArg) -> GetVersionResponse {
    read_state(|s| s.env.version())
}
