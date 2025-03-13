//! Provides version management utilities for Internet Computer canisters.
//!
//! This module implements a semantic versioning system through the `BuildVersion` struct,
//! following the MAJOR.MINOR.PATCH format.

use candid::CandidType;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::str::FromStr;

/// Represents a semantic version number following the MAJOR.MINOR.PATCH format.
///
/// # Examples
///
/// ```
/// use types::BuildVersion;
///
/// let version = BuildVersion::new(1, 2, 3);
/// assert_eq!(version.to_string(), "1.2.3");
///
/// let parsed = "1.2.3".parse::<BuildVersion>().unwrap();
/// assert_eq!(version, parsed);
/// ```
#[derive(
    CandidType, Serialize, Deserialize, Clone, Copy, Debug, Default, Ord, PartialOrd, Eq, PartialEq,
)]
pub struct BuildVersion {
    /// Major version number, incremented for incompatible API changes
    pub major: u32,
    /// Minor version number, incremented for backwards-compatible functionality additions
    pub minor: u32,
    /// Patch version number, incremented for backwards-compatible bug fixes
    pub patch: u32,
}

impl BuildVersion {
    /// Creates a new BuildVersion with the specified version numbers.
    ///
    /// # Arguments
    ///
    /// * `major` - The major version number
    /// * `minor` - The minor version number
    /// * `patch` - The patch version number
    pub fn new(major: u32, minor: u32, patch: u32) -> BuildVersion {
        BuildVersion {
            major,
            minor,
            patch,
        }
    }

    /// Returns the minimum possible version (0.0.0).
    pub fn min() -> BuildVersion {
        BuildVersion::default()
    }
}

impl Display for BuildVersion {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("{}.{}.{}", self.major, self.minor, self.patch))
    }
}

impl FromStr for BuildVersion {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<_> = s.split('.').collect();
        if parts.len() != 3 {
            return Err(format!("Unable to parse version: {s}"));
        }

        let major = u32::from_str(parts[0]).map_err(|e| e.to_string())?;
        let minor = u32::from_str(parts[1]).map_err(|e| e.to_string())?;
        let patch = u32::from_str(parts[2]).map_err(|e| e.to_string())?;

        Ok(BuildVersion {
            major,
            minor,
            patch,
        })
    }
}
