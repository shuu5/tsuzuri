//! `pipe preflight` — 受付と同じ判定を run を作らず撃ち、契約の実態突合を planner が edit time に測る口。
//! 判定は受付の [`super::intake::judge`] の **同じ 1 本**で、断りを最初の 1 件で止めず**全部**並べる。
//! `--design` も `--contract` も渡さない周は bead の周で、台帳を 1 回読み。
//! stdout は **1 行 1 事実**: `design=<doc>#<id> section=<n>` / `done-teeth=<present|absent>`。
//! 出所: contract-source.md §21 §46 pipeline.md §56 判断の記録 ADR-44

use super::base_run::BaseRun;
use super::intake::bead::{bead_contract, ledger_of, source_of, Source};
use super::intake::{ceiling_of, early, generated, generated_from, judge, Denial, Judged, Material, Materials};
use super::{flag, need, present, refused, repo_flag, state_dir_of, REPO_FLAG};
use crate::cli_outcome::{Outcome, RC_BROKEN, RC_OK, RC_REFUSED};
use crate::pipe::bead::{copy_text, form_of, Form};
use crate::pipe::closure::filter_words;
use crate::pipe::contract::Contract;
use crate::pipe::dispatch::pointer_of;
use crate::pipe::refuse::{covered, Refuse};
use crate::pipe::review::{design_material, done_items, section_text};
use crate::pipe::spawn::bead_rows::{ledger_rows, merged, Beads, LedgerRead};
use crate::pipe::spawn::table_rows;
use crate::pipe::table::TableError;
use crate::pipe::{show_head, table};
use crate::rules::manifest::Manifest;
use crate::seat::ledger::{one_read, read_ledger, timeout_of, Issue, DEFAULT_BD};
use std::path::{Path, PathBuf};

/// 末尾の判定行の書き出し。
const TAIL: &str = "preflight:";

/// 閉包の広がりの予想の行の書き出し。
const WIDEN: &str = "widen=";

/// 置き場が無く交差を測れなかった周の行。
const UNMEASURED: &str = "overlap=unmeasured";

/// 空の file の列の字面（交差 0・歯の file 0）。
const NONE: &str = "-";

/// `--bead` の bead を行に置いた bead と名乗り、その acceptance を照らす flag（値なし・在る周だけ台帳を読む）。
pub(super) const PLACED: &str = "--placed";

/// 照らしの断りの名（preflight だけの名・受付の [`crate::pipe::refuse::Refuse`] の外）。
const ACCEPTANCE: &str = "acceptance-pointer";

/// verify の欄の照らしの断りの名（preflight だけの名）: nextest の行が filter 語の候補を 2 つ以上持つ。
const FILTERS: &str = "verify-filters";

/// verify の欄の照らしの断りの名（preflight だけの名）: 行が宣言の common-verify の行と同じ。
const COMMON: &str = "verify-common";

/// 照らしの断りの名（preflight だけの名）: done の項の印の字が審査の材料の節に無い。
const LITERAL: &str = "section-literal";

/// done の項が節の字を囲む印の始まり（全角の二重鉤括弧の始まり）。
const MARK_OPEN: char = '『';

/// 印の終わり（全角の二重鉤括弧の終わり）。
const MARK_CLOSE: char = '』';

