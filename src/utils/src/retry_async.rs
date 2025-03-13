use std::future::Future;

// Provides async retry functionality for operations that may fail.

/// Retries an asynchronous operation a specified number of times.
///
/// # Arguments
///
/// * `operation` - A function that returns a Future with a Result
/// * `retries` - The maximum number of retry attempts
///
/// # Returns
///
/// Returns the result of the operation if successful, or the last error encountered
///
/// # Example
///
/// ```
/// use bity_dfinity_library::utils::retry_async;
///
/// async fn fallible_operation() -> Result<i32, String> {
///     // Some operation that might fail
///     Ok(42)
/// }
///
/// async fn example() {
///     let result = retry_async(|| fallible_operation(), 3).await;
///     assert!(result.is_ok());
/// }
/// ```
pub async fn retry_async<F, Fut, T, E>(mut operation: F, retries: usize) -> Result<T, E>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T, E>>,
{
    let mut attempt = 0;
    while attempt < retries {
        attempt += 1;
        match operation().await {
            Ok(result) => {
                return Ok(result);
            }
            Err(err) => {
                if attempt >= retries {
                    return Err(err);
                }
            }
        }
    }
    unreachable!() // The code should never reach this point.
}

// fn trace(msg: &str) {
//     unsafe {
//         ic0::debug_print(msg.as_ptr() as i32, msg.len() as i32);
//     }
// }

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, rc::Rc};

    use super::*;

    #[tokio::test]
    async fn test_retry_async_works_correctly() {
        let iteration_count = Rc::new(RefCell::new(0));
        let result = retry_async(
            || async {
                let iteration_count = Rc::clone(&iteration_count);
                *iteration_count.borrow_mut() += 1;
                let success = true;
                if success {
                    Ok(1)
                } else {
                    Err(0)
                }
            },
            3,
        )
        .await;
        assert_eq!(*iteration_count.borrow(), 1);
        assert_eq!(result, Ok(1));

        let iteration_count = Rc::new(RefCell::new(0));
        let result = retry_async(
            || async {
                let iteration_count = Rc::clone(&iteration_count);
                *iteration_count.borrow_mut() += 1;

                let success = false;
                if success {
                    Ok(1)
                } else {
                    Err(0)
                }
            },
            3,
        )
        .await;

        assert_eq!(*iteration_count.borrow(), 3);
        assert_eq!(result, Err(0));

        let iteration_count = Rc::new(RefCell::new(0));
        let result = retry_async(
            || async {
                let iteration_count = Rc::clone(&iteration_count);
                *iteration_count.borrow_mut() += 1;

                if *iteration_count.borrow() == 2 {
                    return Ok(1);
                }

                let success = false;
                if success {
                    Ok(2)
                } else {
                    Err(0)
                }
            },
            3,
        )
        .await;

        assert_eq!(*iteration_count.borrow(), 2);
        assert_eq!(result, Ok(1));
    }
}
