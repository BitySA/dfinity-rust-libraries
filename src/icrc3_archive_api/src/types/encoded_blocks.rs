use candid::CandidType;
use ic_stable_structures::{storable::Bound, Storable};
use minicbor::{Decode, Encode};
use serde::{Deserialize, Serialize};
use std::borrow::Cow;

#[derive(
    Clone,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    Hash,
    Debug,
    Encode,
    Decode,
    CandidType,
    Serialize,
    Deserialize,
)]
#[cbor(map)]
pub struct EncodedBlock {
    #[n(0)]
    pub block: Vec<u8>,
}
impl Storable for EncodedBlock {
    fn to_bytes(&self) -> Cow<[u8]> {
        let mut buffer = Vec::new();
        minicbor::encode(self, &mut buffer).expect("failed to encode EncodedBlock");
        Cow::Owned(buffer)
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        minicbor::decode(&bytes).expect("failed to decode EncodedBlock")
    }

    const BOUND: Bound = Bound::Unbounded;
}

impl From<Vec<u8>> for EncodedBlock {
    fn from(bytes: Vec<u8>) -> Self {
        Self::from_vec(bytes)
    }
}

impl EncodedBlock {
    pub fn from_vec(bytes: Vec<u8>) -> Self {
        Self { block: bytes }
    }

    pub fn into_vec(self) -> Vec<u8> {
        self.block
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.block
    }

    pub fn size_bytes(&self) -> usize {
        self.block.len()
    }
}
