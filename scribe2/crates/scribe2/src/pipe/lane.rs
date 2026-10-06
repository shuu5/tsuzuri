//! 便の木の並び（判断の記録 ADR-35 の形 c・宣言の任意 key `build-lanes` を名乗った repo の便だけ）。
//!
//! 並びの木は `<repo>/.worktrees/<NAME>/lane/<n>`（path を固定した worktree）で、木の中の target を便から便へ使い回す。
//! 便の木の path（[`worktree_path`]）は並びの木を指す相対の symlink で、path を読む呼び手は替えずにそのまま並びの木を読む。
//! 印 `lane/<n>.run` は最後にその木を使った便の id で、その便の path の symlink が在る間は便が並びを持つ（ほかの便に渡さない）。
//! 印と symlink は lock `lane/lane.lock` の中でだけ置き・外す（取りの予約・返し・落ちた取りの戻し）ので、並行する便は同じ番号を
//! 取らず、返しはほかの便の印の並びを触らない。木を切る・替える・移す git は、番号を予約した便だけが lock の外で撃つ。
//! 中身の替えは `git checkout -f` と `git clean -ffdx -e /target` で、git は中身の替わる file だけを書くので、cargo は
//! 中身の替わった crate だけを組み直す（中身の替わらない file の mtime は前の組みより古いまま残る）。
//! 並びの木の合計は rules 行 `pipe.lanes_cap_mb` の上限で切る（器の掃除が [`shed`] を撃つ・判断の記録 ADR-35 の決定 (5)）。

use super::land::{move_tree, retired_path, WorktreeCheck};
use super::sweep::measure;
use super::{branch_name, git_ok, worktree_path, worktrees_dir};
use crate::fleet::lifecycle_mark::{hold, Held};
use crate::fleet::store::LockPolicy;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// 並びの木を集める dir の名（`worktrees_dir` の直下）。
const LANE_DIR: &str = "lane";

/// 印の file の拡張子（`<n>.run`）。
const MARK_EXT: &str = "run";

/// 並びの印と symlink を置き・外す lock の名（並びの dir の直下・repo ごとに 1 本）。
const LOCK: &str = "lane.lock";

/// 中身の替えで消さない dir（木の中の組みの置き場）。
const KEEP: &str = "/target";

/// 上限の掃除が退かせる候補の並び（番号・大きさ〔byte〕・印の便・印の新しさ・印の便が並びを持つか）。
struct Idle {
    n: usize,
    bytes: u64,
    run: String,
    since: SystemTime,
    held: bool,
}

/// 並びの木を集める dir。
fn lanes_dir(repo: &Path) -> PathBuf {
    worktrees_dir(repo).join(LANE_DIR)
}

/// 並び `n` の木。
fn lane_path(repo: &Path, n: usize) -> PathBuf {
    lanes_dir(repo).join(n.to_string())
}

/// 並び `n` の印（最後にその木を使った便の id）。
fn mark_path(repo: &Path, n: usize) -> PathBuf {
    lanes_dir(repo).join(format!("{n}.{MARK_EXT}"))
}

/// 並びの lock を取る（並びの dir を作り、死んだ所有者の lock だけを外す・`Drop` で外す）。
fn lock(repo: &Path, policy: LockPolicy) -> Result<Held, String> {
    let dir = lanes_dir(repo);
    hold(&dir, LOCK, policy).ok_or_else(|| format!("{} を取れない", dir.join(LOCK).display()))
}

/// 便が持つ並びの番号（便の path が `lane/<n>` を指す symlink の周だけ・ほかは `None`）。
fn held(repo: &Path, run: &str) -> Option<usize> {
    std::fs::read_link(worktree_path(repo, run)).ok()?.strip_prefix(LANE_DIR).ok()?.to_str()?.parse().ok()
}

/// 便が持つ並びの木（便の path が並びの木を指す symlink の周だけ・ほかは `None`）。
pub(crate) fn held_lane(repo: &Path, run: &str) -> Option<PathBuf> {
    held(repo, run).map(|n| lane_path(repo, n))
}

/// 並び `n` の印の便（印の無い周は `None`）。
fn mark_of(repo: &Path, n: usize) -> Option<String> {
    std::fs::read_to_string(mark_path(repo, n)).ok().map(|run| run.trim().to_owned())
}

/// 並び `n` を便が持っているか（印の便の path が並び `n` を指す・木を切る前の予約も持つと読む）。
fn is_held(repo: &Path, n: usize) -> bool {
    mark_of(repo, n).is_some_and(|run| held(repo, &run) == Some(n))
}

