//! tsuzuri-surface の歯の群 teeth3 の共通の手（群の 2 つ以上の module が字で同じ写しを持っていた helper を 1 つだけ置く・畳みの道具 fold2.py が寄せる・判断の記録 ADR-63 の決定 (8) の (c)）。
//! 写しを持っていた module は use crate::common::… で呼ぶ。字が 1 文字でも違う物はここへ寄せない。
#![cfg(test)]

use std::path::PathBuf;
use tsuzuri_contract::account::AccountDoc;
use tsuzuri_contract::board::Reading;
use tsuzuri_contract::stats::LedgerStats;
use tsuzuri_surface::project::Body;
use tsuzuri_surface::widgets::hover::Card;

/// 着地済みの行とこの文書の行の verify の filter の語。
pub(crate) const FILTERS: [&str; 49] = [
    "accept_",
    "account_",
    "acctcore_",
    "acctdoc_",
    "acctframe_",
    "accthb_",
    "accthome_",
    "acctled_",
    "acctlook_",
    "acctpcore_",
    "acctproj_",
    "acctsess_",
    "acctwin_",
    "acctwire_",
    "askcard_",
    "batchpanel_",
    "board_min_",
    "contract_form_",
    "frame_",
    "gapspage_",
    "gquestion_",
    "graph_",
    "gview_",
    "hook_",
    "ledgerblock_",
    "mapgraph_",
    "mapview_",
    "nextstep_",
    "nodepage_",
    "parts_",
    "pipe_",
    "project_",
    "question_",
    "seatblock_",
    "seatcard_",
    "server_",
    "skeleton_",
    "stage_",
    "stats_",
    "steady_",
    "topbar_",
    "tz_",
    "hbproc_",
    "hbconf_",
    "hbroute_",
    "hbpost_",
    "hsblock_",
    "hspage_",
    "hsderive_",
];

/// fixture の proj-a の台帳（Known）の複製。
pub(crate) fn a_stats(doc: &AccountDoc) -> LedgerStats {
    match &doc.projects[0].ledger {
        Reading::Known(s) => s.clone(),
        Reading::Unknown => panic!("fixture の proj-a の台帳が Known でない"),
    }
}

/// `text` で最初に出る `word` の位置。
pub(crate) fn at(text: &str, word: &str) -> usize {
    text.find(word)
        .unwrap_or_else(|| panic!("{word} が無い: {text}"))
}

/// card の 4 行と詳しくを比べる。
pub(crate) fn check(card: &Card, (title, kind): (&str, &str), value: &str, src: &str, more: &[&str]) {
    assert_eq!(card.title, title);
    assert_eq!(card.kind, kind, "{title} の種類");
    assert_eq!(card.value, value, "{title} の値");
    assert_eq!(card.src, src, "{title} の出所");
    assert_eq!(card.more, more, "{title} の詳しく");
}

pub(crate) fn crate_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// 字の「mod dom」より後（DOM の部分）。
pub(crate) fn dom_of(rel: &str) -> String {
    let text = read(rel);
    let at = text
        .find("mod dom")
        .unwrap_or_else(|| panic!("{rel} に mod dom が無い"));
    text[at..].to_string()
}

pub(crate) fn filled<T: std::fmt::Debug>(body: Body<T>) -> T {
    match body {
        Body::Filled(v) => v,
        other => panic!("中身ありでない: {other:?}"),
    }
}

/// DOM の部分の fn の本体（宣言の字から、次の行頭 4 空白の fn か pub fn の宣言の前まで・無ければ終わりまで）。
pub(crate) fn fn_body(dom: &str, name: &str) -> String {
    let start = dom
        .find(&format!("fn {name}("))
        .unwrap_or_else(|| panic!("DOM の部分に fn {name} が無い"));
    let rest = &dom[start..];
    let first = rest.find('\n').map_or(rest.len(), |i| i + 1);
    let mut end = rest.len();
    let mut at = first;
    for line in rest[first..].split_inclusive('\n') {
        if line.starts_with("    fn ") || line.starts_with("    pub fn ") {
            end = at;
            break;
        }
        at += line.len();
    }
    rest[..end].to_string()
}

/// 関数の本文（`fn name(` から次の行頭の `fn `・`pub fn `・`async fn `・`pub async fn ` まで）。
pub(crate) fn function<'a>(text: &'a str, name: &str) -> &'a str {
    let start = text
        .find(&format!("fn {name}("))
        .unwrap_or_else(|| panic!("fn {name} が在る"));
    let rest = &text[start..];
    let end = ["\nfn ", "\npub fn ", "\nasync fn ", "\npub async fn "]
        .iter()
        .filter_map(|m| rest[1..].find(m).map(|i| i + 1))
        .min()
        .unwrap_or(rest.len());
    &rest[..end]
}

pub(crate) fn read(rel: &str) -> String {
    std::fs::read_to_string(crate_dir().join(rel)).unwrap_or_else(|e| panic!("{rel} を読む: {e}"))
}

pub(crate) fn squeeze(text: &str) -> String {
    text.chars().filter(|c| !c.is_whitespace()).collect()
}

/// 本体の中の字 head の所から最初の > までの tag。
pub(crate) fn tag(body: &str, head: &str) -> String {
    let at = body
        .find(head)
        .unwrap_or_else(|| panic!("本体に字 {head} が無い"));
    let rest = &body[at..];
    rest[..rest.find('>').unwrap_or(rest.len())].to_string()
}
