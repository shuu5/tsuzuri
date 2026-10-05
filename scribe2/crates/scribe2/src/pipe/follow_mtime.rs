//! 追随の載せ替えで、便の自分の差の file のうち中身の替わらない物の更新の時刻を載せ替えの前の値へ戻す（設計 pipeline.md
//! §70・判断の記録 ADR-62・器の行 v-follow-mtime）。
//!
//! 載せ替え（[`super::follow_step::rebase`]）は main の木を取り出してから便の commit を当て直すので、便の自分の差の file を
//! 中身が同じのまま書き直し、cargo が組み直す。ここは載せ替えの直前に控え（[`snapshot`]）、通った周に、木の上の型と中身と
//! 作業木の byte の指紋が控えと同じ普通の file だけ時刻をナノ秒まで戻す（[`settle`]）。dir の時刻は戻さない。読めない・
//! 確かめられない周は 1 file も戻さない。戻しのどの失敗も載せ替えの結末を替えず、載せ替えが通った追随ごとに便の dir の
//! [`RECORD_FILE`] へ 1 行を書く（書けない周も結末を替えない・衝突の周は何も書かない）。

use super::land::WorktreeCheck;
use super::{git_bytes, run_dir};
use crate::fleet::json_lite::{self, Value};
use crate::fleet::store::{append_line, LockPolicy};
use std::collections::BTreeMap;
use std::path::Path;
use std::time::SystemTime;

/// 記録の file の名（便の dir の直下・追随ごとに 1 行）。
pub(in crate::pipe) const RECORD_FILE: &str = "follow-mtime.jsonl";

/// 戻す型（木の上の普通の file・実行の bit の有無）。
const PLAIN_MODES: [&str; 2] = ["100644", "100755"];

/// 控えの側で git か file を読めない周の倒れの語。
const UNREADABLE_BEFORE: &str = "before:unreadable";

/// 記録の置き場（便の dir と lock の待ち方）。
pub(in crate::pipe) struct Log<'a> {
    /// 置き場。
    pub(in crate::pipe) state_dir: &'a Path,
    /// 便 id。
    pub(in crate::pipe) run: &'a str,
    /// lock の待ち方。
    pub(in crate::pipe) policy: LockPolicy,
}

/// 篩で外した理由の種（**閉じた列**・記録の欄の名と並び）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::pipe) enum Skip {
    /// 葉か途中の dir が symlink・葉が普通の file でない。
    Link,
    /// 木の上の型が普通の file でない（中の印の commit の項ほか）。
    Kind,
    /// 載せ替えの後の木に path が無い。
    Gone,
    /// 木の上の型が替わった。
    Mode,
    /// 木の上の中身（blob）が替わった。
    Blob,
    /// 作業木の byte の指紋が替わった（属性の定めの改行・filter）。
    Bytes,
    /// 時刻を書けない。
    Write,
    /// 書いた後の照らし直しで byte が替わっていた（時刻は今にした）。
    Changed,
}

impl Skip {
    /// 全部の種（記録の欄の並び）。
    const ALL: [Self; 8] = [Self::Link, Self::Kind, Self::Gone, Self::Mode, Self::Blob, Self::Bytes, Self::Write, Self::Changed];

    /// 記録の欄の名。
    fn as_str(self) -> &'static str {
        match self {
            Self::Link => "link",
            Self::Kind => "kind",
            Self::Gone => "gone",
            Self::Mode => "mode",
            Self::Blob => "blob",
            Self::Bytes => "bytes",
            Self::Write => "write",
            Self::Changed => "changed",
        }
    }
}

/// 控えの 1 file（path・木の上の型と中身・作業木の byte の指紋・時刻）。
pub(in crate::pipe) struct Kept {
    path: String,
    mode: String,
    blob: String,
    bytes: String,
    mtime: SystemTime,
}

/// 載せ替えの直前の控え（**閉じた 2 値**）: 取れた控えと篩で外した種、か、取れなかった理由の語。
pub(in crate::pipe) enum Snapshot {
    /// 控えた。
    Taken(Vec<Kept>, Vec<Skip>),
    /// 測れずに倒した（1 file も戻さない）。値は理由の語。
    Fell(String),
}

/// path の葉の姿。
enum Leaf {
    /// 葉が普通の file で、途中の dir に symlink を挟まない。値は時刻。
    Plain(SystemTime),
    /// 戻さない姿（symlink・dir ほか）。
    Other,
    /// 読めない。
    Unreadable,
}

