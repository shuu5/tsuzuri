//! 入口の排他を差の当たりで通す判じ（判断の記録 ADR-60 の決定 (1)(2)(4)(5)・要件 FR1039）。
//!
//! 交差の照らし（純な交わりの関数 [`super::refuse::overlaps`]）が交差を見つけた組に、入口と起動の列がその後に呼ぶ 1 本の
//! 関数 [`judge`] と、結末の閉じた列 [`Verdict`] と、入り切りの規則の行 [`ROW`] の読み [`on`] を持つ。判じは交わりの関数の
//! 中に置かない（決定 (3)）。同じ宣言の名を足す組の拾い（[`Verdict::SameName`] を返す段）と、入口と起動の列への配線と記帳は
//! 後の器の行が足す。
//!
//! 一時の index と物（object）は state dir の下の [`SCRATCH`] に周ごとの dir を切って置き、周の終わりに dir ごと消す（repo の
//! 物の置き場と index には書かない・決定 (2)）。読めない・当たらない・時間切れの周は断る側に倒す（fail-closed）。

use super::declaration::RootsAtHead;
use super::git_line;
use super::land::scope_touched;
use super::refuse::normalize;
use crate::invocation::Invocation;
use crate::rules::manifest::Manifest;
use crate::rules::RuleValue;
use std::collections::BTreeSet;
use std::ffi::OsStr;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

/// 入り切りの規則の行 id（判断の記録 ADR-60 の決定 (5)）。
pub const ROW: &str = "pipe.overlap_commute";

/// 周ごとの一時の dir を切る、state dir の下の dir の名。
pub const SCRATCH: &str = "commute";

/// 子の終わりを見る間隔。
const POLL: Duration = Duration::from_millis(5);

/// 判じの結末（閉じた列・判断の記録 ADR-60 の決定 (4)）。通す 1 つと断る 5 つ。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    /// 通す（交差に数えない）。
    Commutes,
    /// どちらかの契約が差の file を名指さない。
    NoPatch,
    /// 交わる項に dir の項か、再 gate の面の外の path が在る。
    OutsideFace,
    /// 差が交わる path を書かないか、4 通りか積みのどれかが当たらない。
    NotCommuting,
    /// 両方の差の足す行が同じ宣言の名を足す（拾いは後の器の行）。
    SameName,
    /// 差の file を読めない・一時の index を作れない・時間切れ。
    Unreadable,
}

/// 結末の全部（宣言順）。
pub const VERDICTS: [Verdict; 6] =
    [Verdict::Commutes, Verdict::NoPatch, Verdict::OutsideFace, Verdict::NotCommuting, Verdict::SameName, Verdict::Unreadable];

impl Verdict {
    /// 記帳と断りの行に書く字。
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Commutes => "commutes",
            Self::NoPatch => "no-patch",
            Self::OutsideFace => "outside-face",
            Self::NotCommuting => "not-commuting",
            Self::SameName => "same-name",
            Self::Unreadable => "unreadable",
        }
    }
}

/// 交差の相手の便 1 本。
pub struct Partner<'a> {
    /// 相手の契約の写しの欄 patch（差の file の repo 相対 path）。
    pub patch: Option<&'a str>,
    /// 相手の便の base（相手の差の file を読む木）。
    pub base: &'a str,
    /// 候補と交わった項の組（[`super::refuse::overlaps`] の（候補の項, 相手の項））。
    pub crossed: &'a [(String, String)],
}

/// 判じの材料。
pub struct Ask<'a> {
    /// repo の根。
    pub repo: &'a Path,
    /// state dir（周の一時の dir を [`SCRATCH`] の下に切る）。
    pub state_dir: &'a Path,
    /// その周の main の先端（候補の差の file を読む木・差を当てる土台）。
    pub main: &'a str,
    /// main の先端の宣言の根（再 gate の面）。
    pub roots: &'a RootsAtHead,
    /// 候補の契約の欄 patch。
    pub patch: Option<&'a str>,
    /// 相手の便（起こした順）。
    pub partners: &'a [Partner<'a>],
    /// 時間切れの時刻（越えた周の子は止めて unreadable）。
    pub deadline: Instant,
}

/// 規則の行が真の周だけ真（行が無い・不発効・偽の周は偽＝読めない周は偽と読む・要件 FR1039）。値の形は manifest の読みが
/// kind `PipeOverlapCommute` の真偽に限る。
pub fn on(manifest: &Manifest) -> bool {
    manifest.get(ROW).is_some_and(|row| row.enabled && row.value == RuleValue::Bool(true))
}

