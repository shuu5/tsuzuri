//! `pipe intake` の受付。
//! 契約 file を読み、宣言を上限と突き合わせ、上限の余地と write-set の交差で断り、置き場へ写して run を 起こす。
//! 親の共通の材料は `super::` で引く。
//! 行が導く欄を手で書いた周は `promised-field-written`・ `symbols` の名が base と合わない周は `promise-symbol-unresolved`。
//! 出所: 設計 §5 §3 pipeline-conflict.md §2 contract-source.md §3 §33 §21 §23 s2-07l.295 s2-07l.396

use super::base_run::{self, BaseRun, Early};
use super::{broken, flag, int_row, list_row, need, refused, repo_flag, repo_of, state_dir_of, REPO_FLAG};
use crate::cli_outcome::{Outcome, RC_BROKEN, RC_REFUSED};
use crate::fleet::store::{self, LockPolicy};
use crate::fleet::{self, EventKind, Stage};
use crate::hook::command;
use crate::hook::host_guard::WORD_ROWS;
use crate::pipe::closure::{self, Source};
use crate::pipe::contract::{Contract, ContractError, CLASS_ROW};
use crate::pipe::declaration::{self, Ceiling, Effective, EntranceFlip, CEILING_ROW, DENIED_ROW};
use crate::pipe::dispatch::index_build::{status as index_status, Status};
use crate::pipe::refuse::Refuse;
use crate::pipe::table::{self, ContractRow, TableError};
use crate::pipe::commute::{self, Crossed};
use crate::pipe::{contract_path, emit, run_dir, run_id, vessel_path, Emit};
use crate::rules::manifest::Manifest;
use crate::rules::RuleValue;
use crate::seat::ledger::{timeout_of, DEFAULT_BD};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// 断りの組み立てと上限の余地の群（契約表の行 bl・純移動）。
mod refusal;
use refusal::{denied, exclude_cap_shortfall, not_a_repo, refuse};
use refusal::{DENIAL_ARGS, DENIAL_DECLARATION, DENIAL_GENERATED, DENIAL_RULES, DENIAL_STORE};

/// 契約を台帳の bead に置く形の周（引数の読みと台帳の読みと照らし・契約表の行 v-bead-intake）。
pub(super) mod bead;

/// 裁定 id の引用の判定（設計 dispatcher.md §37・契約表の行 al）。断りの組み立ては [`exclude_unresolved_rulings`] が書く。
mod ruling;
use ruling::Ruled;

/// 索引の状態の扱いと閉包の判定（設計 reverse-index.md §7 (b)・契約表の行 d）。断りの組み立ては [`exclude_unindexed`] が書く。
mod index;

/// 欄 `code-facts` の照らしと測り（設計 reverse-index.md §7 (c)・契約表の行 e）。断りの組み立ては [`exclude_unmeasured`] が書く。
mod code_facts;

/// live な便の契約の読みと宣言の同時の数の上限の判じ（tsuzuri の判断の記録 ADR-63 の決定 (13)）。
mod run_cap;
pub(in crate::pipe) use run_cap::capped;
use run_cap::exclude_run_cap;

/// 設計 pointer の行の読み・契約の生成・write-set の弁別（契約表の行 v-intake-split・純移動）。
mod row;
pub(in crate::pipe) use row::{generated, generated_from};
pub(super) use row::regenerated;
use row::{base_of, row_facts, settle_write_set};

/// 同型の停止と焼き直しの門（設計 contract-source.md §23・契約表の行 v-intake-split・純移動）。
mod repeats;
use repeats::exclude_repeats;

/// live な便との交差と同時本数（契約表の行 v-intake-split・純移動）。
mod overlap;
pub(in crate::pipe) use overlap::crossings;
use overlap::{exclude_max_live, exclude_overlap};

/// 契約の write-set の出所（設計 contract-source.md §3 / §33・C10「導出値と宣言値を型で分ける」）。**閉じた 3 値**で、
/// 契約表の行の欄と約束の行の有無だけで決まる（散文の免除を持たない）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum WriteSet {
    /// 器が導出した（`creates` / `tests` / `also` のどれかを持つ行と、新欄も `write-set` も無い行）。
    Derived,
    /// 行が手で列挙した（新欄を持たず `write-set` を持つ行・(h) の前の形）。
    Declared,
    /// 器が約束の行（`[[promise]]`）から導出した（約束の行を 1 つでも持つ行・§33）。
    Promised,
}

/// [`WriteSet`] の全 variant（宣言順・`enum-slices` が集合完全性を測る・読むのは pin の歯だけ）。
#[cfg_attr(not(test), expect(dead_code, reason = "宣言順 pin の歯だけが読む（`REFUSALS` と同じ形）"))]
pub(crate) const WRITE_SETS: &[WriteSet] = &[WriteSet::Derived, WriteSet::Declared, WriteSet::Promised];

impl WriteSet {
    /// 判定行の token の値。
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Derived => "derived",
            Self::Declared => "declared",
            Self::Promised => "promised",
        }
    }
}

/// 受付を通った便（`run` の連鎖は id だけを要る・`intake` は判定行に write-set の弁別も載せる）。
struct Intaken {
    /// run id。
    id: String,
    /// write-set の弁別と本数（設計 pointer を持たない契約は `None`＝従来の形）。
    write_set: Option<(WriteSet, usize)>,
    /// base の木で撃った周の欄（[`BaseRun::fact`]・名乗りの無い周は `None`＝1 行は従来の形・§56 形 5）。
    entrance: Option<String>,
    /// 索引を作れない周（状態が failed）の尾 `index=unavailable:<語>`（touches に型の項目を持つ行だけ・設計 reverse-index.md §7 (b)）。
    index: Option<String>,
}

/// 契約 file を読み込み、置き場へ写して run を起こし、**直後に審査の段を通す**（FR49・設計 contract-source.md
/// §4）。1 行目は受付の判定行・2 行目は審査の判定行で、rc は審査の verdict（PASS = 0 / FAIL = 1 /
/// INCONCLUSIVE = 3）＝受付は通っても PASS でない便は終端で、`run=<id>` は落ちた周も出す。
pub(super) fn intake(args: &[String], manifest: &Manifest, policy: LockPolicy) -> Outcome {
    match intake_run(args, manifest, policy) {
        Ok(found) => {
            let mut line = intake_line(args, &found.id);
            if let Some((kind, files)) = found.write_set {
                line.push_str(&format!(" write-set={} files={files}", kind.as_str()));
            }
            if let Some(fact) = found.entrance {
                line.push_str(&format!(" {fact}"));
            }
            if let Some(tail) = found.index {
                line.push_str(&format!(" {tail}"));
            }
            let mut reviewed = super::step::review_run(args, &found.id, manifest, policy);
            reviewed.out.insert(0, line);
            reviewed
        }
        Err(outcome) => outcome,
    }
}

