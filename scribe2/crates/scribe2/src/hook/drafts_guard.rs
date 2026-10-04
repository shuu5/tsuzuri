//! 席の起草の写しを、起草の置き場の直下に限る門（設計 docs/design/vessel-hook.md §25・契約表の行 q・SRS FR42 / FR68 / NFR5・
//! [ADR-0096](../../../../design-intent/decisions/ADR-0096-seat-drafts-are-vessel-owned-and-swept-after-writes-stop.html)）。
//!
//! 席の Bash の `git worktree add` と `git clone` の行き先を、席の指示文の 12 行目と同じ 1 関数（[`crate::seat::drafts_dir`]）が解く
//! 置き場の直下の子と、anchor の `.worktrees/` の直下の子に限り、置き場の外と解けない行き先と席を解けない周を実行の前に断る。
//! 位置は bypass の門の後ろ・走っている便の行の門の前。行き先の読みは [`copy_places`]（git の segment の読み手の上の pure な 1 関数）。
//!
//! 写しの segment も env の行き先も 0 の周は tmux も event log も読まずに通し、`--pane` が無いか空の周と登録 row の無い席は通す（席でない）。
//!
//! env の行き先（判断の記録 ADR-44 の決定 (3)・行 v-drafts-env）: 同じ門が、組みの置き場の環境変数（[`BUILD_VARS`]）と cargo の
//! `--target-dir` と `TMPDIR` の行き先（[`env_places`]・segment の頭の割り当て・宣言の語と env の後ろの割り当て）も読む。組みの置き場は
//! 起草の置き場の直下の子より下の、掃除の名（[`crate::pipe::sweep::NAMES`]・掃除と同じ列）の dir の中だけを通し、`TMPDIR` は置き場を
//! 問わない。どちらも相対の値・字のまま解けない値・mount の表で tmpfs か ramfs の上の行き先（[`tmpfs_of`]）を断る。写しの断りが先。

use super::command::BASH;
use super::host_guard::verb_of;
use super::ledger_guard::{is_assignment, segments};
use super::live_row::{copy_places, literal, moved, normalized, Dest, Place};
use crate::fleet::{replay, store};
use crate::name::NAME;
use crate::pipe::sweep::NAMES;
use crate::pipe::worktrees_dir;
use crate::polarity::{OnFailure, Polarity, Timing};
use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};

/// 記録の `what` の頭（極性一覧の語と同じ・後ろに理由の語が付く）。
pub const WHAT: &str = "drafts-deny";

/// この境界の極性: 道具の呼び出しの時点で止め、席を解けない周は写しの segment を断る（止める側へ倒れる）。
pub const POLARITY: Polarity = Polarity {
    timing: Timing::InLoop,
    on_failure: OnFailure::FailClosed,
};

/// 組みの産物の置き場を木の外へ向ける環境変数の名（器が対応する言語の閉じた列・Rust と Python・TypeScript は該当なし）。
const BUILD_VARS: [&str; 3] = ["CARGO_TARGET_DIR", "CARGO_BUILD_TARGET_DIR", "PYTHONPYCACHEPREFIX"];

/// 一時の file の置き場の環境変数の名（置き場を問わず、相対・解けない・tmpfs だけを断る）。
const TMP_VAR: &str = "TMPDIR";

/// cargo の組みの置き場の flag（頭の語が cargo の segment だけ・`--` より前）。
const TARGET_FLAG: &str = "--target-dir";

/// cargo の動詞。
const CARGO: &str = "cargo";

/// 後ろの割り当てを読む語（shell の宣言の語と env）。
const DECLARERS: [&str; 6] = ["export", "declare", "typeset", "readonly", "local", "env"];

/// 後ろの segment の作業 dir を替える動詞。
const CD: [&str; 2] = ["cd", "pushd"];

/// 値の頭で解く作業 dir の変数の名。
const PWD: &str = "PWD";

/// mount の表。
const MOUNTS: &str = "/proc/self/mounts";

