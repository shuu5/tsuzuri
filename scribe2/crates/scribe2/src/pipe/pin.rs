//! 便ごとの器の binary の留め（器の memo t3-hub.74.50.2 の候補 1・行 v-pin）。
//!
//! 便の受付（`pipe run` の受付の後・[`note`]）が、置き場の event log の最後の `InstallRecorded` の path の file を run dir の
//! `bin/<NAME>`（[`path`]）に hard link で置く（中身は写さない＝SSD の書き 0）。置いた留めを `--version` で撃った 1 行目が
//! 自分の 1 行目（[`version_line`]）と字で同じ周だけ残し、違う周・読めない周・同じ file system でない周は bin の dir を残さず
//! 理由の語を返す。照らすのは置いた後の留めなので、照らしの間に PATH の器が入れ替わっても別の版を留めない。器は env も
//! HOME も `/proc` の自分の実行 file も読まない（C2.2）ので、自分の実行 file は `InstallRecorded` の path と `--version` の
//! 照らしで知る。
//!
//! 留めの読みは 3 つ: 同じ便の実装役と審査役の命令は [`head`] が最初の語を留めに替え、便に結ばれる子（argv に `--run` を持つ
//! 起こし直しと着地後の検出）は [`program`] が留めを返し、便に結ばれない子（列が起こす新しい便ほか）は [`launcher`] が留めの
//! 形の呼ばれ方を器の名に戻す（新しい便は PATH の器で起きる）。終わった便の留めは器の掃除が消す（[`drop_bin`]）。
//!
//! 器の入れ替えの門 busy は、器の process の行を [`hold`] で 3 値に分け、留めた便の行だけの周は断らない（行 v-pin-swap）。

use super::run_dir;
use crate::fleet::{store, EventKind, Install};
use crate::invocation::Invocation;
use crate::name::{version_line, NAME};
use std::ffi::OsStr;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

/// 留めを置く run dir の直下の dir の名。
pub const BIN_DIR: &str = "bin";

/// 置き場の event log に `InstallRecorded` が無い。
pub const NO_INSTALL: &str = "no-install";
/// event log・`InstallRecorded` の detail・path の file・留めの `--version` のどれかを読めない。
pub const UNREADABLE: &str = "unreadable";
/// path の file と置き場が同じ file system でない（hard link を置けない・写さない）。
pub const CROSS_DEVICE: &str = "cross-device";
/// 留めの `--version` の 1 行目が自分の 1 行目と違う。
pub const OTHER_VERSION: &str = "other-version";

/// 便の留めの path（`<state_dir>/pipe/<run>/bin/<NAME>`）。
pub fn path(state_dir: &Path, run: &str) -> PathBuf {
    run_dir(state_dir, run).join(BIN_DIR).join(NAME)
}

/// 受付の留めの stderr の 1 行（`pipe: pin=linked` か `pipe: pin=skipped:<理由の語>`）。
pub fn note(state_dir: &Path, run: &str) -> String {
    match link(state_dir, run) {
        Ok(()) => "pipe: pin=linked".to_owned(),
        Err(word) => format!("pipe: pin=skipped:{word}"),
    }
}

/// 留めを置く（置けなかった周は bin の dir を残さず理由の語を返す）。
pub fn link(state_dir: &Path, run: &str) -> Result<(), &'static str> {
    let events = store::read_all(state_dir).map_err(|_| UNREADABLE)?;
    let last = events.iter().rev().find(|event| event.kind == EventKind::InstallRecorded).ok_or(NO_INSTALL)?;
    let install = last.detail.as_deref().and_then(Install::parse).ok_or(UNREADABLE)?;
    let (dir, pin) = (run_dir(state_dir, run).join(BIN_DIR), path(state_dir, run));
    let placed = std::fs::create_dir_all(&dir)
        .and_then(|()| std::fs::hard_link(&install.path, &pin))
        .map_err(|err| link_reason(&err))
        .and_then(|()| match first_line(&pin) {
            Some(line) if line == version_line() => Ok(()),
            Some(_) => Err(OTHER_VERSION),
            None => Err(UNREADABLE),
        });
    if placed.is_err() {
        let _ = std::fs::remove_dir_all(&dir);
    }
    placed
}

