//! tz stage の口（行 i-5・要件 FR16・判断の記録 ADR-15 の決定 (6) と (7)・持ち主の裁定 t3-hub.59.3・t3-hub.59.5・t3-hub.59.7）。
//! 席は端末の一覧の名で命令し、命令ごとに決まった旗だけを受ける（端末の値を上書きする旗は持たず、撃つ program の
//! 差し替えの旗だけを持つ・条 N-3）。board の頁の上では見せるだけにし、click・入力・key を撃つ前に断る。
//! 窓を起こすのは持ち主の tz stage open だけで、席の中（環境変数 CLAUDECODE が 1）の open は断る（計画の 3 節の席の決め）。
//! open は端末ごとの錠を持って撃ち、同時の撃ちで窓を 2 つにしない。窓は board ごとに 1 つで、覚えた頁か board の頁が
//! 動いている Chrome に無い時だけ、持ち主の頼みとして起動の引数を 1 回撃つ（動いている Chrome が app の窓を足す・行 i-board-win）。
//! 窓の頁の path は端末ごとの file に project の名で覚え（`memo`）、選びの前に読み、窓を見つけるか起こした後に書く。
//! 窓を前に出す・動かす・頁を作る語は持たない。
//! host の面は器の rules validate --state-dir が rc 0 で返った後にだけ読み、自分の anchor の state dir の host.toml だけを読む。
//! 標準出力の最後の行は、board の URL を組めた後のどの終わり方でも url の line（持ち主へ渡す URL）。
//! --to を省いた撃ちは repo の project の名で表示先の設定（`target`）を引き、初めて見せる端末の時だけ窓を起こしてよいと渡して印を書く（行 i-7）。
//! tz stage open も窓を起こすか頁を 1 つ作った時は、錠の中で同じ印を書く（--to の在る無しによらない・行 i-open-mark）。
//! 設定に値が無ければ席の目に落ちて URL の行を出し（open は断る）、表示先は board の問いで持ち主に問う。
//! 設定の端末の名が層 A（器の host の面の [[device]]）に無ければ名指して断り、既定へ落とさない。
//! tz stage target は show・set --project・set --all・clear --project の 4 つの口で設定を読み書きする（URL の行は出さない）。
//! show --json は show と同じ読みを電文の 1 行で出す（board の server が行を割らずに読む・行 e-stage-target）。
//! 席の自分の board の頁でない board の頁（account board の頁とほかの project の board の頁）の上では、見せる操作も含めてどの命令も撃たず断る（行 i-stage-guard）。
//! click・入力・key の断りは url の ports のほかの board（群の宣言の anchor ごとの project board）にも広げ、
//! port が読めない project は名指して出す（その board の断りは広げず、撃ちは止めない・行 i-board-ports）。
//! tz stage notify は窓を起こさず表示先の端末へ知らせだけを出し、project の最新の知らせの記録を書く（行 i-10）。
//! notify の rc は記録を書けて端末に知らせが届いた時だけ 0 で、URL を組めた後のどの終わり方でも最後の行は url の line。

use std::env;
use std::ffi::{OsStr, OsString};
use std::fs::{self, File, OpenOptions, TryLockError};
use std::io::{self, Read};
use std::os::unix::fs::OpenOptionsExt;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use tsuzuri_contract::stage as msg;
use tsuzuri_contract::wire;
use tsuzuri_core::account::host::declaration;
use tsuzuri_core::account::project_name;

use super::cdp::{Command, Session};
use super::json;
use super::memo;
use super::notify::{self, NotifyCall, Outcome, Record};
use super::relay::{self, Eyes, Reach};
use super::target::{self, Targets};
use super::terminal::{self, Terminal};
use super::tunnel::{self, Tunnel, Window};
use super::url::{self, Board};
use crate::acct;
use crate::server::proc;

/// 使い方の行（命令の撃ちと、表示先の設定の口と、知らせの口）。
pub const USAGE: &str = "usage: tz stage <navigate|viewport|reload|click|type|key|scroll|wait|screenshot|dom|console|run|open> [--to <端末の名>] [--url <URL> | --width <n> --height <n> --scale <n> --mobile <true|false> | --x <n> --y <n> [--dy <n>] | --text <字> | --key <鍵> | --ms <n> | --out <path>] [--repo <dir>] [--ssh <program>] [--scribe2 <program>] [--git <program>] [--tailnet <program>] [--chrome <program>] [--config <path>]
       tz stage target <show [--json] | set --project <project の名> <端末の名> | set --all <端末の名> | clear --project <project の名>> [--repo <dir>] [--scribe2 <program>] [--git <program>] [--config <path>]
       tz stage notify [--to <端末の名>] <題> [--repo <dir>] [--ssh <program>] [--scribe2 <program>] [--git <program>] [--tailnet <program>] [--config <path>]";

/// 席の中の撃ちの印の環境変数（Claude Code の Bash の道具が子の process に 1 を渡す）。
pub const SEAT_ENV: &str = "CLAUDECODE";

/// 器の検めの引数（この後に state dir）。器に上の階の validate は無く、host の面の検めは
/// rules validate の旗 --state-dir（器の scribe2 help rules の FORM・行 i-rules-validate）。
pub const VALIDATE_ARGS: [&str; 3] = ["rules", "validate", "--state-dir"];

/// 子の process と接続の待ち。
pub const TIMEOUT: Duration = Duration::from_secs(10);

