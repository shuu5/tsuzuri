//! 終える前の門の床の撃ち直し（判断の記録 ADR-63 決定 (4)・要件 FR21）。出す物が差の file を名指す係だけを見る。順:
//! 1. 出力の dir の床の記録 `floor.tsv` を読む。無ければ撃ち直さない。
//! 2. 記録に行の在る行が在れば、床の写し `<置き場>/try-<名>-fl` と空の state dir `<名>/floor-state` と道具を解き、器の表の検査
//!    （scribe2 contracts check --repo <床> --base main）を別の子で撃ちながら、tz check と tz derive --check と行ごとの preflight
//!    （空の state dir・--bead は札の対象）を撃つ。解けない物・撃てない物・上限の内に終わらない物は、まだ分からないの字にする。
//!    待ち終えずに手放す子（上限を越えた子と、ほかの撃ちが Err で返った周の器の表の検査）は process group ごと止めて待つ。
//! 3. 中核の `judge` で行ごとに判じ、出力の dir の門の記録 `floor-gate.tsv` に 1 行ずつ足して、欠けの字を返す（書けなければ標準エラーに書く）。
//!
//! 道具の tz は既定でこの process の binary、scribe2 は既定で PATH の名で、旗 --tz と --scribe2 が替える。

use std::collections::BTreeMap;
use std::ffi::{OsStr, OsString};
use std::fs;
use std::io::{Read, Write};
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::mpsc::{self, Receiver};
use std::thread;
use std::time::{Duration, Instant};

use tsuzuri_core::agent::spec::Spec;
use tsuzuri_core::agent::stop::floor::{
    LOG, RECORD, Rerun, STATE, copy_name, judge, locate, log_line, record, rows, table_rc,
};

use crate::out::emit_err;
use crate::server::events::now;
use crate::server::proc::{KILL, stop_with};
use crate::server::ruling::minute;

/// 撃ち直しの全部の上限（hooks.json の終える前の門の timeout の内）。
pub const LIMIT: Duration = Duration::from_secs(100);

/// 子の終わりを確かめる間隔。
const STEP: Duration = Duration::from_millis(20);

/// 撃ち直しの道具（tz の binary の path・無ければ解けない・scribe2 の program）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tools {
    pub tz: Option<PathBuf>,
    pub scribe2: OsString,
}

/// 引数から旗 --tz と --scribe2 を抜き、残りの引数と道具を返す（`--名 値` か `--名=値`・空の値と 2 度目は断る）。
pub fn tools<'a>(rest: &[&'a str]) -> Result<(Vec<&'a str>, Tools), String> {
    let (mut tz, mut scribe2, mut left) = (None, None, Vec::new());
    let mut it = rest.iter();
    while let Some(arg) = it.next() {
        let (name, inline) = match arg.split_once('=') {
            Some((n, v)) => (n, Some(v)),
            None => (*arg, None),
        };
        let slot = match name {
            "--tz" => &mut tz,
            "--scribe2" => &mut scribe2,
            _ => {
                left.push(*arg);
                continue;
            }
        };
        let value = match inline {
            Some(v) => v,
            None => it.next().ok_or_else(|| format!("{name} の値が無い"))?,
        };
        if value.is_empty() {
            return Err(format!("{name} の値が空"));
        }
        if slot.replace(value).is_some() {
            return Err(format!("{name} が 2 度ある"));
        }
    }
    let tools = Tools {
        tz: tz
            .map(PathBuf::from)
            .or_else(|| std::env::current_exe().ok()),
        scribe2: OsString::from(scribe2.unwrap_or("scribe2")),
    };
    Ok((left, tools))
}

/// 撃ち始めた子と、標準出力と標準エラーを読む thread の受け口（待ち終えた子は `child` が空）。
struct Shot {
    child: Option<Child>,
    readers: Vec<Receiver<Vec<u8>>>,
}

