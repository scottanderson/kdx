//! A command-line KDX client.

use std::io::{BufRead, IsTerminal, Write};
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use clap::{Args as _, CommandFactory, FromArgMatches, Parser, Subcommand};
use kdx_client::table;
use kdx_client::{Client, Login, Tracker};
use kdx_proto::msg::admin::Account;
use kdx_proto::msg::chat::{ChatText, ChatTopicChanged, PUBLIC_CHAT};
use kdx_proto::msg::news::{NewsMessage, NewsMessageChange};
use kdx_proto::msg::tracker::TrackerStats;
use kdx_proto::msg::{FilePath, PrivateMessage, UserRecord};
use kdx_proto::session::MessageType;
use kdx_proto::{AccountKey, ChallengeTable, Story};
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

#[derive(Parser)]
#[command(about = "A KDX client")]
struct Args {
    #[command(subcommand)]
    command: Command,
}

// Options for the subcommands that talk to a server; `main` adds them to each but `table` and `tracker`.
#[derive(clap::Args)]
struct Connection {
    /// Server address.
    #[arg(long, short, default_value = "127.0.0.1:10700")]
    server: String,
    /// Account login; omit for guest.
    #[arg(long, short, default_value = "")]
    login: String,
    /// Account password; also read from KDX_PASSWORD.
    #[arg(long, env = "KDX_PASSWORD", hide_env_values = true)]
    password: Option<String>,
    /// Nick to show to other users.
    #[arg(long, default_value = "kdx-rs")]
    nick: String,
    /// Challenge table, or a KDX program file or zip archive that holds it; defaults to the cache.
    #[arg(long)]
    table: Option<PathBuf>,
}

// Options for the `tracker` subcommands.
#[derive(clap::Args)]
struct TrackerConnection {
    /// Tracker address; the port defaults to 10800.
    tracker: String,
    /// Admin login; omit for guest.
    #[arg(long, short, default_value = "")]
    login: String,
    /// Admin password; also read from KDX_PASSWORD.
    #[arg(long, env = "KDX_PASSWORD", hide_env_values = true)]
    password: Option<String>,
    /// Challenge table, or a KDX program file or zip archive that holds it; defaults to the cache.
    #[arg(long)]
    table: Option<PathBuf>,
}

#[derive(Subcommand)]
enum Command {
    /// Get the challenge tables into the cache: KDX's, and GConsole's.
    Table {
        #[command(subcommand)]
        action: TableAction,
    },
    /// Browse a tracker.
    Tracker {
        #[command(subcommand)]
        action: TrackerAction,
    },
    /// List the users online.
    Who,
    /// List the chats.
    Chats,
    /// List a folder; `/` separates names, and `\xNN` gives a raw byte.
    Ls {
        #[arg(default_value = "")]
        path: String,
    },
    /// Download a file.
    Get {
        /// Path on the server, written as `ls` shows it.
        path: String,
        /// Local file; defaults to the file's name in the current folder.
        #[arg(long, short)]
        output: Option<PathBuf>,
        /// Continue a partial download in the local file.
        #[arg(long)]
        resume: bool,
    },
    /// Upload a file into a folder.
    Put {
        /// Local file.
        file: PathBuf,
        /// Folder on the server.
        #[arg(default_value = "")]
        folder: String,
    },
    /// Read news.
    News {
        /// Newsgroup ID; omit to list the newsgroups.
        group: Option<u32>,
        /// Message ID; omit to list the group's messages.
        message: Option<u32>,
    },
    /// Post a news message; the body is read from standard input if omitted.
    Post {
        group: u32,
        subject: String,
        body: Option<String>,
    },
    /// Delete a news message.
    DeleteNews { group: u32, message: u32 },
    /// List, show, create, change, or delete accounts.
    Account {
        #[command(subcommand)]
        action: AccountAction,
    },
    /// Change your own password.
    Passwd {
        /// The new password; also read from KDX_NEW_PASSWORD.
        #[arg(env = "KDX_NEW_PASSWORD", hide_env_values = true)]
        new_password: String,
    },
    /// Say something in a chat.
    Say {
        /// Chat ID.
        #[arg(long, default_value_t = PUBLIC_CHAT)]
        chat: u32,
        text: String,
    },
    /// Send a private message.
    Msg { user: u32, text: String },
    /// Join a chat and print what happens until interrupted.
    Listen {
        /// Chat ID.
        #[arg(long, default_value_t = PUBLIC_CHAT)]
        chat: u32,
    },
}

