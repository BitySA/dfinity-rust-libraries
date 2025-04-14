use crate::state::read_state;
use ic_cdk::query;
pub use icrc3_archive_api::get_version::{Args as GetVersionArg, Response as GetVersionResponse};

#[query]
async fn get_version(_: GetVersionArg) -> GetVersionResponse {
    read_state(|s| s.env.version())
}
