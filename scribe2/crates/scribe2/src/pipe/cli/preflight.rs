//! `pipe preflight` — 受付と同じ判定を run を作らず撃ち、契約の実態突合を planner が edit time に測る口（契約表の行 u・
//! 設計 contract-source.md §21・SRS FR48・C16「逸脱は edit time に止める」）。
//!
//! 引数は `intake` と同じ（`--design <doc>#<id> --bead B --repo R [--state-dir S]`）。判定は受付の [`super::intake::judge`] の
//! **同じ 1 本**（C2・2 本目を作らない）で、断りを最初の 1 件で止めず**全部**（判定関数 1 本につき高々 1 件）並べる。
//! run dir・写し・event は一切書かず、宣言の写しは読むだけ・置き場は交差の読みにだけ使う。
//!
//! stdout は **1 行 1 事実**: `design=<doc>#<id> section=<n>` / `done-teeth=<present|absent>`（行が欄 done-teeth を持つか）/ `write-set=<declared|derived> files=<n>` /
//! `teeth=<filter>:<本数>@<file,…>`（verify の nextest 行ごと）/ `headroom=<file>:<余地>/<file の見込み>`（余地の小さい順・
//! 見込みは行の growth に在ればその値・無ければ size の見積・設計 contract-source.md §46）/
//! `overlap=<live run>:<file,…>`（突き合わせた live な run ごと・交差 0 は `-`・置き場が無ければ `overlap=unmeasured`）/
//! `entrance=green-on-base:<本数>/<行数>[ unmeasurable:<本数>]`（宣言が `detect` / `deny` を名乗る周だけ・契約の検証行を base
//! の木で撃った結果・木は置き場の直下の一時の worktree で撃ち終えたら畳む・設計 pipeline.md §56）/
//! `widen=<項目>@<doc>#<行 id>:<file,…>`（自分の行の § の本文が語として名指す型形の項目をほかの行が touches に持ち、その行の write-set が
//! 自分の write-set の .rs の候補を覆わない組・項目の辞書順・読めない周は `widen=unmeasured:<理由>` の 1 行・rc と判定は変えない）/
//! `index=unavailable:<語>`（索引を作れない周〔状態が failed〕で touches に型の項目を持つ行だけ・字面の閉包に縮退して通す・rc と判定は
//! 変えない）/
//! `acceptance=<matched|unmeasured:ledger>`（`--placed` を渡した周だけ・`--bead` の bead の acceptance の `design = ` の行を dispatch の
//! 列と同じ読み手で読み `--design` と照らす・`done-teeth=` の行の次・判断の記録 ADR-44 の決定 (3)）/
//! `refuse=<名>:<理由>`（judge の断り・全部・名は [`crate::pipe::refuse::Refuse::as_str`]・`--placed` の周の照らしの断りは judge の断りの後に
//! preflight だけの名 `acceptance-pointer` の 1 行・verify の欄の照らしの断りはその後に preflight だけの名 `verify-filters`〔filter 語の候補が 2 つ
//! 以上の nextest の行〕と `verify-common`〔宣言の common-verify と同じ行〕の 1 行ずつ）/ 末尾に
//! `preflight: <ok|refused n=<件数>|broken>`。rc = 0（断り 0）/ 1（断り ≥ 1）/ 2（読めない = `RC_BROKEN` の周）。
//! `--state-dir` が無く git 設定からも解けない周は `overlap=unmeasured` を出し、rc は他の断りで決める（測れないを 0 に
//! 潰さない・C10・`intake` は従来どおり置き場が無い旨で断る）。

use super::base_run::BaseRun;
use super::intake::{ceiling_of, early, generated, generated_from, judge, read_args, Denial, Judged, Material, Materials};
use super::{flag, need, present, refused, repo_flag, state_dir_of, REPO_FLAG};
use crate::cli_outcome::{Outcome, RC_BROKEN, RC_OK, RC_REFUSED};
use crate::pipe::closure::filter_words;
use crate::pipe::dispatch::pointer_of;
use crate::pipe::refuse::{covered, Refuse};
use crate::pipe::review::section_text;
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

/// preflight の本体。
fn checked(args: &[String], manifest: &Manifest) -> Outcome {
    let read = match flag(args, CONTRACT) {
        Ok(Some(given)) => read_contract_args(args, given).map(|(pointer, bead, repo, text)| (pointer, bead, repo, Some(text))),
        Ok(None) => read_args(args).map(|(pointer, bead, repo)| (pointer, bead, repo, None)).map_err(|denial| denial.outcome),
        Err(reason) => Err(refused(reason)),
    };
    let (pointer, bead, repo, file) = match read {
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
    let state_dir = state_dir_of(args).ok();
    let materials = match Materials::read(&repo, &ceiling.borrow(), bd) {
        Ok(found) => match state_dir.as_deref() {
            Some(dir) => found.indexed(dir, &repo, &sha),
            None => found,
        },
        Err(denial) => return tailed(denial),
    };
    let built = match file.as_deref() {
        Some(text) => generated_from(&repo, &pointer, text, &materials),
        None => generated(&repo, &pointer, &materials),
    };
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
    let doc = file.or_else(|| show_head(&repo, &pointer.path));
    let widen = widen_lines(&repo, &sha, &pointer, doc, &contract.write_set);
    let placed = present(args, PLACED).then(|| acceptance_of(bd, &repo, manifest, &bead, &pointer));
    let common = early.frozen.as_ref().map_or(&[][..], |(found, _)| found.common_verify());
    let gaps = verify_gaps(&format!("{}#{}", pointer.path, pointer.id), &contract.verify, common);
    render(&judged, state_dir.is_some(), (entrance, index), (widen, teeth), (placed.as_ref(), &gaps))
}

/// `widen=<項目>@<doc>#<行 id>:<file,…>`（設計 reverse-index.md の閉包の広がりの予想）: 自分の行の § の本文が語として名指す型形の項目を
/// touches に持つほかの行で、write-set が自分の write-set の .rs の候補〔接頭辞が無いか `+`・`+` は剥がす〕を覆わない組。HEAD の木の
/// 契約表を読めない周は `widen=unmeasured:<理由>` の 1 行（理由は「ほかの行の touches」節と同じ字）。`doc` は自分の行を持つ doc の字
/// （`--design` の周は base の読み・`--contract` の周は file の字）で、本文は行が goal を持てば goal・無ければ節の本文。
fn widen_lines(repo: &Path, sha: &str, pointer: &table::Pointer, doc: Option<String>, own: &[String]) -> Vec<String> {
    let unmeasured = |reason: String| vec![format!("{WIDEN}unmeasured:{reason}")];
    let rows = match table_rows(repo, sha) {
        Ok(found) => found,
        Err(reason) => return unmeasured(reason),
    };
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
    found.into_iter().map(|(_, line)| line).collect()
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
    use super::{acceptance_lines, accepted, verify_gaps, Accepted};
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
}