/// 子を撃ち始める（cwd・標準入力は空・新しい process group・`keep` の時だけ標準出力と標準エラーを読む）。
fn fire(program: &OsStr, args: &[OsString], cwd: &Path, keep: bool) -> Result<Shot, String> {
    let pipe = || if keep { Stdio::piped() } else { Stdio::null() };
    let mut child = Command::new(program)
        .args(args)
        .current_dir(cwd)
        .stdin(Stdio::null())
        .stdout(pipe())
        .stderr(pipe())
        .process_group(0)
        .spawn()
        .map_err(|e| {
            let shown = Path::new(program).display();
            format!("道具 {shown} を撃てない（cwd {}・{e}）", cwd.display())
        })?;
    let mut readers = Vec::new();
    let streams: [Option<Box<dyn Read + Send>>; 2] = [
        child
            .stdout
            .take()
            .map(|s| Box::new(s) as Box<dyn Read + Send>),
        child
            .stderr
            .take()
            .map(|s| Box::new(s) as Box<dyn Read + Send>),
    ];
    for mut stream in streams.into_iter().flatten() {
        let (tx, rx) = mpsc::channel();
        thread::spawn(move || {
            let mut bytes = Vec::new();
            let _ = stream.read_to_end(&mut bytes);
            let _ = tx.send(bytes);
        });
        readers.push(rx);
    }
    Ok(Shot {
        child: Some(child),
        readers,
    })
}

impl Shot {
    /// 読み口の終わりと子の終わりを `deadline` まで待ち、rc と読めた字（標準出力の後に標準エラー）を返す
    /// （上限の内に終わらなければ子を手放し、`Drop` が group ごと止める）。
    fn land(mut self, what: &str, deadline: Instant) -> Result<(i32, String), String> {
        let late = || format!("{what} が {} 秒の内に終わらない", LIMIT.as_secs());
        let mut text = String::new();
        for rx in std::mem::take(&mut self.readers) {
            let bytes = rx
                .recv_timeout(deadline.saturating_duration_since(Instant::now()))
                .map_err(|_| late())?;
            text.push_str(&String::from_utf8_lossy(&bytes));
        }
        while let Some(child) = self.child.as_mut() {
            match child.try_wait() {
                Ok(Some(status)) => {
                    self.child = None;
                    return status
                        .code()
                        .map(|rc| (rc, text))
                        .ok_or_else(|| format!("{what} が signal で止まった"));
                }
                Ok(None) if Instant::now() < deadline => thread::sleep(STEP),
                _ => break,
            }
        }
        Err(late())
    }
}

impl Drop for Shot {
    /// 待ち終えずに手放した子（上限を越えた子と、ほかの撃ちが Err で返った周の器の表の検査）を process group ごと止めて待つ。
    fn drop(&mut self) {
        if let Some(child) = self.child.take() {
            stop_with(child, OsStr::new(KILL));
        }
    }
}

/// 子を撃って rc を返す（出力は捨てる）。
fn rc(
    program: &OsStr,
    args: &[&str],
    cwd: &Path,
    what: &str,
    deadline: Instant,
) -> Result<i32, String> {
    let args: Vec<OsString> = args.iter().map(OsString::from).collect();
    fire(program, &args, cwd, false)?
        .land(what, deadline)
        .map(|(rc, _)| rc)
}

/// 床の写しの contracts の直下の .toml から、行の在る file（`contracts/<名>.toml`）と行の頭と verify の行の数。
fn find(floor: &Path, row: &str) -> Option<(String, usize, usize)> {
    let mut files: Vec<PathBuf> = fs::read_dir(floor.join("contracts"))
        .ok()?
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|x| x == "toml"))
        .collect();
    files.sort();
    files.iter().find_map(|p| {
        let (head, lines) = locate(&fs::read_to_string(p).ok()?, row)?;
        Some((
            format!("contracts/{}", p.file_name()?.to_str()?),
            head,
            lines,
        ))
    })
}

/// 床の写しと空の state dir（解けなければ見つからない物の字）。
fn places(dir: &Path, name: &str) -> Result<(PathBuf, PathBuf), String> {
    let floor = dir.join(copy_name(name));
    if !floor.is_dir() {
        return Err(format!("床の写し {} が dir として無い", floor.display()));
    }
    let state = dir.join(name).join(STATE);
    let empty = fs::create_dir_all(&state)
        .and_then(|()| fs::read_dir(&state))
        .is_ok_and(|mut d| d.next().is_none());
    if !empty {
        return Err(format!(
            "空の state dir {} が無い（作れないか空でない）",
            state.display()
        ));
    }
    Ok((floor, state))
}

/// 1 行の preflight の rc と、行の頭（ノートの file `contracts/<名>.toml` と番号）と verify の行の数。
struct Pre {
    rc: i32,
    note: String,
    head: usize,
    lines: usize,
}

