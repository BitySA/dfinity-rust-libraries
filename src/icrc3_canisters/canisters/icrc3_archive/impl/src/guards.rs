use crate::state::read_state;

pub fn caller_is_authorized() -> Result<(), String> {
    if read_state(|state| state.is_caller_authorized()) {
        Ok(())
    } else {
        Err("Caller is not an authorized principal".to_string())
    }
}

pub fn caller_is_main_canister() -> Result<(), String> {
    if read_state(|state| state.is_caller_main_canister()) {
        Ok(())
    } else {
        Err("Caller is not an admin principal".to_string())
    }
}

pub fn caller_is_main_canister_or_authorized() -> Result<(), String> {
    if read_state(|state| state.is_caller_main_canister() || state.is_caller_authorized()) {
        Ok(())
    } else {
        Err("Caller is not an admin principal".to_string())
    }
}