/// 置いた bead の acceptance の照らし（閉じた 3 値）。
#[derive(Debug, Clone, PartialEq, Eq)]
enum Accepted {
    /// acceptance の `design = ` の行が `--design` と同じ置き場と行 id を指す。
    Matched,
    /// 断る（理由の字）。
    Refused(String),
    /// 台帳を読めない（語）。
    Unmeasured(&'static str),
}

/// 台帳の issue の列から `bead` を id で引き、acceptance を dispatch の列と同じ読み手（[`pointer_of`]）で読んで `pointer` と照らす。
fn accepted(issues: &[Issue], bead: &str, pointer: &table::Pointer) -> Accepted {
    let design = format!("{}#{}", pointer.path, pointer.id);
    let Some(issue) = issues.iter().find(|issue| issue.id == bead) else {
        return Accepted::Refused(format!("bead {bead} が台帳に無い — 行に置いた bead の id を --bead に渡す"));
    };
    let Some(found) = pointer_of(&issue.acceptance) else {
        return Accepted::Refused(format!(
            "bead {bead} の acceptance に design = の行が無い — bdw update {bead} --acceptance 'design = {design}' で足す（dispatch は acceptance の design = の行だけを契約の pointer と読む）"
        ));
    };
    if found != *pointer {
        return Accepted::Refused(format!(
            "bead {bead} の acceptance は design = {}#{} で --design {design} と違う — 行に合う bead を渡すか acceptance を直す",
            found.path, found.id
        ));
    }
    Accepted::Matched
}

/// `--placed` の周の照らし（rules の待ち上限の行が無い周と台帳を読めない周は `Unmeasured`）。台帳は `--repo` を作業 dir に `bd` で読む
/// （裁定 id の判定と同じ引数・[`one_read`] の区間の中で 1 回の出力を分ける）。
fn acceptance_of(bd: &str, repo: &Path, manifest: &Manifest, bead: &str, pointer: &table::Pointer) -> Accepted {
    let read = timeout_of(manifest).and_then(|timeout| read_ledger(bd, repo, timeout).ok());
    read.map_or(Accepted::Unmeasured("ledger"), |issues| accepted(&issues, bead, pointer))
}

/// 照らしの行の対（事実の行・断りの行）。
fn acceptance_lines(found: Option<&Accepted>) -> (Option<String>, Option<String>) {
    match found {
        None => (None, None),
        Some(Accepted::Matched) => (Some("acceptance=matched".to_owned()), None),
        Some(Accepted::Unmeasured(word)) => (Some(format!("acceptance=unmeasured:{word}")), None),
        Some(Accepted::Refused(why)) => (None, Some(format!("refuse={ACCEPTANCE}:{why}"))),
    }
}

/// verify の欄の照らし（受付の judge の外・契約の verify を 1 行ずつ）: 読み手 [`filter_words`] が filter 語の候補を 2 つ以上返す nextest の
/// 行（読み手の [`crate::pipe::closure`] は最後の 1 語だけを filter 語に読み、前の語の歯を置き場と審査の対応の表に数えない）と、宣言の共通
/// verify `common` の行と空白の並びの外で同じ行（gate は共通 verify を verify の行の前に撃つので同じ木で 2 度撃つ）を、断りの 1 行
/// （`refuse=<名>:<理由>`）ずつにする。`design` は行の pointer の字。
fn verify_gaps(design: &str, verify: &[String], common: &[String]) -> Vec<String> {
    let same = |line: &str, other: &str| line.split_whitespace().eq(other.split_whitespace());
    let mut found = Vec::new();
    for line in verify {
        let words = filter_words(line);
        if let [.., last] = words.as_slice() {
            if words.len() > 1 {
                found.push(format!(
                    "refuse={FILTERS}:行 {design} の verify {line:?} は nextest の filter の語を {} つ持つ（{}）— 器は最後の語 {last} だけを filter に読み、前の語の歯を置き場と審査の対応の表に数えない。1 行に filter の語を 1 つずつ割る",
                    words.len(),
                    words.join(" ")
                ));
            }
        }
        if common.iter().any(|other| same(line, other)) {
            found.push(format!(
                "refuse={COMMON}:行 {design} の verify {line:?} は宣言の common-verify の行と同じ — gate は共通 verify を verify の行の前に撃つので同じ木で 2 度撃つ。行の verify から外す"
            ));
        }
    }
    found
}

/// done の項の印の照らし（受付の judge の外）: 項（[`done_items`]・列が空なら done の全体）を前から番号 1, 2, … で見て、各項の印 `『…』` の字が審査の
/// 材料 `material` の部分の字に無い印と、空の印と、閉じない印を、断りの 1 行（`refuse=section-literal:<理由>`）ずつにする。行は項の順と項の中の出る順。
/// `design` は行の pointer の字。
fn literal_gaps(design: &str, done: &str, material: &str) -> Vec<String> {
    let mut items = done_items(done);
    if items.is_empty() {
        items.push(done.trim().to_owned());
    }
    let mut found = Vec::new();
    for (at, item) in items.iter().enumerate() {
        let head = format!("refuse={LITERAL}:行 {design} の done の項 {} の", at.saturating_add(1));
        let mut rest = item.as_str();
        while let Some((_, after)) = rest.split_once(MARK_OPEN) {
            let Some((literal, next)) = after.split_once(MARK_CLOSE) else {
                found.push(format!("{head}印 {MARK_OPEN} が閉じない — {MARK_CLOSE} で閉じる"));
                break;
            };
            if literal.is_empty() {
                found.push(format!("{head}印 {MARK_OPEN}{MARK_CLOSE} が空 — 印の中に節の字を書く"));
            } else if !material.contains(literal) {
                found.push(format!("{head}字 {MARK_OPEN}{literal}{MARK_CLOSE} が審査の材料の節に無い — 節の本文に同じ字を書くか印を外す"));
            }
            rest = next;
        }
    }
    found
}

/// `pipe preflight`: judge だけを撃ち、事実と断りを stdout に並べる（台帳の読みは 1 回の出力を分ける区間の中）。
pub(super) fn preflight(args: &[String], manifest: &Manifest) -> Outcome {
    one_read(|| checked(args, manifest))
}

/// 契約の file を直に渡す flag（`--design` と同時には渡せない・値は `.toml` の 1 行の file）。
const CONTRACT: &str = "--contract";

/// `--contract` の周の引数の読み（`--design` と `--contract` は 1 つ・`--placed` は `--design` と・`.toml`・読めること・行が 1 つ、の順で断る）。
/// 戻りは pointer（path は file の絶対 path）・bead・repo・file の字。行の読めない周は後段の [`generated_from`] の断りに任せる。
fn read_contract_args(args: &[String], given: &str) -> Result<(table::Pointer, String, PathBuf, String), Outcome> {
    if present(args, "--design") {
        return Err(refused(format!("--design と {CONTRACT} は 1 つだけ渡す")));
    }
    if present(args, PLACED) {
        return Err(refused(format!("{PLACED} は --design と使う")));
    }
    let not_toml = || refused(format!("{CONTRACT} {given} は .toml の file でない"));
    if !given.ends_with(".toml") {
        return Err(not_toml());
    }
    let bead = need(args, "--bead").map_err(refused)?.to_owned();
    let repo = repo_flag(args).and_then(|found| found.ok_or(format!("{REPO_FLAG} が要る"))).map_err(refused)?;
    let cwd = std::env::current_dir().map_err(|err| refused(format!("作業 dir を読めない: {err}")))?;
    let path = cwd.join(given).display().to_string();
    let text = std::fs::read_to_string(&path).map_err(|err| {
        let found = Refuse::ContractTable(TableError::Unreadable { line: 0, reason: format!("{path} を読めない: {err}") });
        let outcome = Outcome::failed(found.rc(), vec![format!("pipe: {}", found.reason())]);
        tailed(Denial { name: found.as_str(), outcome, refusals: vec![found] })
    })?;
    let rows = table::read_rows(&path, &text).ok();
    if let Some(found) = rows.as_deref().filter(|found| found.len() != 1) {
        return Err(refused(format!("契約の file は [[contract]] の行を 1 つだけ持つ（{} 行）", found.len())));
    }
    let id = rows.and_then(|found| found.into_iter().next()).map_or_else(|| "-".to_owned(), |row| row.id);
    let pointer = table::parse_pointer(&format!("{path}#{id}")).map_err(|_| not_toml())?;
    Ok((pointer, bead, repo, text))
}

/// 契約の出所（引数の読みの結果）。
enum Given {
    /// 設計 pointer（`--design`・`--contract` の file の字を持つ周はその字・pointer の path は file の絶対 path）。
    Pointer(table::Pointer, Option<String>),
    /// bead の周（`--design` も `--contract` も無い周）で、写しを書く置き場。
    Bead(PathBuf),
}

/// bead の周の引数の照らし: `--placed` は断り（`--contract` の周と同じ字）、置き場は写しを書くので解けない周は受付と同じ `state_dir_of` の
/// 理由の字で断る（台帳を読まず写しを書かない）。
fn given_of(args: &[String], source: Source) -> Result<Given, Outcome> {
    match source {
        Source::Design(pointer) => Ok(Given::Pointer(pointer, None)),
        Source::Bead if present(args, PLACED) => Err(refused(format!("{PLACED} は --design と使う"))),
        Source::Bead => state_dir_of(args).map(Given::Bead).map_err(refused),
    }
}

/// 組んだ契約と本文（組めない周は断り）。
type Built = Result<(Contract, String), Denial>;

/// 引数の読み（契約の出所・bead・repo）。`--contract` の周と bead の周と `--design` の周を分ける。
fn read_given(args: &[String]) -> Result<(Given, String, PathBuf), Outcome> {
    match flag(args, CONTRACT) {
        Ok(Some(given)) => read_contract_args(args, given).map(|(pointer, bead, repo, text)| (Given::Pointer(pointer, Some(text)), bead, repo)),
        Ok(None) => source_of(args)
            .map_err(|denial| denial.outcome)
            .and_then(|(source, bead, repo)| given_of(args, source).map(|given| (given, bead, repo))),
        Err(reason) => Err(refused(reason)),
    }
}

/// 受付と**同じ 1 本**で契約を組む（C2）。戻りは契約・pointer（bead の周は `None`）・widen の本文の字（`--contract` の周と bead の形の bead だけ）。
fn build(given: Given, (repo, bd): (&Path, &str), manifest: &Manifest, bead: &str, materials: &Materials) -> (Built, Option<table::Pointer>, Option<String>) {
    match given {
        Given::Pointer(pointer, Some(text)) => (generated_from(repo, &pointer, &text, materials), Some(pointer), Some(text)),
        Given::Pointer(pointer, None) => (generated(repo, &pointer, materials), Some(pointer), None),
        Given::Bead(dir) => {
            let (built, text) = from_ledger(bd, (repo, &dir), manifest, bead, materials);
            (built, None, text)
        }
    }
}

/// bead の周: 台帳を 1 回読み、受付と同じ 1 本 [`bead_contract`] で契約を組む。widen の本文の字は、bead の形の bead では写しの字・
/// Design の形の bead では `None`（呼び手が `--design` の周と同じく base の木の doc を読む）。
fn from_ledger(bd: &str, (repo, state_dir): (&Path, &Path), manifest: &Manifest, bead: &str, materials: &Materials) -> (Built, Option<String>) {
    let issues = match ledger_of(bd, repo, manifest, bead) {
        Ok(found) => found,
        Err(denial) => return (Err(denial), None),
    };
    let held = issues.iter().find(|issue| issue.id == bead).filter(|issue| matches!(form_of(&issue.acceptance), Form::Bead));
    let text = held.and_then(|issue| copy_text(bead, &issue.acceptance, &issue.description).ok());
    (bead_contract((repo, state_dir), manifest, bead, &issues, materials), text)
}

/// preflight の本体。
fn checked(args: &[String], manifest: &Manifest) -> Outcome {
    let (given, bead, repo) = match read_given(args) {
        Ok(found) => found,
        Err(outcome) => return outcome,
    };
    // 受付と同じ順（§56 形 2）: HEAD の sha を材料の読みの前に読む。読めない repo は judge が `not-a-repo` で断る。
    let sha = super::head_of(&repo).unwrap_or_default();
    let ceiling = match ceiling_of(manifest) {
        Ok(found) => found,
        Err(denial) => return denial.outcome,
    };
    // 受付と**同じ 1 本**で base の行から契約を組む（C2）。材料を読めない・行を引けない・表の検査に落ちる
    // 周は判定の対象が揃わないので、受付と同じ断りをそのまま返して末尾に判定行を積む（0 件と混ぜない）。
    // **repo の材料の読みは 1 回**（設計 dispatcher.md §5）。生成も判定も同じ 1 つを借りる。材料の読みの
    // 断り（tracked を読めない・宣言が上限に外れる）は、`s2-07l.366` の前は [`generated`] の中で立って
    // いた＝**末尾の判定行は同じ 1 本で積む**（C2・積み忘れると「対象が揃わなかった周」だけ末尾を失う）。
    // 台帳の client は受付と同じ（`--bd` を明示した周はその名・無ければ既定の名・読むのは引用を持つ契約が出たときだけ）。
    let bd = match flag(args, "--bd") {
        Ok(found) => found.unwrap_or(DEFAULT_BD),
        Err(reason) => return refused(reason),
    };
    // 置き場は交差と重複 run の 2 検査と base の木の置き場と索引の状態の読みにだけ要る。解けない周は断りでなく `overlap=unmeasured`
    // （base の木を撃つ名乗りの周は全行が測れない）で、索引の状態は載せない（状態なし＝字面の閉包だけ）。
    let state_dir = match &given {
        Given::Bead(dir) => Some(dir.clone()),
        Given::Pointer(..) => state_dir_of(args).ok(),
    };
    let materials = match Materials::read(&repo, &ceiling.borrow(), bd) {
        Ok(found) => match state_dir.as_deref() {
            Some(dir) => found.indexed(dir, &repo, &sha),
            None => found,
        },
        Err(denial) => return tailed(denial),
    };
    let (built, pointer, file) = build(given, (&repo, bd), manifest, &bead, &materials);
    let (contract, teeth) = match built {
        Ok((found, body)) => (found, !crate::pipe::contract::done_teeth_in(&body).is_empty()),
        Err(denial) => return tailed(denial),
    };
    let index = materials.index_tail(&contract.touches);
    let early = early(&repo, manifest, &contract, state_dir.as_deref(), &sha);
    let material = Material {
        repo: &repo,
        manifest,
        contract: &contract,
        state_dir: state_dir.as_deref(),
        bead: &bead,
        materials: &materials,
        early: Some(&early),
    };
    let entrance = early.base.as_ref().map(BaseRun::fact);
    let judged = judge(&material);
    // bead の周の pointer は組んだ契約の design（bead の形は写しの path・Design の形は表の pointer）から読む。
    let own = pointer.or_else(|| table::parse_pointer(&contract.design).ok());
    let doc = file.or_else(|| own.as_ref().and_then(|found| show_head(&repo, &found.path)));
    let beads = state_dir.as_deref().map_or(Beads::Off, |dir| ledger_rows(dir, &repo, LedgerRead { bd, timeout: timeout_of(manifest) }));
    let widen = own.as_ref().map_or_else(
        || vec![format!("{WIDEN}unmeasured:{} を読めない", contract.design)],
        |found| widen_lines(&repo, &sha, found, doc, (&contract.write_set, &beads)),
    );
    let placed = own.as_ref().filter(|_| present(args, PLACED)).map(|found| acceptance_of(bd, &repo, manifest, &bead, found));
    let common = early.frozen.as_ref().map_or(&[][..], |(found, _)| found.common_verify());
    let design = contract.design.clone();
    let mut gaps = verify_gaps(&design, &contract.verify, common);
    gaps.extend(literal_gaps(&design, &contract.done, &design_material(&repo, &design)));
    render(&judged, state_dir.is_some(), (entrance, index), (widen, teeth), (placed.as_ref(), &gaps))
}

/// `widen=<項目>@<doc>#<行 id>:<file,…>`（設計 reverse-index.md の閉包の広がりの予想）: 自分の行の § の本文が語として名指す型形の項目を
/// touches に持つほかの行で、write-set が自分の write-set の .rs の候補〔接頭辞が無いか `+`・`+` は剥がす〕を覆わない組。HEAD の木の
/// 契約表を読めない周は `widen=unmeasured:<理由>` の 1 行（理由は「ほかの行の touches」節と同じ字）。`doc` は自分の行を持つ doc の字
/// （`--design` の周は base の読み・`--contract` の周は file の字）で、本文は行が goal を持てば goal・無ければ節の本文。`own` は自分の
/// write-set と台帳の契約の bead の行（[`merged`] が表の行の後に足す・台帳を読めない周は並べた行の後に
/// `widen=unmeasured:契約の bead の行を読めない（<理由>）` の 1 行を足す）。
fn widen_lines(repo: &Path, sha: &str, pointer: &table::Pointer, doc: Option<String>, own: (&[String], &Beads)) -> Vec<String> {
    let (own, beads) = own;
    let unmeasured = |reason: String| vec![format!("{WIDEN}unmeasured:{reason}")];
    let rows = match table_rows(repo, sha) {
        Ok(found) => found,
        Err(reason) => return unmeasured(reason),
    };
    let (rows, unread) = merged(rows, beads, &format!("{}#{}", pointer.path, pointer.id));
    let body = doc.and_then(|text| {
        let row = table::find_row(&pointer.path, &text, &pointer.id).ok()?;
        Some(if row.goal.is_empty() { section_text(&text, &row.section) } else { row.goal })
    });
    let Some(body) = body else {
        return unmeasured(format!("{} の節を base から読めない", pointer.path));
    };
    let candidates: Vec<&str> = own.iter().filter_map(|item| rs_candidate(item)).collect();
    let mut found: Vec<(&str, String)> = Vec::new();
    let own_pointer = format!("{}#{}", pointer.path, pointer.id);
    for (row, touches, write_set) in rows.iter().filter(|(row, _, write_set)| *row != own_pointer && !write_set.is_empty()) {
        let missing: Vec<&str> = candidates.iter().copied().filter(|file| !covered(write_set, file)).collect();
        if missing.is_empty() {
            continue;
        }
        for item in touches.iter().filter(|item| names_type(item, &body)) {
            let line = format!("{WIDEN}{item}@{row}:{}", missing.join(","));
            if !found.iter().any(|(_, seen)| *seen == line) {
                found.push((item.as_str(), line));
            }
        }
    }
    found.sort_by(|left, right| left.0.cmp(right.0));
    let mut lines: Vec<String> = found.into_iter().map(|(_, line)| line).collect();
    lines.extend(unread.map(|reason| format!("{WIDEN}unmeasured:契約の bead の行を読めない（{reason}）")));
    lines
}

/// write-set の項目が .rs の候補か（接頭辞が無いか `+`・`+` は剥がす・dir と .rs でない file は候補でない）。
fn rs_candidate(item: &str) -> Option<&str> {
    let file = item.strip_prefix('+').unwrap_or(item);
    (file.ends_with(".rs") && !file.starts_with(['-', '~', '=', '+'])).then_some(file)
}

/// touches の項目が型形（末尾の段が大文字始まり）で、その末尾の段が `body` に語の境界で在るか。
fn names_type(item: &str, body: &str) -> bool {
    let name = item.rsplit("::").next().unwrap_or_default();
    if !name.starts_with(char::is_uppercase) {
        return false;
    }
    let is_word = |found: Option<char>| found.is_some_and(|c| c.is_ascii_alphanumeric() || c == '_');
    body.match_indices(name).any(|(at, _)| {
        let before = body.get(..at).and_then(|head| head.chars().next_back());
        let after = body.get(at + name.len()..).and_then(|tail| tail.chars().next());
        !is_word(before) && !is_word(after)
    })
}

/// 判定の対象が揃わなかった周の断りに**末尾の判定行**を積む（judge を撃てないので事実の行は無い）。
///
/// 末尾は **rc に従う**（読めない = broken・撃てない = 断り 1 件）。行の欠陥は「読めない」ではない。
fn tailed(denial: Denial) -> Outcome {
    let mut outcome = denial.outcome;
    let tail = if outcome.rc == RC_BROKEN { format!("{TAIL} broken") } else { format!("{TAIL} refused n=1") };
    outcome.out.push(tail);
    outcome
}

/// judge の結果を 1 行 1 事実に描く。`measured` は置き場が在った（交差を撃った）か。`facts` は base の木で撃った周の欄 `entrance` と、
/// 索引を作れない周の尾 `index=unavailable:<語>`（設計 reverse-index.md §7 (b)）。`tail` は閉包の広がりの行と、行が欄 `done-teeth` を持つか
/// （`done-teeth=present|absent`・`design=` の行の次・設計 contract-source.md §66 行 bx）。`placed` は `--placed` の周の照らし（事実の行は
/// `done-teeth=` の次・断りは judge の断りの後に 1 行で末尾の件数と rc に数える）と、verify の欄の照らしの断りの行（[`verify_gaps`]・
/// acceptance の断りの後に並べ、末尾の件数と rc に数える）。
fn render(judged: &Judged, measured: bool, facts: (Option<String>, Option<String>), tail: (Vec<String>, bool), own: (Option<&Accepted>, &[String])) -> Outcome {
    let (entrance, index) = facts;
    let (widen, teeth) = tail;
    let (placed, gaps) = own;
    let (accepted, refused) = acceptance_lines(placed);
    let mut out: Vec<String> = Vec::new();
    if let Some((design, section)) = &judged.design {
        out.push(format!("design={design} section={section}"));
        out.push(format!("done-teeth={}", if teeth { "present" } else { "absent" }));
    }
    out.extend(accepted);
    if let Some((kind, files)) = judged.write_set {
        out.push(format!("write-set={} files={files}", kind.as_str()));
    }
    for (filter, files) in &judged.teeth {
        out.push(format!("teeth={filter}:{}@{}", files.len(), listed(files)));
    }
    if let Some(found) = &judged.headroom {
        out.extend(found.rooms.iter().map(|(file, room)| format!("headroom={file}:{room}/{}", found.estimate(file))));
    }
    if !measured {
        out.push(UNMEASURED.to_owned());
    }
    if let Some(found) = &judged.overlap {
        out.extend(found.runs.iter().map(|(run, files)| format!("overlap={run}:{}", listed(files))));
    }
    out.extend(entrance);
    out.extend(index);
    out.extend(widen);
    out.extend(judged.denials.iter().map(refuse_line));
    let count = judged.denials.len().saturating_add(usize::from(refused.is_some())).saturating_add(gaps.len());
    out.extend(refused);
    out.extend(gaps.iter().cloned());
    let broken = judged.denials.iter().any(|denial| denial.outcome.rc == RC_BROKEN);
    let (rc, tail) = match (count, broken) {
        (0, _) => (RC_OK, "ok".to_owned()),
        (_, true) => (RC_BROKEN, "broken".to_owned()),
        (count, false) => (RC_REFUSED, format!("refused n={count}")),
    };
    out.push(format!("{TAIL} {tail}"));
    Outcome { out, err: Vec::new(), rc }
}

/// `refuse=<名>:<理由>` の 1 行（stderr の行の `pipe: ` を剥がし、理由の後ろに並ぶ行〔交差の全組・余地の残り〕は
/// ` / ` で同じ行に続ける＝1 断り 1 行）。
fn refuse_line(denial: &Denial) -> String {
    let reasons: Vec<&str> = denial.outcome.err.iter().map(|line| line.strip_prefix("pipe: ").unwrap_or(line)).collect();
    format!("refuse={}:{}", denial.name, reasons.join(" / "))
}

/// file の列を 1 つの語に（空は [`NONE`]）。
fn listed(files: &[String]) -> String {
    if files.is_empty() {
        NONE.to_owned()
    } else {
        files.join(",")
    }
}

#[cfg(test)]
mod tests {
    use super::{acceptance_lines, accepted, literal_gaps, verify_gaps, Accepted};
    use crate::pipe::closure::filter_words;
    use crate::pipe::table::parse_pointer;
    use crate::seat::ledger::Issue;

