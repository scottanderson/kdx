//! Message type numbers and their titles.

/// A message type number.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MessageType(pub u32);

macro_rules! message_types {
    ($($name:ident = $num:literal, $title:literal;)*) => {
        impl MessageType {
            $(
                #[doc = concat!(stringify!($num), " ", $title)]
                pub const $name: Self = Self($num);
            )*

            /// The type's title, from the server's name table.
            ///
            /// The server has no name for 1 Status; that title is this project's.
            pub const fn title(self) -> Option<&'static str> {
                match self.0 {
                    $($num => Some($title),)*
                    _ => None,
                }
            }
        }
    };
}

message_types! {
    STATUS = 1, "Status";
    SET_IDENTITY = 100, "Set Identity";
    USER_LIST = 101, "User List";
    USER_CHANGED = 102, "User Changed";
    USER_JOINED = 103, "User Joined";
    USER_DEPARTED = 104, "User Departed";
    SERVER_INFO = 105, "Server Info";
    SERVER_ICON = 106, "Server Icon";
    SERVER_GREETING = 107, "Server Greeting";
    PING = 108, "Ping";
    PONG = 109, "Pong";
    SET_MINI_USER_ICON = 110, "Set Mini User Icon";
    MINI_USER_ICON_CHANGED = 111, "Mini User Icon Changed";
    GET_EXTRA_SERVER_INFO = 113, "Get Extra Server Info";
    EXTRA_SERVER_INFO = 114, "Extra Server Info";
    GET_MY_ADDRESS = 115, "Get My Address";
    YOUR_ADDRESS = 116, "Your Address";
    SKPK = 117, "SKPK";
    OPEN_CONNECTION_MONITOR = 140, "Open Connection Monitor";
    CLOSE_CONNECTION_MONITOR = 141, "Close Connection Monitor";
    GET_CONNECTION_MONITOR_INFO = 142, "Get Connection Monitor Info";
    CONNECTION_MONITOR_INFO = 143, "Connection Monitor Info";
    DISCONNECT_USER = 150, "Disconnect User";
    DISCONNECT_MESSAGE = 151, "Disconnect Message";
    GET_USER_INFO = 152, "Get User Info";
    USER_INFO = 153, "User Info";
    SEND_BROADCAST_MESSAGE = 154, "Send Broadcast Message";
    RCV_BROADCAST_MESSAGE = 155, "Rcv Broadcast Message";
    GET_SERVER_SETTINGS = 170, "Get Server Settings";
    SERVER_SETTINGS_INFO = 171, "Server Settings Info";
    SET_SERVER_SETTINGS = 172, "Set Server Settings";
    SET_SERVER_SETTINGS_DONE = 173, "Set Server Settings Done";
    SHUTDOWN_SERVER = 174, "Shutdown Server";
    UPGRADE_SERVER_SOFTWARE = 175, "Upgrade Server Software";
    SERVER_HISTORY_JOIN = 180, "Server History Join";
    SERVER_HISTORY_JOIN_REPLY = 181, "Server History Join (R)";
    SERVER_HISTORY_LEAVE = 182, "Server History Leave";
    SERVER_HISTORY_ADDED = 183, "Server History Added";
    HISTORY_LOGIN = 184, "Login";
    HISTORY_DISCONNECTED = 185, "Disconnected";
    HISTORY_ABORT_FILE_DOWNLOAD = 186, "Abort File Download";
    HISTORY_ABORT_FILE_UPLOAD = 187, "Abort File Upload";
    HISTORY_FINISHED_FILE_DOWNLOAD = 188, "Finished File Download";
    HISTORY_FINISHED_FILE_UPLOAD = 189, "Finished File Upload";
    HISTORY_SERVER_STARTED = 190, "Server Started";
    HISTORY_LOGIN_FAILED = 191, "Login Failed";
    CREATE_PRIVATE_CHAT = 200, "Create Private Chat";
    DELETE_CHAT = 201, "Delete Chat";
    JOIN_CHAT = 202, "Join Chat";
    CHAT_WINDOW_INFO = 203, "Chat Window Info";
    LEAVE_CHAT = 204, "Leave Chat";
    CHAT_USER_JOINED = 205, "Chat User Joined";
    CHAT_USER_DEPARTED = 206, "Chat User Departed";
    SEND_CHAT_TEXT = 207, "Send Chat Text";
    RECEIVE_CHAT_TEXT = 208, "Receive Chat Text";
    SET_CHAT_TOPIC = 209, "Set Chat Topic";
    CHAT_TOPIC_CHANGED = 210, "Chat Topic Changed";
    CHAT_INVITE = 211, "Chat Invite";
    CHAT_INVITATION = 212, "Chat Invitation";
    MUST_JOIN_CHAT = 213, "Must Join Chat";
    GET_CHAT_LIST = 214, "Get Chat List";
    CHAT_LIST = 215, "Chat List";
    CREATE_PRIVATE_CHAT_REPLY = 216, "Create Private Chat (R)";
    CREATE_CHAT = 217, "Create Chat";
    GET_CHAT = 218, "Get Chat";
    CHAT_INFO = 219, "Chat Info";
    SET_CHAT = 220, "Set Chat";
    CHAT_DELETED = 221, "Chat Deleted";
    GET_CHAT_DCC_ADDRESSES = 222, "Get Chat DCC Addresses";
    CHAT_DCC_ADDRESSES = 223, "Chat DCC Addresses";
    SEND_PRIVATE_MESSAGE = 300, "Send Private Message";
    INCOMING_PRIVATE_MESSAGE = 301, "Incoming Private Message";
    GET_FILE_LIST = 400, "Get File List";
    FILE_LIST = 401, "File List";
    CREATE_FOLDER = 402, "Create Folder";
    CREATE_ALIAS = 403, "Create Alias";
    DELETE_FILE_FOLDER = 404, "Delete File/Folder";
    MOVE_TO_TRASH = 405, "Move To Trash";
    RENAME_FILE_FOLDER = 406, "Rename File/Folder";
    MOVE_FILE_FOLDER = 407, "Move File/Folder";
    COPY_FILE_FOLDER = 408, "Copy File/Folder";
    GET_FILE_FOLDER_INFO = 409, "Get File/Folder Info";
    FILE_FOLDER_INFO = 410, "File/Folder Info";
    SET_FILE_FOLDER_INFO = 411, "Set File/Folder Info";
    DOWNLOAD_FILE = 412, "Download File";
    DOWNLOAD_FILE_REPLY = 413, "Download File (R)";
    DOWNLOAD_FILE_DATA = 414, "Download File (D)";
    UPLOAD_FILE = 415, "Upload File";
    UPLOAD_FILE_REPLY = 416, "Upload File (R)";
    UPLOAD_FILE_DATA = 417, "Upload File (D)";
    UPLOAD_FILE_COMPLETE = 418, "Upload File (C)";
    NOTIFY_SPEED_LIMIT = 419, "Notify Speed Limit";
    LAUNCH_FILE = 420, "Launch File";
    EMPTY_TRASH = 421, "Empty Trash";
    VOLUME_INFO = 422, "Volume Info";
    SET_VOLUME_INFO = 423, "Set Volume Info";
    GENERATE_CATALOG = 450, "Generate Catalog";
    GENERATE_CATALOG_STATUS = 451, "Generate Catalog (S)";
    FILE_SEARCH = 452, "File Search";
    FILE_SEARCH_REPLY = 453, "File Search (R)";
    GET_ACCOUNT_CLASS = 502, "Get Account Class";
    ACCOUNT_CLASS_INFO = 503, "Account Class Info";
    SET_ACCOUNT_CLASS = 504, "Set Account Class";
    GET_ACCOUNT_CLASS_LIST = 505, "Get Account Class List";
    ACCOUNT_CLASS_LIST = 506, "Account Class List";
    CREATE_ACCOUNT = 507, "Create Account";
    DELETE_ACCOUNT = 508, "Delete Account";
    GET_ACCOUNT = 509, "Get Account";
    ACCOUNT_INFO = 510, "Account Info";
    SET_ACCOUNT = 511, "Set Account";
    GET_ACCOUNT_LIST = 512, "Get Account List";
    ACCOUNT_LIST = 513, "Account List";
    ACCOUNT_CHANGED = 514, "Account Changed";
    ACCOUNT_DELETED = 515, "Account Deleted";
    CHANGE_OWN_PASSWORD = 550, "Change Own Password";
    CHANGE_OWN_PASSWORD_DONE = 551, "Change Own Password Done";
    CREATE_NEWSGROUP = 600, "Create Newsgroup";
    DELETE_NEWSGROUP = 601, "Delete Newsgroup";
    GET_NEWSGROUP = 602, "Get Newsgroup";
    NEWSGROUP_INFO = 603, "Newsgroup Info";
    SET_NEWSGROUP = 604, "Set Newsgroup";
    GET_NEWSGROUP_LIST = 605, "Get Newsgroup List";
    NEWSGROUP_LIST = 606, "Newsgroup List";
    GET_NEWS_MESSAGES = 607, "Get News Messages";
    NEWS_MESSAGES = 608, "News Messages";
    GET_NEWS_MESSAGE = 609, "Get News Message";
    NEWS_MESSAGE = 610, "News Message";
    POST_NEWS_MESSAGE = 611, "Post News Message";
    DELETE_NEWS_MESSAGE = 612, "Delete News Message";
    NEWS_MESSAGE_POSTED = 613, "News Message Posted";
    NEWS_MESSAGE_DELETED = 614, "News Message Deleted";
    GET_TRACKER_GROUP_LIST = 700, "Get Tracker Group List";
    TRACKER_GROUP_LIST = 701, "Tracker Group List";
    GET_TRACKER_SERVER_LIST = 702, "Get Tracker Server List";
    TRACKER_SERVER_LIST = 703, "Tracker Server List";
    GET_TRACKERSERV_SETTINGS = 710, "Get TrackerServ Settings";
    TRACKERSERV_SETTINGS_INFO = 711, "TrackerServ Settings Info";
    SET_TRACKERSERV_SETTINGS = 712, "Set TrackerServ Settings";
    SET_TRACKERSERV_SETTINGS_DONE = 713, "Set TrackerServ Settings Done";
    GET_TRACKERSERV_STATISTICS = 720, "Get TrackerServ Statistics";
    TRACKERSERV_STATISTICS = 721, "TrackerServ Statistics";
    OPEN_PROCESS_MONITOR = 800, "Open Process Monitor";
    CLOSE_PROCESS_MONITOR = 801, "Close Process Monitor";
    GET_PROCESS_LIST = 802, "Get Process List";
    PROCESS_LIST = 803, "Process List";
    EXIT_TERMINATE_PROCESS = 804, "Exit/Terminate Process";
    OPEN_VIEW_DISPLAY = 900, "Open View Display";
    CLOSE_VIEW_DISPLAY = 901, "Close View Display";
    VIEW_DISPLAY_PARAMS = 902, "View Display Params";
    VIEW_DISPLAY_UPDATE = 903, "View Display Update";
    VIEW_DISPLAY_MOUSE = 904, "View Display Mouse";
    VIEW_DISPLAY_KEYBOARD = 905, "View Display Keyboard";
    START_VOICE = 1000, "Start Voice";
    START_VOICE_REPLY = 1001, "Start Voice (R)";
    SET_VOICE_FORMAT = 1002, "Set Voice Format";
    VOICE_DATA = 1003, "Voice Data";
    CREATE_FOLDER_ACCESS = 1100, "Create Folder Access";
    DELETE_FOLDER_ACCESS = 1101, "Delete Folder Access";
    FOLDER_ACCESS_DELETED = 1102, "Folder Access Deleted";
    GET_FOLDER_ACCESS = 1103, "Get Folder Access";
    FOLDER_ACCESS_INFO = 1104, "Folder Access Info";
    SET_FOLDER_ACCESS = 1105, "Set Folder Access";
    GET_FOLDER_ACCESS_LIST = 1106, "Get Folder Access List";
    FOLDER_ACCESS_LIST = 1107, "Folder Access List";
}