/// 表を読めない周の表（/dev/shm だけを tmpfs と読む）。
const FALLBACK_MOUNTS: &str = "tmpfs /dev/shm tmpfs rw 0 0";

/// RAM の上の fs の型。
const TMPFS_TYPES: [&str; 2] = ["tmpfs", "ramfs"];

/// mount の点の字の 8 進の escape（表の字・解いた字）。
const ESCAPES: [(&str, &str); 4] = [("\\040", " "), ("\\011", "\t"), ("\\012", "\n"), ("\\134", "\\")];

/// 断る理由の語（閉じた 5 つ・宣言順）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Reason {
    /// 行き先が置き場の直下でない（置き場そのもの・深い所・外）か、組みの置き場が起草の置き場の子の下の掃除の名の dir の外。
    Outside,
    /// 行き先を字面で解けない。
    Unresolved,
    /// 席（tmux の target か event log）を解けない。
    SeatUnresolved,
    /// env の行き先が相対の字。
    Relative,
    /// env の行き先が tmpfs か ramfs の上（RAM の上の置き場）。
    Tmpfs,
}

impl Reason {
    /// 断りの行と記録に出す理由の語。
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Outside => "outside",
            Self::Unresolved => "unresolved",
            Self::SeatUnresolved => "seat-unresolved",
            Self::Relative => "relative",
            Self::Tmpfs => "tmpfs",
        }
    }
}

/// 判定（閉じた 2 値）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DraftsDecision {
    /// 通す。
    Pass,
    /// 断る。
    Deny {
        /// 当たった理由。
        reason: Reason,
        /// stderr の 1 行。
        line: String,
    },
}

/// 判定の場（hook が解いた入力だけ）。
pub struct Scene<'a> {
    /// tool 名。
    pub tool: &'a str,
    /// Bash の command 行。
    pub command: Option<&'a str>,
    /// payload の作業 dir。
    pub cwd: &'a Path,
    /// hook の `--project` の root（anchor）。
    pub root: &'a Path,
    /// hook の置き場。
    pub state_dir: &'a Path,
    /// 自席の pane id（無い・空は席でない）。
    pub pane: Option<&'a str>,
    /// tmux の socket。
    pub socket: Option<&'a str>,
}

/// pane から解いた席。
enum Seat {
    /// 登録 row の在る席の target。
    Registered(String),
    /// 登録 row の無い target（席でない）。
    Unregistered,
    /// target か event log を読めない。
    Unresolved,
}

/// 判定する。Bash の周で写しの segment か env の行き先が在り、`--pane` が在る周だけ席を解き、写しの断りを先に、env の行き先は
/// command の中の順に最初の断りを返す。
pub fn decide(scene: &Scene) -> DraftsDecision {
    if scene.tool != BASH {
        return DraftsDecision::Pass;
    }
    let command = scene.command.unwrap_or_default();
    let places = copy_places(command, scene.cwd, scene.root);
    let envs = env_places(command, scene.cwd);
    let pane = scene.pane.filter(|found| !found.trim().is_empty() && !(places.is_empty() && envs.is_empty()));
    let Some(pane) = pane else {
        return DraftsDecision::Pass;
    };
    match seat_of(scene, pane) {
        Seat::Unresolved => match (places.first(), envs.first()) {
            (Some(first), _) => deny(Reason::SeatUnresolved, first, &Home::default()),
            (None, Some(first)) => env_deny(Reason::SeatUnresolved, first, Path::new(""), ""),
            (None, None) => DraftsDecision::Pass,
        },
        Seat::Unregistered => DraftsDecision::Pass,
        Seat::Registered(target) => {
            let drafts = crate::seat::drafts_dir(scene.state_dir, &target);
            let home = Home::of(scene.root, std::path::absolute(&drafts).unwrap_or(drafts));
            let copied = places.iter().find_map(|place| judged(place, &home).map(|reason| deny(reason, place, &home)));
            copied.or_else(|| envs_denied(&envs, &home.drafts)).unwrap_or(DraftsDecision::Pass)
        }
    }
}