/// 名乗った repo の便の初回の木を並びから用意し、便の path（並びの木を指す symlink）を返す。
///
/// 番号は lock の中で予約し（[`claim`]）、木を切る・替える git は予約を持つこの便だけが lock の外で撃つ。使い回す木は中身を
/// base に替え、替えが落ちた周（替えた木が clean でない周を含む）は今の退役の形（`retired/<前の便>` への move）で退かせてから
/// 新しく切る。落ちた周は予約を戻してから理由を返す（[`undo`]）。
pub(crate) fn take(repo: &Path, run: &str, base: &str, policy: LockPolicy) -> Result<PathBuf, String> {
    let (n, prev) = claim(repo, run, policy)?;
    let lane = lane_path(repo, n);
    let made = match prev.as_deref() {
        Some(_) if switch(&lane, run, base) => Ok(()),
        Some(prev) => match move_tree(repo, &lane, &retired_path(repo, prev)).into_iter().next() {
            Some(failure) => Err(failure),
            None => cut(repo, &lane, run, base),
        },
        None => cut(repo, &lane, run, base),
    };
    if let Err(reason) = made {
        undo(repo, run, n, policy);
        return Err(reason);
    }
    Ok(worktree_path(repo, run))
}

/// lock の中で並びを 1 つ予約する（印を便の id に書き、便の path に並びの木を指す symlink を置く）。持たれていない並びのうち
/// 木の在る最も小さい番号を使い回し、無ければ木が無く、ほかの便の予約も無い最も小さい番号を取る。
/// 返すのは並びの番号と、使い回す周の前の便の id（新しく切る周は `None`）。
fn claim(repo: &Path, run: &str, policy: LockPolicy) -> Result<(usize, Option<String>), String> {
    let _held = lock(repo, policy)?;
    let link = worktree_path(repo, run);
    if std::fs::symlink_metadata(&link).is_ok() {
        return Err(format!("{} は既に在る", link.display()));
    }
    let lanes = numbers(repo)?;
    let reuse = lanes.iter().copied().find(|n| !is_held(repo, *n));
    let n = reuse.unwrap_or_else(|| fresh(repo, &lanes));
    let prev = reuse.map(|n| mark_of(repo, n).unwrap_or_default());
    let mark = mark_path(repo, n);
    std::fs::write(&mark, format!("{run}\n")).map_err(|err| format!("{} を書けない: {err}", mark.display()))?;
    std::os::unix::fs::symlink(Path::new(LANE_DIR).join(n.to_string()), &link)
        .map_err(|err| format!("{} を置けない: {err}", link.display()))?;
    Ok((n, prev))
}

/// 並びの番号の列（並びの dir の直下の数の名の dir・昇順）。
fn numbers(repo: &Path) -> Result<Vec<usize>, String> {
    let dir = lanes_dir(repo);
    let mut lanes: Vec<usize> = std::fs::read_dir(&dir)
        .map_err(|err| format!("{} を読めない: {err}", dir.display()))?
        .flatten()
        .filter(|entry| entry.file_type().is_ok_and(|kind| kind.is_dir()))
        .filter_map(|entry| entry.file_name().to_str().and_then(|name| name.parse().ok()))
        .collect();
    lanes.sort_unstable();
    Ok(lanes)
}

/// 木が無く、ほかの便の予約（印と、印の便の path の symlink）も無い最も小さい番号。
fn fresh(repo: &Path, lanes: &[usize]) -> usize {
    let mut n = 0;
    while lanes.contains(&n) || is_held(repo, n) {
        n += 1;
    }
    n
}

/// 予約した並び `lane` に branch `<NAME>/<run>` の木を base から新しく切る。
fn cut(repo: &Path, lane: &Path, run: &str, base: &str) -> Result<(), String> {
    let (path, branch) = (lane.display().to_string(), branch_name(run));
    if git_ok(repo, &["worktree", "add", "-b", &branch, &path, base]) {
        return Ok(());
    }
    Err(format!("並び {path} を切れない（branch {branch}）"))
}

/// 落ちた取りの予約を戻す（lock の中）: 予約した並び `n` の印と便の path の symlink を外し、番号を次の便が取れるようにする。
fn undo(repo: &Path, run: &str, n: usize, policy: LockPolicy) {
    if let Ok(_held) = lock(repo, policy) {
        let _ = std::fs::remove_file(mark_path(repo, n));
        let _ = std::fs::remove_file(worktree_path(repo, run));
    }
}

/// 並びの木の中身を base に替え（[`renew`]）、clean の周だけ branch `<NAME>/<run>` を付けて真。
fn switch(lane: &Path, run: &str, base: &str) -> bool {
    renew(lane, base) && git_ok(lane, &["checkout", "-q", "-B", &branch_name(run)])
}