    /// acceptance だけを持つ台帳の 1 件。
    fn issue(id: &str, acceptance: &str) -> Issue {
        Issue {
            id: id.to_owned(),
            status: "open".to_owned(),
            priority: None,
            labels: Vec::new(),
            acceptance: acceptance.to_owned(),
            deps: Vec::new(),
            kind: String::new(),
            description: String::new(),
            notes: String::new(),
            close_reason: String::new(),
            created_at: None,
            closed_at: None,
            updated_at: None,
            effect: String::new(),
        }
    }

    /// 台帳 `issues` で `bead` を `docs/design/toy.md#a` と照らした結果。
    fn judged(issues: &[Issue], bead: &str) -> Accepted {
        let pointer = parse_pointer("docs/design/toy.md#a").unwrap();
        accepted(issues, bead, &pointer)
    }

    /// 通る見本（行を指す bead）と、通る見本から 1 句だけ外した断る見本。
    #[test]
    fn vaccpt_pointer_of_the_bead_is_matched_against_the_design() {
        let matched = "design = docs/design/toy.md#a";
        assert_eq!(judged(&[issue("s2-a", matched)], "s2-a"), Accepted::Matched);
        assert_eq!(judged(&[issue("s2-a", &format!("先の行\n  {matched}  \n後の行"))], "s2-a"), Accepted::Matched);
        let refused = |issues: &[Issue], bead: &str| matches!(judged(issues, bead), Accepted::Refused(why) if why.contains(&format!("bead {bead} ")));
        assert!(refused(&[issue("s2-a", matched)], "s2-z"), "台帳に無い bead");
        assert!(refused(&[issue("s2-a", "先の行だけ")], "s2-a"), "design = の行が無い");
        assert!(refused(&[issue("s2-a", "design = docs/design/toy.md#b")], "s2-a"), "ほかの行 id");
        assert!(refused(&[issue("s2-a", "design = docs/design/other.md#a")], "s2-a"), "ほかの置き場");
        assert!(refused(&[issue("s2-a", "design =docs/design/toy.md#a")], "s2-a"), "design = の後の空白が無い（dispatch の読み手は読まない）");
        assert!(refused(&[issue("s2-a", matched), issue("s2-b", "design = docs/design/toy.md#b")], "s2-b"), "同じ台帳のほかの bead の行");
    }

