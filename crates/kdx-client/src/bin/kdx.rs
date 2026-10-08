//! A command-line KDX client.

use std::fs::File;
use std::io::{BufRead, IsTerminal, Write};
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};
use kdx_client::{table, Client, Login};
use kdx_proto::msg::chat::{ChatText, ChatTopicChanged, PUBLIC_CHAT};
use kdx_proto::msg::news::{NewsMessage, NewsMessageChange};
use kdx_proto::msg::{FilePath, PrivateMessage, UserRecord};
use kdx_proto::session::MessageType;
use kdx_proto::ChallengeTable;

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
    /// List a folder; `/` separates names.
    Ls {
        #[arg(default_value = "")]
        path: String,
    },
    /// Download a file.
    Get {
        /// Path on the server.
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
enum TableAction {
    /// Download the smallest KDX archive from the Internet Archive and extract the table.
    Fetch,
    /// Extract the table from a KDX program file, zip archive, or saved table.
    Import { file: PathBuf },
    /// Show where the cached table is.
    Path,
}

fn main() -> Result<()> {
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
    let mut client = Client::login(&args.server, &login, &me, table)?;

    match args.command {
        Command::Table { .. } => unreachable!("handled above"),
        Command::Who => {
            for user in &client.users {
                println!("{:>5}  {}", user.user_id, text(&user.nick));
            }
        }
        Command::Chats => {
            for chat in client.chat_list()? {
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
            for entry in client.file_list(&parse_path(&path))? {
                if entry.is_folder() {
                    println!("{:>12}  {}/", "", text(&entry.name));
                } else {
                    println!("{:>12}  {}", entry.size, text(&entry.name));
                }
            }
        }
        Command::Get {
            path,
            output,
            resume,
        } => {
            let path = parse_path(&path);
            let name = path.0.last().context("give a file path")?;
            let output = output.unwrap_or_else(|| PathBuf::from(text(name)));
            download(&mut client, &path, &output, resume)?;
        }
        Command::Put { file, folder } => {
            let name = file
                .file_name()
                .context("give a file to upload")?
                .to_string_lossy()
                .into_owned();
            let mut source = File::open(&file)?;
            let len = source.metadata()?.len();
            let start = client.upload(&parse_path(&folder), name.as_bytes(), len, &mut source)?;
            println!("uploaded {} bytes as {name}", len - start);
        }
        Command::News { group: None, .. } => {
            for group in client.newsgroups()? {
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
            for message in client.news(group)?.messages {
                print_news(&message);
            }
        }
        Command::News {
            group: Some(group),
            message: Some(message),
        } => print_news(&client.news_message(group, message)?),
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
            client.post_news(group, subject.as_bytes(), body.as_bytes())?;
        }
        Command::DeleteNews { group, message } => client.delete_news(group, message)?,
        Command::Say { chat, text: line } => {
            client.join_chat(chat, 0)?;
            client.send_chat(chat, line.as_bytes())?;
            // The server echoes the text to every member, the sender included.
            loop {
                let (header, payload) = client.next_event()?;
                if header.kind == MessageType::RECEIVE_CHAT_TEXT
                    && ChatText::decode(&payload)?.user_id == client.info.user_id
                {
                    break;
                }
            }
            client.leave_chat(chat)?;
        }
        Command::Msg { user, text } => client.send_private_message(user, text.as_bytes())?,
        Command::Listen { chat } => {
            let window = client.join_chat(chat, 0)?;
            println!("joined {}", text(window.name()));
            loop {
                let (header, payload) = client.next_event()?;
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

fn download(client: &mut Client, path: &FilePath, output: &Path, resume: bool) -> Result<()> {
    let offset = if resume {
        std::fs::metadata(output).map_or(0, |m| m.len())
    } else {
        0
    };
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(!resume)
        .open(output)
        .with_context(|| format!("opening {}", output.display()))?;
    // The server resumes at a multiple of 0x1000, so the returned start may be earlier.
    let (info, start) = client.download(path, offset, &mut SeekWriter::new(&mut file, offset))?;
    file.set_len(info.data_size)?;
    println!(
        "{}: {} bytes, received {} from offset {start}",
        output.display(),
        info.data_size,
        info.data_size - start
    );
    Ok(())
}

/// Writes at `start`, after seeking there on the first write.
struct SeekWriter<'a> {
    file: &'a mut File,
    start: Option<u64>,
}

impl<'a> SeekWriter<'a> {
    const fn new(file: &'a mut File, offset: u64) -> Self {
        // The server rounds the offset down to 0x1000.
        Self {
            file,
            start: Some(offset & !0xfff),
        }
    }
}

impl Write for SeekWriter<'_> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        if let Some(start) = self.start.take() {
            std::io::Seek::seek(self.file, std::io::SeekFrom::Start(start))?;
        }
        self.file.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.file.flush()
    }
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

fn print_news(message: &NewsMessage) {
    println!(
        "#{} {} — {}\n\n{}\n",
        message.message_id,
        text(&message.subject),
        text(&message.poster),
        text(&message.body).replace('\r', "\n")
    );
}

/// Splits `a/b/c` into a file path; empty for the root.
fn parse_path(path: &str) -> FilePath {
    FilePath::new(path.split('/').filter(|name| !name.is_empty()))
}

/// Shows protocol text as UTF-8, replacing invalid bytes.
fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}