/// path を固定した木の中身を `sha` に替える（branch は付けない・git は中身の替わる file だけを書く・未追跡と無視の file は
/// target の外を全部消す）。替えた木が clean の周だけ真（並びの [`switch`] と、着地の後の主実測の温かい木が使う）。
pub(super) fn renew(tree: &Path, sha: &str) -> bool {
    git_ok(tree, &["checkout", "-q", "-f", "--detach", sha])
        && git_ok(tree, &["clean", "-q", "-ffdx", "-e", KEEP])
        && WorktreeCheck::judge(tree).is_clean()
}

/// 便が並びを返す（退役の move の代わり・判断の記録 ADR-35）。並びを持たない便は `None`（呼び手が今の形で move する）。
/// 失敗は stderr 行の列で返す（[`super::land::retire_worktree`] と同じ形）。
pub(crate) fn give_back(repo: &Path, run: &str) -> Option<Vec<String>> {
    let n = held(repo, run)?;
    Some(hand_back(repo, run, n).err().into_iter().collect())
}

/// [`give_back`] の本体。lock の中で、印が自分の便の並びだけを触る: clean な木は並びに残し（印を書き直して新しさを今にする）、
/// clean でない木は今の退役の形のまま `retired/<run>` へ move して印を外し、どちらも便の path の symlink を外す。印がほかの便の
/// 並びは木も印も symlink も触らずに理由を返す。
fn hand_back(repo: &Path, run: &str, n: usize) -> Result<(), String> {
    let lane = lane_path(repo, n);
    let policy = LockPolicy::embedded().map_err(|err| format!("pipe: lock の規則を読めない: {err}"))?;
    let _held = lock(repo, policy)?;
    if mark_of(repo, n).as_deref() != Some(run) {
        return Err(format!("pipe: 並び {} の印は便 {run} の物でない（触らない）", lane.display()));
    }
    let mark = mark_path(repo, n);
    let marked = if WorktreeCheck::judge(&lane).is_clean() {
        std::fs::write(&mark, format!("{run}\n"))
    } else {
        if let Some(failure) = move_tree(repo, &lane, &retired_path(repo, run)).into_iter().next() {
            return Err(failure);
        }
        std::fs::remove_file(&mark)
    };
    marked
        .and_then(|()| std::fs::remove_file(worktree_path(repo, run)))
        .map_err(|err| format!("pipe: 並び {} を返せなかった: {err}", lane.display()))
}

/// 並びの木の大きさの合計が `cap`（byte）を越える間、live な便が持たない並びを印の新しさの古い順（同じ時刻は番号の順）に丸ごと
/// 退かせる（判断の記録 ADR-35 の決定 (5)）。返すのは退かせた並びの数と、退かせられなかった並びの名（`lane/<n>`）の列。
///
/// 大きさは lock の外で測り（便の取りを待たせない）、持ち手は lock の中で読む。大きさを測れない並び（組みの最中に消えた file など）は
/// 合計に数えず退かせない（0 と読まない・C10）で、退かせる候補なら失敗に数える。印を読めない並びと、live な便か live を測れない便が
/// 持つ並びは退かせない。
pub(crate) fn shed(repo: &Path, cap: u64, policy: LockPolicy, live_of: &dyn Fn(&str) -> Option<bool>) -> (usize, Vec<String>) {
    let Ok(lanes) = numbers(repo) else {
        return (0, Vec::new());
    };
    let sizes: Vec<(usize, Option<u64>)> = lanes.iter().map(|n| (*n, measure(&lane_path(repo, *n)).map(|(bytes, _)| bytes))).collect();
    let Ok(_held) = lock(repo, policy) else {
        return (0, vec![format!("{LANE_DIR}/{LOCK}")]);
    };
    let mut total = sizes.iter().filter_map(|(_, bytes)| *bytes).fold(0_u64, u64::saturating_add);
    let (mut idle, mut failed) = (Vec::new(), Vec::new());
    for (n, bytes) in sizes {
        match (bytes, idle_of(repo, n, bytes.unwrap_or(0), live_of)) {
            (Some(_), Some(found)) => idle.push(found),
            (None, Some(_)) => failed.push(format!("{LANE_DIR}/{n}")),
            _ => {}
        }
    }
    idle.sort_by(|left, right| left.since.cmp(&right.since).then(left.n.cmp(&right.n)));
    let mut shed = 0_usize;
    for found in idle {
        if total <= cap {
            break;
        }
        if evict(repo, &found) {
            total = total.saturating_sub(found.bytes);
            shed = shed.saturating_add(1);
        } else {
            failed.push(format!("{LANE_DIR}/{}", found.n));
        }
    }
    (shed, failed)
}

/// 並び `n` が退かせる候補か（印が読め、印の便が並びを持たないか、持つ便が live でない周だけ `Some`）。
fn idle_of(repo: &Path, n: usize, bytes: u64, live_of: &dyn Fn(&str) -> Option<bool>) -> Option<Idle> {
    let since = std::fs::metadata(mark_path(repo, n)).and_then(|meta| meta.modified()).ok()?;
    let run = mark_of(repo, n)?;
    let held = is_held(repo, n);
    (!run.is_empty() && (!held || live_of(&run) == Some(false))).then_some(Idle { n, bytes, run, since, held })
}