/// env の行き先 1 つの値（閉じた 3 形）。
#[derive(Debug, Clone, PartialEq, Eq)]
enum Value {
    /// 字のまま解けた絶対 path（字面で畳む）。
    Abs(PathBuf),
    /// 頭が `/` でない字。
    Relative,
    /// 字のまま解けない（`~`・同じ command で字のまま決めていない変数・置換・brace・glob）。
    Unresolved,
}

/// env の行き先 1 つ（環境変数の名か flag の名と値）。
#[derive(Debug, Clone, PartialEq, Eq)]
struct EnvPlace {
    /// 環境変数の名か [`TARGET_FLAG`]。
    via: String,
    /// 値。
    value: Value,
}

/// 同じ command の前で決めた変数（名と字・字のまま解けない値は `None`）。
type Vars = Vec<(String, Option<String>)>;

/// command 行の env の行き先の列（command の中の順）。segment の頭の割り当て・宣言の語と env の後ろの割り当てから名が
/// [`BUILD_VARS`] か [`TMP_VAR`] の値を、頭の語が cargo の segment の [`TARGET_FLAG`] の値を拾う。空の値の割り当ては拾わない。
/// 割り当てだけの segment と宣言の語の segment の割り当ては後ろの segment へ残り、command の前置きの割り当てはその segment だけで効く。
fn env_places(command: &str, cwd: &Path) -> Vec<EnvPlace> {
    let (mut vars, mut dir, mut resolved) = (Vars::new(), cwd.to_path_buf(), true);
    let mut found = Vec::new();
    for words in segments(command) {
        let lead = words.iter().take_while(|word| is_assignment(word)).count();
        let rest = words.get(lead..).unwrap_or_default();
        let (assigned, kept) = declared(rest);
        let mut local = vars.clone();
        for (name, raw) in words.iter().take(lead).chain(assigned).filter_map(|word| word.split_once('=')) {
            let value = resolved_of(raw, &local, resolved.then_some(dir.as_path()));
            if (BUILD_VARS.contains(&name) || name == TMP_VAR) && !raw.is_empty() {
                found.push(EnvPlace { via: name.to_owned(), value: valued(value.as_deref()) });
            }
            local.push((name.to_owned(), value));
        }
        let verb = verb_of(rest);
        if let Some(raw) = verb.filter(|(verb, _)| *verb == CARGO).and_then(|(_, after)| target_flag(after)) {
            let value = resolved_of(raw, &local, resolved.then_some(dir.as_path()));
            found.push(EnvPlace { via: TARGET_FLAG.to_owned(), value: valued(value.as_deref()) });
        }
        if kept || rest.is_empty() {
            vars = local;
        }
        if let Some((_, after)) = verb.filter(|(verb, _)| CD.contains(verb)) {
            (dir, resolved) = moved(&dir, resolved, after);
        }
    }
    found
}

/// 宣言の語か env の segment の、後ろの割り当ての語（flag を飛ばし、最初の割り当てでない語まで）と、割り当てが後ろの segment へ
/// 残るか（宣言の語は残り、env は残らない）。
fn declared(rest: &[String]) -> (Vec<&String>, bool) {
    let Some((head, after)) = rest.split_first() else {
        return (Vec::new(), false);
    };
    let name = head.rsplit('/').next().unwrap_or(head);
    if !DECLARERS.contains(&name) {
        return (Vec::new(), false);
    }
    let words = after.iter().filter(|word| !word.starts_with('-')).take_while(|word| is_assignment(word)).collect();
    (words, name != "env")
}

/// cargo の後ろの語の [`TARGET_FLAG`] の値（次の語か `=` の後ろ・`--` より後は読まない・値の無い flag は空の字）。
fn target_flag(after: &[String]) -> Option<&str> {
    let mut words = after.iter().take_while(|word| *word != "--");
    while let Some(word) = words.next() {
        if word == TARGET_FLAG {
            return Some(words.next().map_or("", String::as_str));
        }
        if let Some(value) = word.strip_prefix(TARGET_FLAG).and_then(|tail| tail.strip_prefix('=')) {
            return Some(value);
        }
    }
    None
}

