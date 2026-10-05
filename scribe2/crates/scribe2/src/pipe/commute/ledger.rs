//! 差の当たりで通した組の後の着地の記帳（判断の記録 ADR-60 の決定 (4) の着地の側・要件 FR1039）。
//!
//! 受付が通した組（kind `OverlapCommuted` の行の detail の `run=` か `with=` に名の在る便）の、その後の着地の出来事を、出来事の
//! 記録の kind `OverlapFollowed` に 1 件ずつ記す。語は閉じた 5 つ（[`Followed`]）で、積み直しの衝突・撃ち直しの間に main がまた
//! 動いた周（stale）・追随の再 gate の不合格・着地の列の候補の木の赤・着地。着地の行は、着地の差と差の file の変えた行の数と、
//! 片方にだけ在る変えた行の数を添える（[`landed`]）。組に名の無い便は何も書かない。
//!
//! 記帳は撤退の数えの材料で、着地の判じに使わない。記録を読めない・書けない周も、着地の結末と rc は替えない（着地の後の検出の
//! 起こしと同じ極性）。

use crate::fleet::store::{self, LockPolicy};
use crate::fleet::{cli, Case, Event, EventKind, SCHEMA};
use crate::pipe::git_bytes;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

/// 組の後の着地の出来事の語（閉じた 5 つ・判断の記録 ADR-60 の決定 (4)）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::pipe) enum Followed {
    /// 追随の積み直し（rebase）が衝突した。
    Conflict,
    /// 撃ち直しの間に main がまた動いた（stale）。
    Stale,
    /// 追随の再 gate が不合格（FAIL）だった。
    RegateFail,
    /// 着地の列の候補の木の検査が赤だった（積んだ便の全部に記す・どの便が赤かは帰属しない）。
    TrainRed,
    /// 着地した（[`landed`] が数えを添える）。
    Landed,
}

impl Followed {
    /// detail の `word=` の字。
    pub(in crate::pipe) fn as_str(self) -> &'static str {
        match self {
            Self::Conflict => "conflict",
            Self::Stale => "stale",
            Self::RegateFail => "regate-fail",
            Self::TrainRed => "train-red",
            Self::Landed => "landed",
        }
    }
}

/// 記す便（置き場・便 id・契約の bead・lock の待ち方）。
pub(in crate::pipe) struct Mark<'a> {
    /// 置き場（組の行を読み、記帳を書く）。
    pub(in crate::pipe) state_dir: &'a Path,
    /// 便 id。
    pub(in crate::pipe) run: &'a str,
    /// 便の契約の bead id（行の `bead`）。
    pub(in crate::pipe) bead: &'a str,
    /// lock の待ち方。
    pub(in crate::pipe) policy: LockPolicy,
}

/// 便が組に名の在る周だけ、語 `word` を 1 件記す（detail は `run=<便 id> word=<語>` と、空でない `tail` を空白 1 つの後に）。
pub(in crate::pipe) fn note(mark: &Mark<'_>, word: Followed, tail: &str) {
    if paired(mark.state_dir, mark.run) {
        write(mark, word, tail);
    }
}

/// 着地した便が組に名の在る周だけ、語 landed を 1 件記す。尾は `diff=<着地の差の変えた行の数> patch=<差の file の変えた行の数>
/// off=<片方にだけ在る変えた行の数>`。着地の差は着地の commit `sha` とその親の差、差の file は契約が名指す `patch` を `sha` の木から
/// 読む。読めない側の数と off は字 `-`。
pub(in crate::pipe) fn landed(mark: &Mark<'_>, repo: &Path, sha: &str, patch: Option<&str>) {
    if !paired(mark.state_dir, mark.run) {
        return;
    }
    let parent = format!("{sha}^");
    let read = |args: &[&str]| git_bytes(repo, args).map(|bytes| String::from_utf8_lossy(&bytes).into_owned());
    let landing = read(&["diff", "--no-color", &parent, sha]);
    let file = patch.and_then(|path| read(&["show", &format!("{sha}:{path}")]));
    write(mark, Followed::Landed, &counts(landing.as_deref(), file.as_deref()));
}

