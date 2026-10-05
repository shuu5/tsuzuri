//! 便の同時の数の上限（vessel 宣言の任意 key `run-cap` と `run-cap-paths`・tsuzuri の判断の記録 ADR-63 の決定 (13)）。
//!
//! write-set の項目のどれかが `run-cap-paths` の dir の下に在る便を数える便とし、数える契約は、数える live な便と同じ周に
//! 起こした数える便が `run-cap` 本に達した周に起こさない（受付の断りと列の待ちが同じ判じ〔[`RunCap::counts`] と
//! [`RunCap::holder`] を撃つ受付の `capped`〕を読む・C2）。
//! 数える path の字は器の code に書かず、宣言が持つ。2 key は揃えて書き、片方だけの宣言は不備。読むのは HEAD の宣言だけ。

use super::{head_declaration, int_of, list_of, DeclError, Raw};
use crate::pipe::refuse::covered;
use std::path::Path;

/// 上限の本数の key（1 以上の整数）。
pub(super) const CAP_KEY: &str = "run-cap";

/// 数える path の key（repo 相対の dir・末尾 `/`・1 項目以上）。空の配列と空字の項目は、値の読みが呼ぶ配列の層
/// `rules::manifest::list` が key の名と行で断り、その値は [`declared_of`] に渡らない。
pub(super) const PATHS_KEY: &str = "run-cap-paths";

/// 宣言の上限（2 key を揃えた値）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunCap {
    /// 同時に起こせる数える便の本数（1 以上）。
    pub cap: u64,
    /// 数える dir の列（書いた順）。
    pub paths: Vec<String>,
}

impl RunCap {
    /// write-set の項目のどれかが数える dir の下に在るか（項目の頭の印を剥がして畳む規則は交差と同じ [`covered`]）。
    pub fn counts(&self, write_set: &[String]) -> bool {
        write_set.iter().any(|item| covered(&self.paths, item))
    }

    /// 数える契約の前に、ほかの便 `others`（名と write-set・同じ周に起こした便と live な便）のうち数える便が上限に達していれば
    /// 先頭の 1 本の名を返す（上限の内は `None`・契約を数えるかは呼び手が [`Self::counts`] で先に見る）。
    pub fn holder<'a>(&self, others: &[(&'a str, &[String])]) -> Option<&'a str> {
        let held: Vec<&'a str> = others.iter().filter(|(_, set)| self.counts(set)).map(|&(name, _)| name).collect();
        held.first().copied().filter(|_| u64::try_from(held.len()).unwrap_or(u64::MAX) >= self.cap)
    }

    /// HEAD の宣言の上限（宣言 file が無い周と 2 key の無い周は `Ok(None)`・在って読めない周は `Err`）。
    pub fn at_head(repo: &Path) -> Result<Option<Self>, Vec<DeclError>> {
        head_declaration(repo).transpose().map(|found| found.and_then(|declared| declared.run_cap))
    }
}

/// 宣言の本文から 2 key を読む（無ければ `None`・片方だけ・0・項目の不備は key と行番号を名指して積む）。
pub(super) fn declared_of(found: &[(String, Raw, u64)], errors: &mut Vec<DeclError>) -> Option<RunCap> {
    let line_of = |key: &str| found.iter().find(|(seen, _, _)| seen == key).map(|&(_, _, line)| line);
    let before = errors.len();
    let cap = int_of(found, CAP_KEY, errors);
    let (paths, paths_at) = list_of(found, PATHS_KEY, errors);
    match (line_of(CAP_KEY), line_of(PATHS_KEY)) {
        (None, None) => return None,
        (Some(line), None) => errors.push(DeclError::new(line, format!("{PATHS_KEY} が無い（{CAP_KEY} と揃える）"))),
        (None, Some(line)) => errors.push(DeclError::new(line, format!("{CAP_KEY} が無い（{PATHS_KEY} と揃える）"))),
        (Some(line), Some(_)) if cap == Some(0) => errors.push(DeclError::new(line, format!("{CAP_KEY} は 1 以上である"))),
        (Some(_), Some(_)) => {}
    }
    for item in &paths {
        if let Some(why) = item_defect(item) {
            errors.push(DeclError::new(paths_at, format!("{PATHS_KEY} の項目 {item:?} は{why}")));
        }
    }
    (errors.len() == before).then_some(RunCap { cap: cap?, paths })
}