/// ほかの tz stage open の錠を待つ上限。
pub const LOCK_WAIT: Duration = Duration::from_secs(30);

/// 不合格（断り・使い方の誤り・命令の誤り）。
const FAIL: u8 = 1;

/// 錠の見直しの間隔。
const POLL: Duration = Duration::from_millis(50);

/// どの命令でも受ける旗（端末の名と、repo と、撃つ program の差し替えと、表示先の設定の path）。
const COMMON: [&str; 8] = [
    "--to",
    "--repo",
    "--ssh",
    "--scribe2",
    "--git",
    "--tailnet",
    "--chrome",
    "--config",
];

/// tz stage の命令の語。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Verb {
    /// 1 つの命令と、screenshot の書き先。
    One(Command, Option<PathBuf>),
    /// 標準入力の行ごとの命令の列。
    Run,
    /// 持ち主が表示先の端末に窓を開く。
    Open,
}

/// 読んだ引数。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Call {
    pub verb: Verb,
    pub to: Option<String>,
    pub repo: PathBuf,
    pub ssh: OsString,
    pub scribe2: OsString,
    pub git: OsString,
    pub tailnet: OsString,
    pub chrome: OsString,
    pub config: Option<PathBuf>,
}

/// 旗の列を `--名 値` か `--名=値` の組にする（-- で始まらない字・値の無い旗・空の値は Err）。
fn pairs<'a>(args: &[&'a str]) -> Result<Vec<(&'a str, &'a str)>, String> {
    let mut out = Vec::new();
    let mut it = args.iter();
    while let Some(&arg) = it.next() {
        if !arg.starts_with("--") {
            return Err(format!("旗でない字 {arg}"));
        }
        let (name, value) = match arg.split_once('=') {
            Some(pair) => pair,
            None => (
                arg,
                *it.next().ok_or_else(|| format!("{arg} の値が無い"))?,
            ),
        };
        if value.is_empty() {
            return Err(format!("{name} の値が空"));
        }
        out.push((name, value));
    }
    Ok(out)
}

/// 旗の値を数にする（外れれば旗の名を含む Err）。
fn number<T: FromStr>(name: &str, value: &str, kind: &str) -> Result<T, String> {
    value
        .parse()
        .map_err(|_| format!("{name} の値 {value} は {kind} の数でない"))
}

/// 1 つの命令の語と旗の組を命令にする（命令ごとに決まった旗だけを受ける）。
pub fn one(verb: &str, flags: &[(&str, &str)]) -> Result<(Command, Option<PathBuf>), String> {
    let allowed: &[&str] = match verb {
        "navigate" => &["--url"],
        "viewport" => &["--width", "--height", "--scale", "--mobile"],
        "click" => &["--x", "--y"],
        "type" => &["--text"],
        "key" => &["--key"],
        "scroll" => &["--x", "--y", "--dy"],
        "wait" => &["--ms"],
        "screenshot" => &["--out"],
        "reload" | "dom" | "console" => &[],
        _ => return Err(format!("知らない命令 {verb}")),
    };
    let mut seen: Vec<(&str, &str)> = Vec::new();
    for &(name, value) in flags {
        if !allowed.contains(&name) {
            return Err(format!("{verb} は旗 {name} を受けない"));
        }
        if seen.iter().any(|(n, _)| *n == name) {
            return Err(format!("{name} が 2 度ある"));
        }
        seen.push((name, value));
    }
    let get = |name: &str| {
        seen.iter()
            .find(|(n, _)| *n == name)
            .map(|(_, v)| *v)
            .ok_or_else(|| format!("{verb} に {name} が無い"))
    };
    let u32_of = |name: &str| get(name).and_then(|v| number::<u32>(name, v, "0 以上の u32"));
    let command = match verb {
        "navigate" => {
            let url = get("--url")?;
            let shaped = (url.starts_with("http://") || url.starts_with("https://"))
                && !url.chars().any(|c| c.is_whitespace() || c.is_control());
            if !shaped {
                return Err(format!(
                    "--url の値 {url:?} は http:// か https:// で始まる空白と制御の字の無い字でない（script と端末の file を URL で運ばない）"
                ));
            }
            Command::Navigate {
                url: url.to_string(),
            }
        }
        "viewport" => Command::Viewport {
            width: u32_of("--width")?,
            height: u32_of("--height")?,
            scale: u32_of("--scale")?,
            mobile: match get("--mobile")? {
                "true" => true,
                "false" => false,
                other => return Err(format!("--mobile の値 {other} は true か false でない")),
            },
        },
        "click" => Command::Click {
            x: u32_of("--x")?,
            y: u32_of("--y")?,
        },
        "type" => Command::Type {
            text: get("--text")?.to_string(),
        },
        "key" => Command::Key {
            key: get("--key")?.to_string(),
        },
        "scroll" => Command::Scroll {
            x: u32_of("--x")?,
            y: u32_of("--y")?,
            dy: get("--dy").and_then(|v| number::<i32>("--dy", v, "i32"))?,
        },
        "wait" => Command::Wait {
            ms: get("--ms").and_then(|v| number::<u64>("--ms", v, "0 以上の u64"))?,
        },
        "screenshot" => return Ok((Command::Screenshot, Some(PathBuf::from(get("--out")?)))),
        "reload" => Command::Reload,
        "dom" => Command::Dom,
        _ => Command::Console,
    };
    Ok((command, None))
}

