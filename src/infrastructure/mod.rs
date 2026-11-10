//! Processes and files: git, config/history storage, folder scanning.

pub mod backend;
pub mod git;
pub mod probe;
pub mod scan;
pub mod store;
#[cfg(test)]
pub mod testutil;