/// 項目 1 つの不備（無ければ `None`）。数えの畳み（[`covered`]）が何にも当てない字（前後の空白・名の段の無い `./`）も不備にし、
/// 宣言したのに何も数えない上限を作らない。
fn item_defect(item: &str) -> Option<&'static str> {
    if item.trim().is_empty() {
        Some("空である")
    } else if item.trim() != item {
        Some("前後に空白を持つ")
    } else if !item.ends_with('/') {
        Some("末尾が / でない")
    } else if item.starts_with('/') {
        Some("絶対 path である")
    } else if item.split('/').any(|part| part == "..") {
        Some(".. の段を持つ")
    } else if item.split('/').all(|part| part.is_empty() || part == ".") {
        Some("名の段を持たない")
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::RunCap;
    use crate::pipe::declaration::Declared;

    /// 必須 key だけの宣言（3 行）の後ろに `extra` を足す。
    fn with(extra: &str) -> String {
        format!("schema = 1\nallowed-commands = [\"git\"]\ncommon-verify = [\"git diff --quiet\"]\n{extra}")
    }

    /// 宣言だけの 1 commit を持つ使い捨ての git repo。
    fn repo_declaring(name: &str, declaration: &str) -> std::path::PathBuf {
        let repo = crate::pipe::fixture::scratch(name);
        let setup: [&[&str]; 4] =
            [&["init", "-q", "-b", "main"], &["config", "user.name", "t"], &["config", "user.email", "t@example.invalid"], &["config", "commit.gpgsign", "false"]];
        for args in setup {
            assert!(crate::pipe::git_ok(&repo, args), "{args:?}");
        }
        assert!(std::fs::write(repo.join(crate::pipe::declaration::DECL_FILE), declaration).is_ok(), "宣言を書けた");
        assert!(crate::pipe::git_ok(&repo, &["add", "-A"]), "add");
        assert!(crate::pipe::git_ok(&repo, &["commit", "-q", "-m", "seed"]), "commit");
        repo
    }

    /// 2 key の宣言は値を持ち、2 key の無い宣言は `None`。
    #[test]
    fn vrcap_declaration_reads_the_two_keys_or_nothing() {
        let two = Declared::parse(&with("run-cap = 1\nrun-cap-paths = [\"vessel/\", \"nest/x/\"]\n")).map(|found| found.run_cap);
        assert_eq!(two, Ok(Some(RunCap { cap: 1, paths: vec!["vessel/".to_owned(), "nest/x/".to_owned()] })));
        let three = Declared::parse(&with("run-cap = 3\nrun-cap-paths = [\"vessel/\"]\n")).map(|found| found.run_cap.map(|cap| cap.cap));
        assert_eq!(three, Ok(Some(3)), "本数は宣言の値");
        assert_eq!(Declared::parse(&with("")).map(|found| found.run_cap), Ok(None));
    }

    /// 正しい宣言から 1 句だけ外した 12 形は、key の名と書いた行（4 行目か 5 行目）を名指す不備。
    #[test]
    fn vrcap_declaration_names_twelve_defects() {
        let bad = [
            ("run-cap = 1\n", 4, "run-cap-paths が無い"),
            ("run-cap-paths = [\"vessel/\"]\n", 4, "run-cap が無い"),
            ("run-cap = 0\nrun-cap-paths = [\"vessel/\"]\n", 4, "run-cap は 1 以上"),
            ("run-cap = \"1\"\nrun-cap-paths = [\"vessel/\"]\n", 4, "run-cap は整数"),
            ("run-cap = 1\nrun-cap-paths = \"vessel/\"\n", 5, "run-cap-paths は配列"),
            ("run-cap = 1\nrun-cap-paths = []\n", 5, "run-cap-paths の 配列が空"),
            ("run-cap = 1\nrun-cap-paths = [\" \"]\n", 5, "run-cap-paths の項目 \" \" は空である"),
            ("run-cap = 1\nrun-cap-paths = [\"vessel\"]\n", 5, "run-cap-paths の項目 \"vessel\" は末尾が / でない"),
            ("run-cap = 1\nrun-cap-paths = [\"/vessel/\"]\n", 5, "run-cap-paths の項目 \"/vessel/\" は絶対 path"),
            ("run-cap = 1\nrun-cap-paths = [\"a/../vessel/\"]\n", 5, "run-cap-paths の項目 \"a/../vessel/\" は.. の段"),
            ("run-cap = 1\nrun-cap-paths = [\" vessel/\"]\n", 5, "run-cap-paths の項目 \" vessel/\" は前後に空白を持つ"),
            ("run-cap = 1\nrun-cap-paths = [\"./\"]\n", 5, "run-cap-paths の項目 \"./\" は名の段を持たない"),
        ];
        for (extra, line, want) in bad {
            let errors = Declared::parse(&with(extra)).expect_err(extra);
            assert!(errors.iter().any(|error| error.line == line && error.reason.contains(want)), "{extra}: {errors:?}");
        }
    }

    /// 数える便: 印（`+`・`-`・`~`・`=`）を剥がした項目が数える dir の下か dir そのもの（末尾 / の無い字も・交差と同じ畳み）。
    /// 段の境目で切り、ほかの dir の下は数えない。
    #[test]
    fn vrcap_counts_items_under_the_declared_dirs() {
        let cap = RunCap { cap: 1, paths: vec!["vessel/".to_owned()] };
        let owned = |items: &[&str]| items.iter().map(|item| (*item).to_owned()).collect::<Vec<String>>();
        for item in ["vessel/a.rs", "+vessel/new.rs", "-vessel/b.rs", "~vessel/c.rs", "=vessel/d.rs", "vessel/deep/", "vessel/", "vessel"] {
            assert!(cap.counts(&owned(&["docs/x.md", item])), "{item}");
        }
        for item in ["vesselx/a.rs", "other/vessel/a.rs", "vesse"] {
            assert!(!cap.counts(&owned(&[item])), "{item}");
        }
    }

    /// 上限の判じ: ほかの数える便が上限の本数に達した周に先頭の 1 本を名指し、数えない便は数えない。
    #[test]
    fn vrcap_holder_names_the_first_counted_run_at_the_cap() {
        let owned = |items: &[&str]| items.iter().map(|item| (*item).to_owned()).collect::<Vec<String>>();
        let (mine, theirs, plain) = (owned(&["vessel/a.rs"]), owned(&["+vessel/b.rs"]), owned(&["docs/x.md"]));
        let one = RunCap { cap: 1, paths: vec!["vessel/".to_owned()] };
        let two = RunCap { cap: 2, ..one.clone() };
        assert_eq!(one.holder(&[("r-plain", &plain), ("r-1", &theirs)]), Some("r-1"));
        assert_eq!(one.holder(&[("r-plain", &plain)]), None, "数えない便は数えない");
        assert_eq!(two.holder(&[("r-1", &theirs)]), None, "上限の内");
        assert_eq!(two.holder(&[("r-1", &theirs), ("s2-b", &mine)]), Some("r-1"), "上限に達すると先頭");
    }

    /// HEAD の口: 2 key を持つ宣言は値・持たない宣言は `None`・壊した宣言は `Err`。
    #[test]
    fn vrcap_head_reads_the_declaration_at_head() {
        let declared = repo_declaring("vrcap-declared", &with("run-cap = 1\nrun-cap-paths = [\"vessel/\"]\n"));
        assert_eq!(RunCap::at_head(&declared), Ok(Some(RunCap { cap: 1, paths: vec!["vessel/".to_owned()] })));
        assert_eq!(RunCap::at_head(&repo_declaring("vrcap-plain", &with(""))), Ok(None));
        assert!(RunCap::at_head(&repo_declaring("vrcap-broken", &with("run-cap = 1\n"))).is_err(), "片方だけは読めない");
    }
}
