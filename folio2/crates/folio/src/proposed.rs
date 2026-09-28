//! 編集時の口 `folio check --proposed <path>`（便 198・docs/design/delivery-198.md §1・判断の記録 ADR-33 決定 (1)(2)・要件書 FR28）。
//! 置き場の中の 1 file に書こうとしている中身（標準入力）を、床（`floor`・`folio check` と同じ関数）で数える。
//! 置き場を一時 dir へ写し、書く前（写しのまま）と書いた後（その file だけ差し替え）の 2 回を数え、後にだけ在る違反と
//! 「まだ分からない」を返す。後にだけ在る違反のうち、つながりの違反（`Report::links`）は編集を止めない族として分けて返す。
//! 写しは版管理の外に置くので、写しが版管理の外に在ることだけから出る「まだ分からない」（gitcheck の NO_GIT）は書く前にも後にも数えない
//! （書く前の数えが正本を読めずに短絡した周にだけ後に出るため・照合は事後の床だけが数える）。書く先の path が symlink を通れば数えない（まだ分からない）。
//! 写しの床の子の git には一時の作業場所の親を天井に渡し（外の版管理を見ない）、それでも写しの中で版管理の根が解ければ数えない（まだ分からない）。
//! 口は file を書かない（置き場も正本も変えない・一時 dir は終わりに消す）。

use std::collections::HashMap;
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::check::{self, Materials};
use crate::floor_note::EXTERNAL_PATH;
use crate::gitcheck::{self, NO_GIT};
use crate::graph;
use crate::note;
use crate::phase::Flag;
use crate::verdict::{Report, Verdict};

/// 床の判定の 1 本。`folio check` と編集時の口が同じ関数で数える（条 P-15.2・索引は層 2 の check_dir の外で数える）。
pub fn floor(dir: &Path, flag: Flag) -> (Report, Materials) {
    let (mut report, materials) = check::check_dir(dir, flag);
    graph::check_index(dir, &mut report);
    (report, materials)
}

/// 口の答え。`stop` は編集を止める新しい違反、`links` は止めない新しいつながりの違反、`unknowns` は新しい「まだ分からない」、
/// `before_unknowns` は書く前から在った「まだ分からない」の数（口が数えきれない置き場を名乗る・判定には入れない）。
pub struct Judged {
    pub stop: Vec<(String, String)>,
    pub links: Vec<(String, String)>,
    pub unknowns: Vec<String>,
    pub before_unknowns: usize,
}

impl Judged {
    /// 新しい「まだ分からない」が在れば まだ分からない、止める違反が在れば 不合格、どちらも無ければ 合格（つながりは数えない）。
    pub fn verdict(&self) -> Verdict {
        if !self.unknowns.is_empty() {
            Verdict::Unknown
        } else if !self.stop.is_empty() {
            Verdict::Fail
        } else {
            Verdict::Pass
        }
    }

    /// 口の答えの字（床の 合格・不合格 と分ける＝口の 通す は床の 合格 ではない）。
    pub fn word(&self) -> &'static str {
        match self.verdict() {
            Verdict::Pass => "通す",
            Verdict::Fail => "止める",
            Verdict::Unknown => "まだ分からない",
        }
    }
}

/// 写しの一時 dir（終わりに消す）。
struct Scratch(PathBuf);

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