impl MessageType {
    /// Asks the server to resend 107 Server Greeting. Provisional name.
    pub const REQUEST_GREETING: Self = Self(112);
    /// Sets an unknown per-user value. Provisional name.
    pub const UNNAMED_118: Self = Self(118);
    /// GConsole: opens the client's window. Provisional name.
    pub const GCONSOLE_OPEN_WINDOW: Self = Self(4000);
    /// GConsole: draws in the client's window. Provisional name.
    pub const GCONSOLE_DRAW: Self = Self(4001);
    /// GConsole: a key in the client's window. Provisional name.
    pub const GCONSOLE_KEY: Self = Self(4200);
    /// GConsole: a mouse button or wheel in the client's window. Provisional name.
    pub const GCONSOLE_MOUSE: Self = Self(4201);
}

impl std::fmt::Display for MessageType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.title() {
            Some(title) => write!(f, "{} {title}", self.0),
            None => write!(f, "{}", self.0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn titles_and_display() {
        assert_eq!(MessageType::DOWNLOAD_FILE.0, 412);
        assert_eq!(MessageType(413).title(), Some("Download File (R)"));
        assert_eq!(MessageType::HISTORY_LOGIN.title(), Some("Login"));
        assert_eq!(MessageType::REQUEST_GREETING.title(), None);
        assert_eq!(MessageType::STATUS.to_string(), "1 Status");
        assert_eq!(MessageType(9999).to_string(), "9999");
    }
}