/// hard link を置けなかった理由の語（別の file system の断り EXDEV は cross-device・ほかは unreadable）。
fn link_reason(err: &std::io::Error) -> &'static str {
    if err.kind() == ErrorKind::CrossesDevices {
        CROSS_DEVICE
    } else {
        UNREADABLE
    }
}

/// file を `--version` で 1 回撃った stdout の 1 行目（起動できない・rc 0 でない・行が無い周は `None`）。
fn first_line(program: &Path) -> Option<String> {
    let out = Invocation::new(program).arg("--version").output().ok()?;
    out.status.success().then(|| String::from_utf8_lossy(&out.stdout).lines().next().map(str::to_owned))?
}

/// 命令の最初の語が器の名と字で同じで便の留めの file が在る周だけ、最初の語を留めの絶対 path に替える（実装役と審査役の
/// 命令・呼び手は置き場の穴を埋めた後に撃つ）。絶対 path や別の語で始まる命令と、留めの無い便の命令は 1 字も替えない。
pub fn head(state_dir: &Path, run: &str, line: &str) -> String {
    let pin = path(state_dir, run);
    match line.strip_prefix(NAME) {
        Some(rest) if (rest.is_empty() || rest.starts_with(' ')) && pin.is_file() => format!("{}{rest}", pin.display()),
        _ => line.to_owned(),
    }
}

/// 便に結ばれる子の program（留めの file が在れば留めの path・無ければ [`launcher`] を通した自分の呼ばれ方）。
pub fn program(state_dir: &Path, run: &str) -> String {
    let pin = path(state_dir, run);
    if pin.is_file() {
        pin.display().to_string()
    } else {
        super::dispatch::myself()
    }
}

/// 呼ばれ方（`argv[0]`）が留めの形（`…/pipe/<run>/bin/<NAME>`）の周だけ器の名を返し、ほかは呼ばれ方のまま（無ければ器の名）。
/// 留めた便の起こし直しで起きた driver が、列の 1 周で新しい便を古い版で起こさない。
pub fn launcher(argv0: Option<String>) -> String {
    match argv0 {
        Some(found) if !pinned_form(Path::new(&found)) => found,
        _ => NAME.to_owned(),
    }
}

/// path が留めの形か（名が器の名・親の名が bin・親の親の親の名が pipe の dir）。
fn pinned_form(path: &Path) -> bool {
    let name = |at: Option<&Path>| at.and_then(Path::file_name).and_then(std::ffi::OsStr::to_str).map(str::to_owned);
    let bin = path.parent();
    let pipe = bin.and_then(Path::parent).and_then(Path::parent);
    name(Some(path)).as_deref() == Some(NAME) && name(bin).as_deref() == Some(BIN_DIR) && name(pipe).as_deref() == Some(super::DIR)
}

/// argv の `--run` の値（便に結ばれる子の印・持たない argv は `None`）。
pub(super) fn run_of(argv: &[String]) -> Option<&str> {
    argv.windows(2).find(|pair| pair.first().map(String::as_str) == Some("--run")).and_then(|pair| pair.get(1)).map(String::as_str)
}

/// 器の process の行の分け（閉じた 3 値・器の入れ替えの門 busy が数えるのは留めた以外の 2 つ・行 v-pin-swap）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Hold {
    /// 留めた便の process（同じ便の子は留めを撃つので、PATH の器を替えても便の中で版が混ざらない）。
    Pinned,
    /// 便に解けたが留めの無い process（行 v-pin の前に起きた便・留めを置けなかった便・留めの消えた便）。
    Unpinned,
    /// どの便にも解けない process（測れないを 0 本に読まない・C10）。
    Unresolved,
}

