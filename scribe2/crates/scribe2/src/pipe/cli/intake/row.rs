//! 受付の設計 pointer の行の読み・契約の生成・write-set の弁別（`intake.rs` から純移動・判断の記録 ADR-77 の決定 (7) の行 77-1・本文は不変）。
//!
//! 親の型（[`Materials`] / [`Denial`] / [`Judged`] / [`WriteSet`]）の私有の欄は子から見える（子は親の子孫）ので、型は親に残す。

use super::refusal::refuse_of;
use super::{
    denied, exclude_unindexed, exclude_unmeasured, list_row, refuse, refused, unloadable, Denial, Judged, Materials, WriteSet,
    DENIAL_GENERATED,
};
use crate::cli_outcome::{Outcome, RC_BROKEN, RC_REFUSED};
use crate::name::NAME;
use crate::pipe::closure::{self, ClosureError, Source};
use crate::pipe::contract::{Contract, CLASS_ROW};
use crate::pipe::declaration::{Ceiling, Effective, CEILING_ROW, DENIED_ROW};
use crate::pipe::refuse::{Refuse, NEW_FILE};
use crate::pipe::table::{self, ContractRow, TableError};
use crate::pipe::vessel_path;
use crate::rules::manifest::Manifest;
use crate::seat::ledger::DEFAULT_BD;
use std::collections::BTreeSet;
use std::path::Path;

/// base の設計 pointer から契約を組む（契約 (b)・設計 contract-source.md §2「生成」）。
///
/// 読む先は**作業木でなく base（`HEAD`）**である（[`crate::pipe::show_head`]）: 記録する base と同じ commit の
/// 行だけが契約の正本で、commit していない書きかけを受け付けると runner が base で見るものと食い違う。
/// 行を引いたら **(a) の [`table::check_table`] を同じ ctx でその 1 行に撃ち**（1 実装・C2）、findings が 1 件でも
/// 在れば先頭を理由に断る（run dir を作らない・FR48 / FR54）。
pub(in crate::pipe) fn generated(
    repo: &Path,
    pointer: &table::Pointer,
    materials: &Materials,
) -> Result<(Contract, String), Denial> {
    // 置き場の外の pointer は doc を読む前に断る（置き場の外は表でない＝読む理由が無い・設計 contract-source.md §69 形 8・行 cg）。
    // 置き場を読めない周（宣言の不備は材料の読みが先に断っている）は照らさず、今の読みに任せる。
    if let Some(items) = materials.places.as_deref() {
        let paths = std::slice::from_ref(&pointer.path);
        if table::design_docs(paths, items).is_empty() {
            let outside = TableError::PlaceOutside { line: 0, path: pointer.path.clone() };
            return Err(finding_denial(&pointer.path, &[table::Finding::table(outside)]));
        }
    }
    let Some(text) = crate::pipe::show_head(repo, &pointer.path) else {
        let reason = format!("{} を base（HEAD）から読めない", pointer.path);
        return Err(refuse(&Refuse::ContractTable(TableError::Unreadable { line: 0, reason }), &[]));
    };
    generated_from(repo, pointer, &text, materials)
}

/// 行の本文 text から契約を組む。text は base の doc か preflight の --contract の file。
pub(in crate::pipe) fn generated_from(
    repo: &Path,
    pointer: &table::Pointer,
    text: &str,
    materials: &Materials,
) -> Result<(Contract, String), Denial> {
    let row = table::find_row(&pointer.path, text, &pointer.id).map_err(|errors| {
        let rest: Vec<String> = errors.iter().skip(1).map(|error| format!("pipe: {}", error.reason())).collect();
        let first = errors.into_iter().next().unwrap_or(TableError::RowMissing { line: 0, id: pointer.id.clone() });
        refuse(&Refuse::ContractTable(first), &rest)
    })?;
    let findings = check_row(&pointer.path, text, &row, materials);
    if !findings.is_empty() {
        return Err(finding_denial(&pointer.path, &findings));
    }
    exclude_unmeasured(&row, materials, repo)?;
    exclude_unindexed(&row, materials)?;
    let design = format!("{}#{}", pointer.path, pointer.id);
    // 行が `write-set` を持たない周（Derived の行・§3「write-set の導出」）は**導出値**を写しに書く。
    // 契約 file は write-set を 1 本以上要るので、空のまま書くと器が自分の生成物を読めない。
    // 導出は行と base だけで決まるので、後段の [`settle_write_set`] と同じ 1 実装をここで撃つ（C2）。
    // Promised の行（§33）は約束の行からの生成値（write-set / verify / done）を行の値の代わりに渡す。
    let (row, write_set) = match promised(&pointer.path, text, &row, materials)? {
        Some(found) => (generated_row(&row, &found), found.write_set),
        None if row.write_set.is_empty() => {
            let derived = derived_write_set(&row, materials)?;
            (row, derived)
        }
        None => {
            let written = row.write_set.clone();
            (row, written)
        }
    };
    let body = crate::pipe::contract::render(&row, &design, &write_set);
    let contract = Contract::parse(&body).map_err(|errors| {
        denied(DENIAL_GENERATED, unloadable(errors))
    })?;
    Ok((contract, body))
}