/// 並びを丸ごと退かせる（move だけ・器の憲法 N1）。返された並びは `retired/<印の便>` へ（着地の時に遅らせた退役を済ませる）、
/// 便が持つ並びは symlink を外して便の元の path へ移し、印を外す。移せない周は symlink を戻して偽。
fn evict(repo: &Path, found: &Idle) -> bool {
    let (lane, link) = (lane_path(repo, found.n), worktree_path(repo, &found.run));
    let dest = if found.held { link.clone() } else { retired_path(repo, &found.run) };
    if found.held && std::fs::remove_file(&link).is_err() {
        return false;
    }
    if !move_tree(repo, &lane, &dest).is_empty() {
        if found.held {
            let _ = std::os::unix::fs::symlink(Path::new(LANE_DIR).join(found.n.to_string()), &link);
        }
        return false;
    }
    std::fs::remove_file(mark_path(repo, found.n)).is_ok()
}

/// 退役の後に便の木が在る path（pipe retire の stdout の `retired=`）。退役の前に並びを持っていた便（`held`）が clean な木を並びへ
/// 返した周は並びの木で、ほかは `retired/<run>`。
pub(crate) fn resting_place(repo: &Path, run: &str, held: Option<PathBuf>) -> PathBuf {
    held.filter(|lane| lane.is_dir()).unwrap_or_else(|| retired_path(repo, run))
}

#[cfg(test)]
mod tests {
    use super::super::declaration::build_lanes_at;
    use super::super::fixture::{append_all, event, scratch};
    use super::super::retire::{retire, Retire};
    use super::super::sweep::sweep;
    use super::super::{git_line, git_ok, repo_path, run_dir, worktree_path};
    use super::{claim, give_back, held, lane_path, mark_path, retired_path, take, LANE_DIR};
    use crate::fleet::store::LockPolicy;
    use crate::fleet::{EventKind, Stage};
    use crate::rules::manifest::Manifest;
    use std::path::{Path, PathBuf};
    use std::time::{Duration, SystemTime};

    /// 上限の歯が並びの target に置く file の大きさ（1 MiB）。
    const BLOB: usize = 1 << 20;

    /// 必須 key だけの宣言の本文の後ろに `extra` を足した `.vessel.toml`・`.gitignore`・2 file を持つ repo（`<root>/repo`）。
    fn repo_with(name: &str, extra: &str) -> PathBuf {
        let repo = scratch(name).join("repo");
        let _ = std::fs::create_dir_all(&repo);
        for args in [&["init", "-q", "-b", "main"][..], &["config", "user.name", "lane"], &["config", "user.email", "lane@example.invalid"]] {
            let _ = git_ok(&repo, args);
        }
        let decl = format!("schema = 1\nallowed-commands = [\"git\"]\ncommon-verify = [\"git diff --quiet\"]\n{extra}");
        for (file, body) in [(".vessel.toml", decl.as_str()), (".gitignore", "/target\n/ignored\n/.worktrees\n"), ("a.txt", "a1\n"), ("b.txt", "b1\n")] {
            let _ = std::fs::write(repo.join(file), body);
        }
        commit(&repo, "seed");
        repo
    }

    /// 木の全部を commit し、HEAD の sha を返す。
    fn commit(dir: &Path, message: &str) -> String {
        let _ = git_ok(dir, &["add", "-A"]);
        let _ = git_ok(dir, &["commit", "-q", "-m", message]);
        git_line(dir, &["rev-parse", "HEAD"]).unwrap_or_default()
    }

    fn policy() -> LockPolicy {
        LockPolicy::embedded().expect("埋め込みの lock 規則")
    }

    fn mtime(path: &Path) -> Option<SystemTime> {
        std::fs::metadata(path).and_then(|meta| meta.modified()).ok()
    }

    /// 並び `n` の印の字（無い周は `None`）。
    fn mark(repo: &Path, n: usize) -> Option<String> {
        std::fs::read_to_string(mark_path(repo, n)).ok()
    }

    /// 並びの上限の行 1 本だけの rules の写し（値は MiB）。
    fn cap_rules(mb: u64) -> Manifest {
        let row = format!("[[rule]]\nid = \"pipe.lanes_cap_mb\"\nkind = \"PipeLanesCapMb\"\nvalue = {mb}\nenabled = true\nruling = \"r\"\nruled_at = \"d\"\n");
        Manifest::parse(&format!("schema = 1\n\n{row}")).expect("上限の行の写し")
    }

