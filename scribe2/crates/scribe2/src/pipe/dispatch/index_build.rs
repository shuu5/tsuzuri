//! code の索引の組み立ての口（設計 docs/design/reverse-index.md §4 形 1・2・形 6〜9・契約表の行 a2・FR107・NFR6）。
//!
//! 宣言の 2 key（`index-scip`・`index-roles`）の command を、commit を detach した木（床の検査の [`Worktree`]）の上で
//! 封じ込めの箱と受付札 1 枚の中で撃ち、行 a1（[`crate::pipe::index`]）の読み手と結びで平らな表にして、state dir の
//! `pipe/index` の下に鍵ごとに置く（鍵 = code の木の鍵 + 宣言の字・撃ち中の印は床の検査の印 `<鍵>.lock`）。撃つ側は
//! 床の検査の `run`・`resolve`・印を共用する（2 本目の撃ち方を作らない）。状態の読み（[`status`]）は撃たず待たない。
//! 待つのは組み立て（[`assemble`]）だけで、待ちは `fleet::pid_gone` の 1 本（唯一の待機実装）。

use super::floor::{resolve, run, Lock, Ran, Worktree};
use crate::cli_outcome::{Outcome, RC_BROKEN, RC_OK, RC_REFUSED};
use crate::fleet::pid_gone;
use crate::fleet::store::{lock_owner, owner_pid, started_ms, LockPolicy, Owner};
use crate::invocation::Invocation;
use crate::pipe::admission;
use crate::pipe::cli::{broken, flag, refused, repo_of, state_dir_of};
use crate::pipe::confine::{release_scope, unit_name, wrap_command, Caps, Limit, Wrap};
use crate::pipe::declaration::{index_at, DeclError, IndexLines};
use crate::pipe::gate::Limits;
use crate::pipe::git_line;
use crate::pipe::index::flat::{read_table, render, Row};
use crate::pipe::index::scip::{project_root, read_scip};
use crate::pipe::index::{join, key_digest, RoleLines, RoleRead};
use crate::pipe::row_review::tree_key;
use crate::rules::int_row;
use crate::rules::manifest::Manifest;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::{BufRead, BufReader, PipeReader};
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

/// 置き場の下の dir（`<state>/pipe/index`）。
const DIR: [&str; 2] = ["pipe", "index"];

/// 置き場の量の上限（MiB）の rules 行の id。
const ROW_CAP: &str = "index.cap_mb";

/// 外の道具 1 本と持ち主の終わりを待つ上限（秒）の rules 行の id。
const ROW_TIMEOUT: &str = "index.timeout_s";

/// 記録の 1 行目。
const SCHEMA_LINE: &str = "schema=1";

/// 既定の ref。
const HEAD: &str = "HEAD";

/// 子の cargo の組みの置き場の env の名。子は木の下の [`TARGET_DIR`] で組み、木と一緒に外す（親の置き場を継ぐと、木の path で組んだ
/// 印が親の置き場に残り、消えた木を後の組みが読みに行く）。
const TARGET_ENV: &str = "CARGO_TARGET_DIR";

/// 木の下の子の組みの置き場の名。
const TARGET_DIR: &str = "target";

/// 組み立ての受付札と封じ込めの箱の job の数（箱の memory は job ごとの上限の倍数。rust-analyzer の SCIP は 1 本で 1 job の上限
/// 〔rules 行 `gate.job_memory_mb`〕を越える workspace が在る）。
const INDEX_JOBS: u64 = 2;

/// 状態の読み（閉じた 6 値・設計 reverse-index.md §4 形 9）。
#[derive(Debug, Clone, PartialEq)]
pub enum Status {
    /// 宣言が 2 key を持たない。
    Undeclared,
    /// 片方だけか穴の欠け（key の名と行番号の不備の列・宣言を読めない周も同じ）。
    Half(Vec<DeclError>),
    /// 表も印も失敗の記録も無い。
    Absent,
    /// 撃ち中の印の持ち主が生きている。
    Building,
    /// 記録が失敗の語を持つ。
    Failed(String),
    /// 表を読める。
    Ready(Vec<Row>),
}