/// 表の検査の findings を受付の断りに組む（rc は最大・名は `Refuse::ContractTable` の側から取る＝字面を 2 か所に書かない・C1）。
/// 行は表の検査の描画をそのまま並べる（`contracts check` と 1 byte 同じ行＝読み手が 2 つの形を覚えない）。
fn finding_denial(path: &str, findings: &[table::Finding]) -> Denial {
    let name = Refuse::ContractTable(TableError::RowMissing { line: 0, id: String::new() }).as_str();
    let rc = findings.iter().map(table::Finding::rc).fold(RC_REFUSED, u8::max);
    let lines = findings.iter().map(|finding| finding.render(path)).collect();
    let refusals = findings.iter().map(|finding| finding.refuse().clone()).collect();
    Denial { refusals, ..denied(name, Outcome::failed(rc, lines)) }
}

/// 回答後の再開が写しを取り直す本文（設計 pipeline-question.md §11・契約表の行 a）。
///
/// 写しの `design` が設計 pointer なら受付と**同じ導出**（[`Materials::of`] → [`generated`]）で本文を組み直して返す
/// （受付が写す byte と同じ形＝行が変わらなければ写しと 1 byte も違わない）。pointer でない `design` は `Ok(None)`
/// （取り直す行が無い）。行が受付を通らない周（行が消えた・doc を読めない・表の検査が 1 件でも出る）は受付の断りの
/// 字面をそのまま rc 1 で返す（起こさない・写しと event は呼び手が触らない）。
///
/// 上限の command は**便の写しの有効値**（run dir の `vessel.toml`・受付が上限と突き合わせて凍結した allowlist）で、
/// 禁じる語列は `manifest` の行（追随の [`crate::pipe::follow::stale_rows_in`] と同じ借り方＝resume は受付の `--rules`
/// を持たない）。受付の後に宣言の allowlist が広がった周は上限の外として断る側へ倒れる。
pub(in crate::pipe::cli) fn regenerated(
    repo: &Path,
    state_dir: &Path,
    id: &str,
    manifest: &Manifest,
    design: &str,
) -> Result<Option<String>, Outcome> {
    let Ok(pointer) = table::parse_pointer(design) else {
        return Ok(None);
    };
    let refused_as = |denial: Denial| Outcome { rc: RC_REFUSED, ..denial.outcome };
    let frozen = Effective::load(&vessel_path(state_dir, id))
        .map_err(|errors| Outcome::failed(RC_BROKEN, errors.iter().map(ToString::to_string).collect()))?;
    let denied_commands = list_row(manifest, DENIED_ROW).map_err(refused)?;
    let classes = list_row(manifest, CLASS_ROW).map_err(refused)?;
    let ceiling = Ceiling { row: CEILING_ROW, commands: frozen.allowed(), denied: &denied_commands, classes: &classes };
    // 取り直す本文は台帳を読まない（引用の判定は judge だけが撃つ）ので client は既定の名のまま。
    let materials = Materials::read(repo, &ceiling, DEFAULT_BD).map_err(refused_as)?;
    let (_, body) = generated(repo, &pointer, &materials).map_err(refused_as)?;
    Ok(Some(body))
}

/// 行から導いた write-set（[`settle_write_set`] と同じ [`closure::derive_write_set`] を撃つ）。
fn derived_write_set(row: &ContractRow, materials: &Materials) -> Result<Vec<String>, Denial> {
    let fields = fields_of(row);
    let derived =
        closure::derive_write_set(&fields, &materials.base()).map_err(|error| refuse(&refuse_of(error, row), &[]))?;
    Ok(derived.into_iter().collect())
}

/// Promised の行（設計 contract-source.md §33）の生成値（契約 file に行の値の代わりに載せる）。
struct Generated {
    /// 約束の行から導いた write-set（§3 の導出の 1 本の値・辞書順）。
    write_set: Vec<String>,
    /// 歯を（crate・scope）で束ねた nextest 行。
    verify: Vec<String>,
    /// `n` の順の「(n) expect」の 1 文。
    done: String,
}

