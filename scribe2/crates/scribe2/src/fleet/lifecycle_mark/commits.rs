//! main の commit の trailer の読み（設計 docs/design/case-lifecycle.md §5・FR90・FR92・ADR-0088）と追認の札（判断の記録 ADR-45 の
//! 門 H6）。`fleet/lifecycle_mark.rs` の [`read_commits`] の群（bead の引き・commit 1 本の読み）を移し、追認の札の読みを足した。
//!
//! 追認の札は、発端の trailer を持つ commit の本文の行 `<Name>-Adopts: <sha> <bead id>…`（key は [`adopts_key`]・sha は `%H` の
//! 40 字と同じ字）で、同じ範囲の発端の trailer も器の便の trailer も持たない commit に、名指した id を発端として結ぶ。結んだ id は
//! 本文の発端と同じ判じを受け（台帳に無い id は source-unresolved）、照らしから外れる commit は無い。発端の trailer を持たない
//! commit の札・自分の trailer を持つ commit を名指す札・範囲に無い sha の札は何も結ばない（外れは外れのまま出力に残る）。

use crate::ledger::form::pointer_text;
use crate::ledger::phase_main::Commit;
use crate::pipe::git_bytes;
use crate::pipe::land::{adopts_key, contract_key, source_key, RUN_TRAILER};
use crate::seat::ledger::Issue;
use std::path::Path;

/// 追認の札 1 行（名指す sha と、結ぶ id の列）。
type Adoption = (String, Vec<String>);

/// 設計 pointer から bead を引く（開いた bead が先・無ければ閉じた時刻が最も新しい bead）。
fn bead_of<'i>(issues: &'i [Issue], pointer: &str) -> Option<&'i Issue> {
    let mut named: Vec<&Issue> = issues.iter().filter(|issue| pointer_text(&issue.acceptance) == Some(pointer)).collect();
    named.sort_by_key(|issue| (issue.status == "closed", std::cmp::Reverse(issue.closed_at.clone())));
    named.first().copied()
}

/// main の commit（切り替えの線の main の sha の次から先端まで・first-parent・古い順）を trailer つきで読み、追認の札で発端を結ぶ。
pub fn read_commits(repo: &Path, from: &str, to: &str, issues: &[Issue]) -> Option<Vec<Commit>> {
    let range = format!("{from}..{to}");
    let raw = git_bytes(repo, &["log", "--first-parent", "--format=%H%x1f%ct%x1f%B%x1e", &range])?;
    Some(commits_of(&String::from_utf8_lossy(&raw), issues))
}

/// `git log` の字（record ごとに `%H`・`%ct`・`%B` を `\x1f` で区切り `\x1e` で終える・新しい順）から古い順の commit の列を組み、
/// 発端の trailer を持つ commit の追認の札を、発端の trailer も器の便の trailer も持たない commit に当てる（id は古い札から順に足す）。
fn commits_of(log: &str, issues: &[Issue]) -> Vec<Commit> {
    let mut read: Vec<(Commit, Vec<Adoption>)> = log.split('\x1e').filter_map(|record| commit_of(record.trim_start_matches('\n'), issues)).collect();
    read.reverse();
    let adoptions: Vec<Adoption> = read.iter().filter(|(commit, _)| !commit.sources.is_empty()).flat_map(|(_, adopts)| adopts.clone()).collect();
    let mut commits: Vec<Commit> = read.into_iter().map(|(commit, _)| commit).collect();
    for commit in commits.iter_mut().filter(|commit| commit.sources.is_empty() && commit.run.is_none()) {
        commit.sources = adoptions.iter().filter(|(sha, _)| *sha == commit.sha).flat_map(|(_, ids)| ids.iter().cloned()).collect();
    }
    commits
}