    /// 並び `n` の target に 1 MiB の file を書き、印の新しさを UNIX の紀元から `at` 秒にする（時計に依らない）。
    fn fill(repo: &Path, n: usize, at: u64) {
        let lane = lane_path(repo, n);
        let _ = std::fs::create_dir_all(lane.join("target"));
        let _ = std::fs::write(lane.join("target/blob"), vec![7_u8; BLOB]);
        let when = SystemTime::UNIX_EPOCH.checked_add(Duration::from_secs(at)).unwrap_or(SystemTime::UNIX_EPOCH);
        let _ = std::fs::File::options().write(true).open(mark_path(repo, n)).and_then(|file| file.set_modified(when));
    }

    /// repo の隣の置き場（`<root>/state`）に便を段ごとに積み、便の repo を書く（掃除は repo を置き場の便から引く）。
    fn state_with(repo: &Path, runs: &[(&str, Stage)]) -> PathBuf {
        let state = repo.parent().unwrap_or(repo).join("state");
        let _ = std::fs::create_dir_all(&state);
        for (run, stage) in runs {
            append_all(&state, &[event(run, EventKind::RunStage, Some(*stage), None, None)]);
            let _ = std::fs::create_dir_all(run_dir(&state, run));
            let _ = std::fs::write(repo_path(&state, run), format!("{}\n", repo.display()));
        }
        state
    }

    /// 宣言の key `build-lanes` は値 true の sha だけ真で、false・無い・型違い（字）・宣言の無い sha は偽。
    #[test]
    fn vlane_only_declared_repos_take_a_lane() {
        let repo = repo_with("decl", "build-lanes = true\n");
        let on = git_line(&repo, &["rev-parse", "HEAD"]).unwrap_or_default();
        assert!(build_lanes_at(&repo, &on), "true の宣言は並びを使う");
        for (extra, why) in [("build-lanes = false\n", "false"), ("", "key が無い"), ("build-lanes = \"true\"\n", "字の値は不備")] {
            let decl = format!("schema = 1\nallowed-commands = [\"git\"]\ncommon-verify = [\"git diff --quiet\"]\n{extra}");
            let _ = std::fs::write(repo.join(".vessel.toml"), decl);
            let sha = commit(&repo, why);
            assert!(!build_lanes_at(&repo, &sha), "{why} の宣言は並びを使わない");
        }
        let _ = git_ok(&repo, &["rm", "-q", ".vessel.toml"]);
        let bare = commit(&repo, "no decl");
        assert!(!build_lanes_at(&repo, &bare), "宣言の無い sha は並びを使わない");
        assert!(build_lanes_at(&repo, &on), "作業の木でなく名指した sha の宣言を読む");
    }

    /// 返した clean な並びは次の便が使い回す: 便の path は並びの木を指し、HEAD は branch `<NAME>/<run>` で base、
    /// 中身の替わる file だけが書かれ（替わらない file の mtime は前のまま）、無視の file は target の外だけ消える。
    #[test]
    fn vlane_reuses_a_given_back_lane_and_keeps_unchanged_mtimes() {
        let repo = repo_with("reuse", "build-lanes = true\n");
        let first = git_line(&repo, &["rev-parse", "HEAD"]).unwrap_or_default();
        let tree = take(&repo, "r1", &first, policy()).unwrap_or_default();
        let lane = lane_path(&repo, 0);
        assert_eq!(held(&repo, "r1"), Some(0), "1 便目は並び 0 を持つ");
        assert_eq!(tree, worktree_path(&repo, "r1"));
        for (rel, body) in [("target/debug/x", "built\n"), ("ignored/y", "gen\n")] {
            let _ = std::fs::create_dir_all(lane.join(rel).parent().unwrap_or(&lane));
            let _ = std::fs::write(lane.join(rel), body);
        }
        let old = SystemTime::now().checked_sub(Duration::from_secs(3600)).unwrap_or(SystemTime::UNIX_EPOCH);
        let _ = std::fs::File::options().write(true).open(lane.join("b.txt")).and_then(|file| file.set_modified(old));
        assert_eq!(give_back(&repo, "r1"), Some(Vec::new()));
        assert!(std::fs::symlink_metadata(worktree_path(&repo, "r1")).is_err(), "返した便の path の symlink は外れる");
        assert!(lane.join("a.txt").is_file(), "clean な木は並びに残る");
        let _ = std::fs::write(repo.join("a.txt"), "a2\n");
        let second = commit(&repo, "a2");
        let _ = take(&repo, "r2", &second, policy());
        assert_eq!(held(&repo, "r2"), Some(0), "2 便目は返した並び 0 を使い回す");
        assert_eq!(git_line(&lane, &["rev-parse", "HEAD"]), Some(second));
        assert_eq!(git_line(&lane, &["rev-parse", "--abbrev-ref", "HEAD"]).as_deref(), Some("scribe2/r2"));
        assert_eq!(std::fs::read_to_string(lane.join("a.txt")).ok().as_deref(), Some("a2\n"), "替わる file は書かれる");
        assert_eq!(mtime(&lane.join("b.txt")), Some(old), "替わらない file の mtime は前のまま");
        assert!(lane.join("target/debug/x").is_file(), "target は消さない");
        assert!(!lane.join("ignored").exists(), "target の外の無視の file は消す");
        assert_eq!(mark(&repo, 0).as_deref(), Some("r2\n"));
    }

