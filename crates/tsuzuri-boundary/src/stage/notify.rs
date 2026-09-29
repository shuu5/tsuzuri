//! 表示先の端末への知らせ（行 i-10・要件 FR16・判断の記録 ADR-15 の決定 (6)・持ち主の裁定 t3-hub.59.4 と t3-hub.59.5）。
//! 席が持ち主に見てほしい時、端末の画面に notify-send の知らせだけを出す。窓を起こす・前に出す・動かす語を持たず、
//! Chrome の口を撃たない。1 回の ssh で notify-send の在る無しと撃ちを済ませ（無ければ遠くの shell が `ABSENT` で終わる）、
//! ssh の届かない `UNREACHED` と notify-send の rc を分ける。知らせは linux の端末だけで、本文は board の URL。
//! 記録は project ごとの最新の 1 つを tsuzuri 自前の追跡されない file（XDG の state の dir の下の `DIR` の下）に書く
//! （行 i-11 の server がこの module の `dir`・`files`・`gather` で読む）。書きは同じ dir の一時の file に mode 0600 で書いて
//! rename で置き換える。

use std::ffi::{OsStr, OsString};
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::os::unix::fs::OpenOptionsExt;
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{self, Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use super::json;
use super::launch::quote;
use super::terminal::{Os, Terminal};
use super::url;
use crate::acct;
use crate::server::proc;

/// 端末で撃つ知らせの program。
pub const PROGRAM: &str = "notify-send";

/// 端末に notify-send が無い時の遠くの shell の rc。
pub const ABSENT: i32 = 127;

/// ssh が端末に届かない時の rc。
pub const UNREACHED: i32 = 255;

/// 題の字の数の上限。
pub const TITLE_MAX: usize = 200;

/// state の dir の下の知らせの記録の dir。
pub const DIR: &str = "tsuzuri/notify";

/// 記録の file の名の末の字（名の前の字は project の名）。
pub const EXT: &str = ".json";

/// 鍵だけで入り、接続の待ちを 10 秒で切り、席の口座の接続の共有を使わない（起こしの ssh と同じ 6 語）。
const SSH_OPTIONS: [&str; 6] = [
    "-o",
    "BatchMode=yes",
    "-o",
    "ConnectTimeout=10",
    "-o",
    "ControlPath=none",
];

/// notify の受ける旗（端末の名と、repo と、撃つ program の差し替えと、表示先の設定の path）。
const FLAGS: [&str; 7] = [
    "--to",
    "--repo",
    "--ssh",
    "--scribe2",
    "--git",
    "--tailnet",
    "--config",
];

/// ssh の終わりを見る間隔。
const POLL: Duration = Duration::from_millis(20);

/// 読んだ tz stage notify の引数。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NotifyCall {
    pub title: String,
    pub to: Option<String>,
    pub repo: PathBuf,
    pub ssh: OsString,
    pub scribe2: OsString,
    pub git: OsString,
    pub tailnet: OsString,
    pub config: Option<PathBuf>,
}

/// 端末への撃ちの終わり方。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome {
    /// notify-send が rc 0 で終わった。
    Sent,
    /// 端末に notify-send が無い。
    Absent,
    /// notify-send がこの rc で終わった。
    Failed(i32),
    /// 端末に届かない（理由の字）。
    Unreached(String),
}

/// project の最新の知らせの記録（epoch 秒・project の名・題・board の URL）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Record {
    pub at: u64,
    pub project: String,
    pub title: String,
    pub url: String,
}

/// 知らせに使える題か（空でなく、- で始まらず、制御の字を含まず、`TITLE_MAX` 字まで）。
pub fn title_shaped(title: &str) -> bool {
    !title.is_empty()
        && !title.starts_with('-')
        && !title.chars().any(char::is_control)
        && title.chars().count() <= TITLE_MAX
}

/// tz stage notify の後の引数を読む（旗は `FLAGS` を `--名 値` か `--名=値` の形で 1 度ずつ・-- で始まらない字は題で 1 つだけ）。
pub fn parse(args: &[&str]) -> Result<NotifyCall, String> {
    let mut flags: [Option<&str>; 7] = [None; 7];
    let mut words = Vec::new();
    let mut it = args.iter();
    while let Some(&arg) = it.next() {
        if !arg.starts_with("--") {
            words.push(arg);
            continue;
        }
        let (name, value) = match arg.split_once('=') {
            Some(pair) => pair,
            None => (
                arg,
                *it.next().ok_or_else(|| format!("{arg} の値が無い"))?,
            ),
        };
        let Some(i) = FLAGS.iter().position(|f| *f == name) else {
            return Err(format!("notify は旗 {name} を受けない"));
        };
        if value.is_empty() {
            return Err(format!("{name} の値が空"));
        }
        if flags[i].replace(value).is_some() {
            return Err(format!("{name} が 2 度ある"));
        }
    }
    let [title] = words.as_slice() else {
        return Err(format!(
            "notify の題は 1 つ（旗でない字が {} 個）",
            words.len()
        ));
    };
    if !title_shaped(title) {
        return Err(format!(
            "{title:?} は知らせに使えない（題は空でなく - で始まらず制御の字を含まない {TITLE_MAX} 字まで）"
        ));
    }
    let [to, repo, ssh, scribe2, git, tailnet, config] = flags;
    let program = |value: Option<&str>, default: &str| OsString::from(value.unwrap_or(default));
    Ok(NotifyCall {
        title: title.to_string(),
        to: to.map(str::to_string),
        repo: PathBuf::from(repo.unwrap_or(".")),
        ssh: program(ssh, "ssh"),
        scribe2: program(scribe2, "scribe2"),
        git: program(git, acct::GIT),
        tailnet: program(tailnet, url::TAILNET),
        config: config.map(PathBuf::from),
    })
}

