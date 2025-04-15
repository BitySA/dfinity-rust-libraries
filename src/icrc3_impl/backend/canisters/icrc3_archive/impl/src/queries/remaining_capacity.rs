use crate::state::read_state;
pub use bity_ic_icrc3_archive_api::queries::remaining_capacity::{
    Args as GetArchiveSizeArg, Response as GetArchiveSizeResponse,
};
use ic_cdk::query;

#[query]
async fn remaining_capacity(_: GetArchiveSizeArg) -> GetArchiveSizeResponse {
    read_state(|s| s.data.archive.remaining_capacity())
}