    /// 便が持つ並び（便の path の symlink が在る）はほかの便に渡さず、ほかの便は次の番号に新しく切り、返した後は使い回す。
    #[test]
    fn vlane_live_holder_keeps_its_lane() {
        let repo = repo_with("held", "build-lanes = true\n");
        let base = git_line(&repo, &["rev-parse", "HEAD"]).unwrap_or_default();
        let _ = take(&repo, "r1", &base, policy());
        let _ = take(&repo, "r2", &base, policy());
        assert_eq!(held(&repo, "r1"), Some(0));
        assert_eq!(held(&repo, "r2"), Some(1), "持たれた並び 0 は渡さない");
        assert_eq!(git_line(&lane_path(&repo, 0), &["rev-parse", "--abbrev-ref", "HEAD"]).as_deref(), Some("scribe2/r1"));
        assert_eq!(give_back(&repo, "r1"), Some(Vec::new()));
        let _ = take(&repo, "r3", &base, policy());
        assert_eq!(held(&repo, "r3"), Some(0), "返した並び 0 を使い回す");
        assert!(take(&repo, "r3", &base, policy()).is_err(), "同じ便の 2 度目は断る");
        assert_eq!(mark(&repo, 2), None, "断った取りは印を置かない");
    }

    /// clean でない木を返す周は今の退役の形（`retired/<run>` への move）で、印と symlink を外し、次の便は新しく切る。
    #[test]
    fn vlane_dirty_give_back_retires_the_tree() {
        let repo = repo_with("dirty", "build-lanes = true\n");
        let base = git_line(&repo, &["rev-parse", "HEAD"]).unwrap_or_default();
        let _ = take(&repo, "r1", &base, policy());
        let lane = lane_path(&repo, 0);
        let _ = std::fs::write(lane.join("untracked.txt"), "work\n");
        assert_eq!(give_back(&repo, "r1"), Some(Vec::new()));
        let retired = retired_path(&repo, "r1");
        assert!(retired.join("untracked.txt").is_file(), "clean でない木は退役の置き場へ中身ごと移る");
        assert!(!lane.exists() && !mark_path(&repo, 0).exists(), "並びの木と印は残らない");
        assert!(std::fs::symlink_metadata(worktree_path(&repo, "r1")).is_err());
        let _ = take(&repo, "r2", &base, policy());
        assert_eq!(held(&repo, "r2"), Some(0), "並び 0 に新しく切る");
        assert!(!lane.join("untracked.txt").exists());
    }

    /// ほかの便が lock の中で番号を予約した（印と便の path の symlink を置き、木をまだ切っていない）周は、取りがその番号を使わずに
    /// 次の番号に新しく切り、予約の印と symlink を替えない（並行の 2 便の取りの間を予約の置き場で決定的に作る）。symlink の無い
    /// 印だけの番号は予約と読まない（同じ置き場で symlink の 1 つだけを外した対照）。
    #[test]
    fn vlane_reserved_number_goes_to_no_other_run() {
        let repo = repo_with("lane-reserved", "build-lanes = true\n");
        let base = git_line(&repo, &["rev-parse", "HEAD"]).unwrap_or_default();
        assert_eq!(claim(&repo, "r1", policy()), Ok((0, None)), "r1 は木を切る前に並び 0 を予約する");
        assert!(!lane_path(&repo, 0).exists(), "予約だけで木はまだ無い");
        assert!(take(&repo, "r2", &base, policy()).is_ok());
        assert_eq!(held(&repo, "r2"), Some(1), "予約の在る並び 0 は取らずに並び 1 に切る");
        assert_eq!(held(&repo, "r1"), Some(0), "r1 の symlink は替えない");
        assert_eq!(mark(&repo, 0).as_deref(), Some("r1\n"), "r1 の印は替えない");
        assert!(!lane_path(&repo, 0).exists(), "予約の並び 0 に木を切らない");
        let _ = std::fs::remove_file(worktree_path(&repo, "r1"));
        assert!(take(&repo, "r3", &base, policy()).is_ok());
        assert_eq!(held(&repo, "r3"), Some(0), "symlink の無い印だけの番号は予約でない");
    }

