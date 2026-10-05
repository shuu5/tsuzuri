//! tz consult（相談の窓の命令・判断の記録 ADR-29 決定 (2)〜(10)・設計ノート surface-wave27a 行 cs-open〜cs-close）。
//! 口は open と bundle（行 cs-open）・launch（行 cs-launch）・answer と guard（行 cs-answer）・watch（行 cs-watch）・
//! show と dispose（行 cs-show）・list と close（行 cs-close）・stamp（行 cs-acct-mark・窓の会話の印の hook）・statusline（行 cs-status-verb・窓の状態の 1 行）。終了 code は 0 合格・1 断り（使い方の誤り・欄の欠け・
//! 上限・版のずれ）・2 まだ分からない（台帳か置き場が読めない）。
//! project の値は repo の git config の鍵から引く（state dir は `scribe2.statedir`・起草の置き場は `tsuzuri.draftsdir`・
//! host ごとに分岐しない）。鍵が無ければ rc 2。窓の作業場は起草の置き場の下の `consult-cw<n>`、退いた作業場は
//! `retired-consult-cw<n>`。この module は口の振り分けと、引数の読み・置き場の解き方・作業場の file の読み書き・
//! 台帳の読みと相談の行の書きの共通の手を持つ（行の字と置き場は中核の `consult::lines`）。
//! 口座の信頼の印の書き手（`trust`）は行 cs-trust（ノート surface-wave29b）が置く。

pub mod answer;
pub mod bundle;
pub mod close;
pub mod dispose;
pub mod guard;
pub mod launch;
pub mod list;
pub mod open;
pub mod plain;
pub mod show;
pub mod stamp;
pub mod statusline;
pub mod trust;
pub mod watch;

use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};
use std::time::Duration;

use tsuzuri_contract::board::Reading;
use tsuzuri_contract::consult::{FindingId, ProcMark, WindowFile, WindowId};
use tsuzuri_contract::ledger::{BDW, BeadId, LedgerItem, LedgerWrite};
use tsuzuri_contract::wire;
use tsuzuri_core::consult::lines::{Line, home_of, render, scan};

use crate::acct::{GIT, GIT_ARGS};
use crate::out::emit_err;
use crate::server::events::now;
use crate::server::ledger::{BD, Source, capture, parse_bd};
use crate::server::ruling::{WRITE_TIMEOUT, minute};

/// 断り（使い方の誤り・欄の欠け・上限・版のずれ）。
pub const FAIL: u8 = 1;

/// まだ分からない（台帳か置き場が読めない）。
pub const UNKNOWN: u8 = 2;

/// 起草の置き場を引く git の引数の列の後ろ（前に `-C <repo>` が付く・acct の `GIT_ARGS` と `BOARD_ARGS` と同じ形）。
pub const DRAFTS_ARGS: [&str; 3] = ["config", "--get", "tsuzuri.draftsdir"];

/// cwd の git の根を引く引数の列。
pub const TOPLEVEL_ARGS: [&str; 2] = ["rev-parse", "--show-toplevel"];

/// git の撃ちの上限。
pub const GIT_TIMEOUT: Duration = Duration::from_secs(5);

/// 窓の控えの file（作業場からの相対）。
pub const WINDOW_FILE: &str = ".consult/window.json";

/// 作業場に用意する dir（控え・束・草稿・所見・実験）。
pub const DIRS: [&str; 5] = [".consult", "bundle", "drafts", "findings", "work"];

/// 窓の控えの書き場（作業場からの相対）。
pub const NOTES: &str = "notes.md";

/// 席への 1 行の命令の頭の字を渡す環境変数（plugin の tz の解き方が自分の path を置く・無ければ tz 自身の path）。
pub const WRAPPER_ENV: &str = "TZ_WRAPPER";

/// 共通の値を取る旗（repo の置き場・台帳の読みと書きの program）。
pub const COMMON: [&str; 3] = ["--repo", "--bd", "--bdw"];