/// commit 1 本（`run:`・発端・契約の 3 つの trailer と追認の札を読む・契約の trailer は pointer を bead id へ引き直し、引けない pointer は
/// 渡さない・札は sha の後に id を 1 本以上持つ行だけ）。
fn commit_of(record: &str, issues: &[Issue]) -> Option<(Commit, Vec<Adoption>)> {
    let mut fields = record.splitn(3, '\x1f');
    let sha = fields.next()?.trim().to_owned();
    let at = fields.next()?.trim().parse().ok()?;
    let (source, contract, adopts) = (source_key(), contract_key(), adopts_key());
    let mut found = Commit { sha, at, sources: Vec::new(), run: None, contracts: Vec::new() };
    let mut adoptions = Vec::new();
    for line in fields.next().unwrap_or_default().lines().map(str::trim_end) {
        if let Some(run) = line.strip_prefix(RUN_TRAILER).map(str::trim).filter(|run| !run.is_empty()) {
            found.run.get_or_insert_with(|| run.to_owned());
        } else if let Some(ids) = line.strip_prefix(source.as_str()) {
            found.sources.extend(ids.split_whitespace().map(str::to_owned));
        } else if let Some(pointer) = line.strip_prefix(contract.as_str()) {
            found.contracts.extend(bead_of(issues, pointer.trim()).map(|issue| issue.id.clone()));
        } else if let Some((named, ids)) = line.strip_prefix(adopts.as_str()).and_then(|rest| rest.trim().split_once(char::is_whitespace)) {
            adoptions.push((named.to_owned(), ids.split_whitespace().map(str::to_owned).collect()));
        }
    }
    (!found.sha.is_empty()).then_some((found, adoptions))
}

#[cfg(test)]
mod tests {
    use super::{adopts_key, commits_of, source_key};

    /// 40 字の toy の sha（`n` の 16 進を 0 で埋める）。
    fn sha(n: u32) -> String {
        format!("{n:040x}")
    }

    /// `git log` の 1 record の字（時刻は n）。
    fn record(n: u32, body: &str) -> String {
        format!("{}\x1f{n}\x1f{body}\n\x1e\n", sha(n))
    }

    /// 発端の trailer を持つ commit の札は、発端の trailer も器の便の trailer も持たない commit にだけ、40 字の sha が同じ時に id を
    /// 古い札から順に結ぶ。否定はどれも 1 句だけ外す: 札を持つ commit が発端の trailer を持たない（t2）・名指す commit が自分の発端の
    /// trailer を持つ（t3）・器の便の trailer を持つ（t4）・sha が 12 字の頭だけ（t5）。札を持つ commit 自身は札で結ばれない。
    #[test]
    fn vadopt_binds_the_named_bead_only_to_a_commit_without_trailers() {
        let (source, adopts) = (source_key(), adopts_key());
        assert_eq!(adopts, source.replace("-Source: ", "-Adopts: "), "key は発端の trailer と同じく器の名から導く");
        let adopt = |n: u32, ids: &str| format!("{adopts}{} {ids}", sha(n));
        let short = format!("{adopts}{} toy-d", &sha(5)[..12]);
        let log = [
            record(9, &format!("要旨\n\n{source}s-p3\n{}", adopt(6, "toy-h toy-i"))),
            record(8, &format!("要旨\n\n{}", adopt(2, "toy-f"))),
            record(7, &format!("要旨\n\n{source}s-p1\n{}\n{}\n{}\n{short}\n{}", adopt(1, "toy-a"), adopt(3, "toy-b"), adopt(4, "toy-c"), adopt(6, "toy-e"))),
            record(6, "札の無い commit"),
            record(5, "札の無い commit"),
            record(4, "run: r-4"),
            record(3, &format!("要旨\n\n{source}s-3")),
            record(2, "札の無い commit"),
            record(1, "札の無い commit"),
        ]
        .concat();
        let commits = commits_of(&log, &[]);
        let shas: Vec<String> = commits.iter().map(|commit| commit.sha.clone()).collect();
        assert_eq!(shas, (1..=9).map(sha).collect::<Vec<_>>(), "古い順");
        let sources: Vec<Vec<&str>> = commits.iter().map(|commit| commit.sources.iter().map(String::as_str).collect()).collect();
        let want: [&[&str]; 9] = [&["toy-a"], &[], &["s-3"], &[], &[], &["toy-e", "toy-h", "toy-i"], &["s-p1"], &[], &["s-p3"]];
        assert_eq!(sources, want, "札は発端の trailer を持つ commit から・trailer の無い commit にだけ・40 字の sha で結ぶ");
        assert_eq!(commits.get(3).and_then(|commit| commit.run.as_deref()), Some("r-4"), "器の便の trailer は残る");
    }
}