    /// 照らしの行: 通る周は事実の行・読めない周は unmeasured の事実の行・断る周は refuse=acceptance-pointer の 1 行と経路の字。
    #[test]
    fn vaccpt_lines_name_the_bead_and_the_route() {
        assert_eq!(acceptance_lines(None), (None, None));
        assert_eq!(acceptance_lines(Some(&Accepted::Matched)), (Some("acceptance=matched".to_owned()), None));
        assert_eq!(acceptance_lines(Some(&Accepted::Unmeasured("ledger"))), (Some("acceptance=unmeasured:ledger".to_owned()), None));
        let (fact, line) = acceptance_lines(Some(&judged(&[issue("s2-a", "先の行だけ")], "s2-a")));
        let line = line.unwrap_or_default();
        assert!(fact.is_none() && line.starts_with("refuse=acceptance-pointer:bead s2-a "), "{line}");
        for needle in ["bdw update s2-a", "--acceptance", "design = docs/design/toy.md#a"] {
            assert!(line.contains(needle), "{needle}: {line}");
        }
        let (_, other) = acceptance_lines(Some(&judged(&[issue("s2-a", "design = docs/design/toy.md#b")], "s2-a")));
        assert!(other.unwrap_or_default().contains("design = docs/design/toy.md#b で --design docs/design/toy.md#a と違う"));
    }