/// tz stage の後の引数を読む（最初の字が命令の語・共通の旗は 1 度ずつ・--to を省けば表示先の設定で引く）。
pub fn parse(args: &[&str]) -> Result<Call, String> {
    let Some((&verb, rest)) = args.split_first() else {
        return Err("命令が無い".to_string());
    };
    let mut common: [Option<&str>; 8] = [None; 8];
    let mut own = Vec::new();
    for (name, value) in pairs(rest)? {
        match COMMON.iter().position(|c| *c == name) {
            Some(i) => {
                if common[i].replace(value).is_some() {
                    return Err(format!("{name} が 2 度ある"));
                }
            }
            None => own.push((name, value)),
        }
    }
    let [to, repo, ssh, scribe2, git, tailnet, chrome, config] = common;
    let verb = match verb {
        "run" | "open" => {
            if let Some((name, _)) = own.first() {
                return Err(format!("{verb} は旗 {name} を受けない"));
            }
            if verb == "run" { Verb::Run } else { Verb::Open }
        }
        _ => {
            let (command, out) = one(verb, &own)?;
            Verb::One(command, out)
        }
    };
    let program = |value: Option<&str>, default: &str| OsString::from(value.unwrap_or(default));
    Ok(Call {
        verb,
        to: to.map(str::to_string),
        repo: PathBuf::from(repo.unwrap_or(".")),
        ssh: program(ssh, "ssh"),
        scribe2: program(scribe2, "scribe2"),
        git: program(git, acct::GIT),
        tailnet: program(tailnet, url::TAILNET),
        chrome: program(chrome, relay::CHROME),
        config: config.map(PathBuf::from),
    })
}

/// 表示先の決め。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Aim {
    /// --to で渡した名（設定を読まず、窓を起こさない）。
    Named(String),
    /// 設定で引いた名と、その端末に初めて見せるか（印がまだ無い）。
    Chosen { name: String, first: bool },
    /// 設定に project の値も既定も無い（席の目に落ちる）。
    Unset,
}

/// 表示先を決める（--to が在ればその名・無ければ project に効く設定の値・その名が層 A に無ければ既定へ落とさずに断る）。
pub fn aim(
    to: Option<&str>,
    project: &str,
    targets: &Targets,
    names: &[String],
) -> Result<Aim, String> {
    if let Some(to) = to {
        return Ok(Aim::Named(to.to_string()));
    }
    let Some((name, origin)) = targets.effective(project) else {
        return Ok(Aim::Unset);
    };
    if !names.iter().any(|n| n == name) {
        return Err(format!(
            "表示先の設定の project {project} に効く{}の端末 {name} は層 A（host の面の [[device]]）に無い（既定へ落とさない・在る名は {}）",
            origin.word(),
            target::listed(names)
        ));
    }
    Ok(Aim::Chosen {
        name: name.to_string(),
        first: !targets.shown.contains_key(name),
    })
}

/// repo の project の名（canonicalize した path の最後の段・account の電文の project の名と同じ出所）。
pub fn project(repo: &Path) -> Result<String, String> {
    let path = fs::canonicalize(repo)
        .map_err(|e| format!("repo {} の path を引けない: {e}", repo.display()))?;
    let text = path
        .to_str()
        .ok_or_else(|| format!("repo {} の path は UTF-8 の字でない", path.display()))?;
    let name = project_name(text);
    if !target::shaped(&name) {
        return Err(format!(
            "project の名 {name:?} は表示先の設定に書けない（空でなく引用符・逆斜線・=・制御の字を含まない名）"
        ));
    }
    Ok(name)
}

/// 表示先の設定の path（--config が在ればその path・無ければ環境の XDG_CONFIG_HOME と HOME で決める）。
pub fn config_path(config: Option<&Path>) -> Result<PathBuf, String> {
    if let Some(path) = config {
        return Ok(path.to_path_buf());
    }
    target::path(
        env::var_os("XDG_CONFIG_HOME").as_deref(),
        env::var_os("HOME").as_deref(),
    )
    .ok_or_else(|| {
        "表示先の設定の path を決められない（XDG_CONFIG_HOME も HOME も絶対の path でない・--config で渡す）"
            .to_string()
    })
}

/// tz stage target の命令。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Setting {
    /// 設定と層 A の名を並べる。
    Show,
    /// show と同じ読みを電文の 1 行で出す（show --json・行 e-stage-target）。
    Json,
    /// project の上書きを置く（project の名・端末の名）。
    Project(String, String),
    /// 全体の既定を置き、project ごとの上書きを全部外す。
    All(String),
    /// project の上書きを外す。
    Clear(String),
}

/// 読んだ tz stage target の引数。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TargetCall {
    pub setting: Setting,
    pub repo: PathBuf,
    pub scribe2: OsString,
    pub git: OsString,
    pub config: Option<PathBuf>,
}