/// `pgrep -af` の 1 行（`<pid> <command 行>`）を分ける。器の語（名が器の名で始まり、次の語が pipe・runner・lens の語）が留めの
/// 形ならその file の在否で、器の語の後ろに `--state-dir` と `--run` を持つ行はその便の留めの在否で、`--state-dir` と `--bead` を
/// 持つ `pipe run` の行はその置き場の event log の最後の `RunCreated` の便の driver の札の所有者がこの行の pid の周だけ、その便の
/// 留めの在否で分ける。ほかの行（器の語の無い行・`--state-dir` の無い行・解けない `--bead` の行）は解けない。
pub fn hold(line: &str) -> Hold {
    let words: Vec<&str> = line.split_whitespace().collect();
    let Some((pid, rest)) = words.split_first() else {
        return Hold::Unresolved;
    };
    let named = |word: &&str| Path::new(word).file_name().and_then(OsStr::to_str).is_some_and(|name| name.starts_with(NAME));
    let verb = |word: Option<&&str>| word.is_some_and(|found| matches!(*found, "pipe" | "runner" | "lens"));
    let Some(at) = rest.windows(2).position(|pair| pair.first().is_some_and(named) && verb(pair.get(1))) else {
        return Hold::Unresolved;
    };
    let pinned = |found: bool| if found { Hold::Pinned } else { Hold::Unpinned };
    let word = Path::new(rest.get(at).copied().unwrap_or_default());
    if pinned_form(word) {
        return pinned(word.is_file());
    }
    let args = rest.get(at.saturating_add(1)..).unwrap_or_default();
    let value = |flag: &str| args.windows(2).find(|pair| pair.first() == Some(&flag)).and_then(|pair| pair.get(1)).copied();
    let Some(state) = value("--state-dir").map(Path::new) else {
        return Hold::Unresolved;
    };
    if let Some(run) = value("--run") {
        return pinned(path(state, run).is_file());
    }
    match value("--bead").filter(|_| args.starts_with(&["pipe", "run"])).and_then(|bead| driven_run(state, bead, pid)) {
        Some(run) => pinned(path(state, &run).is_file()),
        None => Hold::Unresolved,
    }
}

/// 置き場の event log の最後の `RunCreated` で `bead` の便を解き、その便の driver の札の所有者が `pid` の周だけ便 id を返す
/// （log を読めない・`RunCreated` が無い・札が無いか読めない・所有者が違う周は `None`）。
fn driven_run(state: &Path, bead: &str, pid: &str) -> Option<String> {
    let events = store::read_all(state).ok()?;
    let run = events.iter().rev().find(|event| event.kind == EventKind::RunCreated && event.bead == bead)?.run.clone();
    let body = std::fs::read_to_string(super::driver_path(state, &run)).ok()?;
    (store::owner_pid(&body)?.to_string() == pid).then_some(run)
}

/// 終わった便の bin の dir を消す（在って消せた周だけ真・器の掃除が live でない便に撃つ）。
pub(super) fn drop_bin(state_dir: &Path, run: &str) -> bool {
    let dir = run_dir(state_dir, run).join(BIN_DIR);
    dir.is_dir() && std::fs::remove_dir_all(&dir).is_ok()
}

#[cfg(test)]
mod tests {
    use super::{
        head, hold, launcher, link, link_reason, path, program, Hold, BIN_DIR, CROSS_DEVICE, NO_INSTALL, OTHER_VERSION, UNREADABLE,
    };
    use crate::fleet::{EventKind, Install};
    use crate::name::{version_line, NAME};
    use crate::pipe::fixture::{append_all, event, exited, scratch, Stub};
    use std::os::unix::fs::{MetadataExt, PermissionsExt};
    use std::path::{Path, PathBuf};