/// 組み立ての結末。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum How {
    /// 撃って表を置いた。
    Built,
    /// 同じ鍵の表が在った（撃たない）。
    Cached,
    /// 失敗の語（`rc`・`timeout`・`unreadable`・`no-rule`・`path`・`confine`・`tree`）。
    Failed(String),
}

/// 組み立ての結果（表の大きさと壁時計つき）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Made {
    /// 索引の鍵（16 桁）。
    pub key: String,
    /// 結末。
    pub how: How,
    /// 表の行数。
    pub rows: usize,
    /// 表に現れる file の数。
    pub files: usize,
    /// 壁時計（秒）。
    pub secs: u64,
}

/// 組み立ての返り（half と undeclared は撃たずにその値）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Assembled {
    /// 宣言が 2 key を持たない。
    Undeclared,
    /// 片方だけか穴の欠け。
    Half(Vec<DeclError>),
    /// 撃った・使い回した・失敗した。
    Made(Made),
}

/// 置き場の dir。
fn dir_of(state_dir: &Path) -> PathBuf {
    DIR.iter().fold(state_dir.to_path_buf(), |path, part| path.join(part))
}

/// 鍵ごとの file（`<鍵>.<拡張子>`）。
fn file(dir: &Path, key: &str, ext: &str) -> PathBuf {
    dir.join(format!("{key}.{ext}"))
}

/// 索引の鍵（code の木の鍵と宣言 2 key の字の digest・木を読めない周は `None`）。
fn key_of(repo: &Path, sha: &str, lines: &IndexLines) -> Option<String> {
    Some(key_digest(&tree_key(repo, sha).ok()?, &lines.scip, &lines.roles))
}

/// 印の持ち主が生きている（読めない持ち主も床の検査の印と同じく取らない側＝作り中に数える）。
fn owner_live(path: &Path) -> bool {
    fs::read_to_string(path).is_ok_and(|body| lock_owner(&body, started_ms) != Owner::Dead)
}

/// 記録の失敗の語（記録が無い・schema が違う・語が無い周は `None`）。
fn failure_of(dir: &Path, key: &str) -> Option<String> {
    let text = fs::read_to_string(file(dir, key, "rec")).ok()?;
    let mut lines = text.lines();
    (lines.next()? == SCHEMA_LINE).then_some(())?;
    lines.find_map(|line| line.strip_prefix("failed=")).map(str::to_owned)
}

/// 鍵の状態（表を読める → 印が生きている → 失敗の記録 → 無い）。表が無い・schema が違う・読めない鍵は「無い」。
fn state_of(dir: &Path, key: &str) -> Status {
    let table = fs::read_to_string(file(dir, key, "tsv")).ok().and_then(|text| read_table(&text).ok().flatten());
    if let Some(rows) = table {
        return Status::Ready(rows);
    }
    if owner_live(&file(dir, key, "lock")) {
        return Status::Building;
    }
    failure_of(dir, key).map_or(Status::Absent, Status::Failed)
}

/// 状態を読む（撃たず待たない・引数は state dir・repo・commit の sha）。
pub fn status(state_dir: &Path, repo: &Path, sha: &str) -> Status {
    match index_at(repo, sha) {
        Ok(None) => Status::Undeclared,
        Err(errors) => Status::Half(errors),
        Ok(Some(lines)) => key_of(repo, sha, &lines).map_or(Status::Absent, |key| state_of(&dir_of(state_dir), &key)),
    }
}

/// 表に現れる file の数。
fn files_of(rows: &[Row]) -> usize {
    rows.iter().map(|row| row.path.as_str()).collect::<BTreeSet<_>>().len()
}

/// file を temp へ書いてから rename する（読み手が書きかけを見ない・置けなければ偽）。
fn put(path: &Path, body: &str) -> bool {
    let temporary = path.with_extension(format!("{}.tmp", std::process::id()));
    let placed = fs::write(&temporary, body).is_ok() && fs::rename(&temporary, path).is_ok();
    if !placed {
        let _ = fs::remove_file(&temporary);
    }
    placed
}

/// 撃てなかった理由（語と stderr の末尾）。
struct Fail {
    /// 失敗の語。
    word: &'static str,
    /// 理由（記録の `stderr=` に 1 行で残る）。
    stderr: String,
}

/// 失敗を組む。
fn fail(word: &'static str, stderr: impl Into<String>) -> Fail {
    Fail { word, stderr: stderr.into() }
}

