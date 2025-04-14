use crate::state::read_state;
use ic_cdk::query;
pub use icrc3_archive_api::queries::remaining_capacity::{
    Args as GetArchiveSizeArg, Response as GetArchiveSizeResponse,
};

#[query]
async fn remaining_capacity(_: GetArchiveSizeArg) -> GetArchiveSizeResponse {
    read_state(|s| s.data.archive.remaining_capacity())
}