    /// `--version` に `line` を答える偽の器を `dir` に置く（実行権つき）。
    fn fake_vessel(dir: &Path, name: &str, line: &str) -> PathBuf {
        let file = dir.join(name);
        std::fs::write(&file, format!("#!/bin/sh\necho '{line}'\n")).expect("偽の器を書ける");
        std::fs::set_permissions(&file, std::fs::Permissions::from_mode(0o755)).expect("実行権を付ける");
        file
    }

    /// 置き場に `InstallRecorded` を 1 件積む（path は呼び手が選ぶ）。
    fn installed(state: &Path, file: &Path) {
        let detail = Install { sha: "0123456789ab".to_owned(), path: file.display().to_string() }.render();
        append_all(state, &[event("", EventKind::InstallRecorded, None, None, Some(&detail))]);
    }

    /// 留めは同じ版の hard link だけ: 自分と同じ 1 行目を答える file は inode の同じ留めになり、InstallRecorded の無い置き場・
    /// path の file の無い置き場・event log の行が壊れた置き場・1 行目の違う file の置き場は bin の dir を残さず理由の語を返す。
    /// 最後の InstallRecorded が勝ち、hard link の断りは別の file system（EXDEV）だけ語 cross-device・ほか（file が無い）は unreadable。
    #[test]
    fn vpin_link_keeps_only_a_same_version_hard_link() {
        let state = scratch("vpin-link");
        let same = fake_vessel(&state, "same", &version_line());
        let other = fake_vessel(&state, "other", &format!("{NAME} 0.0.0 (000000000000)"));
        assert_eq!(link(&state, "r0"), Err(NO_INSTALL), "InstallRecorded の無い置き場");
        assert!(!state.join("pipe").join("r0").join(BIN_DIR).exists(), "置かない周は bin の dir を残さない");
        installed(&state, &other);
        installed(&state, &same);
        assert_eq!(link(&state, "r1"), Ok(()), "最後の InstallRecorded の file が同じ版");
        let inode = |file: &Path| std::fs::metadata(file).map(|found| (found.dev(), found.ino())).ok();
        assert_eq!(inode(&path(&state, "r1")), inode(&same), "留めは hard link（中身を写さない）");
        installed(&state, &other);
        assert_eq!(link(&state, "r2"), Err(OTHER_VERSION), "1 行目の違う file");
        assert!(!state.join("pipe").join("r2").join(BIN_DIR).exists(), "違う版は bin の dir ごと外す");
        installed(&state, &state.join("absent"));
        assert_eq!(link(&state, "r3"), Err(UNREADABLE), "path の file が無い");
        assert!(!state.join("pipe").join("r3").join(BIN_DIR).exists(), "読めない周も bin の dir を残さない");
        let broken = scratch("vpin-link-broken");
        std::fs::create_dir_all(broken.join("fleet")).expect("log の dir を作れる");
        std::fs::write(broken.join("fleet").join("events.jsonl"), "{\"schema\":1,\"kind\":\"Nonsense\"}\n").expect("壊れた行");
        assert_eq!(link(&broken, "r4"), Err(UNREADABLE), "event log を読めない");
        assert_eq!(link_reason(&std::io::Error::from_raw_os_error(18)), CROSS_DEVICE, "EXDEV");
        assert_eq!(link_reason(&std::io::Error::from(std::io::ErrorKind::NotFound)), UNREADABLE, "file が無い");
        let _ = std::fs::remove_dir_all(&state);
        let _ = std::fs::remove_dir_all(&broken);
    }

