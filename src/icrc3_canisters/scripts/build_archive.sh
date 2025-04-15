#!/bin/bash

cargo rustc --crate-type=cdylib --target wasm32-unknown-unknown --target-dir "./canisters/icrc3_archive/target" --release --locked -p icrc3_archive &&
ic-wasm "./canisters/icrc3_archive/target/wasm32-unknown-unknown/release/icrc3_archive.wasm" -o "./canisters/icrc3_archive/target/wasm32-unknown-unknown/release/icrc3_archive.wasm" shrink &&
ic-wasm "./canisters/icrc3_archive/target/wasm32-unknown-unknown/release/icrc3_archive.wasm" -o "./canisters/icrc3_archive/target/wasm32-unknown-unknown/release/icrc3_archive.wasm" optimize --inline-functions-with-loops O3 &&
gzip --no-name -9 -v -c "./canisters/icrc3_archive/target/wasm32-unknown-unknown/release/icrc3_archive.wasm" > "./canisters/icrc3_archive/target/wasm32-unknown-unknown/release/icrc3_archive_canister.wasm.gz" &&
gzip -v -t "./canisters/icrc3_archive/target/wasm32-unknown-unknown/release/icrc3_archive_canister.wasm.gz" &&
mv "./canisters/icrc3_archive/target/wasm32-unknown-unknown/release/icrc3_archive_canister.wasm.gz" "./wasm/icrc3_archive_canister.wasm.gz"
./scripts/generate_did.sh icrc3_archive