/// 置き場 `dir` の中の `rel` に `content` を書いた後の床を、書く前の床と比べる。Err は判定できない理由の字（口は まだ分からない）。
pub fn judge(dir: &Path, rel: &Path, content: &str) -> Result<Judged, String> {
    inside(rel)?;
    if dir.is_symlink() || !dir.is_dir() {
        return Err(format!("置き場 {} が dir でない（symlink・無い）", dir.display()));
    }
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_nanos());
    let scratch = Scratch(std::env::temp_dir().join(format!("folio-proposed-{}-{nanos}", std::process::id())));
    let name = dir.file_name().map_or_else(|| "design-intent".into(), |n| n.to_os_string());
    let copy = scratch.0.join(name);
    copy_tree(dir, &copy).map_err(|e| format!("置き場を写せない: {e}"))?;
    // 書く先の path の途中か終わりが symlink なら書かない（写しの外の file を書き換えない・dir も作らない）
    let mut at = copy.clone();
    for c in rel.components() {
        at.push(c);
        if at.is_symlink() {
            return Err(format!("{} は symlink を通る（写しの外を書きうる）", rel.display()));
        }
    }
    // 器の導出 file は置き場と同じ式（note::external_path）で解き、写しの親の下の同じ字の所へ写す（版管理の外の写しは親の下を読む）
    if let Ok(external) = note::external_path(dir)
        && external.is_file()
    {
        let to = scratch.0.join(EXTERNAL_PATH);
        fs::create_dir_all(to.parent().unwrap_or(&scratch.0))
            .and_then(|()| fs::copy(&external, &to).map(|_| ()))
            .map_err(|e| format!("器の導出 file を写せない: {e}"))?;
    }
    // 写しの床の子の git は一時の作業場所の親より上の版管理を見ない（置き場の器の導出 file を解いた後に置く）
    gitcheck::ceil_at(scratch.0.parent().unwrap_or(&scratch.0));
    if gitcheck::toplevel(&copy).is_some() {
        return Err("一時の作業場所の中で版管理の根が解ける（写しが版管理の中に在る）".to_string());
    }
    let (before, _) = floor(&copy, Flag::None);
    let target = copy.join(rel);
    if target.is_dir() {
        return Err(format!("{} は dir（書く file でない）", rel.display()));
    }
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("書く先の dir を作れない: {e}"))?;
    }
    fs::write(&target, content).map_err(|e| format!("写しへ書けない: {e}"))?;
    let (after, _) = floor(&copy, Flag::None);
    let shown = |s: &str| s.replace(&copy.display().to_string(), &dir.display().to_string());
    let mut seen: HashMap<&(String, String), usize> = HashMap::new();
    for v in &before.violations {
        *seen.entry(v).or_default() += 1;
    }
    let mut judged = Judged {
        stop: Vec::new(),
        links: Vec::new(),
        unknowns: Vec::new(),
        before_unknowns: 0,
    };
    for (i, v) in after.violations.iter().enumerate() {
        match seen.get_mut(v) {
            Some(n) if *n > 0 => *n -= 1,
            _ => {
                let item = (v.0.clone(), shown(&v.1));
                if after.links.contains(&i) {
                    judged.links.push(item);
                } else {
                    judged.stop.push(item);
                }
            }
        }
    }
    // 写しが版管理の外に在ることだけから出る まだ分からない は、書く前にも後にも数えない
    let counted = |u: &&String| u.as_str() != NO_GIT;
    let mut old: HashMap<&String, usize> = HashMap::new();
    for u in before.unknowns.iter().chain(&before.pendings).filter(counted) {
        *old.entry(u).or_default() += 1;
        judged.before_unknowns += 1;
    }
    for u in after.unknowns.iter().chain(&after.pendings).filter(counted) {
        match old.get_mut(u) {
            Some(n) if *n > 0 => *n -= 1,
            _ => judged.unknowns.push(shown(u)),
        }
    }
    Ok(judged)
}

/// 書く file の字は置き場からの相対で、置き場の外へ出ない（絶対 path・`..`・空は断る）。
fn inside(rel: &Path) -> Result<(), String> {
    let ok = rel.components().count() > 0
        && rel.components().all(|c| matches!(c, Component::Normal(_)));
    if ok {
        Ok(())
    } else {
        Err(format!("{} は置き場からの相対の file の字でない（絶対 path・.. ・空は数えない）", rel.display()))
    }
}

/// dir を丸ごと写す（symlink は symlink のまま写す＝床の symlink の断りを写しでも同じに数える）。
fn copy_tree(src: &Path, dst: &Path) -> std::io::Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let to = dst.join(entry.file_name());
        let ty = entry.file_type()?;
        if ty.is_symlink() {
            std::os::unix::fs::symlink(fs::read_link(entry.path())?, &to)?;
        } else if ty.is_dir() {
            copy_tree(&entry.path(), &to)?;
        } else {
            fs::copy(entry.path(), &to)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 便 198: 置き場の外を指す字は数えない（絶対 path・..・空）。
    #[test]
    fn f198_inside_refuses_paths_out_of_the_place() {
        assert!(inside(Path::new("rules.yaml")).is_ok());
        assert!(inside(Path::new("adr/ADR-1.yaml")).is_ok());
        for bad in ["", "/etc/passwd", "../x.yaml", "adr/../../x.yaml", "./rules.yaml"] {
            assert!(inside(Path::new(bad)).is_err(), "{bad}");
        }
    }

    /// 便 198: つながりの違反は止める族に入らず、書く前から在った違反は新しい違反に数えない。
    #[test]
    fn f198_judged_verdict_counts_stop_and_unknowns_only() {
        let mut j = Judged {
            stop: Vec::new(),
            links: vec![("参照 id".into(), "x".into())],
            unknowns: Vec::new(),
            before_unknowns: 3,
        };
        assert_eq!((j.verdict(), j.word()), (Verdict::Pass, "通す"));
        j.stop.push(("未知の欄".into(), "y".into()));
        assert_eq!((j.verdict(), j.word()), (Verdict::Fail, "止める"));
        j.unknowns.push("z".into());
        assert_eq!((j.verdict(), j.word()), (Verdict::Unknown, "まだ分からない"));
    }
}
