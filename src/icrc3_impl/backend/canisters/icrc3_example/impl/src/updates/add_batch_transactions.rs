use crate::guards::caller_is_authorized;
use crate::state::{read_state, replace_state, take_state};
use bity_ic_canister_tracing_macros::trace;
use ic_cdk_macros::update;
use icrc3::interface::ICRC3Interface;
pub use icrc3_example_api::add_batch_transactions::Args as AddTransactionsArgs;
pub use icrc3_example_api::add_batch_transactions::Response as AddTransactionsResponse;

// #[update(guard = "caller_is_authorized")]
// #[trace]
// async fn add_batch_transactions() -> Result<AddTransactionsResponse> {
//     let transaction = read_state(|state| state.data.create_fake_transaction());

//     let result = {
//         let mut state = take_state();
//         let result = state
//             .data
//             .icrc3
//             .add_transaction(transaction)
//             .await
//             .map_err(|e| format!("Error adding transaction: {}", e));
//         replace_state(state);
//         result
//     };

//     match result {
//         Ok(_) => Ok(()),
//         Err(e) => Err(e),
//     }
// }