pub const USAGE: &str =
    "usage: tz consult <open|bundle|launch|answer|watch|show|dispose|list|close|guard|stamp|statusline> [引数]";

/// tz consult の残りの引数を受けて終了 code を返す。
pub fn run(rest: &[&str]) -> u8 {
    match rest {
        ["open", r @ ..] => open::run(r),
        ["bundle", r @ ..] => bundle::run(r),
        ["launch", r @ ..] => launch::run(r),
        ["answer", r @ ..] => answer::run(r),
        ["watch", r @ ..] => watch::run(r),
        ["show", r @ ..] => show::run(r),
        ["dispose", r @ ..] => dispose::run(r),
        ["list", r @ ..] => list::run(r),
        ["close", r @ ..] => close::run(r),
        ["guard", r @ ..] => guard::run(r),
        ["stamp", r @ ..] => stamp::run(r),
        ["statusline", r @ ..] => statusline::run(r),
        _ => {
            emit_err(&format!("tz consult: 口が無い\n{USAGE}"));
            FAIL
        }
    }
}

/// 断りの終了 code と理由。
pub type Refused = (u8, String);

/// 口の名を付けて理由を標準エラーに書き、終了 code を返す。
pub fn refuse(verb: &str, (rc, why): Refused) -> u8 {
    emit_err(&format!("tz consult {verb}: {why}"));
    rc
}

/// 読んだ引数（位置の引数・旗の値・値を取らない旗）。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Flags {
    pub pos: Vec<String>,
    values: Vec<(String, String)>,
    switches: Vec<String>,
}

impl Flags {
    /// 旗の値（何度でも受ける旗は最初の値）。
    pub fn get(&self, name: &str) -> Option<&str> {
        self.values
            .iter()
            .find(|(n, _)| n == name)
            .map(|(_, v)| v.as_str())
    }

    /// 何度でも受ける旗の値の全部（順のまま）。
    pub fn all(&self, name: &str) -> Vec<&str> {
        self.values
            .iter()
            .filter(|(n, _)| n == name)
            .map(|(_, v)| v.as_str())
            .collect()
    }

    /// 値を取らない旗が在るか。
    pub fn has(&self, name: &str) -> bool {
        self.switches.iter().any(|s| s == name)
    }
}

/// `--名 値` か `--名=値` の旗（`values` の名・`many` の名は何度でも）と値を取らない旗（`switches`）と
/// 位置の引数を読む。知らない旗・空の値・2 度の旗（`many` を除く）を断る（rc 1）。
pub fn flags(
    rest: &[&str],
    values: &[&str],
    switches: &[&str],
    many: &[&str],
) -> Result<Flags, Refused> {
    let mut f = Flags::default();
    let mut it = rest.iter();
    while let Some(arg) = it.next() {
        if !arg.starts_with("--") {
            f.pos.push((*arg).to_string());
            continue;
        }
        if switches.contains(arg) {
            if f.has(arg) {
                return Err((FAIL, format!("{arg} が 2 度ある")));
            }
            f.switches.push((*arg).to_string());
            continue;
        }
        let (name, value) = match arg.split_once('=') {
            Some((n, v)) => (n, v),
            None => (*arg, *it.next().ok_or((FAIL, format!("{arg} の値が無い")))?),
        };
        if !values.contains(&name) && !many.contains(&name) {
            return Err((FAIL, format!("知らない引数 {name}")));
        }
        if value.is_empty() {
            return Err((FAIL, format!("{name} の値が空")));
        }
        if !many.contains(&name) && f.get(name).is_some() {
            return Err((FAIL, format!("{name} が 2 度ある")));
        }
        f.values.push((name.to_string(), value.to_string()));
    }
    Ok(f)
}

/// 口の置き場（repo・state dir・起草の置き場・台帳の読みと書きの program）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ctx {
    pub repo: PathBuf,
    pub state: PathBuf,
    pub drafts: PathBuf,
    pub bd: OsString,
    pub bdw: OsString,
}

