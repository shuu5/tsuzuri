//! 台帳の契約の bead の行を、置き場の全部の行を読む 3 つの口（便の runner の節 ほかの行の touches・pipe preflight の閉包の広がり・審査の
//! 材料 index.txt と口 pipe index show）へ運ぶ読み（tsuzuri の判断の記録 ADR-72 の決定 (7)・契約表の行 v-bead-reads-b）。
//!
//! 台帳の読みは [`read_ledger`]（待ちの上限は rules 行 `seat.ledger_timeout_s`）・bead の形の判じと行の読みは [`crate::pipe::bead`] の
//! 1 本で、ここは**表の行の列に足す形**（[`RowFacts`]）にするだけである。置き場に写しの dir が無い置き場は台帳を読まない（[`Beads::Off`]）。

use super::RowFacts;
use crate::pipe::bead::{digest_of_design, form_of, row_of, Form, COPY_DIR};
use crate::pipe::table::parse_pointer;
use crate::rules::manifest::Manifest;
use crate::seat::ledger::{read_ledger, timeout_of, Issue, LedgerError, DEFAULT_BD, ID_TIMEOUT};
use std::path::Path;
use std::time::Duration;

/// 台帳を読む材料（client の名と待ちの上限）。
#[derive(Debug, Clone, Copy)]
pub struct LedgerRead<'a> {
    /// 台帳 client の名（`--bd` か [`DEFAULT_BD`]）。
    pub bd: &'a str,
    /// 待ちの上限（rules 行 `seat.ledger_timeout_s`・読めない周は `None`）。
    pub timeout: Option<Duration>,
}

impl<'a> LedgerRead<'a> {
    /// `--bd` の値（無ければ [`DEFAULT_BD`]）と manifest の待ちの上限から組む。
    pub fn of(bd: Option<&'a str>, manifest: &Manifest) -> Self {
        Self { bd: bd.unwrap_or(DEFAULT_BD), timeout: timeout_of(manifest) }
    }
}

/// 台帳の読みの結果（閉じた 3 値）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Beads {
    /// 置き場に子 [`COPY_DIR`] の dir が無く、台帳を読まない（bead の便を 1 度も受け付けていない置き場）。
    Off,
    /// 契約の bead の行（bead の id の順）。
    Rows(Vec<RowFacts>),
    /// 台帳を読めない（理由の字）。
    Unread(String),
}

/// 台帳の Issue の列から契約の bead の行を組む。形が bead か両方で行を読める bead だけを、鍵を bead の id・欄 touches と write-set を行の
/// 欄にして bead の id の順に返す（閉じた bead も入れる＝表の行が着地した行も持つのと揃える）。
pub fn bead_rows(issues: &[Issue]) -> Vec<RowFacts> {
    let mut rows: Vec<RowFacts> = issues
        .iter()
        .filter(|issue| matches!(form_of(&issue.acceptance), Form::Bead | Form::Both))
        .filter_map(|issue| {
            let row = row_of(&issue.id, &issue.acceptance, &issue.description).ok()?;
            Some((issue.id.clone(), row.touches, row.write_set))
        })
        .collect();
    rows.sort_by(|left, right| left.0.cmp(&right.0));
    rows
}

/// 置き場の契約の bead の行を台帳から読む。写しの dir が無い置き場は台帳を読まず [`Beads::Off`]。
pub fn ledger_rows(state_dir: &Path, repo: &Path, read: LedgerRead<'_>) -> Beads {
    if !state_dir.join(COPY_DIR).is_dir() {
        return Beads::Off;
    }
    let Some(timeout) = read.timeout else {
        return Beads::Unread(format!("rules 行 {ID_TIMEOUT} が無い"));
    };
    match read_ledger(read.bd, repo, timeout) {
        Ok(issues) => Beads::Rows(bead_rows(&issues)),
        Err(LedgerError::Unreadable) => Beads::Unread("台帳を読めない".to_owned()),
        Err(LedgerError::Timeout) => Beads::Unread("台帳の待ちの上限を越えた".to_owned()),
    }
}

/// 契約の design が写しの path の形の周の、写しの dir の bead の id（写しの親の dir の名）。ほかは `None`。
pub fn own_bead(design: &str) -> Option<String> {
    digest_of_design(design)?;
    let pointer = parse_pointer(design).ok()?;
    Path::new(&pointer.path).parent()?.file_name()?.to_str().map(str::to_owned)
}