/// intake の 1 行。`--rules` で上限を差し替えて通した周は**その事実を同じ行に残す**
/// （`ceiling-overridden=<path>`・値は渡した path の字面そのもの・`s2-07l.65`）。
///
/// `--rules` は test の seam で、上限（`runner.allowed_commands`）を無条件に差し替える。
/// 差し替えた周が通常の周と同じ 1 行しか出さないと、review は「埋め込みの上限で通った便」と
/// 区別できない（`.56` lens M1）。差し替えていない周は出さない＝不在が既定。
pub(super) fn intake_line(args: &[String], id: &str) -> String {
    match flag(args, "--rules") {
        Ok(Some(path)) => format!("run={id} ceiling-overridden={path}"),
        _ => format!("run={id}"),
    }
}

/// intake の本体。**id を返す**のは `run` が続きの段へ渡すためである
/// （自分の stdout を読み直して id を取る形にすると、表示を変えた瞬間に連鎖が壊れる）。もう 1 つの値は受付の結果の行に足す索引の
/// 尾（` index=unavailable:<語>`・先頭に空白を持ち、尾の無い周は空）で、`pipe run` の行は同じ尾を足すだけ。
pub(super) fn intake_id(args: &[String], manifest: &Manifest, policy: LockPolicy) -> Result<(String, String), Outcome> {
    intake_run(args, manifest, policy).map(|found| (found.id, found.index.map_or_else(String::new, |tail| format!(" {tail}"))))
}

/// 受付の 1 周（id と write-set の弁別）= [`judge`] → [`create`]。
fn intake_run(args: &[String], manifest: &Manifest, policy: LockPolicy) -> Result<Intaken, Outcome> {
    let (source, bead, repo) = bead::source_of(args).map_err(|denial| denial.outcome)?;
    // repo は spawn まで使わないが、**intake の時点で** git repo かを確かめる（judge も先頭で同じ検査を撃つが、intake は
    // 置き場と行を読む前に断る＝従来の順）。後段で初めて落ちると、契約は受理されたのに進めない run が残る。
    // HEAD の sha は名乗りに依らず材料の読みの前に読む（base の木で撃った後に読み直して比べる・§56 形 2）。
    let Some(sha) = super::head_of(&repo) else {
        return Err(not_a_repo(&repo).outcome);
    };
    let state_dir = state_dir_of(args).map_err(refused)?;
    // 上限・材料（設計 dispatcher.md §5 の 1 回）・行の生成と表の検査・freeze は**入口の lock の前**に 1 周に 1 回
    // （§56 形 2・base の木の実走の長さで入口を塞がない）。lock の中の judge は freeze の結果を借りる。
    let ceiling = ceiling_of(manifest).map_err(|denial| denial.outcome)?;
    let bd = flag(args, "--bd").map_err(refused)?.unwrap_or(DEFAULT_BD);
    let materials = Materials::read(&repo, &ceiling.borrow(), bd).map_err(|denial| denial.outcome)?.indexed(&state_dir, &repo, &sha);
    let (contract, body) = bead::contract_of(&source, (&repo, &state_dir), (manifest, bd), &bead, &materials).map_err(|denial| denial.outcome)?;
    let early = early(&repo, manifest, &contract, Some(&state_dir), &sha);
    // **入口の排他はここから**（ADR-0019 §2.1・設計 pipeline-conflict.md §2）: [`judge`] と [`create`] を
    // 1 つの周として閉じる。持たないと、同時に来た 2 つの受付がどちらも「live な便は無い」と読んでから
    // 両方が run を作る（`s2-07l.366` の実測: 契機を同時に 2 回撃つと 20 回に 1 回 2 本作られた）。
    let _entrance = Entrance::hold(&state_dir, policy)?;
    let material = Material {
        repo: &repo, manifest, contract: &contract, state_dir: Some(&state_dir), bead: &bead, materials: &materials,
        early: Some(&early),
    };
    create(judge(&material), &material, &state_dir, &body, policy)
}

/// lock の前の 1 回の読み（§56 形 2 / 3）: freeze を撃ち、宣言が `detect` / `deny` を名乗る周だけ契約の検証行を base の木で撃つ。
pub(super) fn early(repo: &Path, manifest: &Manifest, contract: &Contract, state_dir: Option<&Path>, sha: &str) -> Early {
    let frozen = freeze(repo, manifest, contract);
    let fires = frozen.as_ref().is_ok_and(|(_, named)| named.is_some_and(EntranceFlip::fires_on_base));
    let base = fires.then(|| base_run::run(repo, state_dir, sha, &contract.verify, manifest));
    Early { frozen, base }
}

/// 受付の入口の lock file の名（置き場の直下・**run dir の側に置かない**）。
///
/// `pipe/` の下に置くと「断った周は run dir を 1 つも作らない」を測る既存の歯が、lock file を run dir と
/// 数えて落ちる（`s2-07l.366` の実測 4 本）。入口の lock は便ではないので便の置き場に混ぜない。
const ENTRANCE_LOCK: &str = "intake.lock";

/// 受付の入口の排他（**[`judge`] と [`create`] を 1 つの周として閉じる**・ADR-0019 §2.1）。
///
/// event log の lock（[`store::append_line`] が中で取る）とは**別の lock file** である——同じものを外から
/// 握ると、`create` の記帳が自分の握った lock を待って止まる。`Drop` で外すので、判定のどの断りに落ちても
/// 残らない。
struct Entrance {
    /// 握っている lock file。
    lock: PathBuf,
}

impl Entrance {
    /// 入口を 1 つだけ通す（取れない周は rc 2＝置き場が壊れている側）。
    fn hold(state_dir: &Path, policy: LockPolicy) -> Result<Self, Outcome> {
        std::fs::create_dir_all(state_dir).map_err(|err| broken(format!("受付の入口を作れない: {err}")))?;
        let lock = state_dir.join(ENTRANCE_LOCK);
        store::acquire(&lock, policy).map_err(|err| broken(format!("受付の入口の lock を取れない: {err}")))?;
        Ok(Self { lock })
    }
}

impl Drop for Entrance {
    fn drop(&mut self) {
        // 外せない lock は次の受付が所有者の生死で回収する（ここで止めない）。
        let _ = std::fs::remove_file(&self.lock);
    }
}

/// rules 行から allowlist と禁じる語を読んで [`Ceiling`] の材料を持つ（借りる側は [`Rows::borrow`]）。
pub(in crate::pipe) struct Rows {
    /// 通す語列（`runner.allowed_commands`）。
    commands: Vec<String>,
    /// 禁じる語列（`runner.denied_commands`）。
    denied: Vec<String>,
    /// クラスの語列表（[`CLASS_ROW`]・設計 contract-source.md §48 の 5）。
    classes: Vec<String>,
}

impl Rows {
    /// 借りた形の上限（`row` は同じ 1 つの rules 行 id）。
    pub(in crate::pipe) fn borrow(&self) -> Ceiling<'_> {
        Ceiling { row: CEILING_ROW, commands: &self.commands, denied: &self.denied, classes: &self.classes }
    }
}