/// git を撃ち、返した字の前後の空白を除いた字（落ちる・空なら None）。
fn git(cwd: &Path, args: &[&OsStr]) -> Option<String> {
    let out = capture(OsStr::new(GIT), args, cwd, GIT_TIMEOUT)?;
    let text = String::from_utf8(out).ok()?;
    let text = text.trim();
    (!text.is_empty()).then(|| text.to_string())
}

/// repo の git config の鍵の値。
fn config(repo: &Path, tail: &[&str]) -> Option<String> {
    let mut args: Vec<&OsStr> = vec![OsStr::new("-C"), repo.as_os_str()];
    args.extend(tail.iter().map(OsStr::new));
    git(repo, &args)
}

/// 置き場を解く（repo は --repo か cwd の git の根・state dir と起草の置き場は repo の git config の鍵・
/// どれかが引けないか起草の置き場が dir でなければ rc 2）。
pub fn ctx(f: &Flags) -> Result<Ctx, Refused> {
    let repo = match f.get("--repo") {
        Some(r) => PathBuf::from(r),
        None => {
            let cwd =
                std::env::current_dir().map_err(|e| (UNKNOWN, format!("cwd が読めない: {e}")))?;
            let args: Vec<&OsStr> = TOPLEVEL_ARGS.iter().map(OsStr::new).collect();
            PathBuf::from(git(&cwd, &args).ok_or((UNKNOWN, "cwd が git の木でない".to_string()))?)
        }
    };
    if !repo.is_dir() {
        return Err((UNKNOWN, format!("repo {} が dir でない", repo.display())));
    }
    let state =
        config(&repo, &GIT_ARGS).ok_or((UNKNOWN, "鍵 scribe2.statedir が無い".to_string()))?;
    let drafts =
        config(&repo, &DRAFTS_ARGS).ok_or((UNKNOWN, "鍵 tsuzuri.draftsdir が無い".to_string()))?;
    let drafts = PathBuf::from(drafts);
    if !drafts.is_dir() {
        return Err((
            UNKNOWN,
            format!("起草の置き場 {} が dir でない", drafts.display()),
        ));
    }
    Ok(Ctx {
        repo,
        state: PathBuf::from(state),
        drafts,
        bd: f.get("--bd").unwrap_or(BD).into(),
        bdw: f.get("--bdw").unwrap_or(BDW).into(),
    })
}

/// 窓の作業場の path。
pub fn workspace(drafts: &Path, id: WindowId) -> PathBuf {
    drafts.join(id.name())
}

/// 退いた作業場の path。
pub fn retired(drafts: &Path, id: WindowId) -> PathBuf {
    drafts.join(id.retired_name())
}

/// 起草の置き場の窓の id と退いたか（作業場と退いた作業場の dir の名から・id の順）。
pub fn windows(drafts: &Path) -> Vec<(WindowId, bool)> {
    let mut out: Vec<(WindowId, bool)> = std::fs::read_dir(drafts)
        .into_iter()
        .flatten()
        .flatten()
        .filter(|e| e.path().is_dir())
        .filter_map(|e| {
            let name = e.file_name().into_string().ok()?;
            let (rest, gone) = match name.strip_prefix("retired-") {
                Some(rest) => (rest.to_string(), true),
                None => (name, false),
            };
            let id = WindowId::parse(rest.strip_prefix("consult-")?).ok()?;
            Some((id, gone))
        })
        .collect();
    out.sort();
    out
}

/// 作業場の窓の控え（読めなければ None）。
pub fn read_window(ws: &Path) -> Option<WindowFile> {
    let text = std::fs::read_to_string(ws.join(WINDOW_FILE)).ok()?;
    wire::decode(&text).ok()
}

/// 窓の控えを書く。
pub fn write_window(ws: &Path, window: &WindowFile) -> Result<(), String> {
    let text = wire::encode(window).map_err(|e| e.to_string())?;
    std::fs::write(ws.join(WINDOW_FILE), text + "\n").map_err(|e| e.to_string())
}