/// 木の根から 1 段ずつ `symlink_metadata` で見た path の葉の姿（根そのものは並びの symlink でもよい）。
fn leaf(worktree: &Path, path: &str) -> Leaf {
    let mut at = worktree.to_path_buf();
    let mut parts = path.split('/').peekable();
    while let Some(part) = parts.next() {
        at.push(part);
        let Ok(meta) = std::fs::symlink_metadata(&at) else {
            return Leaf::Unreadable;
        };
        if meta.file_type().is_symlink() {
            return Leaf::Other;
        }
        if parts.peek().is_none() {
            return match (meta.is_file(), meta.modified()) {
                (false, _) => Leaf::Other,
                (true, Ok(mtime)) => Leaf::Plain(mtime),
                (true, Err(_)) => Leaf::Unreadable,
            };
        }
    }
    Leaf::Unreadable
}

/// NUL で区切った git の出力の項。
fn split_z(bytes: &[u8]) -> Vec<String> {
    bytes.split(|byte| *byte == 0).filter(|item| !item.is_empty()).map(|item| String::from_utf8_lossy(item).into_owned()).collect()
}

/// `HEAD` の木の項の型と中身（`git ls-tree -r -z --full-tree HEAD`）。読めない周は `None`。
fn tree_entries(worktree: &Path) -> Option<BTreeMap<String, (String, String)>> {
    let bytes = git_bytes(worktree, &["ls-tree", "-r", "-z", "--full-tree", "HEAD"])?;
    split_z(&bytes)
        .into_iter()
        .map(|item| {
            let (head, path) = item.split_once('\t')?;
            let mut fields = head.split(' ');
            let (mode, _, blob) = (fields.next()?, fields.next()?, fields.next()?);
            Some((path.to_owned(), (mode.to_owned(), blob.to_owned())))
        })
        .collect()
}

/// 作業木の file の byte の指紋（filter を通さない `git hash-object`・path の順）。読めない周は `None`。
fn fingerprints(worktree: &Path, paths: &[&str]) -> Option<Vec<String>> {
    let mut args = vec!["hash-object", "--no-filters", "--"];
    args.extend_from_slice(paths);
    let found: Vec<String> = String::from_utf8_lossy(&git_bytes(worktree, &args)?).lines().map(str::to_owned).collect();
    (found.len() == paths.len()).then_some(found)
}

/// 木が clean でない周の倒れの語（`<side>:dirty` か `<side>:unreadable`・`side` は before か after）。
fn unclean(worktree: &Path, side: &str) -> Option<String> {
    let check = WorktreeCheck::judge(worktree);
    (!check.is_clean()).then(|| format!("{side}:{}", check.as_str()))
}

/// 載せ替えの直前の控え: 木が clean な周だけ、便の自分の差（`base` から `HEAD` まで・改名は消しと足しに割る・消した
/// path は除く）の path の型と中身と作業木の byte の指紋と時刻を取る。木の上で普通の file でない path と葉か途中の dir が
/// symlink の path は外した種に数える。git か file の読みが 1 つでも落ちた周は [`Snapshot::Fell`]。
pub(in crate::pipe) fn snapshot(worktree: &Path, base: &str) -> Snapshot {
    if let Some(why) = unclean(worktree, "before") {
        return Snapshot::Fell(why);
    }
    let diff = git_bytes(worktree, &["diff", "--name-only", "--no-renames", "--diff-filter=d", "-z", base, "HEAD"]);
    let (Some(diff), Some(tree)) = (diff, tree_entries(worktree)) else {
        return Snapshot::Fell(UNREADABLE_BEFORE.to_owned());
    };
    let (mut found, mut skipped) = (Vec::new(), Vec::new());
    for path in split_z(&diff) {
        let Some((mode, blob)) = tree.get(&path) else {
            return Snapshot::Fell(UNREADABLE_BEFORE.to_owned());
        };
        if !PLAIN_MODES.contains(&mode.as_str()) {
            skipped.push(if mode == "120000" { Skip::Link } else { Skip::Kind });
            continue;
        }
        match leaf(worktree, &path) {
            Leaf::Plain(mtime) => found.push((path, mode.clone(), blob.clone(), mtime)),
            Leaf::Other => skipped.push(Skip::Link),
            Leaf::Unreadable => return Snapshot::Fell(UNREADABLE_BEFORE.to_owned()),
        }
    }
    let paths: Vec<&str> = found.iter().map(|(path, ..)| path.as_str()).collect();
    let Some(bytes) = fingerprints(worktree, &paths) else {
        return Snapshot::Fell(UNREADABLE_BEFORE.to_owned());
    };
    let kept = found.into_iter().zip(bytes).map(|((path, mode, blob, mtime), bytes)| Kept { path, mode, blob, bytes, mtime });
    Snapshot::Taken(kept.collect(), skipped)
}

