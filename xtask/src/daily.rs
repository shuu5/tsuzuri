//! 日に 1 度の全部の撃ち（行 v-daily・判断の記録 ADR-34 の決定 (6)）。host の利用者の timer が、作業木の外の固定の置き場の写しで
//! `cargo run -q -p xtask -- daily` を撃つ。写しの origin の main を fetch して detach で checkout し、入れ子の段に全部を撃たせる
//! 環境変数を置いて xtask の check を nice -n 10 で撃ち、記録の file に 1 行足す。落ちた時は席への memo を台帳に起こし
//! （落ちが続く間は同じ memo の notes に [再発] の行を足す・その memo が open か in_progress でないか読めないか notes が満ちる時は、足さずに
//! 前の memo の id を本文に書いた新しい memo を起こす）、通った時は memo を書かない。host の値（記録の file・出力の file・
//! 台帳の作業木・memo の親・bdw の program）は引数で受け、code を分けない（条 N-2）。組みの置き場は撃つ側の CARGO_TARGET_DIR。

use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

use tsuzuri_contract::ledger::BdLine;
use tsuzuri_contract::wire;

/// 撃ち方。
pub const USAGE: &str = "usage: cargo run -q -p xtask -- daily --log <記録の file> --out <出力の file> --ledger <台帳の作業木> --parent <memo の親の bead> [--bdw <program>]";

/// check に置く環境変数（入れ子の段に全部を撃たせる・行 v-skip の FORCE_ENV と同じ名）。
pub const FORCE: (&str, &str) = ("TSUZURI_CHECK_NESTED_ALL", "1");

/// 写しを送り先の main に合わせる fetch の引数（git -C <写し> の後）。
pub const FETCH: &[&str] = &[
    "fetch",
    "-q",
    "origin",
    "+refs/heads/main:refs/remotes/origin/main",
];

/// fetch した main を detach で取り出す引数。
pub const CHECKOUT: &[&str] = &["checkout", "-q", "--detach", "refs/remotes/origin/main"];

/// check を撃つ argv の頭（nice の program と引数・その後に cargo の program を置く）と cargo の引数。
pub const NICE: (&str, &[&str]) = ("nice", &["-n", "10"]);
pub const CHECK: &[&str] = &["run", "-q", "-p", "xtask", "--", "check"];

/// 続く落ちの前に memo の状態を読む bdw の引数の頭（bd の READ の素通し・台帳を書かない・その後に id と --json）。
pub const SHOW: &[&str] = &["--readonly", "show"];

/// memo の notes の上限 byte（器の rules 行 memo.notes_max_bytes・越えた open な memo を doctor が oversized に数える）。
pub const NOTES_MAX: usize = 8192;

/// 撃つ物の在りか。
#[derive(Debug, PartialEq, Eq)]
pub struct Daily {
    /// 撃つ写しの根（git の作業木）。
    pub root: PathBuf,
    /// 記録の file（1 撃ち 1 行を足す）。
    pub log: PathBuf,
    /// check の出力の file（撃つたびに書き直す）。
    pub out: PathBuf,
    /// 台帳を持つ作業木（bdw の cwd）。
    pub ledger: PathBuf,
    /// memo の親の bead。
    pub parent: String,
    /// cargo の program。
    pub cargo: String,
    /// bdw の program。
    pub bdw: String,
}

/// 引数を読む（--log・--out・--ledger・--parent は要る・--bdw は既定 bdw・知らない旗と 2 度の旗と値の欠けは断る）。
pub fn parse(args: &[String], root: &Path, cargo: &str) -> Result<Daily, String> {
    let mut got: Vec<(&str, &str)> = Vec::new();
    let mut it = args.iter();
    while let Some(flag) = it.next() {
        let name = match flag.as_str() {
            "--log" | "--out" | "--ledger" | "--parent" | "--bdw" => flag.as_str(),
            other => return Err(format!("知らない引数: {other}")),
        };
        let value = it
            .next()
            .filter(|v| !v.is_empty())
            .ok_or_else(|| format!("{name} の値が無い"))?;
        if got.iter().any(|(n, _)| *n == name) {
            return Err(format!("{name} が 2 度在る"));
        }
        got.push((name, value));
    }
    let get = |name: &str| {
        got.iter()
            .find(|(n, _)| *n == name)
            .map(|(_, v)| (*v).to_string())
    };
    let need = |name: &str| get(name).ok_or_else(|| format!("{name} が無い"));
    Ok(Daily {
        root: root.to_path_buf(),
        log: PathBuf::from(need("--log")?),
        out: PathBuf::from(need("--out")?),
        ledger: PathBuf::from(need("--ledger")?),
        parent: need("--parent")?,
        cargo: cargo.to_string(),
        bdw: get("--bdw").unwrap_or_else(|| "bdw".to_string()),
    })
}

/// 撃ちの結果（返す rc と記録の 1 行）。
#[derive(Debug, PartialEq, Eq)]
pub struct Outcome {
    pub rc: i32,
    pub line: String,
}