/// 候補と相手の組を判じる（判断の記録 ADR-60 の決定 (1)(2)）。
///
/// 順に、差の file を名指さない契約（no-patch）・dir の項か面の外の path（outside-face・面は追随の再 gate を撃つかを決める
/// [`scope_touched`] の 1 本で 1 項ずつ照らす）・差の file を読めない周（unreadable・候補は main の先端の木、相手は相手の base の
/// 木から読む）・交わる path を書かない差（not-commuting）を断り、main の先端を読んだ一時の index に 4 通り（候補だけ・候補の後に
/// 相手・相手だけ・相手の後に候補）を当て、相手が 2 本以上なら相手を起こした順に積んだ後に候補を当てる。当たらない周は
/// not-commuting、index を作れない周と時間切れは unreadable。
pub fn judge(ask: &Ask<'_>) -> Verdict {
    let Some(mine) = ask.patch else {
        return Verdict::NoPatch;
    };
    if ask.partners.iter().any(|partner| partner.patch.is_none()) {
        return Verdict::NoPatch;
    }
    let mut items = ask.partners.iter().flat_map(|partner| partner.crossed.iter());
    if items.any(|(left, right)| !on_face(ask.roots, left) || !on_face(ask.roots, right)) {
        return Verdict::OutsideFace;
    }
    Round::cut(ask).map_or(Verdict::Unreadable, |round| round.judge(mine))
}

/// 項が file で、再 gate の面の中か。
fn on_face(roots: &RootsAtHead, item: &str) -> bool {
    let path = normalize(item);
    !path.ends_with('/') && scope_touched(roots, &[path.as_str()])
}

/// 差の file が書く path（`diff --git a/<元> b/<先>` の両方を字で読む）。
fn written(text: &str) -> BTreeSet<String> {
    text.lines()
        .filter_map(|line| line.strip_prefix("diff --git a/")?.split_once(" b/"))
        .flat_map(|(from, to)| [from.to_owned(), to.to_owned()])
        .collect()
}

/// git の子の終わり方。
#[derive(PartialEq, Eq)]
enum Ran {
    /// rc 0。
    Done,
    /// rc ≠ 0。
    Failed,
    /// 起こせない・待てない・時間切れ（子は止めた）。
    Lost,
}

/// 1 周の一時の dir（index と物と差の写し・drop で dir ごと消す）。
struct Round<'a> {
    /// 判じの材料。
    ask: &'a Ask<'a>,
    /// 周の dir（`<state_dir>/commute/<pid>-<通し>`）。
    dir: PathBuf,
    /// repo の物の置き場（一時の物の置き場の alternates）。
    objects: String,
}

impl Drop for Round<'_> {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

impl<'a> Round<'a> {
    /// 周の dir を切る（repo の物の置き場を引けない周と dir を作れない周は `None`）。
    fn cut(ask: &'a Ask<'a>) -> Option<Self> {
        static SEQ: AtomicU64 = AtomicU64::new(0);
        let objects = git_line(ask.repo, &["rev-parse", "--path-format=absolute", "--git-path", "objects"])?;
        let name = format!("{}-{}", std::process::id(), SEQ.fetch_add(1, Ordering::Relaxed));
        let round = Self { ask, dir: ask.state_dir.join(SCRATCH).join(name), objects };
        std::fs::create_dir_all(round.dir.join("objects")).ok()?;
        Some(round)
    }

    /// 差の file を写し、書く path を照らし、当て方を全部撃つ。
    fn judge(&self, mine: &str) -> Verdict {
        let Some((own, text)) = self.copy(self.ask.main, mine, "c.patch") else {
            return Verdict::Unreadable;
        };
        let own_paths = written(&text);
        let mut theirs: Vec<PathBuf> = Vec::new();
        for (at, partner) in self.ask.partners.iter().enumerate() {
            let Some((path, text)) = partner.patch.and_then(|patch| self.copy(partner.base, patch, &format!("p{at}.patch")))
            else {
                return Verdict::Unreadable;
            };
            let their_paths = written(&text);
            let covered = |(left, right): &(String, String)| {
                own_paths.contains(&normalize(left)) && their_paths.contains(&normalize(right))
            };
            if !partner.crossed.iter().all(covered) {
                return Verdict::NotCommuting;
            }
            theirs.push(path);
        }
        let mut orders: Vec<Vec<&Path>> =
            theirs.iter().flat_map(|path| [vec![own.as_path(), path.as_path()], vec![path.as_path(), own.as_path()]]).collect();
        if theirs.len() > 1 {
            orders.push(theirs.iter().map(PathBuf::as_path).chain([own.as_path()]).collect());
        }
        orders.iter().map(|order| self.applies(order)).find(|found| *found != Verdict::Commutes).unwrap_or(Verdict::Commutes)
    }