/// 役の行 1 本の stdout を流れのまま読んだ結果（読めない周は記録の `stderr=` に残る理由）。
type Streamed = Result<RoleRead, String>;

/// 撃ち終えて結んだ表。
struct Shot {
    /// 表の行。
    rows: Vec<Row>,
    /// 9 語の外で捨てた役の一致の数。
    dropped: usize,
    /// 最後の command の stderr の末尾の 1 行。
    stderr: String,
}

/// 1 回の組み立ての材料。
struct Ctx<'a> {
    /// 置き場。
    state_dir: &'a Path,
    /// 対象 repo。
    repo: &'a Path,
    /// 索引を作る commit。
    sha: &'a str,
    /// 索引の鍵。
    key: String,
    /// 宣言の 2 key。
    lines: IndexLines,
    /// 置き場の dir。
    dir: PathBuf,
    /// 規則の値。
    manifest: &'a Manifest,
    /// lock の待ち方。
    policy: LockPolicy,
    /// 始めた時刻。
    started: Instant,
}

/// 索引を組む（引数は state dir・repo・commit の sha・rules・lock の待ち方）。同じ鍵の表が在れば撃たず、撃ち中の印の持ち主が
/// 生きていれば撃たずに終わりを待って読み直し、失敗の記録の在る鍵は撃ち直す。half と undeclared は撃たずにその値を返す。
pub fn assemble(state_dir: &Path, repo: &Path, sha: &str, manifest: &Manifest, policy: LockPolicy) -> Assembled {
    let lines = match index_at(repo, sha) {
        Ok(Some(found)) => found,
        Ok(None) => return Assembled::Undeclared,
        Err(errors) => return Assembled::Half(errors),
    };
    let started = Instant::now();
    let Some(key) = key_of(repo, sha, &lines) else {
        let made = Made { key: "-".to_owned(), how: How::Failed("tree".to_owned()), rows: 0, files: 0, secs: 0 };
        return Assembled::Made(made);
    };
    let ctx = Ctx { state_dir, repo, sha, key, lines, dir: dir_of(state_dir), manifest, policy, started };
    Assembled::Made(ctx.made())
}

