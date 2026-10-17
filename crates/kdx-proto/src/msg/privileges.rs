//! The privilege bits of accounts and account classes.

use std::ops::{BitAnd, BitOr, BitOrAssign, Not};

macro_rules! privileges {
    ($($(#[$doc:meta])* $name:ident = $bit:literal,)*) => {
        /// One privilege, numbered by its bit in [`Privileges`].
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        #[repr(u8)]
        pub enum Privilege {
            $($(#[$doc])* $name = $bit,)*
        }

        impl Privilege {
            /// Every named privilege, in bit order.
            pub const ALL: &'static [Self] = &[$(Self::$name,)*];
        }
    };
}

privileges! {
    /// Override the class lists of chats, and the limits on news and accounts.
    Override = 0,
    /// Color Overrides Class.
    ColorOverridesClass = 3,
    /// Can Login.
    CanLogin = 4,
    /// Download.
    Download = 10,
    /// Upload.
    Upload = 11,
    /// Empty Trash.
    EmptyTrash = 13,
    /// Create Aliases.
    CreateAliases = 15,
    /// Move.
    Move = 16,
    /// Rename.
    Rename = 17,
    /// Delete.
    Delete = 18,
    /// Set File/Folder Attributes.
    SetAttributes = 20,
    /// See All Volumes.
    SeeAllVolumes = 22,
    /// Has Custom Base Folder.
    HasCustomBaseFolder = 23,
    /// See All Bases.
    SeeAllBases = 24,
    /// Allow Simultaneous Downloads.
    AllowSimultaneousDownloads = 25,
    /// Ignore Speed Limits.
    IgnoreSpeedLimits = 26,
    /// View Folder Access.
    ViewFolderAccess = 27,
    /// Modify Folder Access.
    ModifyFolderAccess = 28,
    /// Join Chat.
    JoinChat = 30,
    /// Send Chat.
    SendChat = 31,
    /// Send Messages.
    SendMessages = 32,
    /// Broadcast.
    Broadcast = 33,
    /// Create Private Chat.
    CreatePrivateChat = 34,
    /// Set Chat Topic.
    SetChatTopic = 35,
    /// Join Chat On Connect.
    JoinChatOnConnect = 36,
    /// Create/Modify Chats.
    CreateModifyChats = 37,
    /// Get User Info.
    GetUserInfo = 40,
    /// Disconnect Users.
    DisconnectUsers = 41,
    /// Cannot Be Disconnected.
    CannotBeDisconnected = 42,
    /// Ban Users.
    BanUsers = 43,
    /// View Accounts.
    ViewAccounts = 50,
    /// Create Accounts.
    CreateAccounts = 51,
    /// Modify Accounts.
    ModifyAccounts = 52,
    /// Change Own Password.
    ChangeOwnPassword = 53,
    /// Protected Account.
    ProtectedAccount = 54,
    /// Use Any Name.
    UseAnyName = 60,
    /// Modify Server Settings.
    ModifyServerSettings = 61,
    /// Don't Show Greeting.
    DontShowGreeting = 62,
    /// View Server History.
    ViewServerHistory = 63,
    /// Shutdown Server.
    ShutdownServer = 64,
    /// Disable Encryption.
    DisableEncryption = 65,
    /// View Display.
    ViewDisplay = 66,
    /// Control Display.
    ControlDisplay = 67,
    /// View News.
    ViewNews = 70,
    /// Post News.
    PostNews = 71,
    /// Delete News Messages.
    DeleteNewsMessages = 72,
    /// Create/Modify Newsgroups.
    CreateModifyNewsgroups = 73,
    /// See Logins In News.
    SeeLoginsInNews = 74,
    /// View Processes.
    ViewProcesses = 80,
    /// Terminate Processes.
    TerminateProcesses = 81,
    /// Launch Programs.
    LaunchPrograms = 82,
}

impl Privilege {
    /// The bit number, where bit 0 is the high bit of the first byte.
    pub const fn bit(self) -> u32 {
        self as u32
    }

    const fn mask(self) -> u128 {
        1 << 127 >> self.bit()
    }
}

/// A set of privileges, sent as a big-endian u128 with bit 0 as the most significant bit.
///
/// Bits without a [`Privilege`] name, such as bit 2, are kept so that a set re-encodes as received.
#[derive(Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct Privileges(u128);

impl Privileges {
    /// Encoded length.
    pub const LEN: usize = 16;

    /// The empty set.
    pub const NONE: Self = Self(0);

    /// The set of the given privileges.
    pub const fn of(privileges: &[Privilege]) -> Self {
        let mut bits = 0;
        let mut i = 0;
        while i < privileges.len() {
            bits |= privileges[i].mask();
            i += 1;
        }
        Self(bits)
    }

    /// Whether the set holds `privilege`.
    pub const fn contains(self, privilege: Privilege) -> bool {
        self.0 & privilege.mask() != 0
    }

    /// Adds `privilege`.
    pub const fn insert(&mut self, privilege: Privilege) {
        self.0 |= privilege.mask();
    }

    /// Removes `privilege`.
    pub const fn remove(&mut self, privilege: Privilege) {
        self.0 &= !privilege.mask();
    }

    /// The named privileges in the set, in bit order.
    pub fn iter(self) -> impl Iterator<Item = Privilege> {
        Privilege::ALL
            .iter()
            .copied()
            .filter(move |&p| self.contains(p))
    }

    /// The set bits that have no name, as bits of the u128.
    pub fn unnamed(self) -> u128 {
        self.0 & !Self::from_iter(Privilege::ALL.iter().copied()).0
    }

    /// Decodes the 16 bytes at `at`; the caller checks the length.
    pub(crate) fn decode_at(buf: &[u8], at: usize) -> Self {
        Self::decode(buf[at..at + Self::LEN].try_into().expect("16 bytes"))
    }

    /// Decodes the wire form.
    pub const fn decode(bytes: [u8; Self::LEN]) -> Self {
        Self(u128::from_be_bytes(bytes))
    }

    /// Encodes the wire form.
    pub const fn encode(self) -> [u8; Self::LEN] {
        self.0.to_be_bytes()
    }
}

impl FromIterator<Privilege> for Privileges {
    fn from_iter<I: IntoIterator<Item = Privilege>>(iter: I) -> Self {
        let mut set = Self::NONE;
        set.extend(iter);
        set
    }
}

impl Extend<Privilege> for Privileges {
    fn extend<I: IntoIterator<Item = Privilege>>(&mut self, iter: I) {
        iter.into_iter().for_each(|p| self.insert(p));
    }
}

impl From<Privilege> for Privileges {
    fn from(privilege: Privilege) -> Self {
        Self(privilege.mask())
    }
}

impl BitOr for Privileges {
    type Output = Self;
    fn bitor(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }
}

impl BitOrAssign for Privileges {
    fn bitor_assign(&mut self, other: Self) {
        self.0 |= other.0;
    }
}

impl BitAnd for Privileges {
    type Output = Self;
    fn bitand(self, other: Self) -> Self {
        Self(self.0 & other.0)
    }
}

impl Not for Privileges {
    type Output = Self;
    fn not(self) -> Self {
        Self(!self.0)
    }
}

impl std::fmt::Debug for Privileges {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut set = f.debug_set();
        set.entries(self.iter());
        let unnamed = self.unnamed();
        if unnamed != 0 {
            set.entry(&format_args!("unnamed {unnamed:#x}"));
        }
        set.finish()
    }
}

impl std::fmt::LowerHex for Privileges {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:032x}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::{Privilege, Privileges};

    #[test]
    fn bit_0_is_the_first_byte_high_bit() {
        let p = Privileges::decode([0x80, 0, 0, 0, 0x08, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
        assert!(p.contains(Privilege::Override));
        assert!(p.contains(Privilege::JoinChatOnConnect));
        assert!(!p.contains(Privilege::CanLogin));
        assert_eq!(
            p.iter().collect::<Vec<_>>(),
            [Privilege::Override, Privilege::JoinChatOnConnect]
        );
    }

    #[test]
    fn unnamed_bits_survive() {
        let mut bytes = [0; 16];
        bytes[0] = 0x28;
        let p = Privileges::decode(bytes);
        assert_eq!(p.iter().collect::<Vec<_>>(), [Privilege::CanLogin]);
        assert_eq!(p.unnamed(), 1 << 127 >> 2);
        assert_eq!(p.encode(), bytes);
    }
}