#[derive(Subcommand)]
enum AccountAction {
    /// List the accounts.
    List,
    /// List the account classes.
    Classes,
    /// Show an account.
    Show {
        #[arg(value_name = "LOGIN")]
        account: String,
    },
    /// Create an account.
    Add {
        #[arg(value_name = "LOGIN")]
        account: String,
        /// Account name; defaults to the login.
        #[arg(long)]
        name: Option<String>,
        /// Class index, from `account classes`.
        #[arg(long, default_value_t = 1)]
        class: u8,
        /// Password; also read from KDX_NEW_PASSWORD. Omit for none.
        #[arg(long, env = "KDX_NEW_PASSWORD", hide_env_values = true)]
        new_password: Option<String>,
    },
    /// Change an account.
    Set {
        #[arg(value_name = "LOGIN")]
        account: String,
        /// New login.
        #[arg(long)]
        rename: Option<String>,
        /// New account name.
        #[arg(long)]
        name: Option<String>,
        /// New class index.
        #[arg(long)]
        class: Option<u8>,
        /// New password; also read from KDX_NEW_PASSWORD.
        #[arg(long, env = "KDX_NEW_PASSWORD", hide_env_values = true)]
        new_password: Option<String>,
    },
    /// Delete an account.
    Rm {
        #[arg(value_name = "LOGIN")]
        account: String,
    },
}

#[derive(Subcommand)]
enum TrackerAction {
    /// List the groups.
    Groups {
        #[command(flatten)]
        conn: TrackerConnection,
    },
    /// List the servers in a group.
    Servers {
        #[command(flatten)]
        conn: TrackerConnection,
        group: String,
    },
    /// Show the statistics; needs the admin login.
    Stats {
        #[command(flatten)]
        conn: TrackerConnection,
    },
}

#[derive(Subcommand)]
enum TableAction {
    /// Download the smallest KDX archive that holds each table from the Internet Archive.
    Fetch {
        /// Download even if the cache already holds a valid table.
        #[arg(long, short)]
        force: bool,
    },
    /// Extract each table a KDX program file, zip archive, or saved table holds.
    Import { file: PathBuf },
    /// Show where the cached tables are.
    Path,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let matches = Args::command()
        .mut_subcommands(|command| match command.get_name() {
            "table" | "tracker" => command,
            _ => Connection::augment_args(command),
        })
        .get_matches();
    let args = Args::from_arg_matches(&matches).unwrap_or_else(|e| e.exit());
    match args.command {
        Command::Table { action } => return table_command(&action),
        Command::Tracker { action } => return tracker_command(action).await,
        _ => {}
    }
    let (_, matches) = matches.subcommand().expect("a subcommand is required");
    let conn = Connection::from_arg_matches(matches).unwrap_or_else(|e| e.exit());

    let table = challenge_table(conn.table.as_deref())?;
    let login = Login {
        name: conn.login.into_bytes(),
        password: conn.password.unwrap_or_default().into_bytes(),
    };
    let me = UserRecord {
        nick: conn.nick.into_bytes(),
        ..UserRecord::default()
    };
    let mut client = Client::login(&conn.server, &login, &me, table).await?;

