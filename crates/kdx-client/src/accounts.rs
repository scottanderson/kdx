//! Account and class requests: 502 to 551.

use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{bail, Result};
use kdx_proto::msg::admin::{Account, AccountClass, AccountClassIndex, AccountListEntry};
use kdx_proto::msg::{Privilege, Privileges};
use kdx_proto::session::MessageType;
use kdx_proto::AccountKey;

use crate::client::pad4;
use crate::Client;

/// Milliseconds from 1900-01-01 to the Unix epoch.
const UNIX_EPOCH_1900_MS: u64 = 2_208_988_800_000;

/// The current time as the protocol counts it: milliseconds since 1900.
fn now_1900_ms() -> u64 {
    let since_unix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0);
    since_unix + UNIX_EPOCH_1900_MS
}

/// An account record for 507 Create Account or 511 Set Account.
///
/// `password: None` keeps the stored key, which is all zero on the wire.
pub fn new_account(
    login: &[u8],
    name: &[u8],
    password: Option<&[u8]>,
    class_index: u8,
    privileges: Privileges,
    privilege_mask: Privileges,
) -> Account {
    let now = now_1900_ms();
    Account {
        privileges,
        privilege_mask,
        key: password.map_or([0; 32], |p| AccountKey::from_password(p).0),
        created: now,
        modified: now,
        last_login: 0,
        class_index,
        login: login.to_vec(),
        name: name.to_vec(),
        ..Account::default()
    }
}

/// A privilege's name in kebab-case: `CanLogin` is `can-login`.
pub fn privilege_slug(privilege: Privilege) -> String {
    let mut out = String::new();
    for (i, ch) in format!("{privilege:?}").chars().enumerate() {
        if ch.is_uppercase() && i > 0 {
            out.push('-');
        }
        out.push(ch.to_ascii_lowercase());
    }
    out
}

/// Parses `all`, `none`, or comma-separated privilege slugs; a slug is a
/// name from [`privilege_slug`] or `bit<N>`. Case and spaces are ignored.
pub fn parse_privileges(spec: &str) -> Result<Privileges> {
    let spec = spec.trim();
    if spec.eq_ignore_ascii_case("all") {
        return Ok(!Privileges::NONE);
    }
    if spec.is_empty() || spec.eq_ignore_ascii_case("none") {
        return Ok(Privileges::NONE);
    }
    let mut out = Privileges::NONE;
    for word in spec.split(',') {
        let word = word.trim().to_ascii_lowercase();
        let by_name = Privilege::ALL
            .iter()
            .copied()
            .find(|p| privilege_slug(*p) == word);
        let by_bit = || {
            let bit: u32 = word.strip_prefix("bit")?.parse().ok()?;
            Privilege::ALL.iter().copied().find(|p| p.bit() == bit)
        };
        match by_name.or_else(by_bit) {
            Some(privilege) => out.insert(privilege),
            None => bail!("unknown privilege {word:?}"),
        }
    }
    Ok(out)
}

/// Formats a set as comma-separated slugs, or `none`.
pub fn format_privileges(privileges: &Privileges) -> String {
    let names: Vec<String> = privileges.iter().map(privilege_slug).collect();
    if names.is_empty() {
        "none".to_string()
    } else {
        names.join(",")
    }
}

impl Client {
    /// 512 Get Account List.
    pub async fn accounts(&mut self) -> Result<Vec<AccountListEntry>> {
        let reply = self.request(MessageType::GET_ACCOUNT_LIST, &[]).await?;
        Ok(AccountListEntry::decode_list(&reply)?)
    }

    /// 509 Get Account.
    pub async fn account(&mut self, login: &[u8]) -> Result<Account> {
        let reply = self.request(MessageType::GET_ACCOUNT, login).await?;
        Ok(Account::decode(&reply)?)
    }

    /// 507 Create Account; the server answers with a success Status.
    pub async fn create_account(&mut self, account: &Account) -> Result<()> {
        let record = Account {
            padding: Vec::new(),
            ..account.clone()
        };
        self.request(MessageType::CREATE_ACCOUNT, &record.encode()?)
            .await?;
        Ok(())
    }

    /// 511 Set Account; the server answers with a success Status.
    ///
    /// On the wire `editor` names the account to change; this sets it to
    /// `account.login`, since a 510 Account Info carries the last editor
    /// there, which the server would read as a rename. Use
    /// [`Self::rename_account`] to rename.
    pub async fn set_account(&mut self, account: &Account) -> Result<()> {
        let login = account.login.clone();
        self.rename_account(&login, account).await
    }

    /// 511 Set Account with `editor` = `current`, the stored login, and
    /// `account.login` the new one.
    pub async fn rename_account(&mut self, current: &[u8], account: &Account) -> Result<()> {
        let record = Account {
            editor: current.to_vec(),
            padding: Vec::new(),
            ..account.clone()
        };
        self.request(MessageType::SET_ACCOUNT, &record.encode()?)
            .await?;
        Ok(())
    }

    /// 508 Delete Account; every user gets 515 Account Deleted.
    pub async fn delete_account(&mut self, login: &[u8]) -> Result<()> {
        self.send_checked(MessageType::DELETE_ACCOUNT, login).await
    }

    /// 550 Change Own Password.
    pub async fn change_password(&mut self, password: &[u8]) -> Result<()> {
        let key = AccountKey::from_password(password);
        self.request(MessageType::CHANGE_OWN_PASSWORD, &key.0)
            .await?;
        Ok(())
    }

    /// 505 Get Account Class List.
    pub async fn account_classes(&mut self) -> Result<Vec<AccountClass>> {
        let reply = self
            .request(MessageType::GET_ACCOUNT_CLASS_LIST, &[])
            .await?;
        Ok(AccountClass::decode_list(&reply)?)
    }

    /// 502 Get Account Class.
    pub async fn class(&mut self, index: u8) -> Result<AccountClass> {
        let reply = self
            .request(
                MessageType::GET_ACCOUNT_CLASS,
                &AccountClassIndex(index).encode(),
            )
            .await?;
        Ok(AccountClass::decode_info(&reply)?)
    }

    /// 504 Set Account Class; the server answers with a success Status.
    pub async fn set_class(&mut self, class: &AccountClass) -> Result<()> {
        let record = AccountClass {
            padding: Vec::new(),
            ..class.clone()
        };
        self.request(MessageType::SET_ACCOUNT_CLASS, &pad4(record.encode_set()?))
            .await?;
        Ok(())
    }
}