/// file の時刻を書く（開くのは書きの許しで・中身は替えない）。
fn stamp(worktree: &Path, path: &str, when: SystemTime) -> bool {
    std::fs::File::options().write(true).open(worktree.join(path)).and_then(|file| file.set_modified(when)).is_ok()
}

/// 追随 1 回の数え（記録の 1 行）。
struct Tally {
    restored: u64,
    skipped: Vec<Skip>,
    fell: Option<String>,
}

impl Tally {
    /// 測れずに倒した周。
    fn fell(why: &str) -> Self {
        Self { restored: 0, skipped: Vec::new(), fell: Some(why.to_owned()) }
    }

    /// 記録の 1 行（`rebase`・戻した数・外した数・種ごとの数・倒した理由）。
    fn line(&self, rebase: &str) -> String {
        let count = |kind: Skip| self.skipped.iter().filter(|found| **found == kind).count() as u64;
        let mut fields = vec![
            ("rebase", Value::Str(rebase.to_owned())),
            ("restored", Value::Num(self.restored)),
            ("skipped", Value::Num(self.skipped.len() as u64)),
        ];
        fields.extend(Skip::ALL.iter().map(|kind| (kind.as_str(), Value::Num(count(*kind)))));
        fields.push(("fell", self.fell.clone().map_or(Value::Null, Value::Str)));
        json_lite::write_object(&fields)
    }
}

/// 型と中身と葉の篩（載せ替えの後の木）。通った控えと、外した種を返す。読めない周は `None`。
fn sift(worktree: &Path, kept: Vec<Kept>, skipped: &mut Vec<Skip>) -> Option<Vec<Kept>> {
    let tree = tree_entries(worktree)?;
    let mut same = Vec::new();
    for item in kept {
        let verdict = match tree.get(&item.path) {
            None => Some(Skip::Gone),
            Some((mode, _)) if *mode != item.mode => Some(Skip::Mode),
            Some((_, blob)) if *blob != item.blob => Some(Skip::Blob),
            Some(_) => match leaf(worktree, &item.path) {
                Leaf::Plain(_) => None,
                Leaf::Other => Some(Skip::Link),
                Leaf::Unreadable => return None,
            },
        };
        match verdict {
            Some(kind) => skipped.push(kind),
            None => same.push(item),
        }
    }
    Some(same)
}

/// 載せ替えが通った周の戻し: 後の木が clean で、型と中身と作業木の byte の指紋が控えと同じ file だけ時刻を控えの値へ書き、
/// 書いた後に byte の指紋を 1 度照らし直す（違えばその file の時刻を今にする）。
fn restore(worktree: &Path, snapshot: Snapshot) -> Tally {
    let (kept, mut skipped) = match snapshot {
        Snapshot::Taken(kept, skipped) => (kept, skipped),
        Snapshot::Fell(why) => return Tally::fell(&why),
    };
    if let Some(why) = unclean(worktree, "after") {
        return Tally::fell(&why);
    }
    let Some(same) = sift(worktree, kept, &mut skipped) else {
        return Tally::fell("after:unreadable");
    };
    let Some(bytes) = fingerprints(worktree, &same.iter().map(|item| item.path.as_str()).collect::<Vec<_>>()) else {
        return Tally::fell("after:unreadable");
    };
    let mut written = Vec::new();
    for (item, now) in same.into_iter().zip(bytes) {
        if now != item.bytes {
            skipped.push(Skip::Bytes);
        } else if stamp(worktree, &item.path, item.mtime) {
            written.push(item);
        } else {
            skipped.push(Skip::Write);
        }
    }
    let again = fingerprints(worktree, &written.iter().map(|item| item.path.as_str()).collect::<Vec<_>>());
    let mut restored = 0;
    for (n, item) in written.iter().enumerate() {
        if again.as_ref().and_then(|found| found.get(n)) == Some(&item.bytes) {
            restored += 1;
            continue;
        }
        stamp(worktree, &item.path, SystemTime::now());
        skipped.push(Skip::Changed);
    }
    let fell = again.is_none().then(|| "recheck:unreadable".to_owned());
    Tally { restored, skipped, fell }
}