/// notify-send に渡す画面の env の組（display の DISPLAY・display-env の宣言の順・ime-env は渡さない）。
pub fn notify_env(terminal: &Terminal) -> Vec<(String, String)> {
    terminal
        .display
        .iter()
        .map(|d| ("DISPLAY".to_string(), d.clone()))
        .chain(terminal.display_env.iter().cloned())
        .collect()
}

/// 知らせの ssh の program の後の引数（遠くの shell の 1 語で notify-send の在る無しを確かめてから撃ち、入出力を捨てる）。
/// linux でない端末は断る。
pub fn argv(terminal: &Terminal, title: &str, url: &str) -> Result<Vec<String>, String> {
    if terminal.os != Os::Linux {
        return Err(format!(
            "端末 {} の [os] {} の端末に知らせを出す形をまだ持たない（知らせは linux の端末だけ・持ち主へは board の URL を渡す）",
            terminal.name,
            terminal.os.word()
        ));
    }
    let mut words = vec![quote("env")];
    words.extend(
        notify_env(terminal)
            .iter()
            .map(|(k, v)| quote(&format!("{k}={v}"))),
    );
    words.extend([PROGRAM, title, url].into_iter().map(quote));
    let mut argv: Vec<String> = SSH_OPTIONS.iter().map(|w| w.to_string()).collect();
    argv.push("--".to_string());
    argv.push(terminal.ssh.clone());
    argv.push(format!(
        "command -v {PROGRAM} >/dev/null 2>&1 || exit {ABSENT}; exec {} </dev/null >/dev/null 2>&1",
        words.join(" ")
    ));
    Ok(argv)
}

/// ssh の rc を終わり方にする。
pub fn outcome(rc: i32) -> Outcome {
    match rc {
        0 => Outcome::Sent,
        ABSENT => Outcome::Absent,
        UNREACHED => Outcome::Unreached(format!("ssh の rc {UNREACHED}")),
        rc => Outcome::Failed(rc),
    }
}

/// 端末へ知らせを撃つ（新しい process group で撃ち、入出力は捨て、timeout まで `POLL` ごとに終わりを見る）。
/// 撃てない・signal で終わる・timeout を越える・見られないは Unreached（時間切れは group ごと止める）。
/// linux でない端末は撃たずに argv の Err。
pub fn send(
    ssh: &OsStr,
    terminal: &Terminal,
    title: &str,
    url: &str,
    timeout: Duration,
) -> Result<Outcome, String> {
    let argv = argv(terminal, title, url)?;
    let mut child = match Command::new(ssh)
        .args(&argv)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .process_group(0)
        .spawn()
    {
        Ok(child) => child,
        Err(e) => return Ok(Outcome::Unreached(format!("ssh を撃てない: {e}"))),
    };
    let deadline = Instant::now() + timeout;
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                return Ok(match status.code() {
                    Some(rc) => outcome(rc),
                    None => Outcome::Unreached(format!("ssh が signal で終わった: {status}")),
                });
            }
            Ok(None) if Instant::now() < deadline => thread::sleep(POLL),
            Ok(None) => {
                proc::stop_with(child, OsStr::new(proc::KILL));
                return Ok(Outcome::Unreached(format!(
                    "ssh が {} 秒の内に終わらない",
                    timeout.as_secs_f32()
                )));
            }
            Err(e) => {
                proc::stop_with(child, OsStr::new(proc::KILL));
                return Ok(Outcome::Unreached(format!("ssh を見られない: {e}")));
            }
        }
    }
}

/// 端末 name への撃ちの終わりの 1 行。
pub fn line(name: &str, outcome: &Outcome) -> String {
    match outcome {
        Outcome::Sent => {
            format!("端末 {name} に知らせを出した（{PROGRAM}・窓は起こさず前に出さない）")
        }
        Outcome::Absent => format!(
            "端末 {name} に {PROGRAM} が無い（知らせを出せない・持ち主へは board の URL を渡す）"
        ),
        Outcome::Failed(rc) => format!(
            "端末 {name} の {PROGRAM} が rc {rc} で終わった（知らせを出せない・画面の env と session の bus を確かめる・持ち主へは board の URL を渡す）"
        ),
        Outcome::Unreached(why) => {
            format!("端末 {name} に届かない（{why}・持ち主へは board の URL を渡す）")
        }
    }
}

