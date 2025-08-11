use bity_ic_types::{CanisterId, Cycles};
use ic_cdk::call::CallResult;
use ic_cdk::management_canister;
use ic_management_canister_types::CanisterIdRecord;
use tracing::{error, info};

pub async fn deposit_cycles(canister_id: CanisterId, amount: Cycles) -> CallResult<()> {
    if let Err(error) = ic_cdk::management_canister::deposit_cycles(
        &CanisterIdRecord { canister_id },
        amount.into(),
    )
    .await
    {
        error!(
            %canister_id,
            error = %error,
            "Error calling 'deposit_cycles'"
        );
        Err(error)
    } else {
        info!(
            %canister_id,
            amount,
            "Topped up canister with cycles"
        );
        Ok(())
    }
}

pub async fn get_cycles_balance(canister_id: CanisterId) -> Result<u64, String> {
    match management_canister::canister_status(&CanisterIdRecord { canister_id }).await {
        Ok(res) => {
            let as_u64: Result<u64, _> = res.cycles.0.try_into();
            match as_u64 {
                Ok(amount) => Ok(amount),
                Err(e) => Err(format!("{e:?}")),
            }
        }
        Err(e) => Err(format!("{e:?}")),
    }
}