/// 値の字を字のままの字へ解く。頭が `$NAME` か `${NAME}` なら、NAME が [`PWD`] は segment の作業 dir、ほかは同じ command の前で
/// 決めた変数の字に替え、残りが字のままなら繋ぐ。`~`・ほかの `$`・逆引用符・brace・glob と、解けない作業 dir は `None`。
fn resolved_of(raw: &str, vars: &[(String, Option<String>)], dir: Option<&Path>) -> Option<String> {
    let Some(tail) = raw.strip_prefix('$') else {
        return literal(raw).then(|| raw.to_owned());
    };
    let (name, rest) = match tail.strip_prefix('{') {
        Some(inner) => inner.split_once('}')?,
        None => tail.split_at(tail.find(|found: char| !(found.is_ascii_alphanumeric() || found == '_')).unwrap_or(tail.len())),
    };
    let base = if name == PWD {
        dir?.display().to_string()
    } else {
        vars.iter().rev().find(|(found, _)| found == name)?.1.clone()?
    };
    (rest.is_empty() || literal(rest)).then(|| format!("{base}{rest}"))
}

/// 解いた字を値にする（解けない字は `Unresolved`・頭が `/` でない字は `Relative`・絶対 path は字面で畳む）。
fn valued(found: Option<&str>) -> Value {
    match found.map(Path::new) {
        None => Value::Unresolved,
        Some(path) if path.is_absolute() => Value::Abs(normalized(path)),
        Some(_) => Value::Relative,
    }
}

/// mount の表で、path の段ごとの前置きになる最も長い mount の点（同じ点は後の行）の型が tmpfs か ramfs なら、その点。
fn tmpfs_of(path: &Path, mounts: &str) -> Option<PathBuf> {
    let points = mounts.lines().filter_map(|line| {
        let mut fields = line.split_whitespace().skip(1);
        let point = ESCAPES.iter().fold(fields.next()?.to_owned(), |text, (from, to)| text.replace(from, to));
        Some((PathBuf::from(point), fields.next()?))
    });
    let (point, kind) = points.filter(|(point, _)| path.starts_with(point)).max_by_key(|(point, _)| point.components().count())?;
    TMPFS_TYPES.contains(&kind).then_some(point)
}

/// path が起草の置き場の直下の子より下で、子より下のどれかの段の名が掃除の名の列に在るか（掃除が消す形）。
fn swept_place(path: &Path, drafts: &Path) -> bool {
    let Ok(rest) = path.strip_prefix(drafts) else {
        return false;
    };
    rest.components().skip(1).any(|part| NAMES.iter().any(|name| part.as_os_str() == OsStr::new(name)))
}

/// env の行き先 1 つの判定（通すなら `None`）。`mounts` は mount の表の字。
fn judged_env(place: &EnvPlace, drafts: &Path, mounts: &str) -> Option<Reason> {
    let path = match &place.value {
        Value::Unresolved => return Some(Reason::Unresolved),
        Value::Relative => return Some(Reason::Relative),
        Value::Abs(path) => real(path),
    };
    if tmpfs_of(&path, mounts).is_some() {
        return Some(Reason::Tmpfs);
    }
    (place.via != TMP_VAR && !swept_place(&path, &real(drafts))).then_some(Reason::Outside)
}

/// env の行き先の列のうち command の中の順に最初の断り（行き先が無い周は mount の表を読まない）。
fn envs_denied(envs: &[EnvPlace], drafts: &Path) -> Option<DraftsDecision> {
    if envs.is_empty() {
        return None;
    }
    let mounts = std::fs::read_to_string(MOUNTS).unwrap_or_else(|_| FALLBACK_MOUNTS.to_owned());
    envs.iter().find_map(|place| judged_env(place, drafts, &mounts).map(|reason| env_deny(reason, place, drafts, &mounts)))
}