    match args.command {
        Command::Table { .. } | Command::Tracker { .. } => unreachable!("handled above"),
        Command::Who => {
            for user in &client.users {
                println!("{:>5}  {}", user.user_id, text(&user.nick));
            }
        }
        Command::Chats => {
            for chat in client.chat_list().await? {
                let voice = if chat.flags & 2 != 0 { " (voice)" } else { "" };
                println!(
                    "{:>5}  {}{voice}, {} members: {}",
                    chat.chat_id,
                    text(&chat.name),
                    chat.member_count,
                    text(&chat.description)
                );
            }
        }
        Command::Ls { path } => {
            for entry in client.file_list(&parse_path(&path)?).await? {
                if entry.is_folder() {
                    println!("{:>12}  {}/", "", name_text(&entry.name));
                } else {
                    println!("{:>12}  {}", entry.size, name_text(&entry.name));
                }
            }
        }
        Command::Get {
            path,
            output,
            resume,
        } => {
            let path = parse_path(&path)?;
            let name = path.0.last().context("give a file path")?;
            let output = output.unwrap_or_else(|| PathBuf::from(name_text(name)));
            download(&mut client, &path, &output, resume).await?;
        }
        Command::Put { file, folder } => {
            let name = file
                .file_name()
                .context("give a file to upload")?
                .to_string_lossy()
                .into_owned();
            let mut source = File::open(&file).await?;
            let len = source.metadata().await?.len();
            let start = client
                .upload(&parse_path(&folder)?, &name_bytes(&name)?, len, &mut source)
                .await?;
            println!("uploaded {} bytes as {name}", len - start);
        }
        Command::News { group: None, .. } => {
            for group in client.newsgroups().await? {
                println!(
                    "{:>5}  {} ({} messages)",
                    group.group_id,
                    text(&group.name),
                    group.message_count
                );
            }
        }
        Command::News {
            group: Some(group),
            message: None,
        } => {
            for message in client.news(group).await?.messages {
                print_news(&message);
            }
        }
        Command::News {
            group: Some(group),
            message: Some(message),
        } => print_news(&client.news_message(group, message).await?),
        Command::Post {
            group,
            subject,
            body,
        } => {
            let body = match body {
                Some(body) => body,
                None => std::io::read_to_string(std::io::stdin())?,
            };
            // KDX separates lines with CR.
            let body = body.trim_end_matches(['\r', '\n']);
            let body = body.replace("\r\n", "\r").replace('\n', "\r");
            client
                .post_news(group, subject.as_bytes(), body.as_bytes())
                .await?;
        }
        Command::DeleteNews { group, message } => client.delete_news(group, message).await?,
        Command::Account { action } => account_command(&mut client, action).await?,
        Command::Passwd { new_password } => client.change_password(new_password.as_bytes()).await?,
        Command::Say { chat, text: line } => {
            client.join_chat(chat, 0).await?;
            client.send_chat(chat, line.as_bytes()).await?;
            // The server echoes the text to every member, the sender included.
            loop {
                let (header, payload) = client.next_event().await?;
                if header.kind == MessageType::RECEIVE_CHAT_TEXT
                    && ChatText::decode(&payload)?.user_id == client.info.user_id
                {
                    break;
                }
            }
            client.leave_chat(chat).await?;
        }
        Command::Msg { user, text } => client.send_private_message(user, text.as_bytes()).await?,
        Command::Listen { chat } => {
            let window = client.join_chat(chat, 0).await?;
            println!("joined {}", text(&window.name));
            loop {
                let (header, payload) = client.next_event().await?;
                print_event(header.kind, &payload)?;
            }
        }
    }
    Ok(())
}

fn table_command(action: &TableAction) -> Result<()> {
    if let TableAction::Import { file } = action {
        let loaded = Story::ALL.map(|story| table::load(story, file));
        if loaded.iter().all(Result::is_err) {
            let [Err(err), ..] = &loaded else {
                unreachable!("all failed")
            };
            bail!("{err:#}");
        }
        for t in loaded.into_iter().flatten() {
            println!("saved {}", table::save_to_cache(&t)?.display());
        }
        return Ok(());
    }
    Story::ALL
        .into_iter()
        .try_for_each(|story| table_action(story, action))
}

/// Runs `kdx table path` or `kdx table fetch` for one story.
fn table_action(story: Story, action: &TableAction) -> Result<()> {
    let path = table::cache_path(story).context("no cache directory")?;
    let cached = table::cached(story).is_some();
    match action {
        TableAction::Path => {
            let state = if cached { "present" } else { "missing" };
            println!("{} ({state})", path.display());
        }
        TableAction::Fetch { force: false } if cached => {
            println!(
                "already saved {}; use --force to download again",
                path.display()
            );
        }
        TableAction::Fetch { .. } => {
            eprintln!("downloading {}", table::fetch_url(story));
            let table = table::fetch(story)?;
            println!("saved {}", table::save_to_cache(&table)?.display());
        }
        TableAction::Import { .. } => unreachable!("handled by table_command"),
    }
    Ok(())
}

