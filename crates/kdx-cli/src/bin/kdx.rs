//! A command-line KDX client.

use std::io::{BufRead, IsTerminal, Write};
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};
use kdx_client::{table, Client, Login};
use kdx_proto::mac_roman;
use kdx_proto::msg::admin::Account;
use kdx_proto::msg::chat::{ChatText, ChatTopicChanged, PUBLIC_CHAT};
use kdx_proto::msg::news::{NewsMessage, NewsMessageChange};
use kdx_proto::msg::{FilePath, PrivateMessage, UserRecord};
use kdx_proto::session::MessageType;
use kdx_proto::{AccountKey, ChallengeTable};
use tokio::fs::File;
use tokio::io::{AsyncSeekExt, AsyncWriteExt};

#[derive(Parser)]
#[command(about = "A KDX client")]
struct Args {
    /// Server address.
    #[arg(long, short, default_value = "127.0.0.1:10700", global = true)]
    server: String,
    /// Account login; omit for guest.
    #[arg(long, short, default_value = "", global = true)]
    login: String,
    /// Account password; also read from KDX_PASSWORD.
    #[arg(long, env = "KDX_PASSWORD", hide_env_values = true, global = true)]
    password: Option<String>,
    /// Nick to show to other users.
    #[arg(long, default_value = "kdx-rs", global = true)]
    nick: String,
    /// Challenge table, or a KDX program file or zip archive that holds it; defaults to the cache.
    #[arg(long, global = true)]
    table: Option<PathBuf>,
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Get the challenge table into the cache.
    Table {
        #[command(subcommand)]
        action: TableAction,
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
enum TableAction {
    /// Download the smallest KDX archive from the Internet Archive and extract the table.
    Fetch,
    /// Extract the table from a KDX program file, zip archive, or saved table.
    Import { file: PathBuf },
    /// Show where the cached table is.
    Path,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let args = Args::parse();
    if let Command::Table { action } = &args.command {
        return table_command(action);
    }

    let table = challenge_table(args.table.as_deref())?;
    let login = Login {
        name: args.login.clone().into_bytes(),
        password: args.password.clone().unwrap_or_default().into_bytes(),
    };
    let me = UserRecord {
        nick: args.nick.clone().into_bytes(),
        ..UserRecord::default()
    };
    let mut client = Client::login(&args.server, &login, &me, table).await?;

    match args.command {
        Command::Table { .. } => unreachable!("handled above"),
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
            println!("joined {}", text(window.name()));
            loop {
                let (header, payload) = client.next_event().await?;
                print_event(header.kind, &payload)?;
            }
        }
    }
    Ok(())
}

fn table_command(action: &TableAction) -> Result<()> {
    let table = match action {
        TableAction::Path => {
            let path = table::cache_path().context("no cache directory")?;
            let state = if table::cached().is_some() {
                "present"
            } else {
                "missing"
            };
            println!("{} ({state})", path.display());
            return Ok(());
        }
        TableAction::Fetch => {
            eprintln!("downloading {}", table::FETCH_URL);
            table::fetch()?
        }
        TableAction::Import { file } => table::load(file)?,
    };
    println!("saved {}", table::save_to_cache(&table)?.display());
    Ok(())
}

/// The table from `--table`, the cache, or, at a terminal, a download or file the user picks.
fn challenge_table(path: Option<&Path>) -> Result<ChallengeTable> {
    if let Some(path) = path {
        return table::load(path);
    }
    if let Some(table) = table::cached() {
        return Ok(table);
    }
    let help = "KDX needs Haxial's challenge table, which this program can't include. \
                Run `kdx table fetch` to download it from the Internet Archive, or \
                `kdx table import FILE` with any KDX program or archive.";
    if !std::io::stdin().is_terminal() {
        bail!("{help}");
    }
    eprintln!("{help}");
    eprint!("Download it now from {}? [y/N] ", table::FETCH_URL);
    std::io::stderr().flush()?;
    let table = if prompt()?.eq_ignore_ascii_case("y") {
        table::fetch()?
    } else {
        eprint!("Path to a KDX program, archive, or table (empty to quit): ");
        std::io::stderr().flush()?;
        let answer = prompt()?;
        if answer.is_empty() {
            bail!("no challenge table");
        }
        table::load(Path::new(&answer))?
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
    // The server resumes at a multiple of 0x1000, so the returned start may be earlier.
    file.seek(std::io::SeekFrom::Start(offset & !0xfff)).await?;
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
                hex(&account.privileges),
                hex(&account.privilege_mask)
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

/// Shows a file name as Mac OS Roman, escaping `\\`, `/`, and control bytes so it round-trips.
fn name_text(bytes: &[u8]) -> String {
    let mut out = String::new();
    for &b in bytes {
        match b {
            b'\\' => out.push_str("\\\\"),
            b'/' | ..=0x1f | 0x7f => out.push_str(&format!("\\x{b:02x}")),
            _ => out.push(mac_roman::char_from_byte(b)),
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
            let b = mac_roman::byte_from_char(c).with_context(|| {
                format!("{c:?} in {name:?} is not Mac OS Roman; write it as \\xNN")
            })?;
            out.push(b);
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
        assert_eq!(name_text(b"Erik\xd5s"), "Erik\u{2019}s");
        assert_eq!(name_text(b"a/b\\c\r"), "a\\x2fb\\\\c\\x0d");
    }

    #[test]
    fn typed_paths() {
        let path = parse_path("/Uploads [UL]/Erik\u{2019}s/Erik\\xd5s 2").unwrap();
        assert_eq!(
            path,
            FilePath::new([&b"Uploads [UL]"[..], b"Erik\xd5s", b"Erik\xd5s 2"])
        );
        assert!(parse_path("smile \u{263a}").is_err());
        assert!(parse_path("a\\b").is_err());
        assert!(parse_path("a\\x4").is_err());
        assert!(parse_path("a\\xzz").is_err());
        assert!(parse_path("a\\x+f").is_err());
    }
}