/// env の行き先の断りにする。
fn env_deny(reason: Reason, place: &EnvPlace, drafts: &Path, mounts: &str) -> DraftsDecision {
    DraftsDecision::Deny { reason, line: env_line(reason, place, drafts, mounts) }
}

/// env の行き先の断りの 1 行（`to=` は解けた絶対 path・ほかは `-`）。経路は理由と行き先の種類で 4 つ。
fn env_line(reason: Reason, place: &EnvPlace, drafts: &Path, mounts: &str) -> String {
    let to = match &place.value {
        Value::Abs(path) => Some(path),
        Value::Relative | Value::Unresolved => None,
    };
    let head = format!("{NAME}: deny drafts reason={} via={} to={}", reason.as_str(), place.via, to.map_or_else(|| "-".to_owned(), |path| path.display().to_string()));
    let route = match (reason, place.via == TMP_VAR) {
        (Reason::SeatUnresolved, _) => "席（pane の tmux の target か置き場の event log）を解けないので、組みの置き場と一時の置き場の行き先を確かめられない（fail-closed・\
             vessel-hook.md §25）。tmux と event log を直してから撃つ"
            .to_owned(),
        (Reason::Tmpfs, _) => {
            let point = to.and_then(|path| tmpfs_of(&real(path), mounts)).unwrap_or_default();
            format!(
                "RAM の上の置き場（tmpfs の {}）に組みと一時の file を置かない（tmpfs の頁は書いた席の memory に数えられ、上限を越えると席ごと\
                 落ちる・vessel-hook.md §25）— disk の上の置き場に置く",
                point.display()
            )
        }
        (_, true) => "TMPDIR は字のままの絶対 path で書く（~ と、同じ command で字のまま決めていない変数は門が解かない・vessel-hook.md §25）".to_owned(),
        (_, false) => format!(
            "組みの置き場は起草の置き場 {0} の直下の写しの中の掃除の名の dir（例 {0}/<名>/target）を字のままの絶対 path で書く（器の掃除が\
             消すのはその形だけで、ほかの置き場は誰も消さず disk を満たす・ADR-0096・ADR-0101・vessel-hook.md §25）",
            drafts.display()
        ),
    };
    format!("{head} — {route}")
}

/// pane → target → 登録 row の順に席を解く（event log は replay する・読めない周は解けない側）。
fn seat_of(scene: &Scene, pane: &str) -> Seat {
    let socket = scene.socket.filter(|found| !found.trim().is_empty());
    let Some(target) = crate::seat::target_of_pane(socket, pane) else {
        return Seat::Unresolved;
    };
    let Ok(events) = store::read_all(scene.state_dir) else {
        return Seat::Unresolved;
    };
    match crate::seat::role::registration_of_target(&replay(&events), &target) {
        Some(_) => Seat::Registered(target),
        None => Seat::Unregistered,
    }
}

/// 通す置き場の 2 つ（席の起草の置き場と anchor の `.worktrees/`）。席を解けない周は既定値（置き場を持たない）。
#[derive(Default)]
struct Home {
    /// 席の起草の置き場（絶対 path）。
    drafts: PathBuf,
    /// anchor の `.worktrees/`（`worktrees_dir` の親・字を 2 面に持たない）。
    worktrees: PathBuf,
    /// `.worktrees/` の直下で通さない名（`worktrees_dir` の名＝便の木の置き場）。
    reserved: OsString,
}

impl Home {
    /// anchor の root と席の起草の置き場から置き場の 2 つを解く。
    fn of(root: &Path, drafts: PathBuf) -> Self {
        let tree = worktrees_dir(root);
        Self {
            drafts,
            worktrees: tree.parent().map_or_else(|| root.to_path_buf(), Path::to_path_buf),
            reserved: tree.file_name().map(OsString::from).unwrap_or_default(),
        }
    }
}

/// 写し 1 つの判定（通すなら `None`）。
fn judged(place: &Place, home: &Home) -> Option<Reason> {
    match &place.to {
        Dest::Unresolved => Some(Reason::Unresolved),
        Dest::Path(path) => (!child_of_a_place(path, home)).then_some(Reason::Outside),
        Dest::Under(dir) => (!is_a_place(dir, home)).then_some(Reason::Outside),
    }
}