impl Ctx<'_> {
    /// 結果を組む（壁時計はここで測る）。
    fn result(&self, how: How, rows: usize, files: usize) -> Made {
        Made { key: self.key.clone(), how, rows, files, secs: self.started.elapsed().as_secs() }
    }

    /// 表を使い回した結果。
    fn cached(&self, rows: &[Row]) -> Made {
        self.result(How::Cached, rows.len(), files_of(rows))
    }

    /// 失敗の結果（`record` の周は失敗の語を記録へ書く・表は置かない）。
    fn failed(&self, found: &Fail, record: bool) -> Made {
        if record {
            let _ = fs::create_dir_all(&self.dir);
            put(&file(&self.dir, &self.key, "rec"), &self.record((0, 0, 0), &found.stderr, Some(found.word)));
        }
        self.result(How::Failed(found.word.to_owned()), 0, 0)
    }

    /// 記録の本文（1 行 1 key の `key=value`・失敗の周は末尾に `failed=<語>`）。
    fn record(&self, shape: (usize, usize, usize), stderr: &str, failed: Option<&str>) -> String {
        let (rows, files, dropped) = shape;
        let decl = key_digest("", &self.lines.scip, &self.lines.roles);
        let (secs, at) = (self.started.elapsed().as_secs(), crate::fleet::cli::now_utc());
        let tail = stderr.replace(['\n', '\r'], " ");
        let mut text = format!(
            "{SCHEMA_LINE}\nkey={}\ncommit={}\ndecl={decl}\nrows={rows}\nfiles={files}\ndropped={dropped}\nsecs={secs}\nat={at}\nstderr={tail}\n",
            self.key, self.sha
        );
        if let Some(word) = failed {
            text.push_str(&format!("failed={word}\n"));
        }
        text
    }

    /// 本体: rules 行 2 本を読み、表が在れば使い回し、印を取れれば撃ち、取れなければ持ち主の終わりを待つ。
    fn made(&self) -> Made {
        let (Ok(cap), Ok(timeout)) = (int_row(self.manifest, ROW_CAP), int_row(self.manifest, ROW_TIMEOUT)) else {
            return self.failed(&fail("no-rule", format!("{ROW_CAP} か {ROW_TIMEOUT} を読めない")), true);
        };
        if let Status::Ready(rows) = state_of(&self.dir, &self.key) {
            return self.cached(&rows);
        }
        match Lock::take(&self.dir, &self.key) {
            Some(_lock) => self.fire(cap, timeout),
            None => self.after_owner(cap, timeout),
        }
    }

    /// 撃ち中の印の持ち主の終わりを待ってから状態を読み直す（越えた周は記録を書かずに `failed:timeout`・
    /// 持ち主が表も失敗も残さず死んだ周は印を外して 1 回だけ撃つ）。
    fn after_owner(&self, cap: u64, timeout: u64) -> Made {
        let owner = fs::read_to_string(file(&self.dir, &self.key, "lock")).ok().and_then(|body| owner_pid(&body));
        if !owner.is_some_and(|pid| pid_gone(pid, Duration::from_secs(timeout)).is_ok()) {
            return self.failed(&fail("timeout", format!("撃ち中の持ち主の終わりを {timeout} 秒で待てない")), false);
        }
        match state_of(&self.dir, &self.key) {
            Status::Ready(rows) => self.cached(&rows),
            Status::Failed(word) => self.result(How::Failed(word), 0, 0),
            _ => match Lock::take(&self.dir, &self.key) {
                Some(_lock) => self.fire(cap, timeout),
                None => self.failed(&fail("timeout", "印を取り直せない"), false),
            },
        }
    }

    /// 印を持った周が撃ち、表と記録を置いて、同じ印の中で量の上限まで消す。
    fn fire(&self, cap: u64, timeout: u64) -> Made {
        if let Status::Ready(rows) = state_of(&self.dir, &self.key) {
            return self.cached(&rows);
        }
        let shot = match self.shoot(timeout) {
            Ok(found) => found,
            Err(found) => return self.failed(&found, true),
        };
        let files = files_of(&shot.rows);
        let record = self.record((shot.rows.len(), files, shot.dropped), &shot.stderr, None);
        if !put(&file(&self.dir, &self.key, "tsv"), &render(&shot.rows)) || !put(&file(&self.dir, &self.key, "rec"), &record) {
            return self.failed(&fail("unreadable", "表を置けない"), false);
        }
        self.shed(cap);
        self.result(How::Built, shot.rows.len(), files)
    }

    /// 受付札 1 枚を取り、木を detach して宣言の行を順に撃ち、結ぶ。SCIP の file と木は結んだ後に外す（役の一致は file に書かない）。
    fn shoot(&self, timeout: u64) -> Result<Shot, Fail> {
        let limits = Limits::of(self.manifest).map_err(|reason| fail("no-rule", reason))?;
        let _grant = admission::admit(self.state_dir, &format!("index-{}", self.key), INDEX_JOBS, &limits.admission(self.policy));
        let tree = Worktree::make(self.repo, &self.dir, self.sha).ok_or_else(|| fail("tree", "commit の木を作れない"))?;
        let scips = (0..self.lines.scip.len()).map(|n| file(&self.dir, &self.key, &format!("{n}.scip"))).collect::<Vec<_>>();
        let shot = self.run_rows(&tree.path, &scips, timeout).and_then(|(stderr, roles)| flatten(&tree.path, (&scips, roles), stderr));
        for path in &scips {
            let _ = fs::remove_file(path);
        }
        shot
    }

    /// 宣言の順（SCIP の行 → 役の行）に撃つ。最後に stderr を持った行の末尾と、役の行ごとに流れのまま読んだ結果を返す（読めない
    /// 結果も、後の行を撃ち終えてから結びで名指す）。
    fn run_rows(&self, tree: &Path, scips: &[PathBuf], timeout: u64) -> Result<(String, Vec<Streamed>), Fail> {
        let scip = self.lines.scip.iter().zip(scips).map(|(row, out)| (row, Some(out.as_path())));
        let roles = self.lines.roles.iter().map(|row| (row, None));
        let (mut stderr, mut streamed) = (String::new(), Vec::new());
        for (n, (row, out)) in scip.chain(roles).enumerate() {
            let (tail, read) = self.run_row(row, (tree, out), n, timeout)?;
            if !tail.is_empty() {
                stderr = tail;
            }
            streamed.extend(read);
        }
        Ok((stderr, streamed))
    }

    /// 行 1 本を撃つ（行を空白で割り穴を語ごとに埋め・頭の語は床の検査の resolve で解く・封じ込めの箱の中・cwd は木・
    /// 子の組みの置き場は木の下）。SCIP の行（`out` が在る）は `{out}` を埋め、役の行は stdout を pipe で受けて撃つ間に別の
    /// thread が読み、rc 0 の周に読んだ結果を返す。
    fn run_row(&self, row: &str, place: (&Path, Option<&Path>), n: usize, timeout: u64) -> Result<(String, Option<Streamed>), Fail> {
        let (tree, out) = place;
        let (tree_text, out_text) = (tree.display().to_string(), out.map(|path| path.display().to_string()));
        let fill = |word: &str| {
            let word = word.replace("{tree}", &tree_text);
            out_text.as_ref().map_or_else(|| word.clone(), |text| word.replace("{out}", text))
        };
        let words: Vec<String> = row.split_whitespace().map(fill).collect();
        let program = words.first().and_then(|head| resolve(head)).ok_or_else(|| fail("path", format!("{row}: 頭の語を PATH に解けない")))?;
        let mut cmd = Invocation::new(program);
        cmd.args(words.iter().skip(1));
        let unit = unit_name(&format!("{}.tree", self.sha), "index", n);
        let wrap = Wrap { unit: &unit, limit: Limit::PerJob(INDEX_JOBS), caps: Caps::embedded(), width: None };
        let (mut cmd, confinement) = wrap_command(cmd, &wrap);
        if !confinement.confined() {
            return Err(fail("confine", format!("{row}: 封じ込めの箱で包めない")));
        }
        let (stdout, reader) = if out.is_some() {
            (Stdio::piped(), None)
        } else {
            let (pipe, writer) = std::io::pipe().map_err(|err| fail("path", format!("{row}: {err}")))?;
            (Stdio::from(writer), Some(stream_roles(pipe, row)))
        };
        cmd.current_dir(tree).env(TARGET_ENV, tree.join(TARGET_DIR)).process_group(0).stdin(Stdio::null()).stdout(stdout).stderr(Stdio::piped());
        let ran = run(&mut cmd, Duration::from_secs(timeout));
        let _ = release_scope(&confinement);
        match ran {
            None => Err(fail("path", format!("{row}: 起こせない"))),
            Some(Ran::Timeout) => Err(fail("timeout", format!("{row}: {timeout} 秒を越えた"))),
            Some(Ran::Done { rc: 0, summary }) => {
                let read = reader.map(|handle| handle.join().unwrap_or_else(|_| Err(format!("{row}: 役の一致の読み手が止まった"))));
                Ok((summary, read))
            }
            Some(Ran::Done { rc, summary }) => Err(fail("rc", format!("{row}: rc {rc} {summary}"))),
        }
    }

    /// 置き場の合計が上限（MiB）を越える周に、撃ち中の鍵・anchor の HEAD の鍵・印の生きている鍵を除いて、記録の at の
    /// 古い順に上限まで消す（消すのは器が作った鍵ごとの file だけ）。
    fn shed(&self, cap_mb: u64) {
        let cap = cap_mb.saturating_mul(1 << 20);
        let mut keyed: BTreeMap<String, (u64, Vec<PathBuf>)> = BTreeMap::new();
        for entry in fs::read_dir(&self.dir).into_iter().flatten().flatten() {
            let name = entry.file_name().to_string_lossy().into_owned();
            let Some((key, _)) = name.split_once('.').filter(|(key, _)| key.len() == 16 && key.bytes().all(|byte| byte.is_ascii_hexdigit())) else {
                continue;
            };
            let slot = keyed.entry(key.to_owned()).or_default();
            slot.0 = slot.0.saturating_add(entry.metadata().map_or(0, |meta| meta.len()));
            slot.1.push(entry.path());
        }
        let mut total: u64 = keyed.values().fold(0, |sum, (size, _)| sum.saturating_add(*size));
        if total <= cap {
            return;
        }
        let head = self.head_key();
        let mut doomed: Vec<(String, String)> = keyed
            .keys()
            .filter(|key| **key != self.key && head.as_ref() != Some(*key) && !owner_live(&file(&self.dir, key, "lock")))
            .map(|key| (self.recorded_at(key), key.clone()))
            .collect();
        doomed.sort();
        for (_, key) in doomed {
            let Some((size, paths)) = keyed.get(&key).filter(|_| total > cap) else {
                continue;
            };
            paths.iter().for_each(|path| drop(fs::remove_file(path)));
            total = total.saturating_sub(*size);
        }
    }

    /// anchor（repo）の HEAD の鍵（宣言を持たない・読めない周は `None`）。
    fn head_key(&self) -> Option<String> {
        let head = git_line(self.repo, &["rev-parse", HEAD])?;
        let lines = index_at(self.repo, &head).ok().flatten()?;
        key_of(self.repo, &head, &lines)
    }

    /// 鍵の記録の at（記録が無い・読めない周は空＝一番古い側）。
    fn recorded_at(&self, key: &str) -> String {
        let text = fs::read_to_string(file(&self.dir, key, "rec")).unwrap_or_default();
        text.lines().find_map(|line| line.strip_prefix("at=")).unwrap_or_default().to_owned()
    }
}