async fn tracker_command(action: TrackerAction) -> Result<()> {
    let (TrackerAction::Groups { conn }
    | TrackerAction::Servers { conn, .. }
    | TrackerAction::Stats { conn }) = &action;
    let table = challenge_table(conn.table.as_deref())?;
    let login = Login {
        name: conn.login.clone().into_bytes(),
        password: conn.password.clone().unwrap_or_default().into_bytes(),
    };
    let mut tracker = Tracker::login(tracker_addr(&conn.tracker), &login, table).await?;
    match action {
        TrackerAction::Groups { .. } => {
            for group in tracker.groups().await? {
                println!(
                    "{:>5}  {}{}",
                    group.servers,
                    text(&group.name),
                    described(&group.description)
                );
            }
        }
        TrackerAction::Servers { group, .. } => {
            for server in tracker.servers(group.as_bytes()).await? {
                println!(
                    "{:<21}  {:>4} users  {}{}",
                    server.socket_addr(),
                    server.users,
                    text(&server.name),
                    described(&server.description)
                );
            }
        }
        TrackerAction::Stats { .. } => {
            let TrackerStats {
                start_time,
                connections_accepted,
                datagrams_received,
                hours,
                connections,
                servers,
                ..
            } = tracker.stats().await?;
            println!("started:      {}", time_1900(start_time));
            println!("hours:        {hours}");
            println!("connections:  {connections_accepted} accepted, {connections} open");
            println!("datagrams:    {datagrams_received}");
            println!("servers:      {servers}");
        }
    }
    Ok(())
}

/// Adds the default tracker port to an address that has none.
fn tracker_addr(addr: &str) -> String {
    if addr
        .parse::<std::net::IpAddr>()
        .is_ok_and(|ip| ip.is_ipv6())
    {
        return format!("[{addr}]:{}", Tracker::PORT);
    }
    let has_port = addr.rsplit_once(':').is_some_and(|(host, port)| {
        port.parse::<u16>().is_ok() && (!host.contains(':') || host.ends_with(']'))
    });
    if has_port {
        addr.to_owned()
    } else {
        format!("{addr}:{}", Tracker::PORT)
    }
}

/// `: description`, or nothing for an empty one.
fn described(description: &[u8]) -> String {
    if description.is_empty() {
        String::new()
    } else {
        format!(": {}", text(description))
    }
}

/// Shows milliseconds since 1900 as a UTC time.
fn time_1900(ms: u64) -> String {
    // Days from 1900-01-01 to 1970-01-01.
    const UNIX_EPOCH: i64 = 25_567;
    let secs = (ms / 1000) as i64;
    let (days, rem) = (
        secs.div_euclid(86_400) - UNIX_EPOCH,
        secs.rem_euclid(86_400),
    );
    // Howard Hinnant's civil_from_days.
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + i64::from(month <= 2);
    format!(
        "{year}-{month:02}-{day:02} {:02}:{:02}:{:02}.{:03} UTC",
        rem / 3600,
        rem / 60 % 60,
        rem % 60,
        ms % 1000
    )
}

/// The table from `--table`, the cache, or, at a terminal, a download or file the user picks.
fn challenge_table(path: Option<&Path>) -> Result<ChallengeTable> {
    if let Some(path) = path {
        return table::load(Story::Kdx, path);
    }
    if let Some(table) = table::cached(Story::Kdx) {
        return Ok(table);
    }
    let help = "KDX needs Haxial's challenge table, which this program can't include. \
                Run `kdx table fetch` to download it from the Internet Archive, or \
                `kdx table import FILE` with any KDX program or archive.";
    if !std::io::stdin().is_terminal() {
        bail!("{help}");
    }
    eprintln!("{help}");
    eprint!(
        "Download it now from {}? [y/N] ",
        table::fetch_url(Story::Kdx)
    );
    std::io::stderr().flush()?;
    let table = if prompt()?.eq_ignore_ascii_case("y") {
        table::fetch(Story::Kdx)?
    } else {
        eprint!("Path to a KDX program, archive, or table (empty to quit): ");
        std::io::stderr().flush()?;
        let answer = prompt()?;
        if answer.is_empty() {
            bail!("no challenge table");
        }
        table::load(Story::Kdx, Path::new(&answer))?
    };
    eprintln!("saved {}", table::save_to_cache(&table)?.display());
    Ok(table)
}

fn prompt() -> Result<String> {
    let mut line = String::new();
    std::io::stdin().lock().read_line(&mut line)?;
    Ok(line.trim().to_owned())
}