    /// 共通 verify を `cargo run -q -p xtask -- check` の 1 行とした宣言で、行 `contracts/t.toml#a` の verify の列 `lines` を照らした断りの行。
    fn gaps(lines: &[&str]) -> Vec<String> {
        let verify: Vec<String> = lines.iter().map(|line| (*line).to_owned()).collect();
        verify_gaps("contracts/t.toml#a", &verify, &["cargo run -q -p xtask -- check".to_owned()])
    }

    /// 通る見本: filter 語が 1 つの nextest の行（--manifest-path と -p と --test の引数・--no-tests=fail の旗は語に数えない・`--` の後ろの
    /// 1 語も 1 つ）と nextest でない行は断らず、読み手は manifest の path を filter 語の候補に数えない。
    #[test]
    fn vpfgap_one_filter_lines_and_other_lines_pass() {
        let one = "cargo nextest run --manifest-path s/crates/b/Cargo.toml -p b --test e2e --no-tests=fail vx_";
        assert_eq!(filter_words(one), ["vx_"]);
        assert_eq!(filter_words("cargo nextest run -p b --lib --no-tests=fail -- --exact vx_a"), ["vx_a"]);
        assert!(filter_words("git apply --reverse --check p.patch").is_empty());
        let lines = [
            "git apply --reverse --check p.patch",
            "cargo clippy --manifest-path s/crates/b/Cargo.toml -p b --all-targets --no-deps -- -D warnings",
            one,
            "cargo nextest run -p b --lib --no-tests=fail -- --exact vx_a",
            "cargo nextest run -p b --test e2e --no-tests=fail",
            "cargo run -q -p xtask -- check --fast",
        ];
        assert_eq!(gaps(&lines), Vec::<String>::new());
    }

