use crate::wasms::ICRC3;
use candid::encode_one;
use candid::Principal;
use pocket_ic::PocketIc;

pub fn setup_icrc3_canister(
    pic: &mut PocketIc,
    icrc3_id: Principal,
    args: icrc3_example_api::Args,
    controller: Principal,
) -> Principal {
    let icrc3_wasm = include_bytes!("../../../../../wasm/icrc3_example_canister.wasm.gz").to_vec();
    pic.add_cycles(icrc3_id, 100_000_000_000_000_000_000);

    pic.set_controllers(icrc3_id, Some(controller.clone()), vec![controller.clone()])
        .unwrap();
    pic.tick();

    pic.install_canister(
        icrc3_id,
        icrc3_wasm,
        encode_one(args).unwrap(),
        Some(controller.clone()),
    );

    println!("setup done : icrc3_id: {:?}", icrc3_id);
    icrc3_id
}

pub fn upgrade_icrc3_canister(
    pic: &mut PocketIc,
    icrc3_canister_id: Principal,
    args: icrc3_example_api::Args,
    controller: Principal,
) {
    let icrc3_wasm = include_bytes!("../../../../../wasm/icrc3_example_canister.wasm.gz").to_vec();
    pic.add_cycles(icrc3_canister_id, 100_000_000_000_000_000);

    pic.set_controllers(
        icrc3_canister_id,
        Some(controller.clone()),
        vec![controller.clone()],
    )
    .unwrap();
    pic.tick();

    pic.upgrade_canister(
        icrc3_canister_id,
        icrc3_wasm,
        encode_one(args).unwrap(),
        Some(controller.clone()),
    )
    .unwrap();
}