/// 1 行の preflight を撃つ（行の在る契約表の file を床の写しから探す）。
fn preflight(
    tools: &Tools,
    (floor, state): (&Path, &Path),
    row: &str,
    bead: &str,
    deadline: Instant,
) -> Result<Pre, String> {
    let (note, head, lines) = find(floor, row)
        .ok_or_else(|| format!("床の写しの contracts の直下の .toml に行 {row} が無い"))?;
    let args: Vec<OsString> = [
        OsStr::new("pipe"),
        OsStr::new("preflight"),
        OsStr::new("--state-dir"),
        state.as_os_str(),
        OsStr::new("--repo"),
        floor.as_os_str(),
        OsStr::new("--design"),
        OsStr::new(&format!("{note}#{row}")),
        OsStr::new("--bead"),
        OsStr::new(bead),
    ]
    .iter()
    .map(OsString::from)
    .collect();
    let what = format!("preflight（行 {row}）");
    let (rc, _) = fire(&tools.scribe2, &args, floor, false)?.land(&what, deadline)?;
    Ok(Pre {
        rc,
        note,
        head,
        lines,
    })
}

/// 記録に行の在る行 `ids` の撃ち直し（器の表の検査を別の子で撃ちながらほかの 3 本を撃つ）。
fn rerun_all(
    dir: &Path,
    spec: &Spec,
    ids: &[String],
    tools: &Tools,
) -> Result<BTreeMap<String, Result<Rerun, String>>, String> {
    let (floor, state) = places(dir, &spec.name)?;
    let tz = tools
        .tz
        .as_deref()
        .ok_or("道具 tz（この process の binary の path）が見つからない")?;
    let deadline = Instant::now() + LIMIT;
    let table_args: Vec<OsString> = ["contracts", "check", "--repo"]
        .iter()
        .map(OsString::from)
        .chain([
            floor.clone().into_os_string(),
            "--base".into(),
            "main".into(),
        ])
        .collect();
    // ほかの撃ちが Err で返った周は `table` を手放すので、`Drop` が器の表の検査を group ごと止めて待つ。
    let table = fire(&tools.scribe2, &table_args, &floor, true)?;
    let check = rc(tz.as_os_str(), &["check"], &floor, "tz check", deadline)?;
    let derive_args = ["derive", "--dir", ".", "--out", "../contracts", "--check"];
    let derive = rc(
        tz.as_os_str(),
        &derive_args,
        &floor.join("design-intent"),
        "tz derive --check",
        deadline,
    )?;
    let rows: Vec<(String, Result<Pre, String>)> = ids
        .iter()
        .map(|id| {
            (
                id.clone(),
                preflight(tools, (&floor, &state), id, &spec.target, deadline),
            )
        })
        .collect();
    let (table_rc_all, text) = table.land("器の表の検査", deadline)?;
    Ok(rows
        .into_iter()
        .map(|(id, pre)| {
            let re = pre.map(|p| Rerun {
                rcs: [
                    p.rc,
                    check,
                    derive,
                    table_rc(&text, table_rc_all, &p.note, p.head),
                ],
                verify: p.lines.saturating_sub(1),
            });
            (id, re)
        })
        .collect())
}

/// 門の記録に行を足す（出力の dir が無ければ作る）。
fn append(out: &Path, lines: &str) -> std::io::Result<()> {
    fs::create_dir_all(out)?;
    fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(out.join(LOG))?
        .write_all(lines.as_bytes())
}

/// 差の file を名指す係の床の欠けの字の列（差の file を名指さない係は空で、門の記録を書かない）。
/// `dir` は起草の置き場・`out` は係の出力の dir・`again` は止めた後の 2 度目の終わりか。
pub fn holes(dir: &Path, spec: &Spec, out: &Path, again: bool, tools: &Tools) -> Vec<String> {
    let ids = rows(&spec.outputs);
    if ids.is_empty() {
        return Vec::new();
    }
    let text = fs::read_to_string(out.join(RECORD)).ok();
    let had: Vec<String> = text
        .as_deref()
        .map(record)
        .map(|r| ids.iter().filter(|i| r.contains_key(*i)).cloned().collect())
        .unwrap_or_default();
    let reruns = if had.is_empty() {
        Ok(BTreeMap::new())
    } else {
        rerun_all(dir, spec, &had, tools)
    };
    let (verdicts, lacks) = judge(&ids, text.as_deref(), |row| match &reruns {
        Ok(map) => map
            .get(row)
            .cloned()
            .unwrap_or_else(|| Err(format!("行 {row} を撃ち直していない"))),
        Err(what) => Err(what.clone()),
    });
    let at = minute(now());
    let lines: String = verdicts.iter().map(|v| log_line(&at, again, v)).collect();
    if let Err(e) = append(out, &lines) {
        emit_err(&format!(
            "tz hook agent-stop: 床の門の記録を書けない（通す）: {e}"
        ));
    }
    lacks
}
