#!/bin/bash

cargo rustc --crate-type=cdylib --target wasm32-unknown-unknown --target-dir "./canisters/icrc3_example/target" --release --locked -p icrc3-example &&
ic-wasm "./canisters/icrc3_example/target/wasm32-unknown-unknown/release/icrc3_example.wasm" -o "./canisters/icrc3_example/target/wasm32-unknown-unknown/release/icrc3_example.wasm" shrink &&
ic-wasm "./canisters/icrc3_example/target/wasm32-unknown-unknown/release/icrc3_example.wasm" -o "./canisters/icrc3_example/target/wasm32-unknown-unknown/release/icrc3_example.wasm" optimize --inline-functions-with-loops O3 &&
gzip --no-name -9 -v -c "./canisters/icrc3_example/target/wasm32-unknown-unknown/release/icrc3_example.wasm" > "./canisters/icrc3_example/target/wasm32-unknown-unknown/release/icrc3_example_canister.wasm.gz" &&
gzip -v -t "./canisters/icrc3_example/target/wasm32-unknown-unknown/release/icrc3_example_canister.wasm.gz" &&
mv "./canisters/icrc3_example/target/wasm32-unknown-unknown/release/icrc3_example_canister.wasm.gz" "./wasm/icrc3_example_canister.wasm.gz"
./scripts/generate_did.sh icrc3_example