/// fetch・checkout・check を順に撃ち、記録の file に 1 行足し、落ちた時は memo を起こすか足す（now は epoch 秒）。
pub fn run(d: &Daily, now: u64) -> Outcome {
    let prev = std::fs::read_to_string(&d.log).unwrap_or_default();
    let started = std::time::Instant::now();
    let (step, rc) = shoot(d);
    let sha = git_text(&d.root, &["rev-parse", "HEAD"]).unwrap_or_else(|| "-".to_string());
    let sha = sha.get(..12).unwrap_or(&sha).to_string();
    let seen = Seen {
        stamp: stamp(now),
        sha,
        step,
        rc,
        secs: started.elapsed().as_secs(),
    };
    let memo = if rc == 0 {
        "-".to_string()
    } else {
        tell(d, &prev, &seen)
    };
    let line = record(&seen, &memo);
    let rc = match append(&d.log, &line) {
        Ok(()) => rc,
        Err(_) if rc == 0 => 1,
        Err(_) => rc,
    };
    Outcome { rc, line }
}

/// 撃ち 1 回の材料（時刻・sha の頭 12 字・落ちた段か check・rc・秒）。
#[derive(Debug, PartialEq, Eq)]
pub struct Seen {
    pub stamp: String,
    pub sha: String,
    pub step: &'static str,
    pub rc: i32,
    pub secs: u64,
}

/// 段を順に撃ち、最初に落ちた段の名と rc を返す（全部通れば check と 0）。
fn shoot(d: &Daily) -> (&'static str, i32) {
    for (step, args) in [("fetch", FETCH), ("checkout", CHECKOUT)] {
        let rc = status(
            Command::new("git")
                .arg("-C")
                .arg(&d.root)
                .args(args)
                .stdout(Stdio::null())
                .stderr(Stdio::null()),
        );
        if rc != 0 {
            return (step, rc);
        }
    }
    let out = match std::fs::File::create(&d.out) {
        Ok(f) => f,
        Err(_) => return ("out", 1),
    };
    let err = match out.try_clone() {
        Ok(f) => f,
        Err(_) => return ("out", 1),
    };
    let rc = status(
        Command::new(NICE.0)
            .args(NICE.1)
            .arg(&d.cargo)
            .args(CHECK)
            .current_dir(&d.root)
            .env(FORCE.0, FORCE.1)
            .stdout(out)
            .stderr(err),
    );
    ("check", rc)
}

/// 撃って rc（起動できなければ 127・signal で止まれば 1）。
fn status(cmd: &mut Command) -> i32 {
    match cmd.status() {
        Ok(s) => s.code().unwrap_or(1),
        Err(_) => 127,
    }
}

/// git の標準出力の字（前後の空白を除く・落ちれば None）。
fn git_text(root: &Path, args: &[&str]) -> Option<String> {
    let out = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(args)
        .output()
        .ok()?;
    out.status
        .success()
        .then(|| String::from_utf8_lossy(&out.stdout).trim().to_string())
}

/// 今の epoch 秒。
pub fn now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs())
}

/// 記録の 1 行（時刻・sha の頭 12 字・落ちた段か check・rc・秒・memo の id か -）。
pub fn record(seen: &Seen, memo: &str) -> String {
    let Seen {
        stamp,
        sha,
        step,
        rc,
        secs,
    } = seen;
    format!("{stamp} sha={sha} step={step} rc={rc} secs={secs} memo={memo}")
}

/// 記録の最後の行が落ちで memo の id を持てば、その id（続く落ちは同じ memo に足す）。
pub fn open_memo(log: &str) -> Option<String> {
    let last = log.lines().rev().find(|l| !l.trim().is_empty())?;
    let field = |key: &str| {
        last.split_whitespace()
            .find_map(|w| w.strip_prefix(key).map(str::to_string))
    };
    let failed = field("rc=").is_some_and(|rc| rc != "0");
    field("memo=").filter(|m| failed && m != "-" && m != "failed")
}

/// 落ちの memo を起こすか、続く落ちなら同じ memo の notes に [再発] の行を足し、記録に置く id（書けなければ failed）。
fn tell(d: &Daily, prev: &str, seen: &Seen) -> String {
    let note = recur_note(seen);
    let before = open_memo(prev);
    if let Some(id) = &before
        && appendable(id, show(d, id).as_deref(), &note)
    {
        let ok = bdw(
            d,
            &[
                "update".to_string(),
                id.clone(),
                format!("--append-notes={note}"),
            ],
        )
        .is_some();
        return if ok { id.clone() } else { "failed".to_string() };
    }
    let body = d.log.with_extension("memo.md");
    if std::fs::write(&body, memo_body(seen, &d.out, &d.log, before.as_deref())).is_err() {
        return "failed".to_string();
    }
    bdw(d, &memo_argv(&d.parent, &body, &seen.stamp, &seen.sha))
        .map(|out| String::from_utf8_lossy(&out.stdout).trim().to_string())
        .filter(|id| !id.is_empty() && !id.contains(char::is_whitespace))
        .unwrap_or_else(|| "failed".to_string())
}