async fn download(client: &mut Client, path: &FilePath, output: &Path, resume: bool) -> Result<()> {
    let offset = if resume {
        std::fs::metadata(output).map_or(0, |m| m.len())
    } else {
        0
    };
    let mut file = tokio::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(!resume)
        .open(output)
        .await
        .with_context(|| format!("opening {}", output.display()))?;
    let (info, start) = client.download(path, offset, &mut file).await?;
    file.flush().await?;
    file.set_len(info.data_size).await?;
    println!(
        "{}: {} bytes, received {} from offset {start}",
        output.display(),
        info.data_size,
        info.data_size - start
    );
    Ok(())
}

fn print_event(kind: MessageType, payload: &[u8]) -> Result<()> {
    match kind {
        MessageType::RECEIVE_CHAT_TEXT => {
            let chat = ChatText::decode(payload)?;
            println!("[{}] <{}> {}", chat.chat_id, chat.user_id, text(&chat.text));
        }
        MessageType::CHAT_TOPIC_CHANGED => {
            let topic = ChatTopicChanged::decode(payload)?;
            println!(
                "[{}] {} set the topic: {}",
                topic.chat_id,
                text(&topic.nick),
                text(&topic.topic)
            );
        }
        MessageType::INCOMING_PRIVATE_MESSAGE => {
            let message = PrivateMessage::decode(payload)?;
            println!("private from {}: {}", message.user_id, text(&message.text));
        }
        MessageType::USER_JOINED | MessageType::USER_CHANGED => {
            let (user, _) = UserRecord::decode(payload)?;
            let what = if kind == MessageType::USER_JOINED {
                "joined"
            } else {
                "changed"
            };
            println!("user {} {what}: {}", user.user_id, text(&user.nick));
        }
        MessageType::NEWS_MESSAGE_POSTED | MessageType::NEWS_MESSAGE_DELETED => {
            let change = NewsMessageChange::decode(payload)?;
            let what = if kind == MessageType::NEWS_MESSAGE_POSTED {
                "posted"
            } else {
                "deleted"
            };
            println!(
                "news {what}: group {} message {}, now {} messages",
                change.group_id, change.message_id, change.message_count
            );
        }
        other => println!("{other} ({} bytes)", payload.len()),
    }
    Ok(())
}

