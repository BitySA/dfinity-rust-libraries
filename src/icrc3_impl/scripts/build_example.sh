#!/bin/bash

cargo rustc --crate-type=cdylib --target wasm32-unknown-unknown --target-dir "./backend/canisters/icrc3_example/target" --release --locked -p icrc3-example &&
ic-wasm "./backend/canisters/icrc3_example/target/wasm32-unknown-unknown/release/icrc3_example.wasm" -o "./backend/canisters/icrc3_example/target/wasm32-unknown-unknown/release/icrc3_example.wasm" shrink &&
ic-wasm "./backend/canisters/icrc3_example/target/wasm32-unknown-unknown/release/icrc3_example.wasm" -o "./backend/canisters/icrc3_example/target/wasm32-unknown-unknown/release/icrc3_example.wasm" optimize --inline-functions-with-loops O3 &&
gzip --no-name -9 -v -c "./backend/canisters/icrc3_example/target/wasm32-unknown-unknown/release/icrc3_example.wasm" > "./backend/canisters/icrc3_example/target/wasm32-unknown-unknown/release/icrc3_example_canister.wasm.gz" &&
gzip -v -t "./backend/canisters/icrc3_example/target/wasm32-unknown-unknown/release/icrc3_example_canister.wasm.gz" &&
mv "./backend/canisters/icrc3_example/target/wasm32-unknown-unknown/release/icrc3_example_canister.wasm.gz" "./wasm/icrc3_example_canister.wasm.gz"
./scripts/generate_did.sh icrc3_example