/// 載せ替えの後の 1 段: 通った周（`passed`）だけ戻しを撃ち、便の dir の記録へ 1 行を書く（書けない周も黙って進む・載せ替えの
/// 結末を替えない）。衝突の周は何も戻さず何も書かない（呼び手の後始末と記帳は今のまま・置き場に file を足さない）。
pub(in crate::pipe) fn settle(worktree: &Path, snapshot: Snapshot, passed: bool, rebase: &str, log: &Log<'_>) {
    if passed {
        let line = restore(worktree, snapshot).line(rebase);
        let _ = append_line(&run_dir(log.state_dir, log.run).join(RECORD_FILE), &line, log.policy);
    }
}

#[cfg(test)]
mod tests {
    use super::super::fixture::scratch;
    use super::super::follow_step::rebase;
    use super::super::run_dir;
    use super::{leaf, settle, snapshot, Leaf, Log, RECORD_FILE};
    use crate::fleet::store::LockPolicy;
    use crate::invocation::Invocation;
    use std::os::unix::fs::{symlink, PermissionsExt};
    use std::path::{Path, PathBuf};
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    /// 便か main の側の commit を木に書く手。
    type Side = fn(&Path);

    /// 載せ替えの前に追跡の普通の file へ付ける古い時刻（ナノ秒の端を持つ）。
    fn old() -> SystemTime {
        UNIX_EPOCH + Duration::new(1_600_000_000, 123_456_789)
    }

    /// git を 1 回撃ち、stdout を trim して返す（失敗は読み手の assert が落とす）。
    fn git(dir: &Path, args: &[&str]) -> String {
        Invocation::new("git")
            .arg("-C")
            .arg(dir)
            .args(args)
            .output()
            .map(|out| String::from_utf8_lossy(&out.stdout).trim().to_owned())
            .unwrap_or_default()
    }

    /// 10 行の本文（行ごとに file の名と番号・`edits` の行だけを替える）を木に書く。
    fn put(repo: &Path, path: &str, edits: &[(usize, &str)]) {
        let line = |n: usize| edits.iter().find(|(at, _)| *at == n).map_or_else(|| format!("{path} {n}"), |(_, text)| (*text).to_owned());
        let _ = std::fs::write(repo.join(path), (1..=10).map(|n| line(n) + "\n").collect::<String>());
    }

    /// 全部を add して commit し、先端の sha を返す。
    fn commit(repo: &Path, message: &str) -> String {
        git(repo, &["add", "-A"]);
        git(repo, &["commit", "-q", "-m", message]);
        git(repo, &["rev-parse", "HEAD"])
    }

    /// path の時刻（symlink は自分の時刻）。
    fn mtime(repo: &Path, path: &str) -> SystemTime {
        std::fs::symlink_metadata(repo.join(path)).and_then(|meta| meta.modified()).unwrap_or(UNIX_EPOCH)
    }

    /// base（8 file）の上で main と便（branch run・木はこの先端）に `main` と `own` の commit を 1 本ずつ積み、追跡の普通の
    /// file の時刻を全部 [`old`] にした repo（`(repo, base, main)`・置き場は repo の隣の state）。
    fn fixture(name: &str, own: Side, main: Side) -> (PathBuf, String, String) {
        let repo = scratch(name).join("repo");
        let _ = std::fs::create_dir_all(&repo);
        git(&repo, &["init", "-q", "-b", "main"]);
        git(&repo, &["config", "user.name", "mutant"]);
        git(&repo, &["config", "user.email", "mutant@example.invalid"]);
        for path in ["own.rs", "main.rs", "both.rs", "ret.rs", "ren.rs", "renc.rs", "m.sh", "crlf.txt"] {
            put(&repo, path, &[]);
        }
        let base = commit(&repo, "base");
        main(&repo);
        let tip = commit(&repo, "main");
        git(&repo, &["checkout", "-q", "-b", "run", &base]);
        own(&repo);
        commit(&repo, "own");
        for path in git(&repo, &["ls-files"]).lines().filter(|path| std::fs::symlink_metadata(repo.join(path)).is_ok_and(|meta| meta.is_file())) {
            let _ = std::fs::File::options().write(true).open(repo.join(path)).and_then(|file| file.set_modified(old()));
        }
        (repo, base, tip)
    }

