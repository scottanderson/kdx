//! Logs in to a KDX server, sends raw requests, and dumps every message received.
//!
//! Each request is `TYPE` or `TYPE:HEX`, for example `605` or `607:00000001`.

use std::path::PathBuf;
use std::time::Duration;

use anyhow::{Context, Result};
use clap::Parser;
use kdx_client::{load_table, Connection, Login};
use kdx_proto::session::MessageType;

#[derive(Parser)]
#[command(about = "Send raw requests to a KDX server and dump the replies")]
struct Args {
    /// Server address.
    #[arg(long, default_value = "127.0.0.1:10700")]
    addr: String,
    /// Account login; omit for guest.
    #[arg(long, default_value = "")]
    login: String,
    /// Account password.
    #[arg(long, default_value = "")]
    password: String,
    /// Challenge table, or any KDX program file that contains it.
    #[arg(long)]
    table: PathBuf,
    /// Nickname sent in the login message.
    #[arg(long, default_value = "probe")]
    nick: String,
    /// Skip the login message (type 100).
    #[arg(long)]
    no_login: bool,
    /// How long to wait for more messages after each request, in milliseconds.
    #[arg(long, default_value_t = 700)]
    wait: u64,
    /// Requests to send in order: `TYPE` or `TYPE:HEX`.
    requests: Vec<String>,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let table = load_table(&args.table)?;
    let login = Login {
        name: args.login.into_bytes(),
        password: args.password.into_bytes(),
    };
    let mut conn = Connection::connect(&args.addr, &login, &table)?;
    println!(
        "connected; server version {:#04x}",
        conn.server_hello.version
    );
    conn.stream()
        .set_read_timeout(Some(Duration::from_millis(args.wait)))?;

    let mut requests = Vec::new();
    if !args.no_login {
        requests.push((MessageType(100), login_record(args.nick.as_bytes())));
    }
    for request in &args.requests {
        let (kind, hex) = request.split_once(':').unwrap_or((request, ""));
        let kind: u32 = kind
            .parse()
            .with_context(|| format!("bad type in {request}"))?;
        requests.push((
            MessageType(kind),
            decode_hex(hex).with_context(|| format!("bad hex in {request}"))?,
        ));
    }

    for (kind, payload) in requests {
        let txid = conn.send(kind, &payload)?;
        println!(">> txid {txid:08x} type {} len {}", kind.0, payload.len());
        print!("{}", hexdump(&payload));
        drain(&mut conn)?;
    }
    Ok(())
}

/// Prints messages until the read times out.
fn drain(conn: &mut Connection) -> Result<()> {
    loop {
        match conn.recv() {
            Ok((header, payload)) => {
                let reply = if header.is_reply() { " reply" } else { "" };
                println!(
                    "<< txid {:08x} type {}{reply} len {}",
                    header.txid, header.kind.0, header.len
                );
                print!("{}", hexdump(&payload));
            }
            Err(err) => {
                let timed_out = err.downcast_ref::<std::io::Error>().is_some_and(|e| {
                    matches!(
                        e.kind(),
                        std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
                    )
                });
                return if timed_out { Ok(()) } else { Err(err) };
            }
        }
    }
}

/// A login (type 100) user record with no icon and the given nickname.
fn login_record(nick: &[u8]) -> Vec<u8> {
    let mut record = vec![0; 0x1c];
    record[0x19] = nick.len() as u8;
    record.extend_from_slice(nick);
    record
}

fn decode_hex(hex: &str) -> Result<Vec<u8>> {
    let hex: String = hex
        .chars()
        .filter(|c| !c.is_whitespace() && *c != '_')
        .collect();
    anyhow::ensure!(hex.len().is_multiple_of(2), "odd number of hex digits");
    (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).context("invalid hex"))
        .collect()
}

fn hexdump(data: &[u8]) -> String {
    let mut out = String::new();
    for (i, chunk) in data.chunks(16).enumerate() {
        let hex: Vec<String> = chunk.iter().map(|b| format!("{b:02x}")).collect();
        let ascii: String = chunk
            .iter()
            .map(|&b| {
                if b.is_ascii_graphic() || b == b' ' {
                    b as char
                } else {
                    '.'
                }
            })
            .collect();
        out.push_str(&format!(
            "   {:04x}  {:<47}  {ascii}\n",
            i * 16,
            hex.join(" ")
        ));
    }
    out
}
