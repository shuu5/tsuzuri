//! 受付の bead の周（契約を台帳の bead に置く形・tsuzuri の判断の記録 ADR-72 の決定 (2)・契約表の行 v-bead-intake）。
//!
//! `--design` も `--contract` も渡されない周は bead の周で、`--bead` の bead の acceptance（`[[contract]]` の 1 行）と本文から契約を組む。
//! 組む順は [`bead_contract`] の 7 段で、最初の断りで返る（judge の前・run dir も event も作らない）。行の本文の出所だけを替え、
//! 判定の 1 本 [`super::judge`] と表の検査と写しの読み（[`crate::pipe::table::read_rows`]）は替えない（C2）。形の判じと写しの字は
//! [`crate::pipe::bead`] が持つ。acceptance が `design = ` の行を持つ bead（並ぶ間の 2 形）は今の [`super::generated`] で組む。

use super::{broken, denied, generated, generated_from, int_row, need, read_args, refuse, refused, repo_flag};
use super::{Denial, Materials, DENIAL_ARGS, DENIAL_RULES, DENIAL_STORE, FLAG_CONTRACT, REPO_FLAG};
use crate::pipe::bead::{copy_pointer, copy_text, form_of, row_of, Form};
use crate::pipe::cli::present;
use crate::pipe::contract::Contract;
use crate::pipe::refuse::Refuse;
use crate::pipe::spawn::table_rows;
use crate::pipe::table::{self, TableError};
use crate::rules::manifest::Manifest;
use crate::seat::ledger::{read_ledger, timeout_of, Issue};
use std::path::{Path, PathBuf};

/// 本文（description）の byte の上限の rules 行。
const ROW_BODY_MAX: &str = "contract.body_max_bytes";

/// 欄 acceptance の byte の上限の rules 行。
const ROW_ACCEPTANCE_MAX: &str = "contract.acceptance_max_bytes";

/// 開いた契約の bead の本数の上限の rules 行。
const ROW_OPEN_MAX: &str = "contract.open_max";

/// 台帳を読めない周の理由の句。
const LEDGER_UNREADABLE: &str = "台帳を読めない";

/// bead が台帳に無い周（id が空・区切りの字・`..` を持つ周も同じ）の理由の句。
const LEDGER_ABSENT: &str = "台帳に無い";

/// acceptance が契約の行も pointer の行も持たない周の理由の句。
const NO_FORM: &str = "acceptance に [[contract]] の行も design の行も無い";

/// 契約の出所（受付の引数の読みの結果）。
pub(in crate::pipe) enum Source {
    /// `--design` の pointer（今の形・base の契約表の行）。
    Design(table::Pointer),
    /// `--bead` の bead の契約（台帳の bead に置いた形）。
    Bead,
}

/// 受付と preflight の引数の読み。`--design` も `--contract` も無い周は bead の周で、`--bead` と `--repo` を [`read_args`] と同じ
/// `need` と `repo_flag` で読む。ほかの周は [`read_args`] をそのまま呼ぶ（`--contract` の断りと `--design` の読みは替えない）。
pub(in crate::pipe) fn source_of(args: &[String]) -> Result<(Source, String, PathBuf), Denial> {
    if present(args, "--design") || present(args, FLAG_CONTRACT) {
        return read_args(args).map(|(pointer, bead, repo)| (Source::Design(pointer), bead, repo));
    }
    let args_denied = |reason: String| denied(DENIAL_ARGS, refused(reason));
    let bead = need(args, "--bead").map_err(args_denied)?.to_owned();
    let repo = repo_flag(args).and_then(|found| found.ok_or(format!("{REPO_FLAG} が要る"))).map_err(args_denied)?;
    Ok((Source::Bead, bead, repo))
}

/// 受付が契約を組む口: `Design` は今の [`generated`]・`Bead` は台帳を 1 回読んで [`bead_contract`]。`place` は repo と置き場、`tools` は
/// 規則と台帳の client（`--bd`）。
pub(in crate::pipe) fn contract_of(
    source: &Source,
    place: (&Path, &Path),
    tools: (&Manifest, &str),
    bead: &str,
    materials: &Materials,
) -> Result<(Contract, String), Denial> {
    match source {
        Source::Design(pointer) => generated(place.0, pointer, materials),
        Source::Bead => {
            let issues = ledger_of(tools.1, place.0, tools.0, bead)?;
            bead_contract(place, tools.0, bead, &issues, materials)
        }
    }
}

/// 台帳の全部の Issue を 1 回読む（`bd list --all` の 1 撃ち）。読めない周（client の失敗・待ち上限の行が無い周）は
/// `contract-bead-unreadable` で断る。
pub(in crate::pipe) fn ledger_of(bd: &str, repo: &Path, manifest: &Manifest, bead: &str) -> Result<Vec<Issue>, Denial> {
    let unreadable = || unreadable_of(bead, LEDGER_UNREADABLE);
    let timeout = timeout_of(manifest).ok_or_else(unreadable)?;
    read_ledger(bd, repo, timeout).map_err(|_| unreadable())
}