/// 作業場の findings/ の所見の id（名が `<所見 id>.json` で窓の id が `window` の物・id の順）。
pub fn findings(ws: &Path, window: WindowId) -> Vec<FindingId> {
    let mut out: Vec<FindingId> = std::fs::read_dir(ws.join("findings"))
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|e| {
            let name = e.file_name().into_string().ok()?;
            FindingId::parse(name.strip_suffix(".json")?).ok()
        })
        .filter(|id| id.window() == window)
        .collect();
    out.sort();
    out
}

/// 作業場の process の印（`.consult/proc-<k>.json` の読める物・k の順）。
pub fn procs(ws: &Path) -> Vec<ProcMark> {
    let mut out: Vec<ProcMark> = std::fs::read_dir(ws.join(".consult"))
        .into_iter()
        .flatten()
        .flatten()
        .filter(|e| {
            e.file_name()
                .to_str()
                .is_some_and(|n| n.starts_with("proc-") && n.ends_with(".json"))
        })
        .filter_map(|e| wire::decode::<ProcMark>(&std::fs::read_to_string(e.path()).ok()?).ok())
        .collect();
    out.sort_by_key(|p| p.k);
    out
}

/// process の印の file の path。
pub fn proc_path(ws: &Path, k: u32) -> PathBuf {
    ws.join(format!(".consult/proc-{k}.json"))
}

/// pid の process が在るか（`/proc/<pid>` の在る無し）。
pub fn alive(pid: u32) -> bool {
    pid > 0 && Path::new("/proc").join(pid.to_string()).exists()
}

/// 最後の process の印の process が在るか。
pub fn live(ws: &Path) -> bool {
    procs(ws).last().is_some_and(|p| alive(p.pid))
}

/// 今の UTC の分の字。
pub fn minute_now() -> String {
    minute(now())
}

/// 解いた tz の path（tz 自身の binary・読めなければ字 tz）。
pub fn tz_path() -> String {
    std::env::current_exe().map_or_else(|_| "tz".to_string(), |p| p.display().to_string())
}

/// 席への 1 行の命令の頭の字（環境の `WRAPPER_ENV` か、無ければ tz 自身の path）。
pub fn tzw() -> String {
    std::env::var(WRAPPER_ENV)
        .ok()
        .filter(|v| !v.is_empty())
        .unwrap_or_else(tz_path)
}

/// 台帳を bd で読む（bd の字と bead の列・読めなければ rc 2）。
pub fn ledger(ctx: &Ctx) -> Result<(String, Vec<LedgerItem>), Refused> {
    let text = Source::new(&ctx.repo, &ctx.bd)
        .text_alone()
        .ok_or((UNKNOWN, "台帳が読めない".to_string()))?;
    match parse_bd(&text) {
        Reading::Known(items) => Ok((text, items)),
        Reading::Unknown => Err((UNKNOWN, "台帳の字が読めない".to_string())),
    }
}

/// 台帳の notes の相談の行の全部（bead の順・notes の中の順）。
pub fn lines_of(items: &[LedgerItem]) -> Vec<Line> {
    items.iter().flat_map(|i| scan(&i.notes)).collect()
}

/// 相談の行を置き場（題から中核の `home_of` で選ぶ memo か根）の notes に bdw で足し、置いた bead の id を返す
/// （行の字が書けなければ rc 1・根が無いか書きが落ちれば rc 2）。
pub fn append(
    ctx: &Ctx,
    items: &[LedgerItem],
    topic: Option<&str>,
    line: &Line,
) -> Result<BeadId, Refused> {
    let text = render(line).map_err(|e| (FAIL, format!("行を書けない: {e:?}")))?;
    let home =
        home_of(items, topic, &text).map_err(|e| (UNKNOWN, format!("置き場が無い: {e:?}")))?;
    let write = LedgerWrite::AppendNotes {
        id: home.id.clone(),
        line: home.line(&text),
    };
    capture(&ctx.bdw, write.argv(), &ctx.repo, WRITE_TIMEOUT)
        .ok_or((UNKNOWN, format!("台帳の書きが落ちた: {}", home.id)))?;
    Ok(home.id)
}