    /// head は最初の語が器の名と字で同じ命令だけを留めの path に替え、後ろの字は 1 字も替えない。絶対 path・名を頭に持つ別の語・
    /// 頭の空白・sh の包みの命令と、留めの file の無い便（dir だけ在る便を含む）の命令は替えない。
    #[test]
    fn vpin_head_swaps_only_the_bare_vessel_name() {
        let state = scratch("vpin-head");
        let pin = path(&state, "r1");
        std::fs::create_dir_all(pin.parent().expect("bin の dir")).expect("bin の dir を作れる");
        std::fs::write(&pin, "").expect("留めを置ける");
        let swapped = |line: &str| head(&state, "r1", line);
        assert_eq!(swapped(&format!("{NAME} runner --worktree /w  {{x}}")), format!("{} runner --worktree /w  {{x}}", pin.display()));
        assert_eq!(swapped(NAME), pin.display().to_string(), "名だけの命令");
        for kept in [format!("/opt/{NAME} lens --contract c"), format!("{NAME}x lens"), format!(" {NAME} lens"), format!("sh -c {NAME} lens")] {
            assert_eq!(swapped(&kept), kept, "替えない: {kept}");
        }
        let bare = format!("{NAME} runner --worktree /w");
        assert_eq!(head(&state, "r2", &bare), bare, "留めの無い便");
        std::fs::create_dir_all(path(&state, "r3")).expect("dir だけの留め");
        assert_eq!(head(&state, "r3", &bare), bare, "留めの path が file でない便");
        let _ = std::fs::remove_dir_all(&state);
    }

    /// 起こす子の program: argv に `--run` を持つ子は便の留めが在れば留めを、無ければ自分の呼ばれ方を撃ち、`--run` を持たない子は
    /// 留めの在る置き場でも自分の呼ばれ方を撃つ。呼ばれ方が留めの形（pipe の dir・便・bin・器の名）の周だけ器の名に戻す。
    #[test]
    fn vpin_spawn_self_runs_bound_children_from_the_pin() {
        let state = scratch("vpin-spawn");
        let pin = path(&state, "r1");
        std::fs::create_dir_all(pin.parent().expect("bin の dir")).expect("bin の dir を作れる");
        std::fs::write(&pin, "").expect("留めを置ける");
        let argv = |words: &[&str]| words.iter().map(|word| (*word).to_owned()).collect::<Vec<String>>();
        let stub = Stub::install(|_| exited(0, b""));
        let me = crate::pipe::dispatch::myself();
        for (args, want) in [
            (argv(&["resume", "--run", "r1", "--drive"]), pin.display().to_string()),
            (argv(&["land", "--run", "r2", "--detection-only"]), me.clone()),
            (argv(&["run", "--design", "d", "--bead", "r1"]), me.clone()),
        ] {
            let _ = crate::pipe::dispatch::spawn_self(&state, &args);
            assert_eq!(stub.calls().last().map(|call| call.program.clone()), Some(want.clone()), "{args:?}");
        }
        assert_eq!(program(&state, "r2"), me, "留めの無い便は自分の呼ばれ方");
        let pinned = format!("/s/pipe/r1/{BIN_DIR}/{NAME}");
        assert_eq!(launcher(Some(pinned)), NAME, "留めの形は器の名に戻す");
        let others = [format!("/usr/local/bin/{NAME}"), format!("/s/pip/r1/{BIN_DIR}/{NAME}"), format!("/s/pipe/r1/sbin/{NAME}")];
        for kept in others.into_iter().chain([NAME.to_owned(), format!("/s/pipe/r1/{BIN_DIR}/{NAME}x")]) {
            assert_eq!(launcher(Some(kept.clone())), kept, "留めの形でない呼ばれ方はそのまま");
        }
        assert_eq!(launcher(None), NAME, "呼ばれ方の無い周は器の名");
        let _ = std::fs::remove_dir_all(&state);
    }

