use ic_cdk::api::management_canister::main::raw_rand;
use std::time::Duration;

// Provides random number generation utilities for Internet Computer canisters.

/// Generates a random 64-bit nonce.
///
/// # Returns
///
/// Returns a Result containing a u64 random number, or an error message if generation fails.
pub async fn generate_rand_nonce() -> Result<u64, String> {
    generate_rand_byte_array().await.map(u64::from_be_bytes)
}

/// Generates a random 8-byte array.
///
/// # Returns
///
/// Returns a Result containing an 8-byte array, or an error message if generation fails.
pub async fn generate_rand_byte_array() -> Result<[u8; 8], String> {
    match raw_rand().await {
        Ok((random_bytes,)) => {
            let bytes_array: Result<[u8; 8], _> = random_bytes[0..8].try_into();

            match bytes_array {
                Ok(bytes) => Ok(bytes),
                Err(err) => Err(format!("Initialising slicing byte array: {}", err)),
            }
        }
        Err(err) => Err(format!("Random bytes generation error: {:?}", err)),
    }
}

/// Generates a random delay duration up to the specified maximum interval.
///
/// # Arguments
///
/// * `max_interval` - The maximum duration for the random delay
///
/// # Returns
///
/// Returns a Result containing the random Duration, or an error message if generation fails.
pub async fn generate_random_delay(max_interval: Duration) -> Result<Duration, String> {
    let random_nonce = generate_rand_nonce().await?;

    let random_delay_nanos = random_nonce % (max_interval.as_nanos() as u64);

    Ok(Duration::from_nanos(random_delay_nanos))
}
