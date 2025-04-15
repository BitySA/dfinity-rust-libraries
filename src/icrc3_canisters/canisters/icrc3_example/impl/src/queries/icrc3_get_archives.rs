use crate::state::icrc3_get_archives as icrc3_get_archives_impl;
use crate::state::FakeTransaction;

use ic_cdk::query;
pub use icrc3_example_api::icrc3_get_archives::{
    Args as GetArchivesArg, Response as GetArchivesResponse,
};

#[query]
async fn icrc3_get_archives(_: GetArchivesArg) -> GetArchivesResponse {
    icrc3_get_archives_impl::<FakeTransaction>()
}
