//! A connection to a tracker, for browsing its groups and servers.

use std::collections::VecDeque;

use anyhow::Result;
use kdx_proto::msg::tracker::{GetServerList, GroupListEntry, ServerListEntry, TrackerStats};
use kdx_proto::session::MessageType;
use kdx_proto::{AccountKey, ChallengeTable};
use tokio::net::ToSocketAddrs;

use crate::client::wait_reply;
use crate::{Connection, Event, Login, Responses};

/// A logged-in tracker session.
#[derive(Debug)]
pub struct Tracker {
    conn: Connection,
    events: VecDeque<Event>,
}

impl Tracker {
    /// The default TCP and UDP port.
    pub const PORT: u16 = 10800;

    /// Connects and logs in; an empty login is a guest, which can't use [`Self::stats`].
    pub async fn login(
        addr: impl ToSocketAddrs,
        login: &Login,
        table: ChallengeTable,
    ) -> Result<Self> {
        Self::login_with(addr, login, &Responses::Table(table)).await
    }

    pub(crate) async fn login_with(
        addr: impl ToSocketAddrs,
        login: &Login,
        responses: &Responses,
    ) -> Result<Self> {
        let key = AccountKey::from_password(&login.password);
        let conn = Connection::connect_with_key(addr, login.name.clone(), key, responses).await?;
        Ok(Self {
            conn,
            events: VecDeque::new(),
        })
    }

    /// Sends a request and waits for its reply, queueing other messages as events.
    pub async fn request(&mut self, kind: MessageType, payload: &[u8]) -> Result<Vec<u8>> {
        let txid = self.conn.send(kind, payload).await?;
        wait_reply(&mut self.conn, kind, txid, &mut self.events).await
    }

    /// The next message the tracker sent on its own, such as 107 Server Greeting.
    pub async fn next_event(&mut self) -> Result<Event> {
        match self.events.pop_front() {
            Some(event) => Ok(event),
            None => self.conn.recv().await,
        }
    }

    /// 700 Get Tracker Group List.
    pub async fn groups(&mut self) -> Result<Vec<GroupListEntry>> {
        let reply = self
            .request(MessageType::GET_TRACKER_GROUP_LIST, &[])
            .await?;
        Ok(GroupListEntry::decode_list(&reply)?)
    }

    /// 702 Get Tracker Server List.
    pub async fn servers(&mut self, group: &[u8]) -> Result<Vec<ServerListEntry>> {
        let request = GetServerList {
            group: group.to_vec(),
        };
        let reply = self
            .request(MessageType::GET_TRACKER_SERVER_LIST, &request.encode()?)
            .await?;
        Ok(ServerListEntry::decode_list(&reply)?)
    }

    /// 720 Get TrackerServ Statistics; needs the admin login.
    pub async fn stats(&mut self) -> Result<TrackerStats> {
        let reply = self
            .request(MessageType::GET_TRACKERSERV_STATISTICS, &[])
            .await?;
        Ok(TrackerStats::decode(&reply)?)
    }
}