/// 1 周ぶんの repo の材料（**読みは 1 周に 1 回**・設計 dispatcher.md §5・契約表の行 b）。
///
/// base の tree の走査（tracked の一覧・`.rs`・`.snap`）と、宣言から解いた契約表の facts は**契約 1 本ごとに
/// 変わらない**。受付は 1 便で 1 回読むだけだが、列（[`crate::pipe::dispatch`]）は候補の数だけ [`generated`] と
/// [`judge`] を撃つので、読み直すと 1 周が候補数に比例して伸びる（`s2-07l.345` の実測: 候補 1 件あたり 0.2 秒）。
/// **判定の関数は不変で、材料を受け取る引数だけを足す**（C2）。
#[derive(Clone)]
pub(in crate::pipe) struct Materials {
    /// base の tracked file の repo 相対 path。
    tracked: Vec<String>,
    /// base の `.rs`（閉包を測る側）。
    sources: Vec<Source>,
    /// base の `.snap`（外形 pin を測る側）。
    snapshots: Vec<Source>,
    /// 宣言から解いた allowlist / 禁じる語 / 要件面の path。
    facts: declaration::TableFacts,
    /// 要件面の id の集合（読めない周は理由・表の検査がそのまま断る）。
    requirements: Result<BTreeSet<String>, String>,
    /// repo の全 doc が宣言済みの新規 file（`contracts check` と同じ 1 本 [`table::declared_files`]・読めない周は理由）。
    declared: Result<Vec<String>, String>,
    /// HEAD の宣言の契約表の置き場の項目（key `contract-tables`・書かない宣言は空・読めない周は `None`＝照らさない・設計
    /// contract-source.md §69 形 8・行 cg）。受付の pointer の path をこれに照らす（[`generated`]）。
    places: Option<Vec<String>>,
    /// クラスの語列表（上限の [`Ceiling`] から借りた写し・表の検査が verify 行から 3 クラスを導く）。
    classes: Vec<String>,
    /// 裁定 id の引用の判定の材料（台帳の client と、1 周に 1 回だけ読む宣言・台帳・線・設計 dispatcher.md §37 約束 5）。
    ruling: ruling::Rulings,
    /// base の commit の索引の状態（state dir を持つ呼び手だけが [`Self::indexed`] で載せる・`None` = 状態なし＝字面の閉包だけで判じ
    /// 尾も足さない・表は大きいので借りて持つ）。
    index: Option<Arc<Status>>,
    /// 索引の状態を読んだ commit の sha（[`Self::indexed`] が状態と一緒に載せる・欄 `code-facts` の測りが数える commit）。
    index_at: Option<String>,
}

impl Materials {
    /// repo を 1 回走査して材料を読む。`bd` は裁定 id の引用の判定が台帳を読む client（読むのは引用を持つ契約が出たときだけ）。
    ///
    /// 断りの**順序は従来のまま**である（tracked を読めない＝git repo でない〔rc 2〕→ 宣言が読めない・上限に
    /// 外れる〔rc 1〕）。順を替えると「宣言が壊れている」便が「表を読めない」に化ける。
    pub(in crate::pipe) fn read(repo: &Path, ceiling: &Ceiling<'_>, bd: &str) -> Result<Self, Denial> {
        let Some(tracked) = table::tracked_files(repo) else {
            let reason = format!("{} の tracked file を読めない（git repo でない）", repo.display());
            return Err(refuse(&Refuse::ContractTable(TableError::Unreadable { line: 0, reason }), &[]));
        };
        let facts = declaration::table_facts(repo, ceiling, &tracked).map_err(|errors| {
            denied(DENIAL_DECLARATION, Outcome::failed(RC_REFUSED, errors.iter().map(ToString::to_string).collect()))
        })?;
        let sources = table::read_all(repo, &tracked, ".rs");
        let snapshots = table::read_all(repo, &tracked, ".snap");
        let requirements =
            table::read(repo, &facts.requirements).and_then(|found| table::requirement_ids(&facts.requirements, &found));
        let declared = table::declared_files(repo, &tracked);
        let places = declaration::TablePlaces::at(repo, "HEAD").items().map(<[String]>::to_vec);
        Ok(Self {
            tracked,
            sources,
            snapshots,
            facts,
            requirements,
            declared,
            places,
            classes: ceiling.classes.to_vec(),
            ruling: ruling::Rulings::new(bd),
            index: None,
            index_at: None,
        })
    }

    /// base（`sha`）の commit の索引の状態を載せる（行 a2 の状態の読み・読むだけで撃たず待たない・設計 reverse-index.md §7 (b)）。
    pub(in crate::pipe) fn indexed(self, state_dir: &Path, repo: &Path, sha: &str) -> Self {
        Self { index: Some(Arc::new(index_status(state_dir, repo, sha))), index_at: Some(sha.to_owned()), ..self }
    }

    /// 索引を作れない周の結果の行の尾（[`index::tail`]・`touches` は契約の touches）。
    pub(in crate::pipe) fn index_tail(&self, touches: &[String]) -> Option<String> {
        index::tail(self.index.as_deref(), touches)
    }

    /// rules 行の上限から材料を読む（列の入口・上限の読みと base の走査を 1 本にまとめた口）。
    pub(in crate::pipe) fn of(repo: &Path, manifest: &Manifest, bd: &str) -> Result<Self, Denial> {
        let rows = ceiling_of(manifest)?;
        Self::read(repo, &rows.borrow(), bd)
    }

    /// base の tracked file（交差の dir の展開が読む・**2 本目の走査を作らない**ための借り）。
    pub(in crate::pipe) fn tracked(&self) -> &[String] {
        &self.tracked
    }

    /// 予想の base の写し（**口は 1 つ**・設計 dispatcher.md §27 形 2）: tracked に `add` を足して `remove` を除き、`bodies` の
    /// `.rs` / `.snap` の本文で置き換える（消した file と置き換えた file の元の本文は落とす）。判定の関数は不変。索引の状態は
    /// 持ち越さない（予想の base は状態なし＝設計 reverse-index.md §7 (b)・元の材料の状態は変わらない）。
    pub(in crate::pipe) fn forecast(&self, add: &[String], remove: &[String], bodies: &[Source]) -> Self {
        let dropped = |path: &str| remove.iter().chain(bodies.iter().map(|found| &found.path)).any(|gone| gone == path);
        let swap = |list: &[Source], ext: &str| -> Vec<Source> {
            let kept = list.iter().filter(|found| !dropped(&found.path));
            kept.chain(bodies.iter().filter(|found| found.path.ends_with(ext))).cloned().collect()
        };
        let mut tracked: Vec<String> = self.tracked.iter().filter(|path| !remove.contains(path)).chain(add).cloned().collect();
        tracked.sort();
        tracked.dedup();
        Self { tracked, sources: swap(&self.sources, ".rs"), snapshots: swap(&self.snapshots, ".snap"), index: None, index_at: None, ..self.clone() }
    }