/// 木の下の path の、木からの相対の字（木の外・木そのもの・UTF-8 でない周は `None`・比べは path の要素ごと）。
fn below<'a>(tree: &Path, path: &'a str) -> Option<&'a str> {
    Path::new(path).strip_prefix(tree).ok().and_then(Path::to_str).filter(|rest| !rest.is_empty())
}

/// 役の行の stdout を流れのまま読む thread（`\n` までの塊ごとに 1 行として読み手へ渡す・塊の `lines` は全文の `lines` と同じ
/// 行に割れる）。読めない行の後も終わりまで読み切り（子を pipe の詰まりで止めない）、UTF-8 でない byte が在れば読めない行より
/// 先に名指す（全文を file から読んでいた形と同じ順）。
fn stream_roles(pipe: PipeReader, row: &str) -> JoinHandle<Streamed> {
    let row = row.to_owned();
    std::thread::spawn(move || {
        let (mut reader, mut chunk) = (BufReader::new(pipe), Vec::new());
        let (mut lines, mut first, mut utf8) = (RoleLines::default(), None, true);
        loop {
            chunk.clear();
            match reader.read_until(b'\n', &mut chunk) {
                Ok(0) => break,
                Ok(_) => {}
                Err(err) => return Err(format!("{row}: {err}")),
            }
            match std::str::from_utf8(&chunk) {
                Ok(text) if utf8 && first.is_none() => first = lines.line(text.lines().next().unwrap_or_default()).err(),
                Ok(_) => {}
                Err(_) => utf8 = false,
            }
        }
        if !utf8 {
            return Err(format!("{row}: stream did not contain valid UTF-8"));
        }
        first.map_or_else(|| Ok(lines.finish()), |err| Err(err.to_string()))
    })
}

