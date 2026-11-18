//! Platform integration for NOIR Player: standard directories, filesystem
//! helpers and windowing utilities.

pub mod filesystem;
pub mod paths;
pub mod single_instance;
pub mod windowing;

pub use paths::PlatformPaths;