/// 記録の memo に [再発] の行を足してよいか。bd の show の字（bdw が落ちれば None）が id の bead 1 本だけで、status が open か in_progress で、
/// 足した後の notes（区切りの改行 1 byte を含む）が NOTES_MAX 以下の時だけ真（閉じた・台帳に無い・読めない・満ちる時は新しい memo）。
pub fn appendable(id: &str, shown: Option<&str>, note: &str) -> bool {
    let Some(Ok(lines)) = shown.map(wire::decode::<Vec<BdLine>>) else {
        return false;
    };
    let [line] = lines.as_slice() else {
        return false;
    };
    line.id.as_str() == id
        && matches!(line.status.as_str(), "open" | "in_progress")
        && line.notes.len() + 1 + note.len() <= NOTES_MAX
}

/// memo の bd の show の字（台帳の作業木で bdw --readonly show <id> --json・落ちれば None）。
fn show(d: &Daily, id: &str) -> Option<String> {
    let mut args: Vec<String> = SHOW.iter().map(|a| (*a).to_string()).collect();
    args.extend([id.to_string(), "--json".to_string()]);
    bdw(d, &args).map(|out| String::from_utf8_lossy(&out.stdout).into_owned())
}

/// bdw を台帳の作業木で撃つ（rc 0 の時だけ出力）。
fn bdw(d: &Daily, args: &[String]) -> Option<Output> {
    let out = Command::new(&d.bdw)
        .args(args)
        .current_dir(&d.ledger)
        .output()
        .ok()?;
    out.status.success().then_some(out)
}

/// memo を起こす bdw の引数（親・題・本文の file・label intake:memo・metadata の short）。
pub fn memo_argv(parent: &str, body: &Path, stamp: &str, sha: &str) -> Vec<String> {
    vec![
        "create".to_string(),
        format!("--parent={parent}"),
        "--type=task".to_string(),
        "--priority=3".to_string(),
        "--labels=intake:memo".to_string(),
        "--no-inherit-labels=true".to_string(),
        r#"--metadata={"short":"日次の全部の撃ちが落ちた"}"#.to_string(),
        "--silent=true".to_string(),
        format!("--body-file={}", body.display()),
        "--".to_string(),
        format!("memo 日に 1 度の全部の撃ちが落ちた（{stamp}・origin/main {sha}）"),
    ]
}

/// 続く落ちの notes の 1 行。
pub fn recur_note(seen: &Seen) -> String {
    let Seen {
        stamp,
        sha,
        step,
        rc,
        ..
    } = seen;
    format!("[再発] {stamp} origin/main {sha} の段 {step} が rc {rc}")
}

/// memo の本文（出所・観測・候補・昇格条件の 4 節・path は file の名だけ・足さなかった前の memo が在ればその id）。
fn memo_body(seen: &Seen, out: &Path, log: &Path, before: Option<&str>) -> String {
    let name = |p: &Path| {
        p.file_name()
            .map_or_else(String::new, |n| n.to_string_lossy().into_owned())
    };
    let before = before.map_or_else(String::new, |id| {
        format!("前の memo {id} は open でも in_progress でもないか、読めないか、notes が上限 {NOTES_MAX} byte を越えるので、足さずにこの memo を起こした。\n")
    });
    format!(
        "### 出所\nxtask の daily（判断の記録 ADR-34 の決定 (6)・行 v-daily・host の利用者の timer）。\n{}\n\
         ### 観測\n{} に origin/main {} の木で、環境変数 {}={} を置いた xtask の check を撃ち、段 {} が rc {} で落ちた（{} 秒）。出力は file {}、記録は file {}（どちらも撃つ写しの置き場の側）。\n\n\
         ### 候補\n出力の末で落ちた段と歯を見る。host の道具と設定のずれ（git・tmux・nextest・rustup の版・$CARGO_HOME と祖先の .cargo の設定）か、時刻で落ちる型か、混んだ時だけ揺れる歯かを見分ける。\n\n\
         ### 昇格条件\n引き金: 再発 2\n",
        before,
        seen.stamp,
        seen.sha,
        FORCE.0,
        FORCE.1,
        seen.step,
        seen.rc,
        seen.secs,
        name(out),
        name(log)
    )
}

/// 記録の file に 1 行足す（無ければ作る）。
fn append(path: &Path, line: &str) -> std::io::Result<()> {
    use std::io::Write;
    let mut f = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)?;
    writeln!(f, "{line}")
}

/// epoch 秒の UTC の分の字（年-月-日 T 時:分 Z）。
pub fn stamp(secs: u64) -> String {
    let days = secs / 86_400;
    let (h, m) = ((secs % 86_400) / 3_600, (secs % 3_600) / 60);
    // 日の数から年月日（Howard Hinnant の civil_from_days・3 月始まりの年）。
    let z = days + 719_468;
    let era = z / 146_097;
    let doe = z % 146_097;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let mo = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + u64::from(mo <= 2);
    format!("{y:04}-{mo:02}-{d:02}T{h:02}:{m:02}Z")
}
