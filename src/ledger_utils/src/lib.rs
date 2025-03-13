//! Module for handling Internet Computer ledger operations and account management.
//!
//! This module provides utilities for working with the Internet Computer's ledger system,
//! including account identifier computation, subaccount management, and conversion between
//! different account formats.
//!
//! # Example
//! ```
//! use candid::Principal;
//! use ic_ledger_types::AccountIdentifier;
//!
//! let principal = Principal::from_text("rrkah-fqaaa-aaaaa-aaaaq-cai").unwrap();
//! let account_id = principal_to_legacy_account_id(principal, None);
//! ```

use candid::Principal;
use ic_ledger_types::{AccountIdentifier, Subaccount, DEFAULT_SUBACCOUNT};
use icrc_ledger_types::icrc1::account::Account;
use sha2::{Digest, Sha256};

/// Computes a neuron staking subaccount using SHA-256 hashing.
///
/// This function generates a deterministic subaccount for neuron staking by hashing
/// the controller's principal and a nonce value.
///
/// # Arguments
/// * `controller` - The principal ID of the controller
/// * `nonce` - A unique value to ensure different subaccounts for the same controller
///
/// # Returns
/// A 32-byte array representing the computed subaccount
pub fn compute_neuron_staking_subaccount_bytes(controller: Principal, nonce: u64) -> [u8; 32] {
    const DOMAIN: &[u8] = b"neuron-stake";
    const DOMAIN_LENGTH: [u8; 1] = [0x0c];

    let mut hasher = Sha256::new();
    hasher.update(DOMAIN_LENGTH);
    hasher.update(DOMAIN);
    hasher.update(controller.as_slice());
    hasher.update(nonce.to_be_bytes());
    hasher.finalize().into()
}

/// Converts an ICRC-1 account to a legacy account identifier.
///
/// This function converts an ICRC-1 account format to the legacy account identifier format,
/// handling the conversion of subaccounts appropriately.
///
/// # Arguments
/// * `icrc_account` - The ICRC-1 account to convert
///
/// # Returns
/// The corresponding legacy account identifier
pub fn icrc_account_to_legacy_account_id(icrc_account: Account) -> AccountIdentifier {
    let subaccount: Subaccount = icrc_account
        .subaccount
        .map_or(DEFAULT_SUBACCOUNT, |s| Subaccount(s));
    AccountIdentifier::new(&icrc_account.owner, &subaccount)
}

/// Creates a legacy account identifier from a principal and optional subaccount.
///
/// This function creates a legacy account identifier using the provided principal
/// and an optional subaccount. If no subaccount is provided, the default subaccount is used.
///
/// # Arguments
/// * `principal` - The principal ID to use
/// * `subaccount` - An optional subaccount to use
///
/// # Returns
/// The corresponding legacy account identifier
pub fn principal_to_legacy_account_id(
    principal: Principal,
    subaccount: Option<Subaccount>,
) -> AccountIdentifier {
    AccountIdentifier::new(&principal, &subaccount.unwrap_or(DEFAULT_SUBACCOUNT))
}

#[cfg(test)]
mod tests {
    use candid::Principal;
    use icrc_ledger_types::icrc1::account::Account;

    use crate::icrc_account_to_legacy_account_id;

    #[test]
    fn convert_icrc_account_to_legacy_account_id() {
        let icrc_account = Account {
            owner: Principal::from_text(
                "465sx-szz6o-idcax-nrjhv-hprrp-qqx5e-7mqwr-wadib-uo7ap-lofbe-dae",
            )
            .unwrap(),
            subaccount: None,
        };
        let result = icrc_account_to_legacy_account_id(icrc_account);

        let expected_result =
            "aacba041bbce2b03c66307a68ca2d5a704a1f87397694a1292d89ce757136f11".to_string();

        assert_eq!(result.to_hex(), expected_result)
    }

    #[test]
    fn convert_icrc_account_to_legacy_account_id_with_subaccount() {
        let icrc_account = Account {
            owner: Principal::from_text(
                "465sx-szz6o-idcax-nrjhv-hprrp-qqx5e-7mqwr-wadib-uo7ap-lofbe-dae",
            )
            .unwrap(),
            subaccount: Some([
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 1,
            ]),
        };
        let result = icrc_account_to_legacy_account_id(icrc_account);

        let expected_result =
            "2fab56f6af866bd4580c8bdf821849d470d5d0af6a671191c602e0c434b5e55c".to_string();

        assert_eq!(result.to_hex(), expected_result)
    }
}