    /// 閉包を測る base（`.rs` / `.snap` / tracked の 3 面を 1 つに束ねた借り）。
    fn base(&self) -> closure::Base<'_> {
        base_of(&self.sources, &self.snapshots, &self.tracked, &self.facts.layout)
    }

    /// 表の検査の ctx（`contracts check` と同じ材料の形）。
    fn context(&self) -> table::Context<'_> {
        table::Context {
            allowed: &self.facts.allowed,
            denied: &self.facts.denied,
            classes: &self.classes,
            requirements: &self.requirements,
            sources: &self.sources,
            tracked: &self.tracked,
            snapshots: &self.snapshots,
            declared: &self.declared,
            layout: &self.facts.layout,
        }
    }
}

/// 上限の材料を rules 行から読む（読めない周は [`DENIAL_RULES`] の断り・[`freeze`] と同じ 2 行）。
pub(super) fn ceiling_of(manifest: &Manifest) -> Result<Rows, Denial> {
    let rules = |reason| denied(DENIAL_RULES, refused(reason));
    let commands = list_row(manifest, CEILING_ROW).map_err(rules)?;
    let denied_commands = denied_rows(manifest).map_err(rules)?;
    Ok(Rows { commands, denied: denied_commands, classes: list_row(manifest, CLASS_ROW).map_err(rules)? })
}

/// 禁じる語列: command guard の ∪ の読み手 1 本（[`command::denied_of`]・`runner.denied_commands` ∪ host-guard の語列の
/// 3 行・設計 vessel-hook.md §11 の形 f 3）。揃わない周は欠けた行を名指す（`runner.denied_commands` は従来の字面）。
/// 断り文の行 id は `declaration` が持つ従来の `runner.denied_commands` のまま（限界・後続の純移動で直す）。
fn denied_rows(manifest: &Manifest) -> Result<Vec<String>, String> {
    list_row(manifest, DENIED_ROW)?;
    let sources = command::denied_of(manifest).ok_or_else(|| {
        let id = WORD_ROWS.iter().find(|id| !matches!(manifest.get(id).map(|row| &row.value), Some(RuleValue::List(_))));
        format!("{} が無いか文字列の列でない", id.unwrap_or(&DENIED_ROW))
    })?;
    Ok(sources.into_iter().flat_map(|(_, sequences)| sequences).collect())
}

/// intake / preflight が同じ形で読む引数（`--design` / `--bead` / `--repo`・欠けは理由の 1 行）。
///
/// 契約 (b) 以後、受付が受けるのは**設計 pointer だけ**である（`<doc>#<id>`）。手書きの契約 file
/// （`--contract`）は使い方の誤りでなく [`Refuse::HandWrittenContract`] で断る（FR54）＝「渡し方を間違えた」
/// ではなく「契約の正本はそこに無い」と名乗る。pointer の形が壊れている周は理由の 1 行で断る。
pub(super) fn read_args(args: &[String]) -> Result<(table::Pointer, String, PathBuf), Denial> {
    if let Ok(Some(path)) = flag(args, FLAG_CONTRACT) {
        return Err(refuse(&Refuse::HandWrittenContract { path: path.to_owned() }, &[]));
    }
    let read = |name: &str| need(args, name).map_err(|reason| denied(DENIAL_ARGS, refused(reason)));
    let design = read("--design")?.to_owned();
    let bead = read("--bead")?.to_owned();
    // repo の読み手は [`repo_flag`] の 1 本（絶対 path に直す・設計 dispatcher.md §12）。必須の断りは `need` と同じ字面。
    let repo = repo_flag(args)
        .and_then(|found| found.ok_or(format!("{REPO_FLAG} が要る")))
        .map_err(|reason| denied(DENIAL_ARGS, refused(reason)))?;
    let pointer = table::parse_pointer(&design)
        .map_err(|err| denied(DENIAL_ARGS, refused(format!("--design {design} は設計 pointer の形でない（{}）", err.reason()))))?;
    Ok((pointer, bead, repo))
}

/// 廃止した手書きの契約 file の flag（字面だけ残して断る側に使う・契約 (b)）。
const FLAG_CONTRACT: &str = "--contract";

/// touches に型の項目を持つ行を、base の索引の状態で断る（設計 reverse-index.md §7 (b)・[`generated`] の表の検査が findings 0 で通った
/// 後に撃つ＝表の検査の断りが先）。判定と閉じた結果は [`index::judge`]、ここは断りを組むだけ: 作り中は `index-building`・half は
/// 宣言の不備と同じ `declaration`・閉包の不足は file に ` (索引)` を添えた `write-set-incomplete`。write-set を持つのは手で列挙した
/// 行だけ（導出する行は足す手段が無いので閉包を測らない）。
fn exclude_unindexed(row: &ContractRow, materials: &Materials) -> Result<(), Denial> {
    let declared = row.creates.is_empty() && row.tests.is_empty() && row.also.is_empty() && !row.write_set.is_empty();
    let write_set = declared.then_some(row.write_set.as_slice());
    match index::judge(materials.index.as_deref(), &row.touches, write_set, &materials.sources) {
        index::Indexed::Clear => Ok(()),
        index::Indexed::Building(state) => Err(refuse(&Refuse::IndexBuilding { state: state.to_owned() }, &[])),
        index::Indexed::Half(lines) => Err(denied(DENIAL_DECLARATION, Outcome::failed(RC_REFUSED, lines))),
        index::Indexed::Missing(missing) => Err(refuse(&Refuse::WriteSetIncomplete { missing }, &[])),
    }
}

/// 欄 `code-facts` を持つ行を base の索引の状態と列の数えで断る（設計 reverse-index.md §7 (c)・[`exclude_unindexed`] の前に撃つ・欄の無い行は通す）。
/// 判定は [`code_facts::judge`]、ここは断りを組むだけ: 測れない周は `code-facts-unmeasured`・half は `declaration`・違いは `code-facts`（先頭の要素が
/// 断りの 1 行・残りは後ろの行）。ready で通れば続く [`exclude_unindexed`] が touches の型の索引の閉包も撃つ。
fn exclude_unmeasured(row: &ContractRow, materials: &Materials, repo: &Path) -> Result<(), Denial> {
    if row.code_facts.is_empty() {
        return Ok(());
    }
    let at = materials.index_at.as_deref().map(|sha| (repo, sha));
    match code_facts::judge(&row.code_facts, materials.index.as_deref(), at) {
        code_facts::Facts::Clear => Ok(()),
        code_facts::Facts::Unmeasured { element, state } => Err(refuse(&Refuse::CodeFactsUnmeasured { element, state }, &[])),
        code_facts::Facts::Half(lines) => Err(denied(DENIAL_DECLARATION, Outcome::failed(RC_REFUSED, lines))),
        code_facts::Facts::Differ(found) => {
            let mut all = found.into_iter().map(|one| Refuse::CodeFacts(Box::new(one)));
            let Some(first) = all.next() else { return Ok(()) };
            let rest: Vec<String> = all.map(|more| format!("pipe: {}", more.reason())).collect();
            Err(refuse(&first, &rest))
        }
    }
}