/// path の親がどちらかの置き場と同じか（`.worktrees/` の直下は名が便の木の置き場の名と違う周だけ）。
fn child_of_a_place(path: &Path, home: &Home) -> bool {
    let Some(parent) = path.parent() else {
        return false;
    };
    same(parent, &home.drafts) || (same(parent, &home.worktrees) && path.file_name() != Some(home.reserved.as_os_str()))
}

/// dir がどちらかの置き場と同じか。
fn is_a_place(dir: &Path, home: &Home) -> bool {
    same(dir, &home.drafts) || same(dir, &home.worktrees)
}

/// 2 つの path が同じか（両方を、在る最も深い祖先を実体 path に解いて残りを足した形で比べる）。
fn same(left: &Path, right: &Path) -> bool {
    real(left) == real(right)
}

/// 在る最も深い祖先を実体 path に解き、残りを足した path（解けなければ字面のまま）。
fn real(path: &Path) -> PathBuf {
    let Some(existing) = path.ancestors().find(|dir| dir.exists()) else {
        return path.to_path_buf();
    };
    match (existing.canonicalize(), path.strip_prefix(existing)) {
        (Ok(base), Ok(rest)) => base.join(rest),
        _ => path.to_path_buf(),
    }
}

/// 断りにする。
fn deny(reason: Reason, place: &Place, home: &Home) -> DraftsDecision {
    DraftsDecision::Deny { reason, line: deny_line(reason, place, home) }
}

/// 行き先の字（断りの `to=`・作業 dir の直下の形は作業 dir・解けない形は `-`）。
fn shown(to: &Dest) -> String {
    match to {
        Dest::Path(path) | Dest::Under(path) => path.display().to_string(),
        Dest::Unresolved => "-".to_owned(),
    }
}

/// 断りの 1 行。席を解けない周は置き場を持たない。
fn deny_line(reason: Reason, place: &Place, home: &Home) -> String {
    let head = format!("{NAME}: deny drafts reason={} verb={} to={}", reason.as_str(), place.verb, shown(&place.to));
    if reason == Reason::SeatUnresolved {
        return format!(
            "{head} — 席（pane の tmux の target か置き場の event log）を解けないので、写しの行き先を確かめられない（fail-closed・\
             ADR-0096・vessel-hook.md §25）。tmux と event log を直してから撃つ"
        );
    }
    let (drafts, worktrees) = (home.drafts.display(), home.worktrees.display());
    format!(
        "{head} — 席の起草の写しは起草の置き場 {drafts} の直下に置く（器の掃除が起草の木と読むのは直下の子だけ・ADR-0096・\
         vessel-hook.md §25・行き先を字面で解けない形も断る）。docs PR の写しは anchor の .worktrees/（{worktrees}）の直下でもよい\
         （.worktrees/{} は便の木の置き場）。例: git worktree add {drafts}/<名>",
        home.reserved.to_string_lossy()
    )
}

#[cfg(test)]
mod tests {
    use super::{env_line, env_places, judged_env, tmpfs_of, EnvPlace, Reason, Value, FALLBACK_MOUNTS};
    use std::path::{Path, PathBuf};

    /// 見本の起草の置き場（在らない字の path）。
    const DRAFTS: &str = "/vdrenv-none/seat/drafts";

    /// 見本の mount の表（/ は ext4・/dev/shm と /run は tmpfs・/run/data は ext4・空白を持つ点は tmpfs）。
    const MOUNTS: &str = "/dev/root / ext4 rw 0 0\ntmpfs /dev/shm tmpfs rw 0 0\ntmpfs /run tmpfs rw 0 0\n/dev/sdb /run/data ext4 rw 0 0\ntmpfs /my\\040tmp tmpfs rw 0 0\n";

