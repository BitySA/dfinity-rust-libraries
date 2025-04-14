use bity_ic_types::CanisterWasm;
use lazy_static::lazy_static;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

lazy_static! {
    pub static ref ICRC3: CanisterWasm = get_canister_wasm_from_bin("icrc3");
}

fn get_canister_wasm_from_bin(canister_name: &str) -> CanisterWasm {
    match read_file_from_relative_bin(&format!(
        "../../wasm/{canister_name}/{canister_name}_canister.wasm.gz"
    )) {
        Ok(wasm) => wasm,
        Err(err) => {
            println!(
                "Failed to read {canister_name} wasm: {err}. \n\x1b[31mRun \"./scripts/build_canister.sh {canister_name}\"\x1b[0m"
            );
            panic!()
        }
    }
}

fn get_canister_wasm(canister_name: &str) -> CanisterWasm {
    read_file_from_local_bin(&format!("{canister_name}_canister.wasm"))
}

fn get_canister_wasm_gz(canister_name: &str) -> CanisterWasm {
    read_file_from_local_bin(&format!("{canister_name}_canister.wasm.gz"))
}

fn read_file_from_local_bin(file_name: &str) -> Vec<u8> {
    let mut file_path = local_bin();
    file_path.push(file_name);

    let mut file = File::open(&file_path)
        .unwrap_or_else(|_| panic!("Failed to open file: {}", file_path.to_str().unwrap()));
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes).expect("Failed to read file");
    bytes
}

pub fn local_bin() -> PathBuf {
    let mut file_path = PathBuf::from(
        std::env::var("CARGO_MANIFEST_DIR")
            .expect("Failed to read CARGO_MANIFEST_DIR env variable"),
    );
    file_path.push("wasms");
    file_path
}

fn read_file_from_relative_bin(file_path: &str) -> Result<Vec<u8>, std::io::Error> {
    // Open the wasm file
    let mut file = File::open(file_path)?;

    // Read the contents of the file into a vector
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    Ok(buffer)
}