    /// 便の差: own.rs だけ・both.rs の 1 行目・ret.rs の 1 行目（main と同じ字）と 10 行目・ren.rs と renc.rs の改名・m.sh・
    /// crlf.txt・own.rs を指す symlink の ln。
    fn own_all(repo: &Path) {
        for path in ["own.rs", "both.rs", "m.sh", "crlf.txt"] {
            put(repo, path, &[(1, "own")]);
        }
        put(repo, "ret.rs", &[(1, "same"), (10, "own")]);
        let _ = std::fs::rename(repo.join("ren.rs"), repo.join("ren2.rs"));
        let _ = std::fs::rename(repo.join("renc.rs"), repo.join("renc2.rs"));
        let _ = symlink("own.rs", repo.join("ln"));
    }

    /// main の差: main.rs だけ・both.rs の 10 行目・ret.rs の 1 行目（便と同じ字）・renc.rs の 5 行目・m.sh の実行の bit・
    /// crlf.txt の改行を CRLF にする属性の定め。
    fn main_all(repo: &Path) {
        for (path, edits) in [("main.rs", &[(1, "main")][..]), ("both.rs", &[(10, "main")]), ("ret.rs", &[(1, "same")]), ("renc.rs", &[(5, "main")])] {
            put(repo, path, edits);
        }
        let _ = std::fs::set_permissions(repo.join("m.sh"), std::fs::Permissions::from_mode(0o755));
        let _ = std::fs::write(repo.join(".gitattributes"), "crlf.txt text eol=crlf\n");
    }

    /// 便の dir の記録の行。
    fn record(repo: &Path) -> Vec<String> {
        let path = run_dir(&repo.with_file_name("state"), "r1").join(RECORD_FILE);
        std::fs::read_to_string(path).unwrap_or_default().lines().map(str::to_owned).collect()
    }