    /// 作業 dir /w で読んだ行き先の (名, 値) の列。
    fn read(command: &str) -> Vec<(String, Value)> {
        env_places(command, Path::new("/w")).into_iter().map(|place| (place.via, place.value)).collect()
    }

    /// 絶対 path の値。
    fn abs(path: &str) -> Value {
        Value::Abs(PathBuf::from(path))
    }

    /// 名と値の行き先。
    fn place(via: &str, value: Value) -> EnvPlace {
        EnvPlace { via: via.to_owned(), value }
    }

    /// 見本の置き場と表で判じた理由（通すなら `None`）。
    fn judged(via: &str, path: &str) -> Option<Reason> {
        judged_env(&place(via, abs(path)), Path::new(DRAFTS), MOUNTS)
    }

    /// 拾う形（頭の割り当て・export の後ろ・env の後ろ・cargo の --target-dir の 2 形・変数と PWD の解き・相対・解けない）と、拾う見本から
    /// 1 句だけ外した拾わない形。
    #[test]
    fn vdrenv_reads_the_target_and_tmpdir_assignments() {
        for (command, via, value) in [
            ("CARGO_TARGET_DIR=/a/target cargo build", "CARGO_TARGET_DIR", abs("/a/target")),
            ("export TMPDIR=/b; cargo test", "TMPDIR", abs("/b")),
            ("declare -x TMPDIR=/b; cargo test", "TMPDIR", abs("/b")),
            ("typeset TMPDIR=/b", "TMPDIR", abs("/b")),
            ("readonly TMPDIR=/b", "TMPDIR", abs("/b")),
            ("local TMPDIR=/b", "TMPDIR", abs("/b")),
            ("export T=/f; CARGO_TARGET_DIR=$T/target cargo b", "CARGO_TARGET_DIR", abs("/f/target")),
            ("env T=/f x; CARGO_TARGET_DIR=$T/target cargo b", "CARGO_TARGET_DIR", Value::Unresolved),
            ("env CARGO_BUILD_TARGET_DIR=/c/target cargo x", "CARGO_BUILD_TARGET_DIR", abs("/c/target")),
            ("cargo build --target-dir /d/target", "--target-dir", abs("/d/target")),
            ("cargo build --target-dir=/d/target", "--target-dir", abs("/d/target")),
            ("PYTHONPYCACHEPREFIX=/e/__pycache__ python3 x.py", "PYTHONPYCACHEPREFIX", abs("/e/__pycache__")),
            ("T=/f; CARGO_TARGET_DIR=$T/target cargo b", "CARGO_TARGET_DIR", abs("/f/target")),
            ("T=/f CARGO_TARGET_DIR=${T}/target cargo b", "CARGO_TARGET_DIR", abs("/f/target")),
            ("cd /g && CARGO_TARGET_DIR=$PWD/target cargo b", "CARGO_TARGET_DIR", abs("/g/target")),
            ("CARGO_TARGET_DIR=$PWD/../t/target cargo b", "CARGO_TARGET_DIR", abs("/t/target")),
            ("TMPDIR=rel cmd", "TMPDIR", Value::Relative),
            ("CARGO_TARGET_DIR=$HOME/x cargo b", "CARGO_TARGET_DIR", Value::Unresolved),
            (concat!("CARGO_TARGET_DIR=~", "/x cargo b"), "CARGO_TARGET_DIR", Value::Unresolved),
            ("T=/f cargo b; CARGO_TARGET_DIR=$T/target cargo b", "CARGO_TARGET_DIR", Value::Unresolved),
            ("cd $D && CARGO_TARGET_DIR=$PWD/target cargo b", "CARGO_TARGET_DIR", Value::Unresolved),
        ] {
            assert_eq!(read(command), [(via.to_owned(), value)], "{command}");
        }
        for command in [
            "echo CARGO_TARGET_DIR=/a/target",
            "git build --target-dir /d/target",
            "FOO_TMPDIR=/b cmd",
            "CARGO_TARGET_DIR= cargo build",
            "cargo test -- --target-dir /d/target",
        ] {
            assert_eq!(read(command), [], "{command}");
        }
    }