    /// 断る見本（通る見本から 1 句ずつ外す）: filter 語が 2 つの行（`--` の前に 2 つ・前と後ろに 1 つずつ・後ろに 2 つ）は verify-filters の
    /// 1 行で、行の pointer と verify の字と語の数と語の列と最後の語と直し方を名指す。共通 verify と同じ行（空白の並びの違いも同じ）は
    /// verify-common の 1 行。断りは verify の行の順に並ぶ。
    #[test]
    fn vpfgap_two_filter_lines_and_the_common_line_are_refused_one_line_each() {
        for (line, words) in [
            ("cargo nextest run -p b --test e2e --no-tests=fail vx_ vy_", "vx_ vy_"),
            ("cargo nextest run -p b --no-tests=fail vx_ -- --exact vy_a", "vx_ vy_a"),
            ("cargo nextest run -p b --no-tests=fail -- --exact vx_a vy_b", "vx_a vy_b"),
        ] {
            let found = gaps(&[line]);
            assert_eq!(found.len(), 1, "{line}: {found:?}");
            let last = words.rsplit(' ').next().unwrap_or_default();
            let want = format!("refuse=verify-filters:行 contracts/t.toml#a の verify {line:?} は nextest の filter の語を 2 つ持つ（{words}）— 器は最後の語 {last} だけ");
            assert!(found.first().is_some_and(|got| got.starts_with(&want) && got.ends_with("1 行に filter の語を 1 つずつ割る")), "{found:?}");
        }
        for line in ["cargo run -q -p xtask -- check", "cargo  run -q -p xtask --  check"] {
            let want = format!("refuse=verify-common:行 contracts/t.toml#a の verify {line:?} は宣言の common-verify の行と同じ — gate は共通 verify を verify の行の前に撃つので同じ木で 2 度撃つ。行の verify から外す");
            assert_eq!(gaps(&[line]), [want]);
        }
        let both = gaps(&["cargo run -q -p xtask -- check", "cargo nextest run -p b --no-tests=fail vx_ vy_"]);
        let names: Vec<&str> = both.iter().map(|line| line.split(':').next().unwrap_or_default()).collect();
        assert_eq!(names, ["refuse=verify-common", "refuse=verify-filters"], "verify の行の順");
    }