/// 表の行の列に bead の行を足す（自分の bead の行は除く）。戻りは行の列と、台帳を読めない周の理由（読めた周と台帳を読まない周は `None`）。
pub fn merged(rows: Vec<RowFacts>, beads: &Beads, design: &str) -> (Vec<RowFacts>, Option<String>) {
    match beads {
        Beads::Off => (rows, None),
        Beads::Unread(reason) => (rows, Some(reason.clone())),
        Beads::Rows(found) => {
            let own = own_bead(design);
            let mut all = rows;
            all.extend(found.iter().filter(|(key, ..)| own.as_deref() != Some(key.as_str())).cloned());
            (all, None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{bead_rows, ledger_rows, merged, Beads, LedgerRead};
    use crate::pipe::fixture::scratch;
    use crate::seat::ledger::issues_of;
    use std::time::Duration;

    /// bead の形の acceptance の見本（行 `id`・touches は 1 つ・write-set は 1 つ）。
    fn row_of(id: &str) -> String {
        format!(
            "[[contract]]\nid = \"{id}\"\ntitle = \"行 {id}\"\nreq = [\"FR1\"]\ntouches = [\"crate::x::T\"]\nwrite-set = [\"src/a.rs\"]\nverify = [\"cargo nextest run -p toy --no-tests=fail derive_\"]\nsize = \"S\"\ndone = \"{id} が通る\"\n"
        )
    }

    /// 台帳の JSON の字に escape する（改行と二重引用符）。
    fn escaped(text: &str) -> String {
        text.replace('"', "\\\"").replace('\n', "\\n")
    }

    /// 台帳の 1 件の JSON の字。
    fn issue_json(id: &str, status: &str, acceptance: &str) -> String {
        format!("{{\"id\":\"{id}\",\"status\":\"{status}\",\"acceptance_criteria\":\"{}\",\"description\":\"本文。\"}}", escaped(acceptance))
    }

    /// 行の 1 本（鍵・touches・write-set）。
    fn facts(key: &str) -> (String, Vec<String>, Vec<String>) {
        (key.to_owned(), vec!["crate::x::T".to_owned()], vec!["src/a.rs".to_owned()])
    }

    /// 4 つの bead（契約の行を読める開いた s2-c・design の行だけの s2-e・契約の行を 2 つ持つ s2-f・契約の行を読める閉じた s2-g）から、
    /// 契約の行を読める s2-c と s2-g だけが bead の id の順に残り、s2-c の touches と write-set は acceptance の欄の値になる。
    #[test]
    fn vbrb_bead_rows_keep_only_readable_contract_beads() {
        let two = format!("{}{}", row_of("f1"), row_of("f2"));
        let text = format!(
            "[{}]",
            [
                issue_json("s2-g", "closed", &row_of("g")),
                issue_json("s2-c", "open", &row_of("c")),
                issue_json("s2-e", "open", "design = docs/design/other.md#b1\n"),
                issue_json("s2-f", "open", &two),
            ]
            .join(",")
        );
        let issues = issues_of(&text).unwrap_or_default();
        assert_eq!(issues.len(), 4, "4 本");
        let rows = bead_rows(&issues);
        let keys: Vec<&str> = rows.iter().map(|(key, ..)| key.as_str()).collect();
        assert_eq!(keys, ["s2-c", "s2-g"], "契約の行を読める bead だけ・bead の id の順");
        assert_eq!(rows.first(), Some(&facts("s2-c")), "acceptance の欄の値");
    }

    /// 子 bead-contracts を持たない置き場は在らない client と待ちの上限 1 秒でも Off・持つ置き場は在らない client で理由が `台帳を読めない` の
    /// Unread・待ちの上限が無い周は理由が `seat.ledger_timeout_s` を持つ Unread。
    #[test]
    fn vbrb_ledger_rows_read_only_when_copies_exist() {
        let (bare, held) = (scratch("vbrb-bare"), scratch("vbrb-held"));
        let _ = std::fs::create_dir_all(held.join("bead-contracts"));
        let gone = |timeout| LedgerRead { bd: "bd-gone", timeout };
        let wait = Some(Duration::from_secs(1));
        assert_eq!(ledger_rows(&bare, &bare, gone(wait)), Beads::Off, "写しの dir が無い置き場は台帳を読まない");
        assert_eq!(ledger_rows(&held, &held, gone(wait)), Beads::Unread("台帳を読めない".to_owned()), "在らない client");
        let unset = ledger_rows(&held, &held, gone(None));
        assert!(matches!(&unset, Beads::Unread(reason) if reason.contains("seat.ledger_timeout_s")), "待ちの上限が無い: {unset:?}");
    }

    /// 自分の design が置き場の写し（bead s2-b）を指す周は Rows の s2-b を除いて s2-c だけを表の行の後に足し印は `None`・Unread は行の列のままで
    /// 理由を印に返す・Off は行の列のままで印は `None`。
    #[test]
    fn vbrb_merged_drops_own_bead_and_keeps_table_rows() {
        let table = || vec![facts("d#a"), facts("d#b")];
        let keys = |rows: &[(String, Vec<String>, Vec<String>)]| rows.iter().map(|(key, ..)| key.clone()).collect::<Vec<_>>();
        let own = "/s/bead-contracts/s2-b/0123456789abcdef.toml#b";
        let rows = Beads::Rows(vec![facts("s2-c"), facts("s2-b")]);
        let (found, mark) = merged(table(), &rows, own);
        assert_eq!((keys(&found), mark), (["d#a", "d#b", "s2-c"].map(str::to_owned).to_vec(), None), "自分の bead を除く");
        let (found, mark) = merged(table(), &rows, "docs/design/toy.md#a");
        assert_eq!((keys(&found), mark), (["d#a", "d#b", "s2-c", "s2-b"].map(str::to_owned).to_vec(), None), "写しを指さない周は除かない");
        let (found, mark) = merged(table(), &Beads::Unread("x".to_owned()), own);
        assert_eq!((keys(&found), mark), (["d#a", "d#b"].map(str::to_owned).to_vec(), Some("x".to_owned())), "Unread は理由を印に返す");
        assert_eq!(merged(table(), &Beads::Off, own), (table(), None), "Off は表の行のまま");
    }
}