/// tz stage target の後の引数を読む（--all と --json は値の無い旗で --json は show だけが受ける・受ける旗は
/// --project・--repo・--scribe2・--git・--config で 1 度ずつ・旗でない字は端末の名）。
pub fn parse_target(args: &[&str]) -> Result<TargetCall, String> {
    const FLAGS: [&str; 5] = ["--project", "--repo", "--scribe2", "--git", "--config"];
    let Some((&verb, rest)) = args.split_first() else {
        return Err("target の命令が無い（show・set・clear）".to_string());
    };
    if !matches!(verb, "show" | "set" | "clear") {
        return Err(format!("知らない target の命令 {verb}（show・set・clear）"));
    }
    let (mut all, mut json) = (false, false);
    let mut flags: [Option<&str>; 5] = [None; 5];
    let mut words = Vec::new();
    let mut it = rest.iter();
    while let Some(&arg) = it.next() {
        if arg == "--all" {
            if all {
                return Err("--all が 2 度ある".to_string());
            }
            all = true;
            continue;
        }
        if arg == "--json" {
            if json {
                return Err("--json が 2 度ある".to_string());
            }
            if verb != "show" {
                return Err("--json は target show だけが受ける".to_string());
            }
            json = true;
            continue;
        }
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
            return Err(format!("target は旗 {name} を受けない"));
        };
        if value.is_empty() {
            return Err(format!("{name} の値が空"));
        }
        if flags[i].replace(value).is_some() {
            return Err(format!("{name} が 2 度ある"));
        }
    }
    let [project, repo, scribe2, git, config] = flags;
    let setting = match (verb, project, all, words.as_slice()) {
        ("show", None, false, []) if json => Setting::Json,
        ("show", None, false, []) => Setting::Show,
        ("show", ..) => return Err("target show の形でない（旗 --project と --all と端末の名を受けない）".to_string()),
        ("set", Some(p), false, [name]) => Setting::Project(p.to_string(), name.to_string()),
        ("set", None, true, [name]) => Setting::All(name.to_string()),
        ("set", ..) => {
            return Err("target set の形でない（set --project <project の名> <端末の名> か set --all <端末の名>）".to_string());
        }
        ("clear", Some(p), false, []) => Setting::Clear(p.to_string()),
        _ => return Err("target clear の形でない（clear --project <project の名>）".to_string()),
    };
    let named: Vec<&str> = match &setting {
        Setting::Show | Setting::Json => Vec::new(),
        Setting::Project(p, n) => vec![p, n],
        Setting::All(n) | Setting::Clear(n) => vec![n],
    };
    if let Some(bad) = named.into_iter().find(|n| !target::shaped(n)) {
        return Err(format!(
            "名 {bad:?} は表示先の設定に書けない（空でなく引用符・逆斜線・=・制御の字を含まない名）"
        ));
    }
    let program = |value: Option<&str>, default: &str| OsString::from(value.unwrap_or(default));
    Ok(TargetCall {
        setting,
        repo: PathBuf::from(repo.unwrap_or(".")),
        scribe2: program(scribe2, "scribe2"),
        git: program(git, acct::GIT),
        config: config.map(PathBuf::from),
    })
}

/// tz stage target show の行（既定・上書き・project ごとの効く値と出所・層 A の名）。
pub fn show(targets: &Targets, projects: &[String], names: &[String]) -> Vec<String> {
    let mut out = vec![format!(
        "既定 {}",
        targets.default.as_deref().unwrap_or("無し")
    )];
    out.extend(
        targets
            .projects
            .iter()
            .map(|(project, name)| format!("上書き {project} {name}")),
    );
    for project in projects {
        out.push(match targets.effective(project) {
            None => format!("project {project} 無し（席の目と URL に落ちる）"),
            Some((name, origin)) => {
                let absent = if names.iter().any(|n| n == name) {
                    ""
                } else {
                    "・層 A に無い"
                };
                format!("project {project} {name}（{}{absent}）", origin.word())
            }
        });
    }
    out.push(format!("層 A の名 {}", target::listed(names)));
    out
}

/// tz stage target show --json の電文（`show` の行と同じ材料・`own` は --repo の project の名・出所 `Project` は `Override`）。
pub fn doc(
    targets: &Targets,
    own: &str,
    projects: &[String],
    names: &[String],
) -> msg::StageTargets {
    let origin = |origin: target::Origin| match origin {
        target::Origin::Default => msg::Origin::Default,
        target::Origin::Project => msg::Origin::Override,
    };
    msg::StageTargets {
        project: own.to_string(),
        default: targets.default.clone(),
        overrides: targets
            .projects
            .iter()
            .map(|(project, name)| msg::Override {
                project: project.clone(),
                name: name.clone(),
            })
            .collect(),
        projects: projects
            .iter()
            .map(|project| msg::ProjectTarget {
                project: project.clone(),
                effective: targets
                    .effective(project)
                    .map(|(name, from)| msg::Effective {
                        name: name.to_string(),
                        origin: origin(from),
                        listed: names.iter().any(|n| n == name),
                    }),
            })
            .collect(),
        names: names.to_vec(),
    }
}

/// tz stage target を撃つ（設定を読み、器の検めの後に層 A の名を読んで、並べるか書く）。
fn setting(call: &TargetCall) -> Result<(), String> {
    let path = config_path(call.config.as_deref())?;
    let mut targets = target::load(&path)?;
    let text = face_text(&call.repo, &call.scribe2, &call.git)?;
    let names = terminal::names(&text);
    let line = match &call.setting {
        Setting::Show | Setting::Json => {
            let own = project(&call.repo)?;
            let mut projects: Vec<String> = Vec::new();
            let anchors = declaration(&text)
                .groups
                .into_iter()
                .flat_map(|g| g.anchors.unwrap_or_default());
            for name in anchors.map(|a| project_name(&a)).chain([own.clone()]) {
                if !projects.contains(&name) {
                    projects.push(name);
                }
            }
            if call.setting == Setting::Json {
                let message = doc(&targets, &own, &projects, &names);
                let line = wire::encode(&message).map_err(|e| format!("電文を組めない: {e}"))?;
                println!("{line}");
                return Ok(());
            }
            for line in show(&targets, &projects, &names) {
                println!("{line}");
            }
            return Ok(());
        }
        Setting::Project(project, name) => {
            target::known(name, &names)?;
            targets.set_project(project, name);
            format!("project {project} の表示先を {name} にした（上書き）")
        }
        Setting::All(name) => {
            target::known(name, &names)?;
            let dropped = targets.set_all(name);
            format!("全体の既定を {name} にし、project ごとの上書きを {dropped} 個外した")
        }
        Setting::Clear(project) => {
            if !targets.clear_project(project) {
                return Err(format!("project {project} に上書きは無い"));
            }
            let now = match &targets.default {
                Some(name) => format!("既定 {name}"),
                None => "無し（席の目と URL に落ちる）".to_string(),
            };
            format!("project {project} の上書きを外した（効く値は{now}）")
        }
    };
    target::save(&path, &targets)?;
    println!("{line}");
    Ok(())
}