    /// 材料を字 `docs/design/toy.md#a §1` と `本文。` と `型` の 3 行・行の pointer を `docs/design/toy.md#a` として、done `done` の印の照らしの断りの行。
    fn literal(done: &str) -> Vec<String> {
        literal_gaps("docs/design/toy.md#a", done, "docs/design/toy.md#a §1\n本文。\n型\n")
    }

    /// 通る見本: 材料に在る字を囲んだ項だけの番号つきの done・印の無い done・番号の無い done の印の字が材料に在る周は、どれも空の列。
    #[test]
    fn vpflit_marked_literals_in_the_material_pass() {
        assert_eq!(literal("(1) 字 『本文。』 を出す (2) 『型』 と 『本文。』 を出す"), Vec::<String>::new());
        assert_eq!(literal("a が通る"), Vec::<String>::new());
        assert_eq!(literal("『本文。』 を出す"), Vec::<String>::new());
    }

    /// 断る見本（通る見本から 1 句ずつ外す）: 材料に無い字・空の印・閉じない印は項の番号と字を名指す 1 行ずつで、項の順に並ぶ。番号の無い done は項 1。
    #[test]
    fn vpflit_missing_unclosed_and_empty_marks_are_named() {
        let head = "refuse=section-literal:行 docs/design/toy.md#a の done の項";
        let want = [
            format!("{head} 2 の字 『無い字』 が審査の材料の節に無い — 節の本文に同じ字を書くか印を外す"),
            format!("{head} 3 の印 『』 が空 — 印の中に節の字を書く"),
            format!("{head} 4 の印 『 が閉じない — 』 で閉じる"),
        ];
        assert_eq!(literal("(1) 『本文。』 (2) 『無い字』 と 『本文。』 (3) 『』 (4) 『閉じない"), want);
        let one = format!("{head} 1 の字 『無い字』 が審査の材料の節に無い — 節の本文に同じ字を書くか印を外す");
        assert_eq!(literal("『無い字』 を出す"), [one]);
    }
}