/// 行が約束の行を持てば（Promised）生成値を組む（持たない行は `Ok(None)`＝Declared / Derived の判定は不変）。
///
/// 断る順: 器が導く欄を行が書いた（`promised-field-written`・書かれた欄を全部名指す）→ `symbols` の名が base と合わない
/// （`promise-symbol-unresolved`）→ 導出の欄が解けない（§3 と同じ理由）→ 行の `verify` が生成値と集合で違う（§3 と同じ
/// `write-set-drift`）。約束の行は `text`（base の設計 doc の本文）を同じ parse で読み直して引く。
fn promised(path: &str, text: &str, row: &ContractRow, materials: &Materials) -> Result<Option<Generated>, Denial> {
    let (_, promises) = table::read_table(path, text).unwrap_or_default();
    let own = table::promises_of(&promises, &row.id);
    if own.is_empty() {
        return Ok(None);
    }
    let fields = written_fields(row);
    if !fields.is_empty() {
        return Err(refuse(&Refuse::PromisedFieldWritten { row: row.id.clone(), fields }, &[]));
    }
    check_symbols(&own, materials).map_err(|error| refuse(&error, &[]))?;
    let (derived, inputs) =
        closure::derive_promised(&own, &materials.base()).map_err(|error| refuse(&refuse_of(error, row), &[]))?;
    let verify = crate::pipe::contract::promised_verify(&materials.facts.layout.roots, &inputs.teeth);
    if !row.verify.is_empty() {
        let wanted: BTreeSet<String> = verify.iter().cloned().collect();
        closure::check_drift(&row.verify, &wanted).map_err(|error| refuse(&refuse_of(error, row), &[]))?;
    }
    let done = crate::pipe::contract::promised_done(&own);
    Ok(Some(Generated { write_set: derived.into_iter().collect(), verify, done }))
}

/// Promised の行が書いてはならない欄のうち書かれたもの（欄の宣言順＝`FIELDS` の順）。
fn written_fields(row: &ContractRow) -> Vec<String> {
    let lists = [
        ("touches", &row.touches),
        ("surfaces", &row.surfaces),
        ("write-set", &row.write_set),
        ("creates", &row.creates),
        ("tests", &row.tests),
        ("also", &row.also),
    ];
    let mut found: Vec<String> =
        lists.iter().filter(|(_, items)| !items.is_empty()).map(|(name, _)| (*name).to_owned()).collect();
    if !row.done.is_empty() {
        found.push("done".to_owned());
    }
    found
}

/// 約束の行の `symbols` の実在（§33 項 5・[`closure::symbols_in_base`] の同じ読み手）: `+` 無しの名は base に解け、`+` 付き
/// の名は base に**無い**こと（`creates` の `MustBeAbsent` と同じ極性）。名指しの形でない字面は測れない（下界）。
fn check_symbols(own: &[&table::PromiseRow], materials: &Materials) -> Result<(), Refuse> {
    for promise in own {
        let names: Vec<&str> = promise.symbols.iter().map(|name| name.strip_prefix(NEW_FILE).unwrap_or(name)).collect();
        let found = closure::symbols_in_base(&names, &materials.tracked, &materials.sources)
            .map_err(|error| Refuse::ContractTable(TableError::Unreadable { line: promise.line, reason: error.reason() }))?;
        for (name, in_base) in promise.symbols.iter().zip(found) {
            if in_base == Some(name.starts_with(NEW_FILE)) {
                return Err(Refuse::PromiseSymbolUnresolved { of: promise.of.clone(), n: promise.n, name: name.clone() });
            }
        }
    }
    Ok(())
}

/// 生成値を行の値の代わりに持つ行（`verify` / `done` を差し替える・他の欄は行のまま）。
fn generated_row(row: &ContractRow, generated: &Generated) -> ContractRow {
    ContractRow { verify: generated.verify.clone(), done: generated.done.clone(), ..row.clone() }
}

