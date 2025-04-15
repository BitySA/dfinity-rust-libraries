use crate::state::icrc3_get_properties as icrc3_get_properties_impl;
use crate::state::FakeTransaction;

use ic_cdk::query;
pub use icrc3_example_api::icrc3_get_properties::{
    Args as GetArchivePropsArg, Response as GetArchivePropsResponse,
};

#[query]
async fn icrc3_get_properties(_: GetArchivePropsArg) -> GetArchivePropsResponse {
    icrc3_get_properties_impl::<FakeTransaction>()
}