    /// 記録の置き場（repo の隣の state・便 r1）を作って `with` に渡す。
    fn logged<T>(repo: &Path, with: impl FnOnce(&Log<'_>) -> T) -> T {
        let state = repo.with_file_name("state");
        let _ = std::fs::create_dir_all(run_dir(&state, "r1"));
        with(&Log { state_dir: &state, run: "r1", policy: LockPolicy::embedded().expect("埋め込みの lock 規則") })
    }

    /// `fixture` の repo で共有の載せ替えの 1 段を撃つ（`(repo, base, main, rc, 記録の行)`）。
    fn follow(name: &str, own: Side, main: Side) -> (PathBuf, String, String, bool, Vec<String>) {
        let (repo, base, tip) = fixture(name, own, main);
        let passed = logged(&repo, |log| rebase(&repo, &base, &tip, log));
        let lines = record(&repo);
        (repo, base, tip, passed, lines)
    }

    /// 自分の差だけの file は時刻がナノ秒まで前の値に戻り中身も同じ・main の差だけの file は新しいまま。
    #[test]
    fn vfmtime_own_only_file_gets_the_old_time_back_and_main_only_file_stays_new() {
        let (repo, _, main, passed, _) = follow("own-only", own_all, main_all);
        assert!(passed && git(&repo, &["rev-parse", "HEAD~1"]) == main, "載せ替えが通り木の base は main の先端");
        assert_eq!(mtime(&repo, "own.rs"), old(), "自分の差だけの file はナノ秒まで前の値");
        assert_eq!(std::fs::read_to_string(repo.join("own.rs")).unwrap_or_default().lines().next(), Some("own"), "中身は便の字");
        assert!(mtime(&repo, "main.rs") > old(), "main の差だけの file は新しいまま");
    }

    /// 中身が行って戻る file と中身の同じ改名の先は戻り、両方の差が当たった file と main が元を直した改名の先は新しいまま。
    #[test]
    fn vfmtime_returning_and_renamed_same_content_come_back_and_changed_content_stays_new() {
        let (repo, _, _, passed, _) = follow("changed", own_all, main_all);
        assert!(passed, "載せ替えが通る");
        assert_eq!(mtime(&repo, "ret.rs"), old(), "main が同じ字の直しを先に持つ file は戻る");
        assert_eq!(mtime(&repo, "ren2.rs"), old(), "中身の同じ改名の先は戻る");
        assert!(mtime(&repo, "both.rs") > old(), "同じ file に両方の差が当たった file は新しいまま");
        assert!(mtime(&repo, "renc2.rs") > old(), "main が元の file を直した改名の先は新しいまま");
        assert!(std::fs::read_to_string(repo.join("renc2.rs")).unwrap_or_default().contains("\nmain\n"), "main の直しが改名の先に入る");
    }

    /// 改行の属性で byte が替わった file・実行の bit だけが替わった file は新しいまま・途中の dir の symlink は戻さない姿。
    #[test]
    fn vfmtime_eol_mode_and_symlinked_dir_stay_new() {
        let (repo, _, _, passed, _) = follow("eol-mode", own_all, main_all);
        assert!(passed && git(&repo, &["status", "--porcelain"]).is_empty(), "載せ替えが通り後の木は clean");
        assert!(std::fs::read(repo.join("crlf.txt")).unwrap_or_default().contains(&b'\r'), "作業木の byte は CRLF");
        assert!(mtime(&repo, "crlf.txt") > old(), "blob が同じで作業木の byte が替わった file は新しいまま");
        assert!(mtime(&repo, "m.sh") > old(), "実行の bit だけが替わった file は新しいまま");
        let _ = std::fs::create_dir_all(repo.join("real"));
        put(&repo, "real/x.rs", &[]);
        let _ = symlink(repo.join("real"), repo.join("via"));
        assert!(matches!(leaf(&repo, "real/x.rs"), Leaf::Plain(_)), "普通の file");
        assert!(matches!(leaf(&repo, "via/x.rs"), Leaf::Other), "途中の dir が symlink の path は戻さない姿");
    }

    /// 追随ごとに便の dir の記録へ 1 行（戻した数・外した数・種ごとの数・倒していない）。
    #[test]
    fn vfmtime_record_line_counts_restored_and_skipped() {
        let (_, base, main, passed, lines) = follow("record", own_all, main_all);
        assert!(passed, "載せ替えが通る");
        let want = format!(
            "{{\"rebase\":\"{base}..{main}\",\"restored\":3,\"skipped\":5,\"link\":1,\"kind\":0,\"gone\":0,\"mode\":1,\"blob\":2,\
             \"bytes\":1,\"write\":0,\"changed\":0,\"fell\":null}}"
        );
        assert_eq!(lines, vec![want], "記録は 1 行");
    }

    /// 戻る見本（自分の差だけ）から 1 句ずつ外す: 控えの base を読めない・木が clean でない・衝突。どれも 1 file も戻さない
    /// （衝突の周は記録も書かない）。
    #[test]
    fn vfmtime_unreadable_dirty_and_conflict_rounds_restore_nothing() {
        let own: Side = |repo| put(repo, "own.rs", &[(1, "own")]);
        let main: Side = |repo| put(repo, "main.rs", &[(1, "main")]);
        let fell = |lines: &[String]| lines.first().and_then(|line| line.rsplit("\"fell\":").next().map(str::to_owned));
        let (repo, _, _, passed, lines) = follow("refuse-ok", own, main);
        assert!(passed && mtime(&repo, "own.rs") == old() && fell(&lines).as_deref() == Some("null}"), "正しい見本は戻る: {lines:?}");
        let (repo, base, tip) = fixture("refuse-unreadable", own, main);
        let kept = snapshot(&repo, "refs/heads/no-such-base");
        git(&repo, &["rebase", "-q", "--onto", &tip, &base]);
        logged(&repo, |log| settle(&repo, kept, true, "b..m", log));
        assert!(mtime(&repo, "own.rs") > old(), "控えを読めない周は戻さない");
        assert_eq!(fell(&record(&repo)).as_deref(), Some("\"before:unreadable\"}"));
        let (repo, base, tip) = fixture("refuse-dirty", own, main);
        let _ = std::fs::write(repo.join("note.txt"), "x\n");
        let passed = logged(&repo, |log| rebase(&repo, &base, &tip, log));
        assert!(passed && mtime(&repo, "own.rs") > old(), "木が clean でない周は戻さない");
        assert_eq!(fell(&record(&repo)).as_deref(), Some("\"before:dirty\"}"));
        let (repo, _, _, passed, lines) = follow("refuse-clash", own, |repo| put(repo, "own.rs", &[(1, "main")]));
        git(&repo, &["rebase", "--abort"]);
        assert!(!passed && mtime(&repo, "own.rs") > old(), "衝突の周は戻さない");
        assert!(lines.is_empty(), "衝突の周は記録を書かない: {lines:?}");
    }
}
