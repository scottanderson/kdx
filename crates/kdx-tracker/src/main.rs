//! `kdx-tracker`: a KDX tracker.

use std::net::IpAddr;
use std::path::PathBuf;

use anyhow::{bail, Context, Result};
use clap::Parser;
use kdx_client::table;
use kdx_proto::Story;
use kdx_tracker::{Settings, Sockets, Tracker};

/// A KDX tracker: servers register over UDP, and clients browse over TCP, on one port.
#[derive(Parser)]
#[command(version)]
struct Args {
    /// Settings file, created on the first run.
    #[arg(long, default_value = "kdx-tracker.stg")]
    settings: PathBuf,
    /// Address to listen on.
    #[arg(long, default_value = "0.0.0.0")]
    address: IpAddr,
    /// Tracker port, for both TCP and UDP; saved in the settings.
    #[arg(long, value_parser = clap::value_parser!(u16).range(1..=0xfffe))]
    tracker_port: Option<u16>,
    /// Admin login; saved in the settings. Needed on the first run.
    #[arg(long)]
    admin_login: Option<String>,
    /// Admin password; also read from KDX_TRACKER_PASSWORD. Saved in the settings.
    #[arg(long, env = "KDX_TRACKER_PASSWORD", hide_env_values = true)]
    admin_password: Option<String>,
    /// Challenge table, or a KDX program file or zip archive that holds it; defaults to the cache.
    #[arg(long)]
    table: Option<PathBuf>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let table = match &args.table {
        Some(path) => table::load(Story::Kdx, path)?,
        None => table::cached(Story::Kdx).context("no cached KDX story; pass --table")?,
    };

    let (mut settings, mut changed) = if args.settings.exists() {
        (Settings::load(&args.settings)?, false)
    } else {
        let (Some(login), Some(password)) = (&args.admin_login, &args.admin_password) else {
            bail!(
                "{} doesn't exist; pass --admin-login and --admin-password to create it",
                args.settings.display()
            );
        };
        (Settings::new(login.as_bytes(), password.as_bytes()), true)
    };
    if let Some(login) = &args.admin_login {
        changed |= settings.set_admin_login(login.as_bytes());
    }
    if let Some(password) = &args.admin_password {
        let key = kdx_proto::AccountKey::from_password(password.as_bytes());
        changed |= key != settings.admin_key;
        settings.admin_key = key;
    }
    if let Some(port) = args.tracker_port {
        changed |= port != settings.tracker_port;
        settings.tracker_port = port;
    }
    if changed {
        settings.save(&args.settings)?;
    }

    let sockets = Sockets::bind(args.address, settings.tracker_port)
        .await
        .with_context(|| format!("binding {}:{}", args.address, settings.tracker_port))?;
    eprintln!("listening on {}", sockets.local_addr()?);
    Tracker::new(settings, Some(args.settings), table)
        .run(sockets)
        .await
}