/// bead の契約から契約を組む（台帳の全部の Issue `issues` は呼び手が 1 回読んで渡す）。順は次のとおりで、最初の断りで返る。
///
/// （1）bead の id が空か区切りの字（斜線）か `..` を持つか、`issues` に無い。（2）形: 両方は `contract-bead-both-forms`・どちらも無いは
/// `contract-bead-unreadable`・`design = ` の行だけは今の [`generated`]。（3）byte の上限（本文を先に見る）。（4）写しの行を読む。
/// （5）行の id が契約表の行かほかの bead の行と同じ。（6）開いた契約の bead の本数の上限。（7）写しを置き場に書き（在れば書かない）、
/// [`generated_from`] で契約を組む。
pub(in crate::pipe) fn bead_contract(
    (repo, state_dir): (&Path, &Path),
    manifest: &Manifest,
    bead: &str,
    issues: &[Issue],
    materials: &Materials,
) -> Result<(Contract, String), Denial> {
    let named = !bead.is_empty() && !bead.contains('/') && !bead.contains("..");
    let issue = issues.iter().find(|issue| named && issue.id == bead).ok_or_else(|| unreadable_of(bead, LEDGER_ABSENT))?;
    let (acceptance, description) = (issue.acceptance.as_str(), issue.description.as_str());
    match form_of(acceptance) {
        Form::Both => return Err(refuse(&Refuse::ContractBeadBothForms { bead: bead.to_owned() }, &[])),
        Form::Neither => return Err(unreadable_of(bead, NO_FORM)),
        Form::Design(pointer) => return generated(repo, &pointer, materials),
        Form::Bead => {}
    }
    exclude_oversized(manifest, acceptance, description)?;
    let row = row_of(bead, acceptance, description).map_err(|reason| unreadable_of(bead, &reason))?;
    exclude_id_taken(repo, bead, &row.id, issues)?;
    exclude_open_cap(manifest, issues)?;
    let pointer = copy_pointer(state_dir, bead, acceptance, description).map_err(|reason| unreadable_of(bead, &reason))?;
    let text = copy_text(bead, acceptance, description).map_err(|reason| unreadable_of(bead, &reason))?;
    place_copy(Path::new(&pointer.path), &text)?;
    generated_from(repo, &pointer, &text, materials)
}

/// `contract-bead-unreadable` の断り（理由の句 1 つ）。
fn unreadable_of(bead: &str, reason: &str) -> Denial {
    refuse(&Refuse::ContractBeadUnreadable { bead: bead.to_owned(), reason: reason.to_owned() }, &[])
}

/// rules 行が無いか整数でない周の断り（今の rules の断り）。
fn rules_denied(reason: String) -> Denial {
    denied(DENIAL_RULES, refused(reason))
}

/// 本文か acceptance の byte の数が rules 行の値より大きい周の断り（本文を先に見る・値ちょうどは通す）。
fn exclude_oversized(manifest: &Manifest, acceptance: &str, description: &str) -> Result<(), Denial> {
    for (field, text, row) in [("description", description, ROW_BODY_MAX), ("acceptance", acceptance, ROW_ACCEPTANCE_MAX)] {
        let cap = int_row(manifest, row).map_err(rules_denied)?;
        let bytes = u64::try_from(text.len()).unwrap_or(u64::MAX);
        if bytes > cap {
            return Err(refuse(&Refuse::ContractBytesCap { field, bytes, cap }, &[]));
        }
    }
    Ok(())
}

/// 行の id が置き場の全部の行（契約表）か、ほかの bead の契約の行（閉じた物も・bead の id の順）と同じ周の断り（表を先に見る）。
/// 置き場の読めない周は今の `contract-table` で断る。
fn exclude_id_taken(repo: &Path, bead: &str, id: &str, issues: &[Issue]) -> Result<(), Denial> {
    let rows = table_rows(repo, "HEAD").map_err(|reason| refuse(&Refuse::ContractTable(TableError::Unreadable { line: 0, reason }), &[]))?;
    let taken = |by: &str| refuse(&Refuse::ContractIdTaken { id: id.to_owned(), by: by.to_owned() }, &[]);
    if let Some((pointer, ..)) = rows.iter().find(|(pointer, ..)| pointer.rsplit_once('#').is_some_and(|(_, found)| found == id)) {
        return Err(taken(pointer));
    }
    let mut others: Vec<&Issue> = issues.iter().filter(|issue| issue.id != bead).collect();
    others.sort_by(|left, right| left.id.cmp(&right.id));
    let holds = |issue: &&Issue| {
        matches!(form_of(&issue.acceptance), Form::Bead | Form::Both)
            && row_of(&issue.id, &issue.acceptance, &issue.description).is_ok_and(|row| row.id == id)
    };
    others.into_iter().find(holds).map_or(Ok(()), |issue| Err(taken(&issue.id)))
}

/// 閉じていない契約の bead（形が bead か両方・B を含む）の本数が rules 行の値より大きい周の断り。
fn exclude_open_cap(manifest: &Manifest, issues: &[Issue]) -> Result<(), Denial> {
    let cap = int_row(manifest, ROW_OPEN_MAX).map_err(rules_denied)?;
    let open = issues.iter().filter(|issue| issue.status != "closed" && matches!(form_of(&issue.acceptance), Form::Bead | Form::Both));
    let open = u64::try_from(open.count()).unwrap_or(u64::MAX);
    if open > cap {
        return Err(refuse(&Refuse::ContractOpenCap { open, cap }, &[]));
    }
    Ok(())
}

/// 写しを置き場に書く（在れば書かない・dir は作る・名は中身で決まるので別の名の一時 file に書いて置き換える）。書けない周は壊れた置き場（rc 2）。
fn place_copy(path: &Path, text: &str) -> Result<(), Denial> {
    if path.exists() {
        return Ok(());
    }
    let partial = path.with_extension(format!("part{}", std::process::id()));
    let written = path
        .parent()
        .map_or(Ok(()), std::fs::create_dir_all)
        .and_then(|()| std::fs::write(&partial, text))
        .and_then(|()| std::fs::rename(&partial, path));
    written.map_err(|err| denied(DENIAL_STORE, broken(format!("契約の写し {} を書けない: {err}", path.display()))))
}