/// 撃ち終えた SCIP の file を読み、役の行の流れから読んだ一致と木の本文で結んで表の行にする（読めない物は `unreadable`・SCIP の
/// file を役の一致より先に名指す）。表の path は木からの相対に揃える: SCIP の project_root（`file://` の URI）が木の下の dir
/// なら、その相対を document の path の頭に足し、役の一致の file が木の下の絶対 path なら木の頭を外す（木の外の
/// project_root と file は今のまま）。
fn flatten(tree: &Path, outs: (&[PathBuf], Vec<Streamed>), stderr: String) -> Result<Shot, Fail> {
    let unreadable = |reason: String| fail("unreadable", reason);
    let mut docs = Vec::new();
    for path in outs.0 {
        let bytes = fs::read(path).map_err(|err| unreadable(format!("{}: {err}", path.display())))?;
        let mut read = read_scip(&bytes).map_err(|err| unreadable(err.to_string()))?;
        let root = project_root(&bytes);
        if let Some(head) = root.as_deref().and_then(|uri| uri.strip_prefix("file://")).and_then(|at| below(tree, at)) {
            read.iter_mut().for_each(|doc| doc.path = format!("{head}/{}", doc.path));
        }
        docs.extend(read);
    }
    let (mut matches, mut dropped) = (Vec::new(), 0_usize);
    for read in outs.1 {
        let mut read = read.map_err(unreadable)?;
        for found in &mut read.matches {
            if let Some(rest) = below(tree, &found.file) {
                found.file = rest.to_owned();
            }
        }
        matches.extend(read.matches);
        dropped = dropped.saturating_add(read.dropped);
    }
    let mut bodies = BTreeMap::new();
    for doc in &docs {
        if !bodies.contains_key(&doc.path) {
            let body = fs::read_to_string(tree.join(&doc.path)).map_err(|err| unreadable(format!("{}: {err}", doc.path)))?;
            bodies.insert(doc.path.clone(), body);
        }
    }
    let rows = join(&docs, &matches, &bodies).map_err(|err| unreadable(err.to_string()))?;
    Ok(Shot { rows, dropped, stderr })
}