/// 行 1 つに (a) の表の検査を撃つ（`contracts check` と**同じ 1 実装**・C2）。ctx（allowlist / 禁じる語 /
/// 要件面 / base の tree）は `contracts check` と同じ材料から組み、doc の本文は base（`HEAD`）の字面を渡す。
///
/// 材料（tracked / sources / snapshots / facts）は [`Materials::read`] が 1 周に 1 回だけ読む。base の tree を
/// 読めない周と宣言が上限に外れる周は、その読みの時点で断る（順序は従来のまま）。
///
/// 検査する行は 1 つのままだが、`depends` の解決の母集団は**同じ doc の全行の id**（§30・行 ad）: 既に読んだ
/// base の本文から全行を引き直して id だけを渡す（新しい読みは足さない・閉包と名指しは当該行にだけ撃つ）。
/// 行は [`table::find_row`] で既に読めているので、全行の読みが落ちる周は無い（落ちれば母集団は空＝相手の在る
/// `depends` も断る側に倒れる・黙って通さない）。
fn check_row(path: &str, text: &str, row: &ContractRow, materials: &Materials) -> Vec<table::Finding> {
    let rows = table::read_rows(path, text).unwrap_or_default();
    let ids: Vec<&str> = rows.iter().map(|other| other.id.as_str()).collect();
    let mut found = table::check_table(text, std::slice::from_ref(row), &ids, &materials.context());
    // 欄 done-teeth の在りか（設計 contract-source.md §66 形 2 の (e)）は base を渡す口だけが撃つ＝受付と preflight（表の検査の本体は撃たない）。
    found.extend(table::done_teeth_located_findings(row, &materials.base()));
    found
}

/// 行の事実（設計 pointer と § 番号・verify の nextest 行ごとの歯の置き場）を judge に載せる（§21・判定はしない）。
/// pointer でない `design` と行の解けない周（`settle_write_set` が同じ根で断る）は載せず、読めない `.rs` が在る周は
/// 歯の置き場を測れない（Unreadable の断りが立つ）ので `teeth` を載せない。
///
/// 歯の置き場は導出 (ii) と Declared 行の門が撃つ [`closure::teeth_places`] の同じ 1 実装で、行ごとに `tests` 欄を空に
/// して読む（`tests` の file は置き場でなく write-set の側）。filter 語も同じ 1 実装から取る: 本文 0 本で撃つと nextest
/// 形の行は必ず [`ClosureError::TeethPlaceUnresolved`] で filter 語を返し、nextest 形でない行は空で通る（2 本目の
/// 読み手を作らない）。本文で 0 本の filter 語は 0 本の事実として載せる（断るかは `settle_write_set` の側）。
pub(super) fn row_facts(repo: &Path, contract: &Contract, materials: &Materials, judged: &mut Judged) {
    let Ok(Some(Pointed { row, .. })) = pointed_row(repo, contract, materials) else {
        return;
    };
    judged.design = Some((contract.design.clone(), row.section.clone()));
    let texts: Option<Vec<(&str, &str)>> = materials
        .sources
        .iter()
        .map(|source| source.body.as_deref().ok().map(|text| (source.path.as_str(), text)))
        .collect();
    let Some(texts) = texts else {
        return;
    };
    let fields = fields_of(&row);
    let base = materials.base();
    for line in &row.verify {
        let one = closure::Fields { verify: std::slice::from_ref(line), tests: &[], ..fields };
        let Err(ClosureError::TeethPlaceUnresolved { filter }) = closure::teeth_places(&one, &base, &[]) else {
            continue;
        };
        let files: Vec<String> = closure::teeth_places(&one, &base, &texts).map(Vec::from_iter).unwrap_or_default();
        judged.teeth.push((filter, files));
    }
}

/// 設計 pointer の行から決めた write-set の弁別（設計 contract-source.md §3・受付だけ）。
pub(super) struct Settled {
    /// 導出か手書きか。
    pub(super) kind: WriteSet,
    /// 導出値を契約の写しの write-set にする周（行に `write-set` の無い `Derived`）の導出値。他は `None`。
    pub(super) replaced: Option<Vec<String>>,
    /// 判定行に載せる本数（導出値か手書きの項目数）。
    pub(super) files: usize,
}

/// 設計 pointer の行（Promised の行は `verify` / `done` を生成値に差し替えた形）と、Promised の行の導出した write-set。
pub(super) struct Pointed {
    /// 行（Promised の行は [`generated_row`] の形＝写しと同じ値）。
    pub(super) row: ContractRow,
    /// Promised の行の write-set（他の行は `None`）。
    pub(super) promised: Option<Vec<String>>,
}

