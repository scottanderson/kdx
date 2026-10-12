//! Wire format of the Haxial KDX protocol.
//!
//! This crate covers the connection handshake (which is also the login), the
//! session encryption, and message framing. It does no I/O. The protocol is
//! described in `docs/protocol/` at the repository root.
//!
//! A connection goes through these steps:
//!
//! 1. Each side sends a [`Hello`](handshake::Hello).
//! 2. Each side sends a [`Proof`](handshake::Proof) computed from the
//!    shared [`AccountKey`], the peer's nonce and the [`ChallengeTable`].
//! 3. Both sides exchange messages sealed with a [`SessionCipher`](session::SessionCipher).
//!
//! Servers register with a tracker over UDP with a [`Registration`](registration::Registration).

pub mod account;
pub mod challenge;
pub mod handshake;
pub mod message_type;
pub mod msg;
pub mod registration;
pub mod session;
mod xor;

pub use account::AccountKey;
pub use challenge::ChallengeTable;
pub use message_type::MessageType;