    /// 組みの置き場は起草の置き場の子より下の掃除の名の dir の中だけを通し、ほかは outside・相対は relative・解けない値は unresolved・
    /// tmpfs の上は tmpfs で断る。断りの 1 行は名と行き先と置き場の例を持つ。
    #[test]
    fn vdrenv_target_dir_must_be_a_target_under_a_drafts_child() {
        for path in ["w1/target", "w1/target/nested/scribe2", "w1/sub/target", "w1/__pycache__"] {
            assert_eq!(judged("CARGO_TARGET_DIR", &format!("{DRAFTS}/{path}")), None, "{path}");
        }
        assert_eq!(judged("CARGO_TARGET_DIR", &format!("{DRAFTS}/target")), Some(Reason::Outside), "子そのものが target");
        assert_eq!(judged("CARGO_TARGET_DIR", &format!("{DRAFTS}/w1/build")), Some(Reason::Outside), "掃除の名の段が無い");
        assert_eq!(judged("--target-dir", "/other/w1/target"), Some(Reason::Outside), "置き場の外");
        assert_eq!(judged("CARGO_TARGET_DIR", "/dev/shm/w1/target"), Some(Reason::Tmpfs), "tmpfs の上");
        let drafts = Path::new(DRAFTS);
        assert_eq!(judged_env(&place("CARGO_TARGET_DIR", Value::Relative), drafts, MOUNTS), Some(Reason::Relative));
        assert_eq!(judged_env(&place("CARGO_TARGET_DIR", Value::Unresolved), drafts, MOUNTS), Some(Reason::Unresolved));
        let line = env_line(Reason::Outside, &place("CARGO_TARGET_DIR", abs("/other/w1/target")), drafts, MOUNTS);
        for needle in ["deny drafts reason=outside via=CARGO_TARGET_DIR to=/other/w1/target", &format!("例 {DRAFTS}/<名>/target"), "ADR-0096"] {
            assert!(line.contains(needle), "{needle}: {line}");
        }
    }

    /// tmpfs の判じは表の点の段ごとの前置きで最も長い点の型（escape の空白を解く・表を読めない周は /dev/shm だけ）。TMPDIR は tmpfs の上だけを
    /// 断り、disk の上と起草の置き場の外を通す。
    #[test]
    fn vdrenv_tmpdir_on_tmpfs_is_refused_by_the_mount_table() {
        let point = |path: &str| tmpfs_of(Path::new(path), MOUNTS);
        assert_eq!(point("/dev/shm/x"), Some(PathBuf::from("/dev/shm")));
        assert_eq!(point("/run/x"), Some(PathBuf::from("/run")));
        assert_eq!(point("/my tmp/x"), Some(PathBuf::from("/my tmp")), "8 進の escape の空白");
        assert_eq!(point("/tmp/x"), None);
        assert_eq!(point("/run/data/x"), None, "長い ext4 の点が勝つ");
        assert_eq!(point("/dev/shmx/y"), None, "段の前置きで、字の前置きでない");
        assert_eq!(tmpfs_of(Path::new("/dev/shm/x"), FALLBACK_MOUNTS), Some(PathBuf::from("/dev/shm")));
        assert_eq!(tmpfs_of(Path::new("/run/x"), FALLBACK_MOUNTS), None, "表を読めない周は /dev/shm だけ");
        assert_eq!(judged("TMPDIR", "/tmp/w1"), None, "disk の上は置き場の外でも通す");
        assert_eq!(judged("TMPDIR", &format!("{DRAFTS}/w1/tmp")), None);
        assert_eq!(judged("TMPDIR", "/dev/shm/w1"), Some(Reason::Tmpfs));
        let line = env_line(Reason::Tmpfs, &place("TMPDIR", abs("/dev/shm/w1")), Path::new(DRAFTS), MOUNTS);
        assert!(line.contains("reason=tmpfs via=TMPDIR to=/dev/shm/w1") && line.contains("tmpfs の /dev/shm"), "{line}");
    }
}