    /// 留めの file を置き、`run` に driver の札（`owner` が在れば）と `bead` の `RunCreated` を積む。
    fn pinned_run(state: &Path, run: &str, bead: &str, owner: Option<u32>, pin: bool) {
        let mut created = event(run, EventKind::RunCreated, Some(crate::fleet::Stage::Intake), None, None);
        created.bead = bead.to_owned();
        append_all(state, &[created]);
        let dir = state.join("pipe").join(run);
        std::fs::create_dir_all(dir.join(BIN_DIR)).expect("bin の dir を作れる");
        if let Some(pid) = owner {
            std::fs::write(dir.join("driver"), format!("{pid} 1\n")).expect("札を書ける");
        }
        if pin {
            std::fs::write(path(state, run), "").expect("留めを置ける");
        }
    }

    /// 行の分け: 留めの形の器の語で file の在る行・`--run` の便に留めの在る行・`--bead` の `pipe run` で最後の `RunCreated` の便の札の
    /// 所有者がこの pid でその便に留めの在る行は留めた行（同じ bead の前の便 r0 は読まない）。1 句ずつ外した行（留めの file が無い・
    /// `--run` の便に留めが無い・`--bead` の便に留めが無い・札の所有者が違う pid・`RunCreated` の無い bead・同じ便の行の後ろに壊れた
    /// 行を持つ event log の置き場・`--state-dir` の無い行・`pipe run` でない `--bead` の行・器の語が留めの形でない sh の包みの行・
    /// 空の行）は留めの無い行か解けない行。
    #[test]
    fn vpinswap_hold_reads_three_pinned_forms_and_counts_the_rest() {
        let state = scratch("vpinswap-hold");
        pinned_run(&state, "r1", "b-1", None, true);
        pinned_run(&state, "r0", "b-2", Some(4304), false);
        pinned_run(&state, "r2", "b-2", Some(4304), true);
        pinned_run(&state, "r3", "b-3", Some(4305), false);
        let broken = scratch("vpinswap-hold-broken");
        pinned_run(&broken, "r2", "b-2", Some(4304), true);
        let log = broken.join("fleet").join("events.jsonl");
        let text = std::fs::read_to_string(&log).expect("log を読める");
        std::fs::write(&log, format!("{text}{{\"schema\":1,\"kind\":\"Nonsense\"}}\n")).expect("壊れた行を足せる");
        let (s, b) = (state.display().to_string(), broken.display().to_string());
        let (p1, p9) = (path(&state, "r1").display().to_string(), path(&state, "r9").display().to_string());
        let run = |pid: u32, bead: &str, dir: &str| format!("{pid} /opt/{NAME} pipe run --design d --bead {bead} --state-dir {dir} --drive");
        for (line, want) in [
            (format!("4301 {p1} pipe resume --run r1 --state-dir {s}"), Hold::Pinned),
            (format!("4302 /usr/bin/sh -c {p1} runner --worktree /w"), Hold::Pinned),
            (format!("4303 {NAME} pipe land --run r1 --state-dir {s} --terminal-only"), Hold::Pinned),
            (run(4304, "b-2", &s), Hold::Pinned),
            (format!("4306 {p9} lens --contract c"), Hold::Unpinned),
            (format!("4307 {NAME} pipe land --run r9 --state-dir {s}"), Hold::Unpinned),
            (run(4305, "b-3", &s), Hold::Unpinned),
            (run(4399, "b-2", &s), Hold::Unresolved),
            (run(4304, "b-9", &s), Hold::Unresolved),
            (run(4304, "b-2", &b), Hold::Unresolved),
            (format!("4304 /opt/{NAME} pipe run --design d --bead b-2 --drive"), Hold::Unresolved),
            (format!("4304 /opt/{NAME} pipe dispatch release --bead b-2 --state-dir {s}"), Hold::Unresolved),
            (format!("4308 /usr/bin/sh -c {NAME} runner --worktree /w"), Hold::Unresolved),
            (String::new(), Hold::Unresolved),
        ] {
            assert_eq!(hold(&line), want, "{line}");
        }
        let _ = std::fs::remove_dir_all(&state);
        let _ = std::fs::remove_dir_all(&broken);
    }
}