    /// 落ちた取り（base が無い sha で木を切れない）は Err を返し、印と便の path の symlink を残さず、同じ番号を次の便が新しく切る。
    /// 使い回しの取りが落ちた周も、前の木を `retired/<前の便>` へ移したうえで印と symlink を残さない。
    #[test]
    fn vlane_failed_take_leaves_no_reservation() {
        let repo = repo_with("lane-failed", "build-lanes = true\n");
        let base = git_line(&repo, &["rev-parse", "HEAD"]).unwrap_or_default();
        let missing = "0".repeat(40);
        assert!(take(&repo, "r1", &missing, policy()).is_err(), "base の無い取りは落ちる");
        assert!(std::fs::symlink_metadata(worktree_path(&repo, "r1")).is_err(), "落ちた便の symlink は残らない");
        assert_eq!(mark(&repo, 0), None, "落ちた便の印は残らない");
        assert!(take(&repo, "r2", &base, policy()).is_ok());
        assert_eq!(held(&repo, "r2"), Some(0), "同じ番号を次の便が切る");
        assert_eq!(give_back(&repo, "r2"), Some(Vec::new()));
        assert!(take(&repo, "r3", &missing, policy()).is_err(), "使い回しの取りも base が無ければ落ちる");
        assert!(std::fs::symlink_metadata(worktree_path(&repo, "r3")).is_err(), "使い回しで落ちた便の symlink も残らない");
        assert_eq!(mark(&repo, 0), None, "使い回しで落ちた便の印も残らない");
        assert!(retired_path(&repo, "r2").join("a.txt").is_file(), "前の木は退役の置き場へ移る");
    }

    /// 便の path が並びを指していても、印がほかの便の並びは、返しが理由の行を返し、印とほかの便の symlink を替えず、clean な木を
    /// 次の便に渡さず、clean でない木を move しない。印が自分の便の並びは返る（同じ置き場で印の便の 1 つだけを替えた対照）。
    #[test]
    fn vlane_give_back_leaves_a_lane_marked_for_another_run() {
        let repo = repo_with("lane-foreign", "build-lanes = true\n");
        let base = git_line(&repo, &["rev-parse", "HEAD"]).unwrap_or_default();
        let _ = take(&repo, "r1", &base, policy());
        let lane = lane_path(&repo, 0);
        let _ = std::os::unix::fs::symlink(Path::new(LANE_DIR).join("0"), worktree_path(&repo, "r0"));
        assert_eq!(held(&repo, "r0"), Some(0), "r0 の path も並び 0 を指す");
        assert!(give_back(&repo, "r0").is_some_and(|failures| failures.len() == 1), "印がほかの便の並びは返さない");
        assert_eq!(mark(&repo, 0).as_deref(), Some("r1\n"), "印は替えない");
        assert_eq!(held(&repo, "r1"), Some(0), "ほかの便の symlink は外さない");
        assert!(take(&repo, "r2", &base, policy()).is_ok());
        assert_eq!(held(&repo, "r2"), Some(1), "clean な木を次の便に渡さない");
        let _ = std::fs::write(lane.join("untracked.txt"), "work\n");
        assert!(give_back(&repo, "r0").is_some_and(|failures| failures.len() == 1));
        assert!(lane.join("untracked.txt").is_file() && !retired_path(&repo, "r0").exists(), "clean でない木を move しない");
        assert_eq!(give_back(&repo, "r1"), Some(Vec::new()), "印が自分の便の並びは返る");
        assert!(retired_path(&repo, "r1").join("untracked.txt").is_file());
    }

    /// 並びの合計が上限（MiB）を越える周だけ、live な便が持たない並びを印の古い順に退かせて上限以下で止まり、live な便が持つ並びは
    /// 最も古くても退かせない。掃除の stderr の行は退かせた数を字 `lanes=` で名指す。
    #[test]
    fn vlcap_sheds_least_recent_idle_lanes() {
        let repo = repo_with("cap-order", "build-lanes = true\n");
        let base = git_line(&repo, &["rev-parse", "HEAD"]).unwrap_or_default();
        for run in ["r1", "r2", "r3", "r4"] {
            let _ = take(&repo, run, &base, policy());
        }
        for run in ["r1", "r3", "r4"] {
            let _ = give_back(&repo, run);
        }
        for (n, at) in [(0, 2000), (1, 1000), (2, 3000), (3, 4000)] {
            fill(&repo, n, at);
        }
        let state = state_with(&repo, &[("r1", Stage::Landed), ("r2", Stage::Spawned), ("r3", Stage::Landed), ("r4", Stage::Landed)]);
        let wide = sweep(&state, policy(), &cap_rules(40960));
        assert!((0..4).all(|n| lane_path(&repo, n).is_dir()), "上限の内では退かせない: {wide:?}");
        let line = sweep(&state, policy(), &cap_rules(3)).unwrap_or_default();
        assert!(!lane_path(&repo, 0).exists() && !lane_path(&repo, 2).exists(), "live でない並びを古い順に 2 つ退かせる: {line}");
        assert!(lane_path(&repo, 3).is_dir(), "上限以下になったら止まる（最も新しい並びは残る）");
        assert!(lane_path(&repo, 1).is_dir(), "live な便が持つ並びは最も古くても残る");
        assert_eq!(held(&repo, "r2"), Some(1));
        assert!(line.contains(" lanes=2"), "{line}");
    }