/// 今の epoch 秒（初めて見せた印の値）。
fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs())
}

/// 標準入力の字を命令の列にする（行ごとに JSON の字の配列・空の行は飛ばす・誤りは行の番号を名指す）。
pub fn lines(input: &str) -> Result<Vec<(Command, Option<PathBuf>)>, String> {
    let mut out = Vec::new();
    for (i, line) in input.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let n = i + 1;
        let words: Vec<String> = json::items(line)
            .ok_or_else(|| format!("{n} 行目は JSON の配列でない"))?
            .into_iter()
            .map(|item| {
                json::unquote(item).ok_or_else(|| format!("{n} 行目の要素 {item} は字でない"))
            })
            .collect::<Result<_, _>>()?;
        let Some((verb, rest)) = words.split_first() else {
            return Err(format!("{n} 行目に命令が無い"));
        };
        if matches!(verb.as_str(), "run" | "open") {
            return Err(format!("{n} 行目: 知らない命令 {verb}（行の命令にできない）"));
        }
        let rest: Vec<&str> = rest.iter().map(String::as_str).collect();
        let flags = pairs(&rest).map_err(|e| format!("{n} 行目: {e}"))?;
        out.push(one(verb, &flags).map_err(|e| format!("{n} 行目: {e}"))?);
    }
    Ok(out)
}

/// 席の中の tz stage open の断り（環境変数の値が字 1 の時だけ Some）。
pub fn seat_refusal(value: Option<&OsStr>) -> Option<String> {
    (value == Some(OsStr::new("1"))).then(|| {
        format!(
            "tz stage open は席から撃たない（環境変数 {SEAT_ENV} が 1・窓を開くのは持ち主が素の端末か board の button で撃つ）"
        )
    })
}

/// 命令が board の頁の上で断る click・入力・key か（判断の記録 ADR-15 の決定 (7)・board は見せるだけ）。
pub fn on_board(command: &Command, page: &str, board: &Board) -> bool {
    on_boards(command, page, board, &[])
}

/// 命令が自分の board か ports のほかの board の頁の上で断る click・入力・key か。
pub fn on_boards(command: &Command, page: &str, board: &Board, ports: &[u16]) -> bool {
    guarded(command) && board.holds_any(page, ports)
}

/// board の頁の上で断る命令（click・入力・key）か。
fn guarded(command: &Command) -> bool {
    matches!(
        command,
        Command::Click { .. } | Command::Type { .. } | Command::Key { .. }
    )
}

/// 端末ごとの錠（base の下の tzst-<名>.lock を mode 0600 で開き、wait まで取りに行く・File の drop で放れる）。
pub fn lock(base: &Path, name: &str, wait: Duration) -> Result<File, String> {
    let shaped = !name.is_empty()
        && !name.starts_with('.')
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'));
    if !shaped {
        return Err(format!(
            "端末の名 {name:?} は錠の file の名にできない（英数字と - と _ と . だけで . で始まらない名）"
        ));
    }
    let path = base.join(format!("tzst-{name}.lock"));
    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(false)
        .mode(0o600)
        .open(&path)
        .map_err(|e| format!("錠の file {} を開けない: {e}", path.display()))?;
    let deadline = Instant::now() + wait;
    loop {
        match file.try_lock() {
            Ok(()) => return Ok(file),
            Err(TryLockError::WouldBlock) => {}
            Err(TryLockError::Error(e)) => {
                return Err(format!("錠の file {} を取れない: {e}", path.display()));
            }
        }
        if Instant::now() >= deadline {
            return Err(format!(
                "端末 {name} の錠を {} 秒の内に取れない（ほかの tz stage open が窓を開いている）",
                wait.as_secs_f32()
            ));
        }
        thread::sleep(POLL);
    }
}

/// 端末ごとの覚えから、この project の窓の頁の path を引く（project か端末の名が覚えに書けない形か、file が無ければ None）。
fn recall(base: &Path, terminal: &str, project: Option<&str>) -> Option<String> {
    memo::recall(&memo::path(base, terminal)?, project?)
}

/// 見つけた窓の頁の path を覚える（覚えに書けなくても撃ちは止めず、1 行を出す）。
fn remember(base: &Path, terminal: &str, project: Option<&str>, resource: &str) {
    let (Some(file), Some(project)) = (memo::path(base, terminal), project) else {
        return;
    };
    if let Err(e) = memo::put(&file, project, resource) {
        println!("{e}（撃ちは止めない・次の撃ちは board の頁で窓を選ぶ）");
    }
}