async fn account_command(client: &mut Client, action: AccountAction) -> Result<()> {
    match action {
        AccountAction::List => {
            for account in client.accounts().await? {
                println!(
                    "{:<16} class {:>2}  {}",
                    text(&account.login),
                    account.class_index,
                    text(&account.name)
                );
            }
        }
        AccountAction::Classes => {
            for class in client.account_classes().await? {
                println!("{:>2}  {}", class.index, text(&class.name));
            }
        }
        AccountAction::Show { account: login } => {
            let account = client.account(login.as_bytes()).await?;
            println!("login:      {}", text(&account.login));
            println!("name:       {}", text(&account.name));
            println!("class:      {}", account.class_index);
            println!(
                "password:   {}",
                if account.has_password() {
                    "set"
                } else {
                    "none"
                }
            );
            println!("saved by:   {}", text(&account.editor));
            println!(
                "privileges: {} of {}",
                hex(&account.privileges.encode()),
                hex(&account.privilege_mask.encode())
            );
        }
        AccountAction::Add {
            account: login,
            name,
            class,
            new_password,
        } => {
            let account = Account {
                key: AccountKey::from_password(new_password.unwrap_or_default().as_bytes()).0,
                class_index: class,
                name: name.unwrap_or_else(|| login.clone()).into_bytes(),
                login: login.into_bytes(),
                ..Account::default()
            };
            client.create_account(&account).await?;
        }
        AccountAction::Set {
            account: login,
            rename,
            name,
            class,
            new_password,
        } => {
            let mut account = client.account(login.as_bytes()).await?;
            // 510 holds a password flag where 511 takes a key; zero keeps the key.
            account.key =
                new_password.map_or([0; 32], |p| AccountKey::from_password(p.as_bytes()).0);
            account.editor = login.into_bytes();
            if let Some(rename) = rename {
                account.login = rename.into_bytes();
            }
            if let Some(name) = name {
                account.name = name.into_bytes();
            }
            if let Some(class) = class {
                account.class_index = class;
            }
            client.set_account(&account).await?;
        }
        AccountAction::Rm { account: login } => client.delete_account(login.as_bytes()).await?,
    }
    Ok(())
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn print_news(message: &NewsMessage) {
    println!(
        "#{} {} — {}\n\n{}\n",
        message.message_id,
        text(&message.subject),
        text(&message.poster),
        text(&message.body).replace('\r', "\n")
    );
}

/// Splits `a/b/c` into a file path of names written as [`name_text`] shows them; empty for the root.
fn parse_path(path: &str) -> Result<FilePath> {
    path.split('/')
        .filter(|name| !name.is_empty())
        .map(name_bytes)
        .collect::<Result<_>>()
        .map(FilePath)
}

/// Shows a file name as UTF-8, escaping `\\`, `/`, control characters, and invalid bytes so it round-trips.
fn name_text(bytes: &[u8]) -> String {
    let mut out = String::new();
    for chunk in bytes.utf8_chunks() {
        for c in chunk.valid().chars() {
            match c {
                '\\' => out.push_str("\\\\"),
                '/' => out.push_str("\\x2f"),
                _ if c.is_control() => {
                    for b in c.encode_utf8(&mut [0; 4]).bytes() {
                        out.push_str(&format!("\\x{b:02x}"));
                    }
                }
                _ => out.push(c),
            }
        }
        for b in chunk.invalid() {
            out.push_str(&format!("\\x{b:02x}"));
        }
    }
    out
}

/// Reads a name written as [`name_text`] shows it.
fn name_bytes(name: &str) -> Result<Vec<u8>> {
    let mut out = Vec::new();
    let mut chars = name.chars();
    while let Some(c) = chars.next() {
        if c != '\\' {
            out.extend_from_slice(c.encode_utf8(&mut [0; 4]).as_bytes());
            continue;
        }
        match chars.next() {
            Some('\\') => out.push(b'\\'),
            Some('x') => {
                let hex: String = chars.by_ref().take(2).collect();
                let b = (hex.len() == 2 && hex.chars().all(|c| c.is_ascii_hexdigit()))
                    .then(|| u8::from_str_radix(&hex, 16).ok())
                    .flatten()
                    .with_context(|| format!("bad \\x escape in {name:?}"))?;
                out.push(b);
            }
            _ => bail!("in {name:?}, write a backslash as \\\\ or start \\xNN"),
        }
    }
    Ok(out)
}

/// Shows protocol text as UTF-8, replacing invalid bytes.
fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_round_trip() {
        let all: Vec<u8> = (0..=0xff).collect();
        assert_eq!(name_bytes(&name_text(&all)).unwrap(), all);
        let mixed = ["caf\u{e9} \u{263a}\u{85}".as_bytes(), b"\xd5\xe2\x82"].concat();
        assert_eq!(name_bytes(&name_text(&mixed)).unwrap(), mixed);
        assert_eq!(name_text("caf\u{e9}".as_bytes()), "caf\u{e9}");
        assert_eq!(name_text(b"Erik\xd5s"), "Erik\\xd5s");
        assert_eq!(name_text(b"a/b\\c\r"), "a\\x2fb\\\\c\\x0d");
        assert_eq!(name_text("\u{85}".as_bytes()), "\\xc2\\x85");
    }

    #[test]
    fn tracker_addresses() {
        assert_eq!(tracker_addr("127.0.0.1"), "127.0.0.1:10800");
        assert_eq!(tracker_addr("127.0.0.1:1234"), "127.0.0.1:1234");
        assert_eq!(tracker_addr("example.com"), "example.com:10800");
        assert_eq!(tracker_addr("::1"), "[::1]:10800");
        assert_eq!(tracker_addr("[::1]:1234"), "[::1]:1234");
    }

    #[test]
    fn times() {
        // linux-tracker frame 37: the first seen time of Lab Server.
        assert_eq!(
            time_1900(0x0000_03a3_80f5_a144),
            "2026-10-12 07:15:33.828 UTC"
        );
        assert_eq!(time_1900(0), "1900-01-01 00:00:00.000 UTC");
    }

    #[test]
    fn typed_paths() {
        let path = parse_path("/Uploads [UL]/caf\u{e9}/Erik\\xd5s \u{263a}").unwrap();
        assert_eq!(
            path,
            FilePath::new([
                &b"Uploads [UL]"[..],
                "caf\u{e9}".as_bytes(),
                &[&b"Erik\xd5s "[..], "\u{263a}".as_bytes()].concat(),
            ])
        );
        assert!(parse_path("a\\b").is_err());
        assert!(parse_path("a\\x4").is_err());
        assert!(parse_path("a\\xzz").is_err());
        assert!(parse_path("a\\x+f").is_err());
    }
}
