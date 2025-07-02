use crate::types::FakeTransaction;

pub type Args = FakeTransaction;
pub type Response = Result<(Vec<u8>, u128), String>;