/// 契約 file が読めない周の断り（rc 2・理由を全件出す）。
pub(super) fn unloadable(errors: Vec<ContractError>) -> Outcome {
    Outcome::failed(RC_BROKEN, errors.iter().map(ToString::to_string).collect())
}

/// 判定関数 1 本の断り（**先頭の 1 件が理由**・後続行は stderr に並ぶ・§21）。intake は [`Self::outcome`] で従来どおりの
/// rc と stderr で断り、preflight は [`Self::name`] と理由を `refuse=` の行に写す。
#[derive(Clone)]
pub(in crate::pipe) struct Denial {
    /// 理由の名（[`Refuse::as_str`]・[`Refuse`] を持たない断り〔宣言の写し / rules 行 / 置き場の読み〕は材料の名）。
    pub(in crate::pipe) name: &'static str,
    /// 従来の断り（rc は理由が持つ・stderr の行）。
    pub(super) outcome: Outcome,
    /// 型の断りの列（[`Refuse`] の値・契約表の段は finding ごと・型を持たない断りは空＝事前審査は測れないに倒す・設計
    /// dispatcher.md §27 形 3）。
    pub(in crate::pipe) refusals: Vec<Refuse>,
}

/// judge が読む材料（run を作らずに揃う値・intake と preflight が同じ 1 本を撃つ・C2）。
pub(in crate::pipe) struct Material<'a> {
    /// 対象 repo（base = HEAD）。
    pub(in crate::pipe) repo: &'a Path,
    /// 規則の値（上限と余地の行）。
    pub(in crate::pipe) manifest: &'a Manifest,
    /// 読み込み済みの契約 file。
    pub(in crate::pipe) contract: &'a Contract,
    /// 置き場（交差と重複 run の 2 検査だけが読む・`None` = 撃たず overlap は unmeasured・intake は常に `Some`）。
    pub(in crate::pipe) state_dir: Option<&'a Path>,
    /// 1 周ぶんの repo の材料（**呼び手が 1 回読む**・設計 dispatcher.md §5）。
    pub(in crate::pipe) materials: &'a Materials,
    /// 契約の bead id（この秒の run id の材料）。
    pub(in crate::pipe) bead: &'a str,
    /// lock の前に撃った freeze と base の木の実走（§56 形 2・`None` = 列の候補＝judge が freeze を撃ち base は撃たない）。
    pub(in crate::pipe) early: Option<&'a Early>,
}

/// [`judge`] の結果: 事実（§21 の 1 行 1 事実の材料・関数が Err の周はその関数の事実が無い）と断りの列（判定関数 1 本
/// につき高々 1 件・撃った順＝intake が先頭で断る順）。
pub(in crate::pipe) struct Judged {
    /// 設計 pointer の字面と行の § 番号（pointer でない `design` と行の解けない周は `None`）。
    pub(super) design: Option<(String, String)>,
    /// write-set の弁別と本数（`settle_write_set` が Ok で pointer の在る周）。
    pub(super) write_set: Option<(WriteSet, usize)>,
    /// verify の nextest 行ごとの (filter 語, base の歯の file)。
    pub(super) teeth: Vec<(String, Vec<String>)>,
    /// 上限の余地（`exclude_cap_shortfall` が Ok の周）。
    pub(super) headroom: Option<Headrooms>,
    /// live との交差（`exclude_overlap` が Ok の周・置き場が無い周は撃たない）。
    pub(super) overlap: Option<Crossed>,
    /// 断りの列。
    pub(in crate::pipe) denials: Vec<Denial>,
    /// 導出値で写しの write-set を置き換える周の導出値（create が写しに書く）。
    derived: Option<Vec<String>>,
    /// 宣言の有効値（`freeze` が Ok の周・create が写す）。
    effective: Option<Effective>,
    /// この秒の run id（置き場が在る周・重複 run の検査と create が同じ id を読む）。
    run: Option<String>,
}

/// 受付の判定（run を作らない・§21）。判定関数を `freeze` → `settle_write_set` → `exclude_cap_shortfall` →
/// `exclude_entrance_not_red` → `exclude_unresolved_rulings` → `exclude_max_live` → `exclude_overlap` → 重複 run の順に**全部撃ち**、各関数が返した断りを列に積む。前段の Ok 値を取るのは導出値で
/// write-set を置き換える 1 点だけで、`settle_write_set` が Err の周は契約 file の write-set のまま後段を撃つ
/// （Declared 行は元々置き換えが無い＝前段と後段の断りが同時に載る）。git repo でない対象は他の関数が撃てないので
/// `not-a-repo` の 1 件で止まる。
pub(in crate::pipe) fn judge(material: &Material<'_>) -> Judged {
    let Material { repo, manifest, contract, state_dir, bead, materials, early } = *material;
    let mut judged = Judged {
        design: None,
        write_set: None,
        teeth: Vec::new(),
        headroom: None,
        overlap: None,
        denials: Vec::new(),
        derived: None,
        effective: None,
        run: None,
    };
    // base の tracked file の一覧（交差の dir の展開と上限の余地が読む・設計 contract-source.md §3）。
    // **走査は [`Materials::read`] が 1 周に 1 回だけ撃つ**（設計 dispatcher.md §5）ので、ここでは借りるだけ。
    if super::head_of(repo).is_none() {
        judged.denials.push(not_a_repo(repo));
        return judged;
    }
    let tracked = materials.tracked.as_slice();
    // **宣言は上限と突き合わせてから**。ここで断つ周は run dir も event も作らない
    // ——撃てない契約の run が置き場に残ると、続きから引ける便に見えてしまう。lock の前に撃った周は結果を借りる（§56 形 2）。
    match early.map_or_else(|| freeze(repo, manifest, contract), |found| found.frozen.clone()) {
        Ok((found, _)) => judged.effective = Some(found),
        Err(denial) => judged.denials.push(denial),
    }
    // **write-set の弁別は余地と交差より前**（導出値が write-set になる周は、その導出値で余地と交差を測る）。
    let mut measured = contract.clone();
    match settle_write_set(repo, contract, materials) {
        Ok(settled) => {
            judged.write_set = settled.as_ref().map(|found| (found.kind, found.files));
            judged.derived = settled.and_then(|found| found.replaced);
            if let Some(files) = judged.derived.as_deref() {
                measured.write_set = files.to_vec();
            }
        }
        Err(denial) => judged.denials.push(denial),
    }
    row_facts(repo, contract, materials, &mut judged);
    // **同型の停止と焼き直しの門は余地と交差より前**（§23）: 便の履歴で断る周は、余地と交差を測るまでもない。
    exclude_repeats(material, &measured, &mut judged);
    // **上限の余地は受付だけが撃つ**（§3「撃つ場所は受付だけ」）: その便を今の base に当てたら入るか、という
    // 受付時点の事実で、CI の `contracts check` は撃たない（表は履歴を持つ）。
    match exclude_cap_shortfall(manifest, &measured, materials) {
        Ok(found) => judged.headroom = Some(found),
        Err(denial) => judged.denials.push(denial),
    }
    if let Some(denial) = exclude_entrance_not_red(early) {
        judged.denials.push(denial);
    }
    // **裁定 id の引用の判定は entrance-not-red の直後・置き場が要る判定の前**（設計 dispatcher.md §37 約束 1）: 置き場を持たない
    // 呼び手（事前審査・列の候補）でも撃つ。
    if let Some(denial) = exclude_unresolved_rulings(material) {
        judged.denials.push(denial);
    }
    let Some(state_dir) = state_dir else {
        return judged;
    };
    // **同時本数の上限は交差の前**（設計 gate-cost.md §24）。短絡しない＝上限で断る周も交差は撃ち、組は後続に並ぶ。
    // 宣言の同時の数の上限（tsuzuri の判断の記録 ADR-63 の決定 (13)）も同じ段で、同じく短絡しない。
    judged.denials.extend(exclude_max_live(manifest, state_dir, &measured, tracked).err());
    judged.denials.extend(exclude_run_cap(repo, state_dir, &measured).err());
    // **入口で排他する**（ADR-0019 §2.1）。live な便と write-set が交差する契約は、
    // run dir も event も作らずに断る——後段（land の rebase）で衝突を知るより安い。
    match exclude_overlap(material, state_dir, &measured, tracked) {
        Ok(found) => judged.overlap = Some(found),
        Err(denial) => judged.denials.push(denial),
    }
    let id = run_id(bead, &fleet::cli::now_utc());
    // stamp は秒までなので、同じ bead を同じ秒に 2 回 intake すると id が衝突する。
    // 黙って上書きすると **前の便の契約が別物に化ける**ので、何も書かずに断る。
    if run_dir(state_dir, &id).exists() {
        judged.denials.push(refuse(&Refuse::DuplicateRun { run: id.clone() }, &[]));
    }
    judged.run = Some(id);
    judged
}

