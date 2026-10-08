//! Reads the session messages of a fixture connection.

use kdx_proto::handshake::{Hello, Proof};
use kdx_proto::session::{Header, SessionCipher};
use kdx_proto::AccountKey;

/// The password of the lab's admin account.
pub const PASSWORD: &[u8] = b"lab-only-pw!1";

/// Decrypts every message after the hello and proof of one stream.
pub fn messages(stream: &[u8], cipher: &SessionCipher) -> Vec<(Header, Vec<u8>)> {
    let (_, hello_len) = Hello::decode(stream).unwrap();
    let mut at = hello_len + Proof::LEN;
    let mut out = Vec::new();
    while at < stream.len() {
        let header = cipher
            .open_data_header(stream[at..at + 16].try_into().unwrap())
            .unwrap();
        at += Header::LEN;
        let payload = cipher.open_payload(&header, &stream[at..]).unwrap();
        out.push((header, payload));
        at += header.padded_len();
    }
    assert_eq!(at, stream.len());
    out
}

/// Both directions of a connection logged in with [`PASSWORD`], client first.
#[allow(dead_code)]
pub fn session(c2s: &[u8], s2c: &[u8]) -> Vec<(Header, Vec<u8>)> {
    session_with_key(c2s, s2c, &AccountKey::from_password(PASSWORD))
}

/// Both directions of a connection authenticated with `key`, client first.
#[allow(dead_code)]
pub fn session_with_key(c2s: &[u8], s2c: &[u8], key: &AccountKey) -> Vec<(Header, Vec<u8>)> {
    let cipher = SessionCipher::new(key);
    let mut all = messages(c2s, &cipher);
    all.extend(messages(s2c, &cipher));
    all
}