    /// `<tree>:<path>` の差の file を周の dir の `name` へ写し、写しの path と本文を返す（読めない周は `None`）。
    fn copy(&self, tree: &str, path: &str, name: &str) -> Option<(PathBuf, String)> {
        let out = self.dir.join(name);
        let spec = format!("{tree}:{path}");
        let ran = self.git(&[OsStr::new("cat-file"), OsStr::new("blob"), OsStr::new(&spec)], Some(&out));
        (ran == Ran::Done).then_some(())?;
        let text = std::fs::read_to_string(&out).ok()?;
        Some((out, text))
    }

    /// main の先端を一時の index に読み、差を順に当てる（1 本ずつ当てるので、列の頭の差だけの当たりも同じ撃ちで測る）。
    fn applies(&self, order: &[&Path]) -> Verdict {
        if self.git(&[OsStr::new("read-tree"), OsStr::new(self.ask.main)], None) != Ran::Done {
            return Verdict::Unreadable;
        }
        for patch in order {
            match self.git(&[OsStr::new("apply"), OsStr::new("--cached"), patch.as_os_str()], None) {
                Ran::Done => {}
                Ran::Failed => return Verdict::NotCommuting,
                Ran::Lost => return Verdict::Unreadable,
            }
        }
        Verdict::Commutes
    }