/// `deny` の名乗りで base の木で撃った結果に base で緑か測れない行が 1 本以上在る周の断り（§56 形 6・余地の判定の後・置き場の
/// 要る判定の前）。結果を持たない judge（列の候補）と他の名乗りは立てない。
fn exclude_entrance_not_red(early: Option<&Early>) -> Option<Denial> {
    let base = early.and_then(Early::denying)?;
    let count = base.not_red();
    (count > 0).then(|| refuse(&Refuse::EntranceNotRed { count, values: base.values() }, &[]))
}

/// ruling-check が true の repo で、契約の設計の節か契約表の行が解けない裁定 id の引用を持つ周（と、台帳か線を読めない周）の断り
/// （設計 dispatcher.md §37・置き場の要る判定の前）。判定は [`ruling`] の素の値で、断りの組み立てはここが書く。
fn exclude_unresolved_rulings(material: &Material<'_>) -> Option<Denial> {
    let Material { repo, manifest, contract, materials, .. } = *material;
    let (section, row, unmeasured) = match materials.ruling.judge_contract(repo, timeout_of(manifest), contract) {
        Ruled::Clear => return None,
        Ruled::Unresolved { section, row } => (section, row, None),
        Ruled::Unmeasured(word) => (Vec::new(), Vec::new(), Some(word.to_owned())),
    };
    Some(refuse(&Refuse::RulingUnresolved { section, row, unmeasured }, &[]))
}

/// 便を作る（run dir・写し・event）。judge の断りが 1 件でも在れば**先頭の 1 件**で断り、何も書かない（従来の外形）。
fn create(
    judged: Judged,
    material: &Material<'_>,
    state_dir: &Path,
    body: &str,
    policy: LockPolicy,
) -> Result<Intaken, Outcome> {
    let Judged { write_set, denials, derived, effective, run, overlap, .. } = judged;
    if let Some(first) = denials.into_iter().next() {
        return Err(first.outcome);
    }
    // 断り 0 の周は freeze が Ok（有効値が在る）で、置き場 `Some` の judge は run id を持つ。
    let (Some(effective), Some(id)) = (effective, run) else {
        return Err(broken("受付の判定が有効値と run id を持たない".to_owned()));
    };
    write_contract(state_dir, &id, body, derived.as_deref()).map_err(broken)?;
    copy_vessel(state_dir, &id, &effective).map_err(broken)?;
    remember_repo(state_dir, &id, material.repo).map_err(broken)?;
    let base = material.early.and_then(|found| found.base.as_ref());
    if let Some(found) = base {
        found.write(&run_dir(state_dir, &id)).map_err(broken)?;
    }
    let emitted = emit(
        state_dir,
        &Emit {
            kind: EventKind::RunCreated,
            run: &id,
            bead: material.bead,
            stage: Some(Stage::Intake),
            seat: None,
            pid: None,
            detail: Some(format!("classes:{}", material.contract.classes.join("+"))),
        },
        policy,
    );
    let commuted = overlap.and_then(|found| found.commuted);
    match emitted.and_then(|()| commute::record(state_dir, &id, material.bead, commuted.as_ref(), policy)) {
        Err(err) => Err(broken(err.to_string())),
        Ok(()) => Ok(Intaken {
            id,
            write_set,
            entrance: base.map(BaseRun::fact),
            index: material.materials.index_tail(&material.contract.touches),
        }),
    }
}

/// 上限の余地の事実（[`exclude_cap_shortfall`] が通った周・§21 の `headroom=` の材料）。
pub(super) struct Headrooms {
    /// write-set の `.rs`（dir は配下に展開・`+` の新規 file は 0 行・`-` / `~` / `=` は余地を求めない）ごとの余地
    /// （R-C4-2 の値 − base の行数）・余地の小さい順（同じ余地は path の辞書順）。
    pub(super) rooms: Vec<(String, u64)>,
    /// 上限の値と契約の `size` の見積（行・rules 行 `pipe.size_<s|m|l>_lines` の値）。
    caps: declaration::Caps,
    /// 契約の growth を読んだ file ごとの見込み（path と行数・設計 contract-source.md §46）。
    growth: Vec<(String, u64)>,
}

impl Headrooms {
    /// file の見込み（growth に在ればその値・無ければ `size` の見積＝受付の判定と同じ [`declaration::Caps::estimate`]）。
    pub(super) fn estimate(&self, file: &str) -> u64 {
        self.caps.estimate(file, &self.growth)
    }
}

/// 対象 repo の HEAD から vessel 宣言を読み、器の上限と突き合わせて有効値にする。
///
/// **外れは rc 1**（前提違反）で、宣言が読めない周も同じ極性である——「宣言が無い」と
/// 「宣言が壊れている」で扱いを変えると、器の視野の外の verify 行が片方から入る。有効値に宣言の名乗りを添える（§56 形 2）。
fn freeze(repo: &Path, manifest: &Manifest, contract: &Contract) -> Result<(Effective, Option<EntranceFlip>), Denial> {
    let rows = ceiling_of(manifest)?;
    declaration::measure_named(repo, &rows.borrow(), &contract.verify).map_err(|errors| {
        denied(DENIAL_DECLARATION, Outcome::failed(RC_REFUSED, errors.iter().map(ToString::to_string).collect()))
    })
}

