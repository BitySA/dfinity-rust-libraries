use crate::state::icrc3_get_tip_certificate as icrc3_get_tip_certificate_impl;
use crate::state::FakeTransaction;

use ic_cdk::query;
pub use icrc3_example_api::icrc3_get_tip_certificate::{
    Args as GetTipCertificateArg, Response as GetTipCertificateResponse,
};

#[query]
async fn icrc3_get_tip_certificate(_: GetTipCertificateArg) -> GetTipCertificateResponse {
    icrc3_get_tip_certificate_impl::<FakeTransaction>()
}
