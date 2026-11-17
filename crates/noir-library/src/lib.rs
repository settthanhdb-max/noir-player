//! Media library, scanner, history and playlist persistence for NOIR Player.

pub mod database;
pub mod history;
pub mod library;
pub mod m3u;
pub mod scanner;

pub use database::Database;
pub use library::{LibraryScanUpdate, LibraryService};
pub use scanner::LibraryScanner;