/// 字 = の詰めの付いた標準の base64 を byte に戻す（形の外れた字は None）。
fn unbase64(text: &str) -> Option<Vec<u8>> {
    let bytes = text.as_bytes();
    if !bytes.len().is_multiple_of(4) {
        return None;
    }
    let groups = bytes.len() / 4;
    let mut out = Vec::with_capacity(groups * 3);
    for (i, chunk) in bytes.chunks(4).enumerate() {
        let pad = chunk.iter().rev().take_while(|b| **b == b'=').count();
        if pad > 2 || (pad > 0 && i + 1 != groups) {
            return None;
        }
        let mut n: u32 = 0;
        for &b in &chunk[..4 - pad] {
            let v = match b {
                b'A'..=b'Z' => b - b'A',
                b'a'..=b'z' => b - b'a' + 26,
                b'0'..=b'9' => b - b'0' + 52,
                b'+' => 62,
                b'/' => 63,
                _ => return None,
            };
            n = (n << 6) | u32::from(v);
        }
        n <<= 6 * pad as u32;
        out.extend_from_slice(&n.to_be_bytes()[1..4 - pad]);
    }
    Some(out)
}

/// 命令の語。
fn word(command: &Command) -> &'static str {
    match command {
        Command::Navigate { .. } => "navigate",
        Command::Viewport { .. } => "viewport",
        Command::Reload => "reload",
        Command::Click { .. } => "click",
        Command::Type { .. } => "type",
        Command::Key { .. } => "key",
        Command::Scroll { .. } => "scroll",
        Command::Wait { .. } => "wait",
        Command::Screenshot => "screenshot",
        Command::Dom => "dom",
        Command::Console => "console",
    }
}

/// 使い方の誤り（標準エラーに書いて FAIL）。
fn usage(what: &str) -> u8 {
    eprintln!("tz stage: {what}\n{USAGE}");
    FAIL
}

/// tz stage の後の引数を撃つ（rc は 0 か 1・最初の字が target なら表示先の設定の口・notify なら知らせの口）。
pub fn run(args: &[&str]) -> u8 {
    if let Some((&"notify", rest)) = args.split_first() {
        return match notify::parse(rest) {
            Ok(call) => tell(&call),
            Err(e) => usage(&e),
        };
    }
    if let Some((&"target", rest)) = args.split_first() {
        let call = match parse_target(rest) {
            Ok(call) => call,
            Err(e) => return usage(&e),
        };
        return match setting(&call) {
            Ok(()) => 0,
            Err(e) => {
                println!("{e}");
                FAIL
            }
        };
    }
    let call = match parse(args) {
        Ok(call) => call,
        Err(e) => return usage(&e),
    };
    let script = match &call.verb {
        Verb::One(command, out) => vec![(command.clone(), out.clone())],
        Verb::Open => Vec::new(),
        Verb::Run => {
            let mut input = String::new();
            if let Err(e) = io::stdin().read_to_string(&mut input) {
                return usage(&format!("標準入力を読めない: {e}"));
            }
            match lines(&input) {
                Ok(script) => script,
                Err(e) => return usage(&e),
            }
        }
    };
    let board = match url::board(&call.tailnet, &call.git, &call.repo, TIMEOUT) {
        Ok(board) => board,
        Err(e) => {
            println!("{e}");
            return FAIL;
        }
    };
    let rc = match stage(&call, &script, &board) {
        Ok(()) => 0,
        Err(e) => {
            println!("{e}");
            FAIL
        }
    };
    println!("{}", url::line(&board.url));
    rc
}

/// 席の中の open を断り、器の検めの後に表示先を決めて端末の行を引き、open か命令の列を撃つ。
/// --to を省いた撃ちは表示先の設定で引き、初めて見せる端末の時だけ錠の中で印を読み直して窓を起こしてよいと渡す。
fn stage(call: &Call, script: &[(Command, Option<PathBuf>)], board: &Board) -> Result<(), String> {
    if call.verb == Verb::Open
        && let Some(line) = seat_refusal(env::var_os(SEAT_ENV).as_deref())
    {
        return Err(line);
    }
    let text = face_text(&call.repo, &call.scribe2, &call.git)?;
    let mut ports = Vec::new();
    if !script.is_empty() {
        let read = url::ports(&text, &call.git, &call.repo, TIMEOUT);
        for name in &read.unread {
            println!("project {name} の board の port が読めない（その board の頁の上の断りは広げない）");
        }
        ports = read.ports;
    }
    let base = tunnel::user_dir(&env::temp_dir())?;
    let (name, first, config) = match &call.to {
        Some(to) => (to.clone(), false, None),
        None => {
            let path = config_path(call.config.as_deref())?;
            let own = project(&call.repo)?;
            let names = terminal::names(&text);
            match aim(None, &own, &target::load(&path)?, &names)? {
                Aim::Named(name) => (name, false, None),
                Aim::Chosen { name, first } => (name, first, Some(path)),
                Aim::Unset if call.verb == Verb::Open => {
                    return Err(format!(
                        "tz stage open に --to が無く、表示先の設定に project {own} の値も既定も無い（窓を開く端末は --to で渡すか tz stage target set で決める）"
                    ));
                }
                Aim::Unset => {
                    let (_eyes, session) = Eyes::open(&call.chrome, &base, &board.url, TIMEOUT)?;
                    println!(
                        "表示先の設定に project {own} の値も既定も無いので席の目（この server の headless の Chrome）に落ちた・持ち主へは board の URL を渡し、表示先は board の問いで持ち主に問う"
                    );
                    return drive(session, script, board, &ports);
                }
            }
        }
    };
    let terminal = terminal::lookup(&text, &name)?;
    let own = project(&call.repo).ok();
    if call.verb == Verb::Open {
        return open(call, &terminal, &base, board, own.as_deref());
    }
    let hint = recall(&base, &name, own.as_deref());
    let held = match (first, config.as_deref()) {
        (true, Some(_)) => Some(lock(&base, &name, LOCK_WAIT)?),
        _ => None,
    };
    let first = match config.as_deref() {
        Some(path) if first => !target::load(path)?.shown.contains_key(&name),
        _ => false,
    };
    match relay::reach(&call.ssh, &call.chrome, &terminal, &base, &board.url, TIMEOUT)? {
        Reach::Terminal(tunnel) => match tunnel.window(board, hint.as_deref(), first)? {
            Window::Page { resource, launched } => {
                if launched && let Some(path) = config.as_deref() {
                    target::mark(path, &name, now())?;
                    println!(
                        "端末 {name} に初めて表示面の窓を起こし、表示先の設定に印を書いた（持ち主が閉じた後は起こし直さない）"
                    );
                }
                remember(&base, &name, own.as_deref(), &resource);
                drop(held);
                let session = Session::open(tunnel.socket(), &resource, TIMEOUT)?;
                drive(session, script, board, &ports)
            }
            Window::Absent(line) => Err(line),
        },
        Reach::Eyes {
            eyes: _eyes,
            session,
            line,
        } => {
            drop(held);
            println!("{line}");
            drive(session, script, board, &ports)
        }
    }
}

