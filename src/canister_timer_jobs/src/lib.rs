//! Module for managing timer-based jobs in Internet Computer canisters.
//!
//! This module provides functionality for scheduling and managing recurring jobs
//! in Internet Computer canisters using the timer API. It supports job scheduling,
//! cancellation, and monitoring of job execution.
//!
//! # Example
//! ```
//! use bity_ic_canister_timer_jobs::TimerJobs;
//! use std::time::Duration;
//!
//! let mut jobs = TimerJobs::new();
//! jobs.schedule("my_job", Duration::from_secs(60), || {
//!     println!("Job executed!");
//! });
//! ```

use bity_ic_utils::env::Environment;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use crate::timer_manager::TimerManager;

pub mod timer_manager;

/// A collection of timer-based jobs managed together.
///
/// This struct provides a way to manage multiple timer jobs in a canister,
/// allowing for scheduling, cancellation, and monitoring of job execution.
///
/// # Type Parameters
/// * `J` - The type of the job function
/// * `R` - The return type of the job function
///
/// # Examples
/// ```
/// use bity_ic_canister_timer_jobs::TimerJobs;
/// use std::time::Duration;
///
/// let jobs = TimerJobs::new();
/// ```
pub struct TimerJobs<J, R>
where
    J: Fn() -> R,
    R: 'static,
{
    jobs: BTreeMap<String, TimerManager<J, R>>,
}

type JobWrapper<J> = Rc<RefCell<Option<J>>>;

impl<J, R> TimerJobs<J, R>
where
    J: Fn() -> R,
    R: 'static,
{
    /// Returns an iterator over all timer managers in the collection.
    ///
    /// # Returns
    /// An iterator yielding references to `TimerManager` instances
    pub fn iter(&self) -> impl Iterator<Item = &TimerManager<J, R>> {
        self.jobs.values()
    }

    /// Returns the number of jobs in the collection.
    ///
    /// # Returns
    /// The number of jobs currently managed
    pub fn len(&self) -> usize {
        self.jobs.len()
    }

    /// Returns whether the collection is empty.
    ///
    /// # Returns
    /// `true` if there are no jobs in the collection
    pub fn is_empty(&self) -> bool {
        self.jobs.is_empty()
    }
}

/// Trait defining the behavior of a job that can be executed.
///
/// This trait must be implemented by any type that represents a job
/// to be executed by the timer system.
///
/// # Examples
/// ```
/// use bity_ic_canister_timer_jobs::Job;
///
/// struct MyJob;
///
/// impl Job for MyJob {
///     fn execute(self) {
///         println!("Executing my job!");
///     }
/// }
/// ```
pub trait Job: 'static {
    /// Executes the job.
    ///
    /// This method is called when the job's timer triggers.
    fn execute(self);
}
