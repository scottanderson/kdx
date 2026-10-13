//! A KDX tracker, compatible with Tracker 1.110.
//!
//! Servers register over UDP and clients browse over TCP, both on one port
//! (`docs/protocol/tracker.md`). [`Tracker::run`] serves both:
//!
//! ```no_run
//! # async fn run(table: kdx_proto::ChallengeTable) -> anyhow::Result<()> {
//! use kdx_tracker::{Settings, Sockets, Tracker};
//!
//! let settings = Settings::new(b"admin", b"password");
//! let sockets = Sockets::bind("0.0.0.0".parse()?, settings.port).await?;
//! Tracker::new(settings, None, table).run(sockets).await
//! # }
//! ```
//!
//! It doesn't register with upstream trackers; it keeps their entries only
//! to hand them back in 711.

use std::collections::HashMap;
use std::io;
use std::net::{IpAddr, SocketAddr};
use std::path::PathBuf;
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use anyhow::Result;
use kdx_proto::msg::tracker::{
    GetServerList, GroupListEntry, ServerListEntry, TrackerSettings, TrackerStats,
};
use kdx_proto::registration::Registration;
use kdx_proto::session::MessageType;
use kdx_proto::ChallengeTable;
use tokio::net::{TcpListener, UdpSocket};
use tokio::sync::watch;

pub mod registry;
mod session;
pub mod settings;
#[cfg(test)]
mod tests;

pub use registry::Registry;
pub use settings::{Group, Settings};

use registry::Registered;

/// Largest payload the tracker accepts.
pub const MAX_PAYLOAD_LEN: u32 = 0x10000;
/// Most TCP connections one IP address may hold.
pub const MAX_CONNECTIONS_PER_IP: usize = 5;

/// Error codes of the tracker's 1 Status.
pub mod error {
    /// The request is too short.
    pub const SHORT: u32 = 0x10002;
    /// No such group.
    pub const NOT_FOUND: u32 = 0x1000c;
    /// The request needs the admin login.
    pub const ACCESS_DENIED: u32 = 0x10014;
}

/// Milliseconds from 1900 to 1970.
const EPOCH_1900_MS: u64 = 2_208_988_800_000;

/// The TCP listener and UDP socket that share the tracker's port.
#[derive(Debug)]
pub struct Sockets {
    tcp: TcpListener,
    udp: UdpSocket,
}

impl Sockets {
    /// Binds TCP and UDP on `port`; for port 0, UDP takes the port TCP got.
    pub async fn bind(ip: IpAddr, port: u16) -> io::Result<Self> {
        let tcp = TcpListener::bind((ip, port)).await?;
        let udp = UdpSocket::bind(tcp.local_addr()?).await?;
        Ok(Self { tcp, udp })
    }

    /// The address both sockets are bound to.
    pub fn local_addr(&self) -> io::Result<SocketAddr> {
        self.tcp.local_addr()
    }
}

/// A running tracker.
#[derive(Debug)]
pub struct Tracker {
    table: ChallengeTable,
    settings_path: Option<PathBuf>,
    state: Mutex<State>,
    port: watch::Sender<u16>,
}

#[derive(Debug)]
struct State {
    settings: Settings,
    registry: Registry,
    start: Instant,
    start_wall: u64,
    connections_accepted: u64,
    datagrams_received: u64,
    open: HashMap<IpAddr, usize>,
}

/// What a request gets back.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Reply {
    /// Nothing, as for an unknown request.
    None,
    /// A reply message.
    Message(MessageType, Vec<u8>),
    /// A 1 Status error.
    Error(u32),
}

impl Tracker {
    /// Creates a tracker that saves settings changed by 712 to `settings_path`, if given.
    pub fn new(
        settings: Settings,
        settings_path: Option<PathBuf>,
        table: ChallengeTable,
    ) -> Arc<Self> {
        Arc::new(Self {
            table,
            settings_path,
            port: watch::Sender::new(settings.port),
            state: Mutex::new(State::new(settings)),
        })
    }

    /// The current settings.
    pub fn settings(&self) -> Settings {
        self.state().settings.clone()
    }