    /// git を一時の index と物の置き場で撃ち、時間切れまで待つ（stdout は `out` の file か捨てる）。
    fn git(&self, args: &[&OsStr], out: Option<&Path>) -> Ran {
        let stdout = match out.map(File::create) {
            Some(Ok(file)) => Stdio::from(file),
            Some(Err(_)) => return Ran::Lost,
            None => Stdio::null(),
        };
        let mut invocation = Invocation::new("git");
        invocation
            .arg("-C")
            .arg(self.ask.repo)
            .args(args)
            .env("GIT_INDEX_FILE", self.dir.join("index"))
            .env("GIT_OBJECT_DIRECTORY", self.dir.join("objects"))
            .env("GIT_ALTERNATE_OBJECT_DIRECTORIES", &self.objects)
            .stdin(Stdio::null())
            .stdout(stdout)
            .stderr(Stdio::null());
        let Ok(mut child) = invocation.spawn() else {
            return Ran::Lost;
        };
        loop {
            if Instant::now() >= self.ask.deadline {
                let _ = child.kill();
                let _ = child.wait();
                return Ran::Lost;
            }
            match child.try_wait() {
                Ok(Some(status)) => return if status.success() { Ran::Done } else { Ran::Failed },
                Ok(None) => std::thread::sleep(POLL),
                Err(_) => return Ran::Lost,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{judge, on, Ask, Partner, Verdict, ROW, SCRATCH, VERDICTS};
    use crate::pipe::declaration::RootsAtHead;
    use crate::pipe::fixture::scratch;
    use crate::pipe::{git_bytes, git_line, git_ok};
    use crate::rules::manifest::Manifest;
    use crate::rules::{RuleKind, RuleValue};
    use std::path::{Path, PathBuf};
    use std::time::{Duration, Instant};

    /// 面の中の file 2 本（固定の根 `crates/` の下）。
    const FILE: &str = "crates/a/src/lib.rs";
    const OTHER: &str = "crates/a/src/other.rs";
    /// 差の file の dir。
    const DIR: &str = "docs/design/patch";
    /// 固定の根だけの宣言。
    static FIXED: RootsAtHead = RootsAtHead::Fixed;

    /// 差 1 本（名・（file, 行, 替えた字）の列・文脈の行数・文脈の 1 行目を壊すか）。名が `c` で始まる差は main、ほかは base に置く。
    type Spec<'a> = (&'a str, &'a [(&'a str, usize, &'a str)], u8, bool);

    /// 場（repo・state dir・seed・base・main の sha）。seed は 2 file の 30 行（行 n は `l<n>`）。
    struct Place {
        repo: PathBuf,
        state: PathBuf,
        seed: String,
        base: String,
        main: String,
    }

    fn lines(at: usize, text: &str) -> String {
        (1..=30_usize).map(|n| if n == at { format!("{text}\n") } else { format!("l{n}\n") }).collect()
    }

    fn commit(repo: &Path, message: &str) -> String {
        assert!(git_ok(repo, &["add", "-A"]) && git_ok(repo, &["commit", "-q", "--allow-empty", "-m", message]), "{message}");
        git_line(repo, &["rev-parse", "HEAD"]).unwrap_or_default()
    }

    /// seed の後に、相手の差の file を足した base と、相手の差の file を消して候補の差の file を足した main を積む。
    fn place(name: &str, specs: &[Spec<'_>]) -> Place {
        let root = scratch(name);
        let (repo, state) = (root.join("repo"), root.join("state"));
        assert!(std::fs::create_dir_all(repo.join("crates/a/src")).is_ok() && std::fs::create_dir_all(repo.join(DIR)).is_ok());
        let setup: [&[&str]; 4] =
            [&["init", "-q", "-b", "main"], &["config", "user.name", "t"], &["config", "user.email", "t@example.invalid"], &["config", "commit.gpgsign", "false"]];
        assert!(setup.iter().all(|args| git_ok(&repo, args)));
        assert!([FILE, OTHER].iter().all(|file| std::fs::write(repo.join(file), lines(0, "")).is_ok()));
        let seed = commit(&repo, "seed");
        let mut made: Vec<(String, Vec<u8>)> = Vec::new();
        for (patch, edits, context, broken) in specs {
            assert!(edits.iter().all(|(file, at, text)| std::fs::write(repo.join(file), lines(*at, text)).is_ok()));
            let form = ["diff", "--no-color", "--no-ext-diff", "--src-prefix=a/", "--dst-prefix=b/", &format!("-U{context}")];
            let diff = git_bytes(&repo, &form).unwrap_or_default();
            assert!(git_ok(&repo, &["checkout", "--", "crates"]), "{patch}");
            let text = String::from_utf8_lossy(&diff);
            made.push(((*patch).to_owned(), if *broken { text.replacen("\n l", "\n x", 1) } else { text.into_owned() }.into_bytes()));
        }
        let put = |own: bool| made.iter().filter(move |(patch, _)| patch.starts_with('c') == own);
        assert!(put(false).all(|(patch, text)| std::fs::write(repo.join(DIR).join(format!("{patch}.patch")), text).is_ok()));
        let base = commit(&repo, "base");
        assert!(put(false).all(|(patch, _)| std::fs::remove_file(repo.join(DIR).join(format!("{patch}.patch"))).is_ok()));
        assert!(put(true).all(|(patch, text)| std::fs::write(repo.join(DIR).join(format!("{patch}.patch")), text).is_ok()));
        let main = commit(&repo, "main");
        Place { repo, state, seed, base, main }
    }

    fn files(paths: &[&str]) -> Vec<(String, String)> {
        paths.iter().map(|path| ((*path).to_owned(), (*path).to_owned())).collect()
    }

    fn path(patch: &str) -> String {
        format!("{DIR}/{patch}.patch")
    }

    /// 候補 `own` と相手（名の列・base から読む）の判じ。交わる項は `crossed`。
    fn verdict(place: &Place, own: &str, partners: &[&str], crossed: &[(String, String)]) -> Verdict {
        let (own, paths) = (path(own), partners.iter().map(|patch| path(patch)).collect::<Vec<String>>());
        let list: Vec<Partner<'_>> =
            paths.iter().map(|patch| Partner { patch: Some(patch), base: &place.base, crossed }).collect();
        judge(&ask(place, Some(&own), &list))
    }

    fn ask<'a>(place: &'a Place, patch: Option<&'a str>, partners: &'a [Partner<'a>]) -> Ask<'a> {
        let deadline = Instant::now() + Duration::from_secs(60);
        Ask { repo: &place.repo, state_dir: &place.state, main: &place.main, roots: &FIXED, patch, partners, deadline }
    }

    /// 4 通りが全部当たる組は通り、候補だけ・相手だけ・候補の後に相手・相手の後に候補の 1 つだけが当たらない組は断る。
    #[test]
    fn vcapply_four_ways_pass_and_a_break_in_one_way_refuses() {
        let cases: [(&str, Spec<'_>, Spec<'_>, Verdict); 5] = [
            ("pass", ("c", &[(FILE, 5, "c5")], 3, false), ("p", &[(FILE, 20, "p20")], 3, false), Verdict::Commutes),
            ("own", ("c", &[(FILE, 5, "c5")], 3, true), ("p", &[(FILE, 20, "p20")], 3, false), Verdict::NotCommuting),
            ("their", ("c", &[(FILE, 5, "c5")], 3, false), ("p", &[(FILE, 20, "p20")], 3, true), Verdict::NotCommuting),
            ("own-then", ("c", &[(FILE, 10, "c10")], 1, false), ("p", &[(FILE, 12, "p12")], 3, false), Verdict::NotCommuting),
            ("their-then", ("c", &[(FILE, 12, "c12")], 3, false), ("p", &[(FILE, 10, "p10")], 1, false), Verdict::NotCommuting),
        ];
        for (name, own, theirs, want) in cases {
            let place = place(&format!("vcapply-ways-{name}"), &[own, theirs]);
            assert_eq!(verdict(&place, "c", &["p"], &files(&[FILE])), want, "{name}");
        }
    }

    /// 相手 2 本は起こした順に積んだ後に候補を当てる（逆の順だけが当たる組は通らない）。積めない相手 2 本は、候補が各々と 4 通りで
    /// 当たっても断る。
    #[test]
    fn vcapply_two_partners_stack_in_order_before_the_candidate() {
        let own: Spec<'_> = ("c", &[(FILE, 5, "c5")], 3, false);
        let ordered = place("vcapply-stack-order", &[own, ("p1", &[(FILE, 12, "a12")], 3, false), ("p2", &[(FILE, 10, "b10")], 1, false)]);
        assert_eq!(verdict(&ordered, "c", &["p1", "p2"], &files(&[FILE])), Verdict::Commutes, "p1 の後に p2 は当たる");
        assert_eq!(verdict(&ordered, "c", &["p2", "p1"], &files(&[FILE])), Verdict::NotCommuting, "p2 の後に p1 は当たらない");
        let clash = place("vcapply-stack-clash", &[own, ("p1", &[(FILE, 20, "a20")], 3, false), ("p2", &[(FILE, 20, "b20")], 3, false)]);
        for one in ["p1", "p2"] {
            assert_eq!(verdict(&clash, "c", &[one], &files(&[FILE])), Verdict::Commutes, "{one} だけなら通る");
        }
        assert_eq!(verdict(&clash, "c", &["p1", "p2"], &files(&[FILE])), Verdict::NotCommuting, "積めない");
    }

    /// 差の file を名指さない契約は no-patch、dir の項と面の外の path は outside-face、交わる path を書かない差は not-commuting。
    #[test]
    fn vcapply_no_patch_outside_face_and_unwritten_paths_refuse() {
        let place = place("vcapply-gate", &[
            ("c", &[(FILE, 5, "c5")], 3, false),
            ("c2", &[(FILE, 5, "c5"), (OTHER, 5, "c5")], 3, false),
            ("p", &[(FILE, 20, "p20")], 3, false),
            ("p2", &[(FILE, 20, "p20"), (OTHER, 20, "p20")], 3, false),
        ]);
        assert_eq!(verdict(&place, "c2", &["p2"], &files(&[FILE, OTHER])), Verdict::Commutes, "両方が両方を書く");
        let crossed = files(&[FILE]);
        let (own, theirs) = (path("c"), path("p"));
        let partner = |patch: Option<&'static str>| Partner { patch, base: &place.base, crossed: &crossed };
        assert_eq!(judge(&ask(&place, None, &[partner(Some("x"))])), Verdict::NoPatch, "候補が名指さない");
        let list = [Partner { patch: Some(&theirs), ..partner(None) }, partner(None)];
        assert_eq!(judge(&ask(&place, Some(&own), &list)), Verdict::NoPatch, "相手の 1 本が名指さない");
        let outside = [(FILE.to_owned(), FILE.to_owned()), ("docs/a.md".to_owned(), "docs/a.md".to_owned())];
        assert_eq!(verdict(&place, "c", &["p"], &outside), Verdict::OutsideFace, "面の外の path");
        assert_eq!(verdict(&place, "c", &["p"], &[("crates/a/".to_owned(), FILE.to_owned())]), Verdict::OutsideFace, "候補の dir の項");
        assert_eq!(verdict(&place, "c", &["p"], &[(FILE.to_owned(), "crates/a/".to_owned())]), Verdict::OutsideFace, "相手の dir の項");
        assert_eq!(verdict(&place, "c", &["p2"], &files(&[FILE, OTHER])), Verdict::NotCommuting, "候補が OTHER を書かない");
        assert_eq!(verdict(&place, "c2", &["p"], &files(&[FILE, OTHER])), Verdict::NotCommuting, "相手が OTHER を書かない");
    }

    /// 候補は main の先端、相手は相手の base から読み、読めない周と時間切れは unreadable。周の dir は消え、repo の物の置き場と
    /// index には書かない。
    #[test]
    fn vcapply_unreadable_rounds_refuse_and_leave_no_trace() {
        let place = place("vcapply-trace", &[("c", &[(FILE, 5, "c5")], 3, false), ("p", &[(FILE, 20, "p20")], 3, false)]);
        let loose = git_line(&place.repo, &["count-objects"]);
        let (crossed, own, theirs) = (files(&[FILE]), path("c"), path("p"));
        let list = [Partner { patch: Some(&theirs), base: &place.base, crossed: &crossed }];
        assert_eq!(judge(&ask(&place, Some(&own), &list)), Verdict::Commutes, "候補は main・相手は base の木");
        assert_eq!(judge(&Ask { main: &place.base, ..ask(&place, Some(&own), &list) }), Verdict::Unreadable, "base に c は無い");
        let seeded = [Partner { patch: Some(&theirs), base: &place.seed, crossed: &crossed }];
        assert_eq!(judge(&ask(&place, Some(&own), &seeded)), Verdict::Unreadable, "seed に p は無い");
        let late = Ask { deadline: Instant::now(), ..ask(&place, Some(&own), &list) };
        assert_eq!(judge(&late), Verdict::Unreadable, "時間切れ");
        let left = std::fs::read_dir(place.state.join(SCRATCH)).map(Iterator::count).ok();
        assert_eq!(left, Some(0), "周の dir は消えた");
        assert_eq!(git_line(&place.repo, &["count-objects"]), loose, "repo の物の置き場に書かない");
        assert!(git_ok(&place.repo, &["diff", "--cached", "--quiet"]), "repo の index を替えない");
    }

    /// 結末の字は閉じた 6 つ。規則の行は埋め込みでは偽で、真の発効の行だけを真と読む。
    #[test]
    fn vcapply_verdict_tokens_and_the_rule_row_read_true_only_when_set() {
        let tokens: Vec<&str> = VERDICTS.iter().map(|found| found.as_str()).collect();
        assert_eq!(tokens, ["commutes", "no-patch", "outside-face", "not-commuting", "same-name", "unreadable"]);
        let embedded = Manifest::embedded().unwrap_or_else(|errors| panic!("{errors:?}"));
        let row = embedded.get(ROW).map(|found| (found.kind, found.value.clone(), found.enabled, found.ruling.as_str()));
        let want = (RuleKind::PipeOverlapCommute, RuleValue::Bool(false), true, "user 2026-10-05T05:13Z 問い t3-hub.89.2");
        assert_eq!(row, Some(want), "埋め込みの行");
        assert!(!on(&embedded), "埋め込みは偽");
        let one = |value: &str, enabled: &str| {
            let rule = format!("id = \"{ROW}\"\nkind = \"PipeOverlapCommute\"\nvalue = {value}\nenabled = {enabled}\n");
            Manifest::parse(&format!("schema = 1\n\n[[rule]]\n{rule}ruling = \"r\"\nruled_at = \"2026-10-05\"\n"))
        };
        assert!(one("true", "true").is_ok_and(|found| on(&found)), "真の発効の行は真");
        assert!(one("true", "false").is_ok_and(|found| !on(&found)), "不発効は偽");
        assert!(one("false", "true").is_ok_and(|found| !on(&found)), "偽は偽");
        assert!(Manifest::parse("schema = 1\n").is_ok_and(|found| !on(&found)), "行が無い周は偽");
        let refused = one("1", "true").err().map(|errors| format!("{errors:?}")).unwrap_or_default();
        assert!(refused.contains("形と合わない"), "整数の値は断る: {refused}");
        assert_eq!(RuleValue::Bool(true).render(), "true", "rules get の字");
    }
}