/// 有効値を便の写し面へ凍結する（以後の段は repo の宣言を読み直さない）。
fn copy_vessel(state_dir: &Path, id: &str, effective: &Effective) -> Result<(), String> {
    let path = vessel_path(state_dir, id);
    std::fs::write(&path, effective.render())
        .map_err(|err| format!("{} を書けない: {err}", path.display()))
}

/// 便の対象 repo を写し面へ書き留める（現在地を cwd に依らせない）。
fn remember_repo(state_dir: &Path, id: &str, repo: &Path) -> Result<(), String> {
    let path = super::repo_path(state_dir, id);
    std::fs::write(&path, format!("{}\n", repo.display()))
        .map_err(|err| format!("{} を書けない: {err}", path.display()))
}

/// 便の repo。`--repo` が上書きし（読み手は [`repo_flag`] の 1 本・絶対 path）、無ければ写し面から解く。写し面も
/// 無い周は [`repo_of`] の flag 不在の断り（cwd を読まない・設計 pipeline.md §15）。
pub(super) fn run_repo(args: &[String], state_dir: &Path, id: &str) -> Result<PathBuf, String> {
    if let Some(found) = repo_flag(args)? {
        return Ok(found);
    }
    match super::repo_of_run(state_dir, id) {
        Some(found) => Ok(found),
        None => repo_of(args),
    }
}

/// 契約 file を置き場へ写す（process 間で持ち越す面は event log とこの写しだけ）。導出値が write-set になる周
/// （`derived`）は写しの `write-set` の行だけをその値に差し替える（他の行は逐語・runner が読むのは写しの write-set）。
fn write_contract(state_dir: &Path, id: &str, body: &str, derived: Option<&[String]>) -> Result<(), String> {
    let dir = run_dir(state_dir, id);
    std::fs::create_dir_all(&dir).map_err(|err| format!("{} を作れない: {err}", dir.display()))?;
    let to = contract_path(state_dir, id);
    let text = match derived {
        Some(files) => with_write_set(body, files),
        None => body.to_owned(),
    };
    std::fs::write(&to, text).map_err(|err| format!("{} を書けない: {err}", to.display()))
}

/// 契約 file の本文の `write-set` の行を `files` の列に差し替える（契約 file は 1 行 1 key・配列は 1 行に収まる）。
fn with_write_set(text: &str, files: &[String]) -> String {
    let quoted: Vec<String> = files.iter().map(|item| format!("\"{item}\"")).collect();
    let line = format!("write-set = [{}]", quoted.join(", "));
    let mut out = String::new();
    for found in text.lines() {
        let key = found.trim().split_once('=').map(|(key, _)| key.trim());
        out.push_str(if key == Some("write-set") { line.as_str() } else { found });
        out.push('\n');
    }
    out
}

#[cfg(test)]
use refusal::{ROW_FILE_LINES, ROW_SIZE_S};
#[cfg(test)]
mod tests {
    // flip-check: moved s2-07l.736.12
    // flip-check: retroactive s2-07l.736.28
    use super::ruling::Rulings;
    use super::{
        exclude_cap_shortfall, exclude_unmeasured, int_row, with_write_set, ContractRow, Entrance, Materials, WriteSet,
        ENTRANCE_LOCK, ROW_FILE_LINES, ROW_SIZE_S, WRITE_SETS,
    };
    use std::path::Path;
    use crate::fleet::store::LockPolicy;
    use crate::order::is_declaration_order;
    use crate::pipe::closure::Source;
    use crate::pipe::contract::Contract;
    use crate::pipe::declaration::TableFacts;
    use crate::pipe::dispatch::index_build::Status;
    use crate::rules::manifest::Manifest;
    use std::collections::BTreeSet;
    use std::sync::Arc;

    /// 受付の入口は**同時に 1 つしか通さない**（[`judge`] と [`create`] を 1 周として閉じる・ADR-0019 §2.1）。
    ///
    /// 契機が重なると 2 つの受付が同時に来る（`s2-07l.366`）。入口を握れていなければ、どちらも「live な
    /// 便は無い」と読んでから両方が run を作る。**握っている間は 2 つ目が取れない・外せば取れる**を
    /// 決定的に測る（同時撃ちの e2e は競合の再現が確率的なので、不変条件はここで固定する）。
    #[test]
    fn pipe_terminal_dispatch_entrance_admits_one_holder_at_a_time() {
        let state = std::env::temp_dir().join(format!("s2-entrance-{}", std::process::id()));
        let policy = LockPolicy { retry_ms: 60, stale_ms: 60_000 };
        let first = Entrance::hold(&state, policy).map_err(|out| out.rc);
        assert!(first.is_ok(), "1 つ目は通る（rc {:?}）", first.as_ref().err());
        let lock = state.join(ENTRANCE_LOCK);
        assert!(lock.exists(), "握っている間は lock file が在る");
        let second = Entrance::hold(&state, policy).map_err(|out| out.rc);
        assert!(second.is_err(), "握っている間は 2 つ目が取れない（母集団 2 回の取得）");
        drop(first);
        assert!(!lock.exists(), "外すと lock file が消える");
        let third = Entrance::hold(&state, policy).map_err(|out| out.rc);
        assert!(third.is_ok(), "外れた後は取れる（rc {:?}）", third.as_ref().err());
        drop(third);
        std::fs::remove_dir_all(&state).ok();
    }

    /// 弁別は閉じた 3 値で、const slice は宣言順・`as_str` は判定行の token（`derived` / `declared` / `promised`・§33 の
    /// Promised は宣言順の末尾）。
    #[test]
    fn contract_derive_write_set_kinds_are_pinned_in_declaration_order() {
        assert_eq!(WRITE_SETS, [WriteSet::Derived, WriteSet::Declared, WriteSet::Promised], "母集団 3 値");
        assert!(is_declaration_order(WRITE_SETS, |kind| kind as usize), "宣言順");
        let names: Vec<&str> = WRITE_SETS.iter().map(|kind| kind.as_str()).collect();
        assert_eq!(names, ["derived", "declared", "promised"], "判定行の token");
    }

    /// 写しの差し替えは `write-set` の行だけ（他の行は逐語・key の前後の空白も同じ key と読む・新規 file の `+` は
    /// そのまま載る）。
    #[test]
    fn contract_derive_copy_replaces_only_the_write_set_line() {
        let text = "goal = \"g\"\nwrite-set = [\"src/lib.rs\"]\n  write-set  = [\"x\"]\nverify = [\"git status\"]\n";
        let files = ["+src/new.rs".to_owned(), "src/a.rs".to_owned()];
        let want = "goal = \"g\"\nwrite-set = [\"+src/new.rs\", \"src/a.rs\"]\nwrite-set = [\"+src/new.rs\", \"src/a.rs\"]\nverify = [\"git status\"]\n";
        assert_eq!(with_write_set(text, &files), want);
    }

