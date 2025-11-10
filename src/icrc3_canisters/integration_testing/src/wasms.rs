use bity_ic_types::CanisterWasm;
use lazy_static::lazy_static;
use std::fs::File;
use std::io::Read;

lazy_static! {
    pub static ref ICRC3: CanisterWasm = get_canister_wasm_from_bin("icrc3");
}

fn get_canister_wasm_from_bin(canister_name: &str) -> CanisterWasm {
    println!("Reading {canister_name} wasm from ./wasm/{canister_name}_canister.wasm.gz");
    match read_file_from_relative_bin(&format!("./wasm/{canister_name}_canister.wasm.gz")) {
        Ok(wasm) => wasm,
        Err(err) => {
            println!(
                "Failed to read {canister_name} wasm: {err}. \n\x1b[31mRun \"./scripts/build_canister.sh {canister_name}\"\x1b[0m"
            );
            panic!()
        }
    }
}

fn read_file_from_relative_bin(file_path: &str) -> Result<Vec<u8>, std::io::Error> {
    // Open the wasm file
    let mut file = File::open(file_path)?;

    // Read the contents of the file into a vector
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    Ok(buffer)
}