/// 結果の 1 行（`[INDEX] key=<16 桁> rows=<n> files=<m> <built|cached|failed:<語>> secs=<s>`）。
fn line_of(made: &Made) -> String {
    let word = match &made.how {
        How::Built => "built".to_owned(),
        How::Cached => "cached".to_owned(),
        How::Failed(why) => format!("failed:{why}"),
    };
    format!("[INDEX] key={} rows={} files={} {word} secs={}", made.key, made.rows, made.files, made.secs)
}

/// `<NAME> pipe index build --repo R --state-dir S [--ref <sha>]`（既定は HEAD）: 前面で最後まで走り、結果の 1 行を返す。
/// rc は built・cached が 0・failed が 1・索引の宣言の不備・repo でない・ref を解けない周が 2・2 key の無い repo は
/// `[INDEX] undeclared` の 1 行で rc 0（撃たない）。
pub(in crate::pipe) fn build(args: &[String], manifest: &Manifest, policy: LockPolicy) -> Outcome {
    if args.get(1).map(String::as_str) != Some("build") {
        return refused("index は build だけを受ける（pipe index build --repo R --state-dir S [--ref SHA]）".to_owned());
    }
    let (state_dir, repo) = match (state_dir_of(args), repo_of(args)) {
        (Ok(state_dir), Ok(repo)) => (state_dir, repo),
        (Err(reason), _) | (_, Err(reason)) => return refused(reason),
    };
    let rev = match flag(args, "--ref") {
        Ok(found) => found.unwrap_or(HEAD),
        Err(reason) => return refused(reason),
    };
    let Some(sha) = git_line(&repo, &["rev-parse", "--verify", "--quiet", &format!("{rev}^{{commit}}")]) else {
        return broken(format!("{rev} を解けない（repo でないか ref が無い）"));
    };
    match assemble(&state_dir, &repo, &sha, manifest, policy) {
        Assembled::Undeclared => Outcome::ok_line("[INDEX] undeclared".to_owned()),
        Assembled::Half(errors) => {
            let mut err = vec!["pipe: 索引の宣言を読めない（index-scip と index-roles は両方を宣言する）".to_owned()];
            err.extend(errors.iter().map(|error| format!("pipe: {error}")));
            Outcome { out: Vec::new(), err, rc: RC_BROKEN }
        }
        Assembled::Made(made) => {
            let rc = if matches!(made.how, How::Failed(_)) { RC_REFUSED } else { RC_OK };
            Outcome { out: vec![line_of(&made)], err: Vec::new(), rc }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{dir_of, file, key_of, status, Status};
    use crate::pipe::declaration::index_at;
    use crate::pipe::git_line;
    use crate::pipe::index::flat::{render, Row};
    use std::path::{Path, PathBuf};

    /// 宣言の本文（必須 3 行の後ろに `extra`）。
    fn declaration(extra: &str) -> String {
        format!("schema = 1\nallowed-commands = [\"git\"]\ncommon-verify = [\"git diff --quiet\"]\n{extra}")
    }

    /// 宣言を 1 commit にした repo と置き場（sha を返す）。
    fn place(name: &str, extra: &str) -> (PathBuf, PathBuf, String) {
        let repo = crate::pipe::fixture::scratch(name);
        let state = crate::pipe::fixture::scratch(&format!("{name}-state"));
        for args in [&["init", "-q", "-b", "main"][..], &["config", "user.name", "t"], &["config", "user.email", "t@example.invalid"], &["config", "commit.gpgsign", "false"]] {
            assert!(crate::pipe::git_ok(&repo, args), "{args:?}");
        }
        assert!(std::fs::write(repo.join(".vessel.toml"), declaration(extra)).is_ok(), "宣言を書けた");
        assert!(crate::pipe::git_ok(&repo, &["add", "-A"]) && crate::pipe::git_ok(&repo, &["commit", "-q", "-m", "c"]), "commit");
        let sha = git_line(&repo, &["rev-parse", "HEAD"]).unwrap_or_default();
        (repo, state, sha)
    }

    /// 2 key の宣言。
    const BOTH: &str = "index-scip = [\"scip {tree} {out}\"]\nindex-roles = [\"roles {tree}\"]\n";

    /// 鍵ごとの file を置き場へ書く（dir は作る）。
    fn write(state: &Path, key: &str, ext: &str, body: &str) {
        assert!(std::fs::create_dir_all(dir_of(state)).is_ok(), "dir を作れた");
        assert!(std::fs::write(file(&dir_of(state), key, ext), body).is_ok(), "{ext} を書けた");
    }

    /// 表の行 1 つ。
    fn row() -> Row {
        Row { path: "src/a.rs".to_owned(), line: 1, col: 1, symbol: "s".to_owned(), definition: true, roles: Vec::new(), test: false, enclosing: String::new(), vis: String::new() }
    }

    /// (k) 宣言が決める 3 値: 宣言なしは undeclared・片方だけと穴の欠けは key の名を持つ half。
    #[test]
    fn pipe_index_status_reads_the_declaration_values() {
        let (bare, bare_state, bare_sha) = place("index-status-bare", "");
        assert_eq!(status(&bare_state, &bare, &bare_sha), Status::Undeclared, "宣言なし");
        let (half, half_state, half_sha) = place("index-status-half", "index-scip = [\"scip {tree} {out}\"]\n");
        assert!(matches!(status(&half_state, &half, &half_sha), Status::Half(errors) if errors.iter().any(|error| error.reason.contains("index-roles"))), "片方だけ");
        let (holes, holes_state, holes_sha) = place("index-status-holes", "index-scip = [\"scip {tree}\"]\nindex-roles = [\"roles {tree}\"]\n");
        assert!(matches!(status(&holes_state, &holes, &holes_sha), Status::Half(errors) if errors.iter().any(|error| error.reason.contains("{out}"))), "穴の欠け");
    }

    /// (k) 置き場が決める 4 値: 表も印も記録も無い・生きた印・失敗の記録・読める表。撃たず待たず、置き場の dir を作らない。
    #[test]
    fn pipe_index_status_reads_the_place_values_without_firing() {
        let (repo, state, sha) = place("index-status-both", BOTH);
        assert_eq!(status(&state, &repo, &sha), Status::Absent, "表も印も記録も無い");
        assert!(!dir_of(&state).exists(), "読むだけで置き場を作らない");
        let lines = index_at(&repo, &sha).ok().flatten().unwrap_or_else(|| panic!("2 key を読める"));
        let key = key_of(&repo, &sha, &lines).unwrap_or_default();
        write(&state, &key, "lock", &format!("{}\n", std::process::id()));
        assert_eq!(status(&state, &repo, &sha), Status::Building, "生きた印");
        write(&state, &key, "lock", "999999999\n");
        assert_eq!(status(&state, &repo, &sha), Status::Absent, "死んだ印は無い側");
        write(&state, &key, "rec", "schema=1\nkey=x\nfailed=rc\n");
        assert_eq!(status(&state, &repo, &sha), Status::Failed("rc".to_owned()), "失敗の記録");
        write(&state, &key, "rec", "schema=2\nfailed=rc\n");
        assert_eq!(status(&state, &repo, &sha), Status::Absent, "schema の違う記録は無い側");
        write(&state, &key, "tsv", &render(&[row()]));
        write(&state, &key, "rec", "schema=1\nfailed=rc\n");
        assert_eq!(status(&state, &repo, &sha), Status::Ready(vec![row()]), "読める表は失敗の記録より先");
        write(&state, &key, "tsv", "schema=9\n");
        assert_eq!(status(&state, &repo, &sha), Status::Failed("rc".to_owned()), "schema の違う表は無い側（記録は読む）");
    }
}