/// 記録の dir（XDG_STATE_HOME が絶対の path ならその下、ほかは絶対の HOME の下の .local/state の下の `DIR`・
/// XDG の決まりどおり相対の値と空の値は捨てる・どちらも使えなければ None）。記録の名の形を決めるのはこの file だけ。
pub fn dir(xdg_state_home: Option<&OsStr>, home: Option<&OsStr>) -> Option<PathBuf> {
    let xdg = xdg_state_home.map(Path::new).filter(|p| p.is_absolute());
    let base = match xdg {
        Some(dir) => dir.to_path_buf(),
        None => home
            .map(Path::new)
            .filter(|p| p.is_absolute())?
            .join(".local/state"),
    };
    Some(base.join(DIR))
}

/// 記録の file の path（`dir` の下の project の名と `EXT`・dir が決まらなければ None）。
pub fn path(
    xdg_state_home: Option<&OsStr>,
    home: Option<&OsStr>,
    project: &str,
) -> Option<PathBuf> {
    Some(dir(xdg_state_home, home)?.join(format!("{project}{EXT}")))
}

/// dir の下の記録の file（名が . で始まらず、`EXT` より長く `EXT` で終わる file・dir は数えない・名の順）。
/// 一時の file は名が . で始まるので数えない。dir が無ければ空、ほかの読めない dir は Err。
pub fn files(dir: &Path) -> io::Result<Vec<PathBuf>> {
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(e) => return Err(e),
    };
    let mut out = Vec::new();
    for entry in entries {
        let entry = entry?;
        let name = entry.file_name();
        let Some(name) = name.to_str() else {
            continue;
        };
        if name.starts_with('.') || name.len() <= EXT.len() || !name.ends_with(EXT) {
            continue;
        }
        if entry.file_type()?.is_file() {
            out.push(entry.path());
        }
    }
    out.sort();
    Ok(out)
}

/// dir の下の記録を集める（`files` の各 file を `read` で読み、読めて中の project が file の名から `EXT` を除いた字と
/// 同じ記録を集め、ほかはその名を unread に名の順で足す）。記録は at の新しい順・同じ at は project の名の順。
/// dir が無ければ 2 つとも空、読めない dir は Err。
pub fn gather(dir: &Path) -> io::Result<(Vec<Record>, Vec<String>)> {
    let mut records = Vec::new();
    let mut unread = Vec::new();
    for file in files(dir)? {
        let Some(name) = file
            .file_name()
            .and_then(|n| n.to_str())
            .and_then(|n| n.strip_suffix(EXT))
        else {
            continue;
        };
        let record = fs::read_to_string(&file)
            .ok()
            .and_then(|text| read(&text))
            .filter(|r| r.project == name);
        match record {
            Some(r) => records.push(r),
            None => unread.push(name.to_string()),
        }
    }
    records.sort_by(|a, b| b.at.cmp(&a.at).then_with(|| a.project.cmp(&b.project)));
    Ok((records, unread))
}

/// 記録の字（鍵 at・project・title・url の順の 1 行の JSON の object と改行）。
pub fn render(record: &Record) -> String {
    format!(
        "{{\"at\":{},\"project\":{},\"title\":{},\"url\":{}}}\n",
        record.at,
        json::escape(&record.project),
        json::escape(&record.title),
        json::escape(&record.url)
    )
}

/// 記録の字を読む（前後の空白を除いた object の at が数字だけの字の時だけ・ほかの 3 つは字の値・読めなければ None）。
pub fn read(text: &str) -> Option<Record> {
    let text = text.trim();
    let at = json::member(text, "at")
        .filter(|v| !v.is_empty() && v.bytes().all(|b| b.is_ascii_digit()))?
        .parse()
        .ok()?;
    let text_of = |key: &str| json::member(text, key).and_then(json::unquote);
    Some(Record {
        at,
        project: text_of("project")?,
        title: text_of("title")?,
        url: text_of("url")?,
    })
}

/// 記録の file を書く（dir を作り、同じ dir の一時の file に mode 0600 で書いて sync し、rename で置き換える）。
pub fn save(path: &Path, record: &Record) -> Result<(), String> {
    let dir = path.parent().unwrap_or(Path::new("."));
    fs::create_dir_all(dir)
        .map_err(|e| format!("知らせの記録の dir {} を作れない: {e}", dir.display()))?;
    let temp = dir.join(format!(".notify.{}", process::id()));
    let _ = fs::remove_file(&temp);
    let written = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(&temp)
        .and_then(|mut file| {
            file.write_all(render(record).as_bytes())?;
            file.sync_all()
        })
        .and_then(|()| fs::rename(&temp, path));
    written.map_err(|e| {
        let _ = fs::remove_file(&temp);
        format!("知らせの記録 {} を書けない: {e}", path.display())
    })
}
