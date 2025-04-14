#!/bin/bash

cargo rustc --crate-type=cdylib --target wasm32-unknown-unknown --target-dir "./backend/canisters/icrc3_archive/target" --release --locked -p icrc3_archive &&
ic-wasm "./backend/canisters/icrc3_archive/target/wasm32-unknown-unknown/release/icrc3_archive.wasm" -o "./backend/canisters/icrc3_archive/target/wasm32-unknown-unknown/release/icrc3_archive.wasm" shrink &&
ic-wasm "./backend/canisters/icrc3_archive/target/wasm32-unknown-unknown/release/icrc3_archive.wasm" -o "./backend/canisters/icrc3_archive/target/wasm32-unknown-unknown/release/icrc3_archive.wasm" optimize --inline-functions-with-loops O3 &&
gzip --no-name -9 -v -c "./backend/canisters/icrc3_archive/target/wasm32-unknown-unknown/release/icrc3_archive.wasm" > "./backend/canisters/icrc3_archive/target/wasm32-unknown-unknown/release/icrc3_archive_canister.wasm.gz" &&
gzip -v -t "./backend/canisters/icrc3_archive/target/wasm32-unknown-unknown/release/icrc3_archive_canister.wasm.gz" &&
mv "./backend/canisters/icrc3_archive/target/wasm32-unknown-unknown/release/icrc3_archive_canister.wasm.gz" "./wasm/icrc3_archive_canister.wasm.gz"
./scripts/generate_did.sh icrc3_archive