    /// Serves registrations and connections until the future is dropped.
    ///
    /// A port set by 712 rebinds both sockets on the same address.
    pub async fn run(self: Arc<Self>, mut sockets: Sockets) -> Result<()> {
        let mut port = self.port.subscribe();
        let mut buf = vec![0; 0x10000];
        loop {
            tokio::select! {
                accepted = sockets.tcp.accept() => {
                    let (stream, peer) = match accepted {
                        Ok(accepted) => accepted,
                        Err(err) => {
                            eprintln!("accept: {err}");
                            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                            continue;
                        }
                    };
                    let Some(guard) = self.admit(peer.ip()) else {
                        eprintln!("{peer}: too many connections");
                        continue;
                    };
                    let tracker = self.clone();
                    tokio::spawn(async move {
                        if let Err(err) = session::serve(&tracker, stream, peer).await {
                            eprintln!("{peer}: {err:#}");
                        }
                        drop(guard);
                    });
                }
                received = sockets.udp.recv_from(&mut buf) => match received {
                    Ok((len, from)) => self.receive(from, &buf[..len]),
                    Err(err) => eprintln!("udp: {err}"),
                },
                Ok(()) = port.changed() => {
                    let new = *port.borrow_and_update();
                    let ip = sockets.local_addr()?.ip();
                    match Sockets::bind(ip, new).await {
                        Ok(bound) => {
                            sockets = bound;
                            eprintln!("listening on {}", sockets.local_addr()?);
                        }
                        Err(err) => eprintln!("port {new}: {err}"),
                    }
                }
            }
        }
    }

    /// Handles one UDP datagram from `from`.
    pub fn receive(&self, from: SocketAddr, datagram: &[u8]) {
        let Some((registered, reg)) = self.state().receive(from, datagram) else {
            return;
        };
        let name = String::from_utf8_lossy(&reg.name);
        match registered {
            Registered::Added => eprintln!("{from}: registered {name:?} port {}", reg.port),
            Registered::Removed => eprintln!("{from}: unregistered port {}", reg.port),
            Registered::Updated | Registered::Ignored => {}
        }
    }

    /// Counts a connection from `ip`, and admits it while the IP holds fewer than the limit.
    fn admit(self: &Arc<Self>, ip: IpAddr) -> Option<ConnectionGuard> {
        let mut state = self.state();
        state.connections_accepted += 1;
        let open = state.open.entry(ip).or_default();
        if *open >= MAX_CONNECTIONS_PER_IP {
            return None;
        }
        *open += 1;
        Some(ConnectionGuard {
            tracker: self.clone(),
            ip,
        })
    }

    /// Handles a request from a logged-in connection.
    fn handle(&self, kind: MessageType, payload: &[u8], admin: bool, peer: SocketAddr) -> Reply {
        let mut state = self.state();
        let port = state.settings.port;
        let reply = state.handle(kind, payload, admin, peer);
        if kind == MessageType::SET_TRACKERSERV_SETTINGS && matches!(reply, Reply::Message(..)) {
            eprintln!("{peer}: TrackerServ settings changed remotely");
            if let Some(path) = &self.settings_path {
                if let Err(err) = state.settings.save(path) {
                    eprintln!("{err:#}");
                }
            }
            if state.settings.port != port {
                self.port.send_replace(state.settings.port);
            }
        }
        reply
    }

    fn state(&self) -> MutexGuard<'_, State> {
        self.state.lock().unwrap_or_else(|e| e.into_inner())
    }
}

impl State {
    fn new(settings: Settings) -> Self {
        let now = Instant::now();
        Self {
            settings,
            registry: Registry::new(now),
            start: now,
            start_wall: wall_ms(),
            connections_accepted: 0,
            datagrams_received: 0,
            open: HashMap::new(),
        }
    }

    /// Counts a datagram and registers it if it checks out.
    fn receive(&mut self, from: SocketAddr, datagram: &[u8]) -> Option<(Registered, Registration)> {
        self.datagrams_received += 1;
        let IpAddr::V4(ip) = from.ip().to_canonical() else {
            return None;
        };
        let reg = Registration::decode(datagram).ok()?;
        let registered = self.registry.register(ip, &reg, Instant::now(), wall_ms());
        Some((registered, reg))
    }

