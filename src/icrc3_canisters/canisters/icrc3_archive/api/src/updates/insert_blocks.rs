use crate::types::encoded_blocks::EncodedBlock;

use candid::CandidType;
use serde::Deserialize;

pub type Args = Vec<EncodedBlock>;

#[derive(CandidType, Deserialize, Debug)]
pub enum Response {
    Success,
    Error(String),
}