/// 契約の `design` が設計 pointer（`<doc>#<id>`）なら base の契約表の行を引く（[`settle_write_set`] と [`row_facts`] が
/// 同じ 1 本で読む）。pointer でない `design`（(b) の前の契約 file）は `Ok(None)`。pointer が解けない（doc を読めない・
/// 区間が無い・行が無い）周は契約表の欠陥として断る（FR54・fail-closed）。Promised の行は [`promised`] の同じ 1 本で
/// 生成値を組む（[`generated`] と同じ断り）。
pub(super) fn pointed_row(repo: &Path, contract: &Contract, materials: &Materials) -> Result<Option<Pointed>, Denial> {
    let Ok(pointer) = table::parse_pointer(&contract.design) else {
        return Ok(None);
    };
    let text = table::read(repo, &pointer.path)
        .map_err(|reason| refuse(&Refuse::ContractTable(TableError::Unreadable { line: 0, reason }), &[]))?;
    let row = table::find_row(&pointer.path, &text, &pointer.id).map_err(|errors| {
        let rest: Vec<String> = errors.iter().skip(1).map(|error| format!("pipe: {}", error.reason())).collect();
        let first = errors.into_iter().next().unwrap_or(TableError::RowMissing { line: 0, id: pointer.id.clone() });
        refuse(&Refuse::ContractTable(first), &rest)
    })?;
    Ok(Some(match promised(&pointer.path, &text, &row, materials)? {
        Some(found) => Pointed { row: generated_row(&row, &found), promised: Some(found.write_set) },
        None => Pointed { row, promised: None },
    }))
}

/// 行の欄を導出の材料に写す（`settle_write_set` と `row_facts` が同じ形で組む）。
fn fields_of(row: &ContractRow) -> closure::Fields<'_> {
    closure::Fields {
        touches: &row.touches,
        surfaces: &row.surfaces,
        verify: &row.verify,
        creates: &row.creates,
        tests: &row.tests,
        also: &row.also,
        files: &[],
    }
}

/// base の tree の事実を導出の材料に写す。
pub(super) fn base_of<'a>(sources: &'a [Source], snapshots: &'a [Source], tracked: &'a [String], layout: &'a closure::CrateLayout) -> closure::Base<'a> {
    closure::Base { sources, snapshots, tracked, core_crate: NAME, layout }
}

/// 契約の `design` が設計 pointer なら base の契約表の行を引いて write-set を弁別する（§3「手書きの write-set の扱いと
/// 撃つ場所」）。pointer でない `design` は `None`＝従来どおり導出しない。行の読みは [`pointed_row`]。
///
/// - `creates` / `tests` / `also` を 1 つも持たず `write-set` を持つ行は [`WriteSet::Declared`]（導出も drift も撃たず、
///   verify の歯の file が write-set に在るかの門〔[`closure::declared_teeth`]・§20〕だけを撃つ）。
/// - それ以外は [`WriteSet::Derived`]: 導出値を作り（解けない欄は typed に断る）、
///   行に `write-set` が在れば集合一致でなければ `write-set-drift`・無ければ導出値が
///   write-set になる。
pub(super) fn settle_write_set(repo: &Path, contract: &Contract, materials: &Materials) -> Result<Option<Settled>, Denial> {
    let Some(Pointed { row, promised }) = pointed_row(repo, contract, materials)? else {
        return Ok(None);
    };
    // Promised の行（§33）は約束の行から導いた値が write-set（行は write-set を持てない＝drift も撃たない）。
    if let Some(files) = promised {
        return Ok(Some(Settled { kind: WriteSet::Promised, files: files.len(), replaced: Some(files) }));
    }
    let declared = row.creates.is_empty() && row.tests.is_empty() && row.also.is_empty() && !row.write_set.is_empty();
    let fields = fields_of(&row);
    let base = materials.base();
    if declared {
        // Declared 行は導出も drift も撃たないが、**歯の置き場の門**だけは撃つ（§20・行 t）: verify の nextest 行の
        // 歯の file が write-set の外に在る契約は、便を作らずに file を全部名指して断る（審査へ先送りしない・C16）。
        closure::declared_teeth(&fields, &base, &row.write_set).map_err(|error| refuse(&refuse_of(error, &row), &[]))?;
        let files = row.write_set.len();
        return Ok(Some(Settled { kind: WriteSet::Declared, replaced: None, files }));
    }
    let derived = closure::derive_write_set(&fields, &base).map_err(|error| refuse(&refuse_of(error, &row), &[]))?;
    let files = derived.len();
    if row.write_set.is_empty() {
        let replaced: Vec<String> = derived.into_iter().collect();
        return Ok(Some(Settled { kind: WriteSet::Derived, replaced: Some(replaced), files }));
    }
    closure::check_drift(&row.write_set, &derived).map_err(|error| refuse(&refuse_of(error, &row), &[]))?;
    Ok(Some(Settled { kind: WriteSet::Derived, replaced: None, files }))
}