    /// 余地の段の除外（§31 (a)）の fixture: 解ける `crates/toy/src/full.rs`（`lines` 行）と base に無い素の
    /// `crates/toy/src/none.rs` の 2 項目の write-set・S の契約・材料。
    fn short_of_room(lines: u64) -> (Contract, Materials) {
        let full = "crates/toy/src/full.rs";
        let contract = Contract {
            goal: "g".to_owned(),
            done: "d".to_owned(),
            size: "S".to_owned(),
            owner: "s2-x".to_owned(),
            disposition: "A-now".to_owned(),
            write_set: vec![full.to_owned(), "crates/toy/src/none.rs".to_owned()],
            verify: vec!["git status".to_owned()],
            req: vec!["FR1".to_owned()],
            design: "docs/design/toy.md".to_owned(),
            classes: Vec::new(),
            opens: Vec::new(),
            touches: Vec::new(),
            growth: Vec::new(),
            patch: None,
        };
        let body = "x\n".repeat(usize::try_from(lines).unwrap_or_default());
        let materials = Materials {
            tracked: vec![full.to_owned()],
            sources: vec![Source { path: full.to_owned(), body: Ok(body) }],
            snapshots: Vec::new(),
            facts: TableFacts { allowed: Vec::new(), denied: Vec::new(), requirements: String::new(), layout: crate::pipe::closure::CrateLayout::bare(vec!["crates/".to_owned()]), teeth_check: false },
            requirements: Ok(BTreeSet::new()),
            declared: Ok(Vec::new()),
            places: Some(Vec::new()),
            classes: Vec::new(),
            ruling: Rulings::new("bd"),
            index: None,
            index_at: None,
        };
        (contract, materials)
    }

    /// 欄 `code-facts` を 1 要素だけ持つ行（欄は最小）。
    fn fielded_row() -> ContractRow {
        ContractRow { id: "a".to_owned(), code_facts: vec!["refs:crate::x::Y=1".to_owned()], ..ContractRow::default() }
    }

    /// (i) 欄 `code-facts` を持つ行の判定は、ready を載せた材料の forecast の写しと状態を載せない材料では `code-facts-unmeasured` と語 `none`、元の ready の
    /// 材料では列の数えへ進む（索引が空で実測 0 件・名乗り 1 と違う `code-facts`）。
    #[test]
    fn pipe_intake_code_facts_stateless_copy_and_missing_state_name_none_while_ready_goes_on_to_count() {
        let (_, materials) = short_of_room(1);
        let repo = Path::new("/nonexistent-repo-for-code-facts");
        let row = fielded_row();
        let ready = Materials { index: Some(Arc::new(Status::Ready(Vec::new()))), index_at: Some("deadbeef".to_owned()), ..materials.clone() };
        for (label, stateless) in [("forecast の写し", ready.forecast(&[], &[], &[])), ("状態を載せない材料", materials)] {
            let denied = exclude_unmeasured(&row, &stateless, repo).err();
            assert_eq!(denied.as_ref().map(|found| found.name), Some("code-facts-unmeasured"), "{label}: 測れない");
            let line = denied.map(|found| found.outcome.err.join("\n")).unwrap_or_default();
            assert!(line.contains("refs:crate::x::Y=1") && line.contains("none"), "{label}: 要素と語 none を名指す: {line}");
        }
        let went_on = exclude_unmeasured(&row, &ready, repo).err();
        assert_eq!(went_on.as_ref().map(|found| found.name), Some("code-facts"), "元の ready の材料は列の数えへ進む");
        let line = went_on.map(|found| found.outcome.err.join("\n")).unwrap_or_default();
        assert!(line.contains("名乗り 1") && line.contains("実測 0"), "名乗りと実測を名指す: {line}");
        let plain = ContractRow { code_facts: Vec::new(), ..row };
        assert!(exclude_unmeasured(&plain, &ready, repo).is_ok(), "欄を持たない行はこの判定の外");
    }

    /// (h) 予想の base の写し（`forecast`）は base の索引の状態を持ち越さず状態なしで、元の材料の状態は変わらない（状態を載せる
    /// `indexed` を撃たず ready の状態を直に置く・写しは tracked と本文の置き換えだけを受ける）。
    #[test]
    fn pipe_intake_index_forecast_copy_drops_the_state_and_leaves_the_original_ready() {
        let (_, materials) = short_of_room(1);
        let ready = Materials { index: Some(Arc::new(Status::Ready(Vec::new()))), ..materials };
        let added = ["crates/toy/src/added.rs".to_owned()];
        let copy = ready.forecast(&added, &[], &[]);
        assert!(copy.index.is_none(), "写しは状態なし");
        assert!(matches!(ready.index.as_deref(), Some(Status::Ready(rows)) if rows.is_empty()), "元の材料は ready のまま");
        assert!(copy.tracked.contains(&added[0]) && !ready.tracked.contains(&added[0]), "写しは tracked を重ね、元は動かない");
    }

    /// (a) 項目の解決に失敗した周は「解けない項目を**除いた**列」で数え直す: 余地の在る full.rs は通って余地の列に
    /// 1 本だけ載り、満杯の full.rs は `cap-headroom` で断られる（`!` を落とすと解けない none.rs だけで数え直し、列が
    /// 空になって満杯の file が通る）。
    #[test]
    fn contract_closure_ext_survivor_a_cap_shortfall_recounts_without_the_unresolved_items() {
        // flip-check: retroactive s2-07l.277
        let Ok(manifest) = Manifest::embedded() else {
            panic!("埋め込み manifest を読める");
        };
        let (cap, size) = (int_row(&manifest, ROW_FILE_LINES), int_row(&manifest, ROW_SIZE_S));
        let (cap, size) = (cap.unwrap_or_default(), size.unwrap_or_default());
        assert!(cap > size && size > 0, "rules 行の値が在る（R-C4-2 {cap} > S {size} > 0）");
        // 余地は境界（(c) の歯の側）から離して置く＝見積の 2 倍と 0。
        let room = size.saturating_mul(2);
        let (contract, materials) = short_of_room(cap.saturating_sub(room));
        let fits = exclude_cap_shortfall(&manifest, &contract, &materials).map(|found| found.rooms);
        let rooms = fits.as_ref().map(Vec::len).unwrap_or_default();
        assert_eq!(
            fits.as_ref().ok(),
            Some(&vec![("crates/toy/src/full.rs".to_owned(), room)]),
            "余地の在る full.rs は通り、解ける項目だけが余地の列に載る（件数 {rooms} / 母集団 write-set 2 項目・解ける 1 項目）"
        );
        let (contract, materials) = short_of_room(cap);
        let short = exclude_cap_shortfall(&manifest, &contract, &materials).err().map(|denial| denial.name);
        assert_eq!(
            short,
            Some("cap-headroom"),
            "余地 0 の full.rs は解ける項目で断る（断り {} 件 / 母集団 write-set 2 項目・解ける 1 項目）",
            usize::from(short.is_some())
        );
    }
}