/// tz stage notify を撃つ（board の URL を組み、端末へ撃つ前に記録を書き、端末へ撃ち、最後に URL の行を出す）。
/// URL を組めなければ 1 行を出して URL の行を出さない。rc は記録を書けて端末に知らせが届いた時だけ 0。
fn tell(call: &NotifyCall) -> u8 {
    let board = match url::board(&call.tailnet, &call.git, &call.repo, TIMEOUT) {
        Ok(board) => board,
        Err(e) => {
            println!("{e}");
            return FAIL;
        }
    };
    let recorded = match record(call, &board) {
        Ok(()) => true,
        Err(e) => {
            println!("{e}");
            false
        }
    };
    let sent = match reach_out(call, &board) {
        Ok(sent) => sent,
        Err(e) => {
            println!("{e}");
            false
        }
    };
    println!("{}", url::line(&board.url));
    if recorded && sent { 0 } else { FAIL }
}

/// project の最新の知らせの記録を書く（置き場は環境の XDG_STATE_HOME と HOME で決める・前の記録は消える）。
fn record(call: &NotifyCall, board: &Board) -> Result<(), String> {
    let own = project(&call.repo)?;
    let path = notify::path(
        env::var_os("XDG_STATE_HOME").as_deref(),
        env::var_os("HOME").as_deref(),
        &own,
    )
    .ok_or_else(|| {
        "知らせの記録の path を決められない（XDG_STATE_HOME も HOME も絶対の path でない）".to_string()
    })?;
    notify::save(
        &path,
        &Record {
            at: now(),
            project: own,
            title: call.title.clone(),
            url: board.url.clone(),
        },
    )
}

/// 器の検めの後に表示先の端末を決め（--to を省けば表示先の設定の効く値・印は読まず書かない）、知らせを撃って終わりの 1 行を出す。
/// 知らせが届いた時だけ真。設定に値が無ければ撃たずにその 1 行を出す。
fn reach_out(call: &NotifyCall, board: &Board) -> Result<bool, String> {
    let text = face_text(&call.repo, &call.scribe2, &call.git)?;
    let name = match &call.to {
        Some(to) => to.clone(),
        None => {
            let own = project(&call.repo)?;
            let path = config_path(call.config.as_deref())?;
            match aim(None, &own, &target::load(&path)?, &terminal::names(&text))? {
                Aim::Named(name) | Aim::Chosen { name, .. } => name,
                Aim::Unset => {
                    println!(
                        "表示先の設定に project {own} の値も既定も無いので端末に知らせを出さない（持ち主へは board の URL を渡し、表示先は tz stage target set で決める）"
                    );
                    return Ok(false);
                }
            }
        }
    };
    let terminal = terminal::lookup(&text, &name)?;
    let outcome = notify::send(&call.ssh, &terminal, &call.title, &board.url, TIMEOUT)?;
    println!("{}", notify::line(&name, &outcome));
    Ok(outcome == Outcome::Sent)
}

/// 自分の anchor の state dir を引き、器の rules validate --state-dir が rc 0 で返った後にだけ host の面の字を読む。
fn face_text(repo: &Path, scribe2: &OsStr, git: &OsStr) -> Result<String, String> {
    let mut args = vec![OsString::from("-C"), repo.as_os_str().to_os_string()];
    args.extend(acct::GIT_ARGS.iter().map(OsString::from));
    let state = proc::capture(git, &args, repo, TIMEOUT)
        .map(|out| String::from_utf8_lossy(&out).trim().to_string())
        .filter(|s| !s.is_empty())
        .ok_or_else(|| {
            format!(
                "{} の git config の scribe2.statedir が読めない（自分の anchor の state dir が無い）",
                repo.display()
            )
        })?;
    let mut validate: Vec<OsString> = VALIDATE_ARGS.iter().map(OsString::from).collect();
    validate.push(OsString::from(&state));
    if proc::capture(scribe2, &validate, repo, TIMEOUT).is_none() {
        return Err(format!(
            "器 {} の {} {state} が {} 秒の内に rc 0 で返らない（host の面を読まない）",
            scribe2.to_string_lossy(),
            VALIDATE_ARGS.join(" "),
            TIMEOUT.as_secs_f32()
        ));
    }
    terminal::read_face(&repo.join(&state))
}