/// 1 件を書く（書けない周は捨てる・着地の結末を替えない）。
fn write(mark: &Mark<'_>, word: Followed, tail: &str) {
    let kind = EventKind::OverlapFollowed;
    let mut detail = format!("run={} word={}", mark.run, word.as_str());
    if !tail.is_empty() {
        detail = format!("{detail} {tail}");
    }
    let event = Event {
        schema: SCHEMA,
        ts: cli::now_utc(),
        kind,
        run: String::new(),
        bead: mark.bead.to_owned(),
        host: cli::host(),
        actor: kind.default_actor().to_owned(),
        stage: None,
        seat: None,
        pid: None,
        detail: Some(detail),
        allowance: None,
        registration: None,
        mark: None,
        account: None,
        cost: None,
        rule: None,
        case: Some(Case::Followed),
    };
    let _ = store::append(mark.state_dir, &event, mark.policy);
}

/// 便 `run` が受付の通した組に名の在るか（記録を読めない周は偽）。
fn paired(state_dir: &Path, run: &str) -> bool {
    store::read_all(state_dir).is_ok_and(|events| {
        events.iter().filter(|event| event.kind == EventKind::OverlapCommuted).any(|event| names(event.detail.as_deref().unwrap_or_default(), run))
    })
}

/// 組の detail（`run=<id> with=<id>,<id> files=… main=…`）の `run=` の値か `with=` の 1 つが空でない `run` と等しいか。
fn names(detail: &str, run: &str) -> bool {
    !run.is_empty()
        && detail.split_whitespace().any(|word| match word.split_once('=') {
            Some(("run", found)) => found == run,
            Some(("with", found)) => found.split(',').any(|one| one == run),
            _ => false,
        })
}

/// 差の字の変えた行（hunk の中の `+` と `-` の行・頭の印を含む字）の字ごとの本数。`diff --git` から最初の `@@` までの頭の行
/// （`---` と `+++` を含む）は数えない。
fn changed(text: &str) -> BTreeMap<&str, usize> {
    let mut found = BTreeMap::new();
    let mut inside = false;
    for line in text.lines() {
        if line.starts_with("diff --git ") {
            inside = false;
        } else if line.starts_with("@@") {
            inside = true;
        } else if inside && (line.starts_with('+') || line.starts_with('-')) {
            *found.entry(line).or_insert(0) += 1;
        }
    }
    found
}

/// 着地の尾（`diff=<n> patch=<m> off=<k>`・読めない側の数と off は `-`）。off は字ごとの本数の差の絶対値の和。
fn counts(landing: Option<&str>, patch: Option<&str>) -> String {
    let (mine, theirs) = (landing.map(changed), patch.map(changed));
    let total = |map: &Option<BTreeMap<&str, usize>>| map.as_ref().map_or_else(|| "-".to_owned(), |found| found.values().sum::<usize>().to_string());
    let off = match (&mine, &theirs) {
        (Some(mine), Some(theirs)) => {
            let lines: BTreeSet<&&str> = mine.keys().chain(theirs.keys()).collect();
            let count = |map: &BTreeMap<&str, usize>, line: &str| map.get(line).copied().unwrap_or(0);
            lines.into_iter().map(|line| count(mine, line).abs_diff(count(theirs, line))).sum::<usize>().to_string()
        }
        _ => "-".to_owned(),
    };
    format!("diff={} patch={} off={off}", total(&mine), total(&theirs))
}

#[cfg(test)]
mod tests {
    use super::{counts, names, note, Followed, Mark};
    use crate::fleet::store::{self, LockPolicy};
    use crate::fleet::{Case, EventKind};
    use crate::pipe::fixture::{event, scratch};

    /// 語の全部（宣言順）。
    const FOLLOWED: [Followed; 5] = [Followed::Conflict, Followed::Stale, Followed::RegateFail, Followed::TrainRed, Followed::Landed];

    /// 組の行の detail（候補 `s2-b-2`・相手 `s2-a-1` と `s2-c-3`）。
    const PAIR: &str = "run=s2-b-2 with=s2-a-1,s2-c-3 files=crates/a.rs main=abc";