    /// 返された並びは `retired/<印の便>` へ、live でない便が持つ並びは symlink を外して便の元の path へ中身ごと移り、どちらも印が消え、
    /// 移した木の target は次の周の便の木の掃除が消す（追跡の file は残る）。
    #[test]
    fn vlcap_moves_lanes_to_where_the_run_would_be() {
        let repo = repo_with("cap-move", "build-lanes = true\n");
        let base = git_line(&repo, &["rev-parse", "HEAD"]).unwrap_or_default();
        let _ = take(&repo, "r1", &base, policy());
        let _ = take(&repo, "r2", &base, policy());
        let _ = give_back(&repo, "r1");
        for (n, at) in [(0, 1000), (1, 2000)] {
            fill(&repo, n, at);
        }
        let state = state_with(&repo, &[("r1", Stage::Landed), ("r2", Stage::Failed)]);
        let _ = sweep(&state, policy(), &cap_rules(0));
        let (retired, home) = (retired_path(&repo, "r1"), worktree_path(&repo, "r2"));
        assert!(retired.join("a.txt").is_file() && retired.join("target/blob").is_file(), "返された並びは retired/<印の便> へ移る");
        assert!(std::fs::symlink_metadata(&home).is_ok_and(|meta| meta.is_dir()), "便が持つ並びは symlink を外して便の元の path へ移る");
        assert!(home.join("target/blob").is_file(), "中身ごと移る");
        assert!((0..2).all(|n| !lane_path(&repo, n).exists() && !mark_path(&repo, n).exists()), "並びの木と印は残らない");
        assert_eq!(held(&repo, "r2"), None);
        let _ = sweep(&state, policy(), &cap_rules(0));
        assert!(!retired.join("target").exists() && !home.join("target").exists(), "移した木の target は次の周の掃除が消す");
        assert!(retired.join("a.txt").is_file() && home.join("a.txt").is_file(), "追跡の file は残る");
    }

    /// 上限の行を読めない rules の写し（同じ置き場と並びから行 1 本だけを外した形）では並びを 1 つも退かせず、行が在れば退かせる。
    #[test]
    fn vlcap_keeps_lanes_without_the_rule() {
        let repo = repo_with("cap-norule", "build-lanes = true\n");
        let base = git_line(&repo, &["rev-parse", "HEAD"]).unwrap_or_default();
        let _ = take(&repo, "r1", &base, policy());
        let _ = give_back(&repo, "r1");
        fill(&repo, 0, 1000);
        let state = state_with(&repo, &[("r1", Stage::Landed)]);
        let bare = Manifest::parse("schema = 1\n").expect("行の無い写し");
        let _ = sweep(&state, policy(), &bare);
        assert!(lane_path(&repo, 0).is_dir() && mark_path(&repo, 0).is_file(), "行を読めない周は退かせない");
        let _ = sweep(&state, policy(), &cap_rules(0));
        assert!(!lane_path(&repo, 0).exists(), "行が在れば同じ並びを退かせる");
    }

    /// 並びを持つ便の pipe retire は clean な木を並びへ返し、stdout の字 `retired=` は並びの木の path を名指す（無い `retired/<run>` を
    /// 名指さない）。
    #[test]
    fn vlcap_retire_names_the_lane_tree() {
        let repo = repo_with("cap-retire", "build-lanes = true\n");
        let base = git_line(&repo, &["rev-parse", "HEAD"]).unwrap_or_default();
        let _ = take(&repo, "r1", &base, policy());
        let state = state_with(&repo, &[]);
        let manifest = Manifest::parse("schema = 1\n").expect("空の rules の写し");
        let entry = Retire {
            run: "r1",
            bead: "b",
            repo: &repo,
            state_dir: &state,
            stage: Stage::Failed,
            policy: policy(),
            bd: "bd",
            fold_only: false,
            manifest: &manifest,
        };
        let outcome = retire(&entry);
        let lane = lane_path(&repo, 0);
        assert_eq!(outcome.out, vec![format!("run=r1 retired={}", lane.display())], "{:?}", outcome.err);
        assert!(lane.join("a.txt").is_file() && !retired_path(&repo, "r1").exists(), "木は並びに残る");
    }
}