/// 持ち主が窓を開く（端末ごとの錠を持ったまま、覚えた頁か board の頁を使うか、無ければ起動の引数を 1 回だけ撃つ）。
/// 窓を起こした時は錠の中で表示先の設定に端末の印を書く（印が在れば書かない）。見つけた窓の頁は端末ごとの覚えに書く。
/// 設定の path が決まらないか設定が読めなければ ssh を撃つ前に断る。
fn open(
    call: &Call,
    terminal: &Terminal,
    base: &Path,
    board: &Board,
    own: Option<&str>,
) -> Result<(), String> {
    let name = &terminal.name;
    let _held = lock(base, name, LOCK_WAIT)?;
    let config = config_path(call.config.as_deref())?;
    target::load(&config)?;
    let dir = tunnel::socket_dir(base)?;
    let tunnel = Tunnel::open(&call.ssh, terminal, &dir, TIMEOUT)?;
    let hint = recall(base, name, own);
    let (resource, line, shown) = match tunnel.window(board, hint.as_deref(), true)? {
        Window::Page {
            resource,
            launched: true,
        } => {
            let line = format!("端末 {name} に board {} の表示面の窓を起こした", board.url);
            (resource, line, true)
        }
        Window::Page { resource, .. } => {
            let named = match tunnel.page_url(&resource) {
                Some(page) if !board.shows(&page) => format!("・頁は {page}"),
                _ => String::new(),
            };
            let line = format!(
                "端末 {name} の board {} の表示面の窓は在る（起こさない{named}）",
                board.url
            );
            (resource, line, false)
        }
        Window::Absent(line) => return Err(line),
    };
    println!("{line}");
    remember(base, name, own, &resource);
    if shown && target::mark(&config, name, now())? {
        println!(
            "表示先の設定に端末 {name} の印を書いた（持ち主が閉じた後は --to を省いた撃ちで起こし直さない）"
        );
    }
    Ok(())
}

/// 命令を順に撃ち（どの命令の Err でもその後の命令を撃たない）、終わりに Session を閉じる。
fn drive(
    mut session: Session,
    script: &[(Command, Option<PathBuf>)],
    board: &Board,
    ports: &[u16],
) -> Result<(), String> {
    let done = script.iter().try_for_each(|(command, out)| {
        shoot(&mut session, command, out.as_deref(), board, ports)
            .map_err(|e| format!("{}: {e}", word(command)))
    });
    let closed = session.close();
    done.and(closed)
}

/// 1 つの命令を撃って出力の行を書く（どの命令の前にも頁の URL を読み、自分の board の頁でない board の頁の上ではどの命令も、
/// 自分の board の頁の上の click・入力・key も撃たずに断る・行 i-stage-guard）。
fn shoot(
    session: &mut Session,
    command: &Command,
    out: Option<&Path>,
    board: &Board,
    ports: &[u16],
) -> Result<(), String> {
    let verb = word(command);
    let page = session.url()?;
    if board.foreign(&page, ports) {
        return Err(format!(
            "頁 {page} は自分の project board でない board の頁なので {verb} を断る（判断の記録 ADR-15 の決定 (7)・席の自分の board の頁のほかでは見せる操作も撃たない）"
        ));
    }
    if on_boards(command, &page, board, ports) {
        return Err(format!(
            "board の頁 {page} の上では {verb} を断る（判断の記録 ADR-15 の決定 (7)・board は見せるだけ）"
        ));
    }
    let from = session.events().len();
    let replies = session.run(command)?;
    let first = replies.first().map(String::as_str);
    let result = first.and_then(|r| json::member(r, "result"));
    match command {
        Command::Screenshot => {
            let bytes = result
                .and_then(|r| json::member(r, "data"))
                .and_then(json::unquote)
                .as_deref()
                .and_then(unbase64)
                .ok_or_else(|| "応答の result の data が base64 の字でない".to_string())?;
            let path = out.ok_or_else(|| "写真の書き先が無い".to_string())?;
            fs::write(path, &bytes)
                .map_err(|e| format!("写真を {} に書けない: {e}", path.display()))?;
            println!("済み screenshot {} {} byte", path.display(), bytes.len());
        }
        Command::Dom => {
            let text = result
                .and_then(|r| json::member(r, "result"))
                .and_then(|r| json::member(r, "value"))
                .and_then(json::unquote)
                .ok_or_else(|| "応答の result の result の value が字でない".to_string())?;
            println!("{text}");
            println!("済み dom");
        }
        Command::Console => {
            for event in &session.events()[from..] {
                let method = json::member(event, "method").and_then(json::unquote);
                if matches!(
                    method.as_deref(),
                    Some("Runtime.consoleAPICalled" | "Log.entryAdded")
                ) {
                    println!("{event}");
                }
            }
            println!("済み console");
        }
        _ => println!("済み {verb}"),
    }
    Ok(())
}
