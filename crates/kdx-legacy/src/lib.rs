//! The original programs' files: `Accounts.dat`, `News.dat`, `Catalog.dat`,
//! the `.stg` settings of server, client and tracker, `.hap` themes, history
//! and news archive text, and the sound list names under `Bases/`.
//!
//! Every reader keeps the bytes it does not understand, so that
//! `encode(decode(file)) == file`; see `docs/formats/`. The wire structs the
//! files embed come from `kdx-proto`; times are [`time::Time1900`].

pub mod accounts;
pub mod bases;
pub mod catalog;
pub mod civil;
pub mod client_stg;
pub mod error;
pub mod history;
pub mod news;
pub mod news_archive;
pub mod scramble;
pub mod server_stg;
pub mod store;
pub mod time;
pub mod tracker_stg;
pub mod wire;

pub use accounts::AccountsFile;
pub use bases::{format_base_name, parse_base_name, BaseName, BaseTag};
pub use error::{LegacyError, Result};
pub use kdx_proto::msg::server_info::ServerSettings;
pub use store::{LegacyDir, ServerFile};
pub use time::Time1900;