    /// 語の列は宣言順に 5 つで、組の detail は `run=` の値と `with=` の 1 つだけを名指し、接頭辞の重なり・ほかの鍵の値・空の字は
    /// 名指さない。
    #[test]
    fn vcledger_words_and_the_names_of_a_pair() {
        let words: Vec<&str> = FOLLOWED.iter().map(|found| found.as_str()).collect();
        assert_eq!(words, ["conflict", "stale", "regate-fail", "train-red", "landed"]);
        for run in ["s2-b-2", "s2-a-1", "s2-c-3"] {
            assert!(names(PAIR, run), "名指す: {run}");
        }
        for run in ["s2-b", "s2-a-12", "s2-c", "crates/a.rs", "abc", ""] {
            assert!(!names(PAIR, run), "名指さない: {run}");
        }
        assert!(!names("run= with=", ""), "空の値も名指さない");
    }

    /// 変えた行は hunk の中の `+` と `-` の行だけを字ごとに数え（文脈の行・hunk の頭・`---` と `+++` の頭は数えない・hunk の中の
    /// 字 `-- x` を消した行 `--- x` は数える）、off は片方にだけ在る変えた行の本数。差の file を読めない周は patch と off が `-`。
    #[test]
    fn vcledger_counts_changed_lines_inside_hunks() {
        let head = "diff --git a/f b/f\nindex 1..2 100644\n--- a/f\n+++ b/f\n";
        let landing = format!("{head}@@ -1,3 +1,3 @@\n c1\n-l2\n+x2\n c3\n@@ -9,2 +9,2 @@\n--- x\n+y\n c10\n");
        let patch = format!("{head}@@ -2,1 +2,1 @@\n-l2\n+x2\n@@ -8,3 +8,3 @@\n c8\n--- x\n+y\n");
        assert_eq!(counts(Some(&landing), Some(&patch)), "diff=4 patch=4 off=0", "同じ変えた行は文脈と頭が違っても off 0");
        let extra = format!("{landing}diff --git a/g b/g\n--- a/g\n+++ b/g\n@@ -1 +1,2 @@\n g1\n+z\n");
        assert_eq!(counts(Some(&extra), Some(&patch)), "diff=5 patch=4 off=1", "着地にだけ在る 1 行");
        let twice = format!("{patch}@@ -20,1 +20,1 @@\n-l2\n+x2\n@@ -30,1 +30,1 @@\n-l2\n+x2\n");
        assert_eq!(counts(Some(&landing), Some(&twice)), "diff=4 patch=8 off=4", "同じ字の本数の差を数える（字の種類の数でない）");
        assert_eq!(counts(Some(&landing), None), "diff=4 patch=- off=-", "差の file を読めない");
        assert_eq!(counts(None, Some(&patch)), "diff=- patch=4 off=-", "着地の差を読めない");
    }

    /// 記帳は、組の行が名指す便だけに kind `OverlapFollowed` の 1 件（bead は便の契約・detail は便 id と語と尾・本体は
    /// `Followed`）を書き、読み直せる。名指さない便は何も書かない。
    #[test]
    fn vcledger_note_writes_only_for_a_named_run() {
        let dir = scratch("vcledger-note");
        let policy = LockPolicy::embedded().unwrap_or_else(|err| panic!("{err:?}"));
        let mut pair = event("", EventKind::OverlapCommuted, None, None, Some(PAIR));
        pair.case = Some(Case::Commuted);
        store::append(&dir, &pair, policy).unwrap_or_else(|err| panic!("{err:?}"));
        let mark = |run: &'static str| Mark { state_dir: &dir, run, bead: "s2-x", policy };
        note(&mark("s2-a-1"), Followed::Conflict, "");
        note(&mark("s2-b-2"), Followed::TrainRed, "train=2");
        note(&mark("s2-b"), Followed::Stale, "");
        let events = store::read_all(&dir).unwrap_or_else(|err| panic!("{err:?}"));
        let followed: Vec<(String, Option<String>, Option<Case>)> = events
            .into_iter()
            .filter(|found| found.kind == EventKind::OverlapFollowed)
            .map(|found| (found.bead, found.detail, found.case))
            .collect();
        let row = |detail: &str| ("s2-x".to_owned(), Some(detail.to_owned()), Some(Case::Followed));
        assert_eq!(followed, [row("run=s2-a-1 word=conflict"), row("run=s2-b-2 word=train-red train=2")]);
    }
}
