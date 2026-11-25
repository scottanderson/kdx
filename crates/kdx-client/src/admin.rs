//! Administration requests: 113 to 116, 140 to 143, 150 to 155, 170 to 175
//! and 180 to 183.

use anyhow::Result;
use kdx_proto::msg::admin::{
    DisconnectUser, ExtraServerInfo, HistoryList, MonitoredConnection, SendBroadcast,
    ShutdownServer, YourAddress,
};
use kdx_proto::msg::server_info::ServerSettings;
use kdx_proto::msg::EncodeError;
use kdx_proto::session::MessageType;

use crate::Client;

/// Replaces the server name in `settings`.
///
/// The name is 1 to 255 bytes without NUL; `MainSettings::encode` writes
/// its length at 0x52, which the server reads (without it, the server
/// keeps the old length).
pub fn set_server_name(settings: &mut ServerSettings, name: &[u8]) -> Result<(), EncodeError> {
    if name.is_empty() || name.len() > 0xff || name.contains(&0) {
        return Err(EncodeError::TooLong("server name"));
    }
    settings.main.name = name.to_vec();
    Ok(())
}

impl Client {
    /// 170 Get Server Settings.
    pub async fn server_settings(&mut self) -> Result<ServerSettings> {
        let reply = self.request(MessageType::GET_SERVER_SETTINGS, &[]).await?;
        Ok(ServerSettings::decode(&reply)?)
    }

    /// 172 Set Server Settings; the server answers with 173 or a success Status.
    pub async fn set_server_settings(&mut self, settings: &ServerSettings) -> Result<()> {
        self.request(MessageType::SET_SERVER_SETTINGS, &settings.encode()?)
            .await?;
        Ok(())
    }

    /// 154 Send Broadcast Message; every user gets 155, the sender included,
    /// and there is no reply.
    pub async fn broadcast(&mut self, text: &[u8]) -> Result<()> {
        let request = SendBroadcast {
            text: text.to_vec(),
            ..SendBroadcast::default()
        };
        self.send_checked(MessageType::SEND_BROADCAST_MESSAGE, &request.encode()?)
            .await
    }

    /// 150 Disconnect User; the server answers with a success Status.
    ///
    /// `ban_seconds` 0 bans nobody; `quiet` sets [`DisconnectUser::SILENT`].
    /// The server's ban list does not apply to 127.0.0.1.
    pub async fn disconnect_user(
        &mut self,
        user_id: u32,
        ban_seconds: u32,
        message: &[u8],
        quiet: bool,
    ) -> Result<()> {
        let request = DisconnectUser {
            user_id,
            ban_seconds,
            options: if quiet { DisconnectUser::SILENT } else { 0 },
            message: message.to_vec(),
            ..DisconnectUser::default()
        };
        self.request(MessageType::DISCONNECT_USER, &request.encode()?)
            .await?;
        Ok(())
    }

    /// 180 Server History Join; 183 Server History Added then arrives as
    /// [`Notice::HistoryAdded`](crate::Notice::HistoryAdded) until
    /// [`Self::history_leave`].
    pub async fn history_join(&mut self) -> Result<HistoryList> {
        let reply = self.request(MessageType::SERVER_HISTORY_JOIN, &[]).await?;
        Ok(HistoryList::decode(&reply)?)
    }

    /// 182 Server History Leave.
    pub async fn history_leave(&mut self) -> Result<()> {
        self.send(MessageType::SERVER_HISTORY_LEAVE, &[]).await
    }

    /// 140 Open Connection Monitor.
    pub async fn monitor_open(&mut self) -> Result<()> {
        self.send(MessageType::OPEN_CONNECTION_MONITOR, &[]).await
    }

    /// 141 Close Connection Monitor.
    pub async fn monitor_close(&mut self) -> Result<()> {
        self.send(MessageType::CLOSE_CONNECTION_MONITOR, &[]).await
    }

    /// 142 Get Connection Monitor Info.
    pub async fn monitor_info(&mut self) -> Result<Vec<MonitoredConnection>> {
        let reply = self
            .request(MessageType::GET_CONNECTION_MONITOR_INFO, &[])
            .await?;
        Ok(MonitoredConnection::decode_list(&reply)?)
    }

    /// 174 Shutdown Server; the server sends 151 with the shutdown flag.
    pub async fn shutdown(&mut self, mode: u8, message: &[u8]) -> Result<()> {
        let request = ShutdownServer {
            mode,
            message: message.to_vec(),
            ..ShutdownServer::default()
        };
        self.send(MessageType::SHUTDOWN_SERVER, &request.encode()?)
            .await
    }

    /// 113 Get Extra Server Info.
    pub async fn extra_server_info(&mut self) -> Result<ExtraServerInfo> {
        let reply = self
            .request(MessageType::GET_EXTRA_SERVER_INFO, &[])
            .await?;
        Ok(ExtraServerInfo::decode(&reply)?)
    }

    /// 115 Get My Address: the address the server sees us at.
    pub async fn my_address(&mut self) -> Result<Vec<u8>> {
        let reply = self.request(MessageType::GET_MY_ADDRESS, &[]).await?;
        Ok(YourAddress::decode(&reply)?.0)
    }
}
