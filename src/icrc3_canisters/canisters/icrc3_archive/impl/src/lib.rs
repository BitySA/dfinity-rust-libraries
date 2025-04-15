use ic_cdk::export_candid;
// use gldt_swap_api_canister::types::swap::*;
mod guards;
mod lifecycle;
mod memory;
pub mod queries;
pub mod state;
pub mod types;
pub mod updates;
mod utils;
// use ::types::{ HttpRequest, HttpResponse };

use lifecycle::*;
use queries::*;
use updates::*;

export_candid!();
