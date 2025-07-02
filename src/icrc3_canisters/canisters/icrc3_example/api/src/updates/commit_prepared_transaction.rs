use crate::types::FakeTransaction;

pub type Args = (FakeTransaction, u128);
pub type Response = Result<u64, String>;