    /// Answers a request; 712 changes only the settings in memory.
    fn handle(
        &mut self,
        kind: MessageType,
        payload: &[u8],
        admin: bool,
        peer: SocketAddr,
    ) -> Reply {
        let state = self;
        let admin_only = matches!(
            kind,
            MessageType::GET_TRACKERSERV_SETTINGS
                | MessageType::SET_TRACKERSERV_SETTINGS
                | MessageType::GET_TRACKERSERV_STATISTICS
        );
        if admin_only && !admin {
            return Reply::Error(error::ACCESS_DENIED);
        }
        match kind {
            MessageType::GET_MY_ADDRESS => {
                let text = peer.to_string();
                let mut reply = vec![text.len() as u8];
                reply.extend_from_slice(text.as_bytes());
                Reply::Message(MessageType::YOUR_ADDRESS, reply)
            }
            MessageType::GET_TRACKER_GROUP_LIST => {
                let groups = &state.settings.groups;
                let list: Vec<_> = groups
                    .iter()
                    .map(|group| GroupListEntry {
                        servers: state.registry.listed(group, groups).count() as u16,
                        value_02: 0,
                        name: group.name.clone(),
                        description: group.description.clone(),
                    })
                    .collect();
                state.registry.sweep(Instant::now());
                let reply = GroupListEntry::encode_list(&list).expect("lengths fit");
                Reply::Message(MessageType::TRACKER_GROUP_LIST, reply)
            }
            MessageType::GET_TRACKER_SERVER_LIST => {
                let Ok(request) = GetServerList::decode(payload) else {
                    return Reply::Error(error::SHORT);
                };
                let groups = &state.settings.groups;
                let Some(group) = groups.iter().find(|g| g.name == request.group) else {
                    return Reply::Error(error::NOT_FOUND);
                };
                let list: Vec<_> = state
                    .registry
                    .listed(group, groups)
                    .map(registry::Record::entry)
                    .collect();
                let reply = ServerListEntry::encode_list(&list).expect("lengths fit");
                Reply::Message(MessageType::TRACKER_SERVER_LIST, reply)
            }
            MessageType::GET_TRACKERSERV_SETTINGS => Reply::Message(
                MessageType::TRACKERSERV_SETTINGS_INFO,
                state.settings.to_blob(false).encode(),
            ),
            MessageType::SET_TRACKERSERV_SETTINGS => {
                // Tracker 1.110 replies 713 even to a blob it can't read.
                if let Ok(blob) = TrackerSettings::decode(payload) {
                    if state.settings.apply(&blob, false).groups {
                        state.registry.clear();
                    }
                }
                Reply::Message(MessageType::SET_TRACKERSERV_SETTINGS_DONE, Vec::new())
            }
            MessageType::GET_TRACKERSERV_STATISTICS => {
                let stats = TrackerStats {
                    start_time: state.start_wall,
                    connections_accepted: state.connections_accepted,
                    datagrams_received: state.datagrams_received,
                    hours: (state.start.elapsed().as_secs() / 3600) as u32,
                    connections: state.open.values().sum::<usize>() as u32,
                    servers: state.registry.len() as u32,
                };
                Reply::Message(MessageType::TRACKERSERV_STATISTICS, stats.encode())
            }
            _ => Reply::None,
        }
    }
}

/// An open connection, counted against its IP until dropped.
struct ConnectionGuard {
    tracker: Arc<Tracker>,
    ip: IpAddr,
}

impl Drop for ConnectionGuard {
    fn drop(&mut self) {
        let mut state = self.tracker.state();
        if let Some(open) = state.open.get_mut(&self.ip) {
            *open -= 1;
            if *open == 0 {
                state.open.remove(&self.ip);
            }
        }
    }
}

/// The tracker's 1 Status: the error code, then the request type.
fn status(code: u32, request: MessageType) -> Vec<u8> {
    let mut status = code.to_be_bytes().to_vec();
    status.extend_from_slice(&request.0.to_be_bytes());
    status
}

/// The time now in milliseconds since 1900, as the protocol counts it.
fn wall_ms() -> u64 {
    let since_1970 = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    EPOCH_1900_MS + since_1970.as_millis() as u64
}
