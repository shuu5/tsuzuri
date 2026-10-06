//! 開いた契約の読み（bead の契約の形つき・契約表の行 v-bead-ledger）。契約の鍵は [`contract_key`]（design の pointer の字か bead の id）で、
//! 開いた契約ごとの [`OpenContract`] と、開いた bead の契約の write-set の項目を持つ。`lifecycle_mark.rs` と `lifecycle.rs` の余地のため子に置く。

use super::is_open_contract;
use crate::fleet::phase::OpenContract;
use crate::ledger::form::contract_key;
use crate::pipe::bead::row_of;
use crate::seat::ledger::Issue;

/// 開いた契約ごとの bead の id と契約の鍵の pointer（台帳の順・鍵の無い契約は `None`）。
pub fn open_contracts(issues: &[Issue]) -> Vec<OpenContract> {
    issues.iter().filter(|issue| is_open_contract(issue)).map(|issue| OpenContract { bead: issue.id.clone(), pointer: contract_key(issue) }).collect()
}

/// 開いた bead の契約（契約の鍵が自分の id の開いた契約）の行の write-set の項目（台帳の順・行を読めない bead は足さない）。
pub(super) fn bead_write_set(issues: &[Issue]) -> Vec<String> {
    let beaded = |issue: &&Issue| is_open_contract(issue) && contract_key(issue).as_deref() == Some(issue.id.as_str());
    let items = |issue: &Issue| row_of(&issue.id, &issue.acceptance, &issue.description).map(|row| row.write_set).unwrap_or_default();
    issues.iter().filter(beaded).flat_map(items).collect()
}

#[cfg(test)]
mod tests {
    use super::{super::open_write_set, open_contracts};
    use crate::fleet::json_lite::quote;
    use crate::seat::ledger::{issues_of, Issue};

    /// 行の形の acceptance（`write_set` は toml の配列の中身）。
    fn row(write_set: &str) -> String {
        format!("[[contract]]\nid = \"b\"\ntitle = \"t\"\nreq = [\"FR1\"]\nwrite-set = [{write_set}]\nverify = [\"cargo nextest run -p toy --no-tests=fail derive_\"]\nsize = \"S\"\ndone = \"d\"\n")
    }

    /// bead 1 本の JSON。
    fn bead(id: &str, status: &str, label: Option<&str>, acceptance: &str) -> String {
        let labels: Vec<String> = label.map(quote).into_iter().collect();
        format!(
            "{{\"id\":{},\"status\":{},\"issue_type\":\"task\",\"labels\":[{}],\"acceptance_criteria\":{},\"description\":\"本文。\"}}",
            quote(id),
            quote(status),
            labels.join(","),
            quote(acceptance)
        )
    }

    fn ledger() -> Vec<Issue> {
        let items = [
            bead("t1", "open", None, "design = docs/design/x.md#a"),
            bead("b1", "open", None, &row("\"+src/a.rs\", \"src/b.rs\"")),
            bead("b2", "closed", None, &row("\"src/c.rs\"")),
            bead("bad", "open", None, "[[contract]]\nid = a\n"),
            bead("n1", "open", None, ""),
            bead("m1", "open", Some("intake:memo"), &row("\"src/m.rs\"")),
        ];
        issues_of(&format!("[{}]", items.join(","))).unwrap_or_default()
    }

    /// 開いた契約は design の pointer か bead の id を鍵に台帳の順に返り、write-set は契約表の項目の後に開いた bead の契約の項目が続く。
    #[test]
    fn vbled_open_contracts_and_write_set_read_bead_contracts() {
        let issues = ledger();
        let found: Vec<(String, Option<String>)> = open_contracts(&issues).into_iter().map(|open| (open.bead, open.pointer)).collect();
        let want = [("t1", Some("docs/design/x.md#a")), ("b1", Some("b1")), ("bad", Some("bad")), ("n1", None)];
        assert_eq!(found, want.map(|(bead, pointer)| (bead.to_owned(), pointer.map(str::to_owned))));
        let write_sets = [("docs/design/x.md#a".to_owned(), vec!["src/t.rs".to_owned()])];
        assert_eq!(open_write_set(&issues, &write_sets), ["src/t.rs", "+src/a.rs", "src/b.rs"]);
    }
